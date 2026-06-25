"""Integration tests for the agentic capability loop.

Tests the full event sequence across all four repos:
  SEA-Forge → SWE_SEED → Context Kernel → GodSpeed-Agent

Run with:
    python -m pytest tests/test_agentic_capability_loop.py -v

All adapters are imported from local packages. These tests do NOT
require live services; they verify the event contract and business
logic of each adapter module.
"""
from __future__ import annotations

import sys
import uuid
import importlib.util
from pathlib import Path

import pytest
from agentic_capability_loop import adapters as swe_adapters
from workspace_roots import CONTEXT_KERNEL_ROOT, GODSPEED_AGENT_ROOT, SEA_ROOT

# ---------------------------------------------------------------------------
# Import adapters from each repo using importlib.util to avoid name shadowing
# ---------------------------------------------------------------------------

def _load_module(name: str, path: Path, *, package: str | None = None):
    """Load a Python module by absolute file path."""
    if not path.exists():
        return None
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)  # type: ignore[arg-type]
    if package:
        mod.__package__ = package
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


def _ensure_sea_package_loaded() -> None:
    """Pre-load domain_model into sys.modules so adapters.py relative imports work."""
    pkg = "agentic_capability_loop"
    if f"{pkg}.domain_model" in sys.modules:
        return
    dm_path = SEA_ROOT / "libs/agentic_capability_loop/domain_model.py"
    if not dm_path.exists():
        return
    spec = importlib.util.spec_from_file_location(f"{pkg}.domain_model", dm_path)
    mod = importlib.util.module_from_spec(spec)  # type: ignore[arg-type]
    mod.__package__ = pkg
    sys.modules[f"{pkg}.domain_model"] = mod
    spec.loader.exec_module(mod)  # type: ignore[union-attr]

# GodSpeed-Agent adapter — load by absolute path to avoid shadowing
gs_adapters = _load_module(
    "gs_acl_adapters",
    GODSPEED_AGENT_ROOT / "agentic_capability_loop" / "adapters.py",
)

# Context-Kernel integration adapter — load by absolute path
_ck_init = _load_module(
    "ck_acl_init",
    CONTEXT_KERNEL_ROOT / "integration" / "agentic_capability_loop" / "__init__.py",
)
ck_adapters = _load_module(
    "ck_acl_adapters",
    CONTEXT_KERNEL_ROOT / "integration" / "agentic_capability_loop" / "adapters.py",
)

# SEA-Forge adapters and policies — load by absolute path
_ensure_sea_package_loaded()
sea_adapters = _load_module(
    "sea_acl_adapters",
    SEA_ROOT / "libs/agentic_capability_loop/adapters.py",
    package="agentic_capability_loop",
)
sea_policies = _load_module(
    "sea_acl_policies",
    SEA_ROOT / "libs/agentic_capability_loop/policies/agentic_capability_loop.py",
)

# Guard: fail fast if required adapters couldn't be loaded
if gs_adapters is None:
    raise ImportError("Could not load GodSpeed-Agent agentic_capability_loop adapter")
if ck_adapters is None:
    raise ImportError("Could not load Context-Kernel integration agentic_capability_loop adapter")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def new_id() -> str:
    return str(uuid.uuid4())


def assert_event(event: dict, expected_type: str) -> None:
    """Assert that a dict is a valid canonical event of the expected type."""
    assert isinstance(event, dict), f"Expected dict, got {type(event)}"
    assert event.get("event_type") == expected_type, (
        f"Expected event_type '{expected_type}', got '{event.get('event_type')}'"
    )
    assert "event_id" in event, "Event missing event_id"
    assert "occurred_at" in event, "Event missing occurred_at"
    assert "namespace" in event, "Event missing namespace"
    assert event.get("namespace") == "agentic_capability_loop", (
        f"Expected namespace 'agentic_capability_loop', got '{event.get('namespace')}'"
    )
    assert "payload" in event, "Event missing payload"
    assert isinstance(event["payload"], dict), "Event payload must be a dict"


# ---------------------------------------------------------------------------
# Happy path: full loop
# ---------------------------------------------------------------------------

class TestFullLoopHappyPath:
    """Test the complete agentic capability loop in the happy-path scenario."""

    def test_work_requested(self):
        """SWE_SEED emits WorkRequested when a task arrives."""
        work_request_id = new_id()
        event = swe_adapters.emit_work_requested(
            work_request_id=work_request_id,
            actor_id="agent-001",
            operation="implement feature",
            resource="src/foo.py",
            risk_level="low",
            repo="my-repo",
        )
        assert_event(event, "WorkRequested")
        p = event["payload"]
        assert p["work_request_id"] == work_request_id
        assert p["actor_id"] == "agent-001"
        assert p["risk_level"] == "low"

    def test_context_required(self):
        """SWE_SEED emits ContextRequired when context is needed."""
        work_request_id = new_id()
        ctx_req_id = new_id()
        event = swe_adapters.emit_context_required(
            work_request_id=work_request_id,
            context_requirement_id=ctx_req_id,
            corpus_id="public:sea-docs",
            query="how to write policies",
        )
        assert_event(event, "ContextRequired")
        p = event["payload"]
        assert p["work_request_id"] == work_request_id
        assert p["context_requirement_id"] == ctx_req_id
        assert p["corpus_id"] == "public:sea-docs"

    def test_context_kernel_handles_context_required(self):
        """Context Kernel handles ContextRequired and emits ContextPacketCreated + ContextAuthorized."""
        work_request_id = new_id()
        ctx_req_id = new_id()
        context_required_event = swe_adapters.emit_context_required(
            work_request_id=work_request_id,
            context_requirement_id=ctx_req_id,
            corpus_id="public:sea-docs",
            query="policy documentation",
        )

        packet_event, auth_event = ck_adapters.handle_context_required(context_required_event)

        assert_event(packet_event, "ContextPacketCreated")
        assert_event(auth_event, "ContextAuthorized")

        # ContextPacketCreated must reference the same IDs
        assert packet_event["payload"]["work_request_id"] == work_request_id
        assert packet_event["payload"]["context_requirement_id"] == ctx_req_id
        assert packet_event["payload"]["authorized"] is True
        assert len(packet_event["payload"]["citations"]) >= 1

        # ContextAuthorized must allow
        assert auth_event["payload"]["result"] == "allow"
        assert auth_event["payload"]["work_request_id"] == work_request_id

    def test_swe_seed_consumes_context_packet(self):
        """SWE_SEED can consume ContextPacketCreated and extract the context."""
        work_request_id = new_id()
        ctx_req_id = new_id()
        ctx_req_event = swe_adapters.emit_context_required(
            work_request_id=work_request_id,
            context_requirement_id=ctx_req_id,
            corpus_id="public:sea-docs",
        )
        packet_event, _ = ck_adapters.handle_context_required(ctx_req_event)

        packet_payload = swe_adapters.consume_context_packet_created(packet_event)
        assert packet_payload["work_request_id"] == work_request_id
        assert "citations" in packet_payload

    def test_route_selected(self):
        """SWE_SEED emits RouteSelected when a harness route is chosen."""
        work_request_id = new_id()
        route_id = new_id()
        proof_command_id = new_id()
        event = swe_adapters.emit_route_selected(
            work_request_id=work_request_id,
            route_id=route_id,
            route_name="implementation",
            proof_command_id=proof_command_id,
        )
        assert_event(event, "RouteSelected")
        p = event["payload"]
        assert p["work_request_id"] == work_request_id
        assert p["route_name"] == "implementation"

    def test_proof_started_and_completed(self):
        """SWE_SEED emits ProofStarted then ProofCompleted."""
        work_request_id = new_id()
        proof_result_id = new_id()
        route_id = new_id()

        started = swe_adapters.emit_proof_started(
            proof_result_id=proof_result_id,
            work_request_id=work_request_id,
            route_id=route_id,
            proof_command="pytest tests/ -v",
        )
        assert_event(started, "ProofStarted")
        assert started["payload"]["proof_result_id"] == proof_result_id

        completed = swe_adapters.emit_proof_completed(
            proof_result_id=proof_result_id,
            work_request_id=work_request_id,
            result="pass",
            exit_code=0,
            output_ref="/tmp/proof-output.log",
            proof_type="live",
        )
        assert_event(completed, "ProofCompleted")
        assert completed["payload"]["result"] == "pass"
        assert completed["payload"]["proof_result_id"] == proof_result_id

    def test_godspeed_handles_evidence_and_emits_settlement(self):
        """GodSpeed-Agent handles EvidenceRecorded and emits SettlementRecorded."""
        work_request_id = new_id()
        proof_result_id = new_id()

        # Build an EvidenceRecorded event (as SEA-Forge would emit)
        if sea_adapters:
            evidence_event = sea_adapters.emit_evidence_recorded(
                work_request_id=work_request_id,
                proof_result_id=proof_result_id,
                observed_result="pass",
                expected_result="pass",
                output_ref="/tmp/proof-output.log",
            )
        else:
            # Stub when SEA-Forge libs not importable
            import datetime
            evidence_event = {
                "event_id": new_id(),
                "event_type": "EvidenceRecorded",
                "namespace": "agentic_capability_loop",
                "occurred_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                "payload": {
                    "work_request_id": work_request_id,
                    "proof_result_id": proof_result_id,
                    "observed_result": "pass",
                    "expected_result": "pass",
                    "output_ref": "/tmp/proof-output.log",
                },
            }

        settlement_event = gs_adapters.handle_evidence_recorded(evidence_event)
        assert_event(settlement_event, "SettlementRecorded")
        p = settlement_event["payload"]
        assert p["work_request_id"] == work_request_id
        assert p["outcome"] == "usable"

    def test_full_loop_end_to_end(self):
        """Full capability loop: WorkRequested → settlement → capability updated → twin updated."""
        work_request_id = new_id()
        ctx_req_id = new_id()
        proof_result_id = new_id()
        route_id = new_id()

        # Step 1: SWE_SEED emits WorkRequested
        work_req_event = swe_adapters.emit_work_requested(
            work_request_id=work_request_id,
            actor_id="agent-001",
            operation="implement feature",
            resource="src/foo.py",
        )
        assert_event(work_req_event, "WorkRequested")

        # Step 2: SWE_SEED emits ContextRequired
        ctx_req_event = swe_adapters.emit_context_required(
            work_request_id=work_request_id,
            context_requirement_id=ctx_req_id,
            corpus_id="public:sea-docs",
        )
        assert_event(ctx_req_event, "ContextRequired")

        # Step 3: Context Kernel handles it
        packet_event, auth_event = ck_adapters.handle_context_required(ctx_req_event)
        assert_event(packet_event, "ContextPacketCreated")
        assert_event(auth_event, "ContextAuthorized")
        assert auth_event["payload"]["result"] == "allow"

        # Step 4: SWE_SEED selects route
        route_event = swe_adapters.emit_route_selected(
            work_request_id=work_request_id,
            route_id=route_id,
            route_name="implementation",
            proof_command_id=new_id(),
        )
        assert_event(route_event, "RouteSelected")

        # Step 5: SWE_SEED runs proof
        proof_started = swe_adapters.emit_proof_started(
            proof_result_id=proof_result_id,
            work_request_id=work_request_id,
            route_id=route_id,
        )
        proof_completed = swe_adapters.emit_proof_completed(
            proof_result_id=proof_result_id,
            work_request_id=work_request_id,
            result="pass",
            exit_code=0,
        )
        assert_event(proof_started, "ProofStarted")
        assert_event(proof_completed, "ProofCompleted")

        # Step 6: SEA-Forge records evidence (stub when not importable)
        import datetime
        evidence_event = {
            "event_id": new_id(),
            "event_type": "EvidenceRecorded",
            "namespace": "agentic_capability_loop",
            "occurred_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "payload": {
                "work_request_id": work_request_id,
                "proof_result_id": proof_result_id,
                "observed_result": "pass",
                "expected_result": "pass",
            },
        }

        # Step 7: GodSpeed-Agent processes evidence → settlement
        settlement_event = gs_adapters.handle_evidence_recorded(evidence_event)
        assert_event(settlement_event, "SettlementRecorded")
        assert settlement_event["payload"]["outcome"] == "usable"

        # Step 8: GodSpeed-Agent updates capability
        capability_event = gs_adapters.update_capability_ledger(
            settlement_event,
            capability_name="implement_feature",
        )
        assert_event(capability_event, "CapabilityUpdated")
        assert capability_event["payload"]["status"] == "active_evidence"

        # Step 9: GodSpeed-Agent schedules repetition
        repetition_event = gs_adapters.schedule_repetition(
            capability_event["payload"]["capability_id"]
        )
        assert_event(repetition_event, "RepetitionPlanned")

        # Step 10: GodSpeed-Agent updates twin
        twin_event = gs_adapters.emit_twin_update(settlement_event, capability_event)
        assert_event(twin_event, "TwinUpdated")
        assert twin_event["payload"]["outcome"] == "usable"


# ---------------------------------------------------------------------------
# Failure mode: missing context packet
# ---------------------------------------------------------------------------

class TestFailureModes:
    """Test failure modes that trigger CoherenceBreak."""

    def test_unauthorized_private_corpus_produces_deny(self):
        """Context Kernel denies access to private corpus without scope claim."""
        work_request_id = new_id()
        ctx_req_event = swe_adapters.emit_context_required(
            work_request_id=work_request_id,
            context_requirement_id=new_id(),
            corpus_id="private:secret-docs",
            is_private=True,
            # No scope_claim provided — should be denied
        )
        packet_event, auth_event = ck_adapters.handle_context_required(ctx_req_event)
        assert_event(auth_event, "ContextAuthorized")
        assert auth_event["payload"]["result"] == "deny"
        assert "POL-ACL-002" in auth_event["payload"]["policy_code"]

        # ContextPacket should have no citations (unauthorized)
        assert packet_event["payload"]["authorized"] is False
        assert packet_event["payload"]["citations"] == []

    def test_authorized_private_corpus_with_scope_claim(self):
        """Context Kernel allows private corpus with valid scope claim."""
        work_request_id = new_id()
        ctx_req_event = swe_adapters.emit_context_required(
            work_request_id=work_request_id,
            context_requirement_id=new_id(),
            corpus_id="private:secret-docs",
            is_private=True,
            scope_claim="team:engineering",
        )
        packet_event, auth_event = ck_adapters.handle_context_required(ctx_req_event)
        assert auth_event["payload"]["result"] == "allow"
        assert packet_event["payload"]["authorized"] is True

    def test_missing_evidence_causes_coherence_break(self):
        """GodSpeed-Agent emits CoherenceBreakDetected when no evidence is available."""
        work_request_id = new_id()
        coherence_event, learning_event = gs_adapters.detect_coherence_break(
            reason="ProofCompleted but no EvidenceRecorded event was emitted",
            work_request_id=work_request_id,
            missing_phase="EvidenceRecorded",
        )
        assert_event(coherence_event, "CoherenceBreakDetected")
        assert_event(learning_event, "LearningProposalCreated")
        assert coherence_event["payload"]["work_request_id"] == work_request_id
        assert coherence_event["payload"]["missing_phase"] == "EvidenceRecorded"
        # LearningProposal must link to the coherence break
        assert (
            learning_event["payload"]["coherence_break_id"]
            == coherence_event["payload"]["coherence_break_id"]
        )

    def test_failed_proof_produces_unusable_settlement(self):
        """A failed proof results in 'unusable' settlement outcome."""
        work_request_id = new_id()
        import datetime
        evidence_event = {
            "event_id": new_id(),
            "event_type": "EvidenceRecorded",
            "namespace": "agentic_capability_loop",
            "occurred_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "payload": {
                "work_request_id": work_request_id,
                "proof_result_id": new_id(),
                "observed_result": "",
                "expected_result": "pass",
            },
        }
        settlement = gs_adapters.handle_evidence_recorded(evidence_event)
        assert settlement["payload"]["outcome"] == "unusable"

    def test_partial_match_produces_partial_settlement(self):
        """A partial proof match results in 'partial' settlement outcome."""
        work_request_id = new_id()
        import datetime
        evidence_event = {
            "event_id": new_id(),
            "event_type": "EvidenceRecorded",
            "namespace": "agentic_capability_loop",
            "occurred_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "payload": {
                "work_request_id": work_request_id,
                "proof_result_id": new_id(),
                "observed_result": "some tests passed",
                "expected_result": "all tests pass",
            },
        }
        settlement = gs_adapters.handle_evidence_recorded(evidence_event)
        assert settlement["payload"]["outcome"] == "partial"

    def test_wrong_event_type_raises_error(self):
        """Adapter raises ValueError if event type doesn't match."""
        wrong_event = {
            "event_id": new_id(),
            "event_type": "WorkRequested",
            "namespace": "agentic_capability_loop",
            "occurred_at": "2024-01-01T00:00:00+00:00",
            "payload": {},
        }
        with pytest.raises(ValueError, match="Expected EvidenceRecorded"):
            gs_adapters.handle_evidence_recorded(wrong_event)

        with pytest.raises(ValueError, match="Expected ContextRequired"):
            ck_adapters.handle_context_required(wrong_event)

        with pytest.raises(ValueError, match="Expected ContextPacketCreated"):
            swe_adapters.consume_context_packet_created(wrong_event)


# ---------------------------------------------------------------------------
# Policy evaluation tests (when SEA-Forge libs are available)
# ---------------------------------------------------------------------------

class TestPolicyEvaluation:
    """Test SEA-Forge policy evaluators."""

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_001_denies_high_risk_without_context_packet(self):
        result = sea_policies.pol_acl_001_high_risk_work_requires_context_packet(
            work_request={"risk_level": "high"},
            context_packet=None,
        )
        assert result["decision"] == "deny"
        assert result["code"] == "POL-ACL-001"

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_001_allows_low_risk_without_context_packet(self):
        result = sea_policies.pol_acl_001_high_risk_work_requires_context_packet(
            work_request={"risk_level": "low"},
            context_packet=None,
        )
        assert result["decision"] == "allow"

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_001_allows_high_risk_with_valid_context_packet(self):
        result = sea_policies.pol_acl_001_high_risk_work_requires_context_packet(
            work_request={"risk_level": "high"},
            context_packet={"citations": [{"source": "doc/1", "content": "text"}]},
        )
        assert result["decision"] == "allow"

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_002_denies_private_corpus_without_scope(self):
        result = sea_policies.pol_acl_002_private_corpus_requires_explicit_scope(
            corpus_id="private:secret",
            scope_claim=None,
            authorized=False,
        )
        assert result["decision"] == "deny"
        assert result["code"] == "POL-ACL-002"

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_007_denies_settlement_without_evidence(self):
        result = sea_policies.pol_acl_007_settlement_requires_evidence(
            settlement={"work_request_id": "wr-001"},
            evidence_event=None,
        )
        assert result["decision"] == "deny"
        assert result["code"] == "POL-ACL-007"

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_007_denies_mismatched_work_request_id(self):
        result = sea_policies.pol_acl_007_settlement_requires_evidence(
            settlement={"work_request_id": "wr-001"},
            evidence_event={"work_request_id": "wr-DIFFERENT"},
        )
        assert result["decision"] == "deny"

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_009_denies_simulation_capability_promotion(self):
        result = sea_policies.pol_acl_009_simulation_cannot_promote_capability(
            proof_result={"type": "simulation"}
        )
        assert result["decision"] == "deny"
        assert result["code"] == "POL-ACL-009"

    @pytest.mark.skipif(sea_policies is None, reason="SEA-Forge libs not importable from this path")
    def test_pol_acl_009_allows_live_proof_capability_promotion(self):
        result = sea_policies.pol_acl_009_simulation_cannot_promote_capability(
            proof_result={"type": "live"}
        )
        assert result["decision"] == "allow"


# ---------------------------------------------------------------------------
# Event ID consistency checks
# ---------------------------------------------------------------------------

class TestEventIdConsistency:
    """Verify that IDs chain correctly across events."""

    def test_proof_result_id_links_started_to_completed(self):
        proof_result_id = new_id()
        work_request_id = new_id()
        started = swe_adapters.emit_proof_started(
            proof_result_id=proof_result_id,
            work_request_id=work_request_id,
            route_id=new_id(),
        )
        completed = swe_adapters.emit_proof_completed(
            proof_result_id=proof_result_id,
            work_request_id=work_request_id,
            result="pass",
        )
        assert started["payload"]["proof_result_id"] == completed["payload"]["proof_result_id"]
        assert started["payload"]["work_request_id"] == completed["payload"]["work_request_id"]

    def test_context_requirement_id_links_required_to_packet(self):
        work_request_id = new_id()
        ctx_req_id = new_id()
        ctx_req_event = swe_adapters.emit_context_required(
            work_request_id=work_request_id,
            context_requirement_id=ctx_req_id,
            corpus_id="public:sea-docs",
        )
        packet_event, auth_event = ck_adapters.handle_context_required(ctx_req_event)
        assert packet_event["payload"]["context_requirement_id"] == ctx_req_id
        assert auth_event["payload"]["context_requirement_id"] == ctx_req_id

    def test_evidence_event_id_links_in_settlement(self):
        work_request_id = new_id()
        import datetime
        evidence_event = {
            "event_id": "evidence-id-123",
            "event_type": "EvidenceRecorded",
            "namespace": "agentic_capability_loop",
            "occurred_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "payload": {
                "work_request_id": work_request_id,
                "proof_result_id": new_id(),
                "observed_result": "pass",
                "expected_result": "pass",
            },
        }
        settlement = gs_adapters.handle_evidence_recorded(evidence_event)
        assert settlement["payload"]["evidence_event_id"] == "evidence-id-123"

    def test_coherence_break_id_links_break_to_learning_proposal(self):
        work_request_id = new_id()
        break_event, learning_event = gs_adapters.detect_coherence_break(
            reason="Missing context packet",
            work_request_id=work_request_id,
        )
        assert (
            break_event["payload"]["coherence_break_id"]
            == learning_event["payload"]["coherence_break_id"]
        )
