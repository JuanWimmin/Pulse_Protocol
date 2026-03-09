#!/usr/bin/env bash
#
# seed_demo_data.sh — Seed demo data on Stellar Testnet for Pulse Protocol MVP
#
# Prerequisites:
#   - stellar CLI installed
#   - Contracts deployed via deploy_testnet.sh (deployed_contracts.json exists)
#   - Identities generated: pulse-deployer, pulse-oracle, pulse-user-alice, pulse-user-bob
#
# Usage:
#   ./scripts/seed_demo_data.sh
#
set -euo pipefail

NETWORK="testnet"
CONTRACTS_FILE="deployed_contracts.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[SEED]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# ── Check prerequisites ──
command -v stellar >/dev/null 2>&1 || err "stellar CLI not found"
[ -f "$CONTRACTS_FILE" ] || err "$CONTRACTS_FILE not found. Run deploy_testnet.sh first."

# ── Read contract IDs ──
VAULT_ID=$(python3 -c "import json; print(json.load(open('$CONTRACTS_FILE'))['contracts']['vault'])" 2>/dev/null \
    || node -e "console.log(JSON.parse(require('fs').readFileSync('$CONTRACTS_FILE','utf8')).contracts.vault)")
POL_ID=$(python3 -c "import json; print(json.load(open('$CONTRACTS_FILE'))['contracts']['proof_of_life'])" 2>/dev/null \
    || node -e "console.log(JSON.parse(require('fs').readFileSync('$CONTRACTS_FILE','utf8')).contracts.proof_of_life)")
BEN_ID=$(python3 -c "import json; print(json.load(open('$CONTRACTS_FILE'))['contracts']['beneficiary'])" 2>/dev/null \
    || node -e "console.log(JSON.parse(require('fs').readFileSync('$CONTRACTS_FILE','utf8')).contracts.beneficiary)")

DEPLOYER_ADDR=$(stellar keys address "pulse-deployer")
ORACLE_ADDR=$(stellar keys address "pulse-oracle")
ALICE_ADDR=$(stellar keys address "pulse-user-alice")
BOB_ADDR=$(stellar keys address "pulse-user-bob")

log "Vault contract:  $VAULT_ID"
log "ProofOfLife:     $POL_ID"
log "Beneficiary:     $BEN_ID"
log "Alice:           $ALICE_ADDR"
log "Bob:             $BOB_ADDR"

# ── Get native token address (XLM) ──
XLM_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
log "Using XLM token: $XLM_TOKEN"

# ── Create vault for Alice ──
log "Creating vault for Alice..."
VAULT_NUM=$(stellar contract invoke \
    --id "$VAULT_ID" \
    --source "pulse-user-alice" \
    --network "$NETWORK" \
    -- \
    create_vault \
    --owner "$ALICE_ADDR" \
    --token "$XLM_TOKEN") \
    || err "Failed to create vault"
log "Vault created with ID: $VAULT_NUM"

# ── Deposit test XLM (1 XLM = 10_000_000 stroops) ──
log "Depositing 10 XLM to vault..."
stellar contract invoke \
    --id "$VAULT_ID" \
    --source "pulse-user-alice" \
    --network "$NETWORK" \
    -- \
    deposit \
    --vault_id "$VAULT_NUM" \
    --from "$ALICE_ADDR" \
    --amount 100000000 \
    || warn "Deposit failed (Alice may need more testnet XLM)"

# ── Set beneficiaries: Alice 60%, Bob 40% ──
log "Setting beneficiaries (Alice 60%, Bob 40%)..."
stellar contract invoke \
    --id "$BEN_ID" \
    --source "pulse-deployer" \
    --network "$NETWORK" \
    -- \
    set_beneficiaries \
    --vault_id "$VAULT_NUM" \
    --beneficiaries "[{\"address\":\"$ALICE_ADDR\",\"percentage\":6000,\"claimed\":false},{\"address\":\"$BOB_ADDR\",\"percentage\":4000,\"claimed\":false}]" \
    || warn "Set beneficiaries failed"

# ── Link ProofOfLife to vault ──
log "Linking ProofOfLife contract to vault..."
stellar contract invoke \
    --id "$VAULT_ID" \
    --source "pulse-user-alice" \
    --network "$NETWORK" \
    -- \
    link_proof_of_life \
    --vault_id "$VAULT_NUM" \
    --pol_contract "$POL_ID" \
    || warn "Link ProofOfLife failed"

# ── Register perceptron model for Alice ──
log "Registering perceptron model for Alice..."
stellar contract invoke \
    --id "$POL_ID" \
    --source "pulse-user-alice" \
    --network "$NETWORK" \
    -- \
    register_model \
    --user "$ALICE_ADDR" \
    --initial_weights "[\"500000\",\"500000\",\"500000\",\"500000\",\"500000\",\"500000\",\"500000\",\"500000\",\"500000\",\"500000\"]" \
    --bias 0 \
    || warn "Register model failed"

log "=============================="
log "Demo data seeded successfully!"
log "=============================="
log ""
log "Pre-loaded data:"
log "  - Vault $VAULT_NUM owned by Alice"
log "  - 10 XLM deposited"
log "  - Beneficiaries: Alice 60%, Bob 40%"
log "  - ProofOfLife linked to vault"
log "  - Perceptron model registered for Alice"
