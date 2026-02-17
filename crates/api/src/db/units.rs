use sqlx::PgPool;

use crate::{
    db::models::{DbLoadoutEntry, DbLocation, DbQuirk, DbUnit, DbUnitChassis},
    error::AppError,
};

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbUnit>, AppError> {
    let row = sqlx::query_as!(
        DbUnit,
        r#"SELECT u.id, u.slug, u.chassis_id, u.variant, u.full_name,
                  u.tech_base::text AS "tech_base!", u.rules_level::text AS "rules_level!",
                  u.tonnage, u.bv, u.cost, u.intro_year, u.extinction_year,
                  u.reintro_year, u.source_book, u.description, NULL::bigint AS total_count
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
                  u.reintro_year, u.source_book, u.description, NULL::bigint AS total_count
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
    pub faction_slug: Option<&'a str>,
    pub era_slug: Option<&'a str>,
}

pub async fn search(
    pool: &PgPool,
    filter: UnitFilter<'_>,
    first: i64,
    after_id: Option<i32>,
) -> Result<(Vec<DbUnit>, i64, bool), AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT u.id, u.slug, u.chassis_id, u.variant, u.full_name,
                  u.tech_base::text AS tech_base, u.rules_level::text AS rules_level,
                  u.tonnage, u.bv, u.cost, u.intro_year, u.extinction_year,
                  u.reintro_year, u.source_book, u.description,
                  COUNT(*) OVER() AS total_count
           FROM units u WHERE TRUE"#,
    );

    if let Some(name) = filter.name_search {
        builder.push(" AND u.full_name ILIKE '%' || ");
        builder.push_bind(name);
        builder.push(" || '%'");
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
    if let Some(aid) = after_id {
        builder.push(" AND u.id > ");
        builder.push_bind(aid);
    }

    builder.push(" ORDER BY u.full_name, u.id LIMIT ");
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
