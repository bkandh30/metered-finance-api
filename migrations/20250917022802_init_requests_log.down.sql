-- Add down migration script here

ALTER TABLE IF EXISTS requests
DROP CONSTRAINT IF EXISTS fk_requests_key;

ALTER TABLE IF EXISTS requests
DROP CONSTRAINT IF EXISTS fk_requests_account;

DROP INDEX IF EXISTS idx_requests_status;
DROP INDEX IF EXISTS idx_requests_account_ts;
DROP INDEX IF EXISTS idx_requests_key_ts;
DROP INDEX IF EXISTS idx_requests_ts;

DROP TABLE IF EXISTS requests;
