-- Raksha Security Platform
-- Migration: Add Oracle, MariaDB, SQL Server support to Database Guard
-- Date: 2026-08-05

-- Update the db_type comment to reflect supported databases
COMMENT ON COLUMN monitored_databases.db_type IS 
'Database type: postgresql | mysql | mongodb | redis | oracle | mariadb | sqlserver';

-- Add Oracle-specific columns (optional, for enhanced monitoring)
ALTER TABLE monitored_databases 
ADD COLUMN IF NOT EXISTS service_name VARCHAR(255),
ADD COLUMN IF NOT EXISTS sid VARCHAR(255),
ADD COLUMN IF NOT EXISTS tns_alias VARCHAR(255);

-- Comments for Oracle-specific columns
COMMENT ON COLUMN monitored_databases.service_name IS 'Oracle service name for connection';
COMMENT ON COLUMN monitored_databases.sid IS 'Oracle SID (System Identifier)';
COMMENT ON COLUMN monitored_databases.tns_alias IS 'Oracle TNS alias from tnsnames.ora';

-- Create index for faster lookups by db_type
CREATE INDEX IF NOT EXISTS idx_monitored_databases_db_type ON monitored_databases(db_type);
