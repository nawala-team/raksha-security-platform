-- Migration: 006_create_mfa
-- Description: Create user_mfa table for multi-factor authentication methods
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enum for MFA methods
CREATE TYPE mfa_method AS ENUM (
    'totp',
    'webauthn',
    'sms',
    'email',
    'recovery_codes'
);

CREATE TABLE user_mfa (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    method           mfa_method NOT NULL,
    secret_encrypted TEXT NOT NULL,
    backup_codes     TEXT[],
    verified         BOOLEAN NOT NULL DEFAULT FALSE,
    verified_at      TIMESTAMPTZ,
    last_used        TIMESTAMPTZ,
    device_name      VARCHAR(255),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_user_mfa_method UNIQUE (user_id, method),
    CONSTRAINT chk_verified_at CHECK (
        (verified = FALSE AND verified_at IS NULL) OR
        (verified = TRUE AND verified_at IS NOT NULL)
    )
);

-- WebAuthn credentials table
CREATE TABLE webauthn_credentials (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id   TEXT NOT NULL UNIQUE,
    public_key      TEXT NOT NULL,
    sign_count      BIGINT NOT NULL DEFAULT 0,
    device_name     VARCHAR(255),
    transports      TEXT[],
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used       TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_user_mfa_user_id ON user_mfa (user_id);
CREATE INDEX idx_user_mfa_method ON user_mfa (method);
CREATE INDEX idx_user_mfa_verified ON user_mfa (user_id) WHERE verified = TRUE;
CREATE INDEX idx_webauthn_user_id ON webauthn_credentials (user_id);
CREATE INDEX idx_webauthn_credential_id ON webauthn_credentials (credential_id);

-- Comments
COMMENT ON TABLE user_mfa IS 'Multi-factor authentication methods registered by users';
COMMENT ON COLUMN user_mfa.secret_encrypted IS 'AES-256-GCM encrypted TOTP secret or method-specific credential';
COMMENT ON COLUMN user_mfa.backup_codes IS 'Array of hashed one-time backup recovery codes';
COMMENT ON TABLE webauthn_credentials IS 'WebAuthn/FIDO2 credentials for hardware security keys';
