-- Migration: 008_create_agents
-- Description: Create agents table for security agent management
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for agent status
CREATE TYPE agent_status AS ENUM (
    'online',
    'offline',
    'degraded',
    'updating',
    'enrolling',
    'decommissioned'
);

CREATE TYPE agent_os AS ENUM (
    'linux',
    'windows',
    'macos',
    'freebsd'
);

CREATE TYPE agent_arch AS ENUM (
    'x86_64',
    'aarch64',
    'armv7',
    'x86'
);

CREATE TABLE agents (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(255) NOT NULL,
    hostname        VARCHAR(255) NOT NULL,
    os              agent_os NOT NULL,
    arch            agent_arch NOT NULL,
    version         VARCHAR(50) NOT NULL,
    status          agent_status NOT NULL DEFAULT 'enrolling',
    last_seen       TIMESTAMPTZ,
    enrolled_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    enrolled_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    token_hash      TEXT NOT NULL UNIQUE,
    modules         JSONB NOT NULL DEFAULT '[]',
    config          JSONB NOT NULL DEFAULT '{}',
    tags            JSONB DEFAULT '[]',
    org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
    ip_address      INET,
    network_zone    VARCHAR(100),
    cpu_cores       INTEGER,
    memory_mb       INTEGER,
    disk_gb         INTEGER,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_agent_version_format CHECK (version ~ '^\d+\.\d+\.\d+(-[a-z0-9.]+)?$')
);

-- Agent groups for bulk management
CREATE TABLE agent_groups (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL,
    description TEXT,
    org_id      UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    filter_rules JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_agent_group_name_org UNIQUE (name, org_id)
);

CREATE TABLE agent_group_members (
    agent_id    UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    group_id    UUID NOT NULL REFERENCES agent_groups(id) ON DELETE CASCADE,
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (agent_id, group_id)
);

-- Indexes
CREATE INDEX idx_agents_hostname ON agents (hostname);
CREATE INDEX idx_agents_status ON agents (status);
CREATE INDEX idx_agents_last_seen ON agents (last_seen);
CREATE INDEX idx_agents_org_id ON agents (org_id);
CREATE INDEX idx_agents_version ON agents (version);
CREATE INDEX idx_agents_os ON agents (os);
CREATE INDEX idx_agents_token_hash ON agents (token_hash);
CREATE INDEX idx_agents_online ON agents (org_id, status) WHERE status = 'online';
CREATE INDEX idx_agents_stale ON agents (last_seen) WHERE status = 'online';
CREATE INDEX idx_agent_groups_org_id ON agent_groups (org_id);
CREATE INDEX idx_agent_group_members_group ON agent_group_members (group_id);

-- Triggers
CREATE TRIGGER set_agents_updated_at
    BEFORE UPDATE ON agents
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_agent_groups_updated_at
    BEFORE UPDATE ON agent_groups
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE agents IS 'Registered security monitoring agents deployed across infrastructure';
COMMENT ON COLUMN agents.token_hash IS 'SHA-256 hash of agent enrollment token for authentication';
COMMENT ON COLUMN agents.modules IS 'JSON array of active modules (e.g., ["file_integrity", "network_monitor", "log_collector"])';
COMMENT ON COLUMN agents.config IS 'Agent configuration as JSON (scan intervals, thresholds, exclusions)';
COMMENT ON COLUMN agents.network_zone IS 'Network security zone where agent operates (e.g., dmz, internal, production)';
COMMENT ON TABLE agent_groups IS 'Logical groups for bulk agent management and policy targeting';
