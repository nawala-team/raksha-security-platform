-- Migration: 004_create_sessions
-- Description: Create sessions table for user authentication tracking
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

CREATE TABLE sessions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    ip_address  INET NOT NULL,
    user_agent  TEXT,
    device_info JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL,
    last_active TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at  TIMESTAMPTZ,

    CONSTRAINT chk_session_expires CHECK (expires_at > created_at),
    CONSTRAINT chk_revoked_after_created CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);

-- Indexes
CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_token_hash ON sessions (token_hash);
CREATE INDEX idx_sessions_expires_at ON sessions (expires_at);
CREATE INDEX idx_sessions_active ON sessions (user_id, expires_at)
    WHERE revoked_at IS NULL;
CREATE INDEX idx_sessions_ip_address ON sessions (ip_address);

-- Cleanup function for expired sessions
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM sessions
    WHERE expires_at < NOW() - INTERVAL '7 days'
       OR revoked_at < NOW() - INTERVAL '1 day';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Comments
COMMENT ON TABLE sessions IS 'Active and historical user sessions';
COMMENT ON COLUMN sessions.token_hash IS 'SHA-256 hash of the session token (token never stored in plaintext)';
COMMENT ON COLUMN sessions.ip_address IS 'Client IP address at session creation';
COMMENT ON COLUMN sessions.device_info IS 'Parsed device/browser information from user agent';
COMMENT ON COLUMN sessions.revoked_at IS 'Timestamp when session was explicitly revoked (logout or admin action)';
