# Honeypot Deployment Guide

> Zero false-positive deception system for early threat detection

## Overview

Raksha deploys lightweight honeypots that mimic real services. Any interaction with a honeypot is automatically classified as malicious since legitimate users and systems have no reason to connect to them.

## Supported Honeypot Types

| Type | Port | Captures |
|------|------|----------|
| SSH | 2222 | Credentials, commands, sessions |
| HTTP | 8888 | Scan patterns, exploit attempts, payloads |
| SMTP | 2525 | Spam, relay attempts, phishing |
| MySQL | 3307 | Auth brute-force, SQL injection, queries |

## Configuration

In `configs/deception/honeypots.yml`:

```yaml
honeypots:
  ssh:
    enabled: true
    port: 2222
    banner: "SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.4"
    max_auth_attempts: 5
    record_sessions: true
    fake_filesystem: true

  http:
    enabled: true
    port: 8888
    paths:
      - /admin
      - /wp-login.php
      - /phpmyadmin
      - /.env
      - /api/v1/admin
    response_delay_ms: 200
    capture_payloads: true

  smtp:
    enabled: true
    port: 2525
    banner: "220 mail.internal ESMTP Postfix"
    allow_relay: false
    capture_emails: true

  mysql:
    enabled: true
    port: 3307
    version: "8.0.33-0ubuntu0.22.04.2"
    fake_databases: ["production", "users", "payments"]
    log_queries: true
```

## Deployment Modes

### Standalone (on agent)

Honeypots run as part of the Raksha agent:

```toml
[agent.honeypots]
enabled = true
types = ["ssh", "http"]
```

### Dedicated (separate container)

```bash
docker run -d \
  --name raksha-honeypot \
  -p 2222:2222 -p 8888:8888 -p 2525:2525 -p 3307:3307 \
  -e RAKSHA_PORTAL="https://your-portal" \
  -e RAKSHA_TOKEN="rkat_xxx" \
  ghcr.io/dansiapa/raksha-honeypot:latest
```

## Alert Behavior

- **Zero false positives**: Any connection = confirmed threat
- **Instant notification**: Telegram + Email within seconds
- **Auto-blocking**: Attacker IP propagated to all agents
- **Evidence capture**: Full session recording for forensics
- **MITRE mapping**: Auto-tagged with ATT&CK techniques

## Attacker Profiling

When a honeypot is triggered, Raksha automatically:

1. Logs source IP, timestamp, and interaction details
2. Enriches IP via threat intel feeds (AbuseIPDB, VirusTotal)
3. Checks if IP appears in other agent telemetry
4. Creates an incident with full context
5. Blocks IP across all enrolled agents
6. Sends alert to configured channels

## Network Placement

Recommended honeypot placement:

- **DMZ**: HTTP honeypot (detect external scanning)
- **Internal network**: SSH + MySQL (detect lateral movement)
- **Server VLAN**: SMTP (detect compromised hosts)
- **Cloud VPC**: All types (detect cloud-based attacks)

---

*Part of the Nawala Ecosystem*
