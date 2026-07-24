<p align="center">
  <h1 align="center">🔱 Raksha Security Platform</h1>
  <p align="center">
    <em>Comprehensive infrastructure security monitoring and compliance platform</em>
  </p>
  <p align="center">
    <a href="#features"><img src="https://img.shields.io/badge/status-active-brightgreen" alt="Status"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License"></a>
    <a href="#nawala-ecosystem"><img src="https://img.shields.io/badge/ecosystem-Nawala-orange" alt="Nawala Ecosystem"></a>
    <a href="#"><img src="https://img.shields.io/badge/rust-1.75%2B-orange" alt="Rust"></a>
    <a href="#"><img src="https://img.shields.io/badge/node-20%2B-green" alt="Node.js"></a>
  </p>
</p>

---

## About

**Raksha** (रक्षा) — from Sanskrit, meaning *protection* or *guardian* — is an enterprise-grade security monitoring platform that provides real-time infrastructure protection, compliance auditing, and threat detection across your entire stack.

Part of the **Nawala Ecosystem** — a suite of security and infrastructure tools built with Sanskrit-inspired naming by the NAWALA TEAM in Indonesia.

## Features

| Module | Description |
|--------|-------------|
| 🖥️ **Server Monitor** | Real-time server health, resource utilization, and anomaly detection |
| 🌐 **Network Scanner** | Port scanning, vulnerability assessment, and network topology mapping |
| 🗄️ **Database Guard** | Database security auditing, access monitoring, and encryption verification |
| 📋 **Compliance Engine** | Automated compliance checking against CIS, NIST, PCI-DSS, ISO 27001 |
| 🤖 **ML Threat Detection** | Machine learning-powered anomaly detection and threat prediction |
| 🌍 **Threat Intelligence** | Auto-syncing IOC feeds from CISA, MITRE, Abuse.ch, OTX, NVD |
| 📝 **Audit Trail** | Immutable audit logging with cryptographic verification |
| 🔔 **Real-time Alerts** | WebSocket-based live alerting with Email, Slack, and Webhook notifications |
| 👥 **User Management** | RBAC with Admin, Analyst, Operator, Viewer roles and privilege control |
| 🕵️ **Agent System** | Cross-platform agents with mTLS enrollment and FIM |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Raksha Security Platform                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐   │
│  │  Web UI   │  │    CLI    │  │  REST API │  │  GraphQL  │   │
│  │  (React)  │  │  (Rust)   │  │  (Axum)   │  │  (Async)  │   │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘   │
│        │               │               │               │         │
│  ┌─────┴───────────────┴───────────────┴───────────────┴─────┐  │
│  │                    API Gateway / Auth                       │  │
│  │                  (JWT + mTLS + RBAC)                        │  │
│  └─────┬───────────────┬───────────────┬───────────────┬─────┘  │
│        │               │               │               │         │
│  ┌─────┴─────┐  ┌─────┴─────┐  ┌─────┴─────┐  ┌─────┴─────┐  │
│  │  Server   │  │  Network  │  │  Database  │  │ Compliance │  │
│  │  Monitor  │  │  Scanner  │  │   Guard    │  │   Engine   │  │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  │
│        │               │               │               │         │
│  ┌─────┴───────────────┴───────────────┴───────────────┴─────┐  │
│  │                    ML Threat Engine                         │  │
│  └─────┬─────────────────────────────────────────────────────┘  │
│        │                                                         │
│  ┌─────┴─────────────────────────────────────────────────────┐  │
│  │                    Audit Trail                              │  │
│  └─────┬─────────────────────────────────────────────────────┘  │
│        │                                                         │
│  ┌─────┴─────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐   │
│  │ PostgreSQL│  │   Redis   │  │   Kafka   │  │    S3     │   │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Docker (Recommended)

```bash
docker run -d \
  --name raksha \
  -p 8080:8080 \
  -p 9090:9090 \
  -v raksha-data:/var/lib/raksha \
  ghcr.io/dansiapa/raksha:latest
```

Or with Docker Compose:

```bash
git clone https://github.com/dansiapa/raksha-security-platform.git
cd raksha-security-platform
docker compose up -d
```

### Native Installation

```bash
# Prerequisites: Rust 1.75+, Node.js 20+, PostgreSQL 16+
git clone https://github.com/dansiapa/raksha-security-platform.git
cd raksha-security-platform
npm install
cargo build --release
cargo run --bin raksha-migrate
cargo run --bin raksha-server
```

The dashboard will be available at `http://localhost:8080`.

## Configuration

Raksha uses a layered configuration system. Create a `raksha.toml` in the project root:

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

Environment variables override config values with the `RAKSHA_` prefix:

```bash
export RAKSHA_DATABASE__URL="postgres://user:pass@host:5432/raksha"
export RAKSHA_SERVER__PORT=9090
```

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Core Engine | Rust | High-performance security scanning |
| API Server | Axum + Tower | HTTP/gRPC API with middleware |
| Web Dashboard | React + TypeScript | Interactive security dashboard |
| ML Engine | Python + scikit-learn | Threat detection models |
| Database | PostgreSQL | Primary data store |
| Cache | Redis | Session and query caching |
| Message Queue | Apache Kafka | Event streaming |
| Object Storage | S3-compatible | Report and artifact storage |
| Container | Docker + K8s | Deployment and orchestration |

## Screenshots

> 📸 Screenshots coming soon. The dashboard provides real-time visibility into your security posture.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design and component overview
- [API Reference](docs/API.md) — REST and GraphQL API documentation
- [Installation Guide](docs/INSTALLATION.md) — Detailed setup instructions
- [Agent Enrollment](docs/AGENT-ENROLLMENT.md) — Agent setup and enrollment flow
- [Threat Intelligence](docs/THREAT-INTELLIGENCE.md) — IOC feeds and correlation
- [Security Standards](docs/SECURITY-STANDARDS.md) — Supported compliance frameworks
- [Security Testing](docs/SECURITY-TESTING.md) — Penetration testing playbook
- [Bug Bounty](docs/BUG-BOUNTY.md) — Community recognition program

## Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md) before submitting pull requests.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with ❤️ by <strong>NAWALA TEAM</strong> in Indonesia 🇮🇩
</p>
<p align="center">
  <em>Part of the Nawala Ecosystem — Sanskrit-inspired tools for modern infrastructure</em>
</p>
