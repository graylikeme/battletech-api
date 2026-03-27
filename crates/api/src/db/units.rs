use sqlx::PgPool;

use crate::{
    db::models::{DbLoadoutEntry, DbLocation, DbMechData, DbQuirk, DbUnit, DbUnitChassis},
    error::AppError,
};

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbUnit>, AppError> {
    let row = sqlx::query_as!(
        DbUnit,
        r#"SELECT u.id, u.slug, u.chassis_id, u.variant, u.full_name,
                  u.tech_base::text AS "tech_base!", u.rules_level::text AS "rules_level!",
                  u.tonnage, u.bv, u.cost, u.intro_year, u.extinction_year,
                  u.reintro_year, u.source_book, u.description,
                  u.mul_id, u.role, u.clan_name, NULL::bigint AS total_count
           FROM units u WHERE u.slug = $1"#,
        slug
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn get_by_ids(pool: &PgPool, slugs: &[String]) -> Result<Vec<DbUnit>, AppError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }
    let rows = sqlx::query_as!(
        DbUnit,
        r#"SELECT u.id, u.slug, u.chassis_id, u.variant, u.full_name,
                  u.tech_base::text AS "tech_base!", u.rules_level::text AS "rules_level!",
                  u.tonnage, u.bv, u.cost, u.intro_year, u.extinction_year,
                  u.reintro_year, u.source_book, u.description,
                  u.mul_id, u.role, u.clan_name, NULL::bigint AS total_count
           FROM units u WHERE u.slug = ANY($1)"#,
        slugs
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub struct UnitFilter<'a> {
    pub name_search: Option<&'a str>,
    pub tech_base: Option<&'a str>,
    pub rules_level: Option<&'a str>,
    pub tonnage_min: Option<f64>,
    pub tonnage_max: Option<f64>,
    pub bv_min: Option<i32>,
    pub bv_max: Option<i32>,
    pub faction_slug: Option<&'a str>,
    pub era_slug: Option<&'a str>,
    pub is_omnimech: Option<bool>,
    pub config: Option<&'a str>,
    pub engine_type: Option<&'a str>,
    pub has_jump: Option<bool>,
    pub role: Option<&'a str>,
    pub unit_type: Option<&'a str>,
}

pub async fn search(
    pool: &PgPool,
    filter: UnitFilter<'_>,
    first: i64,
    after_cursor: Option<(&str, i32)>,
) -> Result<(Vec<DbUnit>, i64, bool), AppError> {
    let has_mech_filter = filter.is_omnimech.is_some()
        || filter.config.is_some()
        || filter.engine_type.is_some()
        || filter.has_jump.is_some();
    let has_chassis_filter = filter.unit_type.is_some();
    let has_cursor = after_cursor.is_some();

    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("");

    // When paginating with a cursor, wrap the base query in a MATERIALIZED CTE
    // so that COUNT(*) OVER() reflects the total filtered rows (not just rows
    // after the cursor position).
    if has_cursor {
        builder.push("WITH filtered AS MATERIALIZED (");
    }

    builder.push(
        r#"SELECT u.id, u.slug, u.chassis_id, u.variant, u.full_name,
                  u.tech_base::text AS tech_base, u.rules_level::text AS rules_level,
                  u.tonnage, u.bv, u.cost, u.intro_year, u.extinction_year,
                  u.reintro_year, u.source_book, u.description,
                  u.mul_id, u.role, u.clan_name,
                  COUNT(*) OVER() AS total_count
           FROM units u"#,
    );

    if has_mech_filter {
        builder.push(" JOIN unit_mech_data md ON md.unit_id = u.id");
    }
    if has_chassis_filter {
        builder.push(" JOIN unit_chassis uc ON uc.id = u.chassis_id");
    }

    builder.push(" WHERE TRUE");

    if let Some(name) = filter.name_search {
        builder.push(" AND (u.full_name ILIKE '%' || ");
        builder.push_bind(name);
        builder.push(" || '%' OR u.clan_name ILIKE '%' || ");
        builder.push_bind(name);
        builder.push(" || '%')");
    }
    if let Some(tb) = filter.tech_base {
        builder.push(" AND u.tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = filter.rules_level {
        builder.push(" AND u.rules_level::text = ");
        builder.push_bind(rl);
    }
    if let Some(min) = filter.tonnage_min {
        builder.push(" AND u.tonnage >= ");
        builder.push_bind(min);
    }
    if let Some(max) = filter.tonnage_max {
        builder.push(" AND u.tonnage <= ");
        builder.push_bind(max);
    }
    if let Some(faction) = filter.faction_slug {
        builder.push(r#" AND EXISTS (
            SELECT 1 FROM unit_availability ua
            JOIN factions f ON f.id = ua.faction_id
            WHERE ua.unit_id = u.id AND f.slug = "#);
        builder.push_bind(faction);
        builder.push(")");
    }
    if let Some(era) = filter.era_slug {
        builder.push(r#" AND EXISTS (
            SELECT 1 FROM unit_availability ua
            JOIN eras e ON e.id = ua.era_id
            WHERE ua.unit_id = u.id AND e.slug = "#);
        builder.push_bind(era);
        builder.push(")");
    }
    if let Some(omni) = filter.is_omnimech {
        builder.push(" AND md.is_omnimech = ");
        builder.push_bind(omni);
    }
    if let Some(cfg) = filter.config {
        builder.push(" AND md.config = ");
        builder.push_bind(cfg);
    }
    if let Some(et) = filter.engine_type {
        builder.push(" AND md.engine_type = ");
        builder.push_bind(et);
    }
    if let Some(has_jump) = filter.has_jump {
        if has_jump {
            builder.push(" AND md.jump_mp > 0");
        } else {
            builder.push(" AND (md.jump_mp IS NULL OR md.jump_mp = 0)");
        }
    }
    if let Some(role) = filter.role {
        builder.push(" AND u.role = ");
        builder.push_bind(role);
    }
    if let Some(min) = filter.bv_min {
        builder.push(" AND u.bv >= ");
        builder.push_bind(min);
    }
    if let Some(max) = filter.bv_max {
        builder.push(" AND u.bv <= ");
        builder.push_bind(max);
    }
    if let Some(ut) = filter.unit_type {
        builder.push(" AND uc.unit_type = ");
        builder.push_bind(ut);
    }

    // Close CTE and apply keyset cursor condition in the outer query.
    // The cursor uses (full_name, id) to match the ORDER BY columns.
    if let Some((sort_val, after_id)) = after_cursor {
        builder.push(") SELECT * FROM filtered WHERE (full_name > ");
        builder.push_bind(sort_val);
        builder.push(" OR (full_name = ");
        builder.push_bind(sort_val);
        builder.push(" AND id > ");
        builder.push_bind(after_id);
        builder.push("))");
    }

    builder.push(" ORDER BY full_name, id LIMIT ");
    builder.push_bind(first + 1);

    let mut rows = builder
        .build_query_as::<DbUnit>()
        .fetch_all(pool)
        .await?;

    let total_count = rows.first().and_then(|r| r.total_count).unwrap_or(0);
    let has_next = rows.len() as i64 > first;
    rows.truncate(first as usize);

    Ok((rows, total_count, has_next))
}

pub async fn get_chassis_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<DbUnitChassis>, AppError> {
    let row = sqlx::query_as!(
        DbUnitChassis,
        r#"SELECT id, slug, name, unit_type, tech_base::text AS "tech_base!",
                  tonnage, intro_year, description
           FROM unit_chassis WHERE slug = $1"#,
        slug
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_chassis(
    pool: &PgPool,
    unit_type: Option<&str>,
    tech_base: Option<&str>,
) -> Result<Vec<DbUnitChassis>, AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, unit_type, tech_base::text AS tech_base,
                  tonnage, intro_year, description
           FROM unit_chassis WHERE TRUE"#,
    );

    if let Some(ut) = unit_type {
        builder.push(" AND unit_type = ");
        builder.push_bind(ut);
    }
    if let Some(tb) = tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }

    builder.push(" ORDER BY name");

    let rows = builder
        .build_query_as::<DbUnitChassis>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_locations(pool: &PgPool, unit_id: i32) -> Result<Vec<DbLocation>, AppError> {
    let rows = sqlx::query_as!(
        DbLocation,
        r#"SELECT id, unit_id, location::text AS "location!",
                  armor_points, rear_armor, structure_points
           FROM unit_locations WHERE unit_id = $1 ORDER BY id"#,
        unit_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_loadout(pool: &PgPool, unit_id: i32) -> Result<Vec<DbLoadoutEntry>, AppError> {
    let rows = sqlx::query_as!(
        DbLoadoutEntry,
        r#"SELECT ul.id, ul.unit_id, ul.equipment_id,
                  ul.location::text AS location,
                  ul.quantity, ul.is_rear_facing, ul.notes,
                  e.slug AS equipment_slug, e.name AS equipment_name
           FROM unit_loadout ul
           JOIN equipment e ON e.id = ul.equipment_id
           WHERE ul.unit_id = $1
           ORDER BY ul.id"#,
        unit_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_mech_data_batch(
    pool: &PgPool,
    unit_ids: &[i32],
) -> Result<Vec<DbMechData>, AppError> {
    let rows = sqlx::query_as::<_, DbMechData>(
        r#"SELECT unit_id, config, is_omnimech, engine_rating, engine_type,
                  walk_mp, jump_mp, heat_sink_count, heat_sink_type,
                  structure_type, armor_type, gyro_type, cockpit_type, myomer_type,
                  engine_type_id, armor_type_id, structure_type_id, heatsink_type_id,
                  gyro_type_id, cockpit_type_id, myomer_type_id
           FROM unit_mech_data WHERE unit_id = ANY($1)"#,
    )
    .bind(unit_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_quirks(pool: &PgPool, unit_id: i32) -> Result<Vec<DbQuirk>, AppError> {
    let rows = sqlx::query_as!(
        DbQuirk,
        r#"SELECT q.id, q.slug, q.name, q.is_positive, q.description
           FROM quirks q
           JOIN unit_quirks uq ON uq.quirk_id = q.id
           WHERE uq.unit_id = $1
           ORDER BY q.name"#,
        unit_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Insert units whose auto-increment IDs do NOT match alphabetical name
    /// order — the exact scenario that triggered the keyset pagination bug.
    async fn seed_units(pool: &PgPool) {
        sqlx::query(
            "INSERT INTO unit_chassis (slug, name, unit_type, tech_base, tonnage)
             VALUES ('test-mech', 'Test', 'BattleMech', 'inner_sphere', 75)",
        )
        .execute(pool)
        .await
        .unwrap();

        let chassis_id: i32 =
            sqlx::query_scalar("SELECT id FROM unit_chassis WHERE slug = 'test-mech'")
                .fetch_one(pool)
                .await
                .unwrap();

        // Inserted in reverse-alpha order → id 1=Zephyr … id 5=Atlas.
        let units = [
            ("zephyr-zph-1", "ZPH-1", "Zephyr ZPH-1"),
            ("dragon-drg-1n", "DRG-1N", "Dragon DRG-1N"),
            ("centurion-cn9-a", "CN9-A", "Centurion CN9-A"),
            ("banshee-bnc-3e", "BNC-3E", "Banshee BNC-3E"),
            ("atlas-as7-d", "AS7-D", "Atlas AS7-D"),
        ];
        for (slug, variant, full_name) in &units {
            sqlx::query(
                "INSERT INTO units (slug, chassis_id, variant, full_name, tech_base, rules_level, tonnage)
                 VALUES ($1, $2, $3, $4, 'inner_sphere', 'standard', 75)",
            )
            .bind(slug)
            .bind(chassis_id)
            .bind(variant)
            .bind(full_name)
            .execute(pool)
            .await
            .unwrap();
        }
    }

    fn empty_filter() -> UnitFilter<'static> {
        UnitFilter {
            name_search: None,
            tech_base: None,
            rules_level: None,
            tonnage_min: None,
            tonnage_max: None,
            bv_min: None,
            bv_max: None,
            faction_slug: None,
            era_slug: None,
            is_omnimech: None,
            config: None,
            engine_type: None,
            has_jump: None,
            role: None,
        }
    }

    /// Regression: pagination must not produce duplicates or skip items when
    /// DB row IDs don't match the alphabetical sort order.
    #[sqlx::test(migrations = "../../migrations")]
    async fn keyset_pagination_no_duplicates_or_gaps(pool: PgPool) {
        seed_units(&pool).await;

        let mut all_names: Vec<String> = vec![];
        let mut cursor: Option<(String, i32)> = None;
        let mut pages = 0;

        loop {
            let cursor_ref = cursor.as_ref().map(|(s, id)| (s.as_str(), *id));
            let (rows, total, has_next) =
                search(&pool, empty_filter(), 2, cursor_ref).await.unwrap();

            assert_eq!(total, 5, "totalCount must be stable across all pages");

            for row in &rows {
                all_names.push(row.full_name.clone());
            }

            pages += 1;
            if !has_next {
                break;
            }
            let last = rows.last().unwrap();
            cursor = Some((last.full_name.clone(), last.id));
        }

        assert_eq!(pages, 3, "5 items / page size 2 = 3 pages");
        assert_eq!(
            all_names,
            ["Atlas AS7-D", "Banshee BNC-3E", "Centurion CN9-A", "Dragon DRG-1N", "Zephyr ZPH-1"],
            "items must appear in alphabetical order with no duplicates or gaps"
        );
    }

    /// Regression: totalCount must reflect all filtered rows, not just rows
    /// remaining after the cursor position.
    #[sqlx::test(migrations = "../../migrations")]
    async fn total_count_stable_across_pages(pool: PgPool) {
        seed_units(&pool).await;

        let (page1, total1, _) = search(&pool, empty_filter(), 2, None).await.unwrap();
        let last = page1.last().unwrap();
        let cursor = (last.full_name.as_str(), last.id);

        let (_, total2, _) = search(&pool, empty_filter(), 2, Some(cursor)).await.unwrap();

        assert_eq!(total1, total2, "totalCount must not change between pages");
    }

    /// Edge case: items with the same name must paginate correctly using the
    /// ID tiebreaker.
    #[sqlx::test(migrations = "../../migrations")]
    async fn pagination_with_duplicate_names(pool: PgPool) {
        sqlx::query(
            "INSERT INTO unit_chassis (slug, name, unit_type, tech_base, tonnage)
             VALUES ('test-mech', 'Test', 'BattleMech', 'inner_sphere', 75)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let chassis_id: i32 =
            sqlx::query_scalar("SELECT id FROM unit_chassis WHERE slug = 'test-mech'")
                .fetch_one(&pool)
                .await
                .unwrap();

        for i in 1..=3 {
            sqlx::query(
                "INSERT INTO units (slug, chassis_id, variant, full_name, tech_base, rules_level, tonnage)
                 VALUES ($1, $2, $3, 'Same Name', 'inner_sphere', 'standard', 75)",
            )
            .bind(format!("same-name-{i}"))
            .bind(chassis_id)
            .bind(format!("V{i}"))
            .execute(&pool)
            .await
            .unwrap();
        }

        let (page1, _, has_next) = search(&pool, empty_filter(), 2, None).await.unwrap();
        assert!(has_next);
        assert_eq!(page1.len(), 2);

        let last = page1.last().unwrap();
        let cursor = (last.full_name.as_str(), last.id);
        let (page2, _, has_next2) = search(&pool, empty_filter(), 2, Some(cursor)).await.unwrap();
        assert!(!has_next2);
        assert_eq!(page2.len(), 1);

        let all_ids: Vec<i32> = page1.iter().chain(page2.iter()).map(|u| u.id).collect();
        assert_eq!(all_ids.len(), 3);
        let unique: std::collections::HashSet<i32> = all_ids.iter().copied().collect();
        assert_eq!(unique.len(), 3, "no duplicate IDs");
        assert!(
            all_ids.windows(2).all(|w| w[0] < w[1]),
            "IDs must be ascending when names are equal"
        );
    }
}
