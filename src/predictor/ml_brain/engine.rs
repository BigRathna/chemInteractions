use ort::session::{Session, builder::GraphOptimizationLevel};
use crate::error::AppError;
use super::tokenizer::SmilesTokenizer;

use tokio::sync::Mutex;

pub struct MlEngine {
    encoder: Mutex<Session>,
    decoder: Mutex<Session>,
    tokenizer: SmilesTokenizer,
}

impl MlEngine {
    pub fn load(model_dir: &str) -> Result<Self, AppError> {
        let tokenizer = SmilesTokenizer::from_file(&format!("{}/tokenizer.json", model_dir))?;

        let encoder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(format!("{}/encoder_model.onnx", model_dir))?;

        let decoder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(format!("{}/decoder_model.onnx", model_dir))?;

        Ok(Self {
            encoder: Mutex::new(encoder),
            decoder: Mutex::new(decoder),
            tokenizer,
        })
    }

    pub async fn predict(&self, smiles_input: &str) -> Result<Vec<MlCandidate>, AppError> {
        // ReactionT5 expects: "REACTANT:{smiles}REAGENT: "
        let formatted_input = format!("REACTANT:{}REAGENT: ", smiles_input);
        
        let input_ids_vec = self.tokenizer.encode(&formatted_input)?;
        let seq_len = input_ids_vec.len();
        
        use ort::value::Value;

        // 1. Encoder Pass
        let input_ids = input_ids_vec.iter().map(|&x| x as i64).collect::<Vec<_>>();
        let attention_mask = vec![1i64; seq_len];

        // Create values from raw data
        let input_ids_val = Value::from_array(([1usize, seq_len], input_ids))?;
        let attention_mask_val = Value::from_array(([1usize, seq_len], attention_mask))?;

        let (lhs_owned_shape, lhs_owned_data) = {
            let mut encoder = self.encoder.lock().await;
            let encoder_outputs = encoder.run(ort::inputs![
                "input_ids" => input_ids_val,
                "attention_mask" => attention_mask_val,
            ])?;
            
            let last_hidden_state_tensor = encoder_outputs["last_hidden_state"].try_extract_tensor::<f32>()?;
            let (lhs_shape, lhs_data) = last_hidden_state_tensor;
            (lhs_shape.iter().map(|&x| x as i64).collect::<Vec<i64>>(), lhs_data.to_vec())
        };

        // 2. Decoder Loop (Greedy)
        let mut decoder_input_ids = vec![0i64]; // Start with <pad> (0)
        let mut top_confidence = 0.0;
        let max_length = 128;

        for _ in 0..max_length {
            let current_dec_len = decoder_input_ids.len();
            let dec_input_val = Value::from_array(([1usize, current_dec_len], decoder_input_ids.clone()))?;
            let lhs_val = Value::from_array((lhs_owned_shape.clone(), lhs_owned_data.clone()))?;
            let enc_mask_val = Value::from_array(([1usize, seq_len], vec![1i64; seq_len]))?;

            let (next_token, logit_val) = {
                let mut decoder = self.decoder.lock().await;
                let decoder_outputs = decoder.run(ort::inputs![
                    "input_ids" => dec_input_val,
                    "encoder_hidden_states" => lhs_val,
                    "encoder_attention_mask" => enc_mask_val,
                ])?;

                let logits_tensor = decoder_outputs["logits"].try_extract_tensor::<f32>()?;
                let (logits_shape, logits_data) = logits_tensor;
                
                let logits_dims: Vec<usize> = logits_shape.iter().map(|&x| x as usize).collect();
                let vocab_size = logits_dims[2];
                let last_token_start = (current_dec_len - 1) * vocab_size;
                let last_token_slice = &logits_data[last_token_start..last_token_start + vocab_size];

                last_token_slice
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("Logits contain NaN"))
                    .map(|(i, &v)| (i as i64, v))
                    .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Logits empty")))?
            };

            if next_token == 1 { // </s> (1)
                break;
            }

            decoder_input_ids.push(next_token);
            top_confidence += logit_val;
        }

        // 3. Detokenize and Result
        let result_smiles = self.tokenizer.decode(&decoder_input_ids[1..].iter().map(|&x| x as u32).collect::<Vec<_>>())?;
        
        Ok(vec![MlCandidate {
            smiles: result_smiles,
            confidence: top_confidence,
            rank: 1,
        }])
    }

    fn normalize_beams(candidates: &mut Vec<MlCandidate>) {
        if candidates.is_empty() { return; }
        let total: f32 = candidates.iter().map(|c| c.confidence.exp()).sum();
        for c in candidates.iter_mut() {
            c.confidence = c.confidence.exp() / total;
        }
    }
}

pub struct MlCandidate {
    pub smiles: String,
    pub confidence: f32,
    pub rank: usize,
}
