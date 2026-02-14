CREATE TABLE verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    score INT NOT NULL CHECK (score >= 0 AND score <= 10000),
    source VARCHAR(30) NOT NULL,
    face_match_score INT,
    face_liveness_score INT,
    fingerprint_frequency INT,
    fingerprint_count INT,
    time_of_day_normality INT,
    typing_pattern_match INT,
    app_usage_match INT,
    movement_pattern_match INT,
    days_since_last_verify INT,
    session_duration INT,
    perceptron_score INT,
    hash VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_verifications_user_created
    ON verifications(user_id, created_at DESC);