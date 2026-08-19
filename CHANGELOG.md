# Changelog

All notable changes to the Raksha Security Platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.3.1] - 2026-08-08

### Security
- **Fixed**: All 35 vulnerabilities resolved (npm + Cargo)
- **Updated**: jsonwebtoken 9.3 → 11.0
- **Updated**: idna → 1.1.1
- **Updated**: validator 0.18 → 0.20
- **Updated**: npm dependencies to latest secure versions

### Added
- GitHub Actions workflow for automatic dependency fixes

### Changed
- All repository URLs updated to nawala-team organization

---

## [1.3.0] - 2026-08-08

### Added

#### Installation Wizard
- **New**: Web-based installation wizard (`apps/installer`)
- Setup wizard with step-by-step configuration
- Automatic database connection testing
- SuperAdmin account creation during setup
- Secure JWT secret auto-generation
- Password hashing with Argon2

### Security

#### Hardcoded Secrets Removal
- **config/default.toml**: Removed hardcoded JWT secret and database credentials
- **docker-compose.yml**: Changed to use environment variables only
- **scripts/install-portal.sh**: Removed hardcoded passwords
- **scripts/install-portal.ps1**: Removed hardcoded passwords
- **.env.example**: Added security warnings and placeholder instructions

#### Code Quality
- **raksha-alert/engine.rs**: Replaced runtime `unwrap()` with lazy static regex
- **raksha-grc/handlers.rs**: Fixed 8x `serde_json::to_value().unwrap()` calls
- Added `once_cell` dependency for thread-safe lazy initialization

### Changed
- Bumped version to 1.3.0 across all packages
- Updated README with Installation Wizard section

---

## [1.2.1] - 2026-08-05

### Fixed

#### Backend Handler Bug Fixes
- **network.rs**: Fixed missing fields in response (event_type, severity, direction, action, is_threat, occurred_at)
- **network.rs**: Fixed NULL timestamp handling with COALESCE for backward compatibility
- **hunting.rs**: Fixed `results_count` column mismatch with database schema using COALESCE
- **hunting.rs**: Added RQL syntax validation before saving queries
- **hunting.rs**: Added `query_source` validation (events, alerts, network, fim, logs)
- **darkweb.rs**: Aligned FindingResponse with database schema and UI interface
- **darkweb.rs**: Fixed field mapping (source → source_name/source_type, found_at → discovered_at)
- **darkweb.rs**: Added missing fields (excerpt_redacted, record_count, confidence, alert_id, incident_id)
- **grc.rs**: Fixed hardcoded `Uuid::parse_str().unwrap()` with safe fallback
- **backups.rs**: Fixed hardcoded tenant_id unwrap with safe fallback
- **documents.rs**: Fixed hardcoded tenant_id unwrap with safe fallback
- **enrollment.rs**: Fixed hardcoded tenant_id unwrap with safe fallback

#### Input Validation
- Added validation for network rule direction (inbound, outbound, both)
- Added validation for network rule action (allow, block, drop, reject, monitor, log)
- Added validation for hunting query source
- Added RQL syntax parsing validation before query save

#### Error Handling
- Improved error handling with `tracing::warn` logging instead of silent `unwrap_or`
- Better error messages for validation failures

### Database Migrations
- `20260805100000_fix_schema_mismatches.sql`: Fix tenant_status enum, hunting_runs.results_count default
- `20260805100001_fix_darkweb_honeypot_hunting.sql`: Add missing darkweb/honeypot/hunting columns
- `20260805100002_fix_network_events.sql`: Add network_events missing columns (event_type, severity, direction, action, is_threat, occurred_at)

### Security Verification
- [x] No SQL injection vulnerabilities (all queries use parameterized bindings)
- [x] No XSS vulnerabilities (React auto-escaping, no dangerouslySetInnerHTML)
- [x] No hardcoded Uuid::parse_str().unwrap() remaining
- [x] No dangerous .expect() calls in handlers
- [x] All UI interfaces match backend responses
- [x] All backend responses match database schema

### Stats
- 10 files changed
- +688 insertions
- -207 deletions
- 3 new migrations

---

## [1.2.0] - 2026-08-05

### Added

#### Database Guard - Extended Database Support
- **Oracle Database**: Full support with service_name, SID, and TNS alias configuration
- **MariaDB**: Native support with auto-detection
- **SQL Server**: Microsoft SQL Server monitoring support
- New API endpoint `/databases/types` for listing supported database types
- Auto-port detection for all 7 supported database types
- Database type validation on registration

#### Server Monitor - Extended OS Support
- **Enterprise Unix**: Oracle Solaris, IBM AIX, HP-UX, illumos
- **Container OS**: Alpine Linux, Flatcar, Bottlerocket, CoreOS, VMware Photon OS, Talos, RancherOS
- **BSD Variants**: NetBSD, DragonFly BSD (in addition to existing FreeBSD, OpenBSD)
- **Linux Distributions**: Enhanced detection for Ubuntu, Debian, RHEL, CentOS, Rocky, AlmaLinux, Fedora, SUSE, Amazon Linux, Oracle Linux, Arch, Gentoo
- New API endpoint `/servers/os-families` with category filtering
- New `os_families` reference table for UI dropdowns
- Color-coded OS display in server cards

### Changed
- Updated README.md with comprehensive database support list
- Enhanced database registration form with Oracle-specific fields (conditional UI)
- Improved server page with OS-specific color coding

### Database Migrations
- `20260805000001_add_oracle_database_support.sql`: Oracle columns for monitored_databases
- `20260805000002_add_extended_os_support.sql`: Extended os_type constraint + os_families table

### Supported Platforms

#### Databases (7 types)
| Database | Default Port | Status |
|----------|--------------|--------|
| PostgreSQL | 5432 | [x] |
| MySQL | 3306 | [x] |
| MongoDB | 27017 | [x] |
| Redis | 6379 | [x] |
| Oracle | 1521 | [NEW] |
| MariaDB | 3306 | [NEW] |
| SQL Server | 1433 | [NEW] |

#### Operating Systems (14 types)
| Category | OS Types |
|----------|----------|
| Linux | linux, ubuntu, debian, rhel, centos, rocky, alma, fedora, suse, amazon, oracle_linux, arch |
| Windows | windows, windows_server |
| macOS | macos |
| BSD | freebsd, openbsd, netbsd, dragonflybsd |
| Enterprise Unix | solaris, aix, hpux, illumos |
| Container OS | alpine, flatcar, bottlerocket, coreos, photon, talos, rancher |

### Stats
- 8 files changed
- +457 insertions
- -14 deletions

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
- * Server monitoring with real-time health metrics
- * Network scanning and vulnerability assessment
- * Database security auditing (PostgreSQL, MySQL, MongoDB)
- * Compliance engine (CIS, NIST, PCI-DSS, ISO 27001)
- * ML-powered threat detection (Isolation Forest, CNN, LSTM)
- * Threat intelligence with auto-syncing IOC feeds
- * Immutable audit trail with SHA3-256 verification
- * Real-time alerts (WebSocket, Email, Telegram, Webhook)
- * RBAC user management with MFA support
- * Agent system with secure enrollment
- * File integrity monitoring (FIM)
- * Honeypot/deception system
- * Dark web monitoring
- * Container security scanning
- * Incident response with playbook engine
- * GRC (Governance, Risk & Compliance) module

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
