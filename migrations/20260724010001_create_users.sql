-- Raksha Security Platform
-- Migration: 20260724010001_create_users
-- Description: Users with RBAC roles, MFA support, and session tracking

CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID REFERENCES tenants(id) ON DELETE CASCADE,
    email               VARCHAR(255) NOT NULL,
    username            VARCHAR(100),
    display_name        VARCHAR(255) NOT NULL,
    avatar_url          TEXT,
    password_hash       TEXT NOT NULL,
    password_changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    password_expires_at TIMESTAMPTZ,
    failed_login_count  INTEGER NOT NULL DEFAULT 0,
    locked_until        TIMESTAMPTZ,
    mfa_enabled         BOOLEAN NOT NULL DEFAULT false,
    mfa_method          VARCHAR(20) CHECK (mfa_method IN ('totp', 'webauthn', 'sms', 'email')),
    mfa_secret_enc      TEXT,
    mfa_backup_codes    TEXT[],
    mfa_verified_at     TIMESTAMPTZ,
    role                VARCHAR(30) NOT NULL DEFAULT 'viewer'
                        CHECK (role IN ('super_admin', 'tenant_admin', 'security_admin',
                                       'analyst', 'operator', 'viewer', 'api_service')),
    permissions         JSONB NOT NULL DEFAULT '[]',
    status              VARCHAR(30) NOT NULL DEFAULT 'pending_verification'
                        CHECK (status IN ('active', 'inactive', 'suspended', 'pending_verification',
                                         'locked', 'password_reset_required')),
    timezone            VARCHAR(50) DEFAULT 'UTC',
    locale              VARCHAR(10) DEFAULT 'en',
    notification_prefs  JSONB NOT NULL DEFAULT '{"email": true, "in_app": true, "slack": false, "severity_threshold": "medium"}',
    last_login_at       TIMESTAMPTZ,
    last_login_ip       INET,
    last_active_at      TIMESTAMPTZ,
    login_count         INTEGER NOT NULL DEFAULT 0,
    sso_subject_id      VARCHAR(255),
    sso_provider        VARCHAR(50),
    invited_by          UUID REFERENCES users(id),
    invited_at          TIMESTAMPTZ,
    verified_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deactivated_at      TIMESTAMPTZ,
    CONSTRAINT uq_users_email_tenant UNIQUE (email, tenant_id),
    CONSTRAINT uq_users_username_tenant UNIQUE (username, tenant_id)
);

CREATE INDEX idx_users_tenant ON users(tenant_id) WHERE tenant_id IS NOT NULL;
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_last_login ON users(last_login_at DESC);
CREATE INDEX idx_users_sso ON users(sso_provider, sso_subject_id) WHERE sso_subject_id IS NOT NULL;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE users IS 'Platform users with RBAC roles, MFA configuration, and multi-tenant isolation. Supports local auth and SSO federation.';
COMMENT ON COLUMN users.role IS 'Primary RBAC role. Permissions column provides additional fine-grained access beyond the role.';
COMMENT ON COLUMN users.mfa_secret_enc IS 'AES-256-GCM encrypted TOTP secret. Encryption key managed by application.';
COMMENT ON COLUMN users.permissions IS 'Array of additional permission strings beyond what the role grants.';

-- User sessions
CREATE TABLE user_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    token_hash      TEXT NOT NULL UNIQUE,
    refresh_hash    TEXT UNIQUE,
    ip_address      INET NOT NULL,
    user_agent      TEXT,
    device_id       VARCHAR(255),
    geo_country     VARCHAR(3),
    geo_city        VARCHAR(100),
    risk_score      SMALLINT DEFAULT 0 CHECK (risk_score BETWEEN 0 AND 100),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    mfa_verified    BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL,
    last_activity   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ,
    revoke_reason   VARCHAR(100)
);

CREATE INDEX idx_sessions_user ON user_sessions(user_id) WHERE is_active = true;
CREATE INDEX idx_sessions_token ON user_sessions(token_hash);
CREATE INDEX idx_sessions_expires ON user_sessions(expires_at) WHERE is_active = true;

COMMENT ON TABLE user_sessions IS 'Active user sessions with device tracking, geo-location, and risk scoring for anomaly detection.';

-- Password history
CREATE TABLE password_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    password_hash   TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_password_history_user ON password_history(user_id, created_at DESC);

COMMENT ON TABLE password_history IS 'Historical password hashes to enforce no-reuse policies.';

-- Default admin user (password: changeme - MUST be changed on first login)
INSERT INTO users (id, tenant_id, email, display_name, password_hash, role, status, password_changed_at)
VALUES (
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000001',
    'admin@localhost',
    'System Administrator',
    '$2b$12$LJ3m4sFQDJOlKZMBkEJxPOhVgiNr4.YjVPGxINTBwMDWFRnJqN5Xa',
    'super_admin',
    'password_reset_required',
    '2020-01-01T00:00:00Z'
);
