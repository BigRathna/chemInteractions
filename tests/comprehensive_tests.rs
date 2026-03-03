use chem_interactions::predictor::fusion::FusionEngine;
use chem_interactions::predictor::ml_brain::engine::{MlPredictor, MlCandidate};
use chem_interactions::predictor::rule_brain::RuleBrain;
use chem_interactions::predictor::pubchem::PubChemClient;
use chem_interactions::models::types::{Conditions, ConfidenceTier};
use chem_interactions::db;
use chem_interactions::error::AppError;
use std::sync::Arc;
use async_trait::async_trait;

struct MockMlEngine {
    expected_smiles: String,
}

#[async_trait]
impl MlPredictor for MockMlEngine {
    async fn predict(&self, _smiles_input: &str) -> Result<Vec<MlCandidate>, AppError> {
        Ok(vec![MlCandidate {
            smiles: self.expected_smiles.clone(),
            confidence: 0.95,
            rank: 1,
        }])
    }
}

async fn setup_test_engine(expected_smiles: &str) -> FusionEngine {
    let pool = db::init_pool("sqlite::memory:").await.unwrap();
    // Seed rules
    db::seed::load_rules(&pool, "knowledge_base/").await.unwrap();
    
    let ml_engine = Arc::new(MockMlEngine {
        expected_smiles: expected_smiles.to_string(),
    });
    let rule_brain = RuleBrain::new(pool.clone());
    let pubchem = PubChemClient::new(pool.clone());
    
    FusionEngine::new(ml_engine, rule_brain, pubchem)
}

#[tokio::test]
async fn test_fischer_esterification() {
    // Acetic Acid (CC(=O)O) + Ethanol (CCO) -> Ethyl Acetate (CC(=O)OCC)
    let engine = setup_test_engine("CC(=O)OCC").await;
    let reactants = vec!["CC(=O)O".to_string(), "CCO".to_string()];
    let conditions = Conditions {
        temperature: Some(70.0),
        ph: None,
        catalyst: Some("H2SO4".to_string()),
        raw_input: None,
    };
    
    let result = engine.predict(&reactants, &conditions).await.unwrap();
    
    assert_eq!(result.reaction_name, "Fischer Esterification");
    assert_eq!(result.confidence_tier, ConfidenceTier::RuleVerified);
    assert!(result.explanation.contains("Verified by Fischer Esterification rule"));
    assert!(result.byproducts.iter().any(|b| b.name == "Water"));
}

#[tokio::test]
async fn test_amide_formation() {
    // Acetic Acid (CC(=O)O) + Methylamine (CN) -> N-methylacetamide (CC(=O)NC)
    let engine = setup_test_engine("CC(=O)NC").await;
    let reactants = vec!["CC(=O)O".to_string(), "CN".to_string()];
    let conditions = Conditions {
        temperature: None,
        ph: None,
        catalyst: None,
        raw_input: None,
    };
    
    let result = engine.predict(&reactants, &conditions).await.unwrap();
    
    assert_eq!(result.reaction_name, "Amide Bond Formation");
    assert_eq!(result.confidence_tier, ConfidenceTier::RuleVerified);
    assert!(result.explanation.contains("Verified by Amide Bond Formation rule"));
}

#[tokio::test]
async fn test_ml_only_prediction() {
    // Some random reaction that doesn't match a rule
    let engine = setup_test_engine("C1CCCCC1").await; 
    let reactants = vec!["C=C".to_string(), "C=C=C".to_string()]; // Random
    let conditions = Conditions::default();
    
    let result = engine.predict(&reactants, &conditions).await.unwrap();
    
    assert_eq!(result.confidence_tier, ConfidenceTier::MlPredicted);
    assert!(result.explanation.contains("Predicted by ReactionT5 Transformer model"));
}

#[tokio::test]
async fn test_functional_group_verification_failure() {
    // Rule matches (e.g. Esterification), but ML predicts something without an ester
    let engine = setup_test_engine("CCCC").await; // Not an ester
    let reactants = vec!["CC(=O)O".to_string(), "CCO".to_string()];
    let conditions = Conditions::default();
    
    let result = engine.predict(&reactants, &conditions).await.unwrap();
    
    assert_eq!(result.confidence_tier, ConfidenceTier::Medium);
    assert!(result.explanation.contains("functional group verification failed"));
}
