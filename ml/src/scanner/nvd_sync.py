"""NVD CVE database sync with local SQLite cache."""

from __future__ import annotations

import asyncio
import json
import sqlite3
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import httpx
import structlog

logger = structlog.get_logger()

NVD_API_BASE = 'https://services.nvd.nist.gov/rest/json/cves/2.0'
RATE_LIMIT_DELAY = 6.0  # NVD allows ~10 requests/minute without API key
RATE_LIMIT_WITH_KEY = 0.6  # ~100 requests/minute with API key


class NVDSync:
    """Synchronize NVD CVE data to a local SQLite cache."""

    def __init__(self, db_path: str = './data/nvd_cache.db', api_key: str | None = None):
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self.api_key = api_key
        self.rate_delay = RATE_LIMIT_WITH_KEY if api_key else RATE_LIMIT_DELAY
        self._last_request_time: float = 0
        self._init_db()

    def _init_db(self) -> None:
        with sqlite3.connect(str(self.db_path)) as conn:
            conn.executescript('''
                CREATE TABLE IF NOT EXISTS cves (
                    cve_id TEXT PRIMARY KEY,
                    source_identifier TEXT,
                    published TEXT,
                    last_modified TEXT,
                    vuln_status TEXT,
                    description TEXT,
                    cvss_v31_score REAL,
                    cvss_v31_vector TEXT,
                    cvss_v31_severity TEXT,
                    cpe_matches TEXT,
                    references_json TEXT,
                    raw_json TEXT,
                    synced_at TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_cves_last_modified ON cves(last_modified);
                CREATE INDEX IF NOT EXISTS idx_cves_cvss ON cves(cvss_v31_score);

                CREATE TABLE IF NOT EXISTS sync_metadata (
                    key TEXT PRIMARY KEY,
                    value TEXT
                );

                CREATE TABLE IF NOT EXISTS cpe_index (
                    cpe_uri TEXT,
                    cve_id TEXT,
                    vulnerable INTEGER DEFAULT 1,
                    version_start TEXT,
                    version_start_type TEXT,
                    version_end TEXT,
                    version_end_type TEXT,
                    PRIMARY KEY (cpe_uri, cve_id)
                );
                CREATE INDEX IF NOT EXISTS idx_cpe_uri ON cpe_index(cpe_uri);
            ''')

    def _rate_limit(self) -> None:
        elapsed = time.time() - self._last_request_time
        if elapsed < self.rate_delay:
            time.sleep(self.rate_delay - elapsed)
        self._last_request_time = time.time()
    def get_last_sync_time(self) -> str | None:
        with sqlite3.connect(str(self.db_path)) as conn:
            row = conn.execute(
                "SELECT value FROM sync_metadata WHERE key = 'last_modified_date'"
            ).fetchone()
            return row[0] if row else None

    def _set_last_sync_time(self, timestamp: str) -> None:
        with sqlite3.connect(str(self.db_path)) as conn:
            conn.execute(
                'INSERT OR REPLACE INTO sync_metadata (key, value) VALUES (?, ?)',
                ('last_modified_date', timestamp)
            )

    def sync(self, start_index: int = 0, results_per_page: int = 200) -> int:
        """Sync CVEs from NVD. Returns total number of CVEs synced."""
        total_synced = 0
        last_modified = self.get_last_sync_time()
        params: dict[str, Any] = {
            'resultsPerPage': results_per_page,
            'startIndex': start_index,
        }
        if last_modified:
            params['lastModStartDate'] = last_modified
            params['lastModEndDate'] = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.000')

        headers = {}
        if self.api_key:
            headers['apiKey'] = self.api_key

        total_results = None
        while True:
            self._rate_limit()
            try:
                with httpx.Client(timeout=30.0) as client:
                    resp = client.get(NVD_API_BASE, params=params, headers=headers)
                    resp.raise_for_status()
                    data = resp.json()
            except (httpx.HTTPError, json.JSONDecodeError) as e:
                logger.error('nvd_sync_request_failed', error=str(e), start_index=params['startIndex'])
                break

            if total_results is None:
                total_results = data.get('totalResults', 0)
                logger.info('nvd_sync_started', total_results=total_results)

            vulnerabilities = data.get('vulnerabilities', [])
            if not vulnerabilities:
                break

            self._store_cves(vulnerabilities)
            total_synced += len(vulnerabilities)
            logger.info('nvd_sync_progress', synced=total_synced, total=total_results)

            params['startIndex'] += results_per_page
            if params['startIndex'] >= total_results:
                break

        now = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.000')
        self._set_last_sync_time(now)
        logger.info('nvd_sync_complete', total_synced=total_synced)
        return total_synced
    def _store_cves(self, vulnerabilities: list[dict[str, Any]]) -> None:
        with sqlite3.connect(str(self.db_path)) as conn:
            for vuln_wrapper in vulnerabilities:
                cve = vuln_wrapper.get('cve', {})
                cve_id = cve.get('id', '')
                if not cve_id:
                    continue

                descriptions = cve.get('descriptions', [])
                desc_en = next(
                    (d['value'] for d in descriptions if d.get('lang') == 'en'), ''
                )

                metrics = cve.get('metrics', {})
                cvss_v31 = None
                cvss_data = metrics.get('cvssMetricV31', [])
                if cvss_data:
                    cvss_v31 = cvss_data[0].get('cvssData', {})

                score = cvss_v31.get('baseScore') if cvss_v31 else None
                vector = cvss_v31.get('vectorString') if cvss_v31 else None
                severity = cvss_v31.get('baseSeverity') if cvss_v31 else None

                # Extract CPE matches
                cpe_matches = []
                configurations = cve.get('configurations', [])
                for config in configurations:
                    for node in config.get('nodes', []):
                        for match in node.get('cpeMatch', []):
                            cpe_matches.append(match)
                            # Index CPE for fast lookup
                            conn.execute(
                                '''INSERT OR REPLACE INTO cpe_index
                                   (cpe_uri, cve_id, vulnerable, version_start,
                                    version_start_type, version_end, version_end_type)
                                   VALUES (?, ?, ?, ?, ?, ?, ?)''',
                                (
                                    match.get('criteria', ''),
                                    cve_id,
                                    1 if match.get('vulnerable', True) else 0,
                                    match.get('versionStartIncluding') or match.get('versionStartExcluding'),
                                    'including' if match.get('versionStartIncluding') else 'excluding' if match.get('versionStartExcluding') else None,
                                    match.get('versionEndIncluding') or match.get('versionEndExcluding'),
                                    'including' if match.get('versionEndIncluding') else 'excluding' if match.get('versionEndExcluding') else None,
                                )
                            )

                refs = cve.get('references', [])
                now = datetime.now(timezone.utc).isoformat()

                conn.execute(
                    '''INSERT OR REPLACE INTO cves
                       (cve_id, source_identifier, published, last_modified,
                        vuln_status, description, cvss_v31_score, cvss_v31_vector,
                        cvss_v31_severity, cpe_matches, references_json, raw_json, synced_at)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)''',
                    (
                        cve_id,
                        cve.get('sourceIdentifier'),
                        cve.get('published'),
                        cve.get('lastModified'),
                        cve.get('vulnStatus'),
                        desc_en,
                        score,
                        vector,
                        severity,
                        json.dumps(cpe_matches),
                        json.dumps(refs),
                        json.dumps(cve),
                        now,
                    )
                )
    def get_cve(self, cve_id: str) -> dict[str, Any] | None:
        """Retrieve a single CVE from the local cache."""
        with sqlite3.connect(str(self.db_path)) as conn:
            conn.row_factory = sqlite3.Row
            row = conn.execute('SELECT * FROM cves WHERE cve_id = ?', (cve_id,)).fetchone()
            if not row:
                return None
            return dict(row)

    def search_by_cpe(self, cpe_uri: str) -> list[dict[str, Any]]:
        """Find all CVEs matching a CPE URI pattern."""
        with sqlite3.connect(str(self.db_path)) as conn:
            conn.row_factory = sqlite3.Row
            rows = conn.execute(
                '''SELECT c.* FROM cves c
                   INNER JOIN cpe_index ci ON c.cve_id = ci.cve_id
                   WHERE ci.cpe_uri LIKE ?
                   ORDER BY c.cvss_v31_score DESC''',
                (cpe_uri + '%',)
            ).fetchall()
            return [dict(r) for r in rows]

    def get_stats(self) -> dict[str, Any]:
        """Get sync statistics."""
        with sqlite3.connect(str(self.db_path)) as conn:
            total = conn.execute('SELECT COUNT(*) FROM cves').fetchone()[0]
            critical = conn.execute('SELECT COUNT(*) FROM cves WHERE cvss_v31_score >= 9.0').fetchone()[0]
            high = conn.execute('SELECT COUNT(*) FROM cves WHERE cvss_v31_score >= 7.0 AND cvss_v31_score < 9.0').fetchone()[0]
            medium = conn.execute('SELECT COUNT(*) FROM cves WHERE cvss_v31_score >= 4.0 AND cvss_v31_score < 7.0').fetchone()[0]
            low = conn.execute('SELECT COUNT(*) FROM cves WHERE cvss_v31_score > 0 AND cvss_v31_score < 4.0').fetchone()[0]
            last_sync = self.get_last_sync_time()
            return {
                'total_cves': total,
                'critical': critical,
                'high': high,
                'medium': medium,
                'low': low,
                'last_sync': last_sync,
            }