"""Breach checker for emails and domains.

Checks if email addresses or domains have appeared in known data breaches
using HIBP-compatible APIs with rate limiting and caching.
"""

from __future__ import annotations

import asyncio
import hashlib
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

import httpx
import structlog

logger = structlog.get_logger()

DEFAULT_API_BASE = "https://haveibeenpwned.com/api/v3"


@dataclass
class BreachResult:
    """Result of a breach check for a single email or domain."""

    email: str
    breaches_found: int
    sources: list[str] = field(default_factory=list)
    dates: list[str] = field(default_factory=list)
    data_types_leaked: list[str] = field(default_factory=list)
    last_checked: str = field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )

    def to_dict(self) -> dict[str, Any]:
        return {
            "email": self.email,
            "breaches_found": self.breaches_found,
            "sources": self.sources,
            "dates": self.dates,
            "data_types_leaked": self.data_types_leaked,
            "last_checked": self.last_checked,
        }


@dataclass
class _CacheEntry:
    result: BreachResult
    expires_at: float


class BreachChecker:
    """Check emails and domains against breach databases.

    Features:
    - HIBP-compatible API integration
    - Rate limiting (configurable requests per second)
    - In-memory caching with TTL
    - Domain-wide breach enumeration
    """

    def __init__(
        self,
        api_base: str = DEFAULT_API_BASE,
        api_key: str | None = None,
        rate_limit_rps: float = 1.5,
        cache_ttl_seconds: int = 3600,
        timeout: float = 10.0,
    ) -> None:
        self.api_base = api_base.rstrip("/")
        self.api_key = api_key
        self._min_interval = 1.0 / rate_limit_rps
        self._cache_ttl = cache_ttl_seconds
        self._timeout = timeout
        self._last_request_time: float = 0.0
        self._cache: dict[str, _CacheEntry] = {}
        self._lock = asyncio.Lock()

    async def check_email(self, email: str) -> BreachResult:
        """Check a single email address against breach databases."""
        email = email.lower().strip()
        cached = self._get_cached(email)
        if cached is not None:
            return cached

        breaches = await self._query_breaches_for_account(email)
        result = self._build_result(email, breaches)
        self._set_cached(email, result)
        logger.info("breach_check_complete", email=email, breaches=result.breaches_found)
        return result

    async def check_domain(self, domain: str) -> list[BreachResult]:
        """Check all known breaches associated with a domain."""
        domain = domain.lower().strip()
        cached = self._get_cached(f"domain:{domain}")
        if cached is not None:
            return [cached]

        breaches = await self._query_breaches_for_domain(domain)
        result = self._build_result(f"*@{domain}", breaches)
        self._set_cached(f"domain:{domain}", result)
        logger.info("breach_domain_check", domain=domain, breaches=result.breaches_found)
        return [result]

    async def check_emails_batch(self, emails: list[str]) -> list[BreachResult]:
        """Check multiple emails sequentially (respects rate limiting)."""
        results: list[BreachResult] = []
        for email in emails:
            results.append(await self.check_email(email))
        return results

    def get_password_hash_prefix(self, password: str) -> str:
        """Return first 5 chars of SHA-1 hash for k-Anonymity password check."""
        sha1 = hashlib.sha1(password.encode("utf-8")).hexdigest().upper()  # noqa: S324
        return sha1[:5]

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    async def _rate_limit(self) -> None:
        """Enforce rate limiting between API calls."""
        async with self._lock:
            now = time.monotonic()
            elapsed = now - self._last_request_time
            if elapsed < self._min_interval:
                await asyncio.sleep(self._min_interval - elapsed)
            self._last_request_time = time.monotonic()

    async def _query_breaches_for_account(self, email: str) -> list[dict[str, Any]]:
        await self._rate_limit()
        url = f"{self.api_base}/breachedaccount/{email}"
        return await self._make_request(url, {"truncateResponse": "false"})

    async def _query_breaches_for_domain(self, domain: str) -> list[dict[str, Any]]:
        await self._rate_limit()
        url = f"{self.api_base}/breaches"
        return await self._make_request(url, {"domain": domain})

    async def _make_request(
        self, url: str, params: dict[str, str]
    ) -> list[dict[str, Any]]:
        headers: dict[str, str] = {
            "User-Agent": "Raksha-Security-Platform/0.1.0",
        }
        if self.api_key:
            headers["hibp-api-key"] = self.api_key

        try:
            async with httpx.AsyncClient(timeout=self._timeout) as client:
                resp = await client.get(url, headers=headers, params=params)

            if resp.status_code == 404:
                return []
            if resp.status_code == 429:
                retry_after = int(resp.headers.get("retry-after", "2"))
                logger.warning("breach_api_rate_limited", retry_after=retry_after)
                await asyncio.sleep(retry_after)
                return await self._make_request(url, params)
            if resp.status_code == 401:
                logger.error("breach_api_unauthorized")
                return []

            resp.raise_for_status()
            return resp.json()
        except httpx.TimeoutException:
            logger.error("breach_api_timeout", url=url)
            return []
        except httpx.HTTPError as exc:
            logger.error("breach_api_error", url=url, error=str(exc))
            return []

    def _build_result(self, email: str, breaches: list[dict[str, Any]]) -> BreachResult:
        if not breaches:
            return BreachResult(email=email, breaches_found=0)

        sources: list[str] = []
        dates: list[str] = []
        data_types: set[str] = set()

        for breach in breaches:
            sources.append(breach.get("Name") or breach.get("name", "Unknown"))
            breach_date = breach.get("BreachDate") or breach.get("breach_date", "")
            if breach_date:
                dates.append(breach_date)
            for dtype in breach.get("DataClasses", breach.get("data_classes", [])):
                data_types.add(dtype)

        return BreachResult(
            email=email,
            breaches_found=len(breaches),
            sources=sources,
            dates=sorted(dates),
            data_types_leaked=sorted(data_types),
        )

    def _get_cached(self, key: str) -> BreachResult | None:
        entry = self._cache.get(key)
        if entry is None:
            return None
        if time.monotonic() > entry.expires_at:
            del self._cache[key]
            return None
        return entry.result

    def _set_cached(self, key: str, result: BreachResult) -> None:
        self._cache[key] = _CacheEntry(
            result=result, expires_at=time.monotonic() + self._cache_ttl
        )

    def clear_cache(self) -> None:
        """Clear the entire breach cache."""
        self._cache.clear()

