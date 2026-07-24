-- Migration: 018_create_enrollment_tokens.sql
-- Enrollment token management for secure agent registration

CREATE TABLE IF NOT EXISTS enrollment_tokens (
    token_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash      VARCHAR(64) NOT NULL UNIQUE,
    token_prefix    VARCHAR(16) NOT NULL,
    org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    agent_name      VARCHAR(255),
    labels          JSONB NOT NULL DEFAULT '[]',
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL,
    max_uses        INTEGER NOT NULL DEFAULT 1,
    use_count       INTEGER NOT NULL DEFAULT 0,
    status          VARCHAR(20) NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'used', 'expired', 'revoked')),
    allowed_modules JSONB NOT NULL DEFAULT '["server", "network", "process"]',
    last_used_ip    INET,
    last_used_at    TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    revoked_by      UUID REFERENCES users(id)
);

CREATE INDEX idx_enrollment_tokens_hash ON enrollment_tokens(token_hash);
CREATE INDEX idx_enrollment_tokens_org ON enrollment_tokens(org_id, status);
CREATE INDEX idx_enrollment_tokens_expires ON enrollment_tokens(expires_at)
    WHERE status = 'active';

COMMENT ON TABLE enrollment_tokens IS 'One-time-use tokens for agent enrollment';
COMMENT ON COLUMN enrollment_tokens.token_hash IS 'SHA-256 hash of the raw token (never store plaintext)';
COMMENT ON COLUMN enrollment_tokens.token_prefix IS 'First 12 chars for display: rkat_xxxx...';

-- Organizations table (needed for multi-tenant)
CREATE TABLE IF NOT EXISTS organizations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(100) NOT NULL UNIQUE,
    status          VARCHAR(20) NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'suspended', 'deleted')),
    settings        JSONB NOT NULL DEFAULT '{}',
    encryption_key  BYTEA,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER set_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();

-- Agent certificates tracking
CREATE TABLE IF NOT EXISTS agent_certificates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id        UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    org_id          UUID NOT NULL REFERENCES organizations(id),
    serial          VARCHAR(64) NOT NULL UNIQUE,
    fingerprint     VARCHAR(64) NOT NULL,
    common_name     VARCHAR(255) NOT NULL,
    not_before      TIMESTAMPTZ NOT NULL,
    not_after       TIMESTAMPTZ NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'pending_rotation', 'revoked', 'expired')),
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ,
    revoked_reason  TEXT
);

CREATE INDEX idx_agent_certs_agent ON agent_certificates(agent_id, status);
CREATE INDEX idx_agent_certs_expiry ON agent_certificates(not_after)
    WHERE status = 'active';

COMMENT ON TABLE agent_certificates IS 'mTLS certificates issued to agents';

-- Add org_id to agents table and identity_hash
ALTER TABLE agents ADD COLUMN IF NOT EXISTS org_id UUID REFERENCES organizations(id);
ALTER TABLE agents ADD COLUMN IF NOT EXISTS identity_hash VARCHAR(64);
ALTER TABLE agents ADD COLUMN IF NOT EXISTS enrolled_via_token UUID REFERENCES enrollment_tokens(token_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_agents_identity 
    ON agents(org_id, identity_hash) WHERE status != 'deregistered';
