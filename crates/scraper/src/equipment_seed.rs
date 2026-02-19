use std::collections::HashMap;
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

/// Build a mapping from clean JSON slugs to MegaMek DB slugs.
///
/// MegaMek uses internal names like `CLERLargeLaser` or `ISUltraAC5` which get
/// slugified to `clerlargelaser` / `isultraac5`.  The JSON uses human-readable
/// slugs like `clan-er-large-laser` / `ultra-autocannon-5`.  This table bridges
/// the two naming conventions.
fn slug_aliases() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        // Clan energy weapons
        ("clan-er-large-laser",     "clerlargelaser"),
        ("clan-er-medium-laser",    "clermediumlaser"),
        ("clan-er-small-laser",     "clersmalllaser"),
        ("clan-er-ppc",             "clerppc"),
        ("clan-large-pulse-laser",  "cllargepulselaser"),
        ("clan-medium-pulse-laser", "clmediumpulselaser"),
        ("clan-small-pulse-laser",  "clsmallpulselaser"),
        ("clan-er-flamer",          "clerflamer"),
        ("clan-plasma-cannon",      "clplasmacannon"),
        // IS energy weapons (pulse)
        ("pulse-large-laser",       "islargepulselaser"),
        ("pulse-medium-laser",      "ismediumpulselaser"),
        ("pulse-small-laser",       "issmallpulselaser"),
        // IS ballistic weapons
        ("ultra-autocannon-2",      "isultraac2"),
        ("ultra-autocannon-5",      "isultraac5"),
        ("ultra-autocannon-10",     "isultraac10"),
        ("ultra-autocannon-20",     "isultraac20"),
        ("rotary-autocannon-5",     "isrotaryac5"),
        ("light-autocannon-5",      "light-ac-5"),
        // Clan ballistic weapons
        ("clan-ultra-autocannon-2",  "clultraac2"),
        ("clan-ultra-autocannon-5",  "clultraac5"),
        ("clan-ultra-autocannon-10", "clultraac10"),
        ("clan-ultra-autocannon-20", "clultraac20"),
        ("clan-lb-2-x-ac",          "cllbxac2"),
        ("clan-lb-5-x-ac",          "cllbxac5"),
        ("clan-lb-10-x-ac",         "cllbxac10"),
        ("clan-lb-20-x-ac",         "cllbxac20"),
        ("clan-gauss-rifle",        "clgaussrifle"),
        // Clan missile weapons
        ("clan-srm-2",              "clsrm2"),
        ("clan-srm-4",              "clsrm4"),
        ("clan-srm-6",              "clsrm6"),
        ("clan-lrm-5",              "cllrm5"),
        ("clan-lrm-10",             "cllrm10"),
        ("clan-lrm-15",             "cllrm15"),
        ("clan-lrm-20",             "cllrm20"),
        ("clan-streak-srm-2",       "clstreaksrm2"),
        ("clan-streak-srm-4",       "clstreaksrm4"),
        ("clan-streak-srm-6",       "clstreaksrm6"),
        ("clan-arrow-iv",           "clarrowiv"),
        // IS missile
        ("narc-missile-beacon",     "narc"),
        // Electronics / equipment
        ("guardian-ecm-suite",      "isguardianecmsuite"),
        ("clan-ecm-suite",          "clecmsuite"),
        ("beagle-active-probe",     "beagleactiveprobe"),
        ("clan-active-probe",       "clactiveprobe"),
        ("clan-anti-missile-system","clantimissilesystem"),
        ("targeting-computer",      "istargeting-computer"),
        ("artemis-iv-fcs",          "isartemisiv"),
        ("c3-master-computer",      "isc3mastercomputer"),
        ("c3-slave-unit",           "isc3slaveunit"),
    ])
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

    let aliases = slug_aliases();
    let mut updated = 0u32;
    let mut not_found = 0u32;
    let mut unchanged = 0u32;
    let mut alias_hits = 0u32;

    for entry in &entries {
        // Try exact slug match first, then alias fallback
        let mut row = sqlx::query("SELECT id FROM equipment WHERE slug = $1")
            .bind(&entry.slug)
            .fetch_optional(&pool)
            .await?;

        if row.is_none() {
            if let Some(&alt) = aliases.get(entry.slug.as_str()) {
                row = sqlx::query("SELECT id FROM equipment WHERE slug = $1")
                    .bind(alt)
                    .fetch_optional(&pool)
                    .await?;
                if row.is_some() {
                    alias_hits += 1;
                }
            }
        }

        let Some(r) = row else {
            warn!(slug = %entry.slug, "no matching equipment row");
            not_found += 1;
            continue;
        };
        let eq_id: i32 = r.try_get("id")?;

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
        alias_hits,
        not_found,
        unchanged,
        "equipment seed complete"
    );

    Ok(())
}
