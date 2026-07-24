"""FastAPI routes for Dark Web Monitoring."""

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Any

import structlog
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from .breach_checker import BreachChecker, BreachResult
from .paste_monitor import PasteMonitor, PasteMatch
from .watchlist import WatchlistManager, WatchlistItemType

logger = structlog.get_logger()

router = APIRouter(prefix="/api/v1/darkweb", tags=["darkweb"])

# Module-level instances (production would use dependency injection)
_breach_checker = BreachChecker()
_paste_monitor = PasteMonitor()
_watchlist_manager = WatchlistManager(
    breach_checker=_breach_checker,
    paste_monitor=_paste_monitor,
)

# In-memory leak store
_discovered_leaks: dict[str, dict[str, Any]] = {}


# --- Request/Response Models ---

class CheckDomainRequest(BaseModel):
    domain: str = Field(..., description="Domain to check for breaches")


class CheckEmailRequest(BaseModel):
    email: str = Field(..., description="Email to check for breaches")



class WatchlistAddRequest(BaseModel):
    value: str = Field(..., description="Domain or email to watch")
    type: str = Field(..., description="Item type: 'domain' or 'email'")
    added_by: str = Field(default="api", description="Who added this item")


class BreachResponse(BaseModel):
    email: str
    breaches_found: int
    sources: list[str]
    dates: list[str]
    data_types_leaked: list[str]
    last_checked: str


class LeakResponse(BaseModel):
    leak_id: str
    email: str
    breaches_found: int
    sources: list[str]
    dates: list[str]
    data_types_leaked: list[str]
    discovered_at: str


class WatchlistItemResponse(BaseModel):
    id: str
    value: str
    type: str
    added_by: str
    last_checked: str
    alert_count: int
    created_at: str


class StatsResponse(BaseModel):
    total_leaks_discovered: int
    total_watchlist_items: int
    total_alerts: int
    domains_watched: int
    emails_watched: int
    paste_monitor_stats: dict[str, Any]


# --- Endpoints ---

@router.post("/check-domain", response_model=list[BreachResponse])
async def check_domain(req: CheckDomainRequest) -> list[BreachResponse]:
    """Check a domain for known data breaches."""
    logger.info("darkweb_check_domain", domain=req.domain)
    results = await _breach_checker.check_domain(req.domain)

    # Store as discovered leaks
    for r in results:
        if r.breaches_found > 0:
            leak_id = str(uuid.uuid4())
            _discovered_leaks[leak_id] = {
                "leak_id": leak_id,
                **r.to_dict(),
                "discovered_at": datetime.now(timezone.utc).isoformat(),
            }

    return [
        BreachResponse(
            email=r.email,
            breaches_found=r.breaches_found,
            sources=r.sources,
            dates=r.dates,
            data_types_leaked=r.data_types_leaked,
            last_checked=r.last_checked,
        )
        for r in results
    ]


@router.post("/check-email", response_model=BreachResponse)
async def check_email(req: CheckEmailRequest) -> BreachResponse:
    """Check a specific email for known data breaches."""
    logger.info("darkweb_check_email", email=req.email)
    result = await _breach_checker.check_email(req.email)

    if result.breaches_found > 0:
        leak_id = str(uuid.uuid4())
        _discovered_leaks[leak_id] = {
            "leak_id": leak_id,
            **result.to_dict(),
            "discovered_at": datetime.now(timezone.utc).isoformat(),
        }

    return BreachResponse(
        email=result.email,
        breaches_found=result.breaches_found,
        sources=result.sources,
        dates=result.dates,
        data_types_leaked=result.data_types_leaked,
        last_checked=result.last_checked,
    )


@router.get("/leaks", response_model=list[LeakResponse])
async def list_leaks() -> list[LeakResponse]:
    """List all discovered leaks."""
    return [LeakResponse(**leak) for leak in _discovered_leaks.values()]


@router.get("/leaks/{leak_id}", response_model=LeakResponse)
async def get_leak_detail(leak_id: str) -> LeakResponse:
    """Get details of a specific discovered leak."""
    if leak_id not in _discovered_leaks:
        raise HTTPException(status_code=404, detail=f"Leak not found: {leak_id}")
    return LeakResponse(**_discovered_leaks[leak_id])


@router.post("/watchlist", response_model=WatchlistItemResponse)
async def add_to_watchlist(req: WatchlistAddRequest) -> WatchlistItemResponse:
    """Add a domain or email to the watchlist."""
    try:
        item_type = WatchlistItemType(req.type.lower())
    except ValueError:
        raise HTTPException(
            status_code=400, detail="Invalid type. Must be 'domain' or 'email'."
        )

    item = _watchlist_manager.add_item(
        value=req.value, item_type=item_type, added_by=req.added_by
    )
    return WatchlistItemResponse(**item.to_dict())


@router.get("/watchlist", response_model=list[WatchlistItemResponse])
async def get_watchlist() -> list[WatchlistItemResponse]:
    """Get all watchlist items."""
    items = _watchlist_manager.get_items()
    return [WatchlistItemResponse(**item.to_dict()) for item in items]


@router.delete("/watchlist/{item_id}")
async def remove_from_watchlist(item_id: str) -> dict[str, Any]:
    """Remove an item from the watchlist."""
    removed = _watchlist_manager.remove_item(item_id)
    if not removed:
        raise HTTPException(status_code=404, detail=f"Watchlist item not found: {item_id}")
    return {"status": "removed", "id": item_id}


@router.get("/stats", response_model=StatsResponse)
async def get_stats() -> StatsResponse:
    """Get dark web monitoring summary statistics."""
    wl_stats = _watchlist_manager.get_stats()
    paste_stats = _paste_monitor.get_stats()

    return StatsResponse(
        total_leaks_discovered=len(_discovered_leaks),
        total_watchlist_items=wl_stats["total_items"],
        total_alerts=wl_stats["total_alerts"],
        domains_watched=wl_stats["domains_watched"],
        emails_watched=wl_stats["emails_watched"],
        paste_monitor_stats=paste_stats,
    )

