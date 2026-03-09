FROM rust:1.82-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests + lock first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY oracle/Cargo.toml oracle/Cargo.toml

# Copy contract manifests (needed by workspace resolver, not compiled)
COPY contracts/vault/Cargo.toml contracts/vault/Cargo.toml
COPY contracts/proof-of-life/Cargo.toml contracts/proof-of-life/Cargo.toml
COPY contracts/beneficiary/Cargo.toml contracts/beneficiary/Cargo.toml
COPY contracts/document-registry/Cargo.toml contracts/document-registry/Cargo.toml
COPY contracts/integration-tests/Cargo.toml contracts/integration-tests/Cargo.toml

# Create dummy sources so cargo can resolve the workspace
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
    && mkdir -p oracle/src && echo "" > oracle/src/lib.rs \
    && mkdir -p contracts/vault/src && echo "#![no_std]" > contracts/vault/src/lib.rs \
    && mkdir -p contracts/proof-of-life/src && echo "#![no_std]" > contracts/proof-of-life/src/lib.rs \
    && mkdir -p contracts/beneficiary/src && echo "#![no_std]" > contracts/beneficiary/src/lib.rs \
    && mkdir -p contracts/document-registry/src && echo "#![no_std]" > contracts/document-registry/src/lib.rs \
    && mkdir -p contracts/integration-tests/src && echo "#![no_std]" > contracts/integration-tests/src/lib.rs

# Pre-build dependencies (cached layer)
RUN cargo build --release -p pulse-oracle 2>/dev/null || true

# Copy actual source code
COPY src/ src/
COPY oracle/src/ oracle/src/
COPY oracle/migrations/ oracle/migrations/

# Build for real
RUN touch src/main.rs && cargo build --release -p pulse-oracle

# ── Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pulse-oracle /usr/local/bin/pulse-oracle
COPY oracle/migrations/ /app/oracle/migrations/

WORKDIR /app
EXPOSE 8080

CMD ["pulse-oracle"]
