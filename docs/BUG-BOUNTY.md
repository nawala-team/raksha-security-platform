# Bug Bounty Program

Raksha Security Platform runs a community-driven bug bounty program. As an open-source project, we offer recognition and Hall of Fame placement rather than monetary rewards.

## Program Overview

This is a **recognition-based** program for the open-source community. We believe in acknowledging the valuable work security researchers contribute to keeping Raksha and its users safe.

## Rewards

| Severity | Recognition |
| -------- | ----------- |
| Critical | Hall of Fame (gold tier) + named credit in release notes + contributor badge |
| High     | Hall of Fame (silver tier) + named credit in release notes |
| Medium   | Hall of Fame (bronze tier) + mention in changelog |
| Low      | Hall of Fame listing |

### Additional Recognition

- Your name/handle and link in our SECURITY.md Hall of Fame
- Contributor badge on your GitHub profile (via repository collaborator invitation)
- Public acknowledgment in the security advisory
- Letter of appreciation (on request, useful for CVs/portfolios)
- Swag (stickers, t-shirts) when project funding allows

## Scope

### Eligible Targets

| Target | Repository / Component |
| ------ | ---------------------- |
| Core Engine | `raksha-core` Rust crates |
| API Layer | REST/gRPC endpoints |
| Auth System | Authentication and authorization |
| Crypto | Key management, encryption, signing |
| Parser | Input parsing, deserialization |
| Config | Configuration and secrets handling |
| Container | Docker images, build pipeline |

### Vulnerability Classes We Care About

- Remote Code Execution (RCE)
- Authentication/Authorization bypass
- Cryptographic weaknesses (weak algorithms, improper key handling)
- Injection attacks (SQL, command, LDAP, etc.)
- Path traversal / arbitrary file access
- Server-Side Request Forgery (SSRF)
- Deserialization vulnerabilities
- Memory safety issues (buffer overflow, use-after-free, double-free)
- Information disclosure of sensitive data
- Privilege escalation
- Supply chain attacks (dependency confusion, typosquatting)

### Out of Scope

- Issues already reported or known (check existing advisories)
- Vulnerabilities in dependencies (report upstream; inform us for tracking)
- Rate limiting or brute force without demonstrated impact
- Missing security headers on non-sensitive endpoints
- Version disclosure
- Theoretical attacks without proof of concept
- Self-XSS or issues requiring unlikely user interaction chains
- Findings from automated scanners without manual validation

## Rules of Engagement

1. **Do not** access, modify, or delete data belonging to other users
2. **Do not** perform denial-of-service attacks
3. **Do not** use automated scanning tools against shared infrastructure
4. **Test locally** — clone the repo and run your own instance
5. **One report per vulnerability** — do not chain unrelated issues
6. **Provide a clear PoC** — steps to reproduce, expected vs actual behavior
7. **Allow remediation time** before any disclosure

## How to Report

1. Email: **dummymailrangga@gmail.com**
2. Subject: `[BUG-BOUNTY] <brief description>`
3. Include:
   - Affected component and version
   - Step-by-step reproduction instructions
   - Proof of concept (code, screenshots, logs)
   - Impact assessment
   - Suggested fix (optional but appreciated)

## Process

```
Reporter submits finding
        │
        ▼
  Acknowledged (24h)
        │
        ▼
  Triaged & assessed (72h)
        │
        ├── Valid → Fix developed (7d) → Credit assigned → Advisory published
        │
        └── Invalid/Duplicate → Reporter notified with explanation
```

## Eligibility

- First reporter of a unique vulnerability receives credit
- Duplicates: first complete report wins; partial reports may share credit
- You must not be a current maintainer or contributor who introduced the vulnerability
- You must follow responsible disclosure practices

## Hall of Fame

### Gold Tier (Critical)
<!-- Critical findings -->
*No entries yet — be the first!*

### Silver Tier (High)
<!-- High findings -->
*No entries yet*

### Bronze Tier (Medium)
<!-- Medium findings -->
*No entries yet*

### Contributors (Low)
<!-- Low findings -->
*No entries yet*

---

## FAQ

**Q: Is there a monetary reward?**
A: Not currently. As an open-source project, we offer recognition, public credit, and community standing. If the project receives security funding in the future, we will retroactively consider past reporters.

**Q: Can I blog about my finding?**
A: Yes, after the fix is released and the advisory is published. We ask for coordinated disclosure (90 days max).

**Q: What if I find something critical?**
A: Report immediately. Critical issues get prioritized and you'll receive the highest tier of recognition.

**Q: Do I need to provide a fix?**
A: No, but suggested fixes are appreciated and will earn additional acknowledgment.

---

*This program is subject to change. Researchers will be grandfathered under the terms active at the time of their report.*
