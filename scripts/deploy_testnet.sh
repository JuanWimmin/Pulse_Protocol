#!/usr/bin/env bash
#
# deploy_testnet.sh — Deploy Pulse Protocol contracts to Stellar Testnet
#
# Prerequisites:
#   - stellar CLI installed (https://developers.stellar.org/docs/tools/cli)
#   - WASM files built: cargo build --workspace --target wasm32-unknown-unknown --release
#
# Usage:
#   ./scripts/deploy_testnet.sh
#
# Output:
#   Creates deployed_contracts.json with all contract IDs and account info.
#
set -euo pipefail

NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org"
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

WASM_DIR="target/wasm32-unknown-unknown/release"
OUTPUT_FILE="deployed_contracts.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[DEPLOY]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# ── Check prerequisites ──
command -v stellar >/dev/null 2>&1 || err "stellar CLI not found. Install: https://developers.stellar.org/docs/tools/cli"

for contract in vault proof_of_life beneficiary; do
    wasm_file="${WASM_DIR}/pulse_${contract}.wasm"
    if [ ! -f "$wasm_file" ]; then
        err "WASM not found: $wasm_file\nRun: cargo build --workspace --target wasm32-unknown-unknown --release"
    fi
done

# ── Generate identities ──
log "Generating Stellar identities..."

generate_identity() {
    local name=$1
    if stellar keys show "$name" >/dev/null 2>&1; then
        warn "Identity '$name' already exists, reusing"
    else
        stellar keys generate "$name" --network "$NETWORK"
        log "Created identity: $name"
    fi
    # Fund via friendbot
    local addr
    addr=$(stellar keys address "$name")
    log "Funding $name ($addr) via friendbot..."
    curl -s "https://friendbot.stellar.org/?addr=$addr" > /dev/null || warn "Friendbot failed for $name"
}

generate_identity "pulse-deployer"
generate_identity "pulse-oracle"
generate_identity "pulse-user-alice"
generate_identity "pulse-user-bob"

DEPLOYER_ADDR=$(stellar keys address "pulse-deployer")
ORACLE_ADDR=$(stellar keys address "pulse-oracle")
ALICE_ADDR=$(stellar keys address "pulse-user-alice")
BOB_ADDR=$(stellar keys address "pulse-user-bob")

log "Deployer: $DEPLOYER_ADDR"
log "Oracle:   $ORACLE_ADDR"
log "Alice:    $ALICE_ADDR"
log "Bob:      $BOB_ADDR"

# ── Deploy contracts ──
log "Deploying Vault contract..."
VAULT_ID=$(stellar contract deploy \
    --wasm "${WASM_DIR}/pulse_vault.wasm" \
    --source "pulse-deployer" \
    --network "$NETWORK")
log "Vault deployed: $VAULT_ID"

log "Deploying ProofOfLife contract..."
POL_ID=$(stellar contract deploy \
    --wasm "${WASM_DIR}/pulse_proof_of_life.wasm" \
    --source "pulse-deployer" \
    --network "$NETWORK")
log "ProofOfLife deployed: $POL_ID"

log "Deploying Beneficiary contract..."
BEN_ID=$(stellar contract deploy \
    --wasm "${WASM_DIR}/pulse_beneficiary.wasm" \
    --source "pulse-deployer" \
    --network "$NETWORK")
log "Beneficiary deployed: $BEN_ID"

# ── Initialize contracts ──
log "Initializing Vault contract..."
stellar contract invoke \
    --id "$VAULT_ID" \
    --source "pulse-deployer" \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$DEPLOYER_ADDR" \
    || warn "Vault initialize failed (may already be initialized)"

log "Initializing ProofOfLife contract..."
stellar contract invoke \
    --id "$POL_ID" \
    --source "pulse-deployer" \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$DEPLOYER_ADDR" \
    --oracle "$ORACLE_ADDR" \
    || warn "ProofOfLife initialize failed (may already be initialized)"

log "Initializing Beneficiary contract..."
stellar contract invoke \
    --id "$BEN_ID" \
    --source "pulse-deployer" \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$DEPLOYER_ADDR" \
    || warn "Beneficiary initialize failed (may already be initialized)"

# ── Write output ──
cat > "$OUTPUT_FILE" <<EOF
{
    "network": "$NETWORK",
    "rpc_url": "$RPC_URL",
    "network_passphrase": "$NETWORK_PASSPHRASE",
    "contracts": {
        "vault": "$VAULT_ID",
        "proof_of_life": "$POL_ID",
        "beneficiary": "$BEN_ID"
    },
    "accounts": {
        "deployer": "$DEPLOYER_ADDR",
        "oracle": "$ORACLE_ADDR",
        "alice": "$ALICE_ADDR",
        "bob": "$BOB_ADDR"
    },
    "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

log "=============================="
log "Deployment complete!"
log "=============================="
log ""
log "Contract IDs saved to: $OUTPUT_FILE"
log ""
ORACLE_SECRET=$(stellar keys show pulse-oracle 2>/dev/null || echo 'S...')

log "Add to your .env:"
log "  VAULT_CONTRACT_ID=$VAULT_ID"
log "  PROOF_OF_LIFE_CONTRACT_ID=$POL_ID"
log "  BENEFICIARY_CONTRACT_ID=$BEN_ID"
log "  ORACLE_SECRET_KEY=$ORACLE_SECRET"
log ""
log "Test accounts funded on testnet:"
log "  Alice: $ALICE_ADDR"
log "  Bob:   $BOB_ADDR"
