-- Raksha Security Platform
-- Monitored database instances for the Database Guard module.
-- Enables real (non-stub) register / list / get / remove of monitored DBs.

CREATE TABLE IF NOT EXISTS monitored_databases (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name                VARCHAR(255) NOT NULL,
    db_type             VARCHAR(50)  NOT NULL,        -- postgresql | mysql | mongodb | redis | ...
    host                VARCHAR(255) NOT NULL,
    port                INTEGER      NOT NULL,
    username            VARCHAR(255) NOT NULL,
    password_enc        TEXT,                          -- encrypted secret (never plaintext in logs)
    ssl_enabled         BOOLEAN      NOT NULL DEFAULT true,
    status              VARCHAR(20)  NOT NULL DEFAULT 'online',
    connections         INTEGER      NOT NULL DEFAULT 0,
    max_connections     INTEGER      NOT NULL DEFAULT 100,
    query_rate          BIGINT       NOT NULL DEFAULT 0,
    replication_lag_ms  BIGINT,
    size_bytes          BIGINT       NOT NULL DEFAULT 0,
    encrypted           BOOLEAN      NOT NULL DEFAULT true,
    version             VARCHAR(100),
    alerts              INTEGER      NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_monitored_databases_tenant ON monitored_databases(tenant_id);
CREATE INDEX IF NOT EXISTS idx_monitored_databases_type ON monitored_databases(db_type);
