-- Migration: Fix schema mismatches between Rust code and database
-- Date: 2026-08-05
-- Description: Adds missing columns and types expected by the portal code

-- 1. Create tenant_status enum type
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'tenant_status') THEN
        CREATE TYPE tenant_status AS ENUM ('active', 'suspended', 'deleted');
    END IF;
END$$;

-- 2. Add status column with enum type to tenants (keep old varchar for compatibility)
-- First add new column, migrate data, then we can use the enum
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS status_enum tenant_status;

-- Migrate existing status values to enum
UPDATE tenants SET status_enum = 
    CASE status
        WHEN 'active' THEN 'active'::tenant_status
        WHEN 'suspended' THEN 'suspended'::tenant_status
        WHEN 'cancelled' THEN 'suspended'::tenant_status
        WHEN 'trial' THEN 'active'::tenant_status
        WHEN 'pending_setup' THEN 'active'::tenant_status
        ELSE 'active'::tenant_status
    END
WHERE status_enum IS NULL;

-- 3. Add missing columns to hunting_runs
ALTER TABLE hunting_runs ADD COLUMN IF NOT EXISTS results_count INTEGER;
UPDATE hunting_runs SET results_count = COALESCE(total_hits::integer, 0) WHERE results_count IS NULL;

-- 4. Add missing columns to documents  
ALTER TABLE documents ADD COLUMN IF NOT EXISTS file_path VARCHAR(1024);
ALTER TABLE documents ADD COLUMN IF NOT EXISTS file_size BIGINT;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS checksum VARCHAR(64);
ALTER TABLE documents ADD COLUMN IF NOT EXISTS retention_until DATE;

-- Migrate existing data
UPDATE documents SET 
    file_path = COALESCE(storage_key, file_name),
    file_size = size_bytes,
    checksum = content_sha256,
    retention_until = expires_at::date
WHERE file_path IS NULL;

-- 5. Add missing timestamp column to alerts (if code expects it)
ALTER TABLE alerts ADD COLUMN IF NOT EXISTS timestamp TIMESTAMPTZ;
UPDATE alerts SET timestamp = created_at WHERE timestamp IS NULL;

-- 6. Add source column to fim_events if missing
ALTER TABLE fim_events ADD COLUMN IF NOT EXISTS source VARCHAR(255);
UPDATE fim_events SET source = 'agent' WHERE source IS NULL;

-- 7. Create _sqlx_migrations table for SQLx tracking
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
);

-- 8. Create view for tenant with proper enum status (workaround)
CREATE OR REPLACE VIEW tenants_v AS
SELECT 
    id, name, slug, domain, plan,
    COALESCE(status_enum, 'active'::tenant_status) as status,
    max_agents, max_users, retention_days,
    contact_email, contact_name, phone, address,
    features, settings, api_key_hash, allowed_ips,
    enforce_mfa, sso_provider, sso_config,
    trial_ends_at, created_at, updated_at, suspended_at, deleted_at
FROM tenants;

-- Done
