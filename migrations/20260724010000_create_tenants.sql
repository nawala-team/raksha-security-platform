-- Raksha Security Platform
-- Migration: 20260724010000_create_tenants
-- Description: Multi-tenant support with tenant isolation and billing tiers
-- Author: Raksha DBA Team

-- Required extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =============================================================================
-- TENANTS TABLE
-- Supports multi-tenant SaaS mode and single-tenant on-prem deployments.
-- Each tenant gets isolated data via tenant_id FK on all domain tables.
-- =============================================================================

CREATE TABLE tenants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(100) NOT NULL UNIQUE,
    domain          VARCHAR(255),
    
    -- Subscription and limits
    plan            VARCHAR(50) NOT NULL DEFAULT 'free'
                    CHECK (plan IN ('free', 'starter', 'professional', 'enterprise', 'on_prem')),
    status          VARCHAR(30) NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'suspended', 'trial', 'cancelled', 'pending_setup')),
    max_agents      INTEGER NOT NULL DEFAULT 10,
    max_users       INTEGER NOT NULL DEFAULT 5,
    retention_days  INTEGER NOT NULL DEFAULT 90,
    
    -- Contact and metadata
    contact_email   VARCHAR(255),
    contact_name    VARCHAR(255),
    phone           VARCHAR(50),
    address         JSONB DEFAULT '{}',
    
    -- Feature flags and settings
    features        JSONB NOT NULL DEFAULT '{
        "fim": true,
        "vulnerability_scan": true,
        "compliance": true,
        "threat_intel": false,
        "incident_response": false,
        "siem_integration": false
    }',
    settings        JSONB NOT NULL DEFAULT '{}',
    
    -- Security
    api_key_hash    TEXT,
    allowed_ips     INET[],
    enforce_mfa     BOOLEAN NOT NULL DEFAULT false,
    sso_provider    VARCHAR(50) CHECK (sso_provider IN ('saml', 'oidc', 'azure_ad', 'okta', NULL)),
    sso_config      JSONB,
    
    -- Trial tracking
    trial_ends_at   TIMESTAMPTZ,
    
    -- Timestamps
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    suspended_at    TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ  -- Soft delete
);

-- Indexes
CREATE INDEX idx_tenants_slug ON tenants(slug);
CREATE INDEX idx_tenants_status ON tenants(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_tenants_plan ON tenants(plan);
CREATE INDEX idx_tenants_domain ON tenants(domain) WHERE domain IS NOT NULL;
CREATE INDEX idx_tenants_created_at ON tenants(created_at DESC);

-- Table comment
COMMENT ON TABLE tenants IS 'Multi-tenant organization registry. Each tenant represents an isolated customer workspace with its own users, agents, and data boundaries.';
COMMENT ON COLUMN tenants.slug IS 'URL-safe unique identifier for the tenant, used in API paths and subdomains';
COMMENT ON COLUMN tenants.retention_days IS 'Data retention policy in days. Logs and metrics older than this are purged.';
COMMENT ON COLUMN tenants.features IS 'Feature flag map controlling which platform modules are enabled for this tenant';
COMMENT ON COLUMN tenants.allowed_ips IS 'IP allowlist for API access. NULL means no restriction.';

-- =============================================================================
-- TENANT API KEYS (separate table for rotation support)
-- =============================================================================

CREATE TABLE tenant_api_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    key_prefix      VARCHAR(12) NOT NULL,  -- First 8 chars for identification
    key_hash        TEXT NOT NULL,
    scopes          TEXT[] NOT NULL DEFAULT ARRAY['read'],
    expires_at      TIMESTAMPTZ,
    last_used_at    TIMESTAMPTZ,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_by      UUID,  -- Will reference users table after it's created
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ
);

CREATE INDEX idx_tenant_api_keys_tenant ON tenant_api_keys(tenant_id) WHERE is_active = true;
CREATE INDEX idx_tenant_api_keys_prefix ON tenant_api_keys(key_prefix);
CREATE UNIQUE INDEX idx_tenant_api_keys_hash ON tenant_api_keys(key_hash);

COMMENT ON TABLE tenant_api_keys IS 'API keys for programmatic tenant access with scope-based permissions and rotation support';

-- =============================================================================
-- UPDATED_AT TRIGGER FUNCTION (reusable across all tables)
-- =============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- DEFAULT TENANT for single-tenant / on-prem deployments
-- =============================================================================

INSERT INTO tenants (id, name, slug, contact_email, plan, status, max_agents, max_users, retention_days)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Default Organization',
    'default',
    'admin@localhost',
    'on_prem',
    'active',
    2147483647,  -- Unlimited agents
    2147483647,  -- Unlimited users
    365          -- 1 year retention
);
