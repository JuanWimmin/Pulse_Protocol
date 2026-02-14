CREATE TABLE vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_vault_id BIGINT,
    owner_id UUID NOT NULL REFERENCES users(id),
    token_address VARCHAR(56) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Active',
    balance BIGINT NOT NULL DEFAULT 0,
    escrow_contract VARCHAR(56),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vaults_owner ON vaults(owner_id);