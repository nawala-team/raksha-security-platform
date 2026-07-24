"""CISA Known Exploited Vulnerabilities (KEV) feed."""

from typing import Any

import httpx
import structlog

logger = structlog.get_logger(__name__)

CISA_KEV_URL = "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json"


class CISAKEVFeed:
    """Fetches CISA Known Exploited Vulnerabilities catalog."""

    async def fetch(self) -> list[dict[str, Any]]:
        """Fetch the full KEV catalog."""
        async with httpx.AsyncClient(timeout=30.0) as client:
            try:
                response = await client.get(CISA_KEV_URL)
                response.raise_for_status()
                data = response.json()
                return self._parse_kev(data)
            except Exception as e:
                logger.error("cisa_kev_fetch_failed", error=str(e))
                return []

    def _parse_kev(self, data: dict[str, Any]) -> list[dict[str, Any]]:
        """Parse KEV catalog into standardized records."""
        vulnerabilities = []
        for vuln in data.get("vulnerabilities", []):
            vulnerabilities.append({
                "cve_id": vuln.get("cveID", ""),
                "vendor": vuln.get("vendorProject", ""),
                "product": vuln.get("product", ""),
                "name": vuln.get("vulnerabilityName", ""),
                "description": vuln.get("shortDescription", ""),
                "date_added": vuln.get("dateAdded", ""),
                "due_date": vuln.get("dueDate", ""),
                "required_action": vuln.get("requiredAction", ""),
                "known_ransomware_use": vuln.get("knownRansomwareCampaignUse", "Unknown"),
                "notes": vuln.get("notes", ""),
            })
        logger.info("cisa_kev_parsed", count=len(vulnerabilities))
        return vulnerabilities

    async def get_overdue(self) -> list[dict[str, Any]]:
        """Get KEVs that are past their remediation due date."""
        from datetime import datetime

        all_kevs = await self.fetch()
        today = datetime.utcnow().strftime("%Y-%m-%d")
        overdue = [k for k in all_kevs if k["due_date"] < today]
        logger.info("cisa_kev_overdue", count=len(overdue))
        return overdue

    async def get_ransomware_related(self) -> list[dict[str, Any]]:
        """Get KEVs known to be used in ransomware campaigns."""
        all_kevs = await self.fetch()
        return [k for k in all_kevs if k["known_ransomware_use"] == "Known"]
