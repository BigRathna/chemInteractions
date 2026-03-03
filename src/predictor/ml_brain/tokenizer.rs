use tokenizers::Tokenizer;
use crate::error::AppError;

pub struct SmilesTokenizer {
    inner: Tokenizer,
}

impl SmilesTokenizer {
    pub fn from_file(path: &str) -> Result<Self, AppError> {
        let inner = Tokenizer::from_file(path)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to load tokenizer: {}", e)))?;
        Ok(Self { inner })
    }

    pub fn encode(&self, smiles: &str) -> Result<Vec<u32>, AppError> {
        let encoding = self.inner.encode(smiles, true)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Tokenization error: {}", e)))?;
        Ok(encoding.get_ids().to_vec())
    }

    pub fn decode(&self, ids: &[u32]) -> Result<String, AppError> {
        let decoded = self.inner.decode(ids, true)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("De-tokenization error: {}", e)))?;
        Ok(decoded)
    }

    pub fn get_vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }

    pub fn id_to_token(&self, id: u32) -> Option<String> {
        self.inner.id_to_token(id)
    }

    pub fn token_to_id(&self, token: &str) -> Option<u32> {
        self.inner.token_to_id(token)
    }
}
