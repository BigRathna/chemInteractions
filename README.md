# ChemInteractions [v0.1.0-ALPHA]

A high-performance, Rust-based chemical interaction engine fusing **ReactionT5 Neural Networks** with **Rule-Based Chemical Logic**.

---

## 🚀 Quick Start

### 1. Requirements

- **Rust** (Stable or Nightly)
- **ONNX Model**: Ensure the ReactionT5 ONNX model is located at the path specified in your `.env` (default: `./models/reaction_t5.onnx`).
- **Knowledge Base**: Seed data is stored in `./knowledge_base/`.

### 2. Environment Setup

Create a `.env` file in the root directory (one should already exist if you've been following the build):

```bash
DATABASE_URL=sqlite:data/chem.db
MODEL_PATH=./models/reaction_t5.onnx
PUBCHEM_API=https://pubchem.ncbi.nlm.nih.gov/rest/pug
```

### 3. Run the Server

```bash
cargo run --bin chem-interactions
```

_Port: `8080` by default._

---

## 🧪 Using the Scientific UI

Once the server is running, navigate to:
👉 **[http://localhost:8080](http://localhost:8080)**

### Sample Interaction: Fischer Esterification

1.  **Reactant 01**: Enter `CC(=O)O` (Acetic Acid)
2.  **Add Reactant**: Click the button.
3.  **Reactant 02**: Enter `CCO` (Ethanol)
4.  **Temperature**: Set to `110` (Celsius)
5.  **Catalyst**: Enter `H2SO4`
6.  **Execute**: Click "**Execute Prediction Pipeline**"

**What to expect**:

- The system will match the **Rule Brain** (Fischer Esterification).
- It will verify the **ML Prediction** (Ethyl Acetate).
- It will annotate **Water** as a byproduct.
- It will provide a **Mechanism Summary**.

---

## 📡 API Reference

### POST `/api/predict`

Predicts interaction between reactants.

```bash
curl -X POST http://localhost:8080/api/predict \
     -H "Content-Type: application/json" \
     -d '{
       "reactants": ["CC(=O)O", "CCO"],
       "conditions": "reflux with H2SO4"
     }'
```

### GET `/api/compounds?q=<QUERY>`

Resolves compound names to SMILES and Formula via PubChem.

```bash
curl "http://localhost:8080/api/compounds?q=Aspirin"
```

---

## 🛠 Project Structure

- **/frontend**: "Supreme Scientific" minimalist dashboard.
- **/src/predictor**: Fusion engine, Rule brain, and ML inference.
- **/knowledge_base**: JSON-based textbook reaction rules.
- **/tests**: Comprehensive integration test suite.
