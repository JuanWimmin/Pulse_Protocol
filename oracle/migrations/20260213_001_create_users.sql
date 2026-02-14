CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stellar_address VARCHAR(56) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    calibration_complete BOOLEAN NOT NULL DEFAULT FALSE,  -- ← ¿Así?
    calibration_started_at TIMESTAMPTZ
);