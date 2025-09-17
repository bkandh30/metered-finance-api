-- Add down migration script here
ALTER TABLE IF EXISTS idempotency_keys
DROP CONSTRAINT IF EXISTS fk_idempotency_txn;

DROP INDEX IF EXISTS idx_idempotency_account_created_at;
DROP INDEX IF EXISTS idx_idempotency_expires_at;

DROP TABLE IF EXISTS idempotency_keys;