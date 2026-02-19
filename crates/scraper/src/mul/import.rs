use std::{
    collections::HashMap,
    path::PathBuf,
};

use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::{info, warn};

use crate::parse::to_slug;

use super::{
    detail,
    mappings,
    matcher::{self, extract_clan_name, Matcher, UnmatchedUnit},
    quicklist,
};

/// Run the mul-import subcommand: import MUL data from local files into the database.
pub async fn run(
    data_dir: PathBuf,
    database_url: &str,
    pool_size: u32,
    skip_availability: bool,
    force: bool,
    overrides_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(database_url)
        .await
        .context("connecting to database")?;

    // ── Step 1: Load local state ──────────────────────────────────────────

    // Load all DB units into HashMaps for matching
    let (units_by_slug, units_by_name) = load_db_units(&pool).await?;
    info!(by_slug = units_by_slug.len(), "loaded DB units for matching");

    // Load era and faction maps from DB
    let era_slug_to_id = load_era_map(&pool).await?;
    let faction_name_to_id = load_faction_map(&pool).await?;
    info!(
        eras = era_slug_to_id.len(),
        factions = faction_name_to_id.len(),
        "loaded era/faction maps"
    );

    // Load overrides if provided
    let overrides = match overrides_path {
        Some(ref path) => {
            let o = matcher::load_overrides(path)?;
            info!(count = o.len(), "loaded override mappings");
            o
        }
        None => HashMap::new(),
    };

    let matcher = Matcher::new(overrides, units_by_slug, units_by_name);

    // ── Step 2: Import QuickList data ─────────────────────────────────────

    // Find all quicklist-*.json files in data_dir
    let mut mul_units: Vec<quicklist::MulUnit> = Vec::new();
    for entry in std::fs::read_dir(&data_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("quicklist-") && name.ends_with(".json") {
            let json = std::fs::read_to_string(entry.path())?;
            let units = quicklist::parse_quicklist(&json)?;
            info!(file = %name, count = units.len(), "loaded QuickList");
            mul_units.extend(units);
        }
    }

    // Deduplicate by MUL ID (keep first occurrence)
    let mut seen_ids = std::collections::HashSet::new();
    mul_units.retain(|u| seen_ids.insert(u.id));

    info!(total = mul_units.len(), "total unique MUL units to process");

    let mut matched_count = 0usize;
    let mut unmatched: Vec<UnmatchedUnit> = Vec::new();
    let mut bv_changed = 0usize;
    let mut cost_changed = 0usize;
    let mut role_assigned = 0usize;
    let mut intro_year_changed = 0usize;

    // Map MUL ID → DB id for availability step
    let mut mul_id_to_db_id: HashMap<u32, i32> = HashMap::new();

    for unit in &mul_units {
        match matcher.match_unit(unit.id, &unit.name, unit.tonnage) {
            Ok(m) => {
                matched_count += 1;
                mul_id_to_db_id.insert(unit.id, m.db_id);

                let bv = unit.bv();
                let cost = unit.cost_value();
                let intro_year = unit.intro_year();
                let role = unit.role_name().map(|s| s.to_string());
                let clan_name = extract_clan_name(&unit.name);

                let changes = update_mul_fields(
                    &pool,
                    m.db_id,
                    unit.id as i32,
                    bv,
                    cost,
                    intro_year,
                    role.as_deref(),
                    clan_name.as_deref(),
                )
                .await?;

                bv_changed += changes.bv_changed as usize;
                cost_changed += changes.cost_changed as usize;
                intro_year_changed += changes.intro_year_changed as usize;
                role_assigned += changes.role_assigned as usize;
            }
            Err(um) => {
                unmatched.push(um);
            }
        }
    }

    info!(
        matched = matched_count,
        unmatched = unmatched.len(),
        bv_changed,
        cost_changed,
        role_assigned,
        intro_year_changed,
        "QuickList import complete"
    );

    // Write unmatched CSV
    if !unmatched.is_empty() {
        let csv_path = data_dir.join("unmatched_mul_units.csv");
        matcher::write_unmatched_csv(&csv_path, &unmatched)?;
        info!(path = %csv_path.display(), count = unmatched.len(), "wrote unmatched units CSV");
    }

    // ── Step 3: Import availability ───────────────────────────────────────

    if skip_availability {
        info!("skipping availability import (--skip-availability)");
    } else {
        let era_map = mappings::era_mappings();
        let faction_map = mappings::faction_mappings();
        let details_dir = data_dir.join("details");

        let mut availability_units = 0usize;
        let mut availability_rows = 0usize;
        let mut new_factions = 0usize;

        // We need mutable access to the faction name→id map for auto-creating factions
        let mut faction_name_to_id = faction_name_to_id;

        for (&mul_id, &db_id) in &mul_id_to_db_id {
            let detail_path = details_dir.join(format!("{mul_id}.html"));
            if !detail_path.exists() {
                continue;
            }

            // Skip if unit already has availability rows (unless --force)
            if !force {
                let existing: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM unit_availability WHERE unit_id = $1",
                )
                .bind(db_id)
                .fetch_one(&pool)
                .await?;

                if existing > 0 {
                    continue;
                }
            }

            let html = std::fs::read_to_string(&detail_path)?;
            let records = detail::parse_availability(&html);

            if records.is_empty() {
                continue;
            }

            // Resolve era/faction IDs
            let mut rows_to_insert: Vec<(i32, i32)> = Vec::new();

            for rec in &records {
                // Resolve era
                let era_slug = match era_map.get(rec.era_name.as_str()) {
                    Some(slug) => *slug,
                    None => {
                        warn!(era = %rec.era_name, mul_id, "unmapped era — skipping record");
                        continue;
                    }
                };
                let era_id = match era_slug_to_id.get(era_slug) {
                    Some(&id) => id,
                    None => {
                        warn!(era_slug, "era not found in DB — skipping");
                        continue;
                    }
                };

                // Resolve faction
                let faction_id = if let Some(&id) = faction_name_to_id.get(&rec.faction_name) {
                    id
                } else if let Some(slug) = faction_map.get(rec.faction_name.as_str()) {
                    // Faction exists by slug mapping but wasn't in our name→id map
                    if let Some(&id) = faction_name_to_id.get(*slug) {
                        // Cache the name→id mapping for future lookups
                        faction_name_to_id.insert(rec.faction_name.clone(), id);
                        id
                    } else {
                        warn!(faction = %rec.faction_name, slug, "mapped faction not in DB — skipping");
                        continue;
                    }
                } else {
                    // Auto-create new faction
                    let slug = to_slug(&rec.faction_name);
                    let is_clan = rec.faction_name.starts_with("Clan ");
                    let faction_type = mappings::infer_faction_type(&rec.faction_name);

                    let id = ensure_faction(
                        &pool,
                        &slug,
                        &rec.faction_name,
                        faction_type,
                        is_clan,
                    )
                    .await?;

                    info!(
                        name = %rec.faction_name,
                        slug = %slug,
                        faction_type,
                        "created new faction"
                    );
                    new_factions += 1;

                    faction_name_to_id.insert(rec.faction_name.clone(), id);
                    id
                };

                rows_to_insert.push((faction_id, era_id));
            }

            if !rows_to_insert.is_empty() {
                // Deduplicate rows
                rows_to_insert.sort_unstable();
                rows_to_insert.dedup();

                let inserted = replace_availability(&pool, db_id, &rows_to_insert).await?;
                availability_units += 1;
                availability_rows += inserted;
            }
        }

        info!(
            units = availability_units,
            rows = availability_rows,
            new_factions,
            "availability import complete"
        );
    }

    // ── Summary ───────────────────────────────────────────────────────────

    info!(
        matched = matched_count,
        total_mul = mul_units.len(),
        bv_changed,
        cost_changed,
        role_assigned,
        intro_year_changed,
        unmatched = unmatched.len(),
        "MUL import finished"
    );

    Ok(())
}

// ── DB helpers ───────────────────────────────────────────────────────────────

/// Load all units from DB into (slug → id) and (lowercase_full_name → (slug, id)) maps.
async fn load_db_units(
    pool: &PgPool,
) -> anyhow::Result<(HashMap<String, i32>, HashMap<String, (String, i32)>)> {
    let rows = sqlx::query("SELECT id, slug, full_name FROM units")
        .fetch_all(pool)
        .await?;

    let mut by_slug = HashMap::new();
    let mut by_name = HashMap::new();

    for row in rows {
        let id: i32 = row.try_get("id")?;
        let slug: String = row.try_get("slug")?;
        let full_name: String = row.try_get("full_name")?;

        by_slug.insert(slug.clone(), id);
        by_name.insert(full_name.to_lowercase(), (slug, id));
    }

    Ok((by_slug, by_name))
}

/// Load era slug → id map from DB.
async fn load_era_map(pool: &PgPool) -> anyhow::Result<HashMap<String, i32>> {
    let rows = sqlx::query("SELECT id, slug FROM eras")
        .fetch_all(pool)
        .await?;

    let mut map = HashMap::new();
    for row in rows {
        let id: i32 = row.try_get("id")?;
        let slug: String = row.try_get("slug")?;
        map.insert(slug, id);
    }
    Ok(map)
}

/// Load faction name → id map from DB (using both name and slug for lookups).
async fn load_faction_map(pool: &PgPool) -> anyhow::Result<HashMap<String, i32>> {
    let rows = sqlx::query("SELECT id, slug, name FROM factions")
        .fetch_all(pool)
        .await?;

    let mut map = HashMap::new();
    for row in rows {
        let id: i32 = row.try_get("id")?;
        let slug: String = row.try_get("slug")?;
        let name: String = row.try_get("name")?;
        map.insert(name, id);
        map.insert(slug, id);
    }
    Ok(map)
}

struct FieldChanges {
    bv_changed: bool,
    cost_changed: bool,
    intro_year_changed: bool,
    role_assigned: bool,
}

/// Update MUL-sourced fields on a unit row, using COALESCE to preserve existing values.
async fn update_mul_fields(
    pool: &PgPool,
    db_id: i32,
    mul_id: i32,
    bv: Option<i32>,
    cost: Option<i64>,
    intro_year: Option<i32>,
    role: Option<&str>,
    clan_name: Option<&str>,
) -> anyhow::Result<FieldChanges> {
    // Fetch current values to count actual changes
    let current = sqlx::query(
        "SELECT bv, cost, intro_year, role FROM units WHERE id = $1",
    )
    .bind(db_id)
    .fetch_one(pool)
    .await?;

    let cur_bv: Option<i32> = current.try_get("bv")?;
    let cur_cost: Option<i64> = current.try_get("cost")?;
    let cur_intro: Option<i32> = current.try_get("intro_year")?;
    let cur_role: Option<String> = current.try_get("role")?;

    let bv_changed = bv.is_some() && bv != cur_bv;
    let cost_changed = cost.is_some() && cost != cur_cost;
    let intro_year_changed = intro_year.is_some() && intro_year != cur_intro;
    let role_assigned = role.is_some() && role != cur_role.as_deref();

    sqlx::query(
        r#"UPDATE units SET
            mul_id = $1,
            bv = COALESCE($2, bv),
            cost = COALESCE($3, cost),
            intro_year = COALESCE($4, intro_year),
            role = COALESCE($5, role),
            bv_source = CASE WHEN $2 IS NOT NULL THEN 'mul' ELSE bv_source END,
            intro_year_source = CASE WHEN $4 IS NOT NULL THEN 'mul' ELSE intro_year_source END,
            clan_name = COALESCE($6, clan_name),
            last_mul_import_at = now()
        WHERE id = $7"#,
    )
    .bind(mul_id)
    .bind(bv)
    .bind(cost)
    .bind(intro_year)
    .bind(role)
    .bind(clan_name)
    .bind(db_id)
    .execute(pool)
    .await
    .with_context(|| format!("update_mul_fields for unit {db_id}"))?;

    Ok(FieldChanges {
        bv_changed,
        cost_changed,
        intro_year_changed,
        role_assigned,
    })
}

/// Insert or get a faction by slug, return its id.
async fn ensure_faction(
    pool: &PgPool,
    slug: &str,
    name: &str,
    faction_type: &str,
    is_clan: bool,
) -> anyhow::Result<i32> {
    let row = sqlx::query(
        r#"INSERT INTO factions (slug, name, faction_type, is_clan)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (slug) DO NOTHING
           RETURNING id"#,
    )
    .bind(slug)
    .bind(name)
    .bind(faction_type)
    .bind(is_clan)
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        return Ok(r.try_get("id")?);
    }

    let r = sqlx::query("SELECT id FROM factions WHERE slug = $1")
        .bind(slug)
        .fetch_one(pool)
        .await?;
    Ok(r.try_get("id")?)
}

/// Replace all availability rows for a unit within a transaction.
async fn replace_availability(
    pool: &PgPool,
    unit_id: i32,
    rows: &[(i32, i32)], // (faction_id, era_id)
) -> anyhow::Result<usize> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM unit_availability WHERE unit_id = $1")
        .bind(unit_id)
        .execute(&mut *tx)
        .await?;

    let mut count = 0usize;
    for &(faction_id, era_id) in rows {
        sqlx::query(
            r#"INSERT INTO unit_availability (unit_id, faction_id, era_id)
               VALUES ($1, $2, $3)
               ON CONFLICT (unit_id, faction_id, era_id) DO NOTHING"#,
        )
        .bind(unit_id)
        .bind(faction_id)
        .bind(era_id)
        .execute(&mut *tx)
        .await?;
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}
