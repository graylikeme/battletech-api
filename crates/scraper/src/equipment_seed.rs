use std::path::Path;

use anyhow::Context;
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::Row;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
pub struct EquipmentStats {
    pub slug: String,
    pub tonnage: Option<f64>,
    pub crits: Option<i32>,
    pub damage: Option<String>,
    pub heat: Option<i32>,
    pub range_min: Option<i32>,
    pub range_short: Option<i32>,
    pub range_medium: Option<i32>,
    pub range_long: Option<i32>,
    pub bv: Option<i32>,
}

pub async fn run(file: &Path, database_url: &str, pool_size: u32, force: bool) -> anyhow::Result<()> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(database_url)
        .await
        .context("connecting to database")?;

    let content = std::fs::read_to_string(file)
        .with_context(|| format!("reading {:?}", file))?;
    let entries: Vec<EquipmentStats> =
        serde_json::from_str(&content).context("parsing equipment stats JSON")?;

    info!(count = entries.len(), "loaded equipment stats entries");

    let mut updated = 0u32;
    let skipped = 0u32;
    let mut not_found = 0u32;
    let mut unchanged = 0u32;

    for entry in &entries {
        let exists = sqlx::query("SELECT id FROM equipment WHERE slug = $1")
            .bind(&entry.slug)
            .fetch_optional(&pool)
            .await?;

        let Some(row) = exists else {
            warn!(slug = %entry.slug, "no matching equipment row");
            not_found += 1;
            continue;
        };
        let eq_id: i32 = row.try_get("id")?;

        if force {
            let result = sqlx::query(
                r#"UPDATE equipment SET
                     tonnage      = $2,
                     crits        = $3,
                     damage       = $4,
                     heat         = $5,
                     range_min    = $6,
                     range_short  = $7,
                     range_medium = $8,
                     range_long   = $9,
                     bv           = $10,
                     stats_source = 'seed',
                     stats_updated_at = now()
                   WHERE id = $1"#,
            )
            .bind(eq_id)
            .bind(entry.tonnage.map(|t| Decimal::try_from(t).unwrap_or_default()))
            .bind(entry.crits)
            .bind(&entry.damage)
            .bind(entry.heat)
            .bind(entry.range_min)
            .bind(entry.range_short)
            .bind(entry.range_medium)
            .bind(entry.range_long)
            .bind(entry.bv)
            .execute(&pool)
            .await?;

            if result.rows_affected() > 0 {
                updated += 1;
            } else {
                unchanged += 1;
            }
        } else {
            // Only update NULL columns
            let result = sqlx::query(
                r#"UPDATE equipment SET
                     tonnage      = COALESCE(tonnage, $2),
                     crits        = COALESCE(crits, $3),
                     damage       = COALESCE(damage, $4),
                     heat         = COALESCE(heat, $5),
                     range_min    = COALESCE(range_min, $6),
                     range_short  = COALESCE(range_short, $7),
                     range_medium = COALESCE(range_medium, $8),
                     range_long   = COALESCE(range_long, $9),
                     bv           = COALESCE(bv, $10),
                     stats_source = COALESCE(stats_source, 'seed'),
                     stats_updated_at = COALESCE(stats_updated_at, now())
                   WHERE id = $1
                     AND (tonnage IS NULL OR crits IS NULL OR damage IS NULL
                          OR heat IS NULL OR range_min IS NULL OR range_short IS NULL
                          OR range_medium IS NULL OR range_long IS NULL OR bv IS NULL)"#,
            )
            .bind(eq_id)
            .bind(entry.tonnage.map(|t| Decimal::try_from(t).unwrap_or_default()))
            .bind(entry.crits)
            .bind(&entry.damage)
            .bind(entry.heat)
            .bind(entry.range_min)
            .bind(entry.range_short)
            .bind(entry.range_medium)
            .bind(entry.range_long)
            .bind(entry.bv)
            .execute(&pool)
            .await?;

            if result.rows_affected() > 0 {
                updated += 1;
            } else {
                unchanged += 1;
            }
        }
    }

    info!(
        updated,
        skipped,
        not_found,
        unchanged,
        "equipment seed complete"
    );

    // Suppress unused variable warning
    let _ = skipped;

    Ok(())
}
