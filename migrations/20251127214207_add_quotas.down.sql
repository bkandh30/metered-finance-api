-- Add down migration script here
DROP FUNCTION IF EXISTS cleanup_rate_limits();
DROP FUNCTION IF EXISTS check_rate_limit(VARCHAR, INTEGER, INTEGER);
DROP FUNCTION IF EXISTS increment_quota_usage(VARCHAR, DATE);

DROP TABLE IF EXISTS rate_limit_tracking;

DROP TABLE IF EXISTS quota_usage;

ALTER TABLE api_keys
DROP COLUMN IF EXISTS rate_limit_per_minute,
DROP COLUMN IF EXISTS daily_quota,
DROP COLUMN IF EXISTS monthly_quota;