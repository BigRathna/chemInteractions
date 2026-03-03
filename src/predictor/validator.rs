use crate::predictor::rule_brain::matcher;

pub struct Validator;

impl Validator {
    /// Validates if the predicted product is chemically plausible given the reactants
    /// and a matched rule.
    pub fn validate_reaction(
        _reactants: &[String],
        product_smiles: &str,
        rule_name: &str,
    ) -> bool {
        let product_groups = matcher::detect_functional_groups(product_smiles);
        
        match rule_name {
            "Fischer Esterification" => {
                // Product must contain an ester group
                product_groups.contains(&"ester".to_string())
            },
            "Amide Bond Formation" => {
                // Product must contain an amide group
                product_groups.contains(&"amide".to_string())
            },
            "Alcohol Oxidation (Primary to Acid)" => {
                // Product must contain a carboxylic acid group
                product_groups.contains(&"carboxylic_acid".to_string())
            },
            _ => true, // Default to true for unknown rules for now
        }
    }
}
