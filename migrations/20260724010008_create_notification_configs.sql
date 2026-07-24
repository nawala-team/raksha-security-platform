-- Raksha Security Platform
-- Migration: 20260724010008_create_notification_configs
-- Description: Notification channel settings (email, telegram, slack, webhook)

CREATE TABLE notification_channels (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    channel_type    VARCHAR(20) NOT NULL
                    CHECK (channel_type IN ('email', 'telegram', 'slack', 'webhook',
                                           'pagerduty', 'msteams', 'opsgenie', 'sms')),
    is_enabled      BOOLEAN NOT NULL DEFAULT true,
    is_default      BOOLEAN NOT NULL DEFAULT false,
    config          JSONB NOT NULL DEFAULT '{}',
    secrets_enc     TEXT,
    last_test_at    TIMESTAMPTZ,
    last_test_ok    BOOLEAN,
    last_error      TEXT,
    send_count      BIGINT NOT NULL DEFAULT 0,
    error_count     BIGINT NOT NULL DEFAULT 0,
    rate_limit_per_hour INTEGER DEFAULT 100,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_notification_channel_name UNIQUE (name, tenant_id)
);

CREATE INDEX idx_notification_channels_tenant ON notification_channels(tenant_id);
CREATE INDEX idx_notification_channels_type ON notification_channels(channel_type);
CREATE INDEX idx_notification_channels_enabled ON notification_channels(is_enabled) WHERE is_enabled = true;

CREATE TRIGGER trg_notification_channels_updated_at
    BEFORE UPDATE ON notification_channels
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE notification_channels IS 'Notification delivery channels with encrypted credentials, rate limiting, and health tracking.';
COMMENT ON COLUMN notification_channels.config IS 'Channel-specific config: email(smtp_host, from), slack(channel, username), telegram(chat_id), webhook(url, headers).';
COMMENT ON COLUMN notification_channels.secrets_enc IS 'AES-256-GCM encrypted secrets: API tokens, SMTP passwords, webhook signing keys.';

-- Notification rules (which alerts trigger which channels)
CREATE TABLE notification_rules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    is_enabled      BOOLEAN NOT NULL DEFAULT true,
    channel_id      UUID NOT NULL REFERENCES notification_channels(id) ON DELETE CASCADE,
    severity_filter TEXT[] DEFAULT ARRAY['critical', 'high'],
    category_filter TEXT[],
    source_filter   TEXT[],
    agent_filter    UUID[],
    conditions      JSONB DEFAULT '{}',
    template_id     UUID,
    cooldown_mins   INTEGER DEFAULT 15,
    group_by        TEXT[],
    group_wait_secs INTEGER DEFAULT 30,
    schedule        JSONB,
    priority        VARCHAR(10) NOT NULL DEFAULT 'medium'
                    CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_rules_tenant ON notification_rules(tenant_id);
CREATE INDEX idx_notification_rules_channel ON notification_rules(channel_id);
CREATE INDEX idx_notification_rules_enabled ON notification_rules(is_enabled) WHERE is_enabled = true;

CREATE TRIGGER trg_notification_rules_updated_at
    BEFORE UPDATE ON notification_rules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE notification_rules IS 'Rules mapping alert conditions to notification channels with filtering, grouping, and cooldown.';
COMMENT ON COLUMN notification_rules.cooldown_mins IS 'Minimum minutes between repeated notifications for the same alert key.';
COMMENT ON COLUMN notification_rules.group_wait_secs IS 'Seconds to wait and batch grouped notifications before sending.';

-- Notification templates
CREATE TABLE notification_templates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    channel_type    VARCHAR(20) NOT NULL
                    CHECK (channel_type IN ('email', 'telegram', 'slack', 'webhook',
                                           'pagerduty', 'msteams', 'opsgenie', 'sms')),
    subject_template TEXT,
    body_template   TEXT NOT NULL,
    format          VARCHAR(10) NOT NULL DEFAULT 'text'
                    CHECK (format IN ('text', 'html', 'markdown', 'json')),
    variables       JSONB DEFAULT '[]',
    is_default      BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_templates_tenant ON notification_templates(tenant_id);
CREATE INDEX idx_notification_templates_type ON notification_templates(channel_type);

CREATE TRIGGER trg_notification_templates_updated_at
    BEFORE UPDATE ON notification_templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE notification_templates IS 'Customizable notification message templates with variable interpolation per channel type.';

-- Notification delivery log
CREATE TABLE notification_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID REFERENCES tenants(id) ON DELETE SET NULL,
    channel_id      UUID REFERENCES notification_channels(id) ON DELETE SET NULL,
    rule_id         UUID REFERENCES notification_rules(id) ON DELETE SET NULL,
    alert_id        UUID REFERENCES alerts(id) ON DELETE SET NULL,
    incident_id     UUID REFERENCES incidents(id) ON DELETE SET NULL,
    recipient       VARCHAR(500),
    subject         VARCHAR(500),
    status          VARCHAR(20) NOT NULL
                    CHECK (status IN ('pending', 'sent', 'delivered', 'failed', 'bounced',
                                     'rate_limited', 'suppressed')),
    error_message   TEXT,
    response_code   INTEGER,
    retry_count     SMALLINT NOT NULL DEFAULT 0,
    max_retries     SMALLINT NOT NULL DEFAULT 3,
    next_retry_at   TIMESTAMPTZ,
    sent_at         TIMESTAMPTZ,
    delivered_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_log_tenant ON notification_log(tenant_id);
CREATE INDEX idx_notification_log_channel ON notification_log(channel_id);
CREATE INDEX idx_notification_log_alert ON notification_log(alert_id) WHERE alert_id IS NOT NULL;
CREATE INDEX idx_notification_log_status ON notification_log(status) WHERE status IN ('pending', 'failed');
CREATE INDEX idx_notification_log_created ON notification_log(created_at DESC);
CREATE INDEX idx_notification_log_retry ON notification_log(next_retry_at) WHERE status = 'failed' AND retry_count < max_retries;

COMMENT ON TABLE notification_log IS 'Delivery log for all notification attempts with retry tracking and status history.';
