mod api;
mod config;
mod db;
mod error;
mod models;
mod predictor;

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub ml_engine: Arc<predictor::ml_brain::engine::MlEngine>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting ChemInteractions server...");

    let cfg = config::Config::from_env()?;

    let db = db::init_pool(&cfg.database_url).await?;
    db::seed::load_rules(&db, "knowledge_base/").await?;

    let ml_engine = Arc::new(predictor::ml_brain::engine::MlEngine::load(&cfg.model_path)?);

    let state = Arc::new(AppState { db, ml_engine });

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api", api::routes())
        .with_state(state);

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
