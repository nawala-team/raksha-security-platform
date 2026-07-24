# Changelog

All notable changes to the Raksha Security Platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project scaffolding with Rust workspace
- Portal API server (Axum) with JWT authentication
- Web dashboard (Next.js + React + TypeScript)
- Setup Wizard for first-time installation
- Agent enrollment system with mTLS and token-based auth
- Cross-platform agent installer (Linux/macOS/Windows)
- Server monitoring module with real-time metrics
- Network security scanner with port/vulnerability detection
- Database security guard (PostgreSQL, MySQL, MongoDB, Redis)
- Compliance engine (CIS, NIST 800-53, PCI-DSS, ISO 27001)
- ML-based anomaly detection engine (Isolation Forest)
- Threat intelligence feed integration (CISA KEV, NVD, Abuse.ch, OTX, MITRE)
- Real-time IOC correlation and matching
- Audit trail with cryptographic integrity verification
- Role-Based Access Control (Admin, Analyst, Operator, Viewer)
- WebSocket-based real-time notifications
- Docker Compose deployment configuration
- Multi-stage Dockerfiles for all services

### Security
- mTLS for agent-to-portal communication
- Argon2id password hashing
- JWT with token rotation
- Rate limiting on all endpoints
- Input validation and sanitization
- RBAC enforcement at API gateway level

## [0.1.0] - 2026-07-24

### Added
- First public release
- Core platform architecture
- Full open-source under MIT license

---

Contact: dummymailrangga@gmail.com
Repository: https://github.com/dansiapa/raksha-security-platform
