use sqlx::PgPool;

use crate::{db::models::DbEra, error::AppError};

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbEra>, AppError> {
    let row = sqlx::query_as!(
        DbEra,
        r#"SELECT id, slug, name, start_year, end_year, description
           FROM eras WHERE slug = $1"#,
        slug
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<DbEra>, AppError> {
    let rows = sqlx::query_as!(
        DbEra,
        r#"SELECT id, slug, name, start_year, end_year, description
           FROM eras ORDER BY start_year"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_by_year(pool: &PgPool, year: i32) -> Result<Vec<DbEra>, AppError> {
    let rows = sqlx::query_as!(
        DbEra,
        r#"SELECT id, slug, name, start_year, end_year, description
           FROM eras
           WHERE start_year <= $1 AND (end_year IS NULL OR end_year >= $1)
           ORDER BY start_year"#,
        year
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
