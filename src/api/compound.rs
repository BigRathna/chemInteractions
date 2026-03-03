use axum::{extract::{Path, State}, Json};
use std::sync::Arc;
use crate::{AppState, models::types::Compound, error::AppError};

pub async fn handler(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Compound>, AppError> {
    // Placeholder logic
    Err(AppError::NotFound(format!("Compound {} not found", name)))
}
