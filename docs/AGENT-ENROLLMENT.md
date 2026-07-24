# Agent Enrollment & Installation Guide

## Overview

Raksha uses a secure enrollment flow to register agents with the portal. This ensures only authorized machines can send telemetry data and receive configuration. The platform is **self-hosted** — each organization runs its own portal instance, guaranteeing complete data isolation.

## Who Can Create Agents?

Only **Admin** and **SuperAdmin** roles can generate enrollment tokens.

| Role | Generate Token | View Agents | Revoke Token | Rotate Certificate |
|------|:-:|:-:|:-:|:-:|
| SuperAdmin | ✅ | ✅ | ✅ | ✅ |
| Admin | ✅ | ✅ | ✅ | ✅ |
| Operator | ❌ | ✅ | ❌ | ❌ |
| Analyst | ❌ | ✅ | ❌ | ❌ |
| Viewer | ❌ | ✅ | ❌ | ❌ |

## Installation Flow (Step by Step)

```
┌───────────────────────────────────────────────────────────────────────────┐
│  STEP 1: Complete Setup Wizard → Portal is running                        │
│  STEP 2: Admin logs into Dashboard → navigates to "Agents" menu          │
│  STEP 3: Click "Add Agent" → Generate enrollment token                    │
│  STEP 4: Portal returns token + install commands (Linux & Windows)        │
│  STEP 5: Admin copies command, runs it on the target server               │
│  STEP 6: Agent auto-enrolls, receives mTLS certificate                    │
│  STEP 7: Agent starts sending telemetry → appears on dashboard            │
└───────────────────────────────────────────────────────────────────────────┘
```

> **Important**: The Setup Wizard must be completed first before any agents can be installed.

## Install Commands

When Admin generates a token, the system provides ready-to-use commands:

### Linux / macOS

```bash
curl -fsSL https://your-portal/api/v1/agent/install | \
  RAKSHA_TOKEN="rkat_xxxx" RAKSHA_PORTAL="https://your-portal" bash
```

### Windows (PowerShell as Administrator)

```powershell
$env:RAKSHA_TOKEN="rkat_xxxx"; $env:RAKSHA_PORTAL="https://your-portal"
irm https://your-portal/api/v1/agent/install.ps1 | iex
```

### What the Installer Does

1. Preflight checks — verifies root/admin, validates token format
2. System detection — OS, architecture, CPU, memory
3. Fingerprint collection — machine-id, MAC hash, hostname
4. Binary download — from portal or GitHub releases fallback
5. Enrollment — POST /api/v1/agents/enroll with token + fingerprint
6. Certificate issuance — receives mTLS cert for secure communication
7. Service installation — systemd (Linux) or Windows Service
8. Start monitoring — agent begins sending metrics immediately

## Enrollment Flow Diagram

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐
│  Admin UI    │     │   Portal API  │     │    Agent     │
└──────┬───────┘     └───────┬───────┘     └──────┬───────┘
       │                     │                     │
       │ 1. Generate Token   │                     │
       │────────────────────>│                     │
       │                     │                     │
       │ 2. Return token     │                     │
       │<────────────────────│                     │
       │                     │                     │
       │ 3. Copy install cmd │                     │
       │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ >│
       │                     │                     │
       │                     │ 4. POST /enroll     │
       │                     │<────────────────────│
       │                     │  (token+fingerprint)│
       │                     │                     │
       │                     │ 5. Validate token   │
       │                     │    Check expiry     │
       │                     │    Check uses       │
       │                     │                     │
       │                     │ 6. Issue mTLS cert  │
       │                     │────────────────────>│
       │                     │   (cert+config)     │
       │                     │                     │
       │                     │ 7. Agent online     │
       │                     │<────────────────────│
       │                     │   (heartbeat)       │
       └─────────────────────┴─────────────────────┘
```

## Token Format & Security

### Token Structure

```
rkat_{org_prefix_8chars}_{random_32hex}
Example: rkat_0192a3b4_deadbeef0123456789abcdef01234567
```

### Security Properties

| Property | Implementation |
|----------|---------------|
| Token secrecy | SHA-256 hash stored in DB; raw token shown only once |
| Token expiry | Default 24h, configurable per token |
| One-time use | `max_uses` counter, default 1 |
| Identity binding | Machine fingerprint (machine-id + MAC hash) |
| Transport security | mTLS after enrollment |
| Anti-cloning | Fingerprint uniqueness check per organization |
| Revocation | Admin can revoke tokens and certificates at any time |
| Audit trail | All enrollment events are logged immutably |

## Multi-Tenant Safety (Multiple Organizations)

Raksha is **self-hosted**. Each organization deploys its own portal instance.

### Why Agents Can Never "Go to the Wrong Portal"

| Layer | Protection Mechanism |
|-------|---------------------|
| Self-Hosted | Each company has its own portal at its own URL |
| Portal URL Lock | Agent's `portal_url` is hardcoded during enrollment |
| Token Prefix | Each org has a unique prefix (`rkat_{org_prefix}`) |
| Machine Fingerprint | Agent bound to machine-id + MAC hash |
| mTLS Certificate | After enrollment, only the specific portal's CA is trusted |
| One-Time Token | Token expires after single use within 24 hours |
| Anti-Cloning | Duplicate fingerprint detection prevents impersonation |

### Example: 10 Companies Using Raksha

```
Company A → Portal A (raksha.company-a.com) → Agents report ONLY to Portal A
Company B → Portal B (raksha.company-b.com) → Agents report ONLY to Portal B
Company C → Portal C (security.company-c.io) → Agents report ONLY to Portal C
```

Each portal is completely independent:
- Separate database
- Separate encryption keys
- Separate mTLS CA certificate
- No cross-portal communication possible

An agent enrolled on Portal A **cannot** send data to Portal B because:
1. The `portal_url` is locked to Portal A's address
2. The mTLS certificate was issued by Portal A's CA
3. Portal B's CA would reject the certificate as untrusted

## API Endpoints

### Generate Token (Admin/SuperAdmin only)

```http
POST /api/v1/agents/tokens
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "agent_name": "web-server-01",
  "labels": ["production", "web"],
  "expiry_hours": 24,
  "max_uses": 1,
  "allowed_modules": ["server", "network", "process"]
}
```

### Enroll Agent (Public — requires valid token)

```http
POST /api/v1/agents/enroll
Content-Type: application/json

{
  "token": "rkat_0192a3b4_deadbeef0123456789abcdef01234567",
  "fingerprint": {
    "hostname": "web-01.example.com",
    "os": "linux",
    "os_version": "6.1.0",
    "arch": "x86_64",
    "machine_id": "a1b2c3d4e5f6...",
    "cpu_cores": 8,
    "total_memory": 16000000000,
    "mac_hash": "sha256_of_primary_mac..."
  }
}
```

### Revoke Token (Admin)

```http
DELETE /api/v1/agents/tokens/{token_id}
Authorization: Bearer <admin_token>
```

### Rotate Certificate (Admin or Agent itself)

```http
POST /api/v1/agents/{agent_id}/rotate-certificate
Authorization: Bearer <admin_token> | mTLS client cert
```

## Certificate Lifecycle

| Phase | Duration | Action |
|-------|----------|--------|
| Issuance | Day 0 | 30-day cert issued during enrollment |
| Active | Day 0–23 | Normal operation |
| Rotation window | Day 23–30 | Agent auto-requests new cert |
| Expiry | Day 30+ | Agent must re-enroll if missed rotation |
| Revocation | Any time | Admin can revoke via UI or API |

## Platform Compatibility

| Platform | Architecture | Install Method |
|----------|-------------|----------------|
| Ubuntu / Debian | x86_64, ARM64 | Shell script + systemd |
| RHEL / CentOS / Rocky | x86_64, ARM64 | Shell script + systemd |
| Amazon Linux | x86_64, ARM64 | Shell script + systemd |
| macOS | x86_64, ARM64 (Apple Silicon) | Shell script + launchd |
| Windows Server | x86_64 | PowerShell + Windows Service |
| Alpine Linux | x86_64, ARM64 | Shell script + OpenRC |

## Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| `invalid_token_format` | Token doesn't match `rkat_*` | Re-generate from Portal UI |
| `token_expired` | Token older than expiry (default 24h) | Generate a new token |
| `token_already_used` | One-time token already consumed | Generate a new token |
| `duplicate_agent` | Machine already enrolled | Deregister old agent first |
| `certificate_expired` | Agent cert not rotated in time | Re-enroll the agent |
| `connection_refused` | Portal URL unreachable | Check network/firewall rules |
| `tls_handshake_failed` | mTLS cert mismatch | Verify portal CA, rotate cert |

---

*Part of the Nawala Ecosystem* 🔱
