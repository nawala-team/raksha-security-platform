"""NVD/CVE feed fetcher for vulnerability intelligence."""

import asyncio
from typing import Any
from datetime import datetime, timedelta

import httpx
import structlog

logger = structlog.get_logger(__name__)

NVD_API_BASE = "https://services.nvd.nist.gov/rest/json/cves/2.0"


class NVDFeed:
    """Fetches and parses NIST National Vulnerability Database CVE data."""

    def __init__(self, api_key: str | None = None):
        self.api_key = api_key
        self.headers = {}
        if api_key:
            self.headers["apiKey"] = api_key

    async def fetch_recent(self, days: int = 7, max_results: int = 500) -> list[dict[str, Any]]:
        """Fetch CVEs published/modified in the last N days."""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        params = {
            "pubStartDate": start_date.strftime("%Y-%m-%dT00:00:00.000"),
            "pubEndDate": end_date.strftime("%Y-%m-%dT23:59:59.999"),
            "resultsPerPage": min(max_results, 2000),
        }

        async with httpx.AsyncClient(timeout=60.0) as client:
            try:
                response = await client.get(NVD_API_BASE, params=params, headers=self.headers)
                response.raise_for_status()
                data = response.json()
                return self._parse_cves(data)
            except httpx.HTTPStatusError as e:
                logger.error("nvd_fetch_failed", status=e.response.status_code)
                return []
            except Exception as e:
                logger.error("nvd_fetch_error", error=str(e))
                return []

    async def fetch_by_keyword(self, keyword: str) -> list[dict[str, Any]]:
        """Search CVEs by keyword."""
        params = {"keywordSearch": keyword, "resultsPerPage": 100}

        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.get(NVD_API_BASE, params=params, headers=self.headers)
            response.raise_for_status()
            return self._parse_cves(response.json())

    async def fetch_by_cpe(self, cpe_name: str) -> list[dict[str, Any]]:
        """Fetch CVEs for a specific CPE (Common Platform Enumeration)."""
        params = {"cpeName": cpe_name, "resultsPerPage": 100}

        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.get(NVD_API_BASE, params=params, headers=self.headers)
            response.raise_for_status()
            return self._parse_cves(response.json())

    def _parse_cves(self, data: dict[str, Any]) -> list[dict[str, Any]]:
        """Parse NVD API response into standardized CVE records."""
        cves = []
        for item in data.get("vulnerabilities", []):
            cve_data = item.get("cve", {})
            metrics = cve_data.get("metrics", {})

            # Extract CVSS scores
            cvss_v31 = None
            if "cvssMetricV31" in metrics:
                cvss_v31 = metrics["cvssMetricV31"][0]["cvssData"]

            # Extract descriptions
            descriptions = cve_data.get("descriptions", [])
            desc_en = next((d["value"] for d in descriptions if d["lang"] == "en"), "")

            cves.append({
                "cve_id": cve_data.get("id", ""),
                "description": desc_en,
                "published": cve_data.get("published", ""),
                "last_modified": cve_data.get("lastModified", ""),
                "cvss_score": cvss_v31.get("baseScore") if cvss_v31 else None,
                "cvss_severity": cvss_v31.get("baseSeverity") if cvss_v31 else None,
                "cvss_vector": cvss_v31.get("vectorString") if cvss_v31 else None,
                "weaknesses": [
                    w["description"][0]["value"]
                    for w in cve_data.get("weaknesses", [])
                    if w.get("description")
                ],
                "references": [
                    ref["url"] for ref in cve_data.get("references", [])
                ],
            })

        logger.info("nvd_cves_parsed", count=len(cves))
        return cves
