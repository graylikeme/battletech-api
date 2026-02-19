use sqlx::PgPool;

use crate::{db::models::DbEquipment, error::AppError};

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbEquipment>, AppError> {
    let row = sqlx::query_as::<_, DbEquipment>(
        r#"SELECT id, slug, name,
                  category::text AS category, tech_base::text AS tech_base,
                  rules_level::text AS rules_level,
                  tonnage, crits, damage, heat,
                  range_min, range_short, range_medium, range_long, bv, intro_year,
                  source_book, description,
                  observed_locations, ammo_for_id, stats_source,
                  NULL::bigint AS total_count
           FROM equipment WHERE slug = $1"#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub struct EquipmentFilter<'a> {
    pub name_search: Option<&'a str>,
    pub category: Option<&'a str>,
    pub tech_base: Option<&'a str>,
    pub rules_level: Option<&'a str>,
    pub max_tonnage: Option<f64>,
    pub max_crits: Option<i32>,
    pub observed_location: Option<&'a str>,
    pub ammo_for_slug: Option<&'a str>,
}

pub async fn search(
    pool: &PgPool,
    filter: EquipmentFilter<'_>,
    first: i64,
    after_id: Option<i32>,
) -> Result<(Vec<DbEquipment>, i64, bool), AppError> {
    // Pre-resolve ammo_for_slug to ID
    let resolved_ammo_for_id = if let Some(slug) = filter.ammo_for_slug {
        sqlx::query_scalar::<_, i32>("SELECT id FROM equipment WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, category::text AS category, tech_base::text AS tech_base,
                  rules_level::text AS rules_level, tonnage, crits, damage, heat,
                  range_min, range_short, range_medium, range_long, bv, intro_year,
                  source_book, description,
                  observed_locations, ammo_for_id, stats_source,
                  COUNT(*) OVER() AS total_count
           FROM equipment WHERE TRUE"#,
    );

    if let Some(n) = filter.name_search {
        builder.push(" AND name ILIKE '%' || ");
        builder.push_bind(n);
        builder.push(" || '%'");
    }
    if let Some(c) = filter.category {
        builder.push(" AND category::text = ");
        builder.push_bind(c);
    }
    if let Some(tb) = filter.tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = filter.rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
    }
    if let Some(max_t) = filter.max_tonnage {
        builder.push(" AND tonnage IS NOT NULL AND tonnage <= ");
        builder.push_bind(rust_decimal::Decimal::try_from(max_t).unwrap_or_default());
    }
    if let Some(max_c) = filter.max_crits {
        builder.push(" AND crits IS NOT NULL AND crits <= ");
        builder.push_bind(max_c);
    }
    if let Some(loc) = filter.observed_location {
        builder.push(" AND observed_locations @> ARRAY[");
        builder.push_bind(loc);
        builder.push("]");
    }
    if let Some(weapon_id) = resolved_ammo_for_id {
        builder.push(" AND ammo_for_id = ");
        builder.push_bind(weapon_id);
    }
    if let Some(aid) = after_id {
        builder.push(" AND id > ");
        builder.push_bind(aid);
    }

    builder.push(" ORDER BY name, id LIMIT ");
    builder.push_bind(first + 1);

    let mut rows = builder
        .build_query_as::<DbEquipment>()
        .fetch_all(pool)
        .await?;

    let total_count = rows.first().and_then(|r| r.total_count).unwrap_or(0);
    let has_next = rows.len() as i64 > first;
    rows.truncate(first as usize);

    Ok((rows, total_count, has_next))
}
