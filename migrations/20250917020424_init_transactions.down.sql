-- Add down migration script here
DROP INDEX IF EXISTS idx_transactions_account_pending_created_desc;
DROP INDEX IF EXISTS idx_transactions_pending_created_desc;
DROP INDEX IF EXISTS idx_transactions_account_created_at;
DROP INDEX IF EXISTS idx_transactions_created_at;
DROP INDEX IF EXISTS idx_transactions_timestamp;

DROP TABLE IF EXISTS transactions;
