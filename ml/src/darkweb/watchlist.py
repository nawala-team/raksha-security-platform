"""Watchlist management for dark web monitoring.

Manages domain/email watchlists with automatic periodic checks
and alert generation when matches are found.
"""

from __future__ import annotations

import asyncio
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Any

import structlog

from .breach_checker import BreachChecker, BreachResult
from .paste_monitor import PasteMonitor, PasteMatch

logger = structlog.get_logger()


class WatchlistItemType(str, Enum):
    DOMAIN = "domain"
    EMAIL = "email"


@dataclass
class WatchlistAlert:
    """Alert generated when a watchlist item has a match."""

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    watchlist_item_id: str = ""
    alert_type: str = ""  # "breach" or "paste"
    details: dict[str, Any] = field(default_factory=dict)
    created_at: str = field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "watchlist_item_id": self.watchlist_item_id,
            "alert_type": self.alert_type,
            "details": self.details,
            "created_at": self.created_at,
        }


@dataclass
class WatchlistItem:
    """A single item being monitored on the watchlist."""

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    value: str = ""
    type: WatchlistItemType = WatchlistItemType.DOMAIN
    added_by: str = "system"
    last_checked: str = ""
    alert_count: int = 0
    created_at: str = field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "value": self.value,
            "type": self.type.value,
            "added_by": self.added_by,
            "last_checked": self.last_checked,
            "alert_count": self.alert_count,
            "created_at": self.created_at,
        }


class WatchlistManager:
    """Manage domain/email watchlists with periodic checks and alerting."""

    def __init__(
        self,
        breach_checker: BreachChecker | None = None,
        paste_monitor: PasteMonitor | None = None,
        check_interval_seconds: int = 3600,
    ) -> None:
        self._breach_checker = breach_checker or BreachChecker()
        self._paste_monitor = paste_monitor or PasteMonitor()
        self._check_interval = check_interval_seconds
        self._items: dict[str, WatchlistItem] = {}
        self._alerts: list[WatchlistAlert] = []
        self._running = False
        self._task: asyncio.Task[None] | None = None

    def add_item(
        self,
        value: str,
        item_type: WatchlistItemType,
        added_by: str = "system",
    ) -> WatchlistItem:
        """Add an item to the watchlist."""
        for item in self._items.values():
            if item.value.lower() == value.lower() and item.type == item_type:
                return item
        item = WatchlistItem(
            value=value.lower().strip(),
            type=item_type,
            added_by=added_by,
        )
        self._items[item.id] = item
        self._paste_monitor.add_keywords([value])
        logger.info("watchlist_item_added", value=value, type=item_type.value)
        return item

    def remove_item(self, item_id: str) -> bool:
        """Remove an item from the watchlist by ID."""
        item = self._items.pop(item_id, None)
        if item is None:
            return False
        self._paste_monitor.remove_keyword(item.value)
        logger.info("watchlist_item_removed", id=item_id, value=item.value)
        return True

    def get_items(self) -> list[WatchlistItem]:
        return list(self._items.values())

    def get_item(self, item_id: str) -> WatchlistItem | None:
        return self._items.get(item_id)

    def get_alerts(self, item_id: str | None = None) -> list[WatchlistAlert]:
        if item_id is None:
            return list(self._alerts)
        return [a for a in self._alerts if a.watchlist_item_id == item_id]

    @property
    def total_alerts(self) -> int:
        return len(self._alerts)

    async def check_item(self, item: WatchlistItem) -> list[WatchlistAlert]:
        """Run breach and paste checks for a single watchlist item."""
        new_alerts: list[WatchlistAlert] = []

        if item.type == WatchlistItemType.EMAIL:
            breach_result = await self._breach_checker.check_email(item.value)
            if breach_result.breaches_found > 0:
                new_alerts.append(WatchlistAlert(
                    watchlist_item_id=item.id,
                    alert_type="breach",
                    details=breach_result.to_dict(),
                ))
            paste_matches = await self._paste_monitor.scan_email(item.value)
            for match in paste_matches:
                new_alerts.append(WatchlistAlert(
                    watchlist_item_id=item.id,
                    alert_type="paste",
                    details=match.to_dict(),
                ))

        elif item.type == WatchlistItemType.DOMAIN:
            breach_results = await self._breach_checker.check_domain(item.value)
            for br in breach_results:
                if br.breaches_found > 0:
                    new_alerts.append(WatchlistAlert(
                        watchlist_item_id=item.id,
                        alert_type="breach",
                        details=br.to_dict(),
                    ))

        item.last_checked = datetime.now(timezone.utc).isoformat()
        item.alert_count += len(new_alerts)
        self._alerts.extend(new_alerts)
        return new_alerts

    def start_monitoring(self) -> None:
        """Start the background watchlist check loop."""
        if self._running:
            return
        self._running = True
        self._task = asyncio.ensure_future(self._check_loop())
        logger.info("watchlist_monitor_started", interval=self._check_interval)

    def stop_monitoring(self) -> None:
        """Stop the background monitoring loop."""
        self._running = False
        if self._task and not self._task.done():
            self._task.cancel()
        logger.info("watchlist_monitor_stopped")

    def get_stats(self) -> dict[str, Any]:
        """Return watchlist statistics."""
        return {
            "total_items": len(self._items),
            "total_alerts": len(self._alerts),
            "domains_watched": sum(
                1 for i in self._items.values()
                if i.type == WatchlistItemType.DOMAIN
            ),
            "emails_watched": sum(
                1 for i in self._items.values()
                if i.type == WatchlistItemType.EMAIL
            ),
            "is_running": self._running,
        }

    async def _check_loop(self) -> None:
        """Background loop for periodic watchlist checks."""
        while self._running:
            try:
                for item in list(self._items.values()):
                    if not self._running:
                        break
                    await self.check_item(item)
            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.error("watchlist_check_error", error=str(exc))
            await asyncio.sleep(self._check_interval)


