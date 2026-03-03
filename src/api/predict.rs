use axum::{extract::State, Json};
use std::sync::Arc;
use crate::AppState;
use crate::{models::types::{PredictionRequest, PredictionResponse, Conditions, ConditionsInput}, error::AppError};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PredictionRequest>,
) -> Result<Json<PredictionResponse>, AppError> {
    // 1. Resolve Conditions and Reactants from Input
    let mut reactants = payload.reactants.clone();
    let conditions = match payload.conditions {
        Some(ConditionsInput::Structured(c)) => c,
        Some(ConditionsInput::Raw(raw)) => {
            let mut c = Conditions {
                temperature: None,
                ph: None,
                catalyst: None,
                raw_input: Some(raw.clone()),
            };
            let raw_lower = raw.to_lowercase();
            
            // Heuristic Reactant Extraction if empty
            if reactants.is_empty() {
                // Look for "React A with B" or "Mix A and B" or "A + B"
                let parts: Vec<String> = if raw_lower.contains("react") || raw_lower.contains("mix") {
                    let cleaned = raw_lower.replace("react", "").replace("mix", "").replace("with", "|").replace("and", "|").replace(",", "|").replace("+", "|");
                    cleaned.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
                } else {
                    raw_lower.split(|c| c == ',' || c == '+' || c == '&').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
                };

                // Filter out common condition keywords to guess reactants
                let condition_keywords = ["reflux", "heat", "acid", "base", "h2so4", "naoh", "koh", "pd/c"];
                for part in parts {
                    let p_low = part.to_lowercase();
                    // If the part is JUST a condition keyword, skip it. 
                    // If it contains " acid" or " base" as a suffix (like "Acetic Acid"), keep it.
                    let is_pure_condition = condition_keywords.iter().any(|&k| p_low == k);
                    if !is_pure_condition {
                        reactants.push(part);
                    }
                }
            }

            if raw_lower.contains("reflux") {
                c.temperature = Some(110.0);
            } else if let Some(m) = regex::Regex::new(r"(\d+)\s*°c").ok().and_then(|re| re.captures(&raw_lower)) {
                c.temperature = m.get(1).and_then(|s| s.as_str().parse().ok());
            } else if raw_lower.contains("heat") {
                c.temperature = Some(60.0);
            }

            if let Some(m) = regex::Regex::new(r"ph\s*(\d+\.?\d*)").ok().and_then(|re| re.captures(&raw_lower)) {
                c.ph = m.get(1).and_then(|s| s.as_str().parse().ok());
            }

            if raw_lower.contains("h2so4") {
                c.catalyst = Some("H2SO4".to_string());
            } else if raw_lower.contains("pd/c") {
                c.catalyst = Some("Pd/C".to_string());
            } else if raw_lower.contains("acid") {
                c.catalyst = Some("Acid Catalyst".to_string());
            }
            c
        },
        None => Conditions::default(),
    };

    // 2. Perform Fusion Prediction
    let result = state.fusion_engine.predict(&reactants, &conditions).await?;
    
    Ok(Json(result))
}
