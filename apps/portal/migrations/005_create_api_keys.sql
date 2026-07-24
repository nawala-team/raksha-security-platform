-- Migration: 005_create_api_keys
-- Description: Create api_keys table for programmatic access
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

CREATE TABLE api_keys (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id      UUID REFERENCES organizations(id) ON DELETE CASCADE,
    name        VARCHAR(255) NOT NULL,
    key_prefix  VARCHAR(8) NOT NULL,
    key_hash    TEXT NOT NULL UNIQUE,
    scopes      JSONB NOT NULL DEFAULT '[]',
    rate_limit  INTEGER DEFAULT 1000,
    last_used   TIMESTAMPTZ,
    last_ip     INET,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ,
    revoked     BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at  TIMESTAMPTZ,
    revoked_by  UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT chk_api_key_prefix_format CHECK (key_prefix ~ '^rk_[a-z]{4}$'),
    CONSTRAINT chk_revoked_consistency CHECK (
        (revoked = FALSE AND revoked_at IS NULL AND revoked_by IS NULL) OR
        (revoked = TRUE AND revoked_at IS NOT NULL)
    ),
    CONSTRAINT chk_rate_limit_positive CHECK (rate_limit > 0)
);

-- Indexes
CREATE INDEX idx_api_keys_user_id ON api_keys (user_id);
CREATE INDEX idx_api_keys_org_id ON api_keys (org_id);
CREATE INDEX idx_api_keys_key_prefix ON api_keys (key_prefix);
CREATE INDEX idx_api_keys_key_hash ON api_keys (key_hash);
CREATE INDEX idx_api_keys_active ON api_keys (user_id)
    WHERE revoked = FALSE AND (expires_at IS NULL OR expires_at > NOW());
CREATE INDEX idx_api_keys_last_used ON api_keys (last_used);

-- Comments
COMMENT ON TABLE api_keys IS 'API keys for programmatic access to Raksha platform';
COMMENT ON COLUMN api_keys.key_prefix IS 'First 8 chars of the key for identification (format: rk_xxxx)';
COMMENT ON COLUMN api_keys.key_hash IS 'SHA-256 hash of the full API key';
COMMENT ON COLUMN api_keys.scopes IS 'JSON array of granted scopes (e.g., ["agents:read", "alerts:write"])';
COMMENT ON COLUMN api_keys.rate_limit IS 'Requests per minute rate limit for this key';
