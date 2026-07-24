"""Paste site monitor for dark web intelligence.

Monitors paste sites (Pastebin-like) for company domain mentions,
email patterns, and keyword matches with deduplication.
"""

from __future__ import annotations

import asyncio
import hashlib
import re
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

import httpx
import structlog

logger = structlog.get_logger()


@dataclass
class PasteMatch:
    """A matched paste containing monitored keywords."""

    source_url: str
    snippet: str
    matched_keyword: str
    discovered_at: str = field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )
    paste_id: str = ""
    title: str = ""

    def __post_init__(self) -> None:
        if not self.paste_id:
            self.paste_id = hashlib.sha256(
                self.source_url.encode()
            ).hexdigest()[:12]

    def to_dict(self) -> dict[str, Any]:
        return {
            "paste_id": self.paste_id,
            "source_url": self.source_url,
            "snippet": self.snippet,
            "matched_keyword": self.matched_keyword,
            "discovered_at": self.discovered_at,
            "title": self.title,
        }


class PasteMonitor:
    """Monitor paste sites for mentions of watched keywords.

    Features:
    - Keyword and regex pattern matching
    - Deduplication via content hashing
    - Configurable scan intervals
    - HIBP paste API integration
    """

    def __init__(
        self,
        api_base: str = "https://haveibeenpwned.com/api/v3",
        api_key: str | None = None,
        scan_interval_seconds: int = 300,
        timeout: float = 10.0,
    ) -> None:
        self.api_base = api_base.rstrip("/")
        self.api_key = api_key
        self._scan_interval = scan_interval_seconds
        self._timeout = timeout
        self._keywords: list[str] = []
        self._seen_hashes: set[str] = set()
        self._matches: list[PasteMatch] = []
        self._running = False
        self._task: asyncio.Task[None] | None = None

    def add_keywords(self, keywords: list[str]) -> None:
        """Add keywords to monitor (domains, emails, company names)."""
        for kw in keywords:
            kw_lower = kw.lower().strip()
            if kw_lower and kw_lower not in self._keywords:
                self._keywords.append(kw_lower)
        logger.info("paste_keywords_updated", count=len(self._keywords))

    def remove_keyword(self, keyword: str) -> bool:
        """Remove a keyword from the watch list."""
        kw_lower = keyword.lower().strip()
        if kw_lower in self._keywords:
            self._keywords.remove(kw_lower)
            return True
        return False

    @property
    def keywords(self) -> list[str]:
        return list(self._keywords)

    @property
    def matches(self) -> list[PasteMatch]:
        return list(self._matches)

    async def scan_email(self, email: str) -> list[PasteMatch]:
        """Check pastes for a specific email via HIBP paste API."""
        email = email.lower().strip()
        pastes = await self._query_pastes_for_email(email)
        results: list[PasteMatch] = []

        for paste in pastes:
            content_hash = hashlib.sha256(
                (paste.get("Id", "") + email).encode()
            ).hexdigest()

            if content_hash in self._seen_hashes:
                continue
            self._seen_hashes.add(content_hash)

            source = paste.get("Source", "Unknown")
            paste_id = paste.get("Id", "")
            url = self._build_paste_url(source, paste_id)

            match = PasteMatch(
                source_url=url,
                snippet=f"Email found in paste: {paste.get('Title', 'Untitled')}",
                matched_keyword=email,
                title=paste.get("Title") or "Untitled",
            )
            results.append(match)
            self._matches.append(match)

        logger.info("paste_scan_email", email=email, matches=len(results))
        return results

    async def scan_content(self, content: str, source_url: str = "") -> list[PasteMatch]:
        """Scan arbitrary text content for keyword matches."""
        content_lower = content.lower()
        content_hash = hashlib.sha256(content.encode()).hexdigest()

        if content_hash in self._seen_hashes:
            return []
        self._seen_hashes.add(content_hash)

        results: list[PasteMatch] = []
        for keyword in self._keywords:
            if keyword in content_lower:
                idx = content_lower.index(keyword)
                start = max(0, idx - 50)
                end = min(len(content), idx + len(keyword) + 50)
                snippet = content[start:end].strip()

                match = PasteMatch(
                    source_url=source_url or f"scan://{content_hash[:16]}",
                    snippet=snippet,
                    matched_keyword=keyword,
                )
                results.append(match)
                self._matches.append(match)

        return results

    def start_monitoring(self) -> None:
        """Start the background paste monitoring loop."""
        if self._running:
            return
        self._running = True
        self._task = asyncio.ensure_future(self._monitor_loop())
        logger.info("paste_monitor_started", interval=self._scan_interval)

    def stop_monitoring(self) -> None:
        """Stop the background monitoring loop."""
        self._running = False
        if self._task and not self._task.done():
            self._task.cancel()
        logger.info("paste_monitor_stopped")

    def get_stats(self) -> dict[str, Any]:
        """Return monitoring statistics."""
        return {
            "keywords_monitored": len(self._keywords),
            "total_matches": len(self._matches),
            "unique_pastes_seen": len(self._seen_hashes),
            "is_running": self._running,
        }

    # ------------------------------------------------------------------
    # Internal
    # ------------------------------------------------------------------

    async def _monitor_loop(self) -> None:
        """Background loop that periodically scans for paste matches."""
        while self._running:
            try:
                await self._run_scan_cycle()
            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.error("paste_monitor_error", error=str(exc))
            await asyncio.sleep(self._scan_interval)

    async def _run_scan_cycle(self) -> None:
        """Run a single scan cycle for all email-pattern keywords."""
        email_pattern = re.compile(r"^[^@]+@[^@]+\.[^@]+$")
        for keyword in self._keywords:
            if email_pattern.match(keyword):
                await self.scan_email(keyword)

    async def _query_pastes_for_email(self, email: str) -> list[dict[str, Any]]:
        headers: dict[str, str] = {
            "User-Agent": "Raksha-Security-Platform/0.1.0",
        }
        if self.api_key:
            headers["hibp-api-key"] = self.api_key

        url = f"{self.api_base}/pasteaccount/{email}"
        try:
            async with httpx.AsyncClient(timeout=self._timeout) as client:
                resp = await client.get(url, headers=headers)
            if resp.status_code == 404:
                return []
            if resp.status_code == 429:
                retry_after = int(resp.headers.get("retry-after", "2"))
                await asyncio.sleep(retry_after)
                return await self._query_pastes_for_email(email)
            resp.raise_for_status()
            return resp.json()
        except (httpx.TimeoutException, httpx.HTTPError) as exc:
            logger.error("paste_api_error", error=str(exc))
            return []

    @staticmethod
    def _build_paste_url(source: str, paste_id: str) -> str:
        source_lower = source.lower()
        if "pastebin" in source_lower:
            return f"https://pastebin.com/{paste_id}"
        if "ghostbin" in source_lower:
            return f"https://ghostbin.com/paste/{paste_id}"
        return f"https://{source_lower}/{paste_id}"

