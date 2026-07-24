"""
Feed Sync - Auto-fetch training data from threat intel sources

Periodically pulls data from:
- NVD (NIST) for vulnerability patterns
- CISA KEV for known exploits
- Abuse.ch for malware/C2 IOCs

This data is used to retrain the anomaly models automatically.
"""

from __future__ import annotations

import httpx
import structlog
from datetime import datetime

logger = structlog.get_logger()

CISA_KEV_URL = "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json"
NVD_API_URL = "https://services.nvd.nist.gov/rest/json/cves/2.0"
FEODO_URL = "https://feodotracker.abuse.ch/downloads/ipblocklist.json"


class FeedSync:
    """Synchronizes threat intel feeds for ML training data."""

    def __init__(self):
        self.client = httpx.AsyncClient(timeout=30.0, headers={"User-Agent": "Raksha-ML/0.1"})
        self.last_sync: dict[str, str] = {}

    async def sync_cisa_kev(self) -> list[dict]:
        """Fetch CISA Known Exploited Vulnerabilities."""
        try:
            resp = await self.client.get(CISA_KEV_URL)
            resp.raise_for_status()
            data = resp.json()
            vulns = data.get("vulnerabilities", [])
            self.last_sync["cisa_kev"] = datetime.utcnow().isoformat()
            logger.info("feed_synced", feed="cisa_kev", count=len(vulns))
            return vulns
        except Exception as e:
            logger.error("feed_sync_failed", feed="cisa_kev", error=str(e))
            return []

    async def sync_feodo(self) -> list[dict]:
        """Fetch Feodo Tracker C2 blocklist."""
        try:
            resp = await self.client.get(FEODO_URL)
            resp.raise_for_status()
            data = resp.json()
            entries = data if isinstance(data, list) else []
            self.last_sync["feodo"] = datetime.utcnow().isoformat()
            logger.info("feed_synced", feed="feodo", count=len(entries))
            return entries
        except Exception as e:
            logger.error("feed_sync_failed", feed="feodo", error=str(e))
            return []

    async def sync_all(self) -> dict[str, int]:
        """Sync all feeds and return counts."""
        results = {}
        kev = await self.sync_cisa_kev()
        results["cisa_kev"] = len(kev)
        feodo = await self.sync_feodo()
        results["feodo"] = len(feodo)
        return results

    async def close(self):
        await self.client.aclose()
