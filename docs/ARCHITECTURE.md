#  Raksha Architecture

> System design and component overview for the Raksha Security Platform

## Overview

Raksha follows a modular, event-driven architecture built primarily in Rust for performance-critical paths, with Python for ML workloads and TypeScript for the web dashboard. The system is designed for horizontal scalability and can operate in both single-node and distributed modes.

## System Layers

### 1. Presentation Layer

| Component | Technology | Role |
|-----------|-----------|------|
| Web Dashboard | React + TypeScript | Interactive UI for security teams |
| CLI Tool | Rust (clap) | Command-line administration |
| REST API | Axum | External integrations |
| GraphQL API | async-graphql | Flexible querying for dashboard |

### 2. Gateway Layer

The API Gateway handles:
- **Authentication**: JWT tokens with refresh rotation
- **mTLS**: Mutual TLS for service-to-service communication
- **RBAC**: Role-based access control (Admin, Analyst, Viewer)
- **Rate Limiting**: Token bucket algorithm per client
- **Request Validation**: Schema validation and sanitization

### 3. Service Layer (Core Modules)

#### Server Monitor (`raksha-server-monitor`)
- Agent-based metrics collection (CPU, memory, disk, processes)
- Threshold-based alerting with hysteresis
- Historical trend analysis
- Integration with Prometheus metrics format

#### Network Scanner (`raksha-network`)
- Async port scanning with configurable concurrency
- Service fingerprinting and version detection
- Vulnerability correlation with CVE databases
- Network topology discovery and mapping
- Scheduled and on-demand scanning modes

#### Database Guard (`raksha-db-guard`)
- Connection auditing and query analysis
- Permission verification against least-privilege policies
- Encryption-at-rest validation
- Backup integrity checking
- Support for PostgreSQL, MySQL, MongoDB

#### Compliance Engine (`raksha-compliance`)
- Rule-based policy engine with custom rule support
- Built-in standards: CIS Benchmarks, NIST 800-53, PCI-DSS, ISO 27001
- Evidence collection and report generation
- Remediation guidance with priority scoring
- Continuous compliance monitoring

#### ML Threat Detection (`raksha-ml`)
- Anomaly detection using Isolation Forest and Autoencoders
- Time-series forecasting for predictive alerts
- Behavioral baseline learning per asset
- Model versioning and A/B testing
- Batch and real-time inference pipelines

#### Audit Trail (`raksha-audit`)
- Append-only event log with hash chaining
- Cryptographic proof of integrity (Merkle trees)
- Tamper detection and alerting
- Configurable retention policies
- Export to SIEM systems

### 4. Data Layer

| Store | Purpose | Characteristics |
|-------|---------|-----------------|
| PostgreSQL | Primary data, configs, audit logs | ACID, relational queries |
| Redis | Cache, sessions, real-time state | Low-latency, pub/sub |
| Apache Kafka | Event streaming, module communication | Ordered, durable, scalable |
| S3-compatible | Reports, ML models, artifacts | Object storage, versioned |

## Communication Patterns

### Synchronous (Request/Response)
- REST API calls from dashboard/CLI to gateway
- GraphQL queries for complex data fetching
- gRPC for internal service-to-service calls requiring immediate response

### Asynchronous (Event-Driven)
- Kafka topics for scan results, alerts, and audit events
- Redis pub/sub for real-time dashboard updates
- Scheduled tasks via internal cron scheduler

### Event Topics

```
raksha.scans.network.started
raksha.scans.network.completed
raksha.scans.server.metrics
raksha.alerts.critical
raksha.alerts.warning
raksha.compliance.check.completed
raksha.audit.event
raksha.ml.anomaly.detected
raksha.ml.model.updated
```

## Deployment Architecture

### Single Node (Development / Small)

```
┌─────────────────────────┐
│    Single Process        │
│  ┌─────┐ ┌─────┐       │
│  │ API │ │ Core│        │
│  └──┬──┘ └──┬──┘       │
│     │        │          │
│  ┌──┴────────┴──┐      │
│  │  Embedded DB  │      │
│  │  (SQLite)     │      │
│  └───────────────┘      │
└─────────────────────────┘
```

### Distributed (Production)

```
┌──────────┐  ┌──────────┐  ┌──────────┐
│ Dashboard│  │   CLI    │  │ External │
│  (CDN)   │  │          │  │  APIs    │
└────┬─────┘  └────┬─────┘  └────┬─────┘
     │              │              │
┌────┴──────────────┴──────────────┴─────┐
│          Load Balancer (L7)             │
└────┬──────────────┬──────────────┬─────┘
     │              │              │
┌────┴────┐  ┌─────┴────┐  ┌─────┴────┐
│ API Pod │  │ API Pod  │  │ API Pod  │
│  (x3)   │  │  (x3)    │  │  (x3)    │
└────┬────┘  └─────┬────┘  └─────┬────┘
     │              │              │
┌────┴──────────────┴──────────────┴─────┐
│              Kafka Cluster              │
└────┬──────────────┬──────────────┬─────┘
     │              │              │
┌────┴────┐  ┌─────┴────┐  ┌─────┴────┐
│ Scanner │  │Compliance│  │    ML    │
│ Workers │  │ Workers  │  │ Workers  │
└────┬────┘  └─────┬────┘  └─────┴────┘
     │              │              │
┌────┴──────────────┴──────────────┴─────┐
│         PostgreSQL (Primary/Replica)    │
│         Redis Cluster                   │
│         S3 Object Store                 │
└────────────────────────────────────────┘
```

## Security Considerations

- All inter-service communication uses mTLS
- Secrets managed via environment variables or HashiCorp Vault
- Database connections use SSL with certificate pinning
- Audit trail is cryptographically sealed (hash chains)
- RBAC enforced at gateway and service levels
- Input validation at every service boundary
- Rate limiting and circuit breakers prevent cascade failures

## Directory Structure

```
raksha-security-platform/
├── crates/
│   ├── raksha-core/          # Shared types, traits, utilities
│   ├── raksha-server/        # API server binary
│   ├── raksha-server-monitor/# Server monitoring module
│   ├── raksha-network/       # Network scanner module
│   ├── raksha-db-guard/      # Database guard module
│   ├── raksha-compliance/    # Compliance engine
│   ├── raksha-audit/         # Audit trail module
│   └── raksha-cli/           # CLI binary
├── dashboard/                # React TypeScript web UI
├── ml/                       # Python ML threat detection
├── migrations/               # Database migrations
├── deploy/                   # Deployment configs (Docker, K8s, Terraform)
├── docs/                     # Documentation
└── tests/                    # Integration and E2E tests
```

---

*Part of the Nawala Ecosystem* 
