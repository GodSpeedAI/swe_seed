# GodSpeed Stack Agentic Loop Wiring Report

**Prepared:** 2026-06-10  
**Investigator:** Antigravity (AI coding assistant)  
**Evidence base:** Direct code inspection across 5 repositories  
**Proof level:** local-confidence (adapter contracts + test coverage verified; live service execution not verified)

---

## 1. Executive Judgment

The GodSpeed stack has a **substantially implemented and surprisingly well-wired** agentic capability loop — significantly more so than typical multi-repo architecture claims. The loop is modeled in a canonical `.sea` file, projected into a JSON manifest, and implemented through Python adapter modules in all four participating repositories (SWE_SEED, Context Kernel, SEA-Forge, GodSpeed-Agent). A cross-repo integration test suite (`SWE_SEED/tests/test_agentic_capability_loop.py`) exercises the full 10-step happy path plus failure modes and ID chaining.

The weakest link is **DomainForge's role in the loop**: DomainForge parses and projects `.sea` files but is not invoked in the integration tests, and the `domain_model_hash` used across all adapters is a naive SHA-256 of the literal string `"agentic_capability_loop"` rather than a hash of the actual `.sea` file content or generated manifest. This means semantic pack provenance is not cryptographically tied to DomainForge's output.

The second weak link is the **live service gap**: Context Kernel's Rust `ContextAgent` trait is defined but not implemented in any crate. The integration tests use Python stubs that bypass the actual Rust MCP service.

---

## 2. Current Implemented Loop

The stack that actually runs (verified by code):

```
SWE_SEED                   → emits WorkRequested, ContextRequired, RouteSelected,
                             ProofStarted, ProofCompleted
Context Kernel (Python)    → handles ContextRequired → emits ContextPacketCreated,
                             ContextAuthorized (POL-ACL-002)
SEA-Forge                  → handles ActionProposed → evaluates POL-ACL-001..010 →
                             emits AuthorityChecked, EvidenceRecorded,
                             CoherenceBreakDetected
GodSpeed-Agent             → handles EvidenceRecorded → emits SettlementRecorded,
                             CapabilityUpdated, RepetitionPlanned, TwinUpdated,
                             CoherenceBreakDetected, LearningProposalCreated
```

All of these flow through a canonical event envelope:

```json
{
  "event_id": "<uuid>",
  "event_type": "<EventName>",
  "namespace": "agentic_capability_loop",
  "occurred_at": "<iso8601>",
  "payload": {
    "domain_model_hash": "<sha256>",
    ...event-specific fields...
  }
}
```

---

## 3. Intended Loop vs. Implemented Loop

| Intended Step                                           | Implemented?   | Evidence                                                                                                 |
| ------------------------------------------------------- | -------------- | -------------------------------------------------------------------------------------------------------- |
| DomainForge parses `.sea` → canonical representation    | ✅ Implemented | `sea-core/src/parser/mod.rs`: `parse()`, `parse_to_graph()`                                              |
| DomainForge emits semantic pack outputs                 | ⚠️ Partial     | `.ast.json`, `.ir.json`, `.manifest.json` committed but not auto-regenerated or cryptographically hashed |
| Context Kernel consumes `.sea`/semantic packs           | ❌ Missing     | CK accepts only canonical events, not `.sea` files directly                                              |
| Context Kernel outputs a context bundle with provenance | ✅ Implemented | `ContextPacketCreated` with `context_packet_id`, `domain_model_hash`, `citations`                        |
| SEA-Forge checks authority before action                | ✅ Implemented | `AuthorityService.evaluate_work_request()`, 10 policy evaluators                                         |
| SEA-Forge records authority trace                       | ✅ Implemented | `AuthorityChecked` event with `policy_code`, `result`, `reason`                                          |
| SWE_SEED requires proof before completion               | ✅ Implemented | POL-ACL-006: `CompletionRequiresFreshProof`; `ProofCompleted` must precede settlement                    |
| SWE_SEED records trace                                  | ✅ Implemented | `WorkRequested` → `RouteSelected` → `ProofStarted` → `ProofCompleted` events with linked IDs             |
| GodSpeed-Agent records settlement                       | ✅ Implemented | `handle_evidence_recorded()` → `SettlementRecorded` with `outcome: usable/partial/unusable`              |
| GodSpeed-Agent updates capability memory                | ✅ Implemented | `update_capability_ledger()` → `CapabilityUpdated` with status `active_evidence`/`metabolized`           |
| GodSpeed-Agent emits semantic updates                   | ⚠️ Partial     | `TwinUpdated` event implemented; no updated semantic pack or diff emitted                                |
| Failure routes to diagnosis                             | ✅ Implemented | `CoherenceBreakDetected` + `LearningProposalCreated` events in both GodSpeed-Agent and SEA-Forge         |

---

## 4. Contract Map

The shared **Semantic Work Envelope** as actually implemented:

| Field                                 | Present?  | Carrier                                                                      |
| ------------------------------------- | --------- | ---------------------------------------------------------------------------- |
| `namespace`                           | ✅        | All events (`"agentic_capability_loop"`)                                     |
| `domain_model_hash`                   | ✅ (weak) | All event payloads; SHA-256 of literal namespace string, not `.sea` content  |
| `work_request_id`                     | ✅        | Chains through all 14 events                                                 |
| `context_requirement_id`              | ✅        | Links ContextRequired → ContextPacketCreated → ContextAuthorized             |
| `context_packet_id`                   | ✅        | Links ContextPacketCreated → EvidenceRecorded → SettlementRecorded           |
| `proof_result_id`                     | ✅        | Links ProofStarted → ProofCompleted → EvidenceRecorded → SettlementRecorded  |
| `evidence_event_id`                   | ✅        | Links EvidenceRecorded → SettlementRecorded                                  |
| `settlement_event_id`                 | ✅        | Links SettlementRecorded → CapabilityUpdated → TwinUpdated                   |
| `capability_id`                       | ✅        | Links CapabilityUpdated → RepetitionPlanned                                  |
| `coherence_break_id`                  | ✅        | Links CoherenceBreakDetected → LearningProposalCreated                       |
| `semantic_pack_hash`                  | ❌        | Not present; `domain_model_hash` is a static constant, not a content hash    |
| `.sea` source reference               | ❌        | Not carried in any event                                                     |
| `actor`                               | ✅        | `actor_id` in WorkRequested, AuthorityChecked                                |
| `operation`                           | ✅        | `action_type` in ActionProposed, AuthorityChecked                            |
| `resource`                            | ✅        | `resource` in ActionProposed                                                 |
| `authority_decision_id`               | ⚠️        | `event_id` of AuthorityChecked event serves this role, not a dedicated field |
| `execution_route_reference`           | ✅        | `route_id` links RouteSelected → ProofStarted → ProofCompleted               |
| Updated semantic pack / semantic diff | ❌        | Not emitted; TwinUpdated carries capability delta, not a semantic diff       |

**Trace chain implemented:**

```
work_request_id
→ context_requirement_id → context_packet_id
→ (authority event_id)
→ route_id → proof_result_id
→ evidence event (EvidenceRecorded)
→ settlement_event_id
→ capability_id → repetition_plan_id
→ (TwinUpdated references settlement + capability)
```

The `semantic_pack_hash → updated_semantic_pack_hash` bookend from the intended design is absent.

---

## 5. Per-Layer Input/Output Map

### DomainForge (SEA Core)

|                          |                                                                                                                                                                                                |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Input**                | `.sea` DSL source text                                                                                                                                                                         |
| **Output**               | `Ast` (parse tree), `Graph` (typed graph), `.ast.json`, `.ir.json`, `.manifest.json`                                                                                                           |
| **Parser entry**         | `sea-core/src/parser/mod.rs`: `parse()`, `parse_to_graph()`, `parse_to_graph_with_options()`                                                                                                   |
| **Test coverage**        | Unit tests in `parser/mod.rs`; 60+ integration tests in `sea-core/tests/`                                                                                                                      |
| **Downstream wiring**    | `.sea` file committed at `SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea`; manifest committed; **but DomainForge binary is not called from the integration tests** |
| **Semantic pack output** | No stable versioned "semantic pack" with content hash; manifest is committed and static                                                                                                        |

### Context Kernel (Rust + Python stub)

|                     |                                                                                                           |
| ------------------- | --------------------------------------------------------------------------------------------------------- |
| **Input**           | `ContextRequired` canonical event                                                                         |
| **Output**          | `ContextPacketCreated`, `ContextAuthorized` canonical events                                              |
| **Python adapter**  | `Context_Kernel_Service_MCP/integration/agentic_capability_loop/adapters.py`: `handle_context_required()` |
| **Rust types**      | `crates/ck-mcp/src/agentic_capability_loop.rs`: `ContextAgent` trait, `EventEnvelope`, payload structs    |
| **Rust trait impl** | ❌ `ContextAgent` trait defined but not implemented in any crate                                          |
| **`.sea` input**    | ❌ Does not accept `.sea` files; only event-driven                                                        |
| **Provenance**      | `context_packet_id`, `context_requirement_id`, `domain_model_hash` in payload                             |
| **Test coverage**   | Used in SWE_SEED integration tests; Python adapter fully exercised                                        |

### SEA-Forge

|                             |                                                                                                                               |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **Input**                   | `WorkRequested` (from SWE_SEED), `ProofCompleted` (from SWE_SEED), `ContextPacketCreated` (from CK)                           |
| **Output**                  | `AuthorityChecked`, `ActionProposed`, `EvidenceRecorded`, `CoherenceBreakDetected`                                            |
| **Authority service**       | `SEA/libs/agentic_capability_loop/authority_service.py`: `AuthorityService.evaluate_work_request()`, `.evaluate_completion()` |
| **Policies**                | `SEA/libs/agentic_capability_loop/policies/agentic_capability_loop.py`: 10 evaluators (POL-ACL-001..010)                      |
| **`.sea` source**           | `SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea` — authoritative domain model                     |
| **Policy wiring**           | PolicyDecisions linked to policy codes; `CoherenceBreakDetected` emitted on deny/escalate                                     |
| **Blocks before execution** | ✅ `evaluate_work_request()` must return `allow` for work to proceed                                                          |
| **Test coverage**           | `test_agentic_capability_hardening.py` tests all 6 policy gates; authority service tested with high/low risk                  |

### SWE_SEED

|                           |                                                                                              |
| ------------------------- | -------------------------------------------------------------------------------------------- |
| **Input**                 | `AuthorityChecked` (SEA-Forge), `ContextPacketCreated` (CK), `SettlementRecorded` (GS-Agent) |
| **Output**                | `WorkRequested`, `ContextRequired`, `RouteSelected`, `ProofStarted`, `ProofCompleted`        |
| **Adapter**               | `SWE_SEED/agentic_capability_loop/adapters.py`: 5 emit functions, 3 consume functions        |
| **Proof gating**          | `emit_proof_completed()` with `proof_type="live"` required; POL-ACL-006 blocks stale proofs  |
| **Placeholder rejection** | Via POL-ACL-009 (`simulation_cannot_promote_capability`) and POL-ACL-006                     |
| **Trace recording**       | 5 events with linked `work_request_id`, `proof_result_id`, `route_id`                        |
| **`.sea` consumption**    | ❌ Does not parse `.sea` files directly at runtime; uses committed manifest                  |
| **Test coverage**         | 605-line test suite; happy path, failure modes, ID consistency, policy evaluation            |

### GodSpeed-Agent

|                            |                                                                                                                                    |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| **Input**                  | `EvidenceRecorded` (from SEA-Forge)                                                                                                |
| **Output**                 | `SettlementRecorded`, `CapabilityUpdated`, `RepetitionPlanned`, `TwinUpdated`, `CoherenceBreakDetected`, `LearningProposalCreated` |
| **Settlement**             | `handle_evidence_recorded()` classifies `usable`/`partial`/`unusable`                                                              |
| **Capability memory**      | `update_capability_ledger()`: status progression `latent` → `active_evidence` → `metabolized` (after 3 settlements)                |
| **Ledger persistence**     | `persist_to_ledger()` writes to JSONL ledgers via `godspeed_nav/storage.py:LedgerStore`                                            |
| **Failure routing**        | `detect_coherence_break()` emits `CoherenceBreakDetected` + `LearningProposalCreated`                                              |
| **Semantic diff emission** | ❌ Not implemented; `TwinUpdated` carries `capability_delta` but no semantic pack update                                           |
| **Test coverage**          | Hardening test verifies JSONL persistence; all adapter functions exercised in SWE_SEED test suite                                  |

---

## 6. Evidence Table

| Claim                                  | File                                                                                   | Function/Type                                                           | Confidence |
| -------------------------------------- | -------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ---------- |
| DomainForge parses `.sea` files        | `domainforge/sea-core/src/parser/mod.rs`                                               | `parse()`, `parse_to_graph()`                                           | High       |
| DomainForge has a PEG grammar          | `domainforge/sea-core/grammar/sea.pest`                                                | `SeaParser`                                                             | High       |
| `.sea` file committed for agentic loop | `SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea`           | 220 lines, 10 policies                                                  | High       |
| Manifest (JSON IR) committed           | `SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json` | 1355 lines, all 14 event flows                                          | High       |
| SWE_SEED adapter emits events          | `SWE_SEED/agentic_capability_loop/adapters.py`                                         | `emit_work_requested()` .. `emit_proof_completed()`                     | High       |
| Context Kernel Python adapter          | `Context_Kernel_Service_MCP/integration/agentic_capability_loop/adapters.py`           | `handle_context_required()`                                             | High       |
| Context Kernel Rust types              | `Context_Kernel_Service_MCP/crates/ck-mcp/src/agentic_capability_loop.rs`              | `ContextAgent` trait, structs                                           | High       |
| Context Kernel Rust impl               | ❌ Not found                                                                           | `ContextAgent` trait has no `impl`                                      | High       |
| SEA-Forge authority service            | `SEA/libs/agentic_capability_loop/authority_service.py`                                | `AuthorityService`                                                      | High       |
| SEA-Forge 10 policy evaluators         | `SEA/libs/agentic_capability_loop/policies/agentic_capability_loop.py`                 | `pol_acl_001..pol_acl_010`                                              | High       |
| GodSpeed-Agent settlement              | `godspeed_agent/agentic_capability_loop/adapters.py`                                   | `handle_evidence_recorded()`                                            | High       |
| GodSpeed-Agent capability tracking     | `godspeed_agent/agentic_capability_loop/adapters.py`                                   | `update_capability_ledger()`                                            | High       |
| JSONL ledger persistence               | `godspeed_agent/godspeed_nav/storage.py`                                               | `LedgerStore`                                                           | High       |
| Cross-repo integration test            | `SWE_SEED/tests/test_agentic_capability_loop.py`                                       | `TestFullLoopHappyPath.test_full_loop_end_to_end()`                     | High       |
| 14-event sequence test                 | `SWE_SEED/tests/test_agentic_capability_hardening.py`                                  | `test_full_fourteen_event_sequence_carries_domain_hash_and_trace_ids()` | High       |
| domain_model_hash is static constant   | All adapter modules                                                                    | `hashlib.sha256(NAMESPACE.encode()).hexdigest()`                        | High       |
| DomainForge invoked in loop tests      | ❌ Not found                                                                           | No `sea-core` or `domainforge` import in any test                       | High       |
| Semantic pack with content hash        | ❌ Not found                                                                           | No content-derived hash anywhere                                        | High       |
| Context Kernel accepts `.sea` input    | ❌ Not found                                                                           | CK only handles event envelopes                                         | High       |

---

## 7. Invariant Verification Table

| #   | Invariant                                                         | Status         | Evidence                                                                                                                                    |
| --- | ----------------------------------------------------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | No governed action without semantic pack / `.sea` / waiver        | ⚠️ Partial     | `.sea` file exists but isn't loaded at runtime; `domain_model_hash` is a static constant, not tied to file                                  |
| 2   | No high-impact action without authority decision                  | ✅ Implemented | `AuthorityService.evaluate_work_request()` + POL-ACL-005 block writes/API calls without prior `allow`                                       |
| 3   | No completion claim without proof evidence                        | ✅ Implemented | POL-ACL-006 `CompletionRequiresFreshProof`; tested in hardening suite                                                                       |
| 4   | No capability update without settlement event                     | ✅ Implemented | `update_capability_ledger()` requires `SettlementRecorded` event as input                                                                   |
| 5   | Semantic pack updates are validated, versioned, traceable         | ❌ Missing     | No semantic pack update or diff; `domain_model_hash` is constant, not versioned                                                             |
| 6   | `.sea`/semantic pack consumed consistently across projects        | ⚠️ Partial     | The `.sea` file is the source of truth for events/policies; adapters hardcode from it, but don't load it at runtime                         |
| 7   | IDs and hashes preserved across the loop                          | ✅ Implemented | `work_request_id`, `proof_result_id`, `context_packet_id`, `settlement_event_id` all chain correctly (verified by `TestEventIdConsistency`) |
| 8   | Authority decisions happen before action, not after               | ✅ Implemented | `evaluate_work_request()` called before proof; hardening test verifies denial without context packet                                        |
| 9   | Proof results attached before settlement                          | ✅ Implemented | `EvidenceRecorded` contains `proof_result_id`; `SettlementRecorded` validates matching `work_request_id` (POL-ACL-007)                      |
| 10  | Settlement results fed back into memory/context/capability        | ✅ Implemented | `SettlementRecorded` → `CapabilityUpdated` → `RepetitionPlanned` + JSONL ledger persistence                                                 |
| 11  | Generated outputs are deterministic enough for regression tests   | ⚠️ Partial     | Manifest is committed and tested; `domain_model_hash` is deterministic but not content-based                                                |
| 12  | Failures route to diagnosis/escalation rather than silent success | ✅ Implemented | `CoherenceBreakDetected` + `LearningProposalCreated` on any deny/escalate; `detect_coherence_break()` in GS-Agent                           |

---

## 8. Missing or Weak Links

### Link 1 — DomainForge not invoked at runtime (High Risk)

DomainForge parses `.sea` files and generates `.ast.json`, `.ir.json`, `.manifest.json` artifacts. These are committed into `SEA/docs/specs/domains/agentic_capability_loop/`. However, **DomainForge is never called from the integration tests or at adapter runtime**. Adapters reference the manifest only indirectly (policies and event names are hardcoded). The `.sea` file could change without re-running DomainForge, and no test would catch the drift.

### Link 2 — `domain_model_hash` is not a content hash (High Risk)

Every adapter module computes:

```python
DOMAIN_MODEL_HASH = hashlib.sha256(NAMESPACE.encode("utf-8")).hexdigest()
```

This produces the same constant regardless of what is in `agentic_capability_loop.sea`. It is not a hash of the `.sea` file content, the manifest, or any semantic IR. Semantic pack provenance cannot be traced from an event back to a specific `.sea` version.

### Link 3 — Context Kernel Rust `ContextAgent` trait has no implementation (High Risk)

`crates/ck-mcp/src/agentic_capability_loop.rs` defines the `ContextAgent` trait but nothing implements it. The integration tests use the Python stub. The production path through Context Kernel's Rust MCP server does not exist for this loop.

### Link 4 — No `.sea` file consumed at runtime by any layer other than SEA-Forge (Medium Risk)

Context Kernel, SWE_SEED, and GodSpeed-Agent do not load `.sea` files. They operate on canonical event envelopes. This is pragmatic, but it means the intended role of "DomainForge supplies executable semantic meaning" is expressed only through the committed artifacts, not through runtime loading.

### Link 5 — No semantic diff or updated semantic pack emitted by GodSpeed-Agent (Medium Risk)

GodSpeed-Agent emits `TwinUpdated` with a `capability_delta`. There is no mechanism to emit an updated semantic pack, a semantic diff, or to feed back into the `.sea` file. The feedback loop from GodSpeed-Agent back to DomainForge is not implemented.

### Link 6 — Live service gap (Medium Risk)

All integration tests are in-process Python. The loop has never been run end-to-end through actual services (Context Kernel's Rust MCP server, a live harness invocation, a live proof command). The `@pytest.mark.live_proof` test (`test_live_swe_seed_harness_validate_emits_proof_completed_shape`) is the only live test.

---

## 9. Breakage Risks

| Risk                                                 | Severity | Trigger                                                                                                                                                        |
| ---------------------------------------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `.sea` file diverges from adapters with no detection | High     | Any edit to `agentic_capability_loop.sea` without regenerating adapters                                                                                        |
| `domain_model_hash` is identical across all versions | High     | Any `.sea` change; produces identical hash, no diff detectable                                                                                                 |
| Context Kernel Rust impl missing                     | High     | Any attempt to route `ContextRequired` through the MCP server                                                                                                  |
| SEA-Forge stubs EvidenceRecorded in full-loop test   | Medium   | Lines 312-325 of `test_agentic_capability_loop.py` use a handcrafted event dict, bypassing `sea_adapters.emit_evidence_recorded()` when `sea_adapters` is None |
| GodSpeed-Agent ledger empty                          | Low      | `ledgers/` directory contains only a README; no ledger data exists yet                                                                                         |
| Capability never reaches "metabolized"               | Low      | No test runs 3 usable settlements for the same capability                                                                                                      |

---

## 10. Required Tests

### Tests That Exist (confirmed by code inspection)

| Test                                                                  | File                                        | What it proves                                  |
| --------------------------------------------------------------------- | ------------------------------------------- | ----------------------------------------------- |
| `test_full_loop_end_to_end`                                           | `test_agentic_capability_loop.py:L258`      | 10-step happy path: WorkRequested → TwinUpdated |
| `test_full_fourteen_event_sequence_carries_domain_hash_and_trace_ids` | `test_agentic_capability_hardening.py:L296` | All 14 events emitted and ID-chained            |
| `test_unauthorized_private_corpus_produces_deny`                      | `test_agentic_capability_loop.py:L359`      | POL-ACL-002 blocks private corpus without scope |
| `test_missing_evidence_causes_coherence_break`                        | `test_agentic_capability_loop.py:L392`      | GodSpeed-Agent detects missing phase            |
| `test_authority_service_emits_authority_checked_and_coherence_break`  | `test_agentic_capability_hardening.py:L68`  | High-risk work blocked without context packet   |
| `test_godspeed_persists_agentic_loop_records_to_real_ledger`          | `test_agentic_capability_hardening.py:L122` | JSONL ledger persistence                        |
| `test_stale_proof_completion_escalates_with_coherence_break`          | `test_agentic_capability_hardening.py:L250` | POL-ACL-006 stale proof escalation              |
| `test_failure_gates_emit_expected_policy_decisions`                   | `test_agentic_capability_hardening.py:L228` | All 6 policy deny/escalate gates fire correctly |

### Tests That Are Missing (proposed)

**Test M1 — DomainForge Parse Round-Trip**

```python
def test_domainforge_parses_sea_and_manifest_matches_committed():
    """Invoke DomainForge CLI on agentic_capability_loop.sea; compare output IR to committed manifest."""
    result = subprocess.run(
        ["sea-core", "parse", str(DOMAIN_DIR / "agentic_capability_loop.sea"), "--format=manifest"],
        capture_output=True, text=True
    )
    live_manifest = json.loads(result.stdout)
    committed = json.loads((DOMAIN_DIR / "agentic_capability_loop.manifest.json").read_text())
    assert live_manifest["model"]["events"].keys() == committed["model"]["events"].keys()
```

**Test M2 — Content-Derived domain_model_hash**

```python
def test_domain_model_hash_is_derived_from_sea_file_content():
    """domain_model_hash in all adapters must hash the actual .sea file, not the namespace string."""
    sea_content = (DOMAIN_DIR / "agentic_capability_loop.sea").read_bytes()
    expected_hash = hashlib.sha256(sea_content).hexdigest()
    # Currently fails — all adapters hash the namespace string, not file content
    assert swe_adapters.DOMAIN_MODEL_HASH == expected_hash
```

**Test M3 — Context Kernel Rust Trait Implementation**

```python
def test_context_kernel_rust_has_concrete_context_agent_impl():
    """ck-mcp must have a concrete impl of ContextAgent, not just the trait."""
    lib_rs = (CONTEXT_KERNEL_ROOT / "crates/ck-mcp/src/lib.rs").read_text()
    assert "impl ContextAgent for" in lib_rs or "impl agentic_capability_loop::ContextAgent for" in lib_rs
```

**Test M4 — Semantic Pack Version Propagates Through Loop**

```python
def test_semantic_pack_version_propagates_through_all_events():
    """Every event in the loop must carry the same semantic pack version or hash."""
    events = [work, ctx_req, ctx_packet, authority, proof_started, proof_completed,
               evidence, settlement, capability, twin]
    hashes = {e["payload"].get("domain_model_hash") for e in events}
    assert len(hashes) == 1  # All must agree
    # AND the hash must match the actual .sea file content:
    sea_hash = hashlib.sha256(sea_content).hexdigest()
    assert hashes.pop() == sea_hash
```

**Test M5 — Capability Never Promoted from Simulation**
(Exists: `test_pol_acl_009_denies_simulation_capability_promotion` — but not in end-to-end loop)

---

## 11. Recommended Fixes in Dependency Order

### Fix 1 — Make `domain_model_hash` a content hash of the `.sea` file (Priority 1)

**Files:** All 4 adapter modules + CK Rust struct  
**Change:**

```python
# Replace in all adapters:
import hashlib
from pathlib import Path
_SEA_FILE = Path(__file__).parents[N] / "docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea"
DOMAIN_MODEL_HASH = hashlib.sha256(_SEA_FILE.read_bytes()).hexdigest()
```

This enables actual semantic pack versioning. All 5 adapters must agree on the path to the `.sea` file.

**Depends on:** Nothing. Independent fix.

### Fix 2 — Add a DomainForge parse round-trip test to CI (Priority 2)

**File:** `SWE_SEED/tests/test_agentic_capability_contracts.py` (already exists, check if it runs DomainForge CLI)  
**Change:** `test_agentic_capability_contracts.py` references DomainForge via CLI subprocess. Verify this test passes and add it to the CI gate.

**Depends on:** Fix 1 (so hash can be validated against live parse output).

### Fix 3 — Implement `ContextAgent` trait in `ck-mcp` (Priority 3)

**File:** `Context_Kernel_Service_MCP/crates/ck-app/` or new `ck-adapter-agentic-loop/`  
**Change:** Create a concrete struct `AclContextAgent` implementing `ContextAgent` that calls the existing corpus/search adapters to produce real citations.

**Depends on:** Understanding CK's internal corpus adapter architecture (`ck-adapter-filesystem`, `ck-adapter-github`, `ck-adapter-web`).

---

## 12. Final Confidence Level

| Layer                                    | Implementation | Tests                   | Live proof          |
| ---------------------------------------- | -------------- | ----------------------- | ------------------- |
| DomainForge (`.sea` parsing)             | Implemented    | Unit tests pass         | Not run in loop     |
| DomainForge (semantic pack output)       | Partial        | No round-trip test      | Not validated       |
| Context Kernel (Python adapter)          | Implemented    | Integration tests       | Not run live        |
| Context Kernel (Rust service)            | Stub only      | Field-check test        | Not run             |
| SEA-Forge (authority service)            | Implemented    | Hardening tests         | Not run live        |
| SWE_SEED (work + proof events)           | Implemented    | Full integration test   | Marked `live_proof` |
| GodSpeed-Agent (settlement + capability) | Implemented    | Hardening + ledger test | Not run end-to-end  |
| Loop feedback (semantic diff)            | Missing        | —                       | —                   |

**Overall:** `local-confidence` — the contract is well-defined and the adapter layer is verified, but no live service execution has been observed.

---

## Final Judgment

### Does the stack implement the intended agentic loop?

**Answer: Partially**

The adapter contract, event schema, and domain model are implemented and cross-repo integration-tested. The loop runs correctly in-process across all 4 repos. The DomainForge → semantic pack provenance bookend is missing; the Rust live service is missing.

### What is the strongest implemented path?

`SWE_SEED` emits `WorkRequested` → `ContextRequired` → `Context Kernel` (Python) responds with `ContextPacketCreated` + `ContextAuthorized` → `SEA-Forge` evaluates `evaluate_work_request()` with POL-ACL-001 → `SWE_SEED` emits `ProofStarted` + `ProofCompleted` → `SEA-Forge` emits `EvidenceRecorded` → `GodSpeed-Agent` emits `SettlementRecorded` + `CapabilityUpdated` + `TwinUpdated`. This 14-event sequence is verified by `test_full_fourteen_event_sequence_carries_domain_hash_and_trace_ids`.

### What is the weakest link?

**DomainForge's role in the loop.** DomainForge parses `.sea` files and emits IR, but it is not called at adapter runtime, the committed artifacts are not validated against the live parser in CI, and `domain_model_hash` is a constant that provides no semantic provenance. The "DomainForge supplies executable semantic meaning" claim is aspirational, not wired.

### What is the shared contract?

The actual shared contract is the **canonical event envelope** with the `agentic_capability_loop` namespace:

```json
{
  "event_id": "<uuid>",
  "event_type": "<one of 14 named events>",
  "namespace": "agentic_capability_loop",
  "occurred_at": "<iso8601>",
  "payload": { "domain_model_hash": "<constant>", ...fields... }
}
```

All 4 repositories emit and consume this same envelope shape, linked by `work_request_id`.

### What must be fixed first?

1. **`domain_model_hash`** — make it a content hash of the `.sea` file so semantic pack versioning is real.
2. **DomainForge round-trip test in CI** — prevent `.sea` drift from adapter code.
3. **Context Kernel Rust `ContextAgent` impl** — without this, the production path is broken.

### What test would prove the loop?

`test_full_fourteen_event_sequence_carries_domain_hash_and_trace_ids` in `SWE_SEED/tests/test_agentic_capability_hardening.py` is the closest existing proof. Augmenting it with:

1. A DomainForge CLI call to parse the `.sea` file first
2. Deriving `domain_model_hash` from the parsed output
3. Verifying all 14 events carry the same derived hash

...would constitute a full loop proof. This requires Fix 1 and Fix 2 to be in place first.
