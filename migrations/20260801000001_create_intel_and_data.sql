-- Tier 3 intelligence & data-protection surfaces
--
-- Dark web monitoring, saved threat-hunting queries, backup jobs and
-- document/evidence management.
--
-- Note on hunting: the RQL executor in raksha-core targets OpenSearch indices
-- (`raksha-events-*`), so these tables persist the saved queries, schedules and
-- run history only. Query results are not duplicated into Postgres.

-- ============================================================
-- Dark web monitoring
-- ============================================================
-- A "monitor" is a standing watch for one asset (domain, email, brand, etc).
-- Findings are the individual exposures discovered for that monitor.

CREATE TABLE IF NOT EXISTS darkweb_monitors (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    monitor_type    VARCHAR(30) NOT NULL CHECK (monitor_type IN (
        'domain', 'email', 'credential', 'brand', 'ip_range',
        'keyword', 'bin_card', 'executive'
    )),
    keyword         VARCHAR(512) NOT NULL,
    is_enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    severity_floor  VARCHAR(20) NOT NULL DEFAULT 'low' CHECK (severity_floor IN (
        'info', 'low', 'medium', 'high', 'critical'
    )),
    sources         JSONB NOT NULL DEFAULT '[]'::jsonb,
    finding_count   BIGINT NOT NULL DEFAULT 0,
    new_finding_count BIGINT NOT NULL DEFAULT 0,
    last_scanned_at TIMESTAMPTZ,
    next_scan_at    TIMESTAMPTZ,
    scan_interval_mins INTEGER NOT NULL DEFAULT 1440,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, monitor_type, keyword)
);

CREATE INDEX IF NOT EXISTS idx_darkweb_monitors_tenant
    ON darkweb_monitors(tenant_id, is_enabled);

CREATE TABLE IF NOT EXISTS darkweb_findings (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    monitor_id      UUID NOT NULL REFERENCES darkweb_monitors(id) ON DELETE CASCADE,
    title           VARCHAR(512) NOT NULL,
    description     TEXT,
    finding_type    VARCHAR(30) NOT NULL CHECK (finding_type IN (
        'credential_leak', 'data_dump', 'ransomware_mention', 'brand_abuse',
        'insider_threat', 'exploit_sale', 'phishing_kit', 'chatter'
    )),
    severity        VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (severity IN (
        'info', 'low', 'medium', 'high', 'critical'
    )),
    status          VARCHAR(20) NOT NULL DEFAULT 'new' CHECK (status IN (
        'new', 'triaging', 'confirmed', 'false_positive', 'remediated'
    )),
    source_name     VARCHAR(255),
    source_type     VARCHAR(30) CHECK (source_type IN (
        'forum', 'marketplace', 'paste_site', 'telegram', 'irc',
        'ransomware_blog', 'other'
    )),
    -- Deliberately no raw onion URLs or credential values: store a redacted
    -- excerpt and a hash so analysts can correlate without the platform
    -- becoming a secondary copy of leaked secrets.
    source_reference VARCHAR(512),
    excerpt_redacted TEXT,
    record_count    INTEGER,
    exposed_fields  JSONB NOT NULL DEFAULT '[]'::jsonb,
    content_hash    VARCHAR(64),
    confidence      SMALLINT CHECK (confidence BETWEEN 0 AND 100),
    alert_id        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    incident_id     UUID REFERENCES incidents(id) ON DELETE SET NULL,
    triaged_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    triaged_at      TIMESTAMPTZ,
    discovered_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_darkweb_findings_tenant
    ON darkweb_findings(tenant_id, discovered_at DESC);
CREATE INDEX IF NOT EXISTS idx_darkweb_findings_monitor
    ON darkweb_findings(monitor_id, discovered_at DESC);
CREATE INDEX IF NOT EXISTS idx_darkweb_findings_status
    ON darkweb_findings(tenant_id, status, severity);

-- ============================================================
-- Threat hunting: saved RQL queries, schedules and run history
-- ============================================================

CREATE TABLE IF NOT EXISTS hunting_queries (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    rql             TEXT NOT NULL,
    query_source    VARCHAR(20) NOT NULL DEFAULT 'events' CHECK (query_source IN (
        'events', 'alerts', 'agents', 'network'
    )),
    tags            JSONB NOT NULL DEFAULT '[]'::jsonb,
    mitre_techniques JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_scheduled    BOOLEAN NOT NULL DEFAULT FALSE,
    -- Interval-based scheduling keeps the scheduler simple; cron can be layered
    -- on later without changing the run-history shape.
    schedule_interval_mins INTEGER,
    alert_on_hits   BOOLEAN NOT NULL DEFAULT FALSE,
    alert_threshold INTEGER NOT NULL DEFAULT 1,
    alert_severity  VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (alert_severity IN (
        'info', 'low', 'medium', 'high', 'critical'
    )),
    last_run_at     TIMESTAMPTZ,
    next_run_at     TIMESTAMPTZ,
    last_hit_count  BIGINT,
    run_count       BIGINT NOT NULL DEFAULT 0,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_hunting_queries_tenant ON hunting_queries(tenant_id);
CREATE INDEX IF NOT EXISTS idx_hunting_queries_due
    ON hunting_queries(next_run_at) WHERE is_scheduled = TRUE;

CREATE TABLE IF NOT EXISTS hunting_runs (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    query_id        UUID NOT NULL REFERENCES hunting_queries(id) ON DELETE CASCADE,
    trigger         VARCHAR(20) NOT NULL DEFAULT 'manual' CHECK (trigger IN (
        'manual', 'scheduled', 'api'
    )),
    status          VARCHAR(20) NOT NULL DEFAULT 'running' CHECK (status IN (
        'running', 'completed', 'failed', 'cancelled'
    )),
    total_hits      BIGINT,
    -- A bounded preview of matches for the UI; the full result set stays in
    -- the search backend rather than being copied here.
    sample_results  JSONB NOT NULL DEFAULT '[]'::jsonb,
    duration_ms     BIGINT,
    alert_triggered BOOLEAN NOT NULL DEFAULT FALSE,
    alert_id        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    error_message   TEXT,
    executed_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_hunting_runs_query
    ON hunting_runs(query_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_hunting_runs_tenant
    ON hunting_runs(tenant_id, started_at DESC);


-- ============================================================
-- Backup jobs & runs
-- ============================================================

CREATE TABLE IF NOT EXISTS backup_jobs (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    backup_type     VARCHAR(20) NOT NULL CHECK (backup_type IN (
        'full', 'incremental', 'differential', 'snapshot'
    )),
    target_kind     VARCHAR(20) NOT NULL CHECK (target_kind IN (
        'database', 'filesystem', 'config', 'volume', 'application'
    )),
    source_ref      VARCHAR(512) NOT NULL,
    destination     VARCHAR(30) NOT NULL CHECK (destination IN (
        's3', 'gcs', 'azure_blob', 'local', 'nfs', 'sftp'
    )),
    destination_path VARCHAR(1024),
    server_id       UUID REFERENCES servers(id) ON DELETE SET NULL,
    is_enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    schedule_interval_mins INTEGER,
    retention_days  INTEGER NOT NULL DEFAULT 30,
    -- Encryption and integrity verification are recorded as posture signals;
    -- an unencrypted or unverified backup is a finding worth surfacing.
    encryption_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    encryption_algo VARCHAR(50),
    verify_after_backup BOOLEAN NOT NULL DEFAULT TRUE,
    last_status     VARCHAR(20) CHECK (last_status IN (
        'success', 'failed', 'running', 'partial', 'never_run'
    )),
    last_run_at     TIMESTAMPTZ,
    next_run_at     TIMESTAMPTZ,
    last_size_bytes BIGINT,
    success_count   BIGINT NOT NULL DEFAULT 0,
    failure_count   BIGINT NOT NULL DEFAULT 0,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_backup_jobs_tenant ON backup_jobs(tenant_id, is_enabled);
CREATE INDEX IF NOT EXISTS idx_backup_jobs_due
    ON backup_jobs(next_run_at) WHERE is_enabled = TRUE;

CREATE TABLE IF NOT EXISTS backup_runs (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    job_id          UUID NOT NULL REFERENCES backup_jobs(id) ON DELETE CASCADE,
    trigger         VARCHAR(20) NOT NULL DEFAULT 'scheduled' CHECK (trigger IN (
        'manual', 'scheduled', 'api'
    )),
    status          VARCHAR(20) NOT NULL DEFAULT 'running' CHECK (status IN (
        'running', 'success', 'failed', 'partial', 'cancelled'
    )),
    size_bytes      BIGINT,
    compressed_bytes BIGINT,
    file_count      BIGINT,
    duration_secs   INTEGER,
    artifact_path   VARCHAR(1024),
    checksum        VARCHAR(128),
    verified        BOOLEAN NOT NULL DEFAULT FALSE,
    verified_at     TIMESTAMPTZ,
    restore_tested  BOOLEAN NOT NULL DEFAULT FALSE,
    error_message   TEXT,
    expires_at      TIMESTAMPTZ,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_backup_runs_job ON backup_runs(job_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_backup_runs_tenant ON backup_runs(tenant_id, started_at DESC);


-- ============================================================
-- Documents & evidence management
-- ============================================================
-- Policy documents, audit evidence and incident artefacts. Binary content is
-- kept in object storage; this table holds metadata plus the storage key so
-- the database stays small and the blob store remains the source of truth.

CREATE TABLE IF NOT EXISTS documents (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title           VARCHAR(512) NOT NULL,
    description     TEXT,
    doc_type        VARCHAR(30) NOT NULL CHECK (doc_type IN (
        'policy', 'procedure', 'evidence', 'report', 'certificate',
        'contract', 'dpa', 'runbook', 'other'
    )),
    category        VARCHAR(100),
    status          VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN (
        'draft', 'in_review', 'approved', 'published', 'archived', 'expired'
    )),
    classification  VARCHAR(20) NOT NULL DEFAULT 'internal' CHECK (classification IN (
        'public', 'internal', 'confidential', 'restricted'
    )),
    version         VARCHAR(20) NOT NULL DEFAULT '1.0',
    storage_key     VARCHAR(1024),
    file_name       VARCHAR(512),
    mime_type       VARCHAR(128),
    size_bytes      BIGINT,
    -- Integrity anchor for evidence that may be produced during an audit.
    content_sha256  VARCHAR(64),
    -- Optional links so evidence can be traced to what it supports.
    grc_policy_id   UUID REFERENCES grc_policies(id) ON DELETE SET NULL,
    grc_control_id  UUID REFERENCES grc_controls(id) ON DELETE SET NULL,
    incident_id     UUID REFERENCES incidents(id) ON DELETE SET NULL,
    compliance_standard_id UUID REFERENCES compliance_standards(id) ON DELETE SET NULL,
    tags            JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    owner_id        UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at     TIMESTAMPTZ,
    effective_date  DATE,
    expires_at      TIMESTAMPTZ,
    download_count  BIGINT NOT NULL DEFAULT 0,
    uploaded_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_documents_tenant ON documents(tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(tenant_id, doc_type, status);
CREATE INDEX IF NOT EXISTS idx_documents_expiring
    ON documents(tenant_id, expires_at) WHERE status = 'published';

