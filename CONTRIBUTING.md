# 🔱 Contributing to Raksha Security Platform

Thank you for your interest in contributing to Raksha! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

1. **Fork** the repository on GitHub
2. **Clone** your fork locally
3. **Create** a new branch for your feature or fix
4. **Commit** your changes with clear messages
5. **Push** to your fork
6. **Submit** a pull request

## Development Setup

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs))
- Node.js 20+ (install via [nvm](https://github.com/nvm-sh/nvm))
- PostgreSQL 16+
- Redis 7+
- Docker (optional, for integration tests)

### Setup

```bash
git clone https://github.com/<your-username>/raksha-security-platform.git
cd raksha-security-platform
git remote add upstream https://github.com/nawala-team/raksha-security-platform.git
npm install
cp .env.example .env
docker compose -f docker-compose.dev.yml up -d
cargo run --bin raksha-migrate
cargo run --bin raksha-server -- --dev
```

## Branch Naming

| Prefix | Purpose | Example |
|--------|---------|---------|
| `feat/` | New feature | `feat/network-topology-map` |
| `fix/` | Bug fix | `fix/auth-token-expiry` |
| `docs/` | Documentation | `docs/api-reference-update` |
| `refactor/` | Code refactoring | `refactor/scanner-module` |
| `test/` | Adding tests | `test/compliance-engine` |
| `perf/` | Performance improvement | `perf/query-optimization` |
| `ci/` | CI/CD changes | `ci/add-integration-tests` |
| `chore/` | Maintenance tasks | `chore/update-dependencies` |

Always branch from `main`:

```bash
git checkout main
git pull upstream main
git checkout -b feat/your-feature-name
```

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>
```

### Types

`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `ci`, `chore`

### Scopes

`server`, `network`, `database`, `compliance`, `ml`, `audit`, `api`, `ui`, `core`

### Examples

```
feat(network): add subnet discovery scanning
fix(compliance): correct PCI-DSS rule 6.2 validation
docs(api): update authentication endpoint examples
perf(ml): optimize anomaly detection batch processing
```

## Pull Request Process

1. Update your branch with the latest `main` via rebase
2. Ensure all tests pass: `cargo test --all && npm run test && npm run lint`
3. Update documentation if your changes affect public APIs
4. Create the PR with a clear title, description, and linked issue(s)
5. At least one maintainer approval is required
6. CI checks must pass and code coverage must not decrease

## Code Style

### Rust
- Use `cargo fmt` and `cargo clippy` (zero warnings policy)
- Document public items with `///` doc comments
- Use `thiserror` for error types

### TypeScript (Dashboard)
- Use ESLint + Prettier with project configuration
- Prefer functional components with hooks
- Strict mode, avoid `any` type

### Python (ML Module)
- Follow PEP 8 with Black formatter
- Use type hints (Python 3.11+)
- Google-style docstrings

## Testing

```bash
cargo test --all                    # All Rust tests
cargo test -p raksha-network       # Specific module
cargo tarpaulin --all --out html   # With coverage
npm run test                       # TypeScript tests
cd ml && pytest --cov              # Python ML tests
```

Aim for 80%+ code coverage. Write unit tests for all public functions and integration tests for API endpoints.

---

Thank you for contributing to Raksha! 🔱

— **NAWALA TEAM** 🇮🇩
