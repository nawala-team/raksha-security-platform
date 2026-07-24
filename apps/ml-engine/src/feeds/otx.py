"""AlienVault OTX (Open Threat Exchange) feed integration."""

from typing import Any

import httpx
import structlog

logger = structlog.get_logger(__name__)

OTX_API_BASE = "https://otx.alienvault.com/api/v1"


class OTXFeed:
    """Fetches threat intelligence from AlienVault OTX."""

    def __init__(self, api_key: str):
        self.api_key = api_key
        self.headers = {"X-OTX-API-KEY": api_key}

    async def fetch_subscribed_pulses(self, days: int = 7, limit: int = 50) -> list[dict[str, Any]]:
        """Fetch pulses from subscribed feeds."""
        async with httpx.AsyncClient(timeout=60.0) as client:
            try:
                response = await client.get(
                    f"{OTX_API_BASE}/pulses/subscribed",
                    headers=self.headers,
                    params={"modified_since": f"{days}d", "limit": limit},
                )
                response.raise_for_status()
                data = response.json()
                return self._parse_pulses(data)
            except Exception as e:
                logger.error("otx_fetch_failed", error=str(e))
                return []

    async def fetch_indicators_for_pulse(self, pulse_id: str) -> list[dict[str, Any]]:
        """Fetch all indicators (IOCs) for a specific pulse."""
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.get(
                f"{OTX_API_BASE}/pulses/{pulse_id}/indicators",
                headers=self.headers,
            )
            response.raise_for_status()
            data = response.json()
            return self._parse_indicators(data.get("results", []))

    async def check_ip(self, ip: str) -> dict[str, Any]:
        """Check an IP against OTX reputation data."""
        async with httpx.AsyncClient(timeout=15.0) as client:
            response = await client.get(
                f"{OTX_API_BASE}/indicators/IPv4/{ip}/general",
                headers=self.headers,
            )
            response.raise_for_status()
            data = response.json()
            return {
                "ip": ip,
                "reputation": data.get("reputation", 0),
                "pulse_count": data.get("pulse_info", {}).get("count", 0),
                "country": data.get("country_name", ""),
                "asn": data.get("asn", ""),
            }

    async def check_domain(self, domain: str) -> dict[str, Any]:
        """Check a domain against OTX reputation data."""
        async with httpx.AsyncClient(timeout=15.0) as client:
            response = await client.get(
                f"{OTX_API_BASE}/indicators/domain/{domain}/general",
                headers=self.headers,
            )
            response.raise_for_status()
            data = response.json()
            return {
                "domain": domain,
                "pulse_count": data.get("pulse_info", {}).get("count", 0),
                "alexa_rank": data.get("alexa", ""),
                "whois": data.get("whois", "")[:200],
            }

    async def check_file_hash(self, file_hash: str) -> dict[str, Any]:
        """Check a file hash against OTX."""
        async with httpx.AsyncClient(timeout=15.0) as client:
            response = await client.get(
                f"{OTX_API_BASE}/indicators/file/{file_hash}/general",
                headers=self.headers,
            )
            response.raise_for_status()
            data = response.json()
            return {
                "hash": file_hash,
                "pulse_count": data.get("pulse_info", {}).get("count", 0),
                "malware_families": data.get("pulse_info", {}).get("related", {}).get("malware_families", []),
            }

    def _parse_pulses(self, data: dict[str, Any]) -> list[dict[str, Any]]:
        pulses = []
        for pulse in data.get("results", []):
            pulses.append({
                "id": pulse.get("id", ""),
                "name": pulse.get("name", ""),
                "description": pulse.get("description", "")[:300],
                "author": pulse.get("author_name", ""),
                "created": pulse.get("created", ""),
                "modified": pulse.get("modified", ""),
                "tags": pulse.get("tags", []),
                "targeted_countries": pulse.get("targeted_countries", []),
                "attack_ids": [
                    a.get("id") for a in pulse.get("attack_ids", [])
                ],
                "indicator_count": pulse.get("indicator_count", 0),
            })
        logger.info("otx_pulses_parsed", count=len(pulses))
        return pulses

    def _parse_indicators(self, indicators: list[dict[str, Any]]) -> list[dict[str, Any]]:
        parsed = []
        for ind in indicators:
            parsed.append({
                "type": ind.get("type", ""),
                "indicator": ind.get("indicator", ""),
                "title": ind.get("title", ""),
                "description": ind.get("description", ""),
                "created": ind.get("created", ""),
                "is_active": ind.get("is_active", 1) == 1,
            })
        return parsed
