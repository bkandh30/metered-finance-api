-- Add down migration script here
ALTER TABLE IF EXISTS api_keys
DROP CONSTRAINT IF EXISTS check_valid_scopes;

DROP INDEX IF EXISTS idx_api_keys_created_at;
DROP INDEX IF EXISTS idx_api_keys_prefix;
DROP INDEX IF EXISTS idx_api_keys_active;

DROP TABLE IF EXISTS api_keys;