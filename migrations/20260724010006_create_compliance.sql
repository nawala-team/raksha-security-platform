-- Raksha Security Platform
-- Migration: 20260724010006_create_compliance
-- Description: Compliance frameworks, controls, check results, and scoring

-- Compliance frameworks/standards
CREATE TABLE compliance_frameworks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    short_name      VARCHAR(50) NOT NULL,
    version         VARCHAR(50) NOT NULL,
    description     TEXT,
    authority       VARCHAR(255),
    url             TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    is_custom       BOOLEAN NOT NULL DEFAULT false,
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_compliance_framework UNIQUE (short_name, version, tenant_id)
);

CREATE INDEX idx_compliance_frameworks_tenant ON compliance_frameworks(tenant_id);
CREATE INDEX idx_compliance_frameworks_active ON compliance_frameworks(is_active) WHERE is_active = true;

CREATE TRIGGER trg_compliance_frameworks_updated_at
    BEFORE UPDATE ON compliance_frameworks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE compliance_frameworks IS 'Compliance standards and regulatory frameworks (CIS, PCI-DSS, HIPAA, SOC2, NIST, etc.).';

-- Compliance controls (individual checks within a framework)
CREATE TABLE compliance_controls (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework_id    UUID NOT NULL REFERENCES compliance_frameworks(id) ON DELETE CASCADE,
    control_ref     VARCHAR(100) NOT NULL,
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    category        VARCHAR(255),
    subcategory     VARCHAR(255),
    parent_id       UUID REFERENCES compliance_controls(id) ON DELETE CASCADE,
    severity        VARCHAR(10) NOT NULL DEFAULT 'medium'
                    CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    is_automated    BOOLEAN NOT NULL DEFAULT false,
    check_command   TEXT,
    remediation     TEXT,
    references      JSONB DEFAULT '[]',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_control_ref_framework UNIQUE (control_ref, framework_id)
);

CREATE INDEX idx_compliance_controls_framework ON compliance_controls(framework_id);
CREATE INDEX idx_compliance_controls_parent ON compliance_controls(parent_id);
CREATE INDEX idx_compliance_controls_severity ON compliance_controls(severity);
CREATE INDEX idx_compliance_controls_automated ON compliance_controls(is_automated) WHERE is_automated = true;

CREATE TRIGGER trg_compliance_controls_updated_at
    BEFORE UPDATE ON compliance_controls
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE compliance_controls IS 'Individual compliance controls within a framework, supporting hierarchy and automated checks.';

-- Compliance check results (per agent per control)
CREATE TABLE compliance_check_results (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    framework_id    UUID NOT NULL REFERENCES compliance_frameworks(id) ON DELETE CASCADE,
    control_id      UUID NOT NULL REFERENCES compliance_controls(id) ON DELETE CASCADE,
    agent_id        UUID REFERENCES agents(id) ON DELETE CASCADE,
    status          VARCHAR(20) NOT NULL
                    CHECK (status IN ('compliant', 'non_compliant', 'partially_compliant',
                                     'not_assessed', 'not_applicable', 'error')),
    score           SMALLINT CHECK (score BETWEEN 0 AND 100),
    evidence        JSONB DEFAULT '{}',
    output          TEXT,
    remediation_status VARCHAR(20) DEFAULT 'pending'
                    CHECK (remediation_status IN ('pending', 'in_progress', 'remediated',
                                                 'accepted_risk', 'deferred', 'not_applicable')),
    remediated_at   TIMESTAMPTZ,
    remediated_by   UUID REFERENCES users(id),
    exception_reason TEXT,
    exception_expires TIMESTAMPTZ,
    assessed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_check_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_compliance_results_tenant ON compliance_check_results(tenant_id);
CREATE INDEX idx_compliance_results_framework ON compliance_check_results(framework_id);
CREATE INDEX idx_compliance_results_control ON compliance_check_results(control_id);
CREATE INDEX idx_compliance_results_agent ON compliance_check_results(agent_id);
CREATE INDEX idx_compliance_results_status ON compliance_check_results(status);
CREATE INDEX idx_compliance_results_noncompliant ON compliance_check_results(framework_id, agent_id)
    WHERE status = 'non_compliant';
CREATE INDEX idx_compliance_results_assessed ON compliance_check_results(assessed_at DESC);

CREATE TRIGGER trg_compliance_results_updated_at
    BEFORE UPDATE ON compliance_check_results
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE compliance_check_results IS 'Per-agent compliance check results with evidence, remediation tracking, and risk acceptance.';

-- Compliance score snapshots (aggregated scores over time)
CREATE TABLE compliance_score_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    framework_id    UUID NOT NULL REFERENCES compliance_frameworks(id) ON DELETE CASCADE,
    agent_id        UUID REFERENCES agents(id) ON DELETE CASCADE,
    overall_score   NUMERIC(5,2) NOT NULL DEFAULT 0,
    controls_total  INTEGER NOT NULL DEFAULT 0,
    controls_passed INTEGER NOT NULL DEFAULT 0,
    controls_failed INTEGER NOT NULL DEFAULT 0,
    controls_na     INTEGER NOT NULL DEFAULT 0,
    breakdown       JSONB NOT NULL DEFAULT '{}',
    assessed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assessed_by     UUID REFERENCES users(id)
);

CREATE INDEX idx_compliance_snapshots_tenant ON compliance_score_snapshots(tenant_id);
CREATE INDEX idx_compliance_snapshots_framework ON compliance_score_snapshots(framework_id, assessed_at DESC);
CREATE INDEX idx_compliance_snapshots_agent ON compliance_score_snapshots(agent_id, assessed_at DESC);

COMMENT ON TABLE compliance_score_snapshots IS 'Point-in-time compliance score snapshots for trend analysis and reporting.';
