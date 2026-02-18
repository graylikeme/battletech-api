mod db;
mod parse;
mod seed;

use std::{
    collections::HashMap,
    io::{BufReader, Read},
    path::PathBuf,
};

use anyhow::{bail, Context};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info, warn};

use parse::{parse_blk, parse_mtf, UnitType};

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "scraper", about = "Import MegaMek unit data into PostgreSQL")]
struct Cli {
    /// Path to unit_files.zip from a MegaMek release.
    #[arg(long, value_name = "FILE")]
    zip: PathBuf,

    /// Override DATABASE_URL (defaults to env var).
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// MegaMek version string stored in dataset_metadata.
    #[arg(long, default_value = "unknown")]
    version: String,

    /// Maximum DB connections in pool.
    #[arg(long, default_value_t = 5)]
    pool_size: u32,

    /// Stop after this many parse errors (0 = unlimited).
    #[arg(long, default_value_t = 0)]
    max_errors: usize,
}

// ── entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    // ── connect ──────────────────────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(cli.pool_size)
        .connect(&cli.database_url)
        .await
        .context("connecting to database")?;

    // ── seed reference data ──────────────────────────────────────────────────
    let era_count = seed::seed_eras(&pool).await?;
    let faction_count = seed::seed_factions(&pool).await?;
    seed::seed_metadata(&pool, &cli.version).await?;
    info!(eras = era_count, factions = faction_count, version = %cli.version, "reference data seeded");

    // ── open zip ─────────────────────────────────────────────────────────────
    let file = std::fs::File::open(&cli.zip)
        .with_context(|| format!("opening zip {:?}", cli.zip))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .context("reading zip archive")?;

    let total_entries = archive.len();
    info!(total_entries, "zip opened");

    // ── pass 1: collect file names to decide what to parse ───────────────────
    // (ZipArchive borrows mutably per entry, so we collect paths first)
    let entry_names: Vec<String> = (0..total_entries)
        .map(|i| archive.by_index(i).map(|e| e.name().to_owned()))
        .collect::<Result<_, _>>()
        .context("listing zip entries")?;

    // ── pass 2: parse & import ───────────────────────────────────────────────
    let mut equipment_cache: HashMap<String, i32> = HashMap::new();

    let mut parsed   = 0usize;
    let mut skipped  = 0usize;
    let mut errors   = 0usize;
    let mut imported = 0usize;

    for (idx, name) in entry_names.iter().enumerate() {
        let (is_mtf, default_type) = classify(name);
        if !is_mtf && default_type.is_none() {
            skipped += 1;
            continue;
        }

        // Read entry content (handle non-UTF-8 files gracefully)
        let content = {
            let mut entry = archive.by_index(idx).context("reading zip entry")?;
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .with_context(|| format!("reading bytes of {name}"))?;
            match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => {
                    warn!(file = %name, "non-UTF-8 content — skipping");
                    skipped += 1;
                    continue;
                }
            }
        };

        // Parse
        let unit = if is_mtf {
            parse_mtf(&content)
        } else {
            parse_blk(&content, default_type.unwrap())
        };

        let unit = match unit {
            Some(u) => u,
            None => {
                warn!(file = %name, "parse returned None — skipping");
                skipped += 1;
                continue;
            }
        };

        parsed += 1;

        // Import
        match import_unit(&pool, &unit, &mut equipment_cache).await {
            Ok(()) => imported += 1,
            Err(e) => {
                errors += 1;
                error!(file = %name, error = %e, "import failed");
                if cli.max_errors > 0 && errors >= cli.max_errors {
                    bail!("reached max_errors limit ({}) — aborting", cli.max_errors);
                }
            }
        }

        if parsed % 500 == 0 {
            info!(parsed, imported, errors, skipped, "progress");
        }
    }

    info!(
        total_entries,
        parsed,
        imported,
        errors,
        skipped,
        "import complete"
    );

    Ok(())
}

// ── per-unit import ───────────────────────────────────────────────────────────

async fn import_unit(
    pool: &sqlx::PgPool,
    unit: &parse::ParsedUnit,
    equipment_cache: &mut HashMap<String, i32>,
) -> anyhow::Result<()> {
    let chassis_id = db::upsert_chassis(pool, unit).await?;
    let unit_id    = db::upsert_unit(pool, unit, chassis_id).await?;

    if !unit.locations.is_empty() {
        db::replace_locations(pool, unit_id, unit).await?;
    }
    if !unit.loadout.is_empty() {
        db::replace_loadout(pool, unit_id, unit, equipment_cache).await?;
    }
    if !unit.quirks.is_empty() {
        db::replace_quirks(pool, unit_id, &unit.quirks).await?;
    }
    if let Some(ref mech_data) = unit.mech_data {
        db::upsert_mech_data(pool, unit_id, mech_data).await?;
    }

    Ok(())
}

// ── zip entry classifier ──────────────────────────────────────────────────────

/// Returns `(is_mtf, default_unit_type)`.
/// `is_mtf = true`  → parse as MTF (mech).
/// `default_type = Some(t)` → parse as BLK with given default type.
/// Both false/None → skip.
fn classify(path: &str) -> (bool, Option<UnitType>) {
    let lower = path.to_lowercase();

    // Skip directories and non-unit files
    if lower.ends_with('/') {
        return (false, None);
    }

    let ext = std::path::Path::new(&lower)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "mtf" => (true, None),
        "blk" => {
            // Infer default unit type from directory component
            let dir = lower.split('/').rev().nth(1).unwrap_or("");
            let unit_type = if dir.contains("vehicle") || dir.contains("vee") {
                UnitType::Vehicle
            } else if dir.contains("fighter") || dir.contains("aero") {
                UnitType::Fighter
            } else {
                UnitType::Other
            };
            (false, Some(unit_type))
        }
        _ => (false, None),
    }
}
