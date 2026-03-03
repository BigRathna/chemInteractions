use axum::{extract::{Path, State}, Json};
use std::sync::Arc;
use crate::{AppState, models::types::ReactionRule, error::AppError};

pub async fn list_handler(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<ReactionRule>>, AppError> {
    Ok(Json(vec![]))
}

pub async fn get_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<ReactionRule>, AppError> {
    Err(AppError::NotFound(format!("Reaction rule {} not found", id)))
}
