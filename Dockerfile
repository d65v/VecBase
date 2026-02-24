# VecBase — Dockerfile
# Multi-stage build: builder → minimal runtime image
# Author: d65v <https://github.com/d65v>

# ── Stage 1: Builder ──────────────────────────────────────────────────────────
FROM rust:1.77-slim AS builder

WORKDIR /app

# Cache dependencies first
COPY vcore/Cargo.toml ./vcore/Cargo.toml
RUN mkdir -p vcore/src && \
    echo "fn main() {}" > vcore/src/main.rs && \
    echo "" > vcore/src/lib.rs && \
    echo "" > vcore/src/embedding.rs && \
    echo "" > vcore/src/processing.rs

WORKDIR /app/vcore
RUN cargo build --release 2>/dev/null || true

# Now copy real source
WORKDIR /app
COPY vcore/src ./vcore/src

WORKDIR /app/vcore
# Touch files so cargo sees them as changed
RUN touch src/main.rs src/lib.rs src/embedding.rs src/processing.rs
RUN cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/vcore/target/release/vecbase /usr/local/bin/vecbase

# Data directory
RUN mkdir -p /app/data

# Copy example env
COPY vcore/src/plug-ins/env.example /app/.env.example

ENV RUST_LOG=info
ENV VECBASE_DIM=128
ENV VECBASE_METRIC=cosine
ENV VECBASE_MAX_ELEMENTS=1000000
ENV VECBASE_STORAGE_PATH=/app/data

EXPOSE 7777

ENTRYPOINT ["vecbase"]
CMD ["run"]
