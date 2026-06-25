from __future__ import annotations

import importlib.util
import json
import re
import subprocess
import sys
from contextlib import contextmanager
from pathlib import Path
from typing import Generator

import pytest

from workspace_roots import CONTEXT_KERNEL_ROOT, GODSPEED_AGENT_ROOT, SEA_ROOT, SWE_SEED_ROOT


DOMAIN_DIR = SEA_ROOT / "docs/specs/domains/agentic_capability_loop"
_PKG = "agentic_capability_loop"


def _load_module(name: str, path: Path, *, package_root: Path | None = None, package: str | None = None):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)  # type: ignore[arg-type]
    assert spec.loader is not None
    if package:
        module.__package__ = package
    if package_root is None:
        spec.loader.exec_module(module)
    else:
        inserted_by_us = str(package_root) not in sys.path
        if inserted_by_us:
            sys.path.insert(0, str(package_root))
        try:
            spec.loader.exec_module(module)
        finally:
            if inserted_by_us:
                sys.path.remove(str(package_root))
    return module


def _ensure_sea_package_loaded() -> None:
    """Pre-load domain_model into sys.modules so adapters.py relative imports work."""
    if f"{_PKG}.domain_model" in sys.modules:
        return
    dm_path = SEA_ROOT / "libs/agentic_capability_loop/domain_model.py"
    if not dm_path.exists():
        return
    spec = importlib.util.spec_from_file_location(f"{_PKG}.domain_model", dm_path)
    mod = importlib.util.module_from_spec(spec)  # type: ignore[arg-type]
    mod.__package__ = _PKG
    sys.modules[f"{_PKG}.domain_model"] = mod
    assert spec.loader is not None
    spec.loader.exec_module(mod)


def _register_sea_module(key: str, path: Path, package: str, is_pkg: bool = False) -> None:
    """Load a SEA module into sys.modules with a proper __package__."""
    if key in sys.modules:
        return
    spec = importlib.util.spec_from_file_location(key, path)
    mod = importlib.util.module_from_spec(spec)  # type: ignore[arg-type]
    mod.__package__ = package
    if is_pkg:
        mod.__path__ = [str(path.parent)]
    sys.modules[key] = mod
    assert spec is not None and spec.loader is not None
    spec.loader.exec_module(mod)


@contextmanager
def _sea_authority_deps() -> Generator[None, None, None]:
    """Temporarily install SEA's domain_model, adapters, and policies into sys.modules.

    This lets authority_service.py's try-block relative imports resolve to SEA's
    versions (which have emit_authority_checked), instead of SWE_SEED's (which don't).
    Original sys.modules entries are restored on exit.
    """
    keys = [
        _PKG,
        f"{_PKG}.domain_model",
        f"{_PKG}.adapters",
        f"{_PKG}.policies",
        f"{_PKG}.policies.{_PKG}",
    ]
    saved = {k: sys.modules.pop(k, _SENTINEL) for k in keys}

    _register_sea_module(_PKG, SEA_ROOT / f"libs/{_PKG}/__init__.py", _PKG, is_pkg=True)
    _register_sea_module(f"{_PKG}.domain_model", SEA_ROOT / f"libs/{_PKG}/domain_model.py", _PKG)
    _register_sea_module(f"{_PKG}.adapters", SEA_ROOT / f"libs/{_PKG}/adapters.py", _PKG)
    _register_sea_module(f"{_PKG}.policies", SEA_ROOT / f"libs/{_PKG}/policies/__init__.py", _PKG, is_pkg=True)
    _register_sea_module(f"{_PKG}.policies.{_PKG}", SEA_ROOT / f"libs/{_PKG}/policies/{_PKG}.py", f"{_PKG}.policies")

    try:
        yield
    finally:
        for k in keys:
            if saved[k] is not _SENTINEL:
                sys.modules[k] = saved[k]
            elif k in sys.modules:
                del sys.modules[k]


_SENTINEL = object()


_ensure_sea_package_loaded()
sea_adapters = _load_module(
    "sea_acl_adapters_hardening",
    SEA_ROOT / "libs/agentic_capability_loop/adapters.py",
    package="agentic_capability_loop",
)
sea_policies = _load_module(
    "sea_acl_policies_hardening",
    SEA_ROOT / "libs/agentic_capability_loop/policies/agentic_capability_loop.py",
)
gs_adapters = _load_module(
    "gs_acl_adapters_hardening",
    GODSPEED_AGENT_ROOT / "agentic_capability_loop/adapters.py",
)

# Pre-load learning_proposals into sys.modules so persist_to_ledger's
# lazy import `from agentic_capability_loop.learning_proposals import ...` works.
_lp_key = "agentic_capability_loop.learning_proposals"
if _lp_key not in sys.modules:
    _lp_spec = importlib.util.spec_from_file_location(
        _lp_key,
        GODSPEED_AGENT_ROOT / "agentic_capability_loop/learning_proposals.py",
    )
    _lp_mod = importlib.util.module_from_spec(_lp_spec)  # type: ignore[arg-type]
    _lp_mod.__package__ = "agentic_capability_loop"
    sys.modules[_lp_key] = _lp_mod
    assert _lp_spec is not None and _lp_spec.loader is not None
    _lp_spec.loader.exec_module(_lp_mod)


def test_sea_policy_blocks_include_structured_semantic_annotations() -> None:
    sea_source = (DOMAIN_DIR / "agentic_capability_loop.sea").read_text(encoding="utf-8")
    expected_policy_names = [
        "HighRiskWorkRequiresContextPacket",
        "PrivateCorpusRequiresExplicitScope",
        "ContextPacketRequiresCitation",
        "StaleSourceRequiresWarningOrRefresh",
        "ActionRequiresAuthorityDecision",
        "CompletionRequiresFreshProof",
        "SettlementRequiresEvidence",
        "CapabilityPromotionRequiresRepeatedSettlement",
        "SimulationCannotPromoteCapability",
        "HarnessMutationRequiresHumanApproval",
    ]

    for policy_name in expected_policy_names:
        marker = f"Policy {policy_name} "
        start = sea_source.index(marker)
        next_policy = sea_source.find("\nPolicy ", start + len(marker))
        block = sea_source[start:] if next_policy == -1 else sea_source[start:next_policy]
        for annotation in ("input_events", "required_fields", "allow_when", "deny_when", "failure_event"):
            assert f"// {annotation}:" in block


def test_authority_service_emits_authority_checked_and_coherence_break() -> None:
    with _sea_authority_deps():
        authority_service = _load_module(
            "sea_acl_authority_service_hardening",
            SEA_ROOT / "libs/agentic_capability_loop/authority_service.py",
            package="agentic_capability_loop",
        )
    service = authority_service.AuthorityService()

    denied = service.evaluate_work_request(
        {"work_request_id": "wr-high", "risk_level": "high", "actor_id": "agent-1"},
        context_packet=None,
    )

    authority_event = denied["authority_event"]
    coherence_event = denied["coherence_break_event"]
    assert authority_event["event_type"] == "AuthorityChecked"
    assert authority_event["payload"]["result"] == "deny"
    assert authority_event["payload"]["policy_code"] == "POL-ACL-001"
    assert coherence_event["event_type"] == "CoherenceBreakDetected"
    assert coherence_event["payload"]["work_request_id"] == "wr-high"

    allowed = service.evaluate_work_request(
        {"work_request_id": "wr-low", "risk_level": "low", "actor_id": "agent-1"},
        context_packet=None,
    )
    assert allowed["authority_event"]["payload"]["result"] == "allow"
    assert allowed["coherence_break_event"] is None


@pytest.mark.live_proof
def test_live_swe_seed_harness_validate_emits_proof_completed_shape() -> None:
    result = subprocess.run(
        [sys.executable, str(SWE_SEED_ROOT / "scripts/harness.py"), "validate"],
        cwd=SWE_SEED_ROOT,
        text=True,
        capture_output=True,
        check=False,
        timeout=120,
    )

    proof_completed = __import__("agentic_capability_loop.adapters", fromlist=["emit_proof_completed"]).emit_proof_completed(
        proof_result_id="proof-live-validate",
        work_request_id="wr-live-validate",
        result="pass" if result.returncode == 0 else "fail",
        exit_code=result.returncode,
        output_ref="scripts/harness.py validate",
        proof_type="live",
    )

    assert result.returncode == 0, result.stdout + result.stderr
    assert proof_completed["event_type"] == "ProofCompleted"
    assert proof_completed["payload"]["proof_type"] == "live"
    assert proof_completed["payload"]["exit_code"] == 0


def test_godspeed_persists_agentic_loop_records_to_real_ledger(tmp_path: Path) -> None:
    storage = _load_module(
        "godspeed_storage_hardening",
        GODSPEED_AGENT_ROOT / "godspeed_nav/storage.py",
        package_root=GODSPEED_AGENT_ROOT,
    )
    ledger_store = storage.LedgerStore(tmp_path)
    settlement = gs_adapters.emit_settlement_recorded("wr-ledger", "proof-1", "evidence-1", "usable", "matched")
    capability = gs_adapters.update_capability_ledger(settlement, capability_name="hardening", settlement_count=1)
    repetition = gs_adapters.schedule_repetition(capability["payload"]["capability_id"])

    persisted = gs_adapters.persist_to_ledger(settlement, capability, repetition, ledger_store)

    assert persisted["settlement"]["record"]["event_id"] == settlement["event_id"]
    assert persisted["capability"]["record"]["event_id"] == capability["event_id"]
    assert persisted["repetition"]["record"]["event_id"] == repetition["event_id"]
    assert ledger_store.read("agentic_settlements")[0]["event_id"] == settlement["event_id"]
    assert ledger_store.read("agentic_capabilities")[0]["event_id"] == capability["event_id"]
    assert ledger_store.read("agentic_repetitions")[0]["event_id"] == repetition["event_id"]


def test_twin_adapter_consumes_twin_updated_into_json_state(tmp_path: Path) -> None:
    twin_adapter = _load_module(
        "sea_acl_twin_adapter_hardening",
        SEA_ROOT / "libs/agentic_capability_loop/twin_adapter.py",
    )
    twin_event = {
        "event_id": "twin-event-1",
        "event_type": "TwinUpdated",
        "namespace": "agentic_capability_loop",
        "occurred_at": "2026-06-03T00:00:00+00:00",
        "payload": {
            "work_request_id": "wr-twin",
            "capability_id": "cap-twin",
            "context_health": "cited",
            "policy_adherence": "allow",
            "capability_delta": {"settlement_count": 1},
        },
    }

    result = twin_adapter.JsonTwinStateAdapter(tmp_path / "twin_state.json").consume_twin_updated(twin_event)

    assert result["last_event_id"] == "twin-event-1"
    assert result["work_requests"]["wr-twin"]["capability_id"] == "cap-twin"
    assert json.loads((tmp_path / "twin_state.json").read_text(encoding="utf-8")) == result


def test_context_kernel_rust_module_is_wired_and_matches_generated_event_names() -> None:
    lib_rs = (CONTEXT_KERNEL_ROOT / "crates/ck-mcp/src/lib.rs").read_text(encoding="utf-8")
    rust_rs = (CONTEXT_KERNEL_ROOT / "crates/ck-mcp/src/agentic_capability_loop.rs").read_text(encoding="utf-8")
    generated_types = (SEA_ROOT / "generated/python/agentic_capability_loop/types.py").read_text(encoding="utf-8")

    assert "pub mod agentic_capability_loop;" in lib_rs
    for name in ("ContextRequired", "ContextPacketCreated", "ContextAuthorized"):
        assert f"{name}Event" in generated_types
    expected_fields = {
        "ContextRequiredPayload": {
            "domain_model_hash",
            "work_request_id",
            "context_requirement_id",
            "corpus_id",
            "query",
            "max_results",
            "is_private",
            "scope_claim",
        },
        "ContextPacketCreatedPayload": {
            "domain_model_hash",
            "context_packet_id",
            "context_requirement_id",
            "work_request_id",
            "corpus_id",
            "citations",
            "authorized",
        },
        "ContextAuthorizedPayload": {
            "domain_model_hash",
            "context_requirement_id",
            "work_request_id",
            "corpus_id",
            "result",
            "reason",
            "policy_code",
            "scope_claim",
            "evaluated_at",
        },
    }
    for struct_name, fields in expected_fields.items():
        python_class = re.search(rf"class {struct_name}\(TypedDict\):\n(?P<body>(?:    .+\n)+)", generated_types)
        rust_struct = re.search(rf"pub struct {struct_name} \{{\n(?P<body>(?:    .+\n)+)\}}", rust_rs)
        assert python_class is not None
        assert rust_struct is not None
        python_fields = {
            line.strip().split(":", 1)[0]
            for line in python_class.group("body").splitlines()
            if ":" in line
        }
        rust_fields = {
            line.strip().removeprefix("pub ").split(":", 1)[0]
            for line in rust_struct.group("body").splitlines()
            if line.strip().startswith("pub ")
        }
        assert fields <= python_fields
        assert fields <= rust_fields


def test_failure_gates_emit_expected_policy_decisions() -> None:
    stale_proof = {"result": "pass", "completed_at": "2020-01-01T00:00:00+00:00"}

    assert sea_policies.pol_acl_001_high_risk_work_requires_context_packet(
        {"risk_level": "high"},
        None,
    )["decision"] == "deny"
    assert sea_policies.pol_acl_006_completion_requires_fresh_proof({}, stale_proof)["decision"] == "escalate"
    assert sea_policies.pol_acl_007_settlement_requires_evidence(
        {"work_request_id": "wr"},
        None,
    )["decision"] == "deny"
    assert sea_policies.pol_acl_009_simulation_cannot_promote_capability({"type": "simulation"})["decision"] == "deny"
    assert sea_policies.pol_acl_008_capability_promotion_requires_repeated_settlement(
        {"settlement_count": 1},
    )["decision"] == "deny"
    assert sea_policies.pol_acl_010_harness_mutation_requires_human_approval(
        "swe_seed_harness",
        None,
    )["decision"] == "deny"


def test_stale_proof_completion_escalates_with_coherence_break() -> None:
    with _sea_authority_deps():
        authority_service = _load_module(
            "sea_acl_authority_service_completion_hardening",
            SEA_ROOT / "libs/agentic_capability_loop/authority_service.py",
            package="agentic_capability_loop",
        )
    service = authority_service.AuthorityService()

    result = service.evaluate_completion(
        {"work_request_id": "wr-stale"},
        {"proof_result_id": "proof-stale", "result": "pass", "completed_at": "2020-01-01T00:00:00+00:00"},
    )

    assert result["authority_event"]["payload"]["result"] == "escalate"
    assert result["authority_event"]["payload"]["policy_code"] == "POL-ACL-006"
    assert result["coherence_break_event"]["event_type"] == "CoherenceBreakDetected"
    assert result["coherence_break_event"]["payload"]["work_request_id"] == "wr-stale"


def test_e2e_capability_compounding_preserves_traceability_and_active_evidence_state() -> None:
    settlement = gs_adapters.emit_settlement_recorded("wr-e2e", "proof-e2e", "evidence-e2e", "usable", "matched")
    capability = gs_adapters.update_capability_ledger(
        settlement,
        capability_name="agentic_loop_hardening",
        settlement_count=1,
    )
    repetition = gs_adapters.schedule_repetition(capability["payload"]["capability_id"])
    twin = gs_adapters.emit_twin_update(
        settlement,
        capability,
        context_id="context-e2e",
        context_health="cited",
        policy_adherence="allow",
    )

    for event in (settlement, capability, repetition, twin):
        assert event["payload"]["domain_model_hash"]

    assert capability["payload"]["settlement_event_id"] == settlement["event_id"]
    assert capability["payload"]["settlement_count"] == 1
    assert capability["payload"]["status"] == "active_evidence"
    assert repetition["payload"]["repetition_plan_id"]
    assert twin["payload"]["context_health"] == "cited"
    assert twin["payload"]["policy_adherence"] == "allow"
    assert twin["payload"]["capability_delta"]["settlement_count"] == 1


def test_full_fourteen_event_sequence_carries_domain_hash_and_trace_ids() -> None:
    swe_adapters = __import__("agentic_capability_loop.adapters", fromlist=["emit_work_requested"])
    ck_adapters = _load_module(
        "ck_acl_adapters_hardening_full",
        CONTEXT_KERNEL_ROOT / "integration/agentic_capability_loop/adapters.py",
    )
    with _sea_authority_deps():
        authority_service = _load_module(
            "sea_acl_authority_service_full_hardening",
            SEA_ROOT / "libs/agentic_capability_loop/authority_service.py",
            package="agentic_capability_loop",
        ).AuthorityService()

    work = swe_adapters.emit_work_requested(
        "wr-full",
        "agent-full",
        "implement hardening",
        "agentic_capability_loop",
        risk_level="high",
    )
    context_required = swe_adapters.emit_context_required("wr-full", "ctx-req-full", "public:sea-docs")
    context_packet, context_authorized = ck_adapters.handle_context_required(context_required)
    action = sea_adapters.emit_action_proposed(
        "wr-full",
        "file_write",
        "libs/agentic_capability_loop",
        context_packet_id=context_packet["payload"]["context_packet_id"],
    )
    authority = authority_service.evaluate_work_request(work["payload"], context_packet["payload"])["authority_event"]
    route = swe_adapters.emit_route_selected("wr-full", "route-full", "implementation", "proof-command-full")
    proof_started = swe_adapters.emit_proof_started("proof-full", "wr-full", "route-full")
    proof_completed = swe_adapters.emit_proof_completed("proof-full", "wr-full", "pass", exit_code=0)
    evidence = sea_adapters.emit_evidence_recorded(
        "wr-full",
        "proof-full",
        "pass",
        "pass",
        output_ref="pytest",
        context_packet_id=context_packet["payload"]["context_packet_id"],
    )
    settlement = gs_adapters.handle_evidence_recorded(evidence)
    capability = gs_adapters.update_capability_ledger(
        settlement,
        "agentic_loop_full",
        settlement_count=1,
    )
    repetition = gs_adapters.schedule_repetition(capability["payload"]["capability_id"])
    twin = gs_adapters.emit_twin_update(
        settlement,
        capability,
        context_id=context_packet["payload"]["context_packet_id"],
        context_health="cited",
        policy_adherence=authority["payload"]["result"],
    )
    events = [
        work,
        context_required,
        context_packet,
        context_authorized,
        action,
        authority,
        route,
        proof_started,
        proof_completed,
        evidence,
        settlement,
        capability,
        repetition,
        twin,
    ]
    assert len(events) == 14
    for event in events:
        assert event["payload"]["domain_model_hash"]
    assert evidence["payload"]["context_packet_id"] == context_packet["payload"]["context_packet_id"]
    assert settlement["payload"]["proof_result_id"] == evidence["payload"]["proof_result_id"]
    assert capability["payload"]["settlement_event_id"] == settlement["event_id"]
    assert capability["payload"]["status"] == "active_evidence"
    assert twin["payload"]["context_id"] == context_packet["payload"]["context_packet_id"]
    assert twin["payload"]["capability_delta"]["settlement_count"] == 1
