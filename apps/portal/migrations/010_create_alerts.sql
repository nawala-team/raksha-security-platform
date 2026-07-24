-- Migration: 010_create_alerts
-- Description: Create alerts and alert_rules tables
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for alerts
CREATE TYPE alert_severity AS ENUM (
    'info',
    'low',
    'medium',
    'high',
    'critical'
);

CREATE TYPE alert_status AS ENUM (
    'open',
    'acknowledged',
    'investigating',
    'resolved',
    'false_positive',
    'suppressed'
);

CREATE TYPE alert_rule_status AS ENUM (
    'active',
    'disabled',
    'testing'
);

-- Alert rules define conditions that trigger alerts
CREATE TABLE alert_rules (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    severity        alert_severity NOT NULL DEFAULT 'medium',
    condition_type  VARCHAR(100) NOT NULL,
    condition_config JSONB NOT NULL,
    throttle_minutes INTEGER DEFAULT 60,
    notification_channels JSONB DEFAULT '[]',
    tags            JSONB DEFAULT '[]',
    org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
    status          alert_rule_status NOT NULL DEFAULT 'active',
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_throttle_positive CHECK (throttle_minutes >= 0)
);

-- Alerts triggered by rules or manual creation
CREATE TABLE alerts (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id         UUID REFERENCES alert_rules(id) ON DELETE SET NULL,
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    severity        alert_severity NOT NULL,
    status          alert_status NOT NULL DEFAULT 'open',
    source          VARCHAR(255) NOT NULL,
    source_ref      VARCHAR(255),
    agent_id        UUID REFERENCES agents(id) ON DELETE SET NULL,
    org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
    assigned_to     UUID REFERENCES users(id) ON DELETE SET NULL,
    acknowledged_by UUID REFERENCES users(id) ON DELETE SET NULL,
    acknowledged_at TIMESTAMPTZ,
    resolved_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at     TIMESTAMPTZ,
    resolution_note TEXT,
    context         JSONB DEFAULT '{}',
    tags            JSONB DEFAULT '[]',
    related_alerts  UUID[],
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_occurrence_count_positive CHECK (occurrence_count > 0),
    CONSTRAINT chk_acknowledged_consistency CHECK (
        (acknowledged_by IS NULL AND acknowledged_at IS NULL) OR
        (acknowledged_by IS NOT NULL AND acknowledged_at IS NOT NULL)
    ),
    CONSTRAINT chk_resolved_consistency CHECK (
        (resolved_by IS NULL AND resolved_at IS NULL) OR
        (resolved_by IS NOT NULL AND resolved_at IS NOT NULL)
    )
);

-- Alert comments/timeline
CREATE TABLE alert_comments (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    alert_id    UUID NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
    author_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    content     TEXT NOT NULL,
    is_system   BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for alert_rules
CREATE INDEX idx_alert_rules_org_id ON alert_rules (org_id);
CREATE INDEX idx_alert_rules_status ON alert_rules (status);
CREATE INDEX idx_alert_rules_severity ON alert_rules (severity);
CREATE INDEX idx_alert_rules_condition_type ON alert_rules (condition_type);

-- Indexes for alerts
CREATE INDEX idx_alerts_rule_id ON alerts (rule_id);
CREATE INDEX idx_alerts_severity ON alerts (severity);
CREATE INDEX idx_alerts_status ON alerts (status);
CREATE INDEX idx_alerts_source ON alerts (source);
CREATE INDEX idx_alerts_agent_id ON alerts (agent_id);
CREATE INDEX idx_alerts_org_id ON alerts (org_id);
CREATE INDEX idx_alerts_assigned_to ON alerts (assigned_to);
CREATE INDEX idx_alerts_created_at ON alerts (created_at DESC);
CREATE INDEX idx_alerts_open_severity ON alerts (severity, created_at DESC) WHERE status = 'open';
CREATE INDEX idx_alerts_context ON alerts USING GIN (context);
CREATE INDEX idx_alerts_tags ON alerts USING GIN (tags);

-- Indexes for alert_comments
CREATE INDEX idx_alert_comments_alert_id ON alert_comments (alert_id, created_at);

-- Triggers
CREATE TRIGGER set_alert_rules_updated_at
    BEFORE UPDATE ON alert_rules
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_alerts_updated_at
    BEFORE UPDATE ON alerts
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE alert_rules IS 'Configurable rules that define conditions for triggering security alerts';
COMMENT ON TABLE alerts IS 'Security alerts generated by rules, agents, or external integrations';
COMMENT ON TABLE alert_comments IS 'Timeline of comments and system events on alerts';
COMMENT ON COLUMN alerts.context IS 'Additional context data specific to the alert type (JSONB)';
COMMENT ON COLUMN alerts.related_alerts IS 'Array of related alert IDs for correlation';
COMMENT ON COLUMN alerts.occurrence_count IS 'Number of times this alert has been triggered (deduplicated)';
COMMENT ON COLUMN alert_rules.condition_config IS 'Rule condition configuration (threshold, pattern, query)';
COMMENT ON COLUMN alert_rules.throttle_minutes IS 'Minimum minutes between repeated alerts from this rule';
