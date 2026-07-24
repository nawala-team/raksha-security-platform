"""Abuse.ch feed integrations (URLhaus, MalwareBazaar, Feodo Tracker)."""

from typing import Any

import httpx
import structlog

logger = structlog.get_logger(__name__)

URLHAUS_API = "https://urlhaus-api.abuse.ch/v1"
MALWAREBAZAAR_API = "https://mb-api.abuse.ch/api/v1"
FEODO_BLOCKLIST = "https://feodotracker.abuse.ch/downloads/ipblocklist.json"


class AbuseCHFeed:
    """Fetches threat intelligence from Abuse.ch services."""

    async def fetch_urlhaus_recent(self, limit: int = 1000) -> list[dict[str, Any]]:
        """Fetch recent malicious URLs from URLhaus."""
        async with httpx.AsyncClient(timeout=30.0) as client:
            try:
                response = await client.post(
                    f"{URLHAUS_API}/urls/recent/",
                    data={"limit": str(limit)},
                )
                response.raise_for_status()
                data = response.json()
                return self._parse_urlhaus(data)
            except Exception as e:
                logger.error("urlhaus_fetch_failed", error=str(e))
                return []

    async def fetch_urlhaus_by_tag(self, tag: str) -> list[dict[str, Any]]:
        """Fetch URLs by tag (e.g., 'emotet', 'qakbot')."""
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{URLHAUS_API}/tag/", data={"tag": tag}
            )
            response.raise_for_status()
            return self._parse_urlhaus(response.json())

    async def fetch_malwarebazaar_recent(self, limit: int = 100) -> list[dict[str, Any]]:
        """Fetch recent malware samples from MalwareBazaar."""
        async with httpx.AsyncClient(timeout=30.0) as client:
            try:
                response = await client.post(
                    f"{MALWAREBAZAAR_API}/",
                    data={"query": "get_recent", "selector": str(limit)},
                )
                response.raise_for_status()
                data = response.json()
                return self._parse_malwarebazaar(data)
            except Exception as e:
                logger.error("malwarebazaar_fetch_failed", error=str(e))
                return []

    async def fetch_malwarebazaar_by_signature(self, signature: str) -> list[dict[str, Any]]:
        """Fetch samples by malware signature name."""
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{MALWAREBAZAAR_API}/",
                data={"query": "get_siginfo", "signature": signature, "limit": "50"},
            )
            response.raise_for_status()
            return self._parse_malwarebazaar(response.json())

    async def fetch_feodo_blocklist(self) -> list[dict[str, Any]]:
        """Fetch Feodo Tracker C2 IP blocklist."""
        async with httpx.AsyncClient(timeout=30.0) as client:
            try:
                response = await client.get(FEODO_BLOCKLIST)
                response.raise_for_status()
                data = response.json()
                return self._parse_feodo(data)
            except Exception as e:
                logger.error("feodo_fetch_failed", error=str(e))
                return []

    def _parse_urlhaus(self, data: dict[str, Any]) -> list[dict[str, Any]]:
        urls = []
        for entry in data.get("urls", []):
            urls.append({
                "url": entry.get("url", ""),
                "status": entry.get("url_status", ""),
                "threat": entry.get("threat", ""),
                "tags": entry.get("tags") or [],
                "host": entry.get("host", ""),
                "date_added": entry.get("dateadded", ""),
                "reporter": entry.get("reporter", ""),
            })
        logger.info("urlhaus_parsed", count=len(urls))
        return urls

    def _parse_malwarebazaar(self, data: dict[str, Any]) -> list[dict[str, Any]]:
        samples = []
        for entry in data.get("data", []):
            samples.append({
                "sha256": entry.get("sha256_hash", ""),
                "md5": entry.get("md5_hash", ""),
                "filename": entry.get("file_name", ""),
                "file_type": entry.get("file_type", ""),
                "signature": entry.get("signature", ""),
                "tags": entry.get("tags") or [],
                "first_seen": entry.get("first_seen", ""),
                "delivery_method": entry.get("delivery_method", ""),
            })
        logger.info("malwarebazaar_parsed", count=len(samples))
        return samples

    def _parse_feodo(self, data: dict[str, Any]) -> list[dict[str, Any]]:
        indicators = []
        for entry in data:
            indicators.append({
                "ip": entry.get("ip_address", ""),
                "port": entry.get("port"),
                "status": entry.get("status", ""),
                "malware": entry.get("malware", ""),
                "first_seen": entry.get("first_seen", ""),
                "last_online": entry.get("last_online", ""),
            })
        logger.info("feodo_parsed", count=len(indicators))
        return indicators
