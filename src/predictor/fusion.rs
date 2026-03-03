use crate::models::types::{PredictionResponse, Conditions, Compound, ConfidenceTier};
use crate::error::AppError;
use crate::predictor::ml_brain::engine::MlEngine;
use crate::predictor::rule_brain::RuleBrain;
use crate::predictor::pubchem::PubChemClient;
use crate::predictor::rule_brain::matcher;

pub struct FusionEngine {
    ml_brain: MlEngine,
    rule_brain: RuleBrain,
    pubchem: PubChemClient,
}

impl FusionEngine {
    pub fn new(ml_brain: MlEngine, rule_brain: RuleBrain, pubchem: PubChemClient) -> Self {
        Self { ml_brain, rule_brain, pubchem }
    }

    pub fn pubchem(&self) -> &PubChemClient {
        &self.pubchem
    }

    pub async fn predict(
        &self,
        reactants: &[String],
        conditions: &Conditions,
    ) -> Result<PredictionResponse, AppError> {
        // 1. Get ML predictions
        // ReactionT5 works on a single SMILES string "A.B.C"
        let ml_input = reactants.join(".");
        let ml_candidates = self.ml_brain.predict(&ml_input).await?;
        let top_ml = ml_candidates.first().ok_or_else(|| AppError::NotFound("No ML candidates found".to_string()))?;

        // 2. Get Rule matches
        let rule_matches = self.rule_brain.predict(reactants, conditions).await?;
        let top_rule = rule_matches.first();

        // 3. Fusion Logic: Verify ML against Rules
        let mut final_response = PredictionResponse {
            reaction_name: "Unknown Reaction".to_string(),
            probability: top_ml.confidence,
            products: vec![Compound {
                name: "Primary Product".to_string(),
                smiles: top_ml.smiles.clone(),
                formula: None,
            }],
            byproducts: vec![],
            confidence_tier: ConfidenceTier::MlPredicted,
            explanation: "Predicted by ReactionT5 Transformer model.".to_string(),
            mechanism: None,
            references: vec![],
        };

        if let Some(rule) = top_rule {
            final_response.reaction_name = rule.name.clone();
            final_response.mechanism = Some(rule.mechanism_summary.clone());
            final_response.references = rule.references.clone();
            final_response.byproducts = rule.byproducts.clone();

            // Verification: Does the ML product possess the functional groups expected by the rule?
            // e.g. For Fischer Esterification, does the product have an 'ester' group?
            let product_groups = matcher::detect_functional_groups(&top_ml.smiles);
            
            // Heuristic for verification:
            // If the rule category is 'condensation', and the rule is Fischer Esterification, 
            // check if the product is an ester.
            let is_verified = match rule.name.as_str() {
                "Fischer Esterification" => product_groups.contains(&"ester".to_string()),
                "Amide Bond Formation" => product_groups.contains(&"amide".to_string()),
                _ => false,
            };

            if is_verified {
                final_response.confidence_tier = ConfidenceTier::RuleVerified;
                final_response.explanation = format!(
                    "Verified by {} rule (Best Match). Mechanism: {}",
                    rule.name, rule.mechanism_summary
                );
            } else {
                final_response.confidence_tier = ConfidenceTier::Medium;
                final_response.explanation = format!(
                    "ML prediction matches {} rule structurally, but functional group verification failed.",
                    rule.name
                );
            }
        }

        // 4. Enrich with PubChem metadata (Optional/Best Effort)
        if let Ok(metadata) = self.pubchem.resolve_by_smiles(&top_ml.smiles).await {
            final_response.products[0].name = metadata.iupac_name.unwrap_or_else(|| "Target Molecule".to_string());
            final_response.products[0].formula = metadata.formula;
        }

        Ok(final_response)
    }
}
