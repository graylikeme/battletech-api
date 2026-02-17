use sqlx::PgPool;

use crate::{db::models::{DbMetadata, DbRuleset}, error::AppError};

pub async fn get_latest(pool: &PgPool) -> Result<Option<DbMetadata>, AppError> {
    let row = sqlx::query_as!(
        DbMetadata,
        r#"SELECT id, version, schema_version, description, release_date, created_at
           FROM dataset_metadata
           ORDER BY id DESC
           LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_rulesets(pool: &PgPool) -> Result<Vec<DbRuleset>, AppError> {
    let rows = sqlx::query_as!(
        DbRuleset,
        r#"SELECT id, slug, name, level::text AS level, description, source_book
           FROM rulesets
           ORDER BY id"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
