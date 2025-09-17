-- Add up migration script here
CREATE OR REPLACE FUNCTION cleanup_expired_idempotency_keys()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM idempotency_keys
    WHERE expires_at < NOW()
    RETURNING COUNT(*) INTO deleted_count;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

CREATE INDEX IF NOT EXISTS idx_idempotency_cleanup 
ON idempotency_keys(expires_at) 
WHERE expires_at < NOW();