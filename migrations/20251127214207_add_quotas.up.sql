-- Add up migration script here
ALTER TABLE api_keys
ADD COLUMN rate_limit_per_minute INTEGER DEFAULT 60,
ADD COLUMN daily_quota INTEGER DEFAULT 10000,
ADD COLUMN monthly_quota INTEGER DEFAULT 300000;

CREATE TABLE quota_usage (
    key_id VARCHAR(255) NOT NULL,
    usage_date DATE NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (key_id, usage_date),
    FOREIGN KEY (key_id) REFERENCES api_keys(key_id) ON DELETE CASCADE
);

CREATE INDEX idx_quota_usage_date ON quota_usage(usage_date);
CREATE INDEX idx_quota_usage_key_date ON quota_usage(key_id, usage_date);

CREATE OR REPLACE FUNCTION increment_quota_usage(
    p_key_id VARCHAR(255),
    p_date DATE DEFAULT CURRENT_DATE
) RETURNS INTEGER AS $$
DECLARE
    v_count INTEGER;
BEGIN
    INSERT INTO quota_usage (key_id, usage_date, request_count, last_updated_at)
    VALUES (p_key_id, p_date, 1, NOW())
    ON CONFLICT (key_id, usage_date)
    DO UPDATE SET
        request_count = quota_usage.request_count + 1,
        last_updated_at = NOW()
    RETURNING request_count INTO v_count;
    
    RETURN v_count;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE rate_limit_tracking (
    key_id VARCHAR(255) NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (key_id, window_start)
);

CREATE INDEX idx_rate_limit_window ON rate_limit_tracking(window_start);

CREATE OR REPLACE FUNCTION check_rate_limit(
    p_key_id VARCHAR(255),
    p_limit INTEGER,
    p_window_minutes INTEGER DEFAULT 1
) RETURNS BOOLEAN AS $$
DECLARE
    v_window_start TIMESTAMPTZ;
    v_count INTEGER;
BEGIN
    v_window_start := date_trunc('minute', NOW());
    
    INSERT INTO rate_limit_tracking (key_id, window_start, request_count)
    VALUES (p_key_id, v_window_start, 1)
    ON CONFLICT (key_id, window_start)
    DO UPDATE SET request_count = rate_limit_tracking.request_count + 1
    RETURNING request_count INTO v_count;
    
    RETURN v_count <= p_limit;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION cleanup_rate_limits() RETURNS void AS $$
BEGIN
    DELETE FROM rate_limit_tracking
    WHERE window_start < NOW() - INTERVAL '5 minutes';
END;
$$ LANGUAGE plpgsql;