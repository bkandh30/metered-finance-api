-- Add up migration script here
CREATE TABLE IF NOT EXISTS api_keys (
    key_id TEXT PRIMARY KEY,
    prefix TEXT NOT NULL,
    secret_hash TEXT NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
);


CREATE INDEX IF NOT EXISTS idx_api_keys_active
ON api_keys(active) WHERE active = TRUE;

CREATE INDEX IF NOT EXISTS idx_api_keys_prefix
ON api_keys(prefix);

CREATE INDEX IF NOT EXISTS idx_api_keys_created_at
ON api_keys(created_at DESC);


ALTER TABLE api_keys 
ADD CONSTRAINT check_valid_scopes 
CHECK (scopes <@ ARRAY['client', 'admin', 'reporting']::TEXT[]);