-- Audit trail (append-only with hash chain)
CREATE TABLE audit_trail (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_id UUID REFERENCES users(id),
    actor_email VARCHAR(255),
    actor_ip INET,
    action_type audit_action_type NOT NULL,
    action_category audit_action_category NOT NULL,
    resource_type VARCHAR(255) NOT NULL,
    resource_id VARCHAR(255),
    changes_before JSONB,
    changes_after JSONB,
    metadata JSONB NOT NULL DEFAULT '{}',
    risk_level audit_risk_level NOT NULL DEFAULT 'low',
    session_id UUID,
    org_id UUID,
    integrity_hash VARCHAR(64) NOT NULL,
    previous_hash VARCHAR(64)
);

CREATE INDEX idx_audit_trail_timestamp ON audit_trail(timestamp DESC);
CREATE INDEX idx_audit_trail_actor ON audit_trail(actor_id);
CREATE INDEX idx_audit_trail_action ON audit_trail(action_type);
CREATE INDEX idx_audit_trail_resource ON audit_trail(resource_type, resource_id);
CREATE INDEX idx_audit_trail_hash ON audit_trail(integrity_hash);

-- Prevent updates/deletes on audit_trail (immutable log)
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit trail entries cannot be modified or deleted';
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER audit_trail_immutable_update
    BEFORE UPDATE ON audit_trail
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();

CREATE TRIGGER audit_trail_immutable_delete
    BEFORE DELETE ON audit_trail
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_modification();
