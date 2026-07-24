-- Migration: 001_create_users
-- Description: Create users table for Raksha Security Platform
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "citext";

-- Create enum for user status
CREATE TYPE user_status AS ENUM (
    'active',
    'inactive',
    'suspended',
    'pending_verification',
    'locked'
);

-- Create users table
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email           CITEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    display_name    VARCHAR(255) NOT NULL,
    status          user_status NOT NULL DEFAULT 'pending_verification',
    mfa_enabled     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login      TIMESTAMPTZ,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until    TIMESTAMPTZ,

    CONSTRAINT chk_email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    CONSTRAINT chk_failed_attempts CHECK (failed_attempts >= 0)
);

-- Indexes
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_status ON users (status);
CREATE INDEX idx_users_created_at ON users (created_at);
CREATE INDEX idx_users_last_login ON users (last_login);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Table comment
COMMENT ON TABLE users IS 'Core user accounts for Raksha Security Platform';
COMMENT ON COLUMN users.id IS 'Unique user identifier (UUID v4)';
COMMENT ON COLUMN users.email IS 'User email address (case-insensitive, unique)';
COMMENT ON COLUMN users.password_hash IS 'Argon2id hashed password';
COMMENT ON COLUMN users.status IS 'Current account status';
COMMENT ON COLUMN users.mfa_enabled IS 'Whether multi-factor authentication is enabled';
COMMENT ON COLUMN users.failed_attempts IS 'Consecutive failed login attempts';
COMMENT ON COLUMN users.locked_until IS 'Account locked until this timestamp after too many failed attempts';
