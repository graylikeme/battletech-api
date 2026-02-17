use sqlx::PgPool;

use crate::{db::models::DbEquipment, error::AppError};

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbEquipment>, AppError> {
    let row = sqlx::query_as!(
        DbEquipment,
        r#"SELECT id, slug, name, category::text AS category, tech_base::text AS tech_base,
                  rules_level::text AS rules_level, tonnage, crits, damage, heat,
                  range_min, range_short, range_medium, range_long, bv, intro_year,
                  source_book, description, NULL::bigint AS total_count
           FROM equipment WHERE slug = $1"#,
        slug
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn search(
    pool: &PgPool,
    name_search: Option<&str>,
    category: Option<&str>,
    tech_base: Option<&str>,
    rules_level: Option<&str>,
    first: i64,
    after_id: Option<i32>,
) -> Result<(Vec<DbEquipment>, i64, bool), AppError> {
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT id, slug, name, category::text AS category, tech_base::text AS tech_base,
                  rules_level::text AS rules_level, tonnage, crits, damage, heat,
                  range_min, range_short, range_medium, range_long, bv, intro_year,
                  source_book, description, COUNT(*) OVER() AS total_count
           FROM equipment WHERE TRUE"#,
    );

    if let Some(n) = name_search {
        builder.push(" AND name ILIKE '%' || ");
        builder.push_bind(n);
        builder.push(" || '%'");
    }
    if let Some(c) = category {
        builder.push(" AND category::text = ");
        builder.push_bind(c);
    }
    if let Some(tb) = tech_base {
        builder.push(" AND tech_base::text = ");
        builder.push_bind(tb);
    }
    if let Some(rl) = rules_level {
        builder.push(" AND rules_level::text = ");
        builder.push_bind(rl);
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
