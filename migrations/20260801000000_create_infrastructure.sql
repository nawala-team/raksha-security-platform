-- Tier 3 infrastructure & monitoring surfaces
--
-- Backs the dashboard pages that previously had no persistence at all:
-- servers, network, containers, honeypots, dark web, threat hunting,
-- backups and documents.
--
-- Design notes:
--  * Every table is tenant-scoped with ON DELETE CASCADE, matching the
--    convention established by the GRC and incident migrations.
--  * Where a concept is already represented by an enrolled agent, the table
--    references `agents(id)` instead of duplicating host identity.
--  * CHECK constraints encode the allowed enum values as plain VARCHAR,
--    following the newer migrations rather than creating more PG enum types.

-- ============================================================
-- Servers / infrastructure inventory
-- ============================================================
-- A server is the logical host record. When an agent is installed the rows are
-- linked, but a server can exist without an agent (agentless inventory).

CREATE TABLE IF NOT EXISTS servers (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id        UUID REFERENCES agents(id) ON DELETE SET NULL,
    hostname        VARCHAR(255) NOT NULL,
    display_name    VARCHAR(255),
    environment     VARCHAR(20) NOT NULL DEFAULT 'production' CHECK (environment IN (
        'production', 'staging', 'development', 'test'
    )),
    role            VARCHAR(50),
    provider        VARCHAR(50),
    region          VARCHAR(100),
    ip_address      INET,
    public_ip       INET,
    os_family       VARCHAR(50),
    os_version      VARCHAR(100),
    cpu_cores       INTEGER,
    memory_mb       INTEGER,
    disk_gb         INTEGER,
    status          VARCHAR(20) NOT NULL DEFAULT 'unknown' CHECK (status IN (
        'online', 'offline', 'degraded', 'maintenance', 'unknown'
    )),
    cpu_usage_pct   DOUBLE PRECISION,
    memory_usage_pct DOUBLE PRECISION,
    disk_usage_pct  DOUBLE PRECISION,
    uptime_secs     BIGINT,
    tags            JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, hostname)
);

CREATE INDEX IF NOT EXISTS idx_servers_tenant ON servers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_servers_status ON servers(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_servers_agent ON servers(agent_id);

-- ============================================================
-- Network events & firewall rules
-- ============================================================

CREATE TABLE IF NOT EXISTS network_events (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id        UUID REFERENCES agents(id) ON DELETE SET NULL,
    event_type      VARCHAR(30) NOT NULL CHECK (event_type IN (
        'connection', 'blocked', 'allowed', 'port_scan', 'dns_query',
        'anomaly', 'intrusion_attempt'
    )),
    severity        VARCHAR(20) NOT NULL DEFAULT 'info' CHECK (severity IN (
        'info', 'low', 'medium', 'high', 'critical'
    )),
    protocol        VARCHAR(10),
    source_ip       INET,
    source_port     INTEGER,
    dest_ip         INET,
    dest_port       INTEGER,
    direction       VARCHAR(10) CHECK (direction IN ('inbound', 'outbound', 'internal')),
    action          VARCHAR(20) CHECK (action IN ('allow', 'block', 'drop', 'reject', 'log')),
    bytes_sent      BIGINT,
    bytes_received  BIGINT,
    packet_count    BIGINT,
    process_name    VARCHAR(255),
    rule_id         UUID,
    country_code    CHAR(2),
    asn             VARCHAR(50),
    is_threat       BOOLEAN NOT NULL DEFAULT FALSE,
    alert_id        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_network_events_tenant_time
    ON network_events(tenant_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_network_events_type ON network_events(tenant_id, event_type);
CREATE INDEX IF NOT EXISTS idx_network_events_threat
    ON network_events(tenant_id, occurred_at DESC) WHERE is_threat = TRUE;

CREATE TABLE IF NOT EXISTS network_rules (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    is_enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    priority        INTEGER NOT NULL DEFAULT 100,
    direction       VARCHAR(10) NOT NULL CHECK (direction IN ('inbound', 'outbound', 'both')),
    action          VARCHAR(20) NOT NULL CHECK (action IN ('allow', 'block', 'drop', 'reject', 'log')),
    protocol        VARCHAR(10),
    source_cidr     VARCHAR(64),
    dest_cidr       VARCHAR(64),
    port_range      VARCHAR(64),
    applies_to      JSONB NOT NULL DEFAULT '[]'::jsonb,
    hit_count       BIGINT NOT NULL DEFAULT 0,
    last_hit_at     TIMESTAMPTZ,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_network_rules_tenant ON network_rules(tenant_id, priority);


-- ============================================================
-- Container inventory & image scanning
-- ============================================================

CREATE TABLE IF NOT EXISTS containers (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id        UUID REFERENCES agents(id) ON DELETE SET NULL,
    server_id       UUID REFERENCES servers(id) ON DELETE SET NULL,
    container_id    VARCHAR(128) NOT NULL,
    name            VARCHAR(255) NOT NULL,
    image           VARCHAR(512) NOT NULL,
    image_tag       VARCHAR(128),
    image_digest    VARCHAR(128),
    runtime         VARCHAR(20) NOT NULL DEFAULT 'docker' CHECK (runtime IN (
        'docker', 'containerd', 'podman', 'cri-o'
    )),
    orchestrator    VARCHAR(20) CHECK (orchestrator IN ('kubernetes', 'swarm', 'nomad', 'none')),
    namespace       VARCHAR(255),
    pod_name        VARCHAR(255),
    status          VARCHAR(20) NOT NULL DEFAULT 'unknown' CHECK (status IN (
        'running', 'stopped', 'paused', 'restarting', 'exited', 'unknown'
    )),
    privileged      BOOLEAN NOT NULL DEFAULT FALSE,
    root_user       BOOLEAN NOT NULL DEFAULT FALSE,
    host_network    BOOLEAN NOT NULL DEFAULT FALSE,
    exposed_ports   JSONB NOT NULL DEFAULT '[]'::jsonb,
    mounts          JSONB NOT NULL DEFAULT '[]'::jsonb,
    env_risk_count  INTEGER NOT NULL DEFAULT 0,
    cpu_usage_pct   DOUBLE PRECISION,
    memory_mb       INTEGER,
    -- Denormalised counts from the most recent image scan, so the list view
    -- does not need a join per row.
    critical_vulns  INTEGER NOT NULL DEFAULT 0,
    high_vulns      INTEGER NOT NULL DEFAULT 0,
    medium_vulns    INTEGER NOT NULL DEFAULT 0,
    low_vulns       INTEGER NOT NULL DEFAULT 0,
    compliance_score DOUBLE PRECISION,
    labels          JSONB NOT NULL DEFAULT '{}'::jsonb,
    started_at      TIMESTAMPTZ,
    last_scanned_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, container_id)
);

CREATE INDEX IF NOT EXISTS idx_containers_tenant ON containers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_containers_status ON containers(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_containers_risk
    ON containers(tenant_id, critical_vulns DESC, high_vulns DESC);

CREATE TABLE IF NOT EXISTS container_image_scans (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    image           VARCHAR(512) NOT NULL,
    image_digest    VARCHAR(128),
    scanner         VARCHAR(50) NOT NULL DEFAULT 'trivy',
    status          VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending', 'running', 'completed', 'failed'
    )),
    critical_count  INTEGER NOT NULL DEFAULT 0,
    high_count      INTEGER NOT NULL DEFAULT 0,
    medium_count    INTEGER NOT NULL DEFAULT 0,
    low_count       INTEGER NOT NULL DEFAULT 0,
    fixable_count   INTEGER NOT NULL DEFAULT 0,
    secrets_found   INTEGER NOT NULL DEFAULT 0,
    misconfigs      INTEGER NOT NULL DEFAULT 0,
    findings        JSONB NOT NULL DEFAULT '[]'::jsonb,
    duration_secs   INTEGER,
    error_message   TEXT,
    initiated_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_container_scans_tenant
    ON container_image_scans(tenant_id, started_at DESC);

-- ============================================================
-- Honeypots & captured interactions
-- ============================================================

CREATE TABLE IF NOT EXISTS honeypots (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    honeypot_type   VARCHAR(30) NOT NULL CHECK (honeypot_type IN (
        'ssh', 'http', 'ftp', 'smb', 'rdp', 'telnet', 'database', 'custom'
    )),
    status          VARCHAR(20) NOT NULL DEFAULT 'stopped' CHECK (status IN (
        'running', 'stopped', 'error', 'deploying'
    )),
    listen_ip       INET,
    listen_port     INTEGER NOT NULL,
    server_id       UUID REFERENCES servers(id) ON DELETE SET NULL,
    emulated_banner VARCHAR(512),
    interaction_count   BIGINT NOT NULL DEFAULT 0,
    unique_attackers    BIGINT NOT NULL DEFAULT 0,
    last_interaction_at TIMESTAMPTZ,
    config          JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_honeypots_tenant ON honeypots(tenant_id, status);

CREATE TABLE IF NOT EXISTS honeypot_interactions (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    honeypot_id     UUID NOT NULL REFERENCES honeypots(id) ON DELETE CASCADE,
    source_ip       INET NOT NULL,
    source_port     INTEGER,
    country_code    CHAR(2),
    asn             VARCHAR(50),
    session_id      VARCHAR(128),
    interaction_type VARCHAR(30) NOT NULL CHECK (interaction_type IN (
        'connection', 'login_attempt', 'command', 'file_upload',
        'file_download', 'exploit_attempt', 'scan'
    )),
    username_tried  VARCHAR(255),
    -- Stored to reveal which credential lists attackers use. These are
    -- attacker-supplied values against a decoy, never real user credentials.
    password_tried  VARCHAR(255),
    commands        JSONB NOT NULL DEFAULT '[]'::jsonb,
    payload         TEXT,
    payload_hash    VARCHAR(64),
    severity        VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (severity IN (
        'info', 'low', 'medium', 'high', 'critical'
    )),
    matched_indicator_id UUID REFERENCES threat_indicators(id) ON DELETE SET NULL,
    alert_id        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_honeypot_interactions_hp
    ON honeypot_interactions(honeypot_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_honeypot_interactions_ip
    ON honeypot_interactions(tenant_id, source_ip);

