/// Database upsert operations for scraped unit data.
///
/// Uses runtime sqlx::query (no `!` macro) to avoid compile-time type resolution
/// issues with custom PostgreSQL enum types.
use std::collections::HashMap;

use anyhow::Context;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

use crate::parse::{ParsedUnit, TechBase, RulesLevel};

// ── helpers ───────────────────────────────────────────────────────────────────

fn tech_base_str(tb: TechBase) -> &'static str {
    match tb {
        TechBase::InnerSphere => "inner_sphere",
        TechBase::Clan       => "clan",
        TechBase::Mixed      => "mixed",
        TechBase::Primitive  => "primitive",
    }
}

fn rules_level_str(rl: RulesLevel) -> &'static str {
    match rl {
        RulesLevel::Introductory => "introductory",
        RulesLevel::Standard     => "standard",
        RulesLevel::Advanced     => "advanced",
        RulesLevel::Experimental => "experimental",
        RulesLevel::Unofficial   => "unofficial",
    }
}

fn to_decimal(f: f64) -> Decimal {
    Decimal::try_from(f).unwrap_or(Decimal::ZERO)
}

// ── equipment ─────────────────────────────────────────────────────────────────

/// Upsert a piece of equipment by slug and return its id.
pub async fn upsert_equipment(
    pool: &PgPool,
    slug: &str,
    name: &str,
    category: &str,
    tech_base: TechBase,
    rules_level: RulesLevel,
) -> anyhow::Result<i32> {
    let row = sqlx::query(
        r#"
        INSERT INTO equipment (slug, name, category, tech_base, rules_level)
        VALUES ($1, $2, $3::equipment_category_enum, $4::tech_base_enum, $5::rules_level_enum)
        ON CONFLICT (slug) DO UPDATE
            SET name        = EXCLUDED.name,
                category    = EXCLUDED.category,
                tech_base   = EXCLUDED.tech_base,
                rules_level = EXCLUDED.rules_level
        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(name)
    .bind(category)
    .bind(tech_base_str(tech_base))
    .bind(rules_level_str(rules_level))
    .fetch_one(pool)
    .await
    .with_context(|| format!("upsert_equipment: {slug}"))?;

    Ok(row.try_get("id")?)
}

// ── chassis ───────────────────────────────────────────────────────────────────

/// Upsert a chassis row and return its id.
///
/// The chassis slug includes the unit type to prevent collisions between
/// different unit types sharing a name (e.g. "Vulcan" mech vs "Vulcan" fighter).
pub async fn upsert_chassis(pool: &PgPool, unit: &ParsedUnit) -> anyhow::Result<i32> {
    let slug = format!(
        "{}-{}",
        crate::parse::to_slug(&unit.chassis),
        unit.unit_type.as_str()
    );
    let tonnage = to_decimal(unit.tonnage);

    let row = sqlx::query(
        r#"
        INSERT INTO unit_chassis (slug, name, unit_type, tech_base, tonnage, intro_year, description)
        VALUES ($1, $2, $3, $4::tech_base_enum, $5, $6, $7)
        ON CONFLICT (slug) DO UPDATE
            SET name        = EXCLUDED.name,
                unit_type   = EXCLUDED.unit_type,
                tech_base   = EXCLUDED.tech_base,
                tonnage     = EXCLUDED.tonnage,
                intro_year  = EXCLUDED.intro_year,
                description = EXCLUDED.description
        RETURNING id
        "#,
    )
    .bind(&slug)
    .bind(&unit.chassis)
    .bind(unit.unit_type.as_str())
    .bind(tech_base_str(unit.tech_base))
    .bind(tonnage)
    .bind(unit.intro_year)
    .bind(unit.description.as_deref())
    .fetch_one(pool)
    .await
    .with_context(|| format!("upsert_chassis: {slug}"))?;

    Ok(row.try_get("id")?)
}

// ── unit ──────────────────────────────────────────────────────────────────────

/// Upsert a unit variant row and return its id.
pub async fn upsert_unit(
    pool: &PgPool,
    unit: &ParsedUnit,
    chassis_id: i32,
) -> anyhow::Result<i32> {
    let full_name = if unit.model.is_empty() {
        unit.chassis.clone()
    } else {
        format!("{} {}", unit.chassis, unit.model)
    };
    let slug    = crate::parse::to_slug(&full_name);
    let tonnage = to_decimal(unit.tonnage);

    let row = sqlx::query(
        r#"
        INSERT INTO units (
            slug, chassis_id, variant, full_name,
            tech_base, rules_level,
            tonnage, intro_year, source_book, description
        )
        VALUES ($1, $2, $3, $4, $5::tech_base_enum, $6::rules_level_enum, $7, $8, $9, $10)
        ON CONFLICT (slug) DO UPDATE
            SET chassis_id  = EXCLUDED.chassis_id,
                variant     = EXCLUDED.variant,
                full_name   = EXCLUDED.full_name,
                tech_base   = EXCLUDED.tech_base,
                rules_level = EXCLUDED.rules_level,
                tonnage     = EXCLUDED.tonnage,
                intro_year  = EXCLUDED.intro_year,
                source_book = EXCLUDED.source_book,
                description = EXCLUDED.description
        RETURNING id
        "#,
    )
    .bind(&slug)
    .bind(chassis_id)
    .bind(&unit.model)          // variant column
    .bind(&full_name)
    .bind(tech_base_str(unit.tech_base))
    .bind(rules_level_str(unit.rules_level))
    .bind(tonnage)
    .bind(unit.intro_year)
    .bind(unit.source.as_deref()) // source_book column
    .bind(unit.description.as_deref())
    .fetch_one(pool)
    .await
    .with_context(|| format!("upsert_unit: {slug}"))?;

    Ok(row.try_get("id")?)
}

// ── locations ─────────────────────────────────────────────────────────────────

/// Delete existing location rows then bulk-insert fresh ones.
pub async fn replace_locations(
    pool: &PgPool,
    unit_id: i32,
    unit: &ParsedUnit,
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM unit_locations WHERE unit_id = $1")
        .bind(unit_id)
        .execute(pool)
        .await?;

    for loc in &unit.locations {
        sqlx::query(
            r#"
            INSERT INTO unit_locations (unit_id, location, armor_points, rear_armor, structure_points)
            VALUES ($1, $2::location_name_enum, $3, $4, $5)
            "#,
        )
        .bind(unit_id)
        .bind(loc.location)
        .bind(loc.armor)
        .bind(loc.rear_armor)
        .bind(loc.structure)
        .execute(pool)
        .await
        .with_context(|| format!("insert location {} for unit {unit_id}", loc.location))?;
    }
    Ok(())
}

// ── loadout ───────────────────────────────────────────────────────────────────

/// Delete existing loadout rows then bulk-insert fresh ones.
/// `equipment_cache` maps equipment slug → id (populated/extended in-place).
pub async fn replace_loadout(
    pool: &PgPool,
    unit_id: i32,
    unit: &ParsedUnit,
    equipment_cache: &mut HashMap<String, i32>,
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM unit_loadout WHERE unit_id = $1")
        .bind(unit_id)
        .execute(pool)
        .await?;

    for entry in &unit.loadout {
        let eq_slug = crate::parse::to_slug(&entry.equipment);
        let eq_id = if let Some(&id) = equipment_cache.get(&eq_slug) {
            id
        } else {
            let category    = crate::parse::categorize_equipment(&entry.equipment);
            let tech_base_s = crate::parse::equipment_tech_base(&entry.equipment);
            let tb = match tech_base_s {
                "clan"      => TechBase::Clan,
                "mixed"     => TechBase::Mixed,
                "primitive" => TechBase::Primitive,
                _           => TechBase::InnerSphere,
            };
            let id = upsert_equipment(pool, &eq_slug, &entry.equipment, category, tb, unit.rules_level).await?;
            equipment_cache.insert(eq_slug.clone(), id);
            id
        };

        sqlx::query(
            r#"
            INSERT INTO unit_loadout (unit_id, equipment_id, location, quantity, is_rear_facing)
            VALUES ($1, $2, $3::location_name_enum, $4, $5)
            "#,
        )
        .bind(unit_id)
        .bind(eq_id)
        .bind(entry.location)   // Option<&'static str> → cast to enum in SQL
        .bind(entry.quantity)
        .bind(entry.is_rear)    // is_rear_facing column
        .execute(pool)
        .await
        .with_context(|| format!("insert loadout entry {} for unit {unit_id}", entry.equipment))?;
    }
    Ok(())
}

// ── observed locations ────────────────────────────────────────────────────────

/// Refresh observed_locations on equipment from loadout data.
pub async fn refresh_observed_locations(pool: &PgPool) -> anyhow::Result<u64> {
    let result = sqlx::query(
        r#"UPDATE equipment e SET observed_locations = sub.locs
           FROM (
             SELECT ul.equipment_id,
                    array_agg(DISTINCT ul.location::text ORDER BY ul.location::text) AS locs
             FROM unit_loadout ul
             WHERE ul.location IS NOT NULL
             GROUP BY ul.equipment_id
           ) sub
           WHERE e.id = sub.equipment_id"#,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

// ── mech data ────────────────────────────────────────────────────────────────

/// Resolve a component alias to its reference table FK ID.
async fn resolve_alias(pool: &PgPool, table: &str, column: &str, alias: &str) -> Option<i32> {
    let query = format!(
        "SELECT {column} FROM {table} WHERE lower(alias) = lower($1)"
    );
    sqlx::query_scalar::<_, i32>(&query)
        .bind(alias.trim())
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

/// Upsert mech-specific structural data for a unit.
pub async fn upsert_mech_data(
    pool: &PgPool,
    unit_id: i32,
    data: &crate::parse::ParsedMechData,
) -> anyhow::Result<()> {
    // Resolve FK aliases for component types
    let engine_type_id = if let Some(ref et) = data.engine_type {
        resolve_alias(pool, "engine_type_aliases", "engine_type_id", et).await
    } else { None };
    let armor_type_id = if let Some(ref at) = data.armor_type {
        resolve_alias(pool, "armor_type_aliases", "armor_type_id", at).await
    } else { None };
    let structure_type_id = if let Some(ref st) = data.structure_type {
        resolve_alias(pool, "structure_type_aliases", "structure_type_id", st).await
    } else { None };
    let heatsink_type_id = if let Some(ref ht) = data.heat_sink_type {
        resolve_alias(pool, "heatsink_type_aliases", "heatsink_type_id", ht).await
    } else { None };
    // Default to standard gyro/cockpit/myomer when MegaMek omits the field
    let gyro_type_id = {
        let gt = data.gyro_type.as_deref().unwrap_or("Standard Gyro");
        resolve_alias(pool, "gyro_type_aliases", "gyro_type_id", gt).await
    };
    let cockpit_type_id = {
        let ct = data.cockpit_type.as_deref().unwrap_or("Standard Cockpit");
        resolve_alias(pool, "cockpit_type_aliases", "cockpit_type_id", ct).await
    };
    let myomer_type_id = {
        let mt = data.myomer_type.as_deref().unwrap_or("Standard");
        resolve_alias(pool, "myomer_type_aliases", "myomer_type_id", mt).await
    };

    sqlx::query(
        r#"INSERT INTO unit_mech_data (
               unit_id, config, is_omnimech, engine_rating, engine_type,
               walk_mp, jump_mp, heat_sink_count, heat_sink_type,
               structure_type, armor_type, gyro_type, cockpit_type, myomer_type,
               engine_type_id, armor_type_id, structure_type_id, heatsink_type_id,
               gyro_type_id, cockpit_type_id, myomer_type_id
           ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
           ON CONFLICT (unit_id) DO UPDATE SET
               config          = EXCLUDED.config,
               is_omnimech     = EXCLUDED.is_omnimech,
               engine_rating   = EXCLUDED.engine_rating,
               engine_type     = EXCLUDED.engine_type,
               walk_mp         = EXCLUDED.walk_mp,
               jump_mp         = EXCLUDED.jump_mp,
               heat_sink_count = EXCLUDED.heat_sink_count,
               heat_sink_type  = EXCLUDED.heat_sink_type,
               structure_type  = EXCLUDED.structure_type,
               armor_type      = EXCLUDED.armor_type,
               gyro_type       = EXCLUDED.gyro_type,
               cockpit_type    = EXCLUDED.cockpit_type,
               myomer_type     = EXCLUDED.myomer_type,
               engine_type_id    = EXCLUDED.engine_type_id,
               armor_type_id     = EXCLUDED.armor_type_id,
               structure_type_id = EXCLUDED.structure_type_id,
               heatsink_type_id  = EXCLUDED.heatsink_type_id,
               gyro_type_id      = EXCLUDED.gyro_type_id,
               cockpit_type_id   = EXCLUDED.cockpit_type_id,
               myomer_type_id    = EXCLUDED.myomer_type_id
        "#,
    )
    .bind(unit_id)
    .bind(&data.config)
    .bind(data.is_omnimech)
    .bind(data.engine_rating)
    .bind(&data.engine_type)
    .bind(data.walk_mp)
    .bind(data.jump_mp)
    .bind(data.heat_sink_count)
    .bind(&data.heat_sink_type)
    .bind(&data.structure_type)
    .bind(&data.armor_type)
    .bind(&data.gyro_type)
    .bind(&data.cockpit_type)
    .bind(&data.myomer_type)
    .bind(engine_type_id)
    .bind(armor_type_id)
    .bind(structure_type_id)
    .bind(heatsink_type_id)
    .bind(gyro_type_id)
    .bind(cockpit_type_id)
    .bind(myomer_type_id)
    .execute(pool)
    .await
    .with_context(|| format!("upsert_mech_data for unit {unit_id}"))?;
    Ok(())
}

// ── quirks ────────────────────────────────────────────────────────────────────

/// Ensure quirk row exists; return its id.
async fn ensure_quirk(pool: &PgPool, slug: &str) -> anyhow::Result<i32> {
    let row = sqlx::query(
        r#"
        INSERT INTO quirks (slug, name)
        VALUES ($1, $2)
        ON CONFLICT (slug) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(slug) // name placeholder; real names not in MTF/BLK
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        return Ok(r.try_get("id")?);
    }

    // Row already existed — fetch id
    let r = sqlx::query("SELECT id FROM quirks WHERE slug = $1")
        .bind(slug)
        .fetch_one(pool)
        .await
        .with_context(|| format!("ensure_quirk fetch: {slug}"))?;
    Ok(r.try_get("id")?)
}

/// Delete existing unit_quirks rows then bulk-insert fresh ones.
pub async fn replace_quirks(
    pool: &PgPool,
    unit_id: i32,
    quirks: &[String],
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM unit_quirks WHERE unit_id = $1")
        .bind(unit_id)
        .execute(pool)
        .await?;

    for quirk_slug in quirks {
        let quirk_id = ensure_quirk(pool, quirk_slug).await?;
        sqlx::query(
            "INSERT INTO unit_quirks (unit_id, quirk_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(unit_id)
        .bind(quirk_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}
