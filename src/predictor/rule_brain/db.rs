use sqlx::SqlitePool;
use crate::models::types::{KbMatch, Conditions};
use crate::error::AppError;

pub async fn find_matching_rules(
    pool: &SqlitePool,
    detected_groups: &[String],
    conditions: &Conditions,
) -> Result<Vec<KbMatch>, AppError> {
    // 1. Load all rules from DB
    let rows = sqlx::query!(
        r#"SELECT 
            id, name, reaction_type, category, reactant_classes, 
            conditions_favored, conditions_inhibited, byproducts, 
            hazards, kb_probability_modifier, mechanism_summary, "references"
        FROM reaction_rules"#
    ).fetch_all(pool).await?;

    let mut matches = Vec::new();
    let user_tokens = conditions.to_tokens();

    for row in rows {
        let reactant_classes: Vec<String> = serde_json::from_str(&row.reactant_classes)?;

        // Match: ALL reactant classes must be present in detected groups
        let all_present = reactant_classes.iter().all(|rc| detected_groups.contains(rc));
        if !all_present { continue; }

        let favored: Vec<String> = serde_json::from_str(&row.conditions_favored)?;
        let inhibited: Vec<String> = serde_json::from_str(&row.conditions_inhibited)?;

        // Score conditions
        let score = calculate_score(&favored, &inhibited, &user_tokens);
        if score < 0.0 { continue; }

        let kb_score = (row.kb_probability_modifier as f32) * score;

        matches.push(KbMatch {
            rule_id: row.id.unwrap(),
            name: row.name,
            kb_score,
            conditions_score: score,
            byproducts: serde_json::from_str(&row.byproducts)?,
            hazards: serde_json::from_str(&row.hazards)?,
            mechanism_summary: row.mechanism_summary,
            references: serde_json::from_str(&row.references)?,
            reactant_classes,
        });
    }

    matches.sort_by(|a, b| b.kb_score.partial_cmp(&a.kb_score).unwrap());
    Ok(matches)
}

fn calculate_score(favored: &[String], inhibited: &[String], user_tokens: &[String]) -> f32 {
    for inh in inhibited {
        if user_tokens.contains(inh) { return -1.0; }
    }

    if favored.is_empty() { return 1.0; }

    let matched = favored.iter().filter(|f| user_tokens.contains(f)).count();
    matched as f32 / favored.len() as f32
}

// Note: Using a trait or local struct for the row data to avoid macro limitations in score_conditions
#[allow(dead_code)]
struct RuleData {
    conditions_favored: String,
    conditions_inhibited: String,
}
