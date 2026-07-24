-- Compliance standards
CREATE TABLE compliance_standards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    authority VARCHAR(255),
    url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Compliance controls
CREATE TABLE compliance_controls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    standard_id UUID NOT NULL REFERENCES compliance_standards(id) ON DELETE CASCADE,
    control_ref VARCHAR(100) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    category VARCHAR(255),
    parent_id UUID REFERENCES compliance_controls(id),
    severity alert_severity NOT NULL DEFAULT 'medium',
    automated BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_compliance_controls_standard ON compliance_controls(standard_id);

-- Compliance scores
CREATE TABLE compliance_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    standard_id UUID NOT NULL REFERENCES compliance_standards(id),
    overall_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    status compliance_status NOT NULL DEFAULT 'not_assessed',
    controls_total INTEGER NOT NULL DEFAULT 0,
    controls_passed INTEGER NOT NULL DEFAULT 0,
    controls_failed INTEGER NOT NULL DEFAULT 0,
    controls_na INTEGER NOT NULL DEFAULT 0,
    breakdown JSONB NOT NULL DEFAULT '{}',
    assessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_assessment TIMESTAMPTZ,
    assessed_by UUID REFERENCES users(id)
);

CREATE INDEX idx_compliance_scores_org ON compliance_scores(org_id);
