#  Raksha Security Standards

> Supported compliance frameworks and security standards

## Overview

Raksha's Compliance Engine provides automated assessment against industry-standard security frameworks. Each standard is implemented as a set of machine-readable rules that map to specific checks performed against your infrastructure.

## Supported Standards

### CIS Benchmarks

The Center for Internet Security (CIS) Benchmarks provide prescriptive configuration guidance for hardening operating systems, middleware, and network devices.

| Benchmark | Version | Coverage |
|-----------|---------|----------|
| CIS Ubuntu Linux | v2.0.0 | 95% |
| CIS Red Hat Enterprise Linux | v3.0.0 | 93% |
| CIS Amazon Linux 2 | v2.0.0 | 91% |
| CIS Docker | v1.6.0 | 88% |
| CIS Kubernetes | v1.8.0 | 85% |
| CIS PostgreSQL | v1.0.0 | 90% |
| CIS nginx | v2.0.0 | 87% |

**Key areas checked:**
- Filesystem configuration and permissions
- Service hardening and unnecessary service removal
- Network configuration and firewall rules
- Logging and auditing configuration
- Access control and authentication settings
- System maintenance procedures

### NIST 800-53 (Rev. 5)

The National Institute of Standards and Technology Special Publication 800-53 provides a catalog of security and privacy controls for information systems.

| Control Family | ID | Automated Checks |
|---------------|-----|-----------------|
| Access Control | AC | 42 |
| Audit and Accountability | AU | 28 |
| Configuration Management | CM | 35 |
| Identification and Authentication | IA | 31 |
| Incident Response | IR | 18 |
| System and Communications Protection | SC | 38 |
| System and Information Integrity | SI | 29 |

**Impact levels supported:**
- Low
- Moderate
- High

### PCI-DSS v4.0

Payment Card Industry Data Security Standard for organizations handling cardholder data.

| Requirement | Description | Checks |
|-------------|-------------|--------|
| Req 1 | Network Security Controls | 15 |
| Req 2 | Secure Configurations | 22 |
| Req 3 | Protect Account Data | 18 |
| Req 4 | Encrypt Transmission | 12 |
| Req 5 | Malware Protection | 8 |
| Req 6 | Secure Development | 24 |
| Req 7 | Restrict Access | 16 |
| Req 8 | Identify and Authenticate | 20 |
| Req 9 | Physical Access | 4 |
| Req 10 | Logging and Monitoring | 26 |
| Req 11 | Security Testing | 14 |
| Req 12 | Organizational Policies | 6 |

### ISO 27001:2022

International standard for Information Security Management Systems (ISMS).

**Annex A controls assessed:**

| Category | Controls | Automated |
|----------|----------|-----------|
| Organizational | A.5 | Partial |
| People | A.6 | Partial |
| Physical | A.7 | Limited |
| Technological | A.8 | Full |

Technological controls with full automation:
- A.8.1 — User endpoint devices
- A.8.5 — Secure authentication
- A.8.9 — Configuration management
- A.8.15 — Logging
- A.8.16 — Monitoring activities
- A.8.20 — Network security
- A.8.24 — Use of cryptography
- A.8.25 — Secure development lifecycle
- A.8.28 — Secure coding

## Custom Rules

Raksha supports custom compliance rules written in a declarative YAML format:

```yaml
rule:
  id: "CUSTOM-001"
  title: "Ensure SSH root login is disabled"
  severity: high
  standard: custom
  check:
    type: file_content
    path: /etc/ssh/sshd_config
    pattern: "^PermitRootLogin\\s+no"
    expect: present
  remediation: |
    Edit /etc/ssh/sshd_config and set:
    PermitRootLogin no
    Then restart sshd: systemctl restart sshd
```

## Report Formats

Compliance reports can be generated in:

| Format | Use Case |
|--------|----------|
| PDF | Executive summaries, auditor handoff |
| JSON | API consumption, automation |
| CSV | Spreadsheet analysis |
| HTML | Interactive viewing |
| OSCAL | NIST Open Security Controls (machine-readable) |

## Assessment Scheduling

```toml
[compliance]
# Run full assessment weekly (Sunday at midnight)
scan_schedule = "0 0 * * 0"

# Run critical checks daily
critical_schedule = "0 6 * * *"

# Continuous monitoring for high-priority controls
continuous_monitoring = true
continuous_interval_minutes = 15
```

## Evidence Collection

Raksha automatically collects evidence for each compliance check:
- Configuration file snapshots
- Command output captures
- Network scan results
- Permission enumerations
- Timestamps and hash verification

Evidence is stored with cryptographic integrity verification and can be exported for auditor review.

---

*Part of the Nawala Ecosystem* 
