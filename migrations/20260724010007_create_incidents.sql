-- Raksha Security Platform
-- Migration: 20260724010007_create_incidents
-- Description: Incident response tracking with timeline, tasks, and severity management

CREATE TABLE incidents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID REFERENCES tenants(id) ON DELETE CASCADE,
    incident_number     VARCHAR(20) NOT NULL,
    title               VARCHAR(500) NOT NULL,
    description         TEXT,
    severity            VARCHAR(10) NOT NULL
                        CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    status              VARCHAR(20) NOT NULL DEFAULT 'open'
                        CHECK (status IN ('open', 'triaging', 'investigating', 'containing',
                                         'eradicating', 'recovering', 'closed', 'post_mortem')),
    priority            VARCHAR(10) NOT NULL DEFAULT 'medium'
                        CHECK (priority IN ('low', 'medium', 'high', 'urgent', 'critical')),
    category            VARCHAR(100),
    subcategory         VARCHAR(100),
    classification      VARCHAR(50)
                        CHECK (classification IN ('true_incident', 'false_alarm', 'near_miss',
                                                 'policy_violation', 'undetermined')),
    commander_id        UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_team       VARCHAR(255),
    affected_systems    JSONB DEFAULT '[]',
    affected_users_count INTEGER DEFAULT 0,
    impact_scope        VARCHAR(30)
                        CHECK (impact_scope IN ('individual', 'team', 'department', 'organization', 'external')),
    mitre_tactics       JSONB DEFAULT '[]',
    mitre_techniques    JSONB DEFAULT '[]',
    attack_vector       VARCHAR(100),
    ioc_summary         JSONB DEFAULT '[]',
    containment_actions JSONB DEFAULT '[]',
    root_cause          TEXT,
    lessons_learned     TEXT,
    remediation_plan    TEXT,
    external_ref        VARCHAR(255),
    external_url        TEXT,
    sla_response_mins   INTEGER,
    sla_resolve_mins    INTEGER,
    sla_breached        BOOLEAN NOT NULL DEFAULT false,
    first_detected_at   TIMESTAMPTZ,
    first_response_at   TIMESTAMPTZ,
    contained_at        TIMESTAMPTZ,
    eradicated_at       TIMESTAMPTZ,
    recovered_at        TIMESTAMPTZ,
    closed_at           TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_incident_number_tenant UNIQUE (incident_number, tenant_id)
);

CREATE INDEX idx_incidents_tenant ON incidents(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_incidents_status ON incidents(status);
CREATE INDEX idx_incidents_severity ON incidents(severity);
CREATE INDEX idx_incidents_commander ON incidents(commander_id) WHERE commander_id IS NOT NULL;
CREATE INDEX idx_incidents_created ON incidents(created_at DESC);
CREATE INDEX idx_incidents_open ON incidents(severity, created_at DESC) WHERE status NOT IN ('closed', 'post_mortem');
CREATE INDEX idx_incidents_number ON incidents(incident_number);

CREATE TRIGGER trg_incidents_updated_at
    BEFORE UPDATE ON incidents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE incidents IS 'Security incident tracking with full lifecycle management, MITRE mapping, SLA tracking, and post-mortem support.';
COMMENT ON COLUMN incidents.incident_number IS 'Human-readable incident identifier (e.g., INC-2026-0042). Unique per tenant.';
COMMENT ON COLUMN incidents.commander_id IS 'Incident commander responsible for coordination and decision-making.';

-- Add FK from alerts to incidents now that incidents table exists
ALTER TABLE alerts
    ADD CONSTRAINT fk_alerts_incident
    FOREIGN KEY (incident_id) REFERENCES incidents(id) ON DELETE SET NULL;

-- Incident timeline events
CREATE TABLE incident_timeline (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id     UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    actor_id        UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type      VARCHAR(30) NOT NULL
                    CHECK (event_type IN ('created', 'status_changed', 'severity_changed',
                                         'assigned', 'comment', 'evidence_added', 'alert_linked',
                                         'task_created', 'task_completed', 'containment_action',
                                         'communication_sent', 'escalated', 'closed')),
    title           VARCHAR(500) NOT NULL,
    content         TEXT,
    metadata        JSONB DEFAULT '{}',
    is_automated    BOOLEAN NOT NULL DEFAULT false,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_incident_timeline_incident ON incident_timeline(incident_id, occurred_at);
CREATE INDEX idx_incident_timeline_type ON incident_timeline(event_type);

COMMENT ON TABLE incident_timeline IS 'Chronological timeline of all incident activities for investigation replay.';

-- Incident tasks (remediation/investigation tasks)
CREATE TABLE incident_tasks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id     UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    title           VARCHAR(500) NOT NULL,
    description     TEXT,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'in_progress', 'completed', 'cancelled', 'blocked')),
    priority        VARCHAR(10) NOT NULL DEFAULT 'medium'
                    CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    assigned_to     UUID REFERENCES users(id) ON DELETE SET NULL,
    due_at          TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    completed_by    UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_incident_tasks_incident ON incident_tasks(incident_id);
CREATE INDEX idx_incident_tasks_assigned ON incident_tasks(assigned_to) WHERE status IN ('pending', 'in_progress');
CREATE INDEX idx_incident_tasks_status ON incident_tasks(status);

CREATE TRIGGER trg_incident_tasks_updated_at
    BEFORE UPDATE ON incident_tasks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE incident_tasks IS 'Actionable tasks within an incident for tracking remediation and investigation work.';
