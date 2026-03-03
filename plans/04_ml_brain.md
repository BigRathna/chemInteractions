# Phase 2 — ML Brain: ReactionT5 via ONNX

## Goal

Export ReactionT5 from HuggingFace to ONNX, load it in Rust via the `ort` crate, run inference, and extract confidence scores.

---

## Step 1 — ONNX Export (One-Time Python Script)

### `scripts/requirements_export.txt`

```
transformers>=4.38.0
torch>=2.2.0
optimum[onnxruntime]>=1.18.0
onnx>=1.16.0
```

### `scripts/export_reactiont5.py`

```python
from optimum.onnxruntime import ORTModelForSeq2SeqLM
from transformers import AutoTokenizer
import os

MODEL_ID = "sagawa/ZINC-t5"         # ReactionT5 equivalent on HuggingFace
# Alternatives:
#   "sagawa/ReactionT5-yield"         # yield prediction variant
#   "zjunlp/MolGen-large"             # mol generation
#   "rxn4chemistry/rxnfp"             # reaction fingerprint

OUTPUT_DIR = "../models/"
os.makedirs(OUTPUT_DIR, exist_ok=True)

print("Exporting encoder and decoder to ONNX...")
model = ORTModelForSeq2SeqLM.from_pretrained(
    MODEL_ID,
    export=True,
    provider="CPUExecutionProvider"
)
tokenizer = AutoTokenizer.from_pretrained(MODEL_ID)

model.save_pretrained(OUTPUT_DIR)
tokenizer.save_pretrained(OUTPUT_DIR)
print(f"Saved to {OUTPUT_DIR}")
print("Files:")
for f in os.listdir(OUTPUT_DIR):
    size = os.path.getsize(f"{OUTPUT_DIR}/{f}") / 1e6
    print(f"  {f}  ({size:.1f} MB)")
```

### Run Export

```bash
cd scripts/
pip install -r requirements_export.txt
python export_reactiont5.py
# Output: models/encoder_model.onnx, models/decoder_model.onnx, models/tokenizer.json
```

---

## Step 2 — Rust ONNX Inference (`src/predictor/ml_brain/engine.rs`)

### How ReactionT5 Works at Inference

```
Input:  "CC(=O)O.CCO>>H+"     (reactants>>reagents in SMILES)
Encode  → token IDs (SentencePiece tokenizer)
        → encoder hidden states
Decode  → beam search, step by step, generate product SMILES tokens
Output: beam candidates with log-probabilities
        e.g. "CCOC(=O)C"  logprob=-0.09   → confidence 0.91
             "CC(=O)OCC"  logprob=-2.81   → confidence 0.06
```

### MlBrain Struct (pseudocode, actual in Rust)

```rust
pub struct MlEngine {
    session_encoder: ort::Session,
    session_decoder: ort::Session,
    tokenizer: tokenizers::Tokenizer,
    beam_width: usize,       // default 5
}

impl MlEngine {
    pub fn load(model_dir: &str) -> anyhow::Result<Self> {
        let env = Environment::builder().build()?.into_arc();
        let encoder = Session::builder(&env)?
            .with_model_from_file(format!("{}/encoder_model.onnx", model_dir))?;
        let decoder = Session::builder(&env)?
            .with_model_from_file(format!("{}/decoder_model.onnx", model_dir))?;
        let tokenizer = Tokenizer::from_file(format!("{}/tokenizer.json", model_dir))?;
        Ok(Self { session_encoder: encoder, session_decoder: decoder, tokenizer, beam_width: 5 })
    }

    pub async fn predict(&self, smiles_input: &str) -> anyhow::Result<Vec<MlCandidate>> {
        // 1. Tokenize input
        // 2. Run encoder
        // 3. Beam search decode
        // 4. Detokenize each beam
        // 5. Convert log-probs to confidence scores
        // Returns: Vec<MlCandidate { smiles: String, confidence: f32 }>
    }
}
```

### Confidence Score Extraction

```rust
// From beam search log-probability to normalized confidence
fn logprob_to_confidence(log_prob: f32) -> f32 {
    log_prob.exp().clamp(0.0, 1.0)
}

// Normalize across all beams so they sum to 1
fn normalize_beams(candidates: &mut Vec<MlCandidate>) {
    let total: f32 = candidates.iter().map(|c| c.raw_confidence).sum();
    for c in candidates.iter_mut() {
        c.confidence = c.raw_confidence / total;
    }
}
```

---

## Step 3 — SMILES Tokenizer (`src/predictor/ml_brain/tokenizer.rs`)

ReactionT5 uses a SentencePiece tokenizer trained on SMILES.

```rust
pub struct SmilesTokenizer {
    inner: tokenizers::Tokenizer,
}

impl SmilesTokenizer {
    pub fn from_file(path: &str) -> anyhow::Result<Self> { ... }

    pub fn encode(&self, smiles: &str) -> Vec<u32> {
        // tokenize SMILES characters
        // T5 uses "<pad>=0, </s>=1" convention
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        // convert token IDs back to SMILES string
    }
}
```

---

## Input Format

ReactionT5 expects reaction SMILES format:

```
reactant1.reactant2>reagent>
```

Examples:

```
CC(=O)O.CCO>OS(=O)(=O)O>          # acetic acid + ethanol + H2SO4
[Na+].[OH-].[H]Cl>>               # NaOH + HCl
C(Cl)>>                            # CH3Cl (for SN2 prediction)
```

The parser builds this from input SMILES + conditions:

```rust
fn build_input_smiles(reactants: &[String], reagents: &[String]) -> String {
    format!("{}>{}>", reactants.join("."), reagents.join("."))
}
```

---

## Output Model Types

```rust
pub struct MlCandidate {
    pub smiles: String,      // predicted product SMILES
    pub confidence: f32,     // normalized (0.0–1.0)
    pub rank: usize,
}

pub struct MlBrainResult {
    pub candidates: Vec<MlCandidate>,
    pub top_smiles: String,
    pub top_confidence: f32,
}
```

---

## Checklist

- [ ] `scripts/export_reactiont5.py` runs successfully
- [ ] `models/encoder_model.onnx` and `models/decoder_model.onnx` generated
- [ ] `ort` loads both sessions without error
- [ ] Tokenizer encodes/decodes sample SMILES correctly
- [ ] Beam search produces 5 candidates for test input
- [ ] Confidence scores sum to ~1.0
- [ ] HCl + NaOH test: top candidate includes NaCl SMILES with confidence ≥ 0.90
