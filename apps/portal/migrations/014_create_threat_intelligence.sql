-- Migration: 014_create_threat_intelligence
-- Description: Create threat intelligence tables
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for threat intelligence
CREATE TYPE indicator_type AS ENUM (
    'ip_address',
    'domain',
    'url',
    'email',
    'file_hash_md5',
    'file_hash_sha1',
    'file_hash_sha256',
    'mutex',
    'registry_key',
    'user_agent',
    'cidr',
    'cve',
    'yara_rule'
);

CREATE TYPE threat_severity AS ENUM (
    'unknown',
    'low',
    'medium',
    'high',
    'critical'
);

CREATE TYPE indicator_status AS ENUM (
    'active',
    'expired',
    'revoked',
    'under_review'
);

CREATE TABLE threat_indicators (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    type        indicator_type NOT NULL,
    value       TEXT NOT NULL,
    source      VARCHAR(255) NOT NULL,
    source_ref  TEXT,
    severity    threat_severity NOT NULL DEFAULT 'unknown',
    confidence  INTEGER NOT NULL DEFAULT 50,
    status      indicator_status NOT NULL DEFAULT 'active',
    first_seen  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ,
    tags        JSONB DEFAULT '[]',
    metadata    JSONB DEFAULT '{}',
    context     TEXT,
    org_id      UUID REFERENCES organizations(id) ON DELETE CASCADE,
    created_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_confidence_range CHECK (confidence >= 0 AND confidence <= 100),
    CONSTRAINT uq_indicator_type_value UNIQUE (type, value)
);

-- Threat feeds (sources of indicators)
CREATE TABLE threat_feeds (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL UNIQUE,
    url         TEXT,
    feed_type   VARCHAR(100) NOT NULL,
    format      VARCHAR(50) NOT NULL DEFAULT 'stix',
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    schedule_cron VARCHAR(100),
    last_fetch  TIMESTAMPTZ,
    last_count  INTEGER DEFAULT 0,
    auth_config JSONB DEFAULT '{}',
    org_id      UUID REFERENCES organizations(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indicator sightings (when indicators are observed in the environment)
CREATE TABLE indicator_sightings (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    indicator_id    UUID NOT NULL REFERENCES threat_indicators(id) ON DELETE CASCADE,
    agent_id        UUID REFERENCES agents(id) ON DELETE SET NULL,
    source          VARCHAR(255) NOT NULL,
    context         JSONB DEFAULT '{}',
    observed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_threat_indicators_type ON threat_indicators (type);
CREATE INDEX idx_threat_indicators_value ON threat_indicators (value);
CREATE INDEX idx_threat_indicators_source ON threat_indicators (source);
CREATE INDEX idx_threat_indicators_severity ON threat_indicators (severity);
CREATE INDEX idx_threat_indicators_status ON threat_indicators (status);
CREATE INDEX idx_threat_indicators_first_seen ON threat_indicators (first_seen);
CREATE INDEX idx_threat_indicators_last_seen ON threat_indicators (last_seen);
CREATE INDEX idx_threat_indicators_org ON threat_indicators (org_id);
CREATE INDEX idx_threat_indicators_tags ON threat_indicators USING GIN (tags);
CREATE INDEX idx_threat_indicators_metadata ON threat_indicators USING GIN (metadata);
CREATE INDEX idx_threat_indicators_active ON threat_indicators (type, value) WHERE status = 'active';

CREATE INDEX idx_threat_feeds_enabled ON threat_feeds (enabled) WHERE enabled = TRUE;
CREATE INDEX idx_indicator_sightings_indicator ON indicator_sightings (indicator_id, observed_at DESC);
CREATE INDEX idx_indicator_sightings_agent ON indicator_sightings (agent_id);

-- Triggers
CREATE TRIGGER set_threat_indicators_updated_at
    BEFORE UPDATE ON threat_indicators
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_threat_feeds_updated_at
    BEFORE UPDATE ON threat_feeds
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE threat_indicators IS 'Threat intelligence indicators of compromise (IOCs)';
COMMENT ON TABLE threat_feeds IS 'External threat intelligence feed sources';
COMMENT ON TABLE indicator_sightings IS 'Observations of threat indicators in the environment';
COMMENT ON COLUMN threat_indicators.confidence IS 'Confidence score 0-100 in indicator accuracy';
COMMENT ON COLUMN threat_indicators.type IS 'Type of indicator (IP, domain, hash, etc.)';
