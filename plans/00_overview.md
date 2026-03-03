# ChemInteractions — System Overview

> Full-Rust, Neuro-Symbolic chemistry interaction predictor combining a ReactionT5 language model with a textbook-sourced knowledge base.

## What It Does

Takes chemical compounds + optional reaction conditions as input and returns:

- Ranked list of likely reactions with **probability scores**
- **Primary products** (SMILES + name + formula)
- **Byproducts** (from knowledge base annotation)
- **Confidence tier** (Corroborated / ML-Predicted / Rule-Based)
- **Mechanism explanation** with textbook citations
- **Hazard warnings** for dangerous reactions

---

## Architecture: Dual-Brain + Fusion

```
User Input (compounds + conditions)
        │
        ▼
  [Input Parser]  ──→  [PubChem Resolver]  (names → canonical SMILES)
        │
        ├──────────────────────────────────┐
        ▼                                  ▼
  [ML Brain]                         [Rule Brain]
  ReactionT5 (ONNX)                  Knowledge Base (SQLite)
  Predicts products from SMILES      Matches textbook reaction rules
  Returns beam candidates + conf.    Returns rules + modifiers + byproducts
        │                                  │
        └─────────────┬────────────────────┘
                      ▼
              [Fusion Layer]
              Weighted score merger
              Confidence tier assignment
              Physical constraint validation
                      │
                      ▼
        [Byproduct Annotator]  +  [Explainer]
                      │
                      ▼
              [REST API Response]
                      │
                      ▼
                 [Web UI]
```

---

## Confidence Tiers

| Tier            | Meaning                                                     |
| --------------- | ----------------------------------------------------------- |
| ✅ Corroborated | Both ML and KB agree — highest reliability                  |
| ⚠️ ML Predicted | Model predicted it, no KB match — novel/edge case           |
| 📖 Rule-Based   | KB matched it, model didn't predict — conservative textbook |

---

## Technology Stack

| Layer         | Technology                            |
| ------------- | ------------------------------------- |
| Web framework | Rust + `axum`                         |
| Database      | `sqlx` + SQLite                       |
| ML inference  | `ort` (ONNX Runtime for Rust)         |
| Tokenizer     | `tokenizers` (HuggingFace Rust crate) |
| HTTP client   | `reqwest` (PubChem API)               |
| Serialization | `serde` + `serde_json`                |
| Async         | `tokio`                               |
| Frontend      | HTML + Vanilla JS                     |

---

## ML Brain: ReactionT5 (2024 SOTA)

| Property       | Value                           |
| -------------- | ------------------------------- |
| Architecture   | T5 encoder-decoder transformer  |
| Parameters     | ~250M                           |
| Top-1 accuracy | **97.5%** on product prediction |
| Training data  | Open Reaction Database          |
| ONNX size      | ~1 GB (encoder + decoder)       |
| RAM usage      | ~1.5 GB                         |
| CPU inference  | 300ms – 1.2s                    |
| License        | Apache 2.0                      |

Also supports: retrosynthesis (71.0%) and yield prediction (R²=0.947).

---

## Rule Brain: Knowledge Base

- **~150+ reaction rules** from inorganic, organic, physical chemistry textbooks
- Each rule includes: reactant classes, favored/inhibited conditions, byproducts, mechanism summary, textbook citation
- Stored in SQLite, seeded from JSON files in `knowledge_base/`

---

## Fusion Formula

```
P_final = 0.65 × P_ml  +  0.35 × P_kb
```

- `P_ml` = ReactionT5 beam search confidence
- `P_kb` = KB rule modifier × conditions compatibility score
- α (0.65) is tunable

---

## Project Phases

| Phase | Focus                                                       |
| ----- | ----------------------------------------------------------- |
| 1     | Rust project, axum, SQLite schema, KB rules, PubChem lookup |
| 2     | ReactionT5 ONNX export, ort integration, beam search        |
| 3     | Fusion layer, physical validator, byproducts, explainer     |
| 4     | Web frontend                                                |
| 5     | Benchmarking, tests, safety warnings, docs                  |

---

## Plan Files Index

| File                         | Contents                                     |
| ---------------------------- | -------------------------------------------- |
| `00_overview.md`             | This file — system overview                  |
| `01_rust_project_setup.md`   | Cargo setup, axum skeleton, crate list       |
| `02_knowledge_base.md`       | KB schema, rule format, seeding              |
| `03_pubchem_integration.md`  | PubChem REST API usage in Rust               |
| `04_ml_brain.md`             | ReactionT5 export, ort inference, tokenizer  |
| `05_rule_brain.md`           | Functional group matching, rule scoring      |
| `06_fusion_layer.md`         | Score merger, tier logic, physical validator |
| `07_explainer_byproducts.md` | Byproduct annotation, mechanism generation   |
| `08_api_design.md`           | REST endpoints, request/response schemas     |
| `09_frontend.md`             | Web UI layout, components, API client        |
| `10_testing_validation.md`   | Test plan, benchmarks, known reactions       |
