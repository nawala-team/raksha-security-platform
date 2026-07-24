# Agent Enrollment

## Overview

Raksha uses a secure enrollment flow to register agents with the portal. This ensures only authorized machines can send telemetry data and receive configuration.

## Flow Diagram

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

## Security Properties

| Property | Implementation |
|----------|---------------|
| Token secrecy | SHA-256 hash stored in DB; raw token shown once |
| Token expiry | Default 24h, configurable |
| One-time use | `max_uses` counter, default 1 |
| Identity binding | Machine fingerprint (machine-id + MAC hash) |
| Transport security | mTLS after enrollment |
| Anti-cloning | Fingerprint uniqueness check per org |
| Revocation | Admin can revoke tokens and certificates |
| Audit trail | All enrollment events logged |

## Token Format

```
rkat_{org_prefix_8chars}_{random_32hex}
Example: rkat_0192a3b4_deadbeef0123456789abcdef01234567
```

## API Endpoints

### Generate Token (Admin)
```
POST /api/v1/agents/tokens
Authorization: Bearer <admin_token>

{
  "agent_name": "web-server-01",
  "labels": ["production", "web"],
  "expiry_hours": 24,
  "max_uses": 1,
  "allowed_modules": ["server", "network", "process"]
}
```

### Enroll Agent (Public with token)
```
POST /api/v1/agents/enroll

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
```
DELETE /api/v1/agents/tokens/{token_id}
```

### Rotate Certificate (Agent/Admin)
```
POST /api/v1/agents/{agent_id}/rotate-certificate
```

## Certificate Lifecycle

1. **Issuance**: 30-day validity, issued during enrollment
2. **Auto-rotation**: Agent requests new cert 7 days before expiry
3. **Revocation**: Admin can revoke via portal UI or API
4. **Expiry**: Agent must re-enroll if cert expires without rotation

## Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| `invalid_token_format` | Token doesn't match `rkat_*` pattern | Re-generate token from Portal UI |
| `token_expired` | Token older than expiry (default 24h) | Generate a new token |
| `token_already_used` | One-time token already consumed | Generate a new token |
| `duplicate_agent` | Machine already enrolled in this org | Deregister old agent first |
| `certificate_expired` | Agent cert not rotated in time | Re-enroll the agent |
