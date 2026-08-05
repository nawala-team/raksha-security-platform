# Changelog

All notable changes to the Raksha Security Platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0] - 2026-08-05

### Added
- **E2E Tests**: Added comprehensive test scripts (`brutal_test.sh`, `brutal_test_v2.sh`, `extreme_test.sh`)
- **UI Components**: Added `dialog.tsx` and `textarea.tsx` components
- **API Endpoints**: Enhanced enrollment service with database persistence

### Changed

#### Agent
- Enhanced backup collector functionality
- Updated config and deception modules
- Improved reporter functionality

#### CLI
- Updated config commands

#### Portal API (30+ handlers updated)
- Improved handlers: agents, attack_surface, audit, auth, backups
- Enhanced: compliance, containers, darkweb, dashboard, database
- Updated: documents, enrollment, fim, grc, honeypots, hunting
- Refined: incidents, network, servers, settings, tenants
- Fixed: threat_intel, users, vulnerabilities, websocket
- Updated middleware and routes

#### Web Frontend
- Updated dashboard pages: backups, documents, grc, honeypots, hunting, network, settings, tenants
- Enhanced token-display component
- Updated API library

#### Crates
- **raksha-alert**: Updated engine and models
- **raksha-audit**: Improved store
- **raksha-grc**: Updated control_mapping, policy_manager, risk_engine
- **raksha-threat-intel**: Updated feeds

### Stats
- 60 files changed
- +3,737 insertions
- -1,450 deletions

---

## [1.0.1] - 2026-08-05

### Fixed
- **use-realtime.ts**: Fixed stale closure bug with `reconnectAttempts` by using `useRef`
- **use-realtime.ts**: Added refs for callbacks to prevent WebSocket recreation on render
- **use-realtime.ts**: Properly listed `useEffect` dependencies (removed `eslint-disable`)
- **use-realtime.ts**: Fixed memory leak from uncleaned reconnection timers
- **realtime-notifications.tsx**: Wrapped `handleEvent` in `useCallback` to stabilize identity

### Eliminated
- React warnings about missing hook dependencies
- Potential infinite WebSocket reconnection loops
- Stale state bugs during reconnection attempts
- Memory leaks from uncleaned timers

---

## [1.0.0] - 2026-08-04

### Added
- Initial stable release of Raksha Security Platform
- **Portal API** (Rust/Axum): Full REST API with JWT authentication
- **Web Dashboard** (Next.js): Real-time security monitoring interface
- **Agent** (Rust): Cross-platform security collector with mTLS enrollment
- **CLI** (Rust): Command-line management tool

### Features
- 🖥️ Server monitoring with real-time health metrics
- 🌐 Network scanning and vulnerability assessment
- 🗄️ Database security auditing (PostgreSQL, MySQL, MongoDB)
- 📋 Compliance engine (CIS, NIST, PCI-DSS, ISO 27001)
- 🤖 ML-powered threat detection (Isolation Forest, CNN, LSTM)
- 🌍 Threat intelligence with auto-syncing IOC feeds
- 📝 Immutable audit trail with SHA3-256 verification
- 🔔 Real-time alerts (WebSocket, Email, Telegram, Webhook)
- 👥 RBAC user management with MFA support
- 🕵️ Agent system with secure enrollment
- 📁 File integrity monitoring (FIM)
- 🍯 Honeypot/deception system
- 🌑 Dark web monitoring
- 📦 Container security scanning
- 🚨 Incident response with playbook engine
- 📊 GRC (Governance, Risk & Compliance) module

### Security
- mTLS for agent-to-portal communication
- Argon2id password hashing
- JWT with token rotation
- Rate limiting on all endpoints
- Input validation and sanitization
- RBAC enforcement at API gateway level

---

## [0.1.0] - 2026-07-24

### Added
- First public release
- Core platform architecture
- Full open-source under MIT license

---

## Links

- [v1.1.0 Release](https://github.com/dansiapa/raksha-security-platform/releases/tag/v1.1.0)
- [v1.0.1 Release](https://github.com/dansiapa/raksha-security-platform/releases/tag/v1.0.1)
- [v1.0.0 Release](https://github.com/dansiapa/raksha-security-platform/releases/tag/v-1.0-release)

---

Contact: dummymailrangga@gmail.com
Repository: https://github.com/dansiapa/raksha-security-platform
