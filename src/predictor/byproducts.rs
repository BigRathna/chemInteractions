use crate::models::types::Byproduct;

pub struct Byproducts;

impl Byproducts {
    pub fn annotate(rule_byproducts: &[Byproduct]) -> Vec<Byproduct> {
        // For now, it just returns the byproducts defined in the rule.
        // In the future, this could be more dynamic based on the actual molecules.
        rule_byproducts.to_vec()
    }
}
