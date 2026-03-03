use std::env;
pub struct Config {
    pub database_url: String,
    pub model_path: String,
    pub pubchem_api: String,
    pub ml_alpha: f32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok(); // Optional if we use dotenvy, but plans mention .env

        Ok(Config {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/knowledge_base.db".to_string()),
            model_path: env::var("MODEL_PATH").unwrap_or_else(|_| "models/".to_string()),
            pubchem_api: env::var("PUBCHEM_API").unwrap_or_else(|_| "https://pubchem.ncbi.nlm.nih.gov/rest/pug".to_string()),
            ml_alpha: env::var("ML_ALPHA")
                .unwrap_or_else(|_| "0.65".to_string())
                .parse::<f32>()
                .unwrap_or(0.65),
        })
    }
}
