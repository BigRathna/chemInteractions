pub mod db;
pub mod matcher;

use sqlx::SqlitePool;
use crate::error::AppError;
use crate::models::types::{KbMatch, Conditions};

pub struct RuleBrain {
    pool: SqlitePool,
}

impl RuleBrain {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn predict(&self, reactants: &[String], conditions: &Conditions) -> Result<Vec<KbMatch>, AppError> {
        let mut all_detected_groups = Vec::new();
        for smiles in reactants {
            let groups = matcher::detect_functional_groups(smiles);
            all_detected_groups.extend(groups);
        }
        all_detected_groups.sort();
        all_detected_groups.dedup();

        db::find_matching_rules(&self.pool, &all_detected_groups, conditions).await
    }
}
