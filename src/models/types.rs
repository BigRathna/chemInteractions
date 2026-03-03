use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Compound {
    pub name: String,
    pub smiles: String,
    pub formula: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactionRule {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub reactant_classes: Vec<String>,
    pub products: Vec<String>,
    pub byproducts: Vec<String>,
    pub citation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PredictionRequest {
    pub reactants: Vec<String>,
    pub conditions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PredictionResponse {
    pub reaction_name: String,
    pub probability: f32,
    pub products: Vec<Compound>,
    pub byproducts: Vec<String>,
    pub confidence_tier: ConfidenceTier,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConfidenceTier {
    Corroborated,
    MlPredicted,
    RuleBased,
}
