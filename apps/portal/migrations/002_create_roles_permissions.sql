-- Migration: 002_create_roles_permissions
-- Description: Create roles, permissions, and role_permissions junction table
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enum for permission action types
CREATE TYPE permission_action AS ENUM (
    'create',
    'read',
    'update',
    'delete',
    'execute',
    'approve',
    'export',
    'manage'
);

-- Create roles table
CREATE TABLE roles (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_system   BOOLEAN NOT NULL DEFAULT FALSE,
    priority    INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_role_name_format CHECK (name ~* '^[a-z][a-z0-9_]{2,99}$')
);

-- Create permissions table
CREATE TABLE permissions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource    VARCHAR(100) NOT NULL,
    action      permission_action NOT NULL,
    description TEXT,
    conditions  JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_permission_resource_action UNIQUE (resource, action)
);

-- Create role_permissions junction table
CREATE TABLE role_permissions (
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    granted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by    UUID REFERENCES users(id) ON DELETE SET NULL,

    PRIMARY KEY (role_id, permission_id)
);

-- Indexes
CREATE INDEX idx_roles_name ON roles (name);
CREATE INDEX idx_roles_is_system ON roles (is_system);
CREATE INDEX idx_permissions_resource ON permissions (resource);
CREATE INDEX idx_permissions_action ON permissions (action);
CREATE INDEX idx_role_permissions_role_id ON role_permissions (role_id);
CREATE INDEX idx_role_permissions_permission_id ON role_permissions (permission_id);

-- Triggers
CREATE TRIGGER set_roles_updated_at
    BEFORE UPDATE ON roles
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE roles IS 'Security roles that group permissions together';
COMMENT ON TABLE permissions IS 'Granular permissions for resource access control';
COMMENT ON TABLE role_permissions IS 'Many-to-many mapping between roles and permissions';
COMMENT ON COLUMN roles.is_system IS 'System roles cannot be deleted or modified by users';
COMMENT ON COLUMN roles.priority IS 'Higher priority roles take precedence in conflict resolution';
COMMENT ON COLUMN permissions.conditions IS 'JSONB conditions for conditional permission grants (e.g., time-based, IP-based)';
