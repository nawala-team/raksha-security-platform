"""Attack surface exposure scoring engine."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

import structlog

from .port_scanner import OpenPort
from .service_fingerprint import ServiceInfo

logger = structlog.get_logger()

# High-risk services that significantly increase exposure
HIGH_RISK_SERVICES: set[str] = {
    'telnet', 'ftp', 'rpcbind', 'netbios', 'smb', 'vnc', 'rdp',
    'mysql', 'postgresql', 'mssql', 'oracle', 'redis', 'mongodb',
    'memcached', 'elasticsearch', 'kubernetes-api', 'webmin',
}

# Medium-risk services
MEDIUM_RISK_SERVICES: set[str] = {
    'smtp', 'pop3', 'imap', 'ssh', 'http-proxy', 'nfs', 'pptp',
    'winrm', 'winrm-ssl', 'sap',
}

# Weights for scoring factors (total weights sum to ~100)
WEIGHTS: dict[str, float] = {
    'exposed_ports': 15.0,
    'high_risk_services': 30.0,
    'medium_risk_services': 10.0,
    'missing_security_headers': 15.0,
    'outdated_tls': 15.0,
    'version_exposure': 10.0,
    'expired_cert': 5.0,
}


@dataclass
class Finding:
    """A single exposure finding with severity and recommendation."""

    category: str
    severity: str  # 'critical', 'high', 'medium', 'low', 'info'
    title: str
    description: str
    recommendation: str
    score_impact: float = 0.0

    def to_dict(self) -> dict[str, Any]:
        return {
            'category': self.category,
            'severity': self.severity,
            'title': self.title,
            'description': self.description,
            'recommendation': self.recommendation,
            'score_impact': round(self.score_impact, 2),
        }


@dataclass
class ExposureReport:
    """Complete exposure assessment report."""

    score: float  # 0-100, higher = more exposed
    findings: list[Finding] = field(default_factory=list)
    recommendations: list[str] = field(default_factory=list)
    summary: str = ''

    def to_dict(self) -> dict[str, Any]:
        return {
            'score': round(self.score, 1),
            'summary': self.summary,
            'findings': [f.to_dict() for f in self.findings],
            'recommendations': self.recommendations,
            'total_findings': len(self.findings),
            'critical_findings': sum(1 for f in self.findings if f.severity == 'critical'),
            'high_findings': sum(1 for f in self.findings if f.severity == 'high'),
            'medium_findings': sum(1 for f in self.findings if f.severity == 'medium'),
            'low_findings': sum(1 for f in self.findings if f.severity == 'low'),
        }


class ExposureScorer:
    """Calculate attack surface exposure score based on scan results."""

    def score(
        self,
        open_ports: list[OpenPort],
        service_infos: list[ServiceInfo] | None = None,
    ) -> ExposureReport:
        """Calculate exposure score from scan results."""
        findings: list[Finding] = []
        total_score = 0.0

        # Factor 1: Number of exposed ports
        port_score = self._score_exposed_ports(open_ports, findings)
        total_score += port_score

        # Factor 2: High-risk services
        high_risk_score = self._score_high_risk_services(open_ports, findings)
        total_score += high_risk_score

        # Factor 3: Medium-risk services
        med_risk_score = self._score_medium_risk_services(open_ports, findings)
        total_score += med_risk_score

        if service_infos:
            # Factor 4: Missing security headers
            header_score = self._score_security_headers(service_infos, findings)
            total_score += header_score

            # Factor 5: TLS issues
            tls_score = self._score_tls(service_infos, findings)
            total_score += tls_score

            # Factor 6: Version exposure
            version_score = self._score_version_exposure(service_infos, findings)
            total_score += version_score

        # Cap at 100
        total_score = min(100.0, max(0.0, total_score))

        # Generate recommendations
        recommendations = self._generate_recommendations(findings)

        # Summary
        if total_score >= 75:
            summary = 'Critical exposure level. Immediate remediation required.'
        elif total_score >= 50:
            summary = 'High exposure level. Multiple security issues need attention.'
        elif total_score >= 25:
            summary = 'Moderate exposure level. Some improvements recommended.'
        else:
            summary = 'Low exposure level. Attack surface is well-managed.'

        report = ExposureReport(
            score=total_score,
            findings=findings,
            recommendations=recommendations,
            summary=summary,
        )
        logger.info('asm_exposure_score_calculated', score=total_score, findings=len(findings))
        return report

    def _score_exposed_ports(self, ports: list[OpenPort], findings: list[Finding]) -> float:
        """Score based on number of open ports."""
        count = len(ports)
        if count == 0:
            return 0.0

        # Logarithmic scaling: more ports = diminishing additional risk
        # 1-5 ports: low, 6-15: medium, 16+: high
        if count <= 5:
            ratio = count / 5.0 * 0.3
        elif count <= 15:
            ratio = 0.3 + (count - 5) / 10.0 * 0.4
        else:
            ratio = min(1.0, 0.7 + (count - 15) / 20.0 * 0.3)

        score = ratio * WEIGHTS['exposed_ports']

        severity = 'high' if count > 15 else 'medium' if count > 5 else 'low'
        findings.append(Finding(
            category='exposed_ports',
            severity=severity,
            title=f'{count} open port(s) detected',
            description=f'Found {count} open TCP ports on the target.',
            recommendation='Review all exposed ports and close unnecessary services.',
            score_impact=score,
        ))
        return score

    def _score_high_risk_services(self, ports: list[OpenPort], findings: list[Finding]) -> float:
        """Score based on high-risk services exposed."""
        high_risk_found = [p for p in ports if p.service in HIGH_RISK_SERVICES]
        if not high_risk_found:
            return 0.0

        # Each high-risk service contributes proportionally
        count = len(high_risk_found)
        ratio = min(1.0, count / 3.0)  # 3+ high-risk services = max score
        score = ratio * WEIGHTS['high_risk_services']

        for port in high_risk_found:
            findings.append(Finding(
                category='high_risk_service',
                severity='critical' if port.service in ('telnet', 'ftp', 'vnc') else 'high',
                title=f'High-risk service: {port.service} on port {port.port}',
                description=(
                    f'{port.service} is exposed on {port.ip}:{port.port}. '
                    'This service is commonly targeted by attackers.'
                ),
                recommendation=f'Restrict access to {port.service} via firewall rules or VPN.',
                score_impact=score / count,
            ))
        return score

    def _score_medium_risk_services(self, ports: list[OpenPort], findings: list[Finding]) -> float:
        """Score based on medium-risk services exposed."""
        med_risk_found = [p for p in ports if p.service in MEDIUM_RISK_SERVICES]
        if not med_risk_found:
            return 0.0

        count = len(med_risk_found)
        ratio = min(1.0, count / 5.0)
        score = ratio * WEIGHTS['medium_risk_services']

        for port in med_risk_found:
            findings.append(Finding(
                category='medium_risk_service',
                severity='medium',
                title=f'Service exposed: {port.service} on port {port.port}',
                description=f'{port.service} is accessible on {port.ip}:{port.port}.',
                recommendation=f'Ensure {port.service} is properly secured and access-controlled.',
                score_impact=score / count,
            ))
        return score

    def _score_security_headers(
        self, services: list[ServiceInfo], findings: list[Finding]
    ) -> float:
        """Score based on missing security headers."""
        total_missing = 0
        total_checked = 0

        for svc in services:
            if svc.security_headers_missing:
                total_missing += len(svc.security_headers_missing)
            total_checked += len(svc.security_headers_present) + len(svc.security_headers_missing)

        if total_checked == 0:
            return 0.0

        ratio = min(1.0, total_missing / max(1, total_checked))
        score = ratio * WEIGHTS['missing_security_headers']

        if total_missing > 0:
            findings.append(Finding(
                category='security_headers',
                severity='medium' if ratio > 0.5 else 'low',
                title=f'{total_missing} missing security header(s)',
                description='HTTP security headers are missing from web services.',
                recommendation=(
                    'Add headers: Strict-Transport-Security, Content-Security-Policy, '
                    'X-Content-Type-Options, X-Frame-Options, Referrer-Policy.'
                ),
                score_impact=score,
            ))
        return score

    def _score_tls(self, services: list[ServiceInfo], findings: list[Finding]) -> float:
        """Score based on TLS certificate issues."""
        score = 0.0

        for svc in services:
            if not svc.tls_info:
                continue

            if svc.tls_info.is_expired:
                impact = WEIGHTS['expired_cert']
                score += impact
                findings.append(Finding(
                    category='tls',
                    severity='critical',
                    title=f'Expired TLS certificate on {svc.host}:{svc.port}',
                    description='The TLS certificate has expired.',
                    recommendation='Renew the TLS certificate immediately.',
                    score_impact=impact,
                ))
            elif svc.tls_info.days_until_expiry < 30:
                impact = WEIGHTS['outdated_tls'] * 0.5
                score += impact
                findings.append(Finding(
                    category='tls',
                    severity='high',
                    title=f'TLS certificate expiring soon on {svc.host}:{svc.port}',
                    description=f'Certificate expires in {svc.tls_info.days_until_expiry} days.',
                    recommendation='Renew the TLS certificate before expiry.',
                    score_impact=impact,
                ))

        return min(score, WEIGHTS['outdated_tls'] + WEIGHTS['expired_cert'])

    def _score_version_exposure(
        self, services: list[ServiceInfo], findings: list[Finding]
    ) -> float:
        """Score based on version information leakage."""
        exposed_count = 0
        for svc in services:
            if svc.detected_version:
                exposed_count += 1

        if exposed_count == 0:
            return 0.0

        ratio = min(1.0, exposed_count / max(1, len(services)))
        score = ratio * WEIGHTS['version_exposure']

        findings.append(Finding(
            category='version_exposure',
            severity='medium' if exposed_count > 1 else 'low',
            title=f'Version information exposed on {exposed_count} service(s)',
            description='Server version headers reveal software versions to attackers.',
            recommendation='Remove or obfuscate version information from server headers.',
            score_impact=score,
        ))
        return score

    def _generate_recommendations(self, findings: list[Finding]) -> list[str]:
        """Generate prioritized recommendations from findings."""
        recs: list[str] = []
        seen: set[str] = set()

        severity_order = {'critical': 0, 'high': 1, 'medium': 2, 'low': 3, 'info': 4}
        sorted_findings = sorted(findings, key=lambda f: severity_order.get(f.severity, 4))

        for finding in sorted_findings:
            if finding.recommendation not in seen:
                seen.add(finding.recommendation)
                recs.append(f'[{finding.severity.upper()}] {finding.recommendation}')

        return recs

