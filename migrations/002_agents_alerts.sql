-- Agents table
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    os agent_os NOT NULL,
    version VARCHAR(50) NOT NULL,
    status agent_status NOT NULL DEFAULT 'enrolling',
    last_seen TIMESTAMPTZ,
    enrolled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    enrolled_by UUID REFERENCES users(id),
    token_hash TEXT NOT NULL,
    modules JSONB NOT NULL DEFAULT '[]',
    config JSONB NOT NULL DEFAULT '{}',
    tags JSONB NOT NULL DEFAULT '[]',
    org_id UUID,
    ip_address INET,
    network_zone VARCHAR(100),
    cpu_cores INTEGER,
    memory_mb INTEGER,
    disk_gb INTEGER,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_last_seen ON agents(last_seen);

-- Agent metrics
CREATE TABLE agent_metrics (
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    metric_name VARCHAR(255) NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_agent_metrics_agent_time ON agent_metrics(agent_id, timestamp DESC);

-- Alerts table
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    severity alert_severity NOT NULL,
    status alert_status NOT NULL DEFAULT 'open',
    source VARCHAR(255) NOT NULL,
    source_id VARCHAR(255),
    agent_id UUID REFERENCES agents(id),
    assigned_to UUID REFERENCES users(id),
    rule_id UUID,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_severity ON alerts(severity);
CREATE INDEX idx_alerts_created_at ON alerts(created_at DESC);
