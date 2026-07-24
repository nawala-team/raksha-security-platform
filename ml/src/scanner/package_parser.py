from __future__ import annotations
import json
import re
from dataclasses import dataclass, field
from enum import Enum
from typing import Any
import structlog

logger = structlog.get_logger()

class PackageSource(str, Enum):
    DPKG = 'dpkg'
    RPM = 'rpm'
    PIP = 'pip'
    NPM = 'npm'
    CARGO = 'cargo'
    APK = 'apk'

@dataclass(frozen=True)
class Package:
    name: str
    version: str
    source: PackageSource
    architecture: str | None = field(default=None)
    epoch: int | None = field(default=None)
    release: str | None = field(default=None)
    def to_dict(self) -> dict[str, Any]:
        return dict(name=self.name, version=self.version, source=self.source.value,
                    architecture=self.architecture, epoch=self.epoch, release=self.release)

def parse_packages(raw_input: str, source: str) -> list[Package]:
    parsers = dict(dpkg=parse_dpkg, rpm=parse_rpm, pip=parse_pip,
                   npm=parse_npm, cargo=parse_cargo, apk=parse_apk)
    source_lower = source.lower().strip()
    if source_lower not in parsers:
        raise ValueError(f'Unsupported package source: {source}')
    packages = parsers[source_lower](raw_input)
    logger.info('packages_parsed', source=source_lower, count=len(packages))
    return packages
def parse_dpkg(raw: str) -> list[Package]:
    packages: list[Package] = []
    for line in raw.strip().splitlines():
        line = line.strip()
        if not line:
            continue
        if '\t' in line:
            parts = line.split('\t')
            if len(parts) >= 3:
                if 'install' in parts[0] or 'hold' in parts[0]:
                    name, version = parts[1].strip(), parts[2].strip()
                    arch = parts[3].strip() if len(parts) > 3 else None
                else:
                    name, version = parts[0].strip(), parts[1].strip()
                    arch = parts[2].strip() if len(parts) > 2 else None
                if name and version:
                    packages.append(Package(name=name, version=version,
                        source=PackageSource.DPKG, architecture=arch))
            continue
        match = re.match(r'^([a-z]{2})\s+(\S+)\s+(\S+)\s+(\S+)', line)
        if match:
            status, name, version, arch = match.groups()
            if status == 'ii':
                packages.append(Package(name=name, version=version,
                    source=PackageSource.DPKG, architecture=arch))
    return packages

def parse_rpm(raw: str) -> list[Package]:
    packages: list[Package] = []
    for line in raw.strip().splitlines():
        line = line.strip()
        if not line:
            continue
        if '\t' in line:
            parts = line.split('\t')
            if len(parts) >= 2:
                name, version = parts[0].strip(), parts[1].strip()
                release = parts[2].strip() if len(parts) > 2 else None
                arch = parts[3].strip() if len(parts) > 3 else None
                epoch_str = parts[4].strip() if len(parts) > 4 else None
                epoch = None
                if epoch_str and epoch_str not in ('(none)', '0'):
                    try:
                        epoch = int(epoch_str)
                    except ValueError:
                        pass
                if name and version:
                    packages.append(Package(name=name, version=version,
                        source=PackageSource.RPM, architecture=arch,
                        epoch=epoch, release=release))
            continue
        match = re.match(r'^(.+)-([^-]+)-([^-]+)\.(\w+)$', line)
        if match:
            name, version, release, arch = match.groups()
            packages.append(Package(name=name, version=version,
                source=PackageSource.RPM, architecture=arch, release=release))
    return packages
def parse_pip(raw: str) -> list[Package]:
    packages: list[Package] = []
    stripped = raw.strip()
    if stripped.startswith('['):
        try:
            data = json.loads(stripped)
            for item in data:
                name = item.get('name', '').strip()
                version = item.get('version', '').strip()
                if name and version:
                    packages.append(Package(name=name, version=version, source=PackageSource.PIP))
            return packages
        except json.JSONDecodeError:
            pass
    for line in stripped.splitlines():
        line = line.strip()
        if not line or line.startswith('#') or line.startswith('-'):
            continue
        if line.startswith('Package') or line.startswith('---'):
            continue
        if '==' in line:
            match = re.match(r'^([a-zA-Z0-9._-]+)==(.+)$', line)
            if match:
                packages.append(Package(name=match.group(1), version=match.group(2), source=PackageSource.PIP))
            continue
        parts = line.split()
        if len(parts) >= 2 and re.match(r'^[\d.]', parts[1]):
            packages.append(Package(name=parts[0], version=parts[1], source=PackageSource.PIP))
    return packages

def parse_npm(raw: str) -> list[Package]:
    packages: list[Package] = []
    stripped = raw.strip()
    if stripped.startswith('{'):
        try:
            data = json.loads(stripped)
            if 'dependencies' in data:
                _extract_npm_deps(data['dependencies'], packages)
                return packages
            if 'packages' in data:
                for pkg_path, info in data['packages'].items():
                    if not pkg_path:
                        continue
                    name = pkg_path.split('node_modules/')[-1] if 'node_modules/' in pkg_path else pkg_path
                    version = info.get('version', '').strip()
                    if name and version:
                        packages.append(Package(name=name, version=version, source=PackageSource.NPM))
                return packages
        except json.JSONDecodeError:
            pass
    for line in stripped.splitlines():
        cleaned = re.sub(r'^[\u2502\u251c\u2514\u2500\u252c\s]+', '', line).strip()
        if not cleaned:
            continue
        match = re.match(r'^(@?[a-zA-Z0-9._/-]+)@(\S+)', cleaned)
        if match:
            name, version = match.groups()
            version = version.split(' ')[0]
            packages.append(Package(name=name, version=version, source=PackageSource.NPM))
    return packages

def _extract_npm_deps(deps: dict[str, Any], packages: list[Package]) -> None:
    for name, info in deps.items():
        if isinstance(info, dict):
            version = info.get('version', '').strip()
            if name and version:
                packages.append(Package(name=name, version=version, source=PackageSource.NPM))
            if 'dependencies' in info:
                _extract_npm_deps(info['dependencies'], packages)
def parse_cargo(raw: str) -> list[Package]:
    packages: list[Package] = []
    stripped = raw.strip()
    if stripped.startswith('{'):
        try:
            data = json.loads(stripped)
            for pkg in data.get('packages', []):
                name = pkg.get('name', '').strip()
                version = pkg.get('version', '').strip()
                if name and version:
                    packages.append(Package(name=name, version=version, source=PackageSource.CARGO))
            return packages
        except json.JSONDecodeError:
            pass
    if '[[package]]' in stripped:
        current_name: str | None = None
        current_version: str | None = None
        for line in stripped.splitlines():
            line = line.strip()
            if line == '[[package]]':
                if current_name and current_version:
                    packages.append(Package(name=current_name, version=current_version, source=PackageSource.CARGO))
                current_name = None
                current_version = None
                continue
            m_name = re.match(r'^name\s*=\s*"(.+)"$', line)
            if m_name:
                current_name = m_name.group(1)
                continue
            m_ver = re.match(r'^version\s*=\s*"(.+)"$', line)
            if m_ver:
                current_version = m_ver.group(1)
        if current_name and current_version:
            packages.append(Package(name=current_name, version=current_version, source=PackageSource.CARGO))
        return packages
    for line in stripped.splitlines():
        cleaned = re.sub(r'^[\u2502\u251c\u2514\u2500\u252c\s]+', '', line).strip()
        if not cleaned:
            continue
        match = re.match(r'^([a-zA-Z0-9_-]+)\s+v([\d.]+\S*)', cleaned)
        if match:
            name, version = match.groups()
            packages.append(Package(name=name, version=version, source=PackageSource.CARGO))
    return packages

def parse_apk(raw: str) -> list[Package]:
    packages: list[Package] = []
    for line in raw.strip().splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        pkg_str = parts[0]
        arch = parts[1] if len(parts) > 1 and not parts[1].startswith('{') else None
        match = re.match(r'^(.+?)-(\d[^-]*(?:-r\d+)?)$', pkg_str)
        if match:
            name, version = match.groups()
            packages.append(Package(name=name, version=version, source=PackageSource.APK, architecture=arch))
    return packages