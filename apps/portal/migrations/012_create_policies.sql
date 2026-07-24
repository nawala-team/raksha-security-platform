-- Migration: 012_create_policies
-- Description: Create policies table for security policy management
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for policies
CREATE TYPE policy_status AS ENUM (
    'draft',
    'pending_review',
    'approved',
    'published',
    'deprecated',
    'archived'
);

CREATE TYPE policy_category AS ENUM (
    'access_control',
    'data_protection',
    'incident_response',
    'network_security',
    'physical_security',
    'business_continuity',
    'risk_management',
    'compliance',
    'acceptable_use',
    'change_management'
);

CREATE TYPE review_cycle AS ENUM (
    'monthly',
    'quarterly',
    'semi_annual',
    'annual',
    'biennial'
);

CREATE TABLE policies (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title            VARCHAR(500) NOT NULL,
    slug             VARCHAR(255) NOT NULL,
    content          TEXT NOT NULL,
    summary          TEXT,
    category         policy_category NOT NULL,
    status           policy_status NOT NULL DEFAULT 'draft',
    standard_mapping JSONB DEFAULT '[]',
    version          INTEGER NOT NULL DEFAULT 1,
    previous_version UUID REFERENCES policies(id) ON DELETE SET NULL,
    org_id           UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by       UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    approved_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at      TIMESTAMPTZ,
    published_at     TIMESTAMPTZ,
    review_cycle     review_cycle NOT NULL DEFAULT 'annual',
    next_review      TIMESTAMPTZ,
    last_reviewed    TIMESTAMPTZ,
    tags             JSONB DEFAULT '[]',
    effective_date   DATE,
    expiry_date      DATE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_policy_slug_org UNIQUE (slug, org_id),
    CONSTRAINT chk_version_positive CHECK (version > 0),
    CONSTRAINT chk_approved_consistency CHECK (
        (approved_by IS NULL AND approved_at IS NULL) OR
        (approved_by IS NOT NULL AND approved_at IS NOT NULL)
    ),
    CONSTRAINT chk_effective_before_expiry CHECK (
        expiry_date IS NULL OR effective_date IS NULL OR effective_date < expiry_date
    )
);

-- Policy acknowledgments (users who have read and accepted)
CREATE TABLE policy_acknowledgments (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_id   UUID NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    acknowledged_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address  INET,
    version_acknowledged INTEGER NOT NULL,

    CONSTRAINT uq_policy_user_version UNIQUE (policy_id, user_id, version_acknowledged)
);

-- Indexes
CREATE INDEX idx_policies_org_id ON policies (org_id);
CREATE INDEX idx_policies_category ON policies (category);
CREATE INDEX idx_policies_status ON policies (status);
CREATE INDEX idx_policies_created_by ON policies (created_by);
CREATE INDEX idx_policies_slug ON policies (slug);
CREATE INDEX idx_policies_review_due ON policies (next_review) WHERE status = 'published';
CREATE INDEX idx_policies_standard_mapping ON policies USING GIN (standard_mapping);
CREATE INDEX idx_policies_tags ON policies USING GIN (tags);
CREATE INDEX idx_policies_version_chain ON policies (previous_version);

CREATE INDEX idx_policy_acks_policy ON policy_acknowledgments (policy_id);
CREATE INDEX idx_policy_acks_user ON policy_acknowledgments (user_id);

-- Triggers
CREATE TRIGGER set_policies_updated_at
    BEFORE UPDATE ON policies
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE policies IS 'Security and compliance policies with versioning and approval workflow';
COMMENT ON COLUMN policies.standard_mapping IS 'JSON array mapping to compliance standard controls (e.g., [{"standard": "SOC2", "controls": ["CC6.1"]}])';
COMMENT ON COLUMN policies.review_cycle IS 'How often this policy must be reviewed';
COMMENT ON COLUMN policies.previous_version IS 'Link to the previous version of this policy for version chain';
COMMENT ON TABLE policy_acknowledgments IS 'Tracks which users have acknowledged each policy version';
