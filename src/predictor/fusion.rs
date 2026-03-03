use crate::models::types::{PredictionResponse, Conditions, Compound, ConfidenceTier};
use crate::error::AppError;
use crate::predictor::ml_brain::engine::MlPredictor;
use crate::predictor::rule_brain::RuleBrain;
use crate::predictor::pubchem::PubChemClient;
use crate::predictor::rule_brain::matcher;
use crate::predictor::explainer::Explainer;
use crate::predictor::byproducts::Byproducts;
use crate::predictor::validator::Validator;
use std::sync::Arc;
use std::collections::HashSet;

pub struct FusionEngine {
    ml_brain: Arc<dyn MlPredictor>,
    rule_brain: RuleBrain,
    pubchem: PubChemClient,
}

impl FusionEngine {
    pub fn new(ml_brain: Arc<dyn MlPredictor>, rule_brain: RuleBrain, pubchem: PubChemClient) -> Self {
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

        // 2b. RESOLVE REACTANT NAMES TO SMILES
        // We try to resolve all reactants to SMILES for functional group detection
        let mut resolved_reactants = Vec::new();
        for r in reactants {
            if let Ok(metadata) = self.pubchem.resolve_by_name(r).await {
                resolved_reactants.push(metadata.smiles);
            } else {
                resolved_reactants.push(r.clone()); // Fallback to raw if not found
            }
        }

        // Detect functional groups once for the whole pipeline
        let reactant_groups: HashSet<String> = resolved_reactants.iter()
            .flat_map(|r| matcher::detect_functional_groups(r))
            .collect();

        // 3. Fusion Logic: Verify ML against Rules
        let mut final_response = PredictionResponse {
            reaction_name: "Unknown Reaction".to_string(),
            probability: (top_ml.confidence / 100.0).clamp(0.0, 0.99), // Pseudo-normalization for logit sums
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
            ml_raw: Some(top_ml.smiles.clone()),
            kb_match: None,
            reactant_groups: reactant_groups.iter().cloned().collect(),
        };

        if let Some(rule) = top_rule {
            final_response.kb_match = Some(rule.clone());
            final_response.reaction_name = rule.name.clone();
            final_response.mechanism = Some(rule.mechanism_summary.clone());
            final_response.references = rule.references.clone();
            final_response.byproducts = rule.byproducts.clone();

            let mut is_verified = rule.reactant_classes.iter().all(|rc| reactant_groups.contains(rc));
            
            if is_verified {
                if !Validator::validate_reaction(reactants, &final_response.products[0].smiles, &rule.name) {
                    is_verified = false;
                }
            }

            if is_verified {
                final_response.confidence_tier = ConfidenceTier::RuleVerified;
                final_response.probability = 0.95 + (final_response.probability * 0.05); // Boost for verified rules
            } else {
                final_response.confidence_tier = ConfidenceTier::Medium;
            }

            let tier_str = match final_response.confidence_tier {
                ConfidenceTier::RuleVerified => "RuleVerified",
                ConfidenceTier::Medium => "Medium",
                _ => "MlPredicted",
            };

            final_response.explanation = Explainer::generate_explanation(
                &rule.name,
                &rule.mechanism_summary,
                tier_str
            );
            
            final_response.byproducts = Byproducts::annotate(&rule.byproducts);
        }

        // 4. Enrich with PubChem metadata (Optional/Best Effort)
        if let Ok(metadata) = self.pubchem.resolve_by_smiles(&top_ml.smiles).await {
            final_response.products[0].name = metadata.iupac_name.unwrap_or_else(|| "Target Molecule".to_string());
            final_response.products[0].formula = metadata.formula;
        }

        Ok(final_response)
    }
}
