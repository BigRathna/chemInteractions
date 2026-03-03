use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Compound {
    pub name: String,
    pub smiles: String,
    pub formula: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactionRule {
    pub id: String,
    pub name: String,
    pub reaction_type: String,
    pub category: String,
    pub reactant_classes: Vec<String>,
    pub conditions_favored: Vec<String>,
    pub conditions_inhibited: Vec<String>,
    pub byproducts: Vec<Byproduct>,
    pub hazards: Vec<String>,
    pub kb_probability_modifier: f32,
    pub mechanism_summary: String,
    pub references: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Byproduct {
    pub smiles: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionalGroup {
    pub id: String,
    pub name: String,
    pub smarts: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PredictionRequest {
    pub reactants: Vec<String>,
    pub conditions: Option<ConditionsInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ConditionsInput {
    Raw(String),
    Structured(Conditions),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PredictionResponse {
    pub reaction_name: String,
    pub probability: f32,
    pub products: Vec<Compound>,
    pub byproducts: Vec<Byproduct>,
    pub confidence_tier: ConfidenceTier,
    pub explanation: String,
    pub mechanism: Option<String>,
    pub references: Vec<String>,
    pub ml_raw: Option<String>,
    pub kb_match: Option<KbMatch>,
    pub reactant_groups: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KbMatch {
    pub rule_id: String,
    pub name: String,
    pub kb_score: f32,
    pub conditions_score: f32,
    pub byproducts: Vec<Byproduct>,
    pub hazards: Vec<String>,
    pub mechanism_summary: String,
    pub references: Vec<String>,
    pub reactant_classes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Conditions {
    pub temperature: Option<f32>,
    pub ph: Option<f32>,
    pub catalyst: Option<String>,
    pub raw_input: Option<String>,
}

impl Conditions {
    pub fn to_tokens(&self) -> Vec<String> {
        let mut tokens = Vec::new();
        if let Some(cat) = &self.catalyst {
            let cat_lower = cat.to_lowercase();
            if cat_lower.contains("h2so4") || cat_lower.contains("hcl") || cat_lower.contains("acid") {
                tokens.push("acid_catalyst".to_string());
            }
            if cat_lower.contains("naoh") || cat_lower.contains("koh") || cat_lower.contains("base") {
                tokens.push("strong_base".to_string());
            }
        }
        if let Some(temp) = self.temperature {
            if temp > 100.0 {
                tokens.push("heat".to_string());
                tokens.push("reflux".to_string());
            }
        }
        tokens
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ConfidenceTier {
    High,
    Medium,
    Low,
    Experimental,
    MlPredicted,
    RuleVerified,
}
