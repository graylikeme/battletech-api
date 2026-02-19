use std::path::PathBuf;

use anyhow::Context;
use tracing::info;

use super::client::MulClient;
use super::quicklist;

/// Run the mul-fetch subcommand: fetch QuickList + detail pages to local files.
pub async fn run(
    output_dir: PathBuf,
    delay_ms: u64,
    base_url: &str,
    types: &[u32],
) -> anyhow::Result<()> {
    let client = MulClient::new(base_url, delay_ms)?;

    // Create directory structure
    let details_dir = output_dir.join("details");
    std::fs::create_dir_all(&details_dir)
        .with_context(|| format!("creating {:?}", details_dir))?;

    // Step 1: Fetch QuickList per type
    let mut all_mul_ids: Vec<u32> = Vec::new();
    let mut quicklist_counts: Vec<(u32, usize)> = Vec::new();

    for &type_id in types {
        info!(type_id, "fetching QuickList");
        let json = client.fetch_quicklist(type_id).await
            .with_context(|| format!("fetching QuickList for type {type_id}"))?;

        let filename = output_dir.join(format!("quicklist-{type_id}.json"));
        std::fs::write(&filename, &json)
            .with_context(|| format!("writing {:?}", filename))?;

        let units = quicklist::parse_quicklist(&json)?;
        let count = units.len();
        info!(type_id, count, "QuickList saved");

        for u in &units {
            all_mul_ids.push(u.id);
        }
        quicklist_counts.push((type_id, count));
    }

    // Deduplicate MUL IDs (some units may appear under multiple types)
    all_mul_ids.sort_unstable();
    all_mul_ids.dedup();

    let total_ids = all_mul_ids.len();
    info!(total = total_ids, "unique MUL IDs to fetch detail pages for");

    // Step 2: Fetch detail pages
    let mut fetched = 0usize;
    let mut skipped = 0usize;

    for (idx, &mul_id) in all_mul_ids.iter().enumerate() {
        let detail_path = details_dir.join(format!("{mul_id}.html"));
        if detail_path.exists() {
            skipped += 1;
            continue;
        }

        let html = client.fetch_detail(mul_id).await
            .with_context(|| format!("fetching detail page for MUL ID {mul_id}"))?;

        std::fs::write(&detail_path, &html)
            .with_context(|| format!("writing {:?}", detail_path))?;

        fetched += 1;

        if (fetched + skipped) % 100 == 0 || idx == total_ids - 1 {
            info!(
                fetched,
                skipped,
                remaining = total_ids - idx - 1,
                "detail page progress"
            );
        }

        client.sleep_with_jitter().await;
    }

    // Step 3: Write manifest
    let manifest = serde_json::json!({
        "fetched_at": chrono::Utc::now().to_rfc3339(),
        "base_url": base_url,
        "types": types,
        "quicklist_counts": quicklist_counts.iter()
            .map(|(t, c)| (t.to_string(), c))
            .collect::<std::collections::HashMap<_, _>>(),
        "detail_pages_fetched": fetched,
        "detail_pages_skipped": skipped,
        "total_mul_ids": total_ids,
    });

    let manifest_path = output_dir.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("writing {:?}", manifest_path))?;

    info!(
        fetched,
        skipped,
        total = total_ids,
        "fetch complete — manifest written to {:?}",
        manifest_path
    );

    Ok(())
}
