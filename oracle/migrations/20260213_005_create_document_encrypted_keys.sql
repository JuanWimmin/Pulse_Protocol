CREATE TABLE document_encrypted_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    beneficiary_address VARCHAR(56) NOT NULL,
    encrypted_key TEXT NOT NULL,
    revealed BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(document_id, beneficiary_address)
);