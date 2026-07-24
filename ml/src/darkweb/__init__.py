"""Raksha Dark Web Monitoring Module.

Provides:
- Breach checking for emails and domains (HIBP-compatible)
- Paste site monitoring with keyword matching
- Watchlist management with automatic periodic checks
- Alert generation on new findings
"""

from .breach_checker import BreachChecker, BreachResult
from .paste_monitor import PasteMonitor, PasteMatch
from .watchlist import WatchlistManager, WatchlistItem, WatchlistAlert, WatchlistItemType

__all__ = [
    "BreachChecker",
    "BreachResult",
    "PasteMonitor",
    "PasteMatch",
    "WatchlistManager",
    "WatchlistItem",
    "WatchlistAlert",
    "WatchlistItemType",
]
