-- Add down migration script here
DROP INDEX IF EXISTS idx_accounts_created_at;

DROP TABLE IF EXISTS accounts;