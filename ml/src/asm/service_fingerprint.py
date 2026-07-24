"""Service fingerprinting: HTTP headers, TLS certs, and technology detection."""

from __future__ import annotations

import ssl
import socket
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

import httpx
import structlog

logger = structlog.get_logger()

# Known server header patterns -> technology
SERVER_PATTERNS: dict[str, str] = {
    'nginx': 'Nginx',
    'apache': 'Apache HTTP Server',
    'microsoft-iis': 'Microsoft IIS',
    'litespeed': 'LiteSpeed',
    'cloudflare': 'Cloudflare',
    'openresty': 'OpenResty',
    'gunicorn': 'Gunicorn',
    'uvicorn': 'Uvicorn',
    'express': 'Express.js (Node.js)',
    'kestrel': 'Kestrel (.NET)',
    'jetty': 'Eclipse Jetty',
    'tomcat': 'Apache Tomcat',
    'caddy': 'Caddy',
    'envoy': 'Envoy Proxy',
    'haproxy': 'HAProxy',
    'traefik': 'Traefik',
}

# Security headers to check
SECURITY_HEADERS: list[str] = [
    'strict-transport-security',
    'x-content-type-options',
    'x-frame-options',
    'content-security-policy',
    'x-xss-protection',
    'referrer-policy',
    'permissions-policy',
]


@dataclass
class TLSInfo:
    """TLS certificate information."""

    subject: str
    issuer: str
    not_before: str
    not_after: str
    serial_number: str
    version: int
    san: list[str] = field(default_factory=list)
    is_expired: bool = False
    days_until_expiry: int = 0

    def to_dict(self) -> dict[str, Any]:
        return {
            'subject': self.subject,
            'issuer': self.issuer,
            'not_before': self.not_before,
            'not_after': self.not_after,
            'serial_number': self.serial_number,
            'version': self.version,
            'san': self.san,
            'is_expired': self.is_expired,
            'days_until_expiry': self.days_until_expiry,
        }


@dataclass
class ServiceInfo:
    """Fingerprinted service information."""

    host: str
    port: int
    technologies: list[str] = field(default_factory=list)
    server_header: str = ''
    detected_version: str | None = None
    tls_info: TLSInfo | None = None
    security_headers_present: list[str] = field(default_factory=list)
    security_headers_missing: list[str] = field(default_factory=list)
    response_headers: dict[str, str] = field(default_factory=dict)
    powered_by: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            'host': self.host,
            'port': self.port,
            'technologies': self.technologies,
            'server_header': self.server_header,
            'detected_version': self.detected_version,
            'tls_info': self.tls_info.to_dict() if self.tls_info else None,
            'security_headers_present': self.security_headers_present,
            'security_headers_missing': self.security_headers_missing,
            'response_headers': self.response_headers,
            'powered_by': self.powered_by,
        }


class ServiceFingerprinter:
    """Fingerprint services via HTTP headers and TLS inspection."""

    def __init__(self, timeout: float = 10.0):
        self.timeout = timeout

    async def fingerprint(self, host: str, port: int) -> ServiceInfo:
        """Fingerprint a service running on host:port."""
        info = ServiceInfo(host=host, port=port)

        use_tls = port in (443, 8443, 9443, 5986, 6443)
        is_http = port in (80, 443, 8080, 8000, 8008, 8443, 8888, 9090, 9443, 3000, 5000)

        if is_http or use_tls:
            await self._fingerprint_http(info, use_tls)

        if use_tls or port == 443:
            tls_info = await self._get_tls_info(host, port)
            if tls_info:
                info.tls_info = tls_info

        return info

    async def _fingerprint_http(self, info: ServiceInfo, use_tls: bool) -> None:
        """Analyze HTTP response headers for technology detection."""
        scheme = 'https' if use_tls else 'http'
        url = f'{scheme}://{info.host}:{info.port}/'

        try:
            async with httpx.AsyncClient(
                timeout=self.timeout, verify=False, follow_redirects=True
            ) as client:
                response = await client.head(url)
                headers = {k.lower(): v for k, v in response.headers.items()}
                info.response_headers = dict(response.headers)
        except (httpx.HTTPError, OSError) as e:
            logger.debug(
                'asm_http_fingerprint_failed', host=info.host, port=info.port, error=str(e)
            )
            return

        # Server header
        server = headers.get('server', '')
        info.server_header = server
        if server:
            self._detect_technology(info, server)

        # X-Powered-By
        powered_by = headers.get('x-powered-by', '')
        if powered_by:
            info.powered_by = powered_by
            info.technologies.append(powered_by)

        # Security headers check
        for header in SECURITY_HEADERS:
            if header in headers:
                info.security_headers_present.append(header)
            else:
                info.security_headers_missing.append(header)

    def _detect_technology(self, info: ServiceInfo, server_header: str) -> None:
        """Detect technology and version from server header."""
        server_lower = server_header.lower()
        for pattern, tech_name in SERVER_PATTERNS.items():
            if pattern in server_lower:
                info.technologies.append(tech_name)
                parts = server_header.split('/')
                if len(parts) >= 2:
                    version_part = parts[1].split()[0]
                    if version_part and version_part[0].isdigit():
                        info.detected_version = f'{tech_name} {version_part}'
                break

    async def _get_tls_info(self, host: str, port: int) -> TLSInfo | None:
        """Extract TLS certificate information."""
        try:
            import asyncio
            loop = asyncio.get_event_loop()
            return await loop.run_in_executor(None, self._fetch_cert_sync, host, port)
        except Exception as e:
            logger.debug('asm_tls_info_failed', host=host, port=port, error=str(e))
            return None

    def _fetch_cert_sync(self, host: str, port: int) -> TLSInfo | None:
        """Synchronously fetch TLS certificate (runs in thread pool)."""
        ctx = ssl.create_default_context()
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE

        try:
            with socket.create_connection((host, port), timeout=self.timeout) as sock:
                with ctx.wrap_socket(sock, server_hostname=host) as tls_sock:
                    cert = tls_sock.getpeercert(binary_form=False)
                    if not cert:
                        return None

                    subject = dict(x[0] for x in cert.get('subject', []))
                    issuer = dict(x[0] for x in cert.get('issuer', []))
                    not_after = cert.get('notAfter', '')

                    is_expired = False
                    days_until_expiry = 0
                    if not_after:
                        try:
                            expiry = datetime.strptime(not_after, '%b %d %H:%M:%S %Y %Z')
                            expiry = expiry.replace(tzinfo=timezone.utc)
                            delta = expiry - datetime.now(timezone.utc)
                            days_until_expiry = delta.days
                            is_expired = days_until_expiry < 0
                        except ValueError:
                            pass

                    san_list: list[str] = []
                    for san_type, san_value in cert.get('subjectAltName', []):
                        if san_type == 'DNS':
                            san_list.append(san_value)

                    return TLSInfo(
                        subject=subject.get('commonName', ''),
                        issuer=issuer.get('organizationName', issuer.get('commonName', '')),
                        not_before=cert.get('notBefore', ''),
                        not_after=not_after,
                        serial_number=str(cert.get('serialNumber', '')),
                        version=cert.get('version', 0),
                        san=san_list,
                        is_expired=is_expired,
                        days_until_expiry=days_until_expiry,
                    )
        except (ssl.SSLError, OSError, socket.timeout):
            return None

