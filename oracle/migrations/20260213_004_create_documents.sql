CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id),
    vault_id UUID REFERENCES vaults(id),
    ipfs_cid VARCHAR(100) NOT NULL,
    doc_hash VARCHAR(64) NOT NULL,
    doc_type VARCHAR(30) NOT NULL,
    is_encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB,
    contract_doc_id BIGINT,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_documents_vault ON documents(vault_id);
CREATE INDEX idx_documents_owner ON documents(owner_id);