-- GRC (Governance, Risk & Compliance) Module
-- Risk register, policy management, control framework mapping

-- ============================================================
-- Risk Register
-- ============================================================

CREATE TABLE IF NOT EXISTS grc_risks (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title           VARCHAR(500) NOT NULL,
    description     TEXT NOT NULL,
    category        VARCHAR(50) NOT NULL CHECK (category IN (
        'technical', 'operational', 'compliance', 'financial',
        'reputational', 'strategic', 'third_party'
    )),
    likelihood      SMALLINT NOT NULL CHECK (likelihood BETWEEN 1 AND 5),
    impact          SMALLINT NOT NULL CHECK (impact BETWEEN 1 AND 5),
    risk_score      SMALLINT NOT NULL GENERATED ALWAYS AS (likelihood * impact) STORED,
    owner           UUID NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'identified' CHECK (status IN (
        'identified', 'assessed', 'mitigated', 'accepted', 'closed'
    )),
    mitigation_plan TEXT,
    review_date     DATE NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_grc_risks_tenant ON grc_risks(tenant_id);
CREATE INDEX idx_grc_risks_status ON grc_risks(tenant_id, status);
CREATE INDEX idx_grc_risks_score ON grc_risks(tenant_id, risk_score DESC);
CREATE INDEX idx_grc_risks_review ON grc_risks(tenant_id, review_date)
    WHERE status NOT IN ('closed', 'accepted');
CREATE INDEX idx_grc_risks_owner ON grc_risks(tenant_id, owner);
CREATE INDEX idx_grc_risks_category ON grc_risks(tenant_id, category);

-- ============================================================
-- Policies
-- ============================================================

CREATE TABLE IF NOT EXISTS grc_policies (
    id                  UUID PRIMARY KEY,
    tenant_id           UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title               VARCHAR(500) NOT NULL,
    version             VARCHAR(20) NOT NULL,
    content             TEXT NOT NULL,
    status              VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN (
        'draft', 'active', 'archived'
    )),
    approved_by         UUID,
    effective_date      DATE,
    review_cycle_days   INTEGER NOT NULL DEFAULT 365,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_grc_policies_tenant ON grc_policies(tenant_id);
CREATE INDEX idx_grc_policies_status ON grc_policies(tenant_id, status);
CREATE INDEX idx_grc_policies_title ON grc_policies(tenant_id, title, version);

-- ============================================================
-- Policy Acknowledgments
-- ============================================================

CREATE TABLE IF NOT EXISTS grc_policy_acknowledgments (
    id                      UUID PRIMARY KEY,
    tenant_id               UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    policy_id               UUID NOT NULL REFERENCES grc_policies(id) ON DELETE CASCADE,
    user_id                 UUID NOT NULL,
    acknowledged_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version_acknowledged    VARCHAR(20) NOT NULL
);

CREATE UNIQUE INDEX idx_grc_ack_unique
    ON grc_policy_acknowledgments(policy_id, user_id, version_acknowledged);
CREATE INDEX idx_grc_ack_policy ON grc_policy_acknowledgments(policy_id);
CREATE INDEX idx_grc_ack_user ON grc_policy_acknowledgments(tenant_id, user_id);

-- ============================================================
-- Controls
-- ============================================================

CREATE TABLE IF NOT EXISTS grc_controls (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title           VARCHAR(500) NOT NULL,
    description     TEXT NOT NULL,
    framework       VARCHAR(20) NOT NULL CHECK (framework IN (
        'CIS', 'NIST', 'PCI-DSS', 'ISO-27001', 'SOC2', 'HIPAA'
    )),
    control_ref     VARCHAR(50) NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'not_implemented' CHECK (status IN (
        'implemented', 'partial', 'not_implemented', 'not_applicable'
    )),
    evidence        TEXT,
    last_assessed   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_grc_controls_tenant ON grc_controls(tenant_id);
CREATE INDEX idx_grc_controls_framework ON grc_controls(tenant_id, framework);
CREATE INDEX idx_grc_controls_status ON grc_controls(tenant_id, framework, status);
CREATE UNIQUE INDEX idx_grc_controls_ref ON grc_controls(tenant_id, framework, control_ref);

-- ============================================================
-- Control Mappings (cross-framework references)
-- ============================================================

CREATE TABLE IF NOT EXISTS grc_control_mappings (
    id              UUID PRIMARY KEY,
    control_id      UUID NOT NULL REFERENCES grc_controls(id) ON DELETE CASCADE,
    framework       VARCHAR(20) NOT NULL CHECK (framework IN (
        'CIS', 'NIST', 'PCI-DSS', 'ISO-27001', 'SOC2', 'HIPAA'
    )),
    framework_ref   VARCHAR(50) NOT NULL,
    rationale       TEXT
);

CREATE INDEX idx_grc_mappings_control ON grc_control_mappings(control_id);
CREATE INDEX idx_grc_mappings_framework ON grc_control_mappings(framework, framework_ref);
CREATE UNIQUE INDEX idx_grc_mappings_unique
    ON grc_control_mappings(control_id, framework, framework_ref);

-- ============================================================
-- Risk History (for trending)
-- ============================================================

CREATE TABLE IF NOT EXISTS grc_risk_history (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    risk_id     UUID NOT NULL REFERENCES grc_risks(id) ON DELETE CASCADE,
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    field       VARCHAR(50) NOT NULL,
    old_value   TEXT,
    new_value   TEXT,
    changed_by  UUID,
    changed_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_grc_risk_history_risk ON grc_risk_history(risk_id);
CREATE INDEX idx_grc_risk_history_tenant ON grc_risk_history(tenant_id, changed_at DESC);

-- ============================================================
-- Trigger: auto-update updated_at
-- ============================================================

CREATE OR REPLACE FUNCTION grc_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_grc_risks_updated
    BEFORE UPDATE ON grc_risks
    FOR EACH ROW EXECUTE FUNCTION grc_update_timestamp();

CREATE TRIGGER trg_grc_policies_updated
    BEFORE UPDATE ON grc_policies
    FOR EACH ROW EXECUTE FUNCTION grc_update_timestamp();

CREATE TRIGGER trg_grc_controls_updated
    BEFORE UPDATE ON grc_controls
    FOR EACH ROW EXECUTE FUNCTION grc_update_timestamp();

-- ============================================================
-- Trigger: record risk changes for trending
-- ============================================================

CREATE OR REPLACE FUNCTION grc_risk_audit()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO grc_risk_history (risk_id, tenant_id, field, old_value, new_value, changed_at)
        VALUES (NEW.id, NEW.tenant_id, 'status', OLD.status, NEW.status, NOW());
    END IF;
    IF OLD.risk_score IS DISTINCT FROM NEW.risk_score THEN
        INSERT INTO grc_risk_history (risk_id, tenant_id, field, old_value, new_value, changed_at)
        VALUES (NEW.id, NEW.tenant_id, 'risk_score', OLD.risk_score::text, NEW.risk_score::text, NOW());
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_grc_risk_audit
    AFTER UPDATE ON grc_risks
    FOR EACH ROW EXECUTE FUNCTION grc_risk_audit();

-- ============================================================
-- Row-Level Security
-- ============================================================

ALTER TABLE grc_risks ENABLE ROW LEVEL SECURITY;
ALTER TABLE grc_policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE grc_policy_acknowledgments ENABLE ROW LEVEL SECURITY;
ALTER TABLE grc_controls ENABLE ROW LEVEL SECURITY;
ALTER TABLE grc_control_mappings ENABLE ROW LEVEL SECURITY;
ALTER TABLE grc_risk_history ENABLE ROW LEVEL SECURITY;

CREATE POLICY grc_risks_tenant_isolation ON grc_risks
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

CREATE POLICY grc_policies_tenant_isolation ON grc_policies
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

CREATE POLICY grc_ack_tenant_isolation ON grc_policy_acknowledgments
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

CREATE POLICY grc_controls_tenant_isolation ON grc_controls
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

CREATE POLICY grc_mappings_tenant_isolation ON grc_control_mappings
    USING (control_id IN (
        SELECT id FROM grc_controls
        WHERE tenant_id = current_setting('app.current_tenant')::uuid
    ));

CREATE POLICY grc_risk_history_tenant_isolation ON grc_risk_history
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

