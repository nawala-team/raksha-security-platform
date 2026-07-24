"""Async TCP port scanner with service fingerprinting."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Any

import structlog

logger = structlog.get_logger()

# Top 100 most common ports (subset of nmap top 1000)
TOP_PORTS: list[int] = [
    21, 22, 23, 25, 53, 80, 110, 111, 135, 139,
    143, 443, 445, 465, 587, 993, 995, 1433, 1434, 1521,
    1723, 2049, 2082, 2083, 2086, 2087, 3306, 3389, 5432, 5900,
    5985, 5986, 6379, 6443, 8000, 8008, 8080, 8443, 8880, 8888,
    9090, 9200, 9300, 9443, 10000, 11211, 27017, 27018, 28017, 50000,
]

# Well-known service names by port
SERVICE_MAP: dict[int, str] = {
    21: 'ftp', 22: 'ssh', 23: 'telnet', 25: 'smtp', 53: 'dns',
    80: 'http', 110: 'pop3', 111: 'rpcbind', 135: 'msrpc', 139: 'netbios',
    143: 'imap', 443: 'https', 445: 'smb', 465: 'smtps', 587: 'submission',
    993: 'imaps', 995: 'pop3s', 1433: 'mssql', 1434: 'mssql-udp',
    1521: 'oracle', 1723: 'pptp', 2049: 'nfs', 3306: 'mysql',
    3389: 'rdp', 5432: 'postgresql', 5900: 'vnc', 5985: 'winrm',
    5986: 'winrm-ssl', 6379: 'redis', 6443: 'kubernetes-api',
    8000: 'http-alt', 8008: 'http-alt', 8080: 'http-proxy',
    8443: 'https-alt', 8888: 'http-alt', 9090: 'prometheus',
    9200: 'elasticsearch', 9300: 'elasticsearch-transport',
    10000: 'webmin', 11211: 'memcached', 27017: 'mongodb',
    27018: 'mongodb', 50000: 'sap',
}


@dataclass
class OpenPort:
    """A discovered open port with service info."""

    ip: str
    port: int
    service: str
    banner: str
    state: str  # 'open', 'closed', 'filtered'

    def to_dict(self) -> dict[str, Any]:
        return {
            'ip': self.ip,
            'port': self.port,
            'service': self.service,
            'banner': self.banner,
            'state': self.state,
        }


class PortScanner:
    """Async TCP port scanner with banner grabbing."""

    def __init__(
        self,
        ports: list[int] | None = None,
        concurrency: int = 200,
        timeout: float = 2.0,
        banner_timeout: float = 3.0,
    ):
        self.ports = ports or TOP_PORTS
        self.concurrency = concurrency
        self.timeout = timeout
        self.banner_timeout = banner_timeout

    async def scan(self, target: str) -> list[OpenPort]:
        """Scan all configured ports on a target IP/hostname."""
        logger.info('asm_port_scan_start', target=target, ports=len(self.ports))
        semaphore = asyncio.Semaphore(self.concurrency)
        tasks = [self._scan_port(target, port, semaphore) for port in self.ports]
        results = await asyncio.gather(*tasks)

        open_ports = [r for r in results if r is not None and r.state == 'open']
        open_ports.sort(key=lambda p: p.port)

        logger.info('asm_port_scan_complete', target=target, open_ports=len(open_ports))
        return open_ports

    async def scan_multiple(self, targets: list[str]) -> dict[str, list[OpenPort]]:
        """Scan multiple targets and return results keyed by target."""
        results: dict[str, list[OpenPort]] = {}
        for target in targets:
            results[target] = await self.scan(target)
        return results

    async def _scan_port(
        self, target: str, port: int, semaphore: asyncio.Semaphore
    ) -> OpenPort | None:
        """Attempt TCP connection to a single port."""
        async with semaphore:
            try:
                reader, writer = await asyncio.wait_for(
                    asyncio.open_connection(target, port),
                    timeout=self.timeout,
                )
                # Connection succeeded - port is open
                banner = await self._grab_banner(target, port, reader, writer)
                service = self._identify_service(port, banner)
                writer.close()
                await writer.wait_closed()

                return OpenPort(
                    ip=target,
                    port=port,
                    service=service,
                    banner=banner,
                    state='open',
                )
            except asyncio.TimeoutError:
                return None
            except (ConnectionRefusedError, ConnectionResetError):
                return None
            except OSError:
                return None

    async def _grab_banner(
        self, target: str, port: int,
        reader: asyncio.StreamReader, writer: asyncio.StreamWriter,
    ) -> str:
        """Attempt to grab a service banner from the connection."""
        try:
            # For HTTP services, send a minimal request to elicit a response
            if port in (80, 8080, 8000, 8008, 8888, 443, 8443):
                writer.write(f'HEAD / HTTP/1.0\r\nHost: {target}\r\n\r\n'.encode())
                await writer.drain()

            # Try to read banner data with timeout
            data = await asyncio.wait_for(
                reader.read(512),
                timeout=self.banner_timeout,
            )
            if data:
                return data.decode('utf-8', errors='replace').strip()
        except (asyncio.TimeoutError, OSError, ConnectionResetError):
            pass
        return ''

    def _identify_service(self, port: int, banner: str) -> str:
        """Identify service from port number and banner content."""
        # Check banner patterns first
        if banner:
            banner_lower = banner.lower()
            if 'ssh' in banner_lower:
                return 'ssh'
            if 'http' in banner_lower:
                return 'http'
            if 'smtp' in banner_lower:
                return 'smtp'
            if 'ftp' in banner_lower:
                return 'ftp'
            if 'mysql' in banner_lower:
                return 'mysql'
            if 'postgresql' in banner_lower or 'postgres' in banner_lower:
                return 'postgresql'
            if 'redis' in banner_lower:
                return 'redis'
            if 'mongodb' in banner_lower or 'mongo' in banner_lower:
                return 'mongodb'
            if 'elasticsearch' in banner_lower:
                return 'elasticsearch'

        # Fall back to well-known port mapping
        return SERVICE_MAP.get(port, 'unknown')

