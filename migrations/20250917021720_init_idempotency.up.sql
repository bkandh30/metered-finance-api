-- Add up migration script here
CREATE TABLE IF NOT EXISTS idempotency_keys (
    idempotency_key TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(account_id) ON DELETE CASCADE,
    txn_id TEXT UNIQUE,
    response_body JSONB,
    status_code INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours'),
    CHECK (expires_at > created_at),
    PRIMARY KEY (account_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_expires_at
ON idempotency_keys(expires_at);

CREATE INDEX IF NOT EXISTS idx_idempotency_account_created_at
ON idempotency_keys(account_id, created_at DESC);

ALTER TABLE idempotency_keys
ADD CONSTRAINT fk_idempotency_txn
FOREIGN KEY (txn_id)
REFERENCES transactions(txn_id)
ON DELETE SET NULL;