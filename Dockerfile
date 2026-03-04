# Stage 1: Build
# Ubuntu 24.04 required — ort's prebuilt ONNX Runtime needs glibc 2.38+.
# debian:bookworm-slim only has glibc 2.36 which causes a linker error.
FROM ubuntu:24.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install the same Rust version used locally
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.92.0
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy source (includes .sqlx/ offline cache from `cargo sqlx prepare`)
COPY . .

# Build without a live database — sqlx reads type-checked query cache from .sqlx/
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin chem-interactions

# ─────────────────────────────────────────────
# Stage 2: Runtime (also Ubuntu 24.04 for glibc)
# ─────────────────────────────────────────────
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    libgomp1 \
    curl \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/chem-interactions /usr/local/bin/chem-interactions
COPY --from=builder /app/knowledge_base ./knowledge_base
COPY --from=builder /app/models         ./models
COPY --from=builder /app/frontend       ./frontend
# Bring schema so we can initialise the DB on first run
COPY --from=builder /app/src/db/schema.sql ./schema.sql

# Pre-create the database with the correct schema so sqlx can open it on start.
# The data volume will overlay /app/data at runtime, so this seed only applies
# when no volume is mounted (or the volume is empty on first use).
RUN mkdir -p /app/data && sqlite3 /app/data/knowledge_base.db < /app/schema.sql

ENV RUST_LOG=info
ENV DATABASE_URL=sqlite://data/knowledge_base.db
ENV MODEL_PATH=/app/models

EXPOSE 8080

CMD ["chem-interactions"]
