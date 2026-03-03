use chem_interactions::predictor::ml_brain::engine::MlEngine;
use chem_interactions::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    println!("Loading ML Engine from: {}", config.model_path);
    
    let engine = MlEngine::load(&config.model_path)?;
    
    let input = "CC(=O)O.OCC"; // Acetic acid + Ethanol
    println!("Predicting for: {}", input);
    
    let candidates = engine.predict(input).await?;
    
    for (i, cand) in candidates.iter().enumerate() {
        println!("  {}. SMILES: {} (Confidence: {:.4})", i + 1, cand.smiles, cand.confidence);
    }
    
    Ok(())
}
