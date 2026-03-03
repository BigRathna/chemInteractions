# Phase 4 — API Design

## Goal

Expose the prediction engine via a clean, versioned REST API using `axum`.

---

## Endpoints

### 1. Predict Interaction

**`POST /api/v1/predict`**

Takes compounds and conditions, returns ranked possible reactions.

**Request:**

```json
{
  "reactants": ["acetic acid", "ethanol"],
  "conditions": {
    "temp": "reflux",
    "pH": null,
    "catalyst": "H2SO4",
    "solvent": null
  },
  "options": {
    "limit": 5,
    "min_probability": 0.05
  }
}
```

**Response:**

```json
{
  "request_id": "uuid-v4",
  "predictions": [
    {
      "rank": 1,
      "tier": "corroborated",
      "name": "Fischer Esterification",
      "probability": 0.87,
      "primary_products": [
        { "name": "Ethyl acetate", "smiles": "CCOC(=O)C", "formula": "C4H8O2" }
      ],
      "byproducts": [
        { "name": "Water", "smiles": "O" }
      ],
      "explanation": "### Mechanism: Fischer Esterification\n\nProtonation of...",
      "citations": ["McMurry Ch.21"],
      "alerts": []
    },
    { "rank": 2, "tier": "ml_predicted", ... }
  ],
  "meta": { "ml_version": "reactiont5-v1", "kb_rules_count": 156 }
}
```

---

### 2. Compound Lookup / Resolve

**`GET /api/v1/compound/resolve?q=acetone`**

Resolves name to SMILES/Formula using PubChem (with cache support).

**Response:**

```json
{
  "name": "acetone",
  "smiles": "CC(C)=O",
  "formula": "C3H6O",
  "cid": 180,
  "functional_groups": ["ketone"]
}
```

---

### 3. List Knowledge Base Rules

**`GET /api/v1/reactions`**

Returns all reaction types the system understands via its Rule Brain.

**Response:**

```json
{
  "rules": [
    {
      "id": "rxn_org_001",
      "name": "Fischer Esterification",
      "type": "organic"
    },
    { "id": "rxn_ino_005", "name": "Precipitation", "type": "inorganic" }
  ]
}
```

---

## Success/Error Handling

| Code | Meaning                                              |
| ---- | ---------------------------------------------------- |
| 200  | Success                                              |
| 400  | Invalid input (unparsable SMILES, missing reactants) |
| 404  | Compound not found (in resolve endpoint)             |
| 500  | Server error (DB connection, ML session failure)     |
| 503  | Model loading (startup phase)                        |

Errors return a structured JSON body:

```json
{ "error": "Invalid SMILES string provided", "code": "ERR_INVALID_SMILES" }
```

---

## Middlewares

1. **CORS**: Allow frontend origin to access API.
2. **Logging**: Trace requests with `tracing`.
3. **Tracing ID**: Injected `X-Request-ID` for debugging.
4. **Timeout**: 30s limit (prediction can take 1-2s).
5. **State**: Share `SqlitePool` and `ort::Session` via `Arc<AppState>`.

---

## Rust Structs (`src/api/predict.rs`)

```rust
#[derive(Deserialize)]
pub struct PredictRequest {
    pub reactants: Vec<String>,
    pub conditions: Option<Conditions>,
    pub options: Option<PredictOptions>,
}

#[derive(Serialize)]
pub struct PredictResponse {
    pub request_id: String,
    pub predictions: Vec<PredictionEntry>,
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PredictRequest>
) -> Result<Json<PredictResponse>, AppError> {
    // 1. Resolve reactants to SMILES via PubChemClient
    // 2. Detect functional groups
    // 3. Run ML Brain (ORT) + Rule Brain (SQL)
    // 4. Fuse & Annotate
    // 5. Return JSON
}
```

---

## Checklist

- [ ] All endpoints respond with correct JSON structure.
- [ ] Error handler avoids leaking server internals (Panics → 500).
- [ ] PubChem resolver is integrated into the predict flow.
- [ ] API is versioned (`/v1/`).
- [ ] Postman or `curl` collection created for testing.
- [ ] Documentation available via `GET /docs` (optional OpenAPI).
      village
