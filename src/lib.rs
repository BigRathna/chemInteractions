pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod predictor;

use std::sync::Arc;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub fusion_engine: Arc<predictor::fusion::FusionEngine>,
}
