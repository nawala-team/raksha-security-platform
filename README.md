<p align="center">
  <img src="assets/images/logo.svg" alt="Raksha Logo" width="160" height="160"/>
</p>

<h1 align="center">Raksha Security Platform</h1>

<p align="center">
  <strong>Enterprise-grade infrastructure security monitoring, compliance auditing, and threat detection.</strong>
</p>

<p align="center">
  <a href="https://github.com/nawala-team/raksha-security-platform/releases/tag/v1.3.1"><img src="https://img.shields.io/badge/version-1.3.1-blue?style=for-the-badge" alt="Version"/></a>
  <a href="#features"><img src="https://img.shields.io/badge/status-active-brightgreen?style=for-the-badge" alt="Status"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge" alt="License"/></a>
  <a href="#nawala-ecosystem"><img src="https://img.shields.io/badge/ecosystem-Nawala-orange?style=for-the-badge" alt="Nawala"/></a>
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/rust-1.75+-orange?style=for-the-badge&logo=rust" alt="Rust"/></a>
  <a href="#"><img src="https://img.shields.io/badge/node-20+-green?style=for-the-badge&logo=node.js" alt="Node.js"/></a>
  <a href="#"><img src="https://img.shields.io/badge/python-3.11+-blue?style=for-the-badge&logo=python" alt="Python"/></a>
</p>

<p align="center">
  <a href="INSTALLATION.md">Installation</a> •
  <a href="docs/ARCHITECTURE.md">Architecture</a> •
  <a href="docs/API.md">API Reference</a> •
  <a href="docs/AGENT-ENROLLMENT.md">Agent Setup</a> •
  <a href="CHANGELOG.md">Changelog</a> •
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

## About

**Raksha** (रक्षा) — from Sanskrit, meaning *protection* or *guardian* — is a full-stack security monitoring platform that provides real-time infrastructure protection, compliance auditing, and ML-powered threat detection across your entire stack.

Part of the [Nawala Ecosystem](https://github.com/nawala-team/nawala-gateway-platform) — a suite of open-source security and infrastructure tools built by the **NAWALA TEAM** in Indonesia.

---

## Architecture

<p align="center">
  <img src="assets/gifs/architecture-full.svg" alt="Raksha Architecture" width="100%"/>
</p>

---

## Features

<p align="center">
  <img src="assets/svg/features-row1.svg" alt="Features Row 1" width="100%"/>
</p>

<p align="center">
  <img src="assets/svg/features-row2.svg" alt="Features Row 2" width="100%"/>
</p>

<p align="center">
  <img src="assets/svg/features-row3.svg" alt="Features Row 3" width="100%"/>
</p>

<p align="center">
  <img src="assets/svg/features-row3b.svg" alt="Features Row 3b" width="50%"/>
</p>

<p align="center">
  <img src="assets/svg/features-row4.svg" alt="Features Row 4" width="100%"/>
</p>

<p align="center">
  <img src="assets/svg/features-row5.svg" alt="Features Row 5" width="100%"/>
</p>

<p align="center">
  <img src="assets/svg/features-row6.svg" alt="Features Row 6" width="100%"/>
</p>

---

## Feature Status

[x] **Production Ready** — All features are fully implemented and tested.

| Category | Features |
|----------|----------|
| **Authentication** | Login, Register, JWT refresh, Role-based access |
| **User Management** | Users, Roles, Permissions, Multi-tenant |
| **Security Monitoring** | Real-time alerts, Threat detection, Incident response |
| **Infrastructure** | Servers, Containers, Network monitoring |
| **Database Guard** | PostgreSQL, MySQL, MongoDB, Redis, Oracle, MariaDB, SQL Server |
| **Compliance** | CIS, NIST, PCI-DSS, ISO 27001, GDPR, SOC2, HIPAA |
| **Threat Intelligence** | IOC feeds, Dark web monitoring, Attack surface |
| **File Integrity** | FIM events, Change tracking, Alerts |
| **GRC** | Risk management, Policy engine, Controls |
| **Deception** | Honeypots, Attacker tracking, Interactions |
| **Hunting** | Query builder, RQL validation, Scheduled runs |
| **Backup & Docs** | Backup jobs, Document management, Expiry tracking |
| **Audit** | Immutable audit trail, SHA3-256 integrity verification |

---

## Dashboard Preview

<p align="center">
  <img src="assets/gifs/dashboard-overview.svg" alt="Dashboard Overview" width="100%"/>
</p>

---

## ML-Powered Threat Detection

<p align="center">
  <img src="assets/gifs/threat-detection.svg" alt="ML Threat Detection" width="100%"/>
</p>

---

## Incident Response

<p align="center">
  <img src="assets/gifs/incident-response.svg" alt="Incident Response" width="100%"/>
</p>

---

## Notification System

<p align="center">
  <img src="assets/gifs/notification-system.svg" alt="Notification System" width="100%"/>
</p>

---

## Compliance Engine

<p align="center">
  <img src="assets/gifs/compliance-engine.svg" alt="Compliance Engine" width="100%"/>
</p>

---

## Honeypot Deception

<p align="center">
  <img src="assets/gifs/honeypot-deception.svg" alt="Honeypot Deception" width="100%"/>
</p>

---

## Dark Web Monitoring

<p align="center">
  <img src="assets/gifs/dark-web-monitor.svg" alt="Dark Web Monitor" width="100%"/>
</p>

---

## Agent Enrollment

<p align="center">
  <img src="assets/gifs/agent-enrollment.svg" alt="Agent Enrollment Flow" width="100%"/>
</p>

### Token Format

<p align="center">
  <img src="assets/diagrams/token-format.svg" alt="Token Format" width="100%"/>
</p>

### How It Works

1. **Admin** completes Setup Wizard → Portal is running
2. **Admin** logs in → navigates to "Agents" → clicks "Add Agent"
3. **Portal** generates a one-time enrollment token (`rkat_<org>_<random>`)
4. **Admin** copies the install command and runs it on the target server
5. **Agent** sends token + machine fingerprint to Portal
6. **Portal** validates token → issues mTLS certificate to agent
7. **Agent** starts reporting metrics over encrypted channel

### Install Commands (Generated by Portal)

```bash
# Linux / macOS
curl -fsSL https://your-portal/api/v1/agent/install | \
  RAKSHA_TOKEN="rkat_orgname_abc123..." RAKSHA_PORTAL="https://your-portal" bash
```

```powershell
# Windows (PowerShell as Administrator)
$env:RAKSHA_TOKEN="rkat_orgname_abc123..."
$env:RAKSHA_PORTAL="https://your-portal"
irm https://your-portal/api/v1/agent/install.ps1 | iex
```

### Multi-Tenant Security

Each organization deploys its **own independent Raksha portal** — no shared infrastructure:

| Isolation Layer | Mechanism |
|----------------|-----------|
| Portal URL | Hardcoded during enrollment — agent only reports to its portal |
| mTLS Certificates | Each portal runs its own CA — certs from Portal A rejected by Portal B |
| Organization ID | Embedded in token prefix (`rkat_<org>_...`) and validated server-side |
| Fingerprint Binding | machine-id + MAC hash + hostname locked to enrollment record |

### Supported Agent Platforms

| Platform | Architecture | Service Manager |
|----------|-------------|-----------------|
| **Linux** |||
| Ubuntu / Debian | x86_64, ARM64 | systemd |
| RHEL / CentOS / Rocky / AlmaLinux | x86_64, ARM64 | systemd |
| Amazon Linux / Oracle Linux | x86_64, ARM64 | systemd |
| Fedora / SUSE / openSUSE | x86_64, ARM64 | systemd |
| Arch Linux / Gentoo | x86_64 | systemd |
| **Container OS** |||
| Alpine Linux | x86_64, ARM64 | OpenRC |
| Flatcar / Bottlerocket / CoreOS | x86_64, ARM64 | systemd |
| VMware Photon OS / Talos | x86_64, ARM64 | systemd |
| **BSD** |||
| FreeBSD / OpenBSD / NetBSD | x86_64, ARM64 | rc.d |
| DragonFly BSD | x86_64 | rc.d |
| **Enterprise Unix** |||
| Oracle Solaris / illumos | x86_64, SPARC | SMF |
| IBM AIX | POWER | SRC |
| HP-UX | IA-64 | init |
| **Other** |||
| macOS | x86_64, Apple Silicon | launchd |
| Windows Server | x86_64 | Windows Service |

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Core Engine** | Rust (Axum + Tower) | High-performance security scanning & API |
| **Web Dashboard** | Next.js 14 + TypeScript + Tailwind | Interactive security dashboard |
| **ML Engine** | Python + scikit-learn + PyTorch | Threat detection & anomaly models |
| **Primary Database** | PostgreSQL 16 | Primary data store |
| **Monitored Databases** | PostgreSQL, MySQL, MongoDB, Redis, Oracle, MariaDB, SQL Server | Database Guard support |
| **Time-series** | TimescaleDB | Metrics storage |
| **Cache** | Redis | Session & query caching |
| **Message Queue** | Apache Kafka | Event streaming |
| **Object Storage** | S3-compatible | Reports & artifacts |
| **Container** | Docker + Kubernetes + Helm | Deployment & orchestration |

---

## Quick Start

```bash
git clone https://github.com/nawala-team/raksha-security-platform.git
cd raksha-security-platform
cargo run -p raksha-installer
```

Open **http://localhost:3000** → Follow the wizard → Done! 

The Installation Wizard will:
- [x] Check system requirements
- [x] Configure database connection
- [x] Create your SuperAdmin account
- [x] Generate secure secrets automatically

> **Alternative methods:** See [INSTALLATION.md](INSTALLATION.md) for Docker, Docker Compose, or manual installation.

---

### Docker (Alternative)

```bash
docker run -d \
  --name raksha \
  -p 8080:8080 \
  -p 9090:9090 \
  -v raksha-data:/var/lib/raksha \
  ghcr.io/nawala-team/raksha:latest
```

### Docker Compose

```bash
git clone https://github.com/nawala-team/raksha-security-platform.git
cd raksha-security-platform
docker compose up -d
```

### Native Installation

Full step-by-step instructions are in [`INSTALLATION.md`](INSTALLATION.md). Quick
summary for bare-metal / Termux:

```bash
# Prerequisites: Rust 1.75+, Node.js 20+, PostgreSQL 13+, Redis 6+
git clone https://github.com/nawala-team/raksha-security-platform.git
cd raksha-security-platform
cp .env.example .env           # set DB URL, Redis URL, JWT secret

# 1. Start PostgreSQL + Redis
pg_ctl -D ~/postgresql-data -l ~/pg.log start
redis-server --port 6379 --save ''
psql -d postgres -c "CREATE ROLE raksha SUPERUSER LOGIN PASSWORD 'raksha_dev';"
psql -d postgres -c "CREATE DATABASE raksha_platform OWNER raksha;"

# 2. Apply migrations (creates all tables + seed roles/tenant)
for f in migrations/*.sql; do
  psql "$DATABASE_URL" -f "$f"
done

# 3. Build & run the portal API
export DATABASE_URL=postgres://raksha:raksha_dev@localhost:5432/raksha_platform
cargo build -p raksha-portal
./target/debug/raksha-portal

# 4. Build & run the web dashboard (separate terminal)
cd apps/web
PORTAL_API_URL=http://localhost:8080 npm install
PORTAL_API_URL=http://localhost:8080 npm run build
PORT=3000 PORTAL_API_URL=http://localhost:8080 npm run start
```

Dashboard runs at **http://localhost:3000** (it proxies `/api/*` to the portal).

> Tip: use `./start-local.sh` to start / stop / check the whole stack in one command.

---

## Configuration

Create `raksha.toml` in the project root:

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgres://raksha:secret@localhost:5432/raksha"
max_connections = 20

[security]
jwt_secret = "your-secret-key"
token_expiry = "24h"
enable_mtls = true

[monitoring]
interval_seconds = 30
retention_days = 90

[ml]
model_path = "./models"
anomaly_threshold = 0.85

[compliance]
standards = ["cis", "nist-800-53", "pci-dss", "iso-27001"]

[audit]
storage_backend = "postgresql"
enable_crypto_proof = true
hash_algorithm = "sha3-256"
```

Environment variables override config with the `RAKSHA_` prefix:

```bash
export RAKSHA_DATABASE__URL="postgres://user:pass@host:5432/raksha"
export RAKSHA_SERVER__PORT=9090
```

---

## Project Structure

```
raksha-security-platform/
+-- apps/
|   +-- portal/          # Rust API server (Axum)
|   +-- web/             # React frontend (Next.js 14)
|   +-- agent/           # Cross-platform agent (Rust)
|   +-- cli/             # CLI tool (Rust)
+-- crates/
|   +-- raksha-core/     # Shared types, tenant, hunting engine (RQL)
|   +-- raksha-auth/     # Authentication, RBAC, session, mTLS
|   +-- raksha-alert/    # Alert engine + notifications (Email/Telegram)
|   +-- raksha-audit/    # Audit trail with crypto proofs
|   +-- raksha-compliance/ # Compliance framework checks
|   +-- raksha-ml/       # ML integration bridge
|   +-- raksha-threat-intel/ # Threat intelligence feeds
|   +-- raksha-siem/     # SIEM integration (Splunk/Elastic/Graylog/Wazuh)
|   +-- raksha-incident/ # Incident response & playbook engine
|   +-- raksha-grc/      # Governance, Risk & Compliance
+-- ml/                  # Python ML/AI service (FastAPI)
|   +-- src/anomaly/     # Anomaly detection models
|   +-- src/scanner/     # Vulnerability scanner (NVD/CVE)
|   +-- src/asm/         # Attack Surface Management
|   +-- src/darkweb/     # Dark web monitoring
+-- configs/             # All configuration files
|   +-- playbooks/       # Incident response playbooks (YAML)
|   +-- deception/       # Honeypot configurations
|   +-- backup/          # Backup verification policies
|   +-- siem/            # SIEM integration defaults
|   +-- container-security/ # Container security policies
+-- deploy/              # Deployment infrastructure
|   +-- docker/          # Dockerfiles
|   +-- k8s/helm/        # Helm chart for Kubernetes
|   +-- k8s/admission-webhook/ # K8s admission controller (Go)
|   +-- gateway/         # Nginx WAF + reverse proxy
|   +-- fluentd/         # Log forwarding
+-- migrations/          # PostgreSQL migrations (SQLx)
+-- docs/                # Documentation
+-- scripts/             # Install scripts
+-- tests/               # E2E, performance, security tests
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | System design and component overview |
| [API Reference](docs/API.md) | REST and GraphQL API documentation |
| [Installation Guide](INSTALLATION.md) | Detailed setup instructions |
| [Agent Enrollment](docs/AGENT-ENROLLMENT.md) | Agent setup and enrollment flow |
| [Branching Strategy](docs/BRANCHING.md) | Git flow and branch protection rules |
| [Threat Intelligence](docs/THREAT-INTELLIGENCE.md) | IOC feeds and correlation |
| [Security Standards](docs/SECURITY-STANDARDS.md) | Supported compliance frameworks |
| [Security Testing](docs/SECURITY-TESTING.md) | Penetration testing playbook |
| [Bug Bounty](docs/BUG-BOUNTY.md) | Community recognition program |
| [Incident Response](docs/INCIDENT-RESPONSE.md) | Playbook engine and lifecycle |
| [Notification Setup](docs/NOTIFICATION-SETUP.md) | Email, Telegram, Webhook config |
| [Dark Web Monitoring](docs/DARK-WEB-MONITORING.md) | Credential leak detection setup |
| [Honeypot Deployment](docs/HONEYPOT-DEPLOYMENT.md) | Deception system configuration |
| [Container Security](docs/CONTAINER-SECURITY.md) | K8s admission and image scanning |

---

## Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md) before submitting pull requests.

---

## License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.

This is a **fully open-source** project -- free for personal, commercial, and enterprise use.

---

<p align="center">
  <img src="assets/images/logo.svg" alt="Raksha" width="48" height="48"/>
</p>

<p align="center">
  Created & Supported by <strong>NAWALA TEAM</strong> in Indonesia
</p>

<p align="center">
  <sub>Part of the <a href="https://github.com/nawala-team/nawala-gateway-platform">Nawala Ecosystem</a> -- Sanskrit-inspired tools for modern infrastructure</sub>
</p>

<p align="center">
  <sub>Contact: <a href="mailto:dummymailrangga@gmail.com">dummymailrangga@gmail.com</a></sub>
</p>
