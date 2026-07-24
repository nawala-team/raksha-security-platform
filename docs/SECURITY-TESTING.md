# Security Testing Playbook

Comprehensive security testing guide for Raksha Security Platform. All tools referenced are free and open source.

## Table of Contents

- [Testing Philosophy](#testing-philosophy)
- [OWASP Top 10 Checklist](#owasp-top-10-checklist)
- [Rust-Specific Security Testing](#rust-specific-security-testing)
- [API Security Testing](#api-security-testing)
- [Container Security Testing](#container-security-testing)
- [Penetration Testing Playbook](#penetration-testing-playbook)
- [Fuzzing Strategy](#fuzzing-strategy)
- [CI Integration](#ci-integration)

## Testing Philosophy

1. **Shift left** — catch vulnerabilities in development, not production
2. **Automate everything** — manual testing does not scale
3. **Defense in depth** — multiple layers of checks catch different classes of bugs
4. **Assume breach** — test what happens when a component is compromised

## OWASP Top 10 Checklist (2021)

### A01:2021 — Broken Access Control

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| Vertical privilege escalation | Attempt admin actions with user token | curl / httpie | [ ] |
| Horizontal privilege escalation | Access other user's resources with valid token | curl / httpie | [ ] |
| Missing function-level access control | Call undocumented endpoints | feroxbuster | [ ] |
| CORS misconfiguration | Send cross-origin requests | curl | [ ] |
| Directory traversal | Path manipulation in file endpoints | manual | [ ] |
| JWT tampering | Modify claims, use `none` algorithm | jwt_tool | [ ] |
| IDOR (Insecure Direct Object Reference) | Enumerate resource IDs | manual | [ ] |

```bash
# Example: Test vertical privilege escalation
curl -H "Authorization: Bearer $USER_TOKEN" \
  -X DELETE http://localhost:8080/api/v1/admin/users/123
# Expected: 403 Forbidden
```

### A02:2021 — Cryptographic Failures

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| TLS configuration | Check cipher suites and protocol versions | testssl.sh | [ ] |
| Weak hashing algorithms | Audit code for MD5/SHA1 in security contexts | grep / semgrep | [ ] |
| Hardcoded secrets | Scan for credentials in source | gitleaks / trufflehog | [ ] |
| Insecure random number generation | Verify use of CSPRNG | code review | [ ] |
| Key management | Check key rotation, storage | manual audit | [ ] |

```bash
# Scan for hardcoded secrets
gitleaks detect --source . --report-format json --report-path leaks.json

# Check for weak crypto usage in Rust code
grep -rn "md5\|sha1\|Sha1\|Md5" --include="*.rs" .
```

### A03:2021 — Injection

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| SQL injection | Parameterized query verification | sqlmap / manual | [ ] |
| Command injection | Shell metacharacter fuzzing | manual | [ ] |
| LDAP injection | Special character input | manual | [ ] |
| Log injection | Newline/ANSI in log inputs | manual | [ ] |

```bash
# Test for command injection in API
curl -X POST http://localhost:8080/api/v1/scan \
  -H "Content-Type: application/json" \
  -d '{"target": "127.0.0.1; cat /etc/passwd"}'
# Expected: Input rejected or safely escaped
```

### A04:2021 — Insecure Design

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| Rate limiting | Burst request testing | vegeta / hey | [ ] |
| Business logic flaws | Manual scenario testing | manual | [ ] |
| Missing input validation | Boundary value testing | cargo-fuzz | [ ] |
| Threat model coverage | Review against threat model | manual | [ ] |

```bash
# Test rate limiting with vegeta
echo "POST http://localhost:8080/api/v1/auth/login" | \
  vegeta attack -duration=10s -rate=100 | vegeta report
```

### A05:2021 — Security Misconfiguration

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| Default credentials | Attempt default logins | manual | [ ] |
| Unnecessary services exposed | Port scanning | nmap | [ ] |
| Stack traces in error responses | Trigger errors via bad input | curl | [ ] |
| Security headers | Verify headers on responses | curl | [ ] |
| Debug endpoints in production | Check /debug, /metrics exposure | feroxbuster | [ ] |

```bash
# Check security headers
curl -sI http://localhost:8080/api/v1/health | grep -iE \
  "x-content-type|x-frame|strict-transport|content-security-policy"
```

### A06:2021 — Vulnerable and Outdated Components

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| Rust dependency audit | Check for known CVEs | cargo-audit | [ ] |
| License compliance | Check dependency licenses | cargo-deny | [ ] |
| Python dependency audit | Check for known CVEs | pip-audit / safety | [ ] |
| Node dependency audit | Check for known CVEs | npm audit | [ ] |
| Container base image CVEs | Scan container images | trivy | [ ] |

```bash
cargo audit
cargo deny check
pip-audit -r requirements.txt
trivy image raksha-security-platform:latest
```

### A07:2021 — Identification and Authentication Failures

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| Brute force protection | Rapid login attempts | hydra / manual | [ ] |
| Session fixation | Reuse session across auth boundary | manual | [ ] |
| Token expiration | Use expired JWT | curl | [ ] |
| Password policy enforcement | Submit weak passwords | manual | [ ] |

```bash
# Test JWT expiration enforcement
curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $EXPIRED_TOKEN" \
  http://localhost:8080/api/v1/protected
# Expected: 401
```

### A08:2021 — Software and Data Integrity Failures

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| CI/CD pipeline integrity | Review workflow permissions | manual | [ ] |
| Dependency pinning | Verify lockfiles are committed | manual | [ ] |
| Unsigned artifacts | Check release signatures | manual | [ ] |
| Deserialization attacks | Malformed payloads | cargo-fuzz | [ ] |

### A09:2021 — Security Logging and Monitoring Failures

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| Auth failures logged | Trigger failed logins, check logs | manual | [ ] |
| Sensitive data not in logs | Review log output for PII/secrets | grep | [ ] |
| Log injection prevention | Inject newlines/ANSI into fields | manual | [ ] |
| Audit trail completeness | Verify all mutations are logged | manual | [ ] |

### A10:2021 — Server-Side Request Forgery (SSRF)

| Test Case | Method | Tool | Status |
| --------- | ------ | ---- | ------ |
| Internal network access | Supply internal URLs as input | curl | [ ] |
| Cloud metadata access | Request 169.254.169.254 | curl | [ ] |
| DNS rebinding | Use rebinding domain | manual | [ ] |
| URL schema bypass | file://, gopher://, dict:// | curl | [ ] |

```bash
# Test SSRF against cloud metadata
curl -X POST http://localhost:8080/api/v1/fetch \
  -H "Content-Type: application/json" \
  -d '{"url": "http://169.254.169.254/latest/meta-data/"}'
# Expected: Request blocked
```


## Rust-Specific Security Testing

### Memory Safety

```bash
# Find all unsafe blocks
grep -rn "unsafe" --include="*.rs" . | grep -v "// SAFETY:"

# Run with address sanitizer (nightly required)
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test --target x86_64-unknown-linux-gnu

# Run Miri for undefined behavior detection
cargo +nightly miri test
```

### Integer Overflow

```bash
# Ensure release builds check overflow:
# In Cargo.toml: overflow-checks = true under [profile.release]
cargo test --release
```

### Clippy Security Lints

```bash
cargo clippy -- \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  -W clippy::panic \
  -W clippy::indexing_slicing \
  -W clippy::arithmetic_side_effects
```

## API Security Testing

### Authentication Tests

```bash
# No auth header
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/v1/protected
# Expected: 401

# Invalid token
curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer invalid.token.here" \
  http://localhost:8080/api/v1/protected
# Expected: 401
```

### Input Validation Tests

```bash
# Oversized payload
dd if=/dev/urandom bs=1M count=100 | \
  curl -X POST -H "Content-Type: application/octet-stream" \
  --data-binary @- http://localhost:8080/api/v1/upload
# Expected: 413 Payload Too Large

# Malformed JSON
curl -X POST -H "Content-Type: application/json" \
  -d '{invalid json' http://localhost:8080/api/v1/data
# Expected: 400 Bad Request
```

## Container Security Testing

```bash
# Scan image for vulnerabilities
trivy image --severity HIGH,CRITICAL raksha-security-platform:latest

# Check for running as root
docker inspect raksha-security-platform:latest | jq '.[0].Config.User'
# Expected: non-empty (non-root user)

# Verify read-only filesystem
docker run --rm raksha-security-platform:latest touch /test 2>&1
# Expected: Read-only file system error
```

## Penetration Testing Playbook

### Phase 1: Reconnaissance

```bash
# Service enumeration (on your own test instance)
nmap -sV -sC -p- localhost

# API endpoint discovery
feroxbuster -u http://localhost:8080 -w /usr/share/wordlists/dirb/common.txt
```

### Phase 2: Authentication Attacks

```bash
# JWT analysis — decode without verification
echo $TOKEN | cut -d. -f2 | base64 -d 2>/dev/null | jq .

# Test algorithm confusion (RS256 -> HS256)
# Use jwt_tool to craft token signed with HS256 using public key as secret
```

### Phase 3: Authorization Testing

```bash
# Test RBAC boundaries — as user role, attempt admin operations
for endpoint in users roles config audit; do
  echo "Testing /api/v1/admin/$endpoint"
  curl -s -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer $USER_TOKEN" \
    "http://localhost:8080/api/v1/admin/$endpoint"
done
```

### Phase 4: Reporting Template

```markdown
## Finding: [Title]
**Severity**: Critical / High / Medium / Low
**CVSS**: [score]
**Component**: [affected component]
### Description
[What the vulnerability is]
### Steps to Reproduce
1. [step]
### Impact
[What an attacker can achieve]
### Remediation
[Suggested fix]
```

## Fuzzing Strategy

Fuzz targets are in `tests/security/cargo-fuzz/`.

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run a fuzz target (5 minutes)
cd tests/security/cargo-fuzz
cargo fuzz run parse_input -- -max_total_time=300

# Run with address sanitizer
cargo fuzz run parse_input --sanitizer=address

# Minimize corpus after finding crashes
cargo fuzz cmin parse_input
```

### Fuzz Target Priority

| Target | Risk | Description |
| ------ | ---- | ----------- |
| Input parsers | Critical | External input deserialization |
| Config parsers | High | YAML/TOML/JSON config loading |
| Network protocol handlers | Critical | Raw bytes from network |
| Crypto operations | High | Key parsing, signature verification |
| Rule engine | Medium | Detection rule evaluation |

## CI Integration

Security scanning is automated via `.github/workflows/security-scan.yml`.

### Running Locally

```bash
cargo audit
cargo deny check
cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used
trivy image raksha-security-platform:latest
gitleaks detect --source .
```

## Tools Reference

All tools are free and open source:

| Tool | Purpose | License |
| ---- | ------- | ------- |
| cargo-audit | Rust dependency CVE scanning | MIT/Apache-2.0 |
| cargo-deny | Dependency license/advisory checking | MIT/Apache-2.0 |
| cargo-fuzz | Rust fuzzing via libFuzzer | MIT/Apache-2.0 |
| trivy | Container vulnerability scanning | Apache-2.0 |
| gitleaks | Secret detection in git repos | MIT |
| semgrep | Static analysis (SAST) | LGPL-2.1 |
| nmap | Network scanning | NPSL |
| feroxbuster | Endpoint brute forcing | MIT |
| jwt_tool | JWT analysis and attacks | GPLv3 |
| testssl.sh | TLS configuration testing | GPLv2 |
| vegeta | HTTP load testing | MIT |
| pip-audit | Python dependency auditing | Apache-2.0 |
| Miri | Rust undefined behavior detection | MIT/Apache-2.0 |

