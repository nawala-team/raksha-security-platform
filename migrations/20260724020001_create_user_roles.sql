-- Raksha Security Platform
-- Migration: 20260724020001_create_user_roles
-- Description: Named roles and tenant-scoped user/role assignments.
--
-- The portal resolves a non-superadmin user's tenant by joining `tenants`
-- against `user_roles.org_id` (see apps/portal/src/middleware/tenant.rs), and
-- reports per-tenant user counts from the same table
-- (see apps/portal/src/handlers/tenants.rs). Without these tables the portal
-- fails to build and every tenant-scoped request errors at runtime.

CREATE TABLE IF NOT EXISTS roles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_system   BOOLEAN NOT NULL DEFAULT false,
    priority    INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_role_name_format CHECK (name ~* '^[a-z][a-z0-9_]{2,99}$')
);

-- `org_id` is the tenant scope of the assignment. It references `tenants`
-- directly so that tenant resolution stays a single join.
CREATE TABLE IF NOT EXISTS user_roles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    org_id      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    granted_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    granted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ,
    is_active   BOOLEAN NOT NULL DEFAULT true,

    CONSTRAINT uq_user_role_org UNIQUE (user_id, role_id, org_id),
    CONSTRAINT chk_user_roles_expires_after_granted
        CHECK (expires_at IS NULL OR expires_at > granted_at)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles (user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles (role_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_org_id ON user_roles (org_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_granted_by ON user_roles (granted_by);
CREATE INDEX IF NOT EXISTS idx_user_roles_expires_at
    ON user_roles (expires_at) WHERE expires_at IS NOT NULL;

-- Supports the tenant-resolution lookup: filter on user + active, order by grant time.
CREATE INDEX IF NOT EXISTS idx_user_roles_active
    ON user_roles (user_id, granted_at) WHERE is_active = true;

-- Built-in roles mirroring the `users.role` values used by the application.
INSERT INTO roles (name, description, is_system, priority) VALUES
    ('super_admin',     'Full platform access across all tenants', true, 100),
    ('tenant_admin',    'Full access within a single tenant',       true, 90),
    ('security_admin',  'Manage security configuration and agents', true, 80),
    ('analyst',         'Investigate alerts and run hunting queries', true, 60),
    ('operator',        'Operate agents and acknowledge alerts',    true, 40),
    ('viewer',          'Read-only access',                         true, 20),
    ('api_service',     'Machine-to-machine service account',       true, 10)
ON CONFLICT (name) DO NOTHING;

COMMENT ON TABLE roles IS 'Named RBAC roles; system roles are seeded and must not be deleted.';
COMMENT ON TABLE user_roles IS 'Tenant-scoped user-to-role assignments.';
COMMENT ON COLUMN user_roles.org_id IS 'Tenant scope for this assignment; references tenants(id).';
COMMENT ON COLUMN user_roles.is_active IS 'Soft-disable an assignment without deleting it.';
