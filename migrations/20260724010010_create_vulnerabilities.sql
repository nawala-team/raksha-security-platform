-- Raksha Security Platform
-- Migration: 20260724010010_create_vulnerabilities
-- Description: CVE scan results, vulnerability tracking, and remediation

CREATE TABLE vulnerability_scans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id        UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    scan_type       VARCHAR(30) NOT NULL
                    CHECK (scan_type IN ('full', 'quick', 'targeted', 'scheduled', 'on_demand')),
    scanner         VARCHAR(50) NOT NULL DEFAULT 'builtin'
                    CHECK (scanner IN ('builtin', 'trivy', 'grype', 'nessus', 'qualys', 'openvas', 'custom')),
    status          VARCHAR(20) NOT NULL DEFAULT 'running'
                    CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled', 'timeout')),
    target_packages BOOLEAN NOT NULL DEFAULT true,
    target_os       BOOLEAN NOT NULL DEFAULT true,
    target_kernel   BOOLEAN NOT NULL DEFAULT true,
    target_configs  BOOLEAN NOT NULL DEFAULT false,
    total_packages  INTEGER DEFAULT 0,
    total_vulns     INTEGER DEFAULT 0,
    critical_count  INTEGER DEFAULT 0,
    high_count      INTEGER DEFAULT 0,
    medium_count    INTEGER DEFAULT 0,
    low_count       INTEGER DEFAULT 0,
    info_count      INTEGER DEFAULT 0,
    fixable_count   INTEGER DEFAULT 0,
    duration_secs   INTEGER,
    error_message   TEXT,
    initiated_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vuln_scans_tenant ON vulnerability_scans(tenant_id);
CREATE INDEX idx_vuln_scans_agent ON vulnerability_scans(agent_id);
CREATE INDEX idx_vuln_scans_status ON vulnerability_scans(status);
CREATE INDEX idx_vuln_scans_created ON vulnerability_scans(created_at DESC);
CREATE INDEX idx_vuln_scans_agent_latest ON vulnerability_scans(agent_id, created_at DESC) WHERE status = 'completed';

COMMENT ON TABLE vulnerability_scans IS 'Vulnerability scan execution records with summary statistics per agent.';

-- Individual vulnerability findings
CREATE TABLE vulnerabilities (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    scan_id         UUID NOT NULL REFERENCES vulnerability_scans(id) ON DELETE CASCADE,
    agent_id        UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    cve_id          VARCHAR(30),
    vulnerability_id VARCHAR(100) NOT NULL,
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    severity        VARCHAR(10) NOT NULL
                    CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    cvss_v3_score   NUMERIC(3,1) CHECK (cvss_v3_score BETWEEN 0.0 AND 10.0),
    cvss_v3_vector  VARCHAR(200),
    cvss_v2_score   NUMERIC(3,1) CHECK (cvss_v2_score BETWEEN 0.0 AND 10.0),
    epss_score      NUMERIC(5,4) CHECK (epss_score BETWEEN 0.0 AND 1.0),
    epss_percentile NUMERIC(5,4),
    is_exploited_in_wild BOOLEAN NOT NULL DEFAULT false,
    exploit_available BOOLEAN NOT NULL DEFAULT false,
    exploit_maturity VARCHAR(20)
                    CHECK (exploit_maturity IN ('unproven', 'poc', 'functional', 'weaponized')),
    package_name    VARCHAR(255),
    package_version VARCHAR(100),
    package_type    VARCHAR(30)
                    CHECK (package_type IN ('os', 'deb', 'rpm', 'apk', 'npm', 'pip', 'gem',
                                           'cargo', 'go', 'maven', 'nuget', 'composer', 'other')),
    fixed_version   VARCHAR(100),
    is_fixable      BOOLEAN NOT NULL DEFAULT false,
    affected_component VARCHAR(255),
    install_path    TEXT,
    status          VARCHAR(20) NOT NULL DEFAULT 'open'
                    CHECK (status IN ('open', 'acknowledged', 'in_remediation', 'fixed',
                                     'wont_fix', 'accepted_risk', 'false_positive')),
    remediation_action TEXT,
    remediated_at   TIMESTAMPTZ,
    remediated_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    risk_accepted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    risk_accepted_reason TEXT,
    risk_accepted_until TIMESTAMPTZ,
    published_at    TIMESTAMPTZ,
    modified_at     TIMESTAMPTZ,
    references      JSONB DEFAULT '[]',
    cwe_ids         TEXT[],
    alert_id        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    first_detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vulns_tenant ON vulnerabilities(tenant_id);
CREATE INDEX idx_vulns_scan ON vulnerabilities(scan_id);
CREATE INDEX idx_vulns_agent ON vulnerabilities(agent_id);
CREATE INDEX idx_vulns_cve ON vulnerabilities(cve_id) WHERE cve_id IS NOT NULL;
CREATE INDEX idx_vulns_severity ON vulnerabilities(severity);
CREATE INDEX idx_vulns_status ON vulnerabilities(status);
CREATE INDEX idx_vulns_package ON vulnerabilities(package_name, package_version);
CREATE INDEX idx_vulns_exploited ON vulnerabilities(is_exploited_in_wild) WHERE is_exploited_in_wild = true;
CREATE INDEX idx_vulns_fixable ON vulnerabilities(is_fixable) WHERE is_fixable = true AND status = 'open';
CREATE INDEX idx_vulns_cvss ON vulnerabilities(cvss_v3_score DESC) WHERE status = 'open';
CREATE INDEX idx_vulns_agent_open ON vulnerabilities(agent_id, severity) WHERE status = 'open';
CREATE INDEX idx_vulns_first_detected ON vulnerabilities(first_detected_at DESC);
CREATE INDEX idx_vulns_epss ON vulnerabilities(epss_score DESC) WHERE epss_score IS NOT NULL AND status = 'open';

CREATE TRIGGER trg_vulnerabilities_updated_at
    BEFORE UPDATE ON vulnerabilities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE vulnerabilities IS 'Individual CVE findings per agent with CVSS/EPSS scoring, exploit intelligence, and remediation tracking.';
COMMENT ON COLUMN vulnerabilities.epss_score IS 'Exploit Prediction Scoring System probability (0-1). Higher = more likely to be exploited.';
COMMENT ON COLUMN vulnerabilities.is_exploited_in_wild IS 'Known exploited vulnerability (KEV). Prioritize remediation.';
COMMENT ON COLUMN vulnerabilities.vulnerability_id IS 'Unique vulnerability identifier from scanner. May differ from CVE ID for non-CVE vulns.';
