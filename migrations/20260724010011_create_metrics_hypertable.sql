-- Raksha Security Platform
-- Migration: 20260724010011_create_metrics_hypertable
-- Description: TimescaleDB hypertable for agent metrics time-series data

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- =============================================================================
-- AGENT METRICS HYPERTABLE
-- High-volume time-series metrics from agents (CPU, memory, disk, network, etc.)
-- Uses TimescaleDB for automatic partitioning, compression, and retention.
-- =============================================================================

CREATE TABLE agent_metrics (
    time            TIMESTAMPTZ NOT NULL,
    tenant_id       UUID,
    agent_id        UUID NOT NULL,
    metric_name     VARCHAR(255) NOT NULL,
    value           DOUBLE PRECISION NOT NULL,
    unit            VARCHAR(30),
    labels          JSONB NOT NULL DEFAULT '{}',
    host            VARCHAR(255),
    source_module   VARCHAR(100)
);

-- Convert to TimescaleDB hypertable (partition by time, 1-day chunks)
SELECT create_hypertable('agent_metrics', 'time',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Indexes on hypertable
CREATE INDEX idx_metrics_agent_time ON agent_metrics(agent_id, time DESC);
CREATE INDEX idx_metrics_name_time ON agent_metrics(metric_name, time DESC);
CREATE INDEX idx_metrics_tenant_time ON agent_metrics(tenant_id, time DESC) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_metrics_labels ON agent_metrics USING GIN (labels);

COMMENT ON TABLE agent_metrics IS 'TimescaleDB hypertable for agent time-series metrics. Auto-partitioned by time with compression and retention policies.';
COMMENT ON COLUMN agent_metrics.metric_name IS 'Metric identifier: cpu_usage_percent, memory_used_bytes, disk_io_read_bytes, network_rx_bytes, etc.';
COMMENT ON COLUMN agent_metrics.labels IS 'Dimensional labels for filtering: {"cpu": "0", "interface": "eth0", "mountpoint": "/"}.';

-- =============================================================================
-- COMPRESSION POLICY
-- Compress chunks older than 7 days for storage efficiency.
-- Typical 10-20x compression ratio for metrics data.
-- =============================================================================

ALTER TABLE agent_metrics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'agent_id, metric_name',
    timescaledb.compress_orderby = 'time DESC'
);

SELECT add_compression_policy('agent_metrics', INTERVAL '7 days', if_not_exists => TRUE);

-- =============================================================================
-- RETENTION POLICY
-- Drop chunks older than retention period (default 90 days, configurable per tenant).
-- =============================================================================

SELECT add_retention_policy('agent_metrics', INTERVAL '90 days', if_not_exists => TRUE);

-- =============================================================================
-- CONTINUOUS AGGREGATES
-- Pre-computed rollups for dashboard queries and trend analysis.
-- =============================================================================

-- Hourly rollup
CREATE MATERIALIZED VIEW agent_metrics_hourly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', time) AS bucket,
    agent_id,
    metric_name,
    AVG(value) AS avg_value,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    COUNT(*) AS sample_count,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY value) AS p95_value,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY value) AS p99_value
FROM agent_metrics
GROUP BY bucket, agent_id, metric_name
WITH NO DATA;

-- Refresh policy for hourly aggregate (refresh every hour, cover last 3 hours)
SELECT add_continuous_aggregate_policy('agent_metrics_hourly',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE
);

-- Daily rollup
CREATE MATERIALIZED VIEW agent_metrics_daily
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', time) AS bucket,
    agent_id,
    metric_name,
    AVG(value) AS avg_value,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    COUNT(*) AS sample_count,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY value) AS p95_value
FROM agent_metrics
GROUP BY bucket, agent_id, metric_name
WITH NO DATA;

-- Refresh policy for daily aggregate
SELECT add_continuous_aggregate_policy('agent_metrics_daily',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Retention on aggregates (keep hourly for 30 days, daily for 1 year)
SELECT add_retention_policy('agent_metrics_hourly', INTERVAL '30 days', if_not_exists => TRUE);
SELECT add_retention_policy('agent_metrics_daily', INTERVAL '365 days', if_not_exists => TRUE);

COMMENT ON MATERIALIZED VIEW agent_metrics_hourly IS 'Pre-computed hourly metric rollups with avg/min/max/p95/p99. Auto-refreshed by TimescaleDB.';
COMMENT ON MATERIALIZED VIEW agent_metrics_daily IS 'Pre-computed daily metric rollups for long-term trend analysis.';

-- =============================================================================
-- SYSTEM HEALTH METRICS TABLE
-- Platform-level metrics (API latency, queue depth, etc.) separate from agent metrics.
-- =============================================================================

CREATE TABLE system_metrics (
    time            TIMESTAMPTZ NOT NULL,
    service         VARCHAR(100) NOT NULL,
    metric_name     VARCHAR(255) NOT NULL,
    value           DOUBLE PRECISION NOT NULL,
    unit            VARCHAR(30),
    labels          JSONB NOT NULL DEFAULT '{}',
    instance_id     VARCHAR(255)
);

SELECT create_hypertable('system_metrics', 'time',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

CREATE INDEX idx_system_metrics_service ON system_metrics(service, time DESC);
CREATE INDEX idx_system_metrics_name ON system_metrics(metric_name, time DESC);

ALTER TABLE system_metrics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'service, metric_name',
    timescaledb.compress_orderby = 'time DESC'
);

SELECT add_compression_policy('system_metrics', INTERVAL '3 days', if_not_exists => TRUE);
SELECT add_retention_policy('system_metrics', INTERVAL '30 days', if_not_exists => TRUE);

COMMENT ON TABLE system_metrics IS 'Platform infrastructure metrics (API latency, queue depth, error rates). Separate from agent telemetry.';
