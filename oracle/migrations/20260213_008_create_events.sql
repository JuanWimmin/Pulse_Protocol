CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    aggregate_type VARCHAR(30) NOT NULL,
    aggregate_id UUID NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_hash VARCHAR(64)
);

CREATE INDEX idx_events_aggregate
    ON events(aggregate_type, aggregate_id, created_at);
CREATE INDEX idx_events_type
    ON events(event_type, created_at);