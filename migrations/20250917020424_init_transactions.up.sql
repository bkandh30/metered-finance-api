-- Add up migration script here
CREATE TABLE IF NOT EXISTS transactions (
    txn_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(account_id),
    transaction_type TEXT NOT NULL CHECK (
        transaction_type IN ('payment', 'refund', 'payout', 'transfer', 'authorization', 'capture', 'reversal')
    ),
    status TEXT NOT NULL CHECK (
        status IN ('pending', 'succeeded', 'failed', 'reversed', 'canceled')
    ),
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency CHAR(3) NOT NULL CHECK (
        currency ~ '^[A-Z]{3}$'
    ),
    description TEXT,
    metadata JSONB,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    failure_reason TEXT CHECK (
        failure_reason IS NULL OR 
    failure_reason IN ('insufficient_funds', 'card_declined', 'risk_blocked', 'duplicate', 'internal_error')
    )
);

CREATE INDEX IF NOT EXISTS idx_transactions_account_created_at
ON transactions(account_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_transactions_created_at
ON transactions(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_transactions_pending_created_desc
ON transactions (created_at DESC)
WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_transactions_account_pending_created_desc
ON transactions (account_id, created_at DESC)
WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_transactions_timestamp
ON transactions(timestamp);