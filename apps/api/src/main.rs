//! HTTP entry point.
//!
//! Responsibilities (and only these):
//!   1. Load configuration from the environment.
//!   2. Open a Postgres connection pool.
//!   3. Run pending migrations.
//!   4. Assemble the router from the feature modules.
//!   5. Start serving.
//!
//! Everything else lives in a feature module.

mod auth;
mod config;
mod error;
mod news;
mod notes;
mod state;

use axum::{Json, Router, routing::get};
use chrono::Utc;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Config::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("connected to postgres");

    sqlx::migrate!("./migrations").run(&db).await?;
    tracing::info!("migrations up to date");

    let addr = config.server_addr.clone();
    let state = AppState::new(db, config);

    let app = Router::new()
        .route("/health", get(health))
        .merge(auth::router())
        .merge(notes::router())
        .merge(news::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    timestamp: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "crypto_backend",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: Utc::now().to_rfc3339(),
    })
}
