use moka::future::Cache;
use sqlx::PgPool;

/// Simple string-keyed cache for costly aggregates (e.g. metadata).
pub type AppCache = Cache<String, serde_json::Value>;

#[derive(Clone)]
#[allow(dead_code)] // cache provisioned for future use
pub struct AppState {
    pub pool: PgPool,
    pub cache: AppCache,
    pub dataset_version: String,
}

impl AppState {
    pub fn new(pool: PgPool, dataset_version: String) -> Self {
        let cache = Cache::builder()
            .max_capacity(1_000)
            .time_to_live(std::time::Duration::from_secs(300))
            .build();
        Self { pool, cache, dataset_version }
    }
}
