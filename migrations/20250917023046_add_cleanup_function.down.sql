-- Add down migration script here
DROP INDEX IF EXISTS idx_idempotency_cleanup;

DROP FUNCTION IF EXISTS cleanup_expired_idempotency_keys();