use std::time::Duration;

use anyhow::{bail, Context};
use rand::Rng;
use tracing::{info, warn};

pub const DEFAULT_BASE_URL: &str = "https://masterunitlist.azurewebsites.net";

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const TIMEOUT_SECS: u64 = 30;
const MAX_RETRIES: u32 = 3;

pub struct MulClient {
    client: reqwest::Client,
    base_url: String,
    delay_ms: u64,
}

impl MulClient {
    pub fn new(base_url: &str, delay_ms: u64) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .gzip(true)
            .build()
            .context("building HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            delay_ms,
        })
    }

    /// Fetch QuickList JSON for a given unit type.
    /// Splits by tonnage ranges to avoid the MUL server's maxJsonLength error.
    pub async fn fetch_quicklist(&self, type_id: u32) -> anyhow::Result<String> {
        // Tonnage ranges to avoid oversized responses
        let ranges: &[(u32, u32)] = &[
            (0, 25), (26, 35), (36, 45), (46, 55), (56, 65),
            (66, 75), (76, 85), (86, 100), (101, 200),
            (201, 999999),
        ];

        let mut all_units: Vec<serde_json::Value> = Vec::new();

        for &(min, max) in ranges {
            let url = format!(
                "{}/Unit/QuickList?Types={}&MinTons={}&MaxTons={}",
                self.base_url, type_id, min, max
            );
            let body = self.fetch_with_retry(&url).await?;
            let parsed: serde_json::Value = serde_json::from_str(&body)
                .with_context(|| format!("parsing QuickList JSON for type={type_id} tons={min}-{max}"))?;

            let units = match parsed.get("Units") {
                Some(arr) => arr.as_array().cloned().unwrap_or_default(),
                None => {
                    // Maybe it's a top-level array
                    if let Some(arr) = parsed.as_array() {
                        arr.clone()
                    } else {
                        warn!(type_id, min, max, "unexpected QuickList shape");
                        continue;
                    }
                }
            };

            info!(type_id, min_tons = min, max_tons = max, count = units.len(), "fetched QuickList range");
            all_units.extend(units);

            self.sleep_with_jitter().await;
        }

        let wrapper = serde_json::json!({ "Units": all_units });
        Ok(serde_json::to_string_pretty(&wrapper)?)
    }

    /// Fetch a detail page HTML by MUL ID.
    pub async fn fetch_detail(&self, mul_id: u32) -> anyhow::Result<String> {
        let url = format!("{}/Unit/Details/{}", self.base_url, mul_id);
        self.fetch_with_retry(&url).await
    }

    /// Sleep with ±30% jitter around the configured delay.
    pub async fn sleep_with_jitter(&self) {
        if self.delay_ms == 0 {
            return;
        }
        let jitter_range = (self.delay_ms as f64 * 0.3) as u64;
        let jitter: u64 = if jitter_range > 0 {
            rand::rng().random_range(0..=jitter_range * 2)
        } else {
            0
        };
        let actual = self.delay_ms.saturating_sub(jitter_range) + jitter;
        tokio::time::sleep(Duration::from_millis(actual)).await;
    }

    /// Fetch a URL with retry logic for 429 and 5xx responses.
    async fn fetch_with_retry(&self, url: &str) -> anyhow::Result<String> {
        let backoff_secs = [2, 5, 15];

        for attempt in 0..=MAX_RETRIES {
            let resp = self.client.get(url).send().await;

            match resp {
                Ok(r) => {
                    let status = r.status();
                    if status.is_success() {
                        return r.text().await.context("reading response body");
                    }

                    if attempt == MAX_RETRIES {
                        bail!("request to {} failed after {} retries: HTTP {}", url, MAX_RETRIES, status);
                    }

                    let wait = if status.as_u16() == 429 {
                        // Honor Retry-After if present, capped at 60s
                        let retry_after = r
                            .headers()
                            .get("retry-after")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(|s| s.min(60))
                            .unwrap_or(5);
                        warn!(url, status = %status, retry_after, attempt, "rate limited, waiting");
                        retry_after
                    } else if status.is_server_error() {
                        let secs = backoff_secs[attempt as usize];
                        warn!(url, status = %status, wait_secs = secs, attempt, "server error, retrying");
                        secs
                    } else {
                        bail!("request to {} failed: HTTP {}", url, status);
                    };

                    tokio::time::sleep(Duration::from_secs(wait)).await;
                }
                Err(e) => {
                    if attempt == MAX_RETRIES {
                        bail!("request to {} failed after {} retries: {}", url, MAX_RETRIES, e);
                    }
                    let secs = backoff_secs[attempt as usize];
                    warn!(url, error = %e, wait_secs = secs, attempt, "request error, retrying");
                    tokio::time::sleep(Duration::from_secs(secs)).await;
                }
            }
        }

        unreachable!()
    }
}
