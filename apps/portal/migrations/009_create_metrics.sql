-- Migration: 009_create_metrics
-- Description: Create agent_metrics table with TimescaleDB hypertable support
-- Created: 2024-01-01
-- Database: PostgreSQL 15+ with TimescaleDB extension

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create enum for metric types
CREATE TYPE metric_type AS ENUM (
    'gauge',
    'counter',
    'histogram',
    'summary'
);

-- Create agent_metrics table (designed for time-series data)
CREATE TABLE agent_metrics (
    timestamp    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    agent_id     UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    metric_type  metric_type NOT NULL,
    metric_name  VARCHAR(255) NOT NULL,
    value        DOUBLE PRECISION NOT NULL,
    labels       JSONB DEFAULT '{}',

    CONSTRAINT chk_metric_name_format CHECK (metric_name ~ '^[a-z][a-z0-9_.]{1,254}$')
);

-- Convert to TimescaleDB hypertable (chunks by 1 day)
SELECT create_hypertable('agent_metrics', 'timestamp',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- Indexes optimized for time-series queries
CREATE INDEX idx_metrics_agent_time ON agent_metrics (agent_id, timestamp DESC);
CREATE INDEX idx_metrics_name_time ON agent_metrics (metric_name, timestamp DESC);
CREATE INDEX idx_metrics_agent_name_time ON agent_metrics (agent_id, metric_name, timestamp DESC);
CREATE INDEX idx_metrics_labels ON agent_metrics USING GIN (labels);

-- Continuous aggregates for common queries
-- Hourly rollup
CREATE MATERIALIZED VIEW agent_metrics_hourly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', timestamp) AS bucket,
    agent_id,
    metric_name,
    metric_type,
    AVG(value) AS avg_value,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    COUNT(*) AS sample_count
FROM agent_metrics
GROUP BY bucket, agent_id, metric_name, metric_type
WITH NO DATA;

-- Daily rollup
CREATE MATERIALIZED VIEW agent_metrics_daily
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', timestamp) AS bucket,
    agent_id,
    metric_name,
    metric_type,
    AVG(value) AS avg_value,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    COUNT(*) AS sample_count
FROM agent_metrics
GROUP BY bucket, agent_id, metric_name, metric_type
WITH NO DATA;

-- Refresh policies for continuous aggregates
SELECT add_continuous_aggregate_policy('agent_metrics_hourly',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour');

SELECT add_continuous_aggregate_policy('agent_metrics_daily',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day');

-- Retention policy: drop raw data older than 30 days
SELECT add_retention_policy('agent_metrics', INTERVAL '30 days');

-- Compression policy: compress chunks older than 7 days
ALTER TABLE agent_metrics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'agent_id,metric_name',
    timescaledb.compress_orderby = 'timestamp DESC'
);

SELECT add_compression_policy('agent_metrics', INTERVAL '7 days');

-- Comments
COMMENT ON TABLE agent_metrics IS 'Time-series metrics from security agents (TimescaleDB hypertable)';
COMMENT ON COLUMN agent_metrics.metric_type IS 'Prometheus-style metric type classification';
COMMENT ON COLUMN agent_metrics.metric_name IS 'Dotted metric name (e.g., cpu.usage_percent, network.bytes_in)';
COMMENT ON COLUMN agent_metrics.labels IS 'Metric labels/dimensions as JSONB for flexible querying';
