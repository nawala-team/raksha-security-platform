-- Raksha Security Platform
-- Migration: 20260724010003_create_alerts
-- Description: Alert events with severity, status, assignment, correlation, and escalation

CREATE TABLE alerts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID REFERENCES tenants(id) ON DELETE CASCADE,
    alert_key           VARCHAR(255),
    title               VARCHAR(500) NOT NULL,
    description         TEXT NOT NULL,
    severity            VARCHAR(10) NOT NULL
                        CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    status              VARCHAR(20) NOT NULL DEFAULT 'open'
                        CHECK (status IN ('open', 'acknowledged', 'investigating', 'escalated',
                                         'resolved', 'false_positive', 'suppressed', 'closed')),
    category            VARCHAR(100),
    subcategory         VARCHAR(100),
    source_type         VARCHAR(50) NOT NULL
                        CHECK (source_type IN ('rule_engine', 'agent', 'fim', 'vuln_scan',
                                              'threat_intel', 'anomaly_detection', 'manual',
                                              'integration', 'compliance')),
    source_id           VARCHAR(255),
    source_name         VARCHAR(255),
    agent_id            UUID REFERENCES agents(id) ON DELETE SET NULL,
    hostname            VARCHAR(255),
    ip_address          INET,
    affected_assets     JSONB DEFAULT '[]',
    mitre_tactic_id     VARCHAR(20),
    mitre_tactic_name   VARCHAR(100),
    mitre_technique_id  VARCHAR(20),
    mitre_technique_name VARCHAR(200),
    detection_rule_id   UUID,
    detection_rule_name VARCHAR(255),
    confidence_score    SMALLINT CHECK (confidence_score BETWEEN 0 AND 100),
    raw_event           JSONB,
    evidence            JSONB DEFAULT '[]',
    indicators          JSONB DEFAULT '[]',
    context             JSONB DEFAULT '{}',
    assigned_to         UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_at         TIMESTAMPTZ,
    escalated_to        UUID REFERENCES users(id) ON DELETE SET NULL,
    escalated_at        TIMESTAMPTZ,
    escalation_level    SMALLINT DEFAULT 0,
    correlation_id      UUID,
    parent_alert_id     UUID REFERENCES alerts(id) ON DELETE SET NULL,
    duplicate_of        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    child_count         INTEGER NOT NULL DEFAULT 0,
    incident_id         UUID,
    resolution_type     VARCHAR(30)
                        CHECK (resolution_type IN ('true_positive', 'false_positive', 'benign',
                                                  'duplicate', 'auto_resolved')),
    resolution_notes    TEXT,
    resolved_by         UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at         TIMESTAMPTZ,
    first_response_at   TIMESTAMPTZ,
    sla_breach          BOOLEAN NOT NULL DEFAULT false,
    sla_deadline        TIMESTAMPTZ,
    suppressed_by_rule  VARCHAR(255),
    suppressed_until    TIMESTAMPTZ,
    notifications_sent  BOOLEAN NOT NULL DEFAULT false,
    last_notified_at    TIMESTAMPTZ,
    triggered_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at           TIMESTAMPTZ
);

CREATE INDEX idx_alerts_tenant ON alerts(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_severity ON alerts(severity);
CREATE INDEX idx_alerts_severity_status ON alerts(severity, status) WHERE status NOT IN ('resolved', 'closed', 'false_positive');
CREATE INDEX idx_alerts_agent ON alerts(agent_id) WHERE agent_id IS NOT NULL;
CREATE INDEX idx_alerts_assigned ON alerts(assigned_to) WHERE assigned_to IS NOT NULL;
CREATE INDEX idx_alerts_created ON alerts(created_at DESC);
CREATE INDEX idx_alerts_triggered ON alerts(triggered_at DESC);
CREATE INDEX idx_alerts_correlation ON alerts(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_alerts_incident ON alerts(incident_id) WHERE incident_id IS NOT NULL;
CREATE INDEX idx_alerts_source ON alerts(source_type, source_id);
CREATE INDEX idx_alerts_key ON alerts(alert_key) WHERE alert_key IS NOT NULL;
CREATE INDEX idx_alerts_mitre ON alerts(mitre_tactic_id, mitre_technique_id) WHERE mitre_tactic_id IS NOT NULL;
CREATE INDEX idx_alerts_open_critical ON alerts(created_at DESC) WHERE severity IN ('critical', 'high') AND status IN ('open', 'acknowledged');

CREATE TRIGGER trg_alerts_updated_at
    BEFORE UPDATE ON alerts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE alerts IS 'Security alert events with severity classification, MITRE ATT&CK mapping, assignment workflow, and SLA tracking.';
COMMENT ON COLUMN alerts.alert_key IS 'Deduplication key. Alerts with same key within a time window are grouped.';
COMMENT ON COLUMN alerts.correlation_id IS 'UUID grouping related alerts from the same attack or incident.';
COMMENT ON COLUMN alerts.confidence_score IS '0-100 detection confidence score. Higher = more likely true positive.';
