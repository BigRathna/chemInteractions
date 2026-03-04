# ChemInteractions — Technical Documentation

> A complete technical breakdown of how ChemInteractions works — from user input to predicted reaction output.

---

## 1 — What It Is

ChemInteractions is a **chemical reaction prediction engine** built entirely in **Rust**. It takes two (or more) chemical reactants and reaction conditions as input, and predicts:

- The **product molecules** (as SMILES strings and IUPAC names)
- The **reaction class** (e.g. Fischer Esterification, Nitro Reduction)
- A **confidence tier** explaining how much the system trusts the prediction
- **Byproducts**, **mechanism summaries**, and **textbook references**

It does this by running two independent analysis systems and **fusing their outputs** — hence the name **Fusion Engine**.

---

## 2 — What Makes It Unique

### 2.1 Dual-Brain Fusion Architecture

This is not a pure ML predictor or a pure rule engine. It is **both, running in parallel, and cross-verifying each other**:

| Brain             | How It Works                                                                                                               | Strength                                                             |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| **ML Brain**      | Runs a 1.2B-parameter ReactionT5 Transformer via ONNX                                                                      | Generalizes to novel reactions it has never seen rules for           |
| **Rule Brain**    | Matches functional groups against a knowledge base of textbook reaction rules                                              | Provides explainability, mechanism summaries, and textbook citations |
| **Fusion Engine** | Cross-references ML output against Rule matches, validates functional groups in the product, and assigns a confidence tier | Trustworthy + explainable predictions                                |

> **Key insight:** The ML Brain generates the raw prediction. The Rule Brain independently identifies what reaction _should_ happen based on functional groups. The Fusion Engine compares these two signals. If they agree AND the product contains the expected functional groups, confidence is boosted to `RuleVerified` (95%+). If they disagree, confidence drops to `Medium`. If no rule matches at all, it falls back to `MlPredicted`.

### 2.2 Native Rust ONNX Inference (No Python Runtime)

The entire ML inference pipeline runs natively in Rust via the `ort` crate (ONNX Runtime bindings). There is **zero Python at runtime**. Python is only used once — offline — to export the HuggingFace model to ONNX format. This means:

- No Python interpreter, no GIL, no virtualenv at runtime
- Sub-second cold-start on modern hardware
- Thread-safe, async-capable inference behind `tokio::sync::Mutex`

### 2.3 Pure-Rust SMILES Parsing

SMILES (Simplified Molecular-Input Line-Entry System) parsing is done via the `purr` crate — a **pure Rust** SMILES parser. No RDKit, no cheminformatics C libraries. The system parses SMILES into a molecular graph and walks atoms + bonds to detect functional groups (carboxylic acid, alcohol, amine, ester, ketone, etc.) entirely in Rust.

### 2.4 Heuristic Natural Language Input

Users can describe reactions in plain English. A regex + keyword heuristic parser extracts reactant names and conditions from free text — no LLM required for NLP parsing.

### 2.5 Live PubChem Integration with SQLite Caching

Product SMILES from the ML Brain are enriched with real-world metadata (IUPAC name, molecular formula, CID) by querying the **PubChem REST API**. Results are cached in SQLite to avoid redundant network calls.

---

## 3 — System Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                        FRONTEND (Vanilla JS)                       │
│   ┌──────────────────┐              ┌───────────────────────────┐  │
│   │  Scientific Mode │              │  Natural Language Mode    │  │
│   │  SMILES / Names  │              │  Free-text descriptions   │  │
│   │  + Conditions    │              │  (heuristic parsed)       │  │
│   └────────┬─────────┘              └─────────────┬─────────────┘  │
│            │           POST /api/predict           │               │
└────────────┼───────────────────────────────────────┼───────────────┘
             └───────────────────┬───────────────────┘
                                 ▼
┌────────────────────────────────────────────────────────────────────┐
│                     AXUM HTTP SERVER (Rust)                        │
│                                                                    │
│   /api/predict  ──────►  Predict Handler                           │
│   /api/compound/:name ►  PubChem Lookup                            │
│   /api/reactions ─────►  List KB Rules                             │
│                                                                    │
│   Static Files: /frontend/* ──► ServeDir                           │
└────────────────────────┬───────────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────────────────────┐
│                      FUSION ENGINE                                 │
│                                                                    │
│   ┌────────────────────┐      ┌────────────────────────────────┐   │
│   │     ML BRAIN       │      │       RULE BRAIN               │   │
│   │                    │      │                                │   │
│   │ 1. Tokenize SMILES │      │ 1. Parse SMILES → graph       │   │
│   │ 2. Encoder pass    │      │ 2. Walk atoms → detect groups │   │
│   │ 3. Greedy decode   │      │ 3. Match groups vs. DB rules  │   │
│   │ 4. Detokenize →    │      │ 4. Score conditions           │   │
│   │    product SMILES  │      │ 5. Rank matches               │   │
│   └────────┬───────────┘      └───────────────┬────────────────┘   │
│            │                                  │                    │
│            └──────────┬───────────────────────┘                    │
│                       ▼                                            │
│             ┌─────────────────────────────────────┐                │
│             │        CROSS-VERIFICATION           │                │
│             │                                     │                │
│             │  • Do reactant groups match rule?    │                │
│             │  • Does ML product contain expected  │                │
│             │    functional groups? (Validator)    │                │
│             │  • Assign ConfidenceTier:            │                │
│             │    RuleVerified / Medium / MlPredicted│               │
│             └─────────────────┬───────────────────┘                │
│                               ▼                                    │
│             ┌─────────────────────────────────────┐                │
│             │       ENRICHMENT (PubChem)           │                │
│             │  Resolve product SMILES → IUPAC name │               │
│             │  + formula via PubChem REST API       │               │
│             │  (with SQLite cache)                 │                │
│             └─────────────────────────────────────┘                │
└────────────────────────────────────────────────────────────────────┘
```

---

## 4 — Technical Breakdown (Start to Finish)

### Step 1: User Input

The frontend provides two input modes:

**Scientific Mode** — The user enters:

- One or more **reactant fields** (SMILES strings like `CC(=O)O` or compound names like `Acetic Acid`)
- **Temperature** (°C)
- **Catalyst** (e.g. `H2SO4`)

**Natural Language Mode** — The user types free text like:

> "React Acetic Acid with Ethanol under reflux with an acid catalyst"

Both modes POST to `/api/predict` with a JSON body:

```json
{
  "reactants": ["CC(=O)O", "CCO"],
  "conditions": "reflux with H2SO4"
}
```

The `conditions` field is polymorphic — it can be either a raw string or a structured `{ temperature, ph, catalyst }` object. This is handled via `#[serde(untagged)]` enum deserialization in Rust.

---

### Step 2: Condition Parsing (predict.rs)

The API handler normalizes the input:

1. **Structured conditions** → used directly
2. **Raw string conditions** → parsed via heuristics:
   - `"reflux"` → temperature = 110°C
   - `"heat"` → temperature = 60°C
   - Regex `(\d+)\s*°c` → exact temperature extraction
   - Keywords `h2so4`, `pd/c`, `acid` → mapped to catalyst strings
   - `ph\s*(\d+\.?\d*)` → pH value extraction

3. **Reactant extraction from NL text** — If the reactants array is empty (NL mode), the raw text is split on `with`, `and`, `+`, `,` delimiters. Common condition keywords (`reflux`, `heat`, `acid`, `base`, `h2so4`) are filtered out, leaving only reactant names.

---

### Step 3: ML Brain Inference (ml_brain/engine.rs)

The ML Brain wraps **ReactionT5** — a T5-based Transformer model fine-tuned on reaction prediction, exported to ONNX format.

**Model Architecture:**

- **Encoder**: `encoder_model.onnx` (~341 MB) — Encodes the reactant SMILES into a latent representation
- **Decoder**: `decoder_model.onnx` (~455 MB) — Autoregressively generates the product SMILES token by token
- **Tokenizer**: HuggingFace SentencePiece tokenizer for SMILES notation (`tokenizer.json`)

**Inference Pipeline:**

```
Input: "REACTANT:CC(=O)O.CCOREAGENT: "
                    │
                    ▼
         ┌──────────────────┐
         │   TOKENIZATION   │  SmilesTokenizer.encode()
         │   → [token_ids]  │  HuggingFace tokenizers crate
         └────────┬─────────┘
                  ▼
         ┌──────────────────┐
         │   ENCODER PASS   │  input_ids + attention_mask
         │   → hidden_state │  ONNX Session with Level3 optimization
         └────────┬─────────┘  4 intra-op threads
                  ▼
         ┌──────────────────────────────┐
         │   GREEDY DECODING LOOP       │  max 128 steps
         │                              │
         │   Start: [<pad>]             │
         │   Each step:                 │
         │     decoder(input_ids,       │
         │            hidden_states,    │
         │            attention_mask)   │
         │     → logits                 │
         │     argmax(last_token) →     │
         │       next_token             │
         │   Until: </s> (token 1)      │
         └────────┬─────────────────────┘
                  ▼
         ┌──────────────────┐
         │  DETOKENIZATION  │  SmilesTokenizer.decode()
         │  → product SMILES│  e.g. "CCOC(C)=O"
         └──────────────────┘
```

The output is one `MlCandidate { smiles, confidence, rank }`.

**Important detail:** The confidence is the sum of logit values across all decoding steps — not a normalized probability. The Fusion Engine later normalizes and clamps this.

---

### Step 4: Rule Brain Analysis (rule_brain/)

The Rule Brain works completely independently of the ML Brain.

#### 4a. Functional Group Detection (matcher.rs)

The system parses each reactant's SMILES string into a molecular graph using `purr::graph::Builder`. It then walks every atom and inspects its bonds + neighbors to detect functional groups:

| Group             | Detection Logic                                            |
| ----------------- | ---------------------------------------------------------- |
| `carboxylic_acid` | Carbon with C=O double bond AND C-OH single bond           |
| `alcohol`         | Oxygen with single bond to one carbon, not adjacent to C=O |
| `amine`           | Nitrogen not bonded to a C=O (to exclude amides)           |
| `ester`           | Carbon with C=O AND C-O-C pattern                          |
| `amide`           | Carbon with C=O AND C-N bond                               |
| `aldehyde`        | Carbon with C=O and fewer than 2 carbon neighbors          |
| `ketone`          | Carbon with C=O and 2+ carbon neighbors                    |
| `alkene`          | C=C double bond                                            |
| `alkyne`          | C≡C triple bond                                            |
| `nitrile`         | C≡N triple bond                                            |
| `nitro`           | Nitrogen bonded to 2+ oxygens                              |
| `halide`          | Carbon bonded to F, Cl, Br, or I                           |
| `aromatic_ring`   | Aromatic atom flag from purr parser                        |
| `ether`           | Oxygen single-bonded to 2 carbons, no adjacent C=O         |

This detection is done entirely by traversing the molecular graph — no SMARTS pattern matching library is used at runtime (those SMARTS patterns in `functional_groups.json` are stored for reference/future use).

#### 4b. Rule Matching (rule_brain/db.rs)

All reaction rules are loaded from the `reaction_rules` SQLite table. For each rule:

1. **Reactant class check**: ALL of the rule's `reactant_classes` (e.g. `["carboxylic_acid", "alcohol"]` for Fischer Esterification) must be present in the detected functional groups. If any are missing → skip.

2. **Condition scoring**: The user's conditions are tokenized (e.g. `H2SO4` → `acid_catalyst`, temperature > 100°C → `heat`, `reflux`). Then:
   - If any `conditions_inhibited` token matches → score = -1 → skip
   - Score = (number of matched `conditions_favored` tokens) / (total favored conditions)

3. **Ranking**: Rules are ranked by `kb_probability_modifier × condition_score`.

The top-ranked rule becomes the `KbMatch` with byproducts, hazards, mechanism summary, and textbook references.

---

### Step 5: Fusion & Cross-Verification (fusion.rs)

This is the core orchestration step. The Fusion Engine:

1. **Calls ML Brain** → gets `MlCandidate` (product SMILES + confidence)
2. **Calls Rule Brain** → gets ranked `Vec<KbMatch>`
3. **Resolves reactant names** to SMILES via PubChem (so functional group detection works even on plain-name inputs like "Ethanol")
4. **Detects functional groups** on all resolved reactants
5. **Builds the response** starting with ML Brain output as the base

If a Rule match exists:

- **Verify**: Do ALL of the rule's `reactant_classes` exist in the detected groups?
- **Validate** (via `Validator`): Does the ML-predicted product contain the expected functional groups for this reaction type?
  - Fischer Esterification → product must contain `ester`
  - Amide Bond Formation → product must contain `amide`
  - Alcohol Oxidation → product must contain `carboxylic_acid`

**Confidence Tier Assignment:**

| Tier           | Condition                                                                           | Probability                 |
| -------------- | ----------------------------------------------------------------------------------- | --------------------------- |
| `RuleVerified` | Rule matches AND functional group verification passes AND product validation passes | 0.95 + (ml_conf × 0.05)     |
| `Medium`       | Rule matches but verification or validation fails                                   | Raw ML confidence (clamped) |
| `MlPredicted`  | No rule matched at all                                                              | Raw ML confidence (clamped) |

6. **Generate explanation** via `Explainer` — different messages for each tier
7. **Annotate byproducts** from the matched rule
8. **Enrich product** via PubChem — resolve the ML-predicted SMILES to an IUPAC name and molecular formula

---

### Step 6: PubChem Enrichment (pubchem.rs)

The `PubChemClient` resolves molecules by querying the NIH PubChem REST API:

```
GET https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/{name}/property/CanonicalSMILES,MolecularFormula,IUPACName/JSON
GET https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/smiles/{smiles}/property/CanonicalSMILES,MolecularFormula,IUPACName/JSON
```

Results are cached in the `compounds_cache` SQLite table (keyed by `name_query` or `smiles`). On subsequent requests for the same molecule, the cache is hit and no network call is made.

---

### Step 7: API Response

The final `PredictionResponse` JSON returned to the frontend:

```json
{
  "reaction_name":  "Fischer Esterification",
  "probability":    0.9527,
  "products":       [{ "name": "ethyl acetate", "smiles": "CCOC(C)=O", "formula": "C4H8O2" }],
  "byproducts":     [{ "name": "Water", "smiles": "O" }],
  "confidence_tier": "RuleVerified",
  "explanation":    "Verified by Fischer Esterification rule. Mechanism: Protonation of the carbonyl...",
  "mechanism":      "Protonation of the carbonyl oxygen by H+...",
  "references":     ["McMurry Organic Chemistry, 9th Ed., Ch.21"],
  "ml_raw":         "CCOC(C)=O",
  "kb_match":       { "rule_id": "rxn_org_001", "name": "Fischer Esterification", ... },
  "reactant_groups": ["alcohol", "carboxylic_acid"]
}
```

---

### Step 8: Frontend Rendering (ui.js)

The frontend renders a **process timeline** showing all 4 stages of the pipeline:

1. **Reactant Analysis** — Shows detected functional groups
2. **Neural Inference** — Shows raw ML SMILES output
3. **Rule Verification** — Shows matched rule name (or `NO_RULE_MATCHED`)
4. **Fusion Output** — Final classified reaction

Each result card also displays:

- **Confidence badge** (color-coded: green = RuleVerified, blue = MlPredicted, yellow = Medium)
- Product name, SMILES, and formula
- Byproducts
- Explanation text
- Full mechanism summary

---

## 5 — Knowledge Base Structure

### Reaction Rules (`knowledge_base/organic.json`)

9 hand-coded textbook reactions covering:

| Reaction                          | Category              | Key Groups                |
| --------------------------------- | --------------------- | ------------------------- |
| Fischer Esterification            | Condensation          | carboxylic_acid + alcohol |
| Amide Bond Formation              | Condensation          | carboxylic_acid + amine   |
| Alcohol Oxidation                 | Redox                 | alcohol                   |
| Grignard Addition to Ketone       | Nucleophilic Addition | ketone                    |
| Nitrile Reduction                 | Reduction             | nitrile                   |
| Nitro Reduction                   | Reduction             | nitro                     |
| Nitrile Hydrolysis                | Hydrolysis            | nitrile                   |
| Ester Hydrolysis (Saponification) | Hydrolysis            | ester                     |
| Grignard Carbonation              | Nucleophilic Addition | grignard_reagent          |

Each rule specifies favored/inhibited conditions, byproducts, hazards, mechanism summaries, and textbook references.

### Functional Groups (`knowledge_base/functional_groups.json`)

10 functional groups with SMARTS patterns (stored for reference; detection is done via the Rust graph walker).

---

## 6 — Database Schema

4 SQLite tables:

| Table               | Purpose                                             |
| ------------------- | --------------------------------------------------- |
| `reaction_rules`    | Textbook reaction rules loaded from JSON on startup |
| `functional_groups` | Functional group definitions loaded on startup      |
| `compounds_cache`   | PubChem API response cache (name/SMILES → metadata) |
| `compounds`         | General compound storage (name, SMILES, formula)    |

The DB is initialized at startup by executing `schema.sql` (CREATE IF NOT EXISTS), then seeded from the `knowledge_base/` JSON files via `seed.rs`.

---

## 7 — Tech Stack Summary

| Layer          | Technology                                                                          |
| -------------- | ----------------------------------------------------------------------------------- |
| Language       | **Rust** (2021 edition)                                                             |
| Web Framework  | **Axum 0.7** + Tokio async runtime                                                  |
| ML Inference   | **ort 2.0** (ONNX Runtime) — native Rust bindings                                   |
| Tokenization   | **tokenizers 0.19** (HuggingFace) — SentencePiece for SMILES                        |
| SMILES Parsing | **purr 0.9** — pure Rust SMILES graph builder                                       |
| Database       | **SQLite** via sqlx 0.7 (async, compile-time checked queries)                       |
| HTTP Client    | **reqwest 0.12** (for PubChem API)                                                  |
| Model          | **ReactionT5** (sagawa/ReactionT5-product-prediction) — T5 Seq2Seq exported to ONNX |
| Frontend       | Vanilla **HTML/CSS/JS** — no framework                                              |
| Design         | "Supreme Scientific" dark UI — Inter + Fira Code, glassmorphism, cyber-cyan accent  |

---

## 8 — Model Export Pipeline

The ReactionT5 model is exported from HuggingFace to ONNX **once, offline** using `scripts/export_reactiont5.py`:

```python
from optimum.onnxruntime import ORTModelForSeq2SeqLM
model = ORTModelForSeq2SeqLM.from_pretrained("sagawa/ReactionT5-product-prediction", export=True)
model.save_pretrained("../models/")
```

This produces 3 ONNX files (~1.2 GB total):

- `encoder_model.onnx` (341 MB)
- `decoder_model.onnx` (455 MB)
- `decoder_with_past_model.onnx` (398 MB — KV-cache variant, not used at runtime)

Plus `tokenizer.json`, `config.json`, and generation configs.

---

## 9 — Confidence System Deep-Dive

The confidence system is the most important differentiator. Here's the full decision tree:

```
User submits reactants + conditions
        │
        ├──► ML Brain generates product SMILES + raw confidence
        │
        ├──► Rule Brain detects functional groups → matches rules
        │
        ▼
   ┌─────────────────────────────┐
   │   Any Rule Brain match?     │
   └──────┬──────────┬───────────┘
          │          │
         YES         NO
          │          │
          ▼          ▼
   ┌──────────┐  Tier = MlPredicted
   │ All rule │  Explanation = "Predicted by
   │ reactant │   ReactionT5. No matching
   │ classes  │   textbook rule found."
   │ present? │
   └──┬───┬───┘
      │   │
     YES  NO ──► Tier = Medium
      │         Explanation = "ML prediction
      ▼          matches rule structurally,
   ┌──────────┐  but functional group
   │ Product  │  verification failed."
   │ contains │
   │ expected │
   │ groups?  │
   │(Validator)│
   └──┬───┬───┘
      │   │
     YES  NO ──► Tier = Medium
      │
      ▼
  Tier = RuleVerified
  Probability = 0.95 + (ml_conf × 0.05)
  Explanation = "Verified by {rule_name} rule.
   Mechanism: {mechanism_summary}"
```

---

_End of technical documentation._
