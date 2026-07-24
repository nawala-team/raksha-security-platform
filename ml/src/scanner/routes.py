"""FastAPI routes for the vulnerability scanner."""

from __future__ import annotations

from typing import Any

import structlog
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from .cpe_matcher import CPEMatcher
from .nvd_sync import NVDSync
from .package_parser import Package, parse_packages
from .scorer import ScoredVulnerability, VulnerabilityScorer

logger = structlog.get_logger()

router = APIRouter(prefix='/api/v1', tags=['scanner'])

# Module-level instances (initialized on import, lightweight)
_nvd_sync = NVDSync()
_cpe_matcher = CPEMatcher()
_scorer = VulnerabilityScorer()

# In-memory store for agent scan results (production would use a DB)
_agent_results: dict[str, list[dict[str, Any]]] = {}


# --- Request/Response Models ---

class PackageScanRequest(BaseModel):
    agent_id: str = Field(..., description='Identifier for the agent submitting packages')
    source: str = Field(..., description='Package manager: dpkg, rpm, pip, npm, cargo, apk')
    package_list: str = Field(..., description='Raw package list output from the agent')
    asset_value: float = Field(default=1.0, ge=0.1, le=3.0, description='Asset importance multiplier')


class VulnerabilityResponse(BaseModel):
    cve_id: str
    package_name: str
    package_version: str
    cvss_score: float
    severity: str
    epss_score: float
    priority_score: float
    priority_level: str
    description: str
    remediation: str
    cvss_vector: str | None = None


class ScanResponse(BaseModel):
    agent_id: str
    packages_scanned: int
    vulnerabilities_found: int
    critical: int
    high: int
    medium: int
    low: int
    results: list[VulnerabilityResponse]


class CVEDetailResponse(BaseModel):
    cve_id: str
    description: str
    cvss_v31_score: float | None
    cvss_v31_vector: str | None
    cvss_v31_severity: str | None
    published: str | None
    last_modified: str | None
    references: list[Any]

# --- Endpoints ---

@router.post('/scan/packages', response_model=ScanResponse)
async def scan_packages(req: PackageScanRequest) -> ScanResponse:
    """Accept a package list from an agent and return vulnerability scan results."""
    try:
        packages = parse_packages(req.package_list, req.source)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

    if not packages:
        return ScanResponse(
            agent_id=req.agent_id, packages_scanned=0,
            vulnerabilities_found=0, critical=0, high=0, medium=0, low=0, results=[],
        )

    # Find vulnerabilities via CPE matching
    raw_vulns = _cpe_matcher.scan_packages(packages)

    # Score and prioritize
    scorer = VulnerabilityScorer(asset_value=req.asset_value)
    scored = scorer.score_vulnerabilities(raw_vulns)

    # Convert to response
    results = [VulnerabilityResponse(**sv.to_dict()) for sv in scored]

    # Count by severity
    critical = sum(1 for r in results if r.priority_level == 'CRITICAL')
    high = sum(1 for r in results if r.priority_level == 'HIGH')
    medium = sum(1 for r in results if r.priority_level == 'MEDIUM')
    low = sum(1 for r in results if r.priority_level == 'LOW')

    # Store results for agent lookup
    _agent_results[req.agent_id] = [r.model_dump() for r in results]

    logger.info(
        'scan_complete',
        agent_id=req.agent_id,
        packages=len(packages),
        vulns=len(results),
    )

    return ScanResponse(
        agent_id=req.agent_id,
        packages_scanned=len(packages),
        vulnerabilities_found=len(results),
        critical=critical,
        high=high,
        medium=medium,
        low=low,
        results=results,
    )


@router.get('/vulnerabilities/{agent_id}', response_model=list[VulnerabilityResponse])
async def get_agent_vulnerabilities(agent_id: str) -> list[VulnerabilityResponse]:
    """Get cached vulnerability results for a specific agent."""
    if agent_id not in _agent_results:
        raise HTTPException(status_code=404, detail=f'No scan results found for agent: {agent_id}')
    return [VulnerabilityResponse(**v) for v in _agent_results[agent_id]]


@router.get('/cve/{cve_id}', response_model=CVEDetailResponse)
async def get_cve_detail(cve_id: str) -> CVEDetailResponse:
    """Get detailed information about a specific CVE."""
    # Validate CVE ID format
    if not cve_id.startswith('CVE-'):
        raise HTTPException(status_code=400, detail='Invalid CVE ID format. Expected: CVE-YYYY-NNNNN')

    cve_data = _nvd_sync.get_cve(cve_id)
    if not cve_data:
        raise HTTPException(status_code=404, detail=f'CVE {cve_id} not found in local cache')

    import json
    refs = []
    if cve_data.get('references_json'):
        try:
            refs = json.loads(cve_data['references_json'])
        except (json.JSONDecodeError, TypeError):
            pass

    return CVEDetailResponse(
        cve_id=cve_data['cve_id'],
        description=cve_data.get('description', ''),
        cvss_v31_score=cve_data.get('cvss_v31_score'),
        cvss_v31_vector=cve_data.get('cvss_v31_vector'),
        cvss_v31_severity=cve_data.get('cvss_v31_severity'),
        published=cve_data.get('published'),
        last_modified=cve_data.get('last_modified'),
        references=refs,
    )


@router.get('/scanner/status')
async def scanner_status() -> dict[str, Any]:
    """Get scanner module status and NVD sync stats."""
    stats = _nvd_sync.get_stats()
    return {
        'status': 'operational',
        'nvd_cache': stats,
        'agents_scanned': len(_agent_results),
    }