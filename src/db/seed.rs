use sqlx::SqlitePool;
use crate::error::AppError;

use std::fs;
use crate::models::types::{ReactionRule, FunctionalGroup};

pub async fn load_rules(pool: &SqlitePool, dir: &str) -> Result<(), AppError> {
    // 1. Load Functional Groups
    let fg_path = format!("{}/functional_groups.json", dir);
    if let Ok(content) = fs::read_to_string(&fg_path) {
        let groups: Vec<FunctionalGroup> = serde_json::from_str(&content)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse {}: {}", fg_path, e)))?;
        
        for fg in groups {
            sqlx::query!(
                "INSERT OR IGNORE INTO functional_groups (id, name, smarts) VALUES (?, ?, ?)",
                fg.id, fg.name, fg.smarts
            )
            .execute(pool)
            .await?;
        }
    }

    // 2. Load Reaction Rules
    for file in ["organic.json", "inorganic.json", "physical.json"] {
        let path = format!("{}/{}", dir, file);
        if let Ok(content) = fs::read_to_string(&path) {
            let rules: Vec<ReactionRule> = serde_json::from_str(&content)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse {}: {}", path, e)))?;

            for rule in rules {
                let reactant_classes = serde_json::to_string(&rule.reactant_classes)?;
                let conditions_favored = serde_json::to_string(&rule.conditions_favored)?;
                let conditions_inhibited = serde_json::to_string(&rule.conditions_inhibited)?;
                let byproducts = serde_json::to_string(&rule.byproducts)?;
                let hazards = serde_json::to_string(&rule.hazards)?;
                let references = serde_json::to_string(&rule.references)?;

                sqlx::query!(
                    r#"INSERT OR IGNORE INTO reaction_rules (
                        id, name, reaction_type, category, reactant_classes, 
                        conditions_favored, conditions_inhibited, byproducts, 
                        hazards, kb_probability_modifier, mechanism_summary, "references"
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                    rule.id, rule.name, rule.reaction_type, rule.category,
                    reactant_classes, conditions_favored, conditions_inhibited,
                    byproducts, hazards, rule.kb_probability_modifier,
                    rule.mechanism_summary, references
                )
                .execute(pool)
                .await?;
            }
        }
    }
    
    Ok(())
}
