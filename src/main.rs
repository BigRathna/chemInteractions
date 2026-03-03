use chem_interactions::{api, config, db, predictor, AppState};
use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting ChemInteractions server...");

    let cfg = config::Config::from_env()?;

    let db = db::init_pool(&cfg.database_url).await?;
    db::seed::load_rules(&db, "knowledge_base/").await?;

    let ml_engine = predictor::ml_brain::engine::MlEngine::load(&cfg.model_path)?;
    let rule_brain = predictor::rule_brain::RuleBrain::new(db.clone());
    let pubchem = predictor::pubchem::PubChemClient::new(db.clone());
    
    let fusion_engine = Arc::new(predictor::fusion::FusionEngine::new(ml_engine, rule_brain, pubchem));

    let state = Arc::new(AppState { db, fusion_engine });

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
