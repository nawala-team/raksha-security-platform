-- Migration: 015_create_ml_models
-- Description: Create ML models registry table
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for ML models
CREATE TYPE ml_model_type AS ENUM (
    'anomaly_detection',
    'classification',
    'regression',
    'clustering',
    'nlp',
    'time_series',
    'reinforcement'
);

CREATE TYPE ml_model_status AS ENUM (
    'training',
    'validating',
    'ready',
    'deployed',
    'deprecated',
    'failed'
);

CREATE TABLE ml_models (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(255) NOT NULL,
    version         VARCHAR(50) NOT NULL,
    type            ml_model_type NOT NULL,
    status          ml_model_status NOT NULL DEFAULT 'training',
    description     TEXT,
    metrics         JSONB DEFAULT '{}',
    hyperparameters JSONB DEFAULT '{}',
    file_path       TEXT,
    file_size       BIGINT,
    checksum        TEXT,
    training_data   JSONB DEFAULT '{}',
    feature_columns JSONB DEFAULT '[]',
    target_column   VARCHAR(255),
    framework       VARCHAR(100),
    runtime_version VARCHAR(50),
    org_id          UUID REFERENCES organizations(id) ON DELETE CASCADE,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    deployed_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_model_name_version UNIQUE (name, version, org_id),
    CONSTRAINT chk_model_version_format CHECK (version ~ '^\d+\.\d+\.\d+

-- Model predictions log
CREATE TABLE ml_predictions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    model_id    UUID NOT NULL REFERENCES ml_models(id) ON DELETE CASCADE,
    input_data  JSONB NOT NULL,
    output_data JSONB NOT NULL,
    confidence  NUMERIC(5,4),
    latency_ms  INTEGER,
    feedback    VARCHAR(50),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Model training runs
CREATE TABLE ml_training_runs (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    model_id    UUID NOT NULL REFERENCES ml_models(id) ON DELETE CASCADE,
    status      VARCHAR(50) NOT NULL DEFAULT 'running',
    started_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    metrics     JSONB DEFAULT '{}',
    logs        TEXT,
    error       TEXT
);

-- Indexes
CREATE INDEX idx_ml_models_name ON ml_models (name);
CREATE INDEX idx_ml_models_type ON ml_models (type);
CREATE INDEX idx_ml_models_status ON ml_models (status);
CREATE INDEX idx_ml_models_org ON ml_models (org_id);
CREATE INDEX idx_ml_models_deployed ON ml_models (status) WHERE status = 'deployed';

CREATE INDEX idx_ml_predictions_model ON ml_predictions (model_id, created_at DESC);
CREATE INDEX idx_ml_predictions_confidence ON ml_predictions (confidence);

CREATE INDEX idx_ml_training_runs_model ON ml_training_runs (model_id, started_at DESC);
CREATE INDEX idx_ml_training_runs_status ON ml_training_runs (status);

-- Triggers
CREATE TRIGGER set_ml_models_updated_at
    BEFORE UPDATE ON ml_models
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE ml_models IS 'Registry of machine learning models for security analytics';
COMMENT ON TABLE ml_predictions IS 'Log of model predictions for monitoring and feedback';
COMMENT ON TABLE ml_training_runs IS 'Training run history for model versioning';
COMMENT ON COLUMN ml_models.metrics IS 'Model performance metrics (accuracy, precision, recall, F1, AUC)';
COMMENT ON COLUMN ml_models.hyperparameters IS 'Training hyperparameters used';
COMMENT ON COLUMN ml_models.framework IS 'ML framework (pytorch, tensorflow, scikit-learn, xgboost)';
)
);

-- Model predictions log
CREATE TABLE ml_predictions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    model_id    UUID NOT NULL REFERENCES ml_models(id) ON DELETE CASCADE,
    input_data  JSONB NOT NULL,
    output_data JSONB NOT NULL,
    confidence  NUMERIC(5,4),
    latency_ms  INTEGER,
    feedback    VARCHAR(50),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Model training runs
CREATE TABLE ml_training_runs (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    model_id    UUID NOT NULL REFERENCES ml_models(id) ON DELETE CASCADE,
    status      VARCHAR(50) NOT NULL DEFAULT 'running',
    started_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    metrics     JSONB DEFAULT '{}',
    logs        TEXT,
    error       TEXT
);

-- Indexes
CREATE INDEX idx_ml_models_name ON ml_models (name);
CREATE INDEX idx_ml_models_type ON ml_models (type);
CREATE INDEX idx_ml_models_status ON ml_models (status);
CREATE INDEX idx_ml_models_org ON ml_models (org_id);
CREATE INDEX idx_ml_models_deployed ON ml_models (status) WHERE status = 'deployed';

CREATE INDEX idx_ml_predictions_model ON ml_predictions (model_id, created_at DESC);
CREATE INDEX idx_ml_predictions_confidence ON ml_predictions (confidence);

CREATE INDEX idx_ml_training_runs_model ON ml_training_runs (model_id, started_at DESC);
CREATE INDEX idx_ml_training_runs_status ON ml_training_runs (status);

-- Triggers
CREATE TRIGGER set_ml_models_updated_at
    BEFORE UPDATE ON ml_models
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE ml_models IS 'Registry of machine learning models for security analytics';
COMMENT ON TABLE ml_predictions IS 'Log of model predictions for monitoring and feedback';
COMMENT ON TABLE ml_training_runs IS 'Training run history for model versioning';
COMMENT ON COLUMN ml_models.metrics IS 'Model performance metrics (accuracy, precision, recall, F1, AUC)';
COMMENT ON COLUMN ml_models.hyperparameters IS 'Training hyperparameters used';
COMMENT ON COLUMN ml_models.framework IS 'ML framework (pytorch, tensorflow, scikit-learn, xgboost)';
