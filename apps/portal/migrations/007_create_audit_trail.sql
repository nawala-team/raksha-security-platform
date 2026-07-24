-- Migration: 007_create_audit_trail
-- Description: Create audit_trail table with blockchain-style integrity hashing
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for audit trail
CREATE TYPE audit_action_type AS ENUM (
    'create',
    'read',
    'update',
    'delete',
    'login',
    'logout',
    'login_failed',
    'permission_change',
    'config_change',
    'export',
    'import',
    'escalation',
    'approval',
    'rejection'
);

CREATE TYPE audit_action_category AS ENUM (
    'authentication',
    'authorization',
    'data_access',
    'data_modification',
    'system_config',
    'security_event',
    'compliance',
    'user_management'
);

CREATE TYPE audit_risk_level AS ENUM (
    'low',
    'medium',
    'high',
    'critical'
);

CREATE TABLE audit_trail (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_id        UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_email     VARCHAR(255),
    actor_ip        INET,
    action_type     audit_action_type NOT NULL,
    action_category audit_action_category NOT NULL,
    resource_type   VARCHAR(100) NOT NULL,
    resource_id     VARCHAR(255),
    changes_before  JSONB,
    changes_after   JSONB,
    metadata        JSONB DEFAULT '{}',
    risk_level      audit_risk_level NOT NULL DEFAULT 'low',
    session_id      UUID,
    org_id          UUID REFERENCES organizations(id) ON DELETE SET NULL,
    integrity_hash  TEXT NOT NULL,
    previous_hash   TEXT,

    CONSTRAINT chk_audit_hash_format CHECK (integrity_hash ~ '^[a-f0-9]{64}$'),
    CONSTRAINT chk_audit_prev_hash_format CHECK (previous_hash IS NULL OR previous_hash ~ '^[a-f0-9]{64}$')
);

-- Indexes for audit trail queries
CREATE INDEX idx_audit_trail_timestamp ON audit_trail (timestamp DESC);
CREATE INDEX idx_audit_trail_actor_id ON audit_trail (actor_id);
CREATE INDEX idx_audit_trail_actor_email ON audit_trail (actor_email);
CREATE INDEX idx_audit_trail_action_type ON audit_trail (action_type);
CREATE INDEX idx_audit_trail_action_category ON audit_trail (action_category);
CREATE INDEX idx_audit_trail_resource ON audit_trail (resource_type, resource_id);
CREATE INDEX idx_audit_trail_risk_level ON audit_trail (risk_level) WHERE risk_level IN ('high', 'critical');
CREATE INDEX idx_audit_trail_org_id ON audit_trail (org_id);
CREATE INDEX idx_audit_trail_session_id ON audit_trail (session_id);

-- Composite index for common query patterns
CREATE INDEX idx_audit_trail_actor_time ON audit_trail (actor_id, timestamp DESC);
CREATE INDEX idx_audit_trail_resource_time ON audit_trail (resource_type, resource_id, timestamp DESC);

-- Partitioning by month for performance (using inheritance-based approach)
-- In production, consider range partitioning on timestamp

-- Function to compute integrity hash
CREATE OR REPLACE FUNCTION compute_audit_integrity_hash()
RETURNS TRIGGER AS $$
DECLARE
    prev_hash TEXT;
    hash_input TEXT;
BEGIN
    -- Get the previous entry's hash
    SELECT integrity_hash INTO prev_hash
    FROM audit_trail
    ORDER BY timestamp DESC, id DESC
    LIMIT 1;

    NEW.previous_hash := prev_hash;

    -- Compute hash of current record
    hash_input := COALESCE(NEW.actor_id::TEXT, '') ||
                  COALESCE(NEW.actor_email, '') ||
                  NEW.action_type::TEXT ||
                  NEW.resource_type ||
                  COALESCE(NEW.resource_id, '') ||
                  COALESCE(NEW.changes_before::TEXT, '') ||
                  COALESCE(NEW.changes_after::TEXT, '') ||
                  COALESCE(prev_hash, '');

    NEW.integrity_hash := encode(digest(hash_input, 'sha256'), 'hex');

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER audit_trail_integrity
    BEFORE INSERT ON audit_trail
    FOR EACH ROW
    EXECUTE FUNCTION compute_audit_integrity_hash();

-- Prevent updates and deletes on audit trail (immutable log)
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit trail records cannot be modified or deleted';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER audit_trail_immutable_update
    BEFORE UPDATE ON audit_trail
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_modification();

CREATE TRIGGER audit_trail_immutable_delete
    BEFORE DELETE ON audit_trail
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_modification();

-- Comments
COMMENT ON TABLE audit_trail IS 'Immutable audit log with blockchain-style integrity verification';
COMMENT ON COLUMN audit_trail.integrity_hash IS 'SHA-256 hash of record fields + previous hash for tamper detection';
COMMENT ON COLUMN audit_trail.previous_hash IS 'Hash of the preceding audit record (chain integrity)';
COMMENT ON COLUMN audit_trail.changes_before IS 'State of the resource before the change (for update/delete)';
COMMENT ON COLUMN audit_trail.changes_after IS 'State of the resource after the change (for create/update)';
COMMENT ON COLUMN audit_trail.risk_level IS 'Assessed risk level of this action for alerting purposes';
