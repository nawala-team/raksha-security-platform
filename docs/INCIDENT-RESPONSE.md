# Incident Response Guide

> Playbook engine, lifecycle management, and automated response

## Overview

Raksha provides a complete incident response framework with YAML-based playbooks, automated escalation, timeline tracking, and integration with the notification system.

## Incident Lifecycle

```
DETECT -> TRIAGE -> RESPOND -> CONTAIN -> ERADICATE -> RECOVER -> LESSONS LEARNED
```

## Playbook Engine

Playbooks are defined in YAML and stored in `configs/playbooks/`.

### Example: Ransomware Response

```yaml
name: ransomware_response
severity: critical
trigger:
  alert_type: malware_detected
  tags: [ransomware, encryption]
steps:
  - action: isolate_network
    target: "{{agent_id}}"
    timeout: 30s
  - action: snapshot_memory
    target: "{{agent_id}}"
  - action: block_iocs
    iocs: "{{extracted_iocs}}"
  - action: notify_team
    channels: [telegram, email]
    message: "Ransomware detected on {{hostname}}"
  - action: create_ticket
    priority: P1
    assignee: security-lead
  - action: escalate_if_spread
    condition: "affected_hosts > 3"
    escalate_to: incident-commander
post_incident:
  - generate_timeline
  - collect_evidence
  - create_report
```

### Example: Brute Force Response

```yaml
name: brute_force_response
severity: high
trigger:
  alert_type: auth_failure
  threshold: 10
  window: 5m
steps:
  - action: block_ip
    target: "{{source_ip}}"
    duration: 24h
  - action: lock_account
    target: "{{target_user}}"
    condition: "failures > 20"
  - action: notify_team
    channels: [telegram]
  - action: enrich_ip
    providers: [abuseipdb, virustotal]
```

## Auto-Escalation Rules

```toml
[incident.escalation]
critical_response_sla = "15m"
high_response_sla = "1h"
medium_response_sla = "4h"

# Escalate if not acknowledged within SLA
[incident.escalation.chain]
level_1 = "on-call-analyst"
level_2 = "security-lead"      # +15min
level_3 = "incident-commander"  # +30min
level_4 = "ciso"               # +1h
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/incidents` | List incidents |
| POST | `/incidents` | Create incident manually |
| GET | `/incidents/:id` | Get incident details |
| PUT | `/incidents/:id/status` | Update status |
| POST | `/incidents/:id/timeline` | Add timeline entry |
| POST | `/incidents/:id/evidence` | Attach evidence |
| GET | `/incidents/:id/report` | Generate report |
| GET | `/playbooks` | List playbooks |
| POST | `/playbooks/execute` | Manually trigger playbook |

## Metrics Tracked

- **MTTD** (Mean Time to Detect)
- **MTTR** (Mean Time to Respond)
- **MTTC** (Mean Time to Contain)
- Incidents by severity over time
- Playbook execution success rate
- Escalation frequency

---

*Part of the Nawala Ecosystem*
