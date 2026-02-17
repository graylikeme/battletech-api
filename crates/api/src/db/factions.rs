use sqlx::PgPool;

use crate::{db::models::DbFaction, error::AppError};

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbFaction>, AppError> {
    let row = sqlx::query_as!(
        DbFaction,
        r#"SELECT id, slug, name, short_name, faction_type, is_clan,
                  founding_year, dissolution_year, description
           FROM factions WHERE slug = $1"#,
        slug
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list(
    pool: &PgPool,
    faction_type: Option<&str>,
    is_clan: Option<bool>,
    era_slug: Option<&str>,
) -> Result<Vec<DbFaction>, AppError> {
    // Build dynamic query using QueryBuilder for safety
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT f.id, f.slug, f.name, f.short_name, f.faction_type, f.is_clan,
                  f.founding_year, f.dissolution_year, f.description
           FROM factions f"#,
    );

    if let Some(era) = era_slug {
        builder.push(
            r#" WHERE EXISTS (
                SELECT 1 FROM faction_eras fe
                JOIN eras e ON e.id = fe.era_id
                WHERE fe.faction_id = f.id AND e.slug = "#,
        );
        builder.push_bind(era);
        builder.push(")");
    }

    let mut has_where = era_slug.is_some();

    if let Some(ft) = faction_type {
        if has_where {
            builder.push(" AND ");
        } else {
            builder.push(" WHERE ");
            has_where = true;
        }
        builder.push("f.faction_type = ");
        builder.push_bind(ft);
    }

    if let Some(clan) = is_clan {
        if has_where {
            builder.push(" AND ");
        } else {
            builder.push(" WHERE ");
        }
        builder.push("f.is_clan = ");
        builder.push_bind(clan);
    }

    builder.push(" ORDER BY f.name");

    let rows = builder
        .build_query_as::<DbFaction>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}
