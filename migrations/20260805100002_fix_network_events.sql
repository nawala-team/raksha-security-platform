-- Migration: Fix network_events and add missing columns for frontend compatibility
-- Date: 2026-08-05
-- Description: Adds missing columns to network_events table for handler compatibility

-- ============================================================
-- 1. Network events - add missing columns for frontend
-- ============================================================
-- Handler expects: event_type, severity, direction, action, is_threat, occurred_at, process_name, country_code

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS event_type VARCHAR(50) DEFAULT 'traffic';

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS severity VARCHAR(20) DEFAULT 'low';

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS direction VARCHAR(20);

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS action VARCHAR(20);

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS is_threat BOOLEAN DEFAULT FALSE;

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS occurred_at TIMESTAMPTZ;

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS process_name VARCHAR(255);

ALTER TABLE network_events 
    ADD COLUMN IF NOT EXISTS country_code VARCHAR(3);

-- Populate occurred_at from existing timestamp column
UPDATE network_events 
SET occurred_at = COALESCE(timestamp, created_at, NOW())
WHERE occurred_at IS NULL;

-- Set default severity based on existing data patterns
UPDATE network_events 
SET severity = CASE
    WHEN action IN ('block', 'drop', 'reject') THEN 'medium'
    WHEN is_threat = true THEN 'high'
    ELSE 'low'
END
WHERE severity IS NULL OR severity = 'low';

-- Set direction defaults
UPDATE network_events 
SET direction = COALESCE(direction, 'inbound')
WHERE direction IS NULL;

-- ============================================================
-- 2. Create indexes for new columns
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_network_events_occurred 
    ON network_events(occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_network_events_threat 
    ON network_events(is_threat) 
    WHERE is_threat = TRUE;

CREATE INDEX IF NOT EXISTS idx_network_events_action 
    ON network_events(action);

-- ============================================================
-- 3. Add missing columns to network_rules if needed
-- ============================================================
ALTER TABLE network_rules 
    ADD COLUMN IF NOT EXISTS port_range VARCHAR(50);

-- Done
