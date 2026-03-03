use crate::error::AppError;
use purr::walk::Follower;

pub struct Parser;

impl Parser {
    pub fn validate_smiles(smiles: &str) -> bool {
        struct DummyFollower;
        impl Follower for DummyFollower {
            fn root(&mut self, _: purr::feature::AtomKind) {}
            fn extend(&mut self, _: purr::feature::BondKind, _: purr::feature::AtomKind) {}
            fn join(&mut self, _: purr::feature::BondKind, _: purr::feature::Rnum) { }
            fn pop(&mut self, _: usize) {}
        }
        let mut follower = DummyFollower;
        purr::read::read(smiles, &mut follower, None).is_ok()
    }

    pub fn extract_reactants(input: &str) -> Vec<String> {
        // Simple split by '.', '+', or whitespace for now
        input.split(|c| c == '.' || c == '+' || c == ' ')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    pub fn parse_raw_input(input: &str) -> Result<Vec<String>, AppError> {
        let reactants = Self::extract_reactants(input);
        if reactants.is_empty() {
            return Err(AppError::BadRequest("No reactants found in input".to_string()));
        }
        
        for smiles in &reactants {
            if !Self::validate_smiles(smiles) {
                // If it's not a valid SMILES, it might be a name that needs resolution later.
                // We'll let the resolver handle it, but we can log a warning if needed.
            }
        }
        
        Ok(reactants)
    }
}
