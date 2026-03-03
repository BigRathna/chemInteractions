use chem_interactions::predictor::rule_brain::RuleBrain;
use chem_interactions::models::types::Conditions;
use sqlx::SqlitePool;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&db_url).await?;
    let brain = RuleBrain::new(pool);

    // Test Case: Acetic Acid + Ethanol
    // Acetic Acid: CC(=O)O
    // Ethanol: CCO
    let reactants = vec!["CC(=O)O".to_string(), "CCO".to_string()];
    let conditions = Conditions {
        temperature: Some(110.0),
        ph: None,
        catalyst: Some("H2SO4".to_string()),
        raw_input: None,
    };

    println!("Predicting for: CC(=O)O + CCO (Acid cat, heat)");
    let matches = brain.predict(&reactants, &conditions).await?;

    println!("Found {} matches:", matches.len());
    for m in matches {
        println!(" - {}: score={:.2}", m.name, m.kb_score);
        println!("   Mechanism: {}", m.mechanism_summary);
        for b in m.byproducts {
            println!("   Byproduct: {} ({})", b.name, b.smiles);
        }
    }

    Ok(())
}
