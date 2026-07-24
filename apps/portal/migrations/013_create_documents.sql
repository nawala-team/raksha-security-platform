-- Migration: 013_create_documents
-- Description: Create documents table with versioning support
-- Created: 2024-01-01
-- Database: PostgreSQL 15+

-- Create enums for documents
CREATE TYPE document_status AS ENUM (
    'draft',
    'published',
    'archived',
    'deleted'
);

CREATE TYPE document_type AS ENUM (
    'policy',
    'procedure',
    'guideline',
    'standard',
    'report',
    'template',
    'evidence',
    'risk_assessment',
    'incident_report'
);

CREATE TABLE documents (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title           VARCHAR(500) NOT NULL,
    slug            VARCHAR(255) NOT NULL,
    doc_type        document_type NOT NULL,
    status          document_status NOT NULL DEFAULT 'draft',
    content         TEXT,
    content_format  VARCHAR(20) NOT NULL DEFAULT 'markdown',
    file_path       TEXT,
    file_size       BIGINT,
    mime_type       VARCHAR(255),
    checksum        TEXT,
    version         INTEGER NOT NULL DEFAULT 1,
    org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    parent_id       UUID REFERENCES documents(id) ON DELETE SET NULL,
    tags            JSONB DEFAULT '[]',
    metadata        JSONB DEFAULT '{}',
    access_level    VARCHAR(50) NOT NULL DEFAULT 'internal',
    retention_until DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_document_slug_org UNIQUE (slug, org_id),
    CONSTRAINT chk_version_positive CHECK (version > 0),
    CONSTRAINT chk_file_size_positive CHECK (file_size IS NULL OR file_size > 0)
);

-- Document versions history
CREATE TABLE document_versions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version     INTEGER NOT NULL,
    title       VARCHAR(500) NOT NULL,
    content     TEXT,
    file_path   TEXT,
    checksum    TEXT,
    change_summary TEXT,
    created_by  UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_doc_version UNIQUE (document_id, version),
    CONSTRAINT chk_doc_version_positive CHECK (version > 0)
);

-- Indexes
CREATE INDEX idx_documents_org_id ON documents (org_id);
CREATE INDEX idx_documents_doc_type ON documents (doc_type);
CREATE INDEX idx_documents_status ON documents (status);
CREATE INDEX idx_documents_slug ON documents (slug);
CREATE INDEX idx_documents_created_by ON documents (created_by);
CREATE INDEX idx_documents_parent ON documents (parent_id);
CREATE INDEX idx_documents_tags ON documents USING GIN (tags);
CREATE INDEX idx_documents_metadata ON documents USING GIN (metadata);
CREATE INDEX idx_documents_retention ON documents (retention_until) WHERE retention_until IS NOT NULL;

CREATE INDEX idx_document_versions_doc ON document_versions (document_id, version DESC);

-- Full text search on documents
ALTER TABLE documents ADD COLUMN search_vector tsvector;

CREATE INDEX idx_documents_fts ON documents USING GIN (search_vector);

CREATE OR REPLACE FUNCTION documents_update_search_vector()
RETURNS TRIGGER AS $
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.content, '')), 'B');
    RETURN NEW;
END;
$ LANGUAGE plpgsql;

CREATE TRIGGER documents_search_update
    BEFORE INSERT OR UPDATE OF title, content ON documents
    FOR EACH ROW
    EXECUTE FUNCTION documents_update_search_vector();

-- Triggers
CREATE TRIGGER set_documents_updated_at
    BEFORE UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

-- Comments
COMMENT ON TABLE documents IS 'Document management with versioning and full-text search';
COMMENT ON TABLE document_versions IS 'Historical versions of documents for audit trail';
COMMENT ON COLUMN documents.access_level IS 'Access classification: public, internal, confidential, restricted';
COMMENT ON COLUMN documents.content_format IS 'Content format: markdown, html, plaintext';
COMMENT ON COLUMN documents.parent_id IS 'Parent document for hierarchical organization';
