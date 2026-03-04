# ChemInteractions [v0.2.0-BETA]

A high-performance, Rust-based chemical interaction engine fusing **ReactionT5 Neural Networks** with **Rule-Based Chemical Logic**. Optimized for deployment via Docker and verified for high-accuracy standard organic reactions.

---

## 🚀 Quick Start (Docker - Recommended)

The easiest way to run the full stack (Backend + Frontend + Database) is using Docker Compose.

### 1. Requirements

- **Docker** and **Docker Compose** installed.
- (Pre-configured) The project includes a pre-seeded SQLite database and the ReactionT5 ONNX model.

### 2. Stand up the Services

```bash
# Build and start in detached mode
docker compose up -d
```

### 3. Access the Dashboard

Navigate to: 👉 **[http://localhost:8080](http://localhost:8080)**

---

## 🛠 Local Development Setup

If you prefer to run natively on your machine:

### 1. Requirements

- **Rust** 1.92+
- **SQLite3**
- **ONNX Runtime**: Managed by the `ort` crate.

### 2. Prepare the Database & SQLx

```bash
# Install sqlx-cli if you haven't
cargo install sqlx-cli --no-default-features --features sqlite

# Initialize the offline query cache (required for compilation)
cargo sqlx prepare -- --lib
```

### 3. Run the Server

```bash
cargo run --release --bin chem-interactions
```

---

## 🧪 Feature Examples

### 1. Scientific Input (SMILES/Names)

| Reactant A                   | Reactant B      | Conditions      | Expected Result                              |
| :--------------------------- | :-------------- | :-------------- | :------------------------------------------- |
| `CC(=O)O` (Acetic Acid)      | `CCO` (Ethanol) | `reflux, H2SO4` | **Fischer Esterification** → Ethyl Acetate   |
| `NC1=CC=C([N+](=O)[O-])C=C1` | (None)          | `H2, Pd/C`      | **Nitro Reduction** → p-Phenylenediamine     |
| `C1=CC=CC=C1` (Benzene)      | `ClCC`          | `AlCl3`         | **Friedel-Crafts Alkylation** → Ethylbenzene |

### 2. Natural Language Input

The engine uses a heuristic parser to extract reactants and conditions from text:

- **Query**: "React Acetic Acid with Ethanol under reflux with an acid catalyst."
- **Parse**: Reactants: `["Acetic Acid", "Ethanol"]`, Conditions: `{ heat: true, catalyst: "Acid" }`.

### 3. API Reference

#### **POST** `/api/predict`

```bash
curl -X POST http://localhost:8080/api/predict \
     -H "Content-Type: application/json" \
     -d '{
       "reactants": ["CC(=O)O", "CCO"],
       "conditions": "reflux with H2SO4"
     }'
```

#### **GET** `/api/compound/:name`

Resolves metadata from PubChem or Local Cache.

```bash
curl http://localhost:8080/api/compound/Aspirin
```

#### **GET** `/api/reactions`

Lists all available textbook rules in the Rule Brain.

```bash
curl http://localhost:8080/api/reactions
```

---

## 🛠 Project Structure

- **/frontend**: "Supreme Scientific" minimalist dashboard (Vanilla JS/CSS).
- **/src/predictor**: Fusion engine, Rule brain (textbook-based), and ML brain (ReactionT5).
- **/knowledge_base**: JSON-based textbook reaction rules (Fischer, Grignard, etc.).
- **/models**: ONNX weights for ReactionT5.
- **/data**: Persistent SQLite storage.

---

## 🏗 CI/CD & Dockerization

The project uses a multi-stage Docker build based on **Ubuntu 24.04** to satisfy the glibc 2.38+ requirements of the `ort` ONNX runtime. Offline SQLx compilation is enabled via `.sqlx` metadata.
