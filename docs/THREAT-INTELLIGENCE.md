# 🔱 Threat Intelligence

> Automated IOC feed synchronization and real-time correlation

## Overview

Raksha integrates with multiple open-source threat intelligence feeds to provide automatic Indicator of Compromise (IOC) correlation against your infrastructure telemetry. Feeds sync on configurable schedules and require no manual intervention.

## Supported Feeds

| Feed | Provider | Update Frequency | IOC Types |
|------|----------|-----------------|-----------|
| NVD | NIST | Every 2 hours | CVE, CVSS |
| CISA KEV | CISA | Every 6 hours | Known exploits |
| MITRE ATT&CK | MITRE | Every 24 hours | TTPs, Attack patterns |
| URLhaus | Abuse.ch | Every 30 min | Malicious URLs |
| MalwareBazaar | Abuse.ch | Every 1 hour | Malware hashes |
| Feodo Tracker | Abuse.ch | Every 1 hour | C2 IPs |
| AlienVault OTX | AT&T | Every 1 hour | Multi-type IOCs |
| Emerging Threats | Proofpoint | Every 12 hours | IDS signatures |
| PhishTank | OpenDNS | Every 1 hour | Phishing URLs |

## IOC Types

- IPv4 / IPv6 addresses
- Domain names
- URLs
- SHA-256 / MD5 / SHA-1 hashes
- Email addresses
- JA3 fingerprints

## How It Works

```
┌────────────────┐     ┌─────────────────┐     ┌──────────────┐
│  Threat Feeds  │────>│  Feed Manager   │────>│   IOC Store  │
│  (External)    │     │  (Sync + Parse) │     │   (Redis)    │
└────────────────┘     └─────────────────┘     └──────┬───────┘
                                                      │
                                                      ▼
┌────────────────┐     ┌─────────────────┐     ┌──────────────┐
│  Agent Data    │────>│  IOC Matcher    │────>│   Alerts     │
│  (Telemetry)   │     │  (Real-time)    │     │   Engine     │
└────────────────┘     └─────────────────┘     └──────────────┘
```

1. **Feed Manager** syncs IOCs from external sources on schedule
2. IOCs are normalized and stored in Redis for fast lookup
3. **IOC Matcher** correlates agent telemetry (IPs, domains, hashes) against the store
4. Matches trigger alerts through the Alert Engine

## Configuration

Feed configuration lives in `configs/feeds/feeds.yml`. Each feed supports:

- Custom sync intervals
- API key authentication (via environment variables)
- Rate limiting
- Priority-based processing

## API Keys (Optional)

Some feeds provide enhanced access with API keys:

```bash
export NVD_API_KEY="your-nvd-key"          # Higher rate limits
export OTX_API_KEY="your-otx-key"          # Access to subscriptions
export PHISHTANK_API_KEY="your-key"        # Direct JSON access
```

All feeds work without API keys at reduced rate limits.

## Manual Policy Addition

Administrators can add custom IOCs via the API:

```bash
curl -X POST http://localhost:8080/api/v1/threat-intel/iocs \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "ip",
    "value": "192.168.1.100",
    "severity": "high",
    "tags": ["internal", "suspicious"],
    "description": "Detected lateral movement source"
  }'
```

## Contact

- Maintainer: dummymailrangga@gmail.com
- Repository: https://github.com/dansiapa/raksha-security-platform
