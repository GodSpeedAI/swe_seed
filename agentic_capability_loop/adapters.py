"""SWE_SEED event adapters for the agentic_capability_loop domain.

SWE_SEED emits:
  - WorkRequested
  - ContextRequired
  - RouteSelected
  - ProofStarted
  - ProofCompleted

SWE_SEED consumes:
  - ContextPacketCreated (from Context Kernel)
  - AuthorityChecked (from SEA-Forge)
  - SettlementRecorded (from GodSpeed-Agent)
"""
from __future__ import annotations

import datetime
import hashlib
import json as _json
import os
import uuid
import warnings
from pathlib import Path as _Path
from typing import Any

NAMESPACE = "agentic_capability_loop"


def _load_hash() -> str:
    """Resolve the SEA manifest and return sea_file_hash.

    Search order:
    1. SEA_ROOT env var pointing at the SEA repo.
    2. SEA_MANIFEST_PATH env var pointing at the manifest file.
    3. Walk parent directories looking for repo markers.
    4. Legacy relative path as last resort.
    """
    manifest_path: _Path | None = None

    sea_root = os.environ.get("SEA_ROOT")
    if sea_root:
        candidate = (
            _Path(sea_root)
            / "docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json"
        )
        if candidate.exists():
            manifest_path = candidate

    if manifest_path is None:
        env_manifest = os.environ.get("SEA_MANIFEST_PATH")
        if env_manifest:
            candidate = _Path(env_manifest)
            if candidate.exists():
                manifest_path = candidate

    if manifest_path is None:
        markers = ["tools/sea_parse.py", "docs/specs"]
        for parent in _Path(__file__).resolve().parents:
            candidate_dir = parent / "SEA"
            if candidate_dir.is_dir() and any((candidate_dir / m).exists() for m in markers):
                manifest_path = (
                    candidate_dir
                    / "docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json"
                )
                if not manifest_path.exists():
                    manifest_path = None
                break
            if any((parent / m).exists() for m in markers) and (parent / "libs").is_dir():
                manifest_path = (
                    parent
                    / "docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json"
                )
                if not manifest_path.exists():
                    manifest_path = None
                break

    if manifest_path is None:
        manifest_path = (
            _Path(__file__).resolve().parents[2]
            / "SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json"
        )

    if manifest_path.exists():
        _m = _json.loads(manifest_path.read_text(encoding="utf-8"))
        _h = (_m.get("meta") or {}).get("sea_file_hash")
        if _h:
            return _h
    warnings.warn("SEA manifest not found; using fallback hash", stacklevel=2)
    return hashlib.sha256(b"agentic_capability_loop").hexdigest()


DOMAIN_MODEL_HASH: str = _load_hash()


def _now_iso() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def _event(event_type: str, payload: dict[str, Any]) -> dict[str, Any]:
    """Build a canonical event envelope."""
    payload = {"domain_model_hash": DOMAIN_MODEL_HASH, **payload}
    return {
        "event_id": str(uuid.uuid4()),
        "event_type": event_type,
        "namespace": NAMESPACE,
        "occurred_at": _now_iso(),
        "payload": payload,
    }


def emit_work_requested(
    work_request_id: str,
    actor_id: str,
    operation: str,
    resource: str,
    *,
    risk_level: str = "low",
    repo: str | None = None,
) -> dict[str, Any]:
    """Emit WorkRequested event when SWE_SEED receives a new task.
    
    Args:
        work_request_id: Unique ID for this work request.
        actor_id: ID of the agent or human initiating the request.
        operation: High-level description of the operation (e.g. 'implement feature').
        resource: Target resource identifier (e.g. file path, issue ID).
        risk_level: 'low' | 'medium' | 'high'. Affects policy evaluation.
        repo: Optional repository name/URL.
    
    Returns:
        Canonical event envelope with WorkRequested payload.
    """
    return _event(
        "WorkRequested",
        {
            "work_request_id": work_request_id,
            "actor_id": actor_id,
            "operation": operation,
            "resource": resource,
            "risk_level": risk_level,
            "repo": repo,
        },
    )


def emit_context_required(
    work_request_id: str,
    context_requirement_id: str,
    corpus_id: str,
    *,
    query: str | None = None,
    max_results: int = 10,
    is_private: bool = False,
    scope_claim: str | None = None,
) -> dict[str, Any]:
    """Emit ContextRequired event when SWE_SEED needs context from Context Kernel.
    
    Args:
        work_request_id: ID of the originating work request.
        context_requirement_id: Unique ID for this context requirement.
        corpus_id: ID of the knowledge corpus to query.
        query: Optional natural-language query string.
        max_results: Maximum number of citations to return.
        is_private: True if the corpus is private-scoped.
        scope_claim: Required if is_private=True; authorization scope token.
    
    Returns:
        Canonical event envelope with ContextRequired payload.
    """
    return _event(
        "ContextRequired",
        {
            "work_request_id": work_request_id,
            "context_requirement_id": context_requirement_id,
            "corpus_id": corpus_id,
            "query": query,
            "max_results": max_results,
            "is_private": is_private,
            "scope_claim": scope_claim,
        },
    )


def emit_route_selected(
    work_request_id: str,
    route_id: str,
    route_name: str,
    proof_command_id: str,
    *,
    selected_by: str | None = None,
) -> dict[str, Any]:
    """Emit RouteSelected event when SWE_SEED selects a harness route card.
    
    Args:
        work_request_id: ID of the originating work request.
        route_id: ID of the selected route card.
        route_name: Human-readable route name.
        proof_command_id: ID of the proof command to run.
        selected_by: Optional ID of the agent/human that selected the route.
    
    Returns:
        Canonical event envelope with RouteSelected payload.
    """
    return _event(
        "RouteSelected",
        {
            "work_request_id": work_request_id,
            "route_id": route_id,
            "route_name": route_name,
            "proof_command_id": proof_command_id,
            "selected_by": selected_by,
        },
    )


def emit_proof_started(
    proof_result_id: str,
    work_request_id: str,
    route_id: str,
    *,
    proof_command: str | None = None,
) -> dict[str, Any]:
    """Emit ProofStarted event when SWE_SEED begins running the proof command.
    
    Args:
        proof_result_id: Unique ID for this proof run.
        work_request_id: ID of the originating work request.
        route_id: ID of the route being proved.
        proof_command: The exact command being executed.
    
    Returns:
        Canonical event envelope with ProofStarted payload.
    """
    return _event(
        "ProofStarted",
        {
            "proof_result_id": proof_result_id,
            "work_request_id": work_request_id,
            "route_id": route_id,
            "proof_command": proof_command,
            "started_at": _now_iso(),
        },
    )


def emit_proof_completed(
    proof_result_id: str,
    work_request_id: str,
    result: str,
    *,
    exit_code: int | None = None,
    output_ref: str | None = None,
    proof_type: str = "live",
) -> dict[str, Any]:
    """Emit ProofCompleted event when the proof command finishes.
    
    Args:
        proof_result_id: ID of this proof run (matches ProofStarted).
        work_request_id: ID of the originating work request.
        result: 'pass' | 'fail'.
        exit_code: Process exit code.
        output_ref: Reference to captured output (file path or log ID).
        proof_type: 'live' | 'simulation' (simulation results cannot promote capabilities).
    
    Returns:
        Canonical event envelope with ProofCompleted payload.
    """
    return _event(
        "ProofCompleted",
        {
            "proof_result_id": proof_result_id,
            "work_request_id": work_request_id,
            "result": result,
            "exit_code": exit_code,
            "output_ref": output_ref,
            "proof_type": proof_type,
            "completed_at": _now_iso(),
        },
    )


def consume_context_packet_created(event: dict[str, Any]) -> dict[str, Any]:
    """Consume ContextPacketCreated event from Context Kernel.
    
    Validates the event type and returns the context packet payload.
    The payload will contain a list of citations for use in the current session.
    
    Args:
        event: Canonical event envelope.
    
    Returns:
        Context packet payload with citations.
    
    Raises:
        ValueError: If the event type does not match or payload is missing.
    """
    if event.get("event_type") != "ContextPacketCreated":
        raise ValueError(
            f"Expected ContextPacketCreated event, got '{event.get('event_type')}'"
        )
    payload = event.get("payload")
    if payload is None:
        raise ValueError("Missing payload in ContextPacketCreated event")
    return payload


def consume_authority_checked(event: dict[str, Any]) -> dict[str, Any]:
    """Consume AuthorityChecked event from SEA-Forge.
    
    Returns the authority decision payload. The 'result' field is 'allow' | 'deny' | 'escalate'.
    
    Args:
        event: Canonical event envelope.
    
    Returns:
        Authority decision payload.
    
    Raises:
        ValueError: If the event type does not match or payload is missing.
    """
    if event.get("event_type") != "AuthorityChecked":
        raise ValueError(
            f"Expected AuthorityChecked event, got '{event.get('event_type')}'"
        )
    payload = event.get("payload")
    if payload is None:
        raise ValueError("Missing payload in AuthorityChecked event")
    return payload


def consume_settlement_recorded(event: dict[str, Any]) -> dict[str, Any]:
    """Consume SettlementRecorded event from GodSpeed-Agent.
    
    Returns the settlement payload. Use this to check the final outcome
    classification for the work request.
    
    Args:
        event: Canonical event envelope.
    
    Returns:
        Settlement payload.
    
    Raises:
        ValueError: If the event type does not match or payload is missing.
    """
    if event.get("event_type") != "SettlementRecorded":
        raise ValueError(
            f"Expected SettlementRecorded event, got '{event.get('event_type')}'"
        )
    payload = event.get("payload")
    if payload is None:
        raise ValueError("Missing payload in SettlementRecorded event")
    return payload
