# Branching Strategy

Raksha follows a **Git Flow** branching model for structured releases and parallel development.

## Branch Structure

```
main                          ← Production-ready code (protected)
├── develop                   ← Integration branch for next release
│   ├── feature/frontend-dashboard   ← UI/UX development
│   ├── feature/backend-api          ← Rust API endpoints
│   ├── feature/ml-engine            ← ML models and threat intel
│   ├── feature/agent-system         ← Cross-platform agent
│   └── feature/database-monitoring  ← DB guard module
├── release/v1.0.0            ← Release candidate (QA + stabilization)
└── hotfix/security-patches   ← Critical production fixes
```

## Branch Rules

| Branch | Purpose | Merge Target | Protection |
|--------|---------|-------------|------------|
| `main` | Stable production code | — | Protected: requires PR + 1 approval + passing CI |
| `develop` | Integration of features | `main` (via release branch) | Protected: requires PR + passing CI |
| `feature/*` | New features and modules | `develop` | No direct push to develop; PR required |
| `release/*` | Release stabilization | `main` + `develop` | Freeze: only bugfixes allowed |
| `hotfix/*` | Critical production fixes | `main` + `develop` | Fast-track: 1 approval required |

## Workflow

### Feature Development

```bash
# Create feature branch from develop
git checkout develop
git checkout -b feature/my-feature

# Work on feature...
git commit -m "feat: implement new feature"

# Push and create PR to develop
git push -u origin feature/my-feature
# Create Pull Request → develop
```

### Release Process

```bash
# Create release branch from develop
git checkout develop
git checkout -b release/v1.1.0

# QA, bugfixes only on this branch
git commit -m "fix: resolve issue found in QA"

# When ready:
# 1. Merge release → main (tag v1.1.0)
# 2. Merge release → develop (backport fixes)
```

### Hotfix Process

```bash
# Create hotfix from main
git checkout main
git checkout -b hotfix/critical-vuln

# Apply fix
git commit -m "fix: patch critical vulnerability CVE-2024-XXXX"

# Merge to both main AND develop
# 1. PR → main (deploy immediately)
# 2. PR → develop (include in next release)
```

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore
Scopes: portal, web, ml, agent, docs, ci, helm
```

Examples:
```
feat(portal): add agent heartbeat endpoint
fix(ml): resolve memory leak in anomaly detector
docs(agent): update enrollment flow diagram
ci: add helm chart linting to pipeline
```

## Branch Protection (main)

The `main` branch enforces:

- ✅ Pull request required (no direct push)
- ✅ At least 1 approving review
- ✅ All CI status checks must pass
- ✅ Branch must be up to date before merge
- ✅ Dismiss stale reviews on new commits
- ✅ Enforce rules for administrators

## Team Assignments

| Branch | Team |
|--------|------|
| `feature/frontend-dashboard` | Front End Developer |
| `feature/backend-api` | Back End Developer |
| `feature/ml-engine` | FullStack Developer, Security (Blue Team) |
| `feature/agent-system` | Infrastructure, Security (Red Team) |
| `feature/database-monitoring` | DBA |
| `release/*` | QA Automation, Project Manager |
| `hotfix/*` | Bug Hunter, Security (Blue Team) |

---

*Part of the Nawala Ecosystem* 🔱
