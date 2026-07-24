# Security Policy

Raksha Security Platform takes security seriously. We appreciate the community's efforts in identifying and responsibly disclosing vulnerabilities.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

Only the latest release on the `main` branch receives security patches. Older versions are not backported unless a critical vulnerability affects production deployments.

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security vulnerabilities.**

Send vulnerability reports to: **dummymailrangga@gmail.com**

### What to Include

- Description of the vulnerability
- Steps to reproduce (proof of concept)
- Affected component(s) and version(s)
- Potential impact assessment
- Any suggested remediation (optional)

### Response Timeline

| Stage             | SLA                    |
| ----------------- | ---------------------- |
| Acknowledge       | 24 hours               |
| Assessment        | 72 hours               |
| Fix / Mitigate    | 7 days                 |
| Public disclosure | 90 days (coordinated)  |

We follow coordinated disclosure. We ask reporters to keep findings confidential until a fix is released or 90 days have elapsed, whichever comes first.

## Scope

### In Scope

- Raksha core engine (Rust crates)
- API gateway and authentication flows
- Cryptographic implementations and key management
- Input parsing and data validation
- Configuration handling and secrets management
- Container image supply chain
- CI/CD pipeline security
- Dependency vulnerabilities (direct dependencies)

### Out of Scope

- Vulnerabilities in third-party services we integrate with (report upstream)
- Social engineering attacks against maintainers
- Denial of service via resource exhaustion on self-hosted instances (unless amplification factor > 100x)
- Findings from automated scanners without a validated proof of concept
- Vulnerabilities requiring physical access to hardware
- Issues in development/test configurations not intended for production

## Severity Classification

We use CVSS v3.1 for scoring:

| Severity | CVSS Score  | Fix SLA  |
| -------- | ----------- | -------- |
| Critical | 9.0 - 10.0  | 24 hours |
| High     | 7.0 - 8.9   | 3 days   |
| Medium   | 4.0 - 6.9   | 7 days   |
| Low      | 0.1 - 3.9   | 30 days  |

## Safe Harbor

We consider security research conducted in good faith to be authorized. We will not pursue legal action against researchers who:

- Act in good faith and avoid privacy violations, data destruction, or service disruption
- Only interact with accounts they own or with explicit permission
- Report vulnerabilities promptly and do not exploit them beyond proof of concept
- Do not publicly disclose before coordinated disclosure timeline

## Hall of Fame

We recognize researchers who responsibly disclose valid vulnerabilities:

| Researcher      | Finding | Severity | Date |
| --------------- | ------- | -------- | ---- |
| *Be the first!* | —       | —        | —    |

See [docs/BUG-BOUNTY.md](docs/BUG-BOUNTY.md) for our community recognition program.

## Security Updates

Security advisories are published via:
- GitHub Security Advisories
- `CHANGELOG.md` entries tagged `[SECURITY]`
- Announcements in project communication channels

## Contact

- Security Team: dummymailrangga@gmail.com
- PGP key: Available upon request
- Response hours: UTC business hours (Mon-Fri), best effort on weekends for critical issues
