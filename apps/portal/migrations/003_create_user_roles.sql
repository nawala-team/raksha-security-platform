-- Migration: 003_create_user_roles
-- Description: Create user_roles assignment table with organization scoping
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create organizations table (needed for org_id reference)
CREATE TABLE organizations (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(255) NOT NULL,
    slug        VARCHAR(100) NOT NULL UNIQUE,
    domain      VARCHAR(255),
    settings    JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_org_slug_format CHECK (slug ~* '^[a-z][a-z0-9-]{2,99}$')
);

CREATE TRIGGER set_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Create user_roles table
CREATE TABLE user_roles (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    org_id      UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    granted_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    granted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,

    CONSTRAINT uq_user_role_org UNIQUE (user_id, role_id, org_id),
    CONSTRAINT chk_expires_after_granted CHECK (expires_at IS NULL OR expires_at > granted_at)
);

-- Indexes
CREATE INDEX idx_user_roles_user_id ON user_roles (user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles (role_id);
CREATE INDEX idx_user_roles_org_id ON user_roles (org_id);
CREATE INDEX idx_user_roles_granted_by ON user_roles (granted_by);
CREATE INDEX idx_user_roles_expires_at ON user_roles (expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_user_roles_active ON user_roles (user_id, org_id) WHERE is_active = TRUE;

-- Comments
COMMENT ON TABLE organizations IS 'Multi-tenant organizations';
COMMENT ON TABLE user_roles IS 'User-to-role assignments scoped by organization';
COMMENT ON COLUMN user_roles.org_id IS 'Organization scope for this role assignment';
COMMENT ON COLUMN user_roles.granted_by IS 'User who granted this role';
COMMENT ON COLUMN user_roles.expires_at IS 'Optional expiration for time-limited role grants';
COMMENT ON COLUMN user_roles.is_active IS 'Soft-disable without removing the assignment';
