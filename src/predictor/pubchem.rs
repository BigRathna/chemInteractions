use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PubChemResult {
    pub smiles: String,
    pub cid: i32,
    pub formula: Option<String>,
    pub iupac_name: Option<String>,
}

pub struct PubChemClient {
    client: Client,
    pool: SqlitePool,
    base_url: String,
}

impl PubChemClient {
    pub fn new(pool: SqlitePool) -> Self {
        let base_url = std::env::var("PUBCHEM_API")
            .unwrap_or_else(|_| "https://pubchem.ncbi.nlm.nih.gov/rest/pug".to_string());
        
        Self {
            client: Client::new(),
            pool,
            base_url,
        }
    }

    pub async fn resolve_molecule(&self, query: &str) -> Result<PubChemResult, AppError> {
        // 1. Check Cache (by name)
        if let Ok(Some(cached)) = self.get_from_cache(query, true).await {
            return Ok(cached);
        }

        // 2. Query PubChem
        let url = format!("{}/compound/name/{}/property/CanonicalSMILES,MolecularFormula,IUPACName/JSON", 
            self.base_url, query);
        
        let result = self.fetch_from_pubchem(&url).await?;

        // 3. Save to Cache
        self.save_to_cache(query, &result).await?;

        Ok(result)
    }

    pub async fn resolve_by_smiles(&self, smiles: &str) -> Result<PubChemResult, AppError> {
        // 1. Check Cache (by smiles)
        if let Ok(Some(cached)) = self.get_from_cache(smiles, false).await {
            return Ok(cached);
        }

        // 2. Query PubChem
        let url = format!("{}/compound/smiles/{}/property/CanonicalSMILES,MolecularFormula,IUPACName/JSON", 
            self.base_url, smiles);
        
        let result = self.fetch_from_pubchem(&url).await?;

        // 3. Save to Cache (using canonical SMILES as name_query for consistency)
        self.save_to_cache(&result.smiles, &result).await?;

        Ok(result)
    }

    async fn fetch_from_pubchem(&self, url: &str) -> Result<PubChemResult, AppError> {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(AppError::NotFound("Molecule not found in PubChem".to_string()));
        }

        let body: serde_json::Value = response.json().await?;
        let properties = body["PropertyTable"]["Properties"]
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| AppError::NotFound("No properties found for molecule".to_string()))?;

        Ok(PubChemResult {
            smiles: properties["CanonicalSMILES"].as_str().unwrap_or("").to_string(),
            cid: properties["CID"].as_i64().unwrap_or(0) as i32,
            formula: properties["MolecularFormula"].as_str().map(|s| s.to_string()),
            iupac_name: properties["IUPACName"].as_str().map(|s| s.to_string()),
        })
    }

    async fn get_from_cache(&self, query: &str, is_name: bool) -> Result<Option<PubChemResult>, AppError> {
        let row = if is_name {
            sqlx::query(
                "SELECT smiles, cid, formula, iupac_name FROM compounds_cache WHERE name_query = ?"
            )
            .bind(query)
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT smiles, cid, formula, iupac_name FROM compounds_cache WHERE smiles = ?"
            )
            .bind(query)
            .fetch_optional(&self.pool)
            .await?
        };

        if let Some(r) = row {
            use sqlx::Row;
            Ok(Some(PubChemResult {
                smiles: r.get("smiles"),
                cid: r.get::<Option<i32>, _>("cid").unwrap_or(0),
                formula: r.get("formula"),
                iupac_name: r.get("iupac_name"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn save_to_cache(&self, query: &str, result: &PubChemResult) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT OR REPLACE INTO compounds_cache (name_query, smiles, formula, iupac_name, cid) VALUES (?, ?, ?, ?, ?)",
            query, result.smiles, result.formula, result.iupac_name, result.cid
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
