"""CPE matching for vulnerability correlation."""

from __future__ import annotations

import re
import sqlite3
from difflib import SequenceMatcher
from typing import Any

import structlog

from .package_parser import Package, PackageSource

logger = structlog.get_logger()

# Mapping from package manager names to CPE vendor/product conventions
VENDOR_MAPPINGS: dict[str, dict[str, str]] = {
    'python': {
        'django': 'djangoproject:django',
        'flask': 'palletsprojects:flask',
        'requests': 'python-requests:requests',
        'numpy': 'numpy:numpy',
        'pandas': 'numfocus:pandas',
        'pillow': 'python:pillow',
        'cryptography': 'cryptography_project:cryptography',
        'urllib3': 'python:urllib3',
        'setuptools': 'python:setuptools',
    },
    'npm': {
        'express': 'expressjs:express',
        'lodash': 'lodash:lodash',
        'axios': 'axios:axios',
        'react': 'facebook:react',
        'jquery': 'jquery:jquery',
        'angular': 'google:angular',
    },
}

# Source to CPE part mapping
SOURCE_TO_PART = {
    PackageSource.DPKG: 'a',
    PackageSource.RPM: 'a',
    PackageSource.PIP: 'a',
    PackageSource.NPM: 'a',
    PackageSource.CARGO: 'a',
    PackageSource.APK: 'a',
}


class CPEMatcher:
    """Match packages against CPE dictionary and find vulnerabilities."""

    def __init__(self, db_path: str = './data/nvd_cache.db'):
        self.db_path = db_path
        self._vendor_cache: dict[str, str] = {}

    def package_to_cpe(self, pkg: Package) -> str:
        """Convert a Package to a CPE 2.3 URI string."""
        part = SOURCE_TO_PART.get(pkg.source, 'a')
        vendor, product = self._resolve_vendor_product(pkg)
        version = self._normalize_version(pkg.version)
        return f'cpe:2.3:{part}:{vendor}:{product}:{version}:*:*:*:*:*:*:*'

    def _resolve_vendor_product(self, pkg: Package) -> tuple[str, str]:
        """Resolve vendor and product name for a package."""
        ecosystem = self._source_to_ecosystem(pkg.source)
        mappings = VENDOR_MAPPINGS.get(ecosystem, {})

        name_lower = pkg.name.lower().replace('_', '-')
        if name_lower in mappings:
            parts = mappings[name_lower].split(':')
            return parts[0], parts[1]

        # Default: use package name as both vendor and product
        sanitized = re.sub(r'[^a-z0-9._-]', '', name_lower)
        return sanitized, sanitized
    def _source_to_ecosystem(self, source: PackageSource) -> str:
        mapping = {
            PackageSource.PIP: 'python',
            PackageSource.NPM: 'npm',
            PackageSource.CARGO: 'cargo',
            PackageSource.DPKG: 'linux',
            PackageSource.RPM: 'linux',
            PackageSource.APK: 'linux',
        }
        return mapping.get(source, 'unknown')

    def _normalize_version(self, version: str) -> str:
        """Normalize version string for CPE format."""
        version = version.strip()
        # Remove epoch prefix (e.g., '1:2.3.4')
        if ':' in version:
            version = version.split(':', 1)[1]
        # Remove Debian/Ubuntu revision suffix
        if '-' in version and not version[0].isalpha():
            version = version.split('-')[0]
        # Remove trailing metadata
        version = re.sub(r'[+~].*$', '', version)
        return version

    def find_vulnerabilities(self, pkg: Package) -> list[dict[str, Any]]:
        """Find CVEs matching a package by CPE lookup."""
        cpe_str = self.package_to_cpe(pkg)
        vendor, product = self._resolve_vendor_product(pkg)
        version = self._normalize_version(pkg.version)

        results: list[dict[str, Any]] = []
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.row_factory = sqlite3.Row
                # Exact CPE prefix match
                rows = conn.execute(
                    '''SELECT ci.*, c.cve_id, c.description, c.cvss_v31_score,
                              c.cvss_v31_vector, c.cvss_v31_severity
                       FROM cpe_index ci
                       INNER JOIN cves c ON ci.cve_id = c.cve_id
                       WHERE ci.cpe_uri LIKE ?
                       ORDER BY c.cvss_v31_score DESC''',
                    (f'cpe:2.3:a:{vendor}:{product}:%',)
                ).fetchall()

                for row in rows:
                    row_dict = dict(row)
                    if self._version_in_range(version, row_dict):
                        results.append({
                            'cve_id': row_dict['cve_id'],
                            'description': row_dict.get('description', ''),
                            'cvss_score': row_dict.get('cvss_v31_score'),
                            'cvss_vector': row_dict.get('cvss_v31_vector'),
                            'severity': row_dict.get('cvss_v31_severity'),
                            'matched_cpe': row_dict.get('cpe_uri', ''),
                            'package_cpe': cpe_str,
                        })
        except sqlite3.OperationalError as e:
            logger.warning('cpe_lookup_failed', error=str(e), package=pkg.name)

        return results
    def _version_in_range(self, version: str, cpe_row: dict[str, Any]) -> bool:
        """Check if a version falls within the CPE version range."""
        v_start = cpe_row.get('version_start')
        v_start_type = cpe_row.get('version_start_type')
        v_end = cpe_row.get('version_end')
        v_end_type = cpe_row.get('version_end_type')

        # If no range specified, check if CPE URI contains exact version
        if not v_start and not v_end:
            cpe_uri = cpe_row.get('cpe_uri', '')
            parts = cpe_uri.split(':')
            if len(parts) >= 6:
                cpe_version = parts[5]
                if cpe_version == '*' or cpe_version == '-':
                    return True
                return self._compare_versions(version, cpe_version) == 0
            return True

        # Check start bound
        if v_start:
            cmp = self._compare_versions(version, v_start)
            if v_start_type == 'including' and cmp < 0:
                return False
            if v_start_type == 'excluding' and cmp <= 0:
                return False

        # Check end bound
        if v_end:
            cmp = self._compare_versions(version, v_end)
            if v_end_type == 'including' and cmp > 0:
                return False
            if v_end_type == 'excluding' and cmp >= 0:
                return False

        return True

    @staticmethod
    def _compare_versions(v1: str, v2: str) -> int:
        """Compare two version strings. Returns -1, 0, or 1."""
        def normalize(v: str) -> list[int]:
            parts = []
            for segment in re.split(r'[._-]', v):
                match = re.match(r'^(\d+)', segment)
                if match:
                    parts.append(int(match.group(1)))
            return parts

        n1, n2 = normalize(v1), normalize(v2)
        # Pad shorter list
        max_len = max(len(n1), len(n2))
        n1.extend([0] * (max_len - len(n1)))
        n2.extend([0] * (max_len - len(n2)))

        for a, b in zip(n1, n2):
            if a < b:
                return -1
            if a > b:
                return 1
        return 0

    def fuzzy_match_package(self, pkg_name: str, threshold: float = 0.8) -> list[str]:
        """Find CPE products that fuzzy-match a package name."""
        candidates: list[str] = []
        sanitized = pkg_name.lower().replace('_', '-')
        try:
            with sqlite3.connect(self.db_path) as conn:
                rows = conn.execute(
                    'SELECT DISTINCT cpe_uri FROM cpe_index LIMIT 50000'
                ).fetchall()
                for (cpe_uri,) in rows:
                    parts = cpe_uri.split(':')
                    if len(parts) >= 5:
                        product = parts[4]
                        ratio = SequenceMatcher(None, sanitized, product).ratio()
                        if ratio >= threshold:
                            candidates.append(cpe_uri)
        except sqlite3.OperationalError:
            pass
        return candidates

    def scan_packages(self, packages: list[Package]) -> list[dict[str, Any]]:
        """Scan a list of packages and return all found vulnerabilities."""
        all_vulns: list[dict[str, Any]] = []
        for pkg in packages:
            vulns = self.find_vulnerabilities(pkg)
            for vuln in vulns:
                vuln['package_name'] = pkg.name
                vuln['package_version'] = pkg.version
                vuln['package_source'] = pkg.source.value
            all_vulns.extend(vulns)
        logger.info('cpe_scan_complete', packages=len(packages), vulns_found=len(all_vulns))
        return all_vulns