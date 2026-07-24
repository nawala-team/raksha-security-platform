"""MITRE ATT&CK framework feed integration."""

from typing import Any

import httpx
import structlog

logger = structlog.get_logger(__name__)

ATTACK_STIX_URL = "https://raw.githubusercontent.com/mitre/cti/master/enterprise-attack/enterprise-attack.json"


class MitreAttackFeed:
    """Fetches and parses MITRE ATT&CK framework data (STIX 2.1 format)."""

    def __init__(self):
        self.techniques: list[dict[str, Any]] = []
        self.tactics: list[dict[str, Any]] = []
        self.mitigations: list[dict[str, Any]] = []
        self.groups: list[dict[str, Any]] = []

    async def fetch(self) -> dict[str, list[dict[str, Any]]]:
        """Fetch full MITRE ATT&CK Enterprise matrix."""
        async with httpx.AsyncClient(timeout=120.0) as client:
            try:
                response = await client.get(ATTACK_STIX_URL)
                response.raise_for_status()
                bundle = response.json()
                return self._parse_stix_bundle(bundle)
            except Exception as e:
                logger.error("mitre_attack_fetch_failed", error=str(e))
                return {"techniques": [], "tactics": [], "mitigations": []}

    def _parse_stix_bundle(self, bundle: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
        """Parse STIX 2.1 bundle into categorized ATT&CK objects."""
        objects = bundle.get("objects", [])

        for obj in objects:
            obj_type = obj.get("type", "")
            if obj.get("revoked", False) or obj.get("x_mitre_deprecated", False):
                continue

            if obj_type == "attack-pattern":
                self.techniques.append(self._parse_technique(obj))
            elif obj_type == "x-mitre-tactic":
                self.tactics.append(self._parse_tactic(obj))
            elif obj_type == "course-of-action":
                self.mitigations.append(self._parse_mitigation(obj))
            elif obj_type == "intrusion-set":
                self.groups.append(self._parse_group(obj))

        logger.info(
            "mitre_attack_parsed",
            techniques=len(self.techniques),
            tactics=len(self.tactics),
            mitigations=len(self.mitigations),
            groups=len(self.groups),
        )
        return {
            "techniques": self.techniques,
            "tactics": self.tactics,
            "mitigations": self.mitigations,
            "groups": self.groups,
        }

    def _parse_technique(self, obj: dict[str, Any]) -> dict[str, Any]:
        """Parse an ATT&CK technique."""
        external_refs = obj.get("external_references", [])
        attack_id = next(
            (r["external_id"] for r in external_refs if r.get("source_name") == "mitre-attack"),
            None,
        )
        kill_chain = obj.get("kill_chain_phases", [])
        tactics = [p["phase_name"] for p in kill_chain]

        return {
            "id": attack_id,
            "name": obj.get("name", ""),
            "description": obj.get("description", "")[:500],
            "tactics": tactics,
            "platforms": obj.get("x_mitre_platforms", []),
            "detection": obj.get("x_mitre_detection", ""),
            "data_sources": obj.get("x_mitre_data_sources", []),
            "is_subtechnique": obj.get("x_mitre_is_subtechnique", False),
        }

    def _parse_tactic(self, obj: dict[str, Any]) -> dict[str, Any]:
        external_refs = obj.get("external_references", [])
        attack_id = next(
            (r["external_id"] for r in external_refs if r.get("source_name") == "mitre-attack"),
            None,
        )
        return {
            "id": attack_id,
            "name": obj.get("name", ""),
            "shortname": obj.get("x_mitre_shortname", ""),
            "description": obj.get("description", "")[:300],
        }

    def _parse_mitigation(self, obj: dict[str, Any]) -> dict[str, Any]:
        external_refs = obj.get("external_references", [])
        attack_id = next(
            (r["external_id"] for r in external_refs if r.get("source_name") == "mitre-attack"),
            None,
        )
        return {
            "id": attack_id,
            "name": obj.get("name", ""),
            "description": obj.get("description", "")[:300],
        }

    def _parse_group(self, obj: dict[str, Any]) -> dict[str, Any]:
        external_refs = obj.get("external_references", [])
        attack_id = next(
            (r["external_id"] for r in external_refs if r.get("source_name") == "mitre-attack"),
            None,
        )
        aliases = obj.get("aliases", [])
        return {
            "id": attack_id,
            "name": obj.get("name", ""),
            "aliases": aliases,
            "description": obj.get("description", "")[:300],
        }

    def get_technique_by_id(self, technique_id: str) -> dict[str, Any] | None:
        """Lookup a technique by ATT&CK ID (e.g., T1059)."""
        return next((t for t in self.techniques if t["id"] == technique_id), None)

    def get_techniques_for_tactic(self, tactic: str) -> list[dict[str, Any]]:
        """Get all techniques for a given tactic phase."""
        return [t for t in self.techniques if tactic in t["tactics"]]
