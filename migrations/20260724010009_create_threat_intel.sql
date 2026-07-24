-- Raksha Security Platform
-- Migration: 20260724010009_create_threat_intel
-- Description: IOC storage, threat feeds, and indicator matching

-- Threat intelligence feeds
CREATE TABLE threat_intel_feeds (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    provider        VARCHAR(255) NOT NULL,
    feed_type       VARCHAR(30) NOT NULL
                    CHECK (feed_type IN ('stix_taxii', 'csv', 'json', 'misp', 'otx',
                                        'abuse_ch', 'custom_api', 'manual')),
    url             TEXT,
    auth_type       VARCHAR(20)
                    CHECK (auth_type IN ('none', 'api_key', 'basic', 'bearer', 'certificate')),
    auth_config_enc TEXT,
    is_enabled      BOOLEAN NOT NULL DEFAULT true,
    polling_interval_mins INTEGER DEFAULT 60,
    last_poll_at    TIMESTAMPTZ,
    last_poll_status VARCHAR(20)
                    CHECK (last_poll_status IN ('success', 'failed', 'partial', 'timeout')),
    last_poll_error TEXT,
    indicators_total BIGINT NOT NULL DEFAULT 0,
    indicators_active BIGINT NOT NULL DEFAULT 0,
    confidence_override SMALLINT CHECK (confidence_override BETWEEN 0 AND 100),
    tags            TEXT[] DEFAULT '{}',
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_threat_feed_name_tenant UNIQUE (name, tenant_id)
);

CREATE INDEX idx_threat_feeds_tenant ON threat_intel_feeds(tenant_id);
CREATE INDEX idx_threat_feeds_enabled ON threat_intel_feeds(is_enabled) WHERE is_enabled = true;
CREATE INDEX idx_threat_feeds_poll ON threat_intel_feeds(last_poll_at) WHERE is_enabled = true;

CREATE TRIGGER trg_threat_feeds_updated_at
    BEFORE UPDATE ON threat_intel_feeds
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE threat_intel_feeds IS 'Threat intelligence feed sources with polling configuration and health tracking.';

-- Indicators of Compromise (IOCs)
CREATE TABLE threat_indicators (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    feed_id         UUID REFERENCES threat_intel_feeds(id) ON DELETE SET NULL,
    indicator_type  VARCHAR(30) NOT NULL
                    CHECK (indicator_type IN ('ip_v4', 'ip_v6', 'domain', 'url', 'email',
                                            'file_hash_md5', 'file_hash_sha1', 'file_hash_sha256',
                                            'file_name', 'mutex', 'registry_key', 'user_agent',
                                            'ja3_hash', 'certificate_hash', 'cidr', 'asn')),
    value           TEXT NOT NULL,
    value_normalized TEXT NOT NULL,
    threat_type     VARCHAR(50)
                    CHECK (threat_type IN ('malware', 'c2', 'phishing', 'exploit', 'botnet',
                                          'ransomware', 'apt', 'spam', 'scanner', 'tor_exit',
                                          'proxy', 'miner', 'dropper', 'unknown')),
    confidence      SMALLINT NOT NULL DEFAULT 50 CHECK (confidence BETWEEN 0 AND 100),
    severity        VARCHAR(10) NOT NULL DEFAULT 'medium'
                    CHECK (severity IN ('info', 'low', 'medium', 'high', 'critical')),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    is_whitelisted  BOOLEAN NOT NULL DEFAULT false,
    whitelist_reason TEXT,
    first_seen_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ,
    kill_chain_phase VARCHAR(50),
    malware_families TEXT[],
    threat_actors   TEXT[],
    campaigns       TEXT[],
    tags            TEXT[] DEFAULT '{}',
    context         JSONB DEFAULT '{}',
    external_refs   JSONB DEFAULT '[]',
    stix_id         VARCHAR(255),
    hit_count       BIGINT NOT NULL DEFAULT 0,
    last_hit_at     TIMESTAMPTZ,
    source_ref      VARCHAR(500),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_threat_indicators_tenant ON threat_indicators(tenant_id);
CREATE INDEX idx_threat_indicators_type_value ON threat_indicators(indicator_type, value_normalized);
CREATE INDEX idx_threat_indicators_value ON threat_indicators(value_normalized);
CREATE INDEX idx_threat_indicators_feed ON threat_indicators(feed_id);
CREATE INDEX idx_threat_indicators_active ON threat_indicators(is_active) WHERE is_active = true AND is_whitelisted = false;
CREATE INDEX idx_threat_indicators_threat_type ON threat_indicators(threat_type) WHERE threat_type IS NOT NULL;
CREATE INDEX idx_threat_indicators_expires ON threat_indicators(expires_at) WHERE expires_at IS NOT NULL AND is_active = true;
CREATE INDEX idx_threat_indicators_confidence ON threat_indicators(confidence DESC) WHERE is_active = true;
CREATE INDEX idx_threat_indicators_hash ON threat_indicators(value_normalized) WHERE indicator_type IN ('file_hash_md5', 'file_hash_sha1', 'file_hash_sha256');
CREATE INDEX idx_threat_indicators_network ON threat_indicators(value_normalized) WHERE indicator_type IN ('ip_v4', 'ip_v6', 'domain', 'url', 'cidr');
CREATE INDEX idx_threat_indicators_stix ON threat_indicators(stix_id) WHERE stix_id IS NOT NULL;

CREATE TRIGGER trg_threat_indicators_updated_at
    BEFORE UPDATE ON threat_indicators
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE threat_indicators IS 'Indicators of Compromise (IOCs) from threat intelligence feeds with confidence scoring, expiry, and match tracking.';
COMMENT ON COLUMN threat_indicators.value_normalized IS 'Normalized indicator value (lowercased, defanged URLs resolved, IPs without port). Used for matching.';
COMMENT ON COLUMN threat_indicators.hit_count IS 'Number of times this indicator matched observed data in the environment.';
COMMENT ON COLUMN threat_indicators.confidence IS '0-100 confidence score. Feed confidence can be overridden at feed level.';

-- Threat indicator matches (when an IOC is observed in the environment)
CREATE TABLE threat_indicator_matches (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    indicator_id    UUID NOT NULL REFERENCES threat_indicators(id) ON DELETE CASCADE,
    agent_id        UUID REFERENCES agents(id) ON DELETE SET NULL,
    alert_id        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    match_source    VARCHAR(50) NOT NULL
                    CHECK (match_source IN ('network_traffic', 'dns_query', 'file_scan',
                                           'process_exec', 'log_entry', 'email_header', 'url_access')),
    matched_value   TEXT NOT NULL,
    context         JSONB DEFAULT '{}',
    is_confirmed    BOOLEAN,
    confirmed_by    UUID REFERENCES users(id),
    matched_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_threat_matches_tenant ON threat_indicator_matches(tenant_id);
CREATE INDEX idx_threat_matches_indicator ON threat_indicator_matches(indicator_id);
CREATE INDEX idx_threat_matches_agent ON threat_indicator_matches(agent_id);
CREATE INDEX idx_threat_matches_matched ON threat_indicator_matches(matched_at DESC);
CREATE INDEX idx_threat_matches_source ON threat_indicator_matches(match_source);

COMMENT ON TABLE threat_indicator_matches IS 'Records of IOC matches observed in the monitored environment, linking indicators to agents and alerts.';
