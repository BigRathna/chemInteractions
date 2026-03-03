# Phase 1 — Rust Project Setup

## Goal

Scaffold the full Rust project with all dependencies declared, the `axum` server running, and health check endpoint live.

---

## Cargo.toml

```toml
[package]
name = "chem-interactions"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }

# HTTP client (PubChem)
reqwest = { version = "0.12", features = ["json"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# ONNX inference
ort = { version = "2", features = ["load-dynamic"] }

# Tokenizer (HuggingFace SentencePiece for ReactionT5)
tokenizers = "0.19"

# Error handling
anyhow = "1"
thiserror = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Misc
once_cell = "1"
regex = "1"
```

---

## Project File Structure

```
chemInteractions/
├── Cargo.toml
├── Cargo.lock
├── .env                         # DATABASE_URL, MODEL_PATH, etc.
├── src/
│   ├── main.rs                  # Server startup, route registration, DB init
│   ├── config.rs                # Environment config struct
│   ├── error.rs                 # AppError type + IntoResponse impl
│   ├── api/
│   │   ├── mod.rs               # Route aggregator
│   │   ├── predict.rs           # POST /api/predict
│   │   ├── compound.rs          # GET /api/compound/:name
│   │   └── reactions.rs         # GET /api/reactions, GET /api/reactions/:id
│   ├── predictor/
│   │   ├── mod.rs               # predict() entry point
│   │   ├── parser.rs            # Input normalization, SMILES validation
│   │   ├── pubchem.rs           # PubChem REST calls (reqwest)
│   │   ├── ml_brain/
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs        # ONNX session, encode/decode, beam search
│   │   │   └── tokenizer.rs     # SentencePiece tokenizer wrapper
│   │   ├── rule_brain/
│   │   │   ├── mod.rs
│   │   │   ├── matcher.rs       # Functional group detection + rule matching
│   │   │   └── db.rs            # sqlx queries against knowledge_base DB
│   │   ├── fusion.rs            # Score merger + tier assignment
│   │   ├── validator.rs         # Physical constraint checks
│   │   ├── byproducts.rs        # Byproduct annotation using KB
│   │   └── explainer.rs         # Mechanism text + textbook citation assembly
│   ├── db/
│   │   ├── mod.rs               # DB pool initialization
│   │   ├── schema.sql           # SQLite table definitions
│   │   └── seed.rs              # Load JSON rule files → SQLite on startup
│   └── models/
│       └── types.rs             # All shared domain types (Compound, Reaction, etc.)
├── knowledge_base/
│   ├── inorganic.json
│   ├── organic.json
│   └── physical.json
├── models/
│   ├── reactiont5_encoder.onnx
│   └── reactiont5_decoder.onnx
├── scripts/
│   ├── export_reactiont5.py     # One-time: HuggingFace → ONNX
│   └── requirements_export.txt  # transformers, onnx, optimum
├── frontend/
│   ├── index.html
│   ├── styles/main.css
│   └── scripts/
│       ├── app.js
│       ├── predictor.js
│       └── api.js
├── plans/                       # This folder
└── tests/
    ├── test_ml_brain.rs
    ├── test_rule_brain.rs
    ├── test_fusion.rs
    └── test_api.rs
```

---

## main.rs Skeleton

```rust
use axum::{Router, routing::{get, post}};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;

mod api;
mod config;
mod db;
mod error;
mod models;
mod predictor;

pub struct AppState {
    pub db: SqlitePool,
    pub ml_engine: Arc<predictor::ml_brain::engine::MlEngine>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Load config from .env
    let cfg = config::Config::from_env()?;

    // Initialize DB pool and seed knowledge base
    let db = db::init_pool(&cfg.database_url).await?;
    db::seed::load_rules(&db, "knowledge_base/").await?;

    // Load ML model
    let ml_engine = Arc::new(predictor::ml_brain::engine::MlEngine::load(&cfg.model_path)?);

    let state = Arc::new(AppState { db, ml_engine });

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/predict", post(api::predict::handler))
        .route("/api/compound/:name", get(api::compound::handler))
        .route("/api/reactions", get(api::reactions::list_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
```

---

## .env File

```
DATABASE_URL=sqlite://data/knowledge_base.db
MODEL_PATH=models/
PUBCHEM_API=https://pubchem.ncbi.nlm.nih.gov/rest/pug
ML_ALPHA=0.65
```

---

## Build & Run

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run (dev)
cargo run

# Test
cargo test

# Check health
curl http://localhost:8080/health
```

---

## First Milestone Checklist

- [ ] `cargo build` succeeds with all dependencies
- [ ] `cargo run` starts server on port 8080
- [ ] `GET /health` returns `"ok"`
- [ ] SQLite DB initializes and schema is created
- [ ] JSON rule files are loaded into DB on startup
- [ ] `GET /api/reactions` returns the seeded rules list
