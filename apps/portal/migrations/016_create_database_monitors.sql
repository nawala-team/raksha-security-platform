-- Migration: 016_create_database_monitors
-- Description: Create monitored_databases table for multi-database monitoring
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enum for supported database types
CREATE TYPE monitored_db_type AS ENUM (
    'postgresql',
    'mysql',
    'mongodb',
    'sqlserver',
    'oracle',
    'redis'
);

CREATE TYPE monitor_status AS ENUM (
    'active',
    'inactive',
    'error',
    'maintenance'
);

CREATE TABLE monitored_databases (
    id                   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name                 VARCHAR(255) NOT NULL,
    db_type              monitored_db_type NOT NULL,
    host                 VARCHAR(500) NOT NULL,
    port                 INTEGER NOT NULL,
    database_name        VARCHAR(255),
    credentials_encrypted TEXT NOT NULL,
    ssl_enabled          BOOLEAN NOT NULL DEFAULT TRUE,
    ssl_ca_cert          TEXT,
    monitoring_config    JSONB NOT NULL DEFAULT '{}',
    status               monitor_status NOT NULL DEFAULT 'inactive',
    last_check           TIMESTAMPTZ,
    last_check_result    JSONB,
    health_score         NUMERIC(5,2),
    alert_thresholds     JSONB DEFAULT '{}',
    org_id               UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by           UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_port_range CHECK (port > 0 AND port <= 65535),
    CONSTRAINT chk_health_score_range CHECK (health_score IS NULL OR (health_score >= 0 AND health_score <= 100)),
    CONSTRAINT uq_monitor_name_org UNIQUE (name, org_id)
);

-- Database metrics history
CREATE TABLE database_metrics (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    database_id     UUID NOT NULL REFERENCES monitored_databases(id) ON DELETE CASCADE,
    connections     INTEGER,
    active_queries  INTEGER,
    slow_queries    INTEGER,
    replication_lag_ms BIGINT,
    disk_usage_bytes BIGINT,
    memory_usage_bytes BIGINT,
    cache_hit_ratio NUMERIC(5,4),
    uptime_seconds  BIGINT,
    custom_metrics  JSONB DEFAULT '{}',
    collected_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Database query log (for auditing)
CREATE TABLE database_query_log (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    database_id     UUID NOT NULL REFERENCES monitored_databases(id) ON DELETE CASCADE,
    query_hash      TEXT NOT NULL,
    query_normalized TEXT,
    duration_ms     INTEGER NOT NULL,
    rows_affected   BIGINT,
    user_name       VARCHAR(255),
    is_anomalous    BOOLEAN DEFAULT FALSE,
    executed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_monitored_db_org ON monitored_databases (org_id);
CREATE INDEX idx_monitored_db_type ON monitored_databases (db_type);
CREATE INDEX idx_monitored_db_status ON monitored_databases (status);

CREATE INDEX idx_db_metrics_database ON database_metrics (database_id, collected_at DESC);
CREATE INDEX idx_db_metrics_collected ON database_metrics (collected_at DESC);

CREATE INDEX idx_db_query_log_database ON database_query_log (database_id, executed_at DESC);
CREATE INDEX idx_db_query_log_duration ON database_query_log (duration_ms DESC);
CREATE INDEX idx_db_query_log_anomalous ON database_query_log (is_anomalous) WHERE is_anomalous = TRUE;

-- Triggers
CREATE TRIGGER set_monitored_databases_updated_at
    BEFORE UPDATE ON monitored_databases
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE monitored_databases IS 'Registry of databases monitored by Raksha platform';
COMMENT ON TABLE database_metrics IS 'Time-series metrics collected from monitored databases';
COMMENT ON TABLE database_query_log IS 'Audit log of database queries with anomaly detection';
COMMENT ON COLUMN monitored_databases.credentials_encrypted IS 'AES-256-GCM encrypted connection credentials';
COMMENT ON COLUMN monitored_databases.monitoring_config IS 'Per-database monitoring configuration (intervals, metrics to collect)';
COMMENT ON COLUMN monitored_databases.db_type IS 'Supported: postgresql, mysql, mongodb, sqlserver, oracle, redis';
