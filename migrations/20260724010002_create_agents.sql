-- Raksha Security Platform
-- Migration: 20260724010002_create_agents
-- Description: Agent enrollment, lifecycle management, fingerprints, and health tracking

CREATE TABLE agents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name                VARCHAR(255) NOT NULL,
    hostname            VARCHAR(255) NOT NULL,
    fqdn                VARCHAR(500),
    os_type             VARCHAR(20) NOT NULL
                        CHECK (os_type IN ('linux', 'windows', 'macos', 'freebsd', 'openbsd')),
    os_version          VARCHAR(100),
    os_arch             VARCHAR(20) NOT NULL DEFAULT 'x86_64'
                        CHECK (os_arch IN ('x86_64', 'aarch64', 'armv7', 'i686', 'ppc64le', 's390x')),
    kernel_version      VARCHAR(100),
    agent_version       VARCHAR(50) NOT NULL,
    agent_build         VARCHAR(100),
    update_channel      VARCHAR(20) NOT NULL DEFAULT 'stable'
                        CHECK (update_channel IN ('stable', 'beta', 'nightly', 'pinned')),
    pinned_version      VARCHAR(50),
    last_update_at      TIMESTAMPTZ,
    status              VARCHAR(30) NOT NULL DEFAULT 'enrolling'
                        CHECK (status IN ('online', 'offline', 'degraded', 'updating',
                                         'enrolling', 'decommissioned', 'quarantined', 'unreachable')),
    health_score        SMALLINT DEFAULT 100 CHECK (health_score BETWEEN 0 AND 100),
    last_heartbeat_at   TIMESTAMPTZ,
    last_seen_at        TIMESTAMPTZ,
    disconnect_reason   VARCHAR(255),
    primary_ip          INET,
    secondary_ips       INET[],
    mac_addresses       TEXT[],
    network_zone        VARCHAR(100),
    proxy_endpoint      VARCHAR(500),
    fingerprint_hash    VARCHAR(64) NOT NULL,
    cpu_model           VARCHAR(200),
    cpu_cores           SMALLINT,
    memory_mb           INTEGER,
    disk_total_gb       INTEGER,
    serial_number       VARCHAR(255),
    bios_uuid           VARCHAR(100),
    enrollment_token_hash TEXT NOT NULL,
    certificate_serial    VARCHAR(255),
    certificate_expires   TIMESTAMPTZ,
    auth_method         VARCHAR(20) NOT NULL DEFAULT 'token'
                        CHECK (auth_method IN ('token', 'mtls', 'certificate')),
    active_modules      TEXT[] NOT NULL DEFAULT ARRAY['heartbeat', 'metrics'],
    config              JSONB NOT NULL DEFAULT '{}',
    labels              JSONB NOT NULL DEFAULT '{}',
    tags                TEXT[] NOT NULL DEFAULT '{}',
    agent_group_id      UUID,
    environment         VARCHAR(50) CHECK (environment IN ('production', 'staging', 'development', 'testing')),
    enrolled_by         UUID REFERENCES users(id),
    enrolled_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    enrollment_method   VARCHAR(30) DEFAULT 'manual'
                        CHECK (enrollment_method IN ('manual', 'auto_discovery', 'api', 'fleet_deploy', 'image_baked')),
    decommissioned_at   TIMESTAMPTZ,
    decommissioned_by   UUID REFERENCES users(id),
    decommission_reason TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_agents_hostname_tenant UNIQUE (hostname, tenant_id)
);

CREATE INDEX idx_agents_tenant ON agents(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_os_type ON agents(os_type);
CREATE INDEX idx_agents_last_heartbeat ON agents(last_heartbeat_at DESC);
CREATE INDEX idx_agents_health ON agents(health_score) WHERE status NOT IN ('decommissioned');
CREATE INDEX idx_agents_fingerprint ON agents(fingerprint_hash);
CREATE INDEX idx_agents_group ON agents(agent_group_id) WHERE agent_group_id IS NOT NULL;
CREATE INDEX idx_agents_environment ON agents(environment) WHERE environment IS NOT NULL;
CREATE INDEX idx_agents_labels ON agents USING GIN (labels);
CREATE INDEX idx_agents_tags ON agents USING GIN (tags);
CREATE INDEX idx_agents_active_modules ON agents USING GIN (active_modules);

CREATE TRIGGER trg_agents_updated_at
    BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE agents IS 'Enrolled security agents with hardware fingerprinting, health tracking, and module configuration.';
COMMENT ON COLUMN agents.fingerprint_hash IS 'SHA-256 hash of hardware identifiers. Used to detect agent cloning or VM migration.';
COMMENT ON COLUMN agents.health_score IS '0-100 composite health score from heartbeat freshness, resource usage, and module status.';
