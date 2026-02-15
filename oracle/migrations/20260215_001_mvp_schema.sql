-- MVP Schema: 4 tables for Pulse Protocol
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stellar_address VARCHAR(56) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Vaults (local cache of on-chain state)
CREATE TABLE vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id VARCHAR(56) UNIQUE,
    owner_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    escrow_contract_id VARCHAR(56),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Beneficiaries (local cache)
CREATE TABLE beneficiaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    stellar_address VARCHAR(56) NOT NULL,
    percentage INT NOT NULL CHECK (percentage > 0 AND percentage <= 10000),
    claimed BOOLEAN NOT NULL DEFAULT FALSE,
    claimed_at TIMESTAMPTZ,
    UNIQUE(vault_id, stellar_address)
);

-- Verifications (simplified history)
CREATE TABLE verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    score INT NOT NULL CHECK (score >= 0 AND score <= 10000),
    source VARCHAR(30) NOT NULL,
    perceptron_output INT,
    on_chain_tx_hash VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indices
CREATE INDEX idx_vaults_owner ON vaults(owner_id);
CREATE INDEX idx_beneficiaries_vault ON beneficiaries(vault_id);
CREATE INDEX idx_verifications_user_created ON verifications(user_id, created_at DESC);
