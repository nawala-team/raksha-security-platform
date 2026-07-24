"""Vulnerability scoring with CVSS v3.1, exploit prediction, and priority calculation."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Any

import structlog

logger = structlog.get_logger()


@dataclass
class ScoredVulnerability:
    """A vulnerability with computed priority score."""
    cve_id: str
    package_name: str
    package_version: str
    cvss_score: float
    severity: str
    epss_score: float
    priority_score: float
    priority_level: str
    description: str
    remediation: str
    cvss_vector: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            'cve_id': self.cve_id,
            'package_name': self.package_name,
            'package_version': self.package_version,
            'cvss_score': self.cvss_score,
            'severity': self.severity,
            'epss_score': round(self.epss_score, 4),
            'priority_score': round(self.priority_score, 2),
            'priority_level': self.priority_level,
            'description': self.description,
            'remediation': self.remediation,
            'cvss_vector': self.cvss_vector,
        }


class VulnerabilityScorer:
    """Score and prioritize vulnerabilities."""

    # CVSS v3.1 severity thresholds
    SEVERITY_MAP = {
        (0.0, 0.1): 'NONE',
        (0.1, 4.0): 'LOW',
        (4.0, 7.0): 'MEDIUM',
        (7.0, 9.0): 'HIGH',
        (9.0, 10.1): 'CRITICAL',
    }

    # Priority level thresholds (0-100 scale)
    PRIORITY_LEVELS = {
        (0, 25): 'LOW',
        (25, 50): 'MEDIUM',
        (50, 75): 'HIGH',
        (75, 101): 'CRITICAL',
    }

    def __init__(self, asset_value: float = 1.0):
        """
        Args:
            asset_value: Multiplier for asset importance (0.1 to 3.0).
                         1.0 = standard, 2.0 = important, 3.0 = critical asset.
        """
        self.asset_value = max(0.1, min(3.0, asset_value))
    def score_vulnerability(self, vuln: dict[str, Any]) -> ScoredVulnerability:
        """Score a single vulnerability with CVSS, EPSS-like prediction, and priority."""
        cvss_score = vuln.get('cvss_score') or 0.0
        severity = vuln.get('severity') or self._severity_from_score(cvss_score)
        cvss_vector = vuln.get('cvss_vector', '')

        # Compute EPSS-like exploit probability
        epss_score = self._predict_exploitability(cvss_score, cvss_vector, vuln)

        # Priority = CVSS_normalized * exploitability * asset_value
        cvss_normalized = cvss_score / 10.0
        priority_score = min(100.0, cvss_normalized * epss_score * self.asset_value * 100)

        priority_level = self._priority_level(priority_score)
        remediation = self._generate_remediation(vuln)

        return ScoredVulnerability(
            cve_id=vuln.get('cve_id', ''),
            package_name=vuln.get('package_name', ''),
            package_version=vuln.get('package_version', ''),
            cvss_score=cvss_score,
            severity=severity,
            epss_score=epss_score,
            priority_score=priority_score,
            priority_level=priority_level,
            description=vuln.get('description', ''),
            remediation=remediation,
            cvss_vector=cvss_vector,
        )

    def score_vulnerabilities(self, vulns: list[dict[str, Any]]) -> list[ScoredVulnerability]:
        """Score and sort a list of vulnerabilities by priority."""
        scored = [self.score_vulnerability(v) for v in vulns]
        scored.sort(key=lambda s: s.priority_score, reverse=True)
        return scored

    def _severity_from_score(self, score: float) -> str:
        for (low, high), label in self.SEVERITY_MAP.items():
            if low <= score < high:
                return label
        return 'NONE'

    def _priority_level(self, score: float) -> str:
        for (low, high), label in self.PRIORITY_LEVELS.items():
            if low <= score < high:
                return label
        return 'LOW'
    def _predict_exploitability(self, cvss_score: float, cvss_vector: str, vuln: dict[str, Any]) -> float:
        """Predict exploit probability (EPSS-like score 0.0 to 1.0).

        Uses heuristic based on:
        - Attack vector (network > adjacent > local > physical)
        - Attack complexity (low > high)
        - Privileges required (none > low > high)
        - User interaction (none > required)
        - CVSS score magnitude
        """
        base_prob = 0.1

        # Factor in CVSS score (higher = more likely to be exploited)
        base_prob += (cvss_score / 10.0) * 0.3

        if cvss_vector:
            # Attack Vector
            if 'AV:N' in cvss_vector:
                base_prob += 0.2
            elif 'AV:A' in cvss_vector:
                base_prob += 0.1
            elif 'AV:L' in cvss_vector:
                base_prob += 0.05

            # Attack Complexity
            if 'AC:L' in cvss_vector:
                base_prob += 0.15

            # Privileges Required
            if 'PR:N' in cvss_vector:
                base_prob += 0.15
            elif 'PR:L' in cvss_vector:
                base_prob += 0.05

            # User Interaction
            if 'UI:N' in cvss_vector:
                base_prob += 0.1

        # Cap at 1.0
        return min(1.0, base_prob)

    def _generate_remediation(self, vuln: dict[str, Any]) -> str:
        """Generate a remediation suggestion based on vulnerability data."""
        pkg_name = vuln.get('package_name', 'the package')
        severity = vuln.get('severity', 'UNKNOWN')
        cve_id = vuln.get('cve_id', '')

        if severity == 'CRITICAL':
            urgency = 'Immediate action required.'
            action = f'Upgrade {pkg_name} to the latest patched version immediately.'
        elif severity == 'HIGH':
            urgency = 'High priority.'
            action = f'Upgrade {pkg_name} to a patched version as soon as possible.'
        elif severity == 'MEDIUM':
            urgency = 'Schedule for next maintenance window.'
            action = f'Plan upgrade of {pkg_name} to a patched version.'
        else:
            urgency = 'Low priority.'
            action = f'Consider upgrading {pkg_name} during routine maintenance.'

        return f'{urgency} {action} Reference: https://nvd.nist.gov/vuln/detail/{cve_id}'