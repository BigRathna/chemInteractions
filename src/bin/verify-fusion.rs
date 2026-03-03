use chem_interactions::predictor::fusion::FusionEngine;
use chem_interactions::predictor::ml_brain::engine::MlEngine;
use chem_interactions::predictor::rule_brain::RuleBrain;
use chem_interactions::predictor::pubchem::PubChemClient;
use chem_interactions::models::types::Conditions;
use chem_interactions::config::Config;
use chem_interactions::db;
use dotenvy::dotenv;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let cfg = Config::from_env()?;
    
    println!("--- Initializing Fusion Pipeline ---");
    let pool = db::init_pool(&cfg.database_url).await?;
    let ml_engine = MlEngine::load(&cfg.model_path)?;
    let rule_brain = RuleBrain::new(pool.clone());
    let pubchem = PubChemClient::new(pool.clone());
    
    let fusion = FusionEngine::new(ml_engine, rule_brain, pubchem);

    // Test Case: Fischer Esterification (Acetic Acid + Ethanol)
    let reactants = vec!["CC(=O)O".to_string(), "CCO".to_string()];
    let conditions = Conditions {
        temperature: Some(110.0),
        ph: None,
        catalyst: Some("H2SO4".to_string()),
        raw_input: Some("reflux with H2SO4".to_string()),
    };

    println!("\n>>> Testing Fusion: Acetic Acid + Ethanol");
    let response = fusion.predict(&reactants, &conditions).await?;

    println!("\nRESULT:");
    println!("Reaction Name: {}", response.reaction_name);
    println!("Confidence: {:.4} ({:?})", response.probability, response.confidence_tier);
    println!("Product SMILES: {}", response.products[0].smiles);
    println!("Product Name: {}", response.products[0].name);
    println!("Product Formula: {}", response.products[0].formula.as_deref().unwrap_or("N/A"));
    println!("Explanation: {}", response.explanation);
    
    if let Some(mech) = response.mechanism {
        println!("\nMECHANISM SUMMARY:\n{}", mech);
    }

    if !response.byproducts.is_empty() {
        println!("\nBYPRODUCTS:");
        for b in response.byproducts {
            println!(" - {} ({})", b.name, b.smiles);
        }
    }

    println!("\nREFERENCES:");
    for r in response.references {
        println!(" - {}", r);
    }

    Ok(())
}
