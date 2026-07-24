-- Raksha Security Platform
-- Migration: 20260724010005_create_fim_events
-- Description: File Integrity Monitoring events tracking file changes across agents

CREATE TABLE fim_events (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id            UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    hostname            VARCHAR(255) NOT NULL,
    event_type          VARCHAR(20) NOT NULL
                        CHECK (event_type IN ('created', 'modified', 'deleted', 'renamed',
                                            'permissions_changed', 'ownership_changed', 'attributes_changed')),
    severity            VARCHAR(10) NOT NULL DEFAULT 'medium'
                        CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    file_path           TEXT NOT NULL,
    file_name           VARCHAR(500) NOT NULL,
    directory           TEXT NOT NULL,
    old_path            TEXT,
    file_type           VARCHAR(30)
                        CHECK (file_type IN ('file', 'directory', 'symlink', 'hardlink', 'pipe', 'socket', 'block_device', 'char_device')),
    file_size           BIGINT,
    old_file_size       BIGINT,
    hash_algorithm      VARCHAR(10) NOT NULL DEFAULT 'sha256'
                        CHECK (hash_algorithm IN ('sha256', 'sha512', 'sha1', 'md5', 'blake3')),
    hash_before         VARCHAR(128),
    hash_after          VARCHAR(128),
    content_changed     BOOLEAN,
    diff_available      BOOLEAN NOT NULL DEFAULT false,
    diff_content        TEXT,
    permissions_before  VARCHAR(20),
    permissions_after   VARCHAR(20),
    owner_before        VARCHAR(100),
    owner_after         VARCHAR(100),
    group_before        VARCHAR(100),
    group_after         VARCHAR(100),
    mtime_before        TIMESTAMPTZ,
    mtime_after         TIMESTAMPTZ,
    inode               BIGINT,
    process_id          INTEGER,
    process_name        VARCHAR(255),
    process_user        VARCHAR(100),
    process_command     TEXT,
    rule_id             VARCHAR(255),
    rule_name           VARCHAR(255),
    baseline_id         UUID,
    is_baseline_drift   BOOLEAN NOT NULL DEFAULT false,
    is_whitelisted      BOOLEAN NOT NULL DEFAULT false,
    whitelist_rule      VARCHAR(255),
    alert_id            UUID REFERENCES alerts(id) ON DELETE SET NULL,
    tags                TEXT[] DEFAULT '{}',
    metadata            JSONB DEFAULT '{}',
    detected_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fim_events_tenant ON fim_events(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_fim_events_agent ON fim_events(agent_id);
CREATE INDEX idx_fim_events_path ON fim_events(file_path);
CREATE INDEX idx_fim_events_type ON fim_events(event_type);
CREATE INDEX idx_fim_events_severity ON fim_events(severity) WHERE severity IN ('high', 'critical');
CREATE INDEX idx_fim_events_detected ON fim_events(detected_at DESC);
CREATE INDEX idx_fim_events_agent_time ON fim_events(agent_id, detected_at DESC);
CREATE INDEX idx_fim_events_baseline_drift ON fim_events(agent_id) WHERE is_baseline_drift = true;
CREATE INDEX idx_fim_events_directory ON fim_events(directory);
CREATE INDEX idx_fim_events_hash_after ON fim_events(hash_after) WHERE hash_after IS NOT NULL;

COMMENT ON TABLE fim_events IS 'File Integrity Monitoring events capturing file system changes with before/after state, process attribution, and baseline drift detection.';
COMMENT ON COLUMN fim_events.hash_before IS 'Cryptographic hash of file content before the change. NULL for newly created files.';
COMMENT ON COLUMN fim_events.hash_after IS 'Cryptographic hash of file content after the change. NULL for deleted files.';
COMMENT ON COLUMN fim_events.is_baseline_drift IS 'True if this change deviates from the approved baseline snapshot.';
COMMENT ON COLUMN fim_events.diff_content IS 'Unified diff of text file changes. Only stored for small text files when configured.';

-- FIM baselines table: stores approved file state snapshots
CREATE TABLE fim_baselines (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id        UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    status          VARCHAR(20) NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'archived', 'building', 'failed')),
    paths           TEXT[] NOT NULL,
    exclude_paths   TEXT[] DEFAULT '{}',
    file_count      INTEGER NOT NULL DEFAULT 0,
    total_size_bytes BIGINT NOT NULL DEFAULT 0,
    snapshot_hash   VARCHAR(64),
    approved_by     UUID REFERENCES users(id),
    approved_at     TIMESTAMPTZ,
    built_at        TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fim_baselines_agent ON fim_baselines(agent_id) WHERE status = 'active';
CREATE INDEX idx_fim_baselines_tenant ON fim_baselines(tenant_id);

CREATE TRIGGER trg_fim_baselines_updated_at
    BEFORE UPDATE ON fim_baselines
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE fim_baselines IS 'Approved file integrity baselines per agent. Used to detect drift from known-good state.';
