# Notification Setup Guide

> Configure Email, Telegram, and Webhook notifications for Raksha alerts

## Overview

Raksha supports three notification channels that can be enabled independently or together. All channels support alert severity filtering, rate limiting, and deduplication.

## Email (SMTP)

### Requirements

- SMTP server (Gmail, SendGrid, Mailgun, or self-hosted)
- Valid credentials with send permission

### Configuration

In `raksha.toml`:

```toml
[notifications.email]
enabled = true
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "your-email@gmail.com"
smtp_password = "your-app-password"  # Use App Password for Gmail
use_tls = true
from_address = "raksha-alerts@yourcompany.com"
from_name = "Raksha Security"
recipients = ["security-team@yourcompany.com", "admin@yourcompany.com"]
```

Or via environment variables:

```bash
export RAKSHA_NOTIFICATIONS__EMAIL__ENABLED=true
export RAKSHA_NOTIFICATIONS__EMAIL__SMTP_HOST=smtp.gmail.com
export RAKSHA_NOTIFICATIONS__EMAIL__SMTP_PORT=587
export RAKSHA_NOTIFICATIONS__EMAIL__SMTP_USERNAME=your-email@gmail.com
export RAKSHA_NOTIFICATIONS__EMAIL__SMTP_PASSWORD=your-app-password
export RAKSHA_NOTIFICATIONS__EMAIL__USE_TLS=true
```

### Gmail Setup

1. Enable 2-Factor Authentication on your Google account
2. Go to: https://myaccount.google.com/apppasswords
3. Generate an App Password for "Mail"
4. Use that 16-character password as `smtp_password`

### SendGrid Setup

```toml
[notifications.email]
smtp_host = "smtp.sendgrid.net"
smtp_port = 587
smtp_username = "apikey"
smtp_password = "SG.your-sendgrid-api-key"
```

## Telegram Bot

### Requirements

- Telegram Bot Token (from @BotFather)
- Chat ID (group, channel, or individual)

### Step 1: Create a Bot

1. Open Telegram, search for `@BotFather`
2. Send `/newbot`
3. Choose a name: `Raksha Security Alerts`
4. Choose a username: `raksha_alerts_bot`
5. Copy the token: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`

### Step 2: Get Chat ID

For a **group**:
1. Add the bot to your group
2. Send a message in the group
3. Visit: `https://api.telegram.org/bot<TOKEN>/getUpdates`
4. Find `"chat":{"id":-1001234567890}`

For **individual**:
1. Send `/start` to your bot
2. Visit the getUpdates URL above
3. Find your chat ID

### Step 3: Configure

```toml
[notifications.telegram]
enabled = true
bot_token = "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
chat_ids = ["-1001234567890", "987654321"]
parse_mode = "MarkdownV2"
thread_id = 0  # Optional: for topics in supergroups
```

### Step 4: Test

```bash
curl -X POST http://localhost:8080/api/v1/notifications/test \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '''{"channel": "telegram", "message": "Test alert from Raksha"}'''
```

## Webhook

### Configuration

```toml
[notifications.webhook]
enabled = true
url = "https://your-endpoint.com/webhooks/raksha"
secret = "your-hmac-secret"  # Used to sign payloads
headers = { "X-Custom-Header" = "value" }
retry_count = 3
retry_delay_seconds = 5
timeout_seconds = 10
```

### Payload Format

```json
{
  "event": "alert.triggered",
  "severity": "critical",
  "timestamp": "2026-07-24T10:30:00Z",
  "source": "ml-engine",
  "title": "Anomaly detected on web-server-01",
  "description": "Network traffic anomaly score 0.94 (threshold: 0.85)",
  "metadata": {
    "agent_id": "agt_abc123",
    "hostname": "web-server-01",
    "module": "network_anomaly"
  }
}
```

### Signature Verification

Payloads are signed with HMAC-SHA256. Verify using the `X-Raksha-Signature` header:

```python
import hmac, hashlib

def verify_webhook(payload_bytes, signature, secret):
    expected = hmac.new(secret.encode(), payload_bytes, hashlib.sha256).hexdigest()
    return hmac.compare_digest(f"sha256={expected}", signature)
```

## Alert Severity Filtering

Each channel can filter by severity:

```toml
[notifications.email]
severity_filter = ["critical", "high"]  # Only critical and high

[notifications.telegram]
severity_filter = ["critical", "high", "medium"]  # All except low

[notifications.webhook]
severity_filter = ["critical", "high", "medium", "low"]  # Everything
```

## Rate Limiting

Prevent notification storms:

```toml
[notifications]
rate_limit_per_minute = 30
dedup_window_seconds = 300  # Suppress duplicate alerts for 5 min
quiet_hours_start = "23:00"
quiet_hours_end = "07:00"
quiet_hours_timezone = "Asia/Jakarta"
```

## Dashboard Configuration

Notifications can also be configured via the web UI:

1. Navigate to **Settings > Notifications**
2. Toggle channels on/off
3. Configure credentials
4. Use the **Test** button to verify
5. Set per-user notification preferences

---

*Part of the Nawala Ecosystem*
