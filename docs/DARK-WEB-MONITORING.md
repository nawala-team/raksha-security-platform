# Dark Web Monitoring

> Credential leak detection, paste monitoring, and domain watchlist

## Overview

Raksha monitors dark web sources, paste sites, and breach databases to detect when your organization credentials or domains appear in leaked data.

## Features

### 1. Credential Leak Detection
- Monitor company email domains
- Check against known breach databases
- Real-time alerts when new leaks found
- Password hash analysis

### 2. Paste Site Monitoring
- Pastebin, Ghostbin, etc.
- Keyword and regex watchlist
- Auto-classification of content type
- Historical archive with search

### 3. Domain Watchlist
- Typosquatting detection (Levenshtein distance)
- Lookalike domain registration alerts
- Brand impersonation monitoring
- Certificate transparency log scanning

## Configuration

```toml
[darkweb]
enabled = true
scan_interval_hours = 6

[darkweb.domains]
watch = ["yourcompany.com", "yourbrand.io"]
keywords = ["yourcompany", "internal-project-name"]

[darkweb.credentials]
email_domains = ["yourcompany.com", "corp.yourcompany.com"]
alert_on_plaintext = true
alert_on_hash = true

[darkweb.typosquat]
max_distance = 2
check_tlds = [".com", ".net", ".org", ".io", ".co"]
```

## Alert Examples

### Credential Leak Found
```json
{
  "type": "credential_leak",
  "severity": "critical",
  "source": "breach_database",
  "affected_emails": 12,
  "breach_name": "Example Corp 2026",
  "data_types": ["email", "password_hash", "name"],
  "recommended_action": "Force password reset for affected accounts"
}
```

### Typosquatting Detected
```json
{
  "type": "typosquat_domain",
  "severity": "high",
  "original": "yourcompany.com",
  "detected": "yourcompanny.com",
  "registered_date": "2026-07-20",
  "registrar": "namecheap",
  "recommended_action": "Report domain, add to blocklist"
}
```

## Data Sources

| Source | Type | Update Frequency |
|--------|------|------------------|
| Have I Been Pwned (API) | Breach DB | Real-time |
| Paste monitors | Text dumps | Every 30 min |
| Certificate Transparency | New certs | Every 1 hour |
| Domain registrars | New registrations | Every 6 hours |
| Underground forums | Listings | Every 12 hours |

## Privacy

- Raksha never stores full plaintext passwords
- Only hash prefixes are used for breach checking (k-anonymity)
- All monitoring data stored locally in your portal
- No data sent to external services beyond API queries

---

*Part of the Nawala Ecosystem*
