use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod db;
mod error;
mod graphql;
mod handlers;
mod metrics;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Logging ───────────────────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ── Config ────────────────────────────────────────────────────────────────
    let cfg = Config::from_env().expect("failed to load config");

    // ── Metrics ───────────────────────────────────────────────────────────────
    let prom_handle = metrics::setup();

    // ── Database ──────────────────────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await
        .expect("failed to connect to database");

    info!("connected to database");

    // ── App state ─────────────────────────────────────────────────────────────
    let state = AppState::new(pool, cfg.expected_schema_version.to_string());

    // ── GraphQL schema ────────────────────────────────────────────────────────
    let gql_schema = graphql::schema::build(state.clone());

    // ── CORS ──────────────────────────────────────────────────────────────────
    let cors = {
        let origins = cfg.allowed_origins_list();
        if origins.is_empty() {
            CorsLayer::new()
        } else if origins.iter().any(|o| o == "*") {
            CorsLayer::new().allow_origin(Any)
        } else {
            let parsed: Vec<axum::http::HeaderValue> = origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect();
            CorsLayer::new().allow_origin(AllowOrigin::list(parsed))
        }
    };

    // ── Rate limiting (100 req/min burst per IP) ───────────────────────────────
    // 2 tokens/s replenishment → 120/min sustained; burst up to 100
    let governor_conf = {
        let mut b = GovernorConfigBuilder::default();
        b.per_second(2).burst_size(100);
        Arc::new(b.finish().expect("invalid governor config"))
    };

    // ── Sub-routers (each has its own state type) ─────────────────────────────
    let graphql_router = {
        let mut r = Router::new()
            .route("/graphql", post(handlers::graphql::graphql_handler));

        // Playground only in debug builds
        #[cfg(debug_assertions)]
        {
            r = r.route("/graphql", get(graphql_playground));
        }

        r.with_state(gql_schema)
    };

    let ready_router = Router::new()
        .route("/ready", get(handlers::ready::ready_handler))
        .with_state(state);

    let metrics_router = Router::new()
        .route("/metrics", get(metrics::metrics_handler))
        .with_state(prom_handle);

    // ── Assemble full app ─────────────────────────────────────────────────────
    let app = Router::new()
        .route("/health", get(handlers::health::health_handler))
        .merge(graphql_router)
        .merge(ready_router)
        .merge(metrics_router)
        .layer(GovernorLayer::new(governor_conf))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // ── Serve ─────────────────────────────────────────────────────────────────
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

#[cfg(debug_assertions)]
async fn graphql_playground() -> impl axum::response::IntoResponse {
    use async_graphql::http::GraphiQLSource;
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
