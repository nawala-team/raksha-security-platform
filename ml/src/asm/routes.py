"""FastAPI routes for Attack Surface Management."""

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Any

import structlog
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from .subdomain_enum import SubdomainEnumerator, DiscoveredSubdomain
from .port_scanner import PortScanner, OpenPort
from .service_fingerprint import ServiceFingerprinter
from .exposure_scorer import ExposureScorer

logger = structlog.get_logger()

router = APIRouter(prefix='/api/v1/asm', tags=['asm'])

# In-memory store (production would use a database)
_discovered_assets: dict[str, dict[str, Any]] = {}
_scan_results: dict[str, dict[str, Any]] = {}


# --- Request/Response Models ---

class DiscoverRequest(BaseModel):
    domain: str = Field(..., description='Target domain to discover assets for')
    wordlist: list[str] | None = Field(None, description='Custom subdomain wordlist')
    concurrency: int = Field(default=50, ge=1, le=500, description='DNS resolution concurrency')


class PortScanRequest(BaseModel):
    targets: list[str] = Field(..., description='List of IPs or hostnames to scan')
    ports: list[int] | None = Field(None, description='Custom port list (default: top 50)')
    concurrency: int = Field(default=200, ge=1, le=1000, description='Scan concurrency')


class SubdomainResponse(BaseModel):
    domain: str
    ip: str | None
    source: str
    first_seen: str
    record_type: str | None = None


class OpenPortResponse(BaseModel):
    ip: str
    port: int
    service: str
    banner: str
    state: str


class FindingResponse(BaseModel):
    category: str
    severity: str
    title: str
    description: str
    recommendation: str
    score_impact: float


class ExposureScoreResponse(BaseModel):
    score: float
    summary: str
    findings: list[FindingResponse]
    recommendations: list[str]
    total_findings: int
    critical_findings: int
    high_findings: int
    medium_findings: int
    low_findings: int


class DiscoverResponse(BaseModel):
    scan_id: str
    domain: str
    subdomains_found: int
    assets: list[SubdomainResponse]


class PortScanResponse(BaseModel):
    scan_id: str
    targets_scanned: int
    open_ports: list[OpenPortResponse]
    exposure_score: ExposureScoreResponse | None = None


class AssetSummary(BaseModel):
    asset_id: str
    domain: str
    ip: str | None
    source: str
    first_seen: str
    open_ports: int
    exposure_score: float | None = None


# --- Endpoints ---

@router.post('/discover', response_model=DiscoverResponse)
async def discover_assets(req: DiscoverRequest) -> DiscoverResponse:
    """Start a discovery scan for a domain - enumerate subdomains and assets."""
    logger.info('asm_discover_start', domain=req.domain)

    enumerator = SubdomainEnumerator(
        wordlist=req.wordlist,
        concurrency=req.concurrency,
    )
    subdomains = await enumerator.enumerate(req.domain)

    scan_id = str(uuid.uuid4())

    # Store discovered assets
    for sub in subdomains:
        asset_id = str(uuid.uuid4())
        _discovered_assets[asset_id] = {
            'asset_id': asset_id,
            'domain': sub.domain,
            'ip': sub.ip,
            'source': sub.source,
            'first_seen': sub.first_seen,
            'record_type': sub.record_type,
            'open_ports': [],
            'exposure_score': None,
            'scan_id': scan_id,
        }

    _scan_results[scan_id] = {
        'scan_id': scan_id,
        'domain': req.domain,
        'timestamp': datetime.now(timezone.utc).isoformat(),
        'subdomains_found': len(subdomains),
    }

    logger.info('asm_discover_complete', domain=req.domain, found=len(subdomains))

    return DiscoverResponse(
        scan_id=scan_id,
        domain=req.domain,
        subdomains_found=len(subdomains),
        assets=[
            SubdomainResponse(
                domain=s.domain,
                ip=s.ip,
                source=s.source,
                first_seen=s.first_seen,
                record_type=s.record_type,
            )
            for s in subdomains
        ],
    )

@router.get('/assets', response_model=list[AssetSummary])
async def list_assets() -> list[AssetSummary]:
    """List all discovered assets."""
    return [
        AssetSummary(
            asset_id=asset['asset_id'],
            domain=asset['domain'],
            ip=asset['ip'],
            source=asset['source'],
            first_seen=asset['first_seen'],
            open_ports=len(asset.get('open_ports', [])),
            exposure_score=asset.get('exposure_score'),
        )
        for asset in _discovered_assets.values()
    ]


@router.get('/assets/{asset_id}')
async def get_asset_detail(asset_id: str) -> dict[str, Any]:
    """Get detailed information about a specific asset."""
    if asset_id not in _discovered_assets:
        raise HTTPException(status_code=404, detail=f'Asset not found: {asset_id}')
    return _discovered_assets[asset_id]


@router.get('/exposure-score', response_model=ExposureScoreResponse)
async def get_exposure_score() -> ExposureScoreResponse:
    """Calculate current overall exposure score from all scanned assets."""
    all_ports: list[OpenPort] = []
    for asset in _discovered_assets.values():
        for port_data in asset.get('open_ports', []):
            all_ports.append(OpenPort(**port_data))

    if not all_ports:
        return ExposureScoreResponse(
            score=0.0,
            summary='No port scan data available. Run a port scan first.',
            findings=[],
            recommendations=['Run a port scan to assess exposure.'],
            total_findings=0,
            critical_findings=0,
            high_findings=0,
            medium_findings=0,
            low_findings=0,
        )

    scorer = ExposureScorer()
    report = scorer.score(all_ports)
    return ExposureScoreResponse(**report.to_dict())


@router.post('/scan/ports', response_model=PortScanResponse)
async def scan_ports(req: PortScanRequest) -> PortScanResponse:
    """Scan specific targets for open ports and calculate exposure."""
    logger.info('asm_port_scan_request', targets=req.targets, ports=req.ports)

    scanner = PortScanner(
        ports=req.ports,
        concurrency=req.concurrency,
    )

    all_open_ports: list[OpenPort] = []
    for target in req.targets:
        ports = await scanner.scan(target)
        all_open_ports.extend(ports)

        # Update any matching assets with port scan results
        for asset in _discovered_assets.values():
            if asset.get('ip') == target or asset.get('domain') == target:
                asset['open_ports'] = [p.to_dict() for p in ports]

    # Fingerprint HTTP services
    fingerprinter = ServiceFingerprinter()
    service_infos = []
    for port in all_open_ports:
        if port.service in ('http', 'https', 'http-proxy', 'http-alt', 'https-alt'):
            info = await fingerprinter.fingerprint(port.ip, port.port)
            service_infos.append(info)

    # Calculate exposure score
    scorer = ExposureScorer()
    report = scorer.score(all_open_ports, service_infos or None)

    # Update asset exposure scores
    for asset in _discovered_assets.values():
        if asset.get('ip') in req.targets or asset.get('domain') in req.targets:
            asset['exposure_score'] = report.score

    scan_id = str(uuid.uuid4())

    return PortScanResponse(
        scan_id=scan_id,
        targets_scanned=len(req.targets),
        open_ports=[OpenPortResponse(**p.to_dict()) for p in all_open_ports],
        exposure_score=ExposureScoreResponse(**report.to_dict()),
    )


@router.get('/status')
async def asm_status() -> dict[str, Any]:
    """Get ASM module status."""
    return {
        'status': 'operational',
        'total_assets': len(_discovered_assets),
        'total_scans': len(_scan_results),
    }


