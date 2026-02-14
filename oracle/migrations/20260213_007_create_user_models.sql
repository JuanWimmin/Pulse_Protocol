CREATE TABLE user_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id),
    weights JSONB NOT NULL,
    bias VARCHAR(30) NOT NULL,
    version INT NOT NULL DEFAULT 1,
    calibration_complete BOOLEAN NOT NULL DEFAULT FALSE,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);