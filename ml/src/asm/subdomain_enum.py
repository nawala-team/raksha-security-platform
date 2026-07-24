"""Subdomain enumeration via DNS brute-force and Certificate Transparency logs."""

from __future__ import annotations

import asyncio
import socket
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

import httpx
import structlog

logger = structlog.get_logger()

# Common subdomain prefixes for brute-force enumeration
DEFAULT_WORDLIST: list[str] = [
    'www', 'mail', 'ftp', 'smtp', 'pop', 'imap', 'webmail', 'ns1', 'ns2',
    'dns', 'dns1', 'dns2', 'mx', 'mx1', 'mx2', 'vpn', 'remote', 'gateway',
    'admin', 'portal', 'api', 'dev', 'staging', 'test', 'beta', 'demo',
    'app', 'apps', 'cloud', 'cdn', 'static', 'assets', 'media', 'img',
    'images', 'docs', 'wiki', 'blog', 'shop', 'store', 'pay', 'payment',
    'secure', 'login', 'auth', 'sso', 'id', 'account', 'accounts',
    'dashboard', 'panel', 'cp', 'cpanel', 'whm', 'plesk',
    'db', 'database', 'mysql', 'postgres', 'redis', 'mongo', 'elastic',
    'jenkins', 'ci', 'cd', 'build', 'deploy', 'git', 'gitlab', 'github',
    'jira', 'confluence', 'slack', 'teams', 'meet', 'zoom',
    'monitoring', 'grafana', 'prometheus', 'kibana', 'nagios', 'zabbix',
    'backup', 'bak', 'old', 'legacy', 'archive', 'temp', 'tmp',
    'internal', 'intranet', 'extranet', 'corp', 'office',
    'status', 'health', 'metrics', 'telemetry', 'logs',
    'm', 'mobile', 'wap', 'api2', 'api3', 'v1', 'v2',
    'proxy', 'cache', 'lb', 'load', 'node1', 'node2', 'node3',
    'web', 'web1', 'web2', 'srv', 'server', 'host', 'vps',
    's3', 'storage', 'bucket', 'files', 'download', 'upload',
    'exchange', 'autodiscover', 'owa', 'outlook',
    'support', 'help', 'helpdesk', 'ticket', 'service', 'services',
]


@dataclass
class DiscoveredSubdomain:
    """A discovered subdomain with metadata."""

    domain: str
    ip: str | None
    source: str  # 'dns_bruteforce', 'crt_sh', 'dns_records'
    first_seen: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    record_type: str | None = None
    raw_records: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            'domain': self.domain,
            'ip': self.ip,
            'source': self.source,
            'first_seen': self.first_seen,
            'record_type': self.record_type,
            'raw_records': self.raw_records,
        }


class SubdomainEnumerator:
    """Enumerate subdomains using multiple discovery techniques."""

    def __init__(
        self,
        wordlist: list[str] | None = None,
        concurrency: int = 50,
        timeout: float = 3.0,
    ):
        self.wordlist = wordlist or DEFAULT_WORDLIST
        self.concurrency = concurrency
        self.timeout = timeout

    async def enumerate(self, domain: str) -> list[DiscoveredSubdomain]:
        """Run all enumeration methods and return deduplicated results."""
        logger.info('asm_subdomain_enum_start', domain=domain)

        results: list[DiscoveredSubdomain] = []

        dns_task = self._dns_bruteforce(domain)
        ct_task = self._query_crt_sh(domain)
        records_task = self._lookup_dns_records(domain)

        dns_results, ct_results, record_results = await asyncio.gather(
            dns_task, ct_task, records_task, return_exceptions=True
        )

        if isinstance(dns_results, list):
            results.extend(dns_results)
        else:
            logger.warning('asm_dns_bruteforce_error', error=str(dns_results))

        if isinstance(ct_results, list):
            results.extend(ct_results)
        else:
            logger.warning('asm_crt_sh_error', error=str(ct_results))

        if isinstance(record_results, list):
            results.extend(record_results)
        else:
            logger.warning('asm_dns_records_error', error=str(record_results))

        deduplicated = self._deduplicate(results)
        logger.info('asm_subdomain_enum_complete', domain=domain, total_found=len(deduplicated))
        return deduplicated

    async def _dns_bruteforce(self, domain: str) -> list[DiscoveredSubdomain]:
        """Brute-force subdomains by resolving common prefixes."""
        results: list[DiscoveredSubdomain] = []
        semaphore = asyncio.Semaphore(self.concurrency)

        async def resolve_one(prefix: str) -> DiscoveredSubdomain | None:
            fqdn = f'{prefix}.{domain}'
            async with semaphore:
                try:
                    loop = asyncio.get_event_loop()
                    ip = await asyncio.wait_for(
                        loop.run_in_executor(None, socket.gethostbyname, fqdn),
                        timeout=self.timeout,
                    )
                    return DiscoveredSubdomain(
                        domain=fqdn, ip=ip, source='dns_bruteforce', record_type='A',
                    )
                except (socket.gaierror, asyncio.TimeoutError, OSError):
                    return None

        tasks = [resolve_one(prefix) for prefix in self.wordlist]
        resolved = await asyncio.gather(*tasks)
        for result in resolved:
            if result is not None:
                results.append(result)
        return results

    async def _query_crt_sh(self, domain: str) -> list[DiscoveredSubdomain]:
        """Query Certificate Transparency logs via crt.sh."""
        results: list[DiscoveredSubdomain] = []
        url = f'https://crt.sh/?q=%.{domain}&output=json'

        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                response = await client.get(url)
                response.raise_for_status()
                entries = response.json()
        except (httpx.HTTPError, ValueError) as e:
            logger.warning('asm_crt_sh_fetch_failed', domain=domain, error=str(e))
            return results

        seen_names: set[str] = set()
        for entry in entries:
            name_value = entry.get('name_value', '')
            for name in name_value.split('\n'):
                name = name.strip().lower()
                if name.startswith('*.'):
                    name = name[2:]
                if not name or name in seen_names:
                    continue
                if not name.endswith(f'.{domain}') and name != domain:
                    continue
                seen_names.add(name)
                ip = await self._resolve_ip(name)
                results.append(DiscoveredSubdomain(
                    domain=name, ip=ip, source='crt_sh',
                ))
        return results

    async def _lookup_dns_records(self, domain: str) -> list[DiscoveredSubdomain]:
        """Perform DNS record lookups for the base domain."""
        results: list[DiscoveredSubdomain] = []
        loop = asyncio.get_event_loop()

        # A record
        try:
            ip = await asyncio.wait_for(
                loop.run_in_executor(None, socket.gethostbyname, domain),
                timeout=self.timeout,
            )
            results.append(DiscoveredSubdomain(
                domain=domain, ip=ip, source='dns_records', record_type='A',
            ))
        except (socket.gaierror, asyncio.TimeoutError, OSError):
            pass

        # AAAA record
        try:
            infos = await asyncio.wait_for(
                loop.run_in_executor(
                    None, lambda: socket.getaddrinfo(domain, None, socket.AF_INET6),
                ),
                timeout=self.timeout,
            )
            if infos:
                ip6 = infos[0][4][0]
                results.append(DiscoveredSubdomain(
                    domain=domain, ip=ip6, source='dns_records', record_type='AAAA',
                ))
        except (socket.gaierror, asyncio.TimeoutError, OSError):
            pass

        return results

    async def _resolve_ip(self, hostname: str) -> str | None:
        """Resolve a hostname to its IP address."""
        try:
            loop = asyncio.get_event_loop()
            return await asyncio.wait_for(
                loop.run_in_executor(None, socket.gethostbyname, hostname),
                timeout=self.timeout,
            )
        except (socket.gaierror, asyncio.TimeoutError, OSError):
            return None

    def _deduplicate(self, results: list[DiscoveredSubdomain]) -> list[DiscoveredSubdomain]:
        """Deduplicate results, keeping the first occurrence per domain."""
        seen: dict[str, DiscoveredSubdomain] = {}
        for result in results:
            key = result.domain.lower()
            if key not in seen:
                seen[key] = result
            else:
                existing = seen[key]
                if not existing.ip and result.ip:
                    existing.ip = result.ip
                if result.raw_records:
                    existing.raw_records.extend(result.raw_records)
        return list(seen.values())
