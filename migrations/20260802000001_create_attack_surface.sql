-- Raksha Security Platform
-- Attack surface discovered assets (subdomains, services, ports, cloud).
-- Real (non-stub) backing store for the Attack Surface module.

CREATE TABLE IF NOT EXISTS attack_surface_assets (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    UUID REFERENCES tenants(id) ON DELETE CASCADE,
    domain       VARCHAR(512) NOT NULL,
    asset_type   VARCHAR(20)  NOT NULL,   -- subdomain | service | port | cloud
    status       VARCHAR(20)  NOT NULL DEFAULT 'exposed',  -- exposed | internal
    risk         VARCHAR(20)  NOT NULL DEFAULT 'low',      -- critical|high|medium|low
    details      TEXT,
    last_scan_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_attack_surface_tenant ON attack_surface_assets(tenant_id);
CREATE INDEX IF NOT EXISTS idx_attack_surface_type ON attack_surface_assets(asset_type);
