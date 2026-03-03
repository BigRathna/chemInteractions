use axum::{routing::{get, post}, Router};
use crate::AppState;

pub mod predict;
pub mod compound;
pub mod reactions;

pub fn routes() -> Router<std::sync::Arc<AppState>> {
    Router::new()
        .route("/predict", post(predict::handler))
        .route("/compound/:name", get(compound::handler))
        .route("/reactions", get(reactions::list_handler))
        .route("/reactions/:id", get(reactions::get_handler))
}
