-- Migration: 011_create_compliance
-- Description: Create compliance management tables
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for compliance
CREATE TYPE compliance_status AS ENUM (
    'compliant',
    'non_compliant',
    'partially_compliant',
    'not_assessed',
    'not_applicable'
);

CREATE TYPE check_result AS ENUM (
    'pass',
    'fail',
    'warning',
    'error',
    'skip'
);

-- Compliance standards (e.g., SOC2, ISO27001, HIPAA, PCI-DSS)
CREATE TABLE compliance_standards (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL UNIQUE,
    version     VARCHAR(50) NOT NULL,
    description TEXT,
    authority   VARCHAR(255),
    url         TEXT,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    metadata    JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Controls within a standard
CREATE TABLE compliance_controls (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    standard_id     UUID NOT NULL REFERENCES compliance_standards(id) ON DELETE CASCADE,
    control_ref     VARCHAR(50) NOT NULL,
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    category        VARCHAR(255),
    parent_id       UUID REFERENCES compliance_controls(id) ON DELETE SET NULL,
    severity        alert_severity NOT NULL DEFAULT 'medium',
    implementation_guidance TEXT,
    automated       BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_control_ref_standard UNIQUE (standard_id, control_ref)
);

-- Automated compliance checks (linked to controls)
CREATE TABLE compliance_checks (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    control_id      UUID NOT NULL REFERENCES compliance_controls(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    check_type      VARCHAR(100) NOT NULL,
    check_config    JSONB NOT NULL DEFAULT '{}',
    schedule_cron   VARCHAR(100),
    last_run        TIMESTAMPTZ,
    last_result     check_result,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Compliance check results history
CREATE TABLE compliance_check_results (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    check_id    UUID NOT NULL REFERENCES compliance_checks(id) ON DELETE CASCADE,
    result      check_result NOT NULL,
    details     JSONB DEFAULT '{}',
    evidence    JSONB DEFAULT '[]',
    agent_id    UUID REFERENCES agents(id) ON DELETE SET NULL,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms INTEGER
);

-- Compliance scores (aggregated per org/standard)
CREATE TABLE compliance_scores (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    standard_id     UUID NOT NULL REFERENCES compliance_standards(id) ON DELETE CASCADE,
    overall_score   NUMERIC(5,2) NOT NULL,
    status          compliance_status NOT NULL,
    controls_total  INTEGER NOT NULL DEFAULT 0,
    controls_passed INTEGER NOT NULL DEFAULT 0,
    controls_failed INTEGER NOT NULL DEFAULT 0,
    controls_na     INTEGER NOT NULL DEFAULT 0,
    breakdown       JSONB DEFAULT '{}',
    assessed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_assessment TIMESTAMPTZ,
    assessed_by     UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT chk_score_range CHECK (overall_score >= 0 AND overall_score <= 100),
    CONSTRAINT chk_controls_sum CHECK (controls_passed + controls_failed + controls_na <= controls_total)
);

-- Indexes
CREATE INDEX idx_compliance_standards_name ON compliance_standards (name);
CREATE INDEX idx_compliance_standards_active ON compliance_standards (is_active) WHERE is_active = TRUE;

CREATE INDEX idx_compliance_controls_standard ON compliance_controls (standard_id);
CREATE INDEX idx_compliance_controls_ref ON compliance_controls (control_ref);
CREATE INDEX idx_compliance_controls_parent ON compliance_controls (parent_id);
CREATE INDEX idx_compliance_controls_category ON compliance_controls (category);

CREATE INDEX idx_compliance_checks_control ON compliance_checks (control_id);
CREATE INDEX idx_compliance_checks_org ON compliance_checks (org_id);
CREATE INDEX idx_compliance_checks_enabled ON compliance_checks (enabled) WHERE enabled = TRUE;
CREATE INDEX idx_compliance_checks_last_run ON compliance_checks (last_run);

CREATE INDEX idx_check_results_check ON compliance_check_results (check_id, executed_at DESC);
CREATE INDEX idx_check_results_result ON compliance_check_results (result);

CREATE INDEX idx_compliance_scores_org ON compliance_scores (org_id);
CREATE INDEX idx_compliance_scores_standard ON compliance_scores (standard_id);
CREATE INDEX idx_compliance_scores_org_std ON compliance_scores (org_id, standard_id, assessed_at DESC);
CREATE INDEX idx_compliance_scores_status ON compliance_scores (status);

-- Triggers
CREATE TRIGGER set_compliance_standards_updated_at
    BEFORE UPDATE ON compliance_standards
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_compliance_controls_updated_at
    BEFORE UPDATE ON compliance_controls
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_compliance_checks_updated_at
    BEFORE UPDATE ON compliance_checks
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE compliance_standards IS 'Regulatory and security compliance frameworks';
COMMENT ON TABLE compliance_controls IS 'Individual controls within a compliance standard (hierarchical)';
COMMENT ON TABLE compliance_checks IS 'Automated checks that verify control implementation';
COMMENT ON TABLE compliance_check_results IS 'Historical results of compliance check executions';
COMMENT ON TABLE compliance_scores IS 'Aggregated compliance scores per organization and standard';
COMMENT ON COLUMN compliance_scores.overall_score IS 'Percentage score (0-100) of compliance';
COMMENT ON COLUMN compliance_scores.breakdown IS 'Category-level score breakdown as JSONB';
