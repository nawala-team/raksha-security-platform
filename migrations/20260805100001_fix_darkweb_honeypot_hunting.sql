-- Migration: Fix schema mismatches for darkweb, honeypot, and hunting
-- Date: 2026-08-05
-- Description: Adds missing columns and aliases expected by portal handlers

-- ============================================================
-- 1. Darkweb findings - add missing columns for handler compatibility
-- ============================================================
-- Handler expects: source, url, content, found_at
-- Schema has: source_name, source_reference, excerpt_redacted, discovered_at

-- Add the columns that handlers expect
ALTER TABLE darkweb_findings 
    ADD COLUMN IF NOT EXISTS source VARCHAR(255);

ALTER TABLE darkweb_findings 
    ADD COLUMN IF NOT EXISTS url VARCHAR(512);

ALTER TABLE darkweb_findings 
    ADD COLUMN IF NOT EXISTS content TEXT;

ALTER TABLE darkweb_findings 
    ADD COLUMN IF NOT EXISTS found_at TIMESTAMPTZ;

-- Populate from existing columns for existing records
UPDATE darkweb_findings 
SET 
    source = COALESCE(source, source_name),
    url = COALESCE(url, source_reference),
    content = COALESCE(content, excerpt_redacted),
    found_at = COALESCE(found_at, discovered_at, created_at);

-- ============================================================
-- 2. Honeypot interactions - add missing columns
-- ============================================================
-- Handler expects: status
-- Also summary query expects: is_threat

ALTER TABLE honeypot_interactions 
    ADD COLUMN IF NOT EXISTS status VARCHAR(30) DEFAULT 'captured';

ALTER TABLE honeypot_interactions 
    ADD COLUMN IF NOT EXISTS is_threat BOOLEAN DEFAULT FALSE;

-- Set is_threat based on severity and interaction_type
UPDATE honeypot_interactions 
SET is_threat = TRUE 
WHERE severity IN ('high', 'critical') 
   OR interaction_type = 'exploit_attempt';

UPDATE honeypot_interactions 
SET status = CASE 
    WHEN is_threat THEN 'threat_detected'
    WHEN interaction_type = 'exploit_attempt' THEN 'exploit_attempt'
    WHEN interaction_type = 'login_attempt' THEN 'login_captured'
    ELSE 'captured'
END
WHERE status IS NULL OR status = 'captured';

-- ============================================================
-- 3. Hunting runs - ensure results_count exists with proper type
-- ============================================================
-- Make sure results_count exists and is populated
ALTER TABLE hunting_runs 
    ADD COLUMN IF NOT EXISTS results_count INTEGER DEFAULT 0;

-- ============================================================
-- 4. Create indexes for new columns
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_honeypot_interactions_threat 
    ON honeypot_interactions(tenant_id, is_threat) 
    WHERE is_threat = TRUE;

CREATE INDEX IF NOT EXISTS idx_darkweb_findings_found 
    ON darkweb_findings(tenant_id, found_at DESC);

-- ============================================================
-- 5. Refresh summary counters
-- ============================================================
-- Update honeypot unique_attackers count
UPDATE honeypots hp 
SET unique_attackers = (
    SELECT COUNT(DISTINCT source_ip) 
    FROM honeypot_interactions hi 
    WHERE hi.honeypot_id = hp.id
),
interaction_count = (
    SELECT COUNT(*) 
    FROM honeypot_interactions hi 
    WHERE hi.honeypot_id = hp.id
);

-- Update darkweb_monitors finding counts
UPDATE darkweb_monitors dm 
SET finding_count = (
    SELECT COUNT(*) 
    FROM darkweb_findings df 
    WHERE df.monitor_id = dm.id
),
new_finding_count = (
    SELECT COUNT(*) 
    FROM darkweb_findings df 
    WHERE df.monitor_id = dm.id AND df.status = 'new'
);

