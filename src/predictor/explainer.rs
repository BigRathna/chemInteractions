pub struct Explainer;

impl Explainer {
    pub fn generate_explanation(
        reaction_name: &str,
        mechanism_summary: &str,
        confidence_tier: &str,
    ) -> String {
        match confidence_tier {
            "RuleVerified" => format!(
                "Verified by {} rule. Mechanism: {}",
                reaction_name, mechanism_summary
            ),
            "MlPredicted" => "Predicted by ReactionT5 Transformer model. No matching textbook rule found.".to_string(),
            "Medium" => format!(
                "ML prediction matches {} rule structurally, but functional group verification failed.",
                reaction_name
            ),
            _ => format!("Reaction identified as {}. {}", reaction_name, mechanism_summary),
        }
    }
}
