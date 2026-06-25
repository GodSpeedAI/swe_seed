"""Integration test: coherence break -> learning proposal -> Context Kernel surfaces it.

Verifies the full GodSpeed-Agent -> CK feedback loop:
1. A coherence break is detected and a LearningProposalCreated event is emitted.
2. The proposal is written to the JSONL ledger.
3. Context Kernel's Python adapter, when asked for 'learning-proposals' corpus,
   returns the proposal as a citation in ContextPacketCreated.
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

from workspace_roots import CONTEXT_KERNEL_ROOT, GODSPEED_AGENT_ROOT


def _load_module(name: str, path: Path, *, package_root: Path | None = None):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    inserted_by_us = False
    if package_root and str(package_root) not in sys.path:
        sys.path.insert(0, str(package_root))
        inserted_by_us = True
    try:
        spec.loader.exec_module(module)
    finally:
        if inserted_by_us:
            sys.path.remove(str(package_root))
    return module


def test_learning_proposals_surface_in_future_context_packet(tmp_path: Path) -> None:
    """Failed work -> LearningProposal written -> CK returns it as a citation."""
    gs_adapters = _load_module(
        "gs_adapters_feedback",
        GODSPEED_AGENT_ROOT / "agentic_capability_loop/adapters.py",
        package_root=GODSPEED_AGENT_ROOT,
    )
    learning_proposals_mod = _load_module(
        "learning_proposals_feedback",
        GODSPEED_AGENT_ROOT / "agentic_capability_loop/learning_proposals.py",
        package_root=GODSPEED_AGENT_ROOT,
    )
    ck_adapter_path = CONTEXT_KERNEL_ROOT / "integration/agentic_capability_loop/adapters.py"
    if not ck_adapter_path.exists():
        pytest.skip("CK integration adapter not found")
    ck_adapters = _load_module(
        "ck_adapters_feedback",
        ck_adapter_path,
        package_root=CONTEXT_KERNEL_ROOT,
    )

    # Step 1: detect coherence break and write proposal to ledger
    ledger_path = tmp_path / "agentic_learning_proposals.jsonl"
    _, proposal = gs_adapters.detect_coherence_break(
        "Missing context packet for agentic loop work",
        "wr-feedback-test",
        domain_area="agentic_capability_loop",
        evidence_ref="proof-feedback-1",
    )

    ledger = learning_proposals_mod.LearningProposalLedger(tmp_path)
    ledger.append(proposal)
    assert ledger_path.exists()
    assert len(ledger.read_all()) == 1

    # Step 2: future ContextRequired for learning-proposals corpus
    context_required_event = {
        "event_id": "evt-feedback-ctx-req",
        "event_type": "ContextRequired",
        "namespace": "agentic_capability_loop",
        "occurred_at": "2026-01-02T00:00:00+00:00",
        "payload": {
            "domain_model_hash": gs_adapters.DOMAIN_MODEL_HASH,
            "work_request_id": "wr-feedback-test-2",
            "context_requirement_id": "ctx-req-feedback",
            "corpus_id": "learning-proposals:agentic_capability_loop",
            "query": "context packet",
            "max_results": 10,
            "is_private": False,
            "scope_claim": None,
            "learning_proposals_ledger_path": str(ledger_path),
        },
    }
    context_packet, context_authorized = ck_adapters.handle_context_required(context_required_event)

    # Step 3: verify proposal appears as a citation
    assert context_packet["event_type"] == "ContextPacketCreated"
    citations = context_packet["payload"]["citations"]
    assert len(citations) >= 1, (
        "Expected at least 1 citation from the learning-proposals ledger, got 0"
    )
    assert any("context packet" in c["content"].lower() for c in citations), (
        f"No citation matched 'context packet' query. Citations: {citations}"
    )
    assert context_authorized["payload"]["result"] == "allow"
