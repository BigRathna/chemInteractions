use axum::{extract::{Path, State}, Json};
use std::sync::Arc;
use crate::{AppState, models::types::Compound, error::AppError};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Compound>, AppError> {
    let result = state.fusion_engine.pubchem().resolve_molecule(&name).await?;
    
    Ok(Json(Compound {
        name: result.iupac_name.unwrap_or(name),
        smiles: result.smiles,
        formula: result.formula,
    }))
}
