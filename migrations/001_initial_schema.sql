-- Raksha Security Platform - Database Migrations
-- Initial schema setup

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Custom types
CREATE TYPE user_role AS ENUM ('super_admin', 'admin', 'analyst', 'operator', 'viewer');
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'suspended', 'pending_verification', 'locked');
CREATE TYPE agent_status AS ENUM ('online', 'offline', 'degraded', 'updating', 'enrolling', 'decommissioned');
CREATE TYPE agent_os AS ENUM ('linux', 'windows', 'macos', 'freebsd');
CREATE TYPE alert_severity AS ENUM ('info', 'low', 'medium', 'high', 'critical');
CREATE TYPE alert_status AS ENUM ('open', 'acknowledged', 'investigating', 'resolved', 'false_positive', 'suppressed');
CREATE TYPE compliance_status AS ENUM ('compliant', 'non_compliant', 'partially_compliant', 'not_assessed', 'not_applicable');
CREATE TYPE audit_action_type AS ENUM ('create', 'read', 'update', 'delete', 'login', 'logout', 'login_failed', 'permission_change', 'config_change', 'export', 'import', 'escalation', 'approval', 'rejection');
CREATE TYPE audit_action_category AS ENUM ('authentication', 'authorization', 'data_access', 'data_modification', 'system_config', 'security_event', 'compliance', 'user_management');
CREATE TYPE audit_risk_level AS ENUM ('low', 'medium', 'high', 'critical');

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'viewer',
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
