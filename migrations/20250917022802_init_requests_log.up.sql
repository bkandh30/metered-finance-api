-- Add up migration script here
CREATE TABLE IF NOT EXISTS requests (
    id BIGSERIAL PRIMARY KEY,
    key_id TEXT,
    account_id TEXT,
    path TEXT NOT NULL,
    method TEXT NOT NULL,
    status INT NOT NULL,
    latency_ms INT NOT NULL,
    ts TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


CREATE INDEX IF NOT EXISTS idx_requests_ts 
ON requests(ts DESC);

CREATE INDEX IF NOT EXISTS idx_requests_key_ts 
ON requests(key_id, ts DESC) WHERE key_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_requests_account_ts 
ON requests(account_id, ts DESC) WHERE account_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_requests_status 
ON requests(status) WHERE status >= 400;


ALTER TABLE requests 
ADD CONSTRAINT fk_requests_key 
FOREIGN KEY (key_id) 
REFERENCES api_keys(key_id) 
ON DELETE SET NULL;

ALTER TABLE requests 
ADD CONSTRAINT fk_requests_account 
FOREIGN KEY (account_id) 
REFERENCES accounts(account_id) 
ON DELETE SET NULL;