use axum::{extract::State, Json};
use std::sync::Arc;
use crate::AppState;
use crate::{models::types::{PredictionRequest, PredictionResponse, Conditions}, error::AppError};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PredictionRequest>,
) -> Result<Json<PredictionResponse>, AppError> {
    // 1. Parse conditions if raw_input provided (Simple heuristic)
    let mut conditions = Conditions {
        temperature: None,
        ph: None,
        catalyst: None,
        raw_input: payload.conditions.clone(),
    };

    if let Some(raw) = &payload.conditions {
        let raw_lower = raw.to_lowercase();
        
        // Temperature extraction
        if raw_lower.contains("reflux") {
            conditions.temperature = Some(110.0);
        } else if let Some(m) = regex::Regex::new(r"(\d+)\s*°c").ok().and_then(|re| re.captures(&raw_lower)) {
            conditions.temperature = m.get(1).and_then(|s| s.as_str().parse().ok());
        } else if raw_lower.contains("heat") {
            conditions.temperature = Some(60.0);
        }

        // pH extraction
        if let Some(m) = regex::Regex::new(r"ph\s*(\d+\.?\d*)").ok().and_then(|re| re.captures(&raw_lower)) {
            conditions.ph = m.get(1).and_then(|s| s.as_str().parse().ok());
        }

        // Catalyst extraction
        if raw_lower.contains("h2so4") {
            conditions.catalyst = Some("H2SO4".to_string());
        } else if raw_lower.contains("pd/c") {
            conditions.catalyst = Some("Pd/C".to_string());
        }
    }

    // 2. Perform Fusion Prediction
    let result = state.fusion_engine.predict(&payload.reactants, &conditions).await?;
    
    Ok(Json(result))
}
