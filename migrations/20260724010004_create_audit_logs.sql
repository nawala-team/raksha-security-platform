-- Raksha Security Platform
-- Migration: 20260724010004_create_audit_logs
-- Description: Immutable audit trail with cryptographic hash chain for tamper detection

CREATE TABLE audit_logs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID REFERENCES tenants(id) ON DELETE SET NULL,
    sequence_num        BIGINT GENERATED ALWAYS AS IDENTITY,
    timestamp           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_id            UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_email         VARCHAR(255),
    actor_role          VARCHAR(30),
    actor_ip            INET,
    actor_user_agent    TEXT,
    session_id          UUID,
    action_type         VARCHAR(30) NOT NULL
                        CHECK (action_type IN ('create', 'read', 'update', 'delete', 'login',
                                              'logout', 'login_failed', 'permission_change',
                                              'config_change', 'export', 'import', 'escalation',
                                              'approval', 'rejection', 'mfa_enroll', 'mfa_verify',
                                              'password_change', 'api_key_create', 'api_key_revoke')),
    action_category     VARCHAR(30) NOT NULL
                        CHECK (action_category IN ('authentication', 'authorization', 'data_access',
                                                  'data_modification', 'system_config', 'security_event',
                                                  'compliance', 'user_management', 'agent_management')),
    resource_type       VARCHAR(100) NOT NULL,
    resource_id         VARCHAR(255),
    resource_name       VARCHAR(255),
    description         TEXT,
    changes_before      JSONB,
    changes_after       JSONB,
    metadata            JSONB NOT NULL DEFAULT '{}',
    risk_level          VARCHAR(10) NOT NULL DEFAULT 'low'
                        CHECK (risk_level IN ('low', 'medium', 'high', 'critical')),
    outcome             VARCHAR(10) NOT NULL DEFAULT 'success'
                        CHECK (outcome IN ('success', 'failure', 'denied', 'error')),
    error_message       TEXT,
    integrity_hash      VARCHAR(64) NOT NULL,
    previous_hash       VARCHAR(64),
    chain_version       SMALLINT NOT NULL DEFAULT 1
);

-- Performance indexes
CREATE INDEX idx_audit_logs_tenant ON audit_logs(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_id) WHERE actor_id IS NOT NULL;
CREATE INDEX idx_audit_logs_action ON audit_logs(action_type);
CREATE INDEX idx_audit_logs_category ON audit_logs(action_category);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_risk ON audit_logs(risk_level) WHERE risk_level IN ('high', 'critical');
CREATE INDEX idx_audit_logs_sequence ON audit_logs(sequence_num);
CREATE INDEX idx_audit_logs_hash ON audit_logs(integrity_hash);
CREATE INDEX idx_audit_logs_session ON audit_logs(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX idx_audit_logs_outcome ON audit_logs(outcome) WHERE outcome != 'success';

-- Immutability triggers: prevent UPDATE and DELETE
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit log entries cannot be modified or deleted. Table is append-only.';
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_audit_logs_no_update
    BEFORE UPDATE ON audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

CREATE TRIGGER trg_audit_logs_no_delete
    BEFORE DELETE ON audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

COMMENT ON TABLE audit_logs IS 'Immutable append-only audit trail with SHA-256 hash chain for tamper detection. UPDATE and DELETE operations are blocked by trigger.';
COMMENT ON COLUMN audit_logs.integrity_hash IS 'SHA-256 hash of (sequence_num || timestamp || actor_id || action_type || resource_type || resource_id || previous_hash)';
COMMENT ON COLUMN audit_logs.previous_hash IS 'Hash of the preceding audit entry, forming a blockchain-like chain for integrity verification.';
COMMENT ON COLUMN audit_logs.sequence_num IS 'Monotonically increasing sequence for ordering and gap detection.';
COMMENT ON COLUMN audit_logs.chain_version IS 'Hash chain algorithm version for forward-compatible upgrades.';

-- Verification function: validates hash chain integrity
CREATE OR REPLACE FUNCTION verify_audit_chain(p_tenant_id UUID, p_limit INTEGER DEFAULT 1000)
RETURNS TABLE(entry_id UUID, seq BIGINT, is_valid BOOLEAN, expected_prev VARCHAR, actual_prev VARCHAR) AS $$
BEGIN
    RETURN QUERY
    WITH ordered_logs AS (
        SELECT
            al.id,
            al.sequence_num,
            al.integrity_hash,
            al.previous_hash,
            LAG(al.integrity_hash) OVER (ORDER BY al.sequence_num) AS expected_previous
        FROM audit_logs al
        WHERE al.tenant_id = p_tenant_id OR (p_tenant_id IS NULL AND al.tenant_id IS NULL)
        ORDER BY al.sequence_num
        LIMIT p_limit
    )
    SELECT
        ol.id,
        ol.sequence_num,
        (ol.previous_hash IS NOT DISTINCT FROM ol.expected_previous) AS is_valid,
        ol.expected_previous,
        ol.previous_hash
    FROM ordered_logs ol
    WHERE ol.previous_hash IS DISTINCT FROM ol.expected_previous;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION verify_audit_chain IS 'Verifies audit log hash chain integrity. Returns entries where previous_hash does not match expected value (potential tampering).';
