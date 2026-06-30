# 0011 — SEA Loop Federation (Optional Envelope Boundary)

> **Reconciled by [0012](0012-existing-harness-reconciliation.md) (authoritative).** Unchanged
> in substance. Note the loop's `ProofStarted/ProofCompleted` map to the proven **ProofRecord**/
> **EvalResult** (0013); proof emission is the same live-pass-only artifact. Federation remains
> optional and default-off; the 3-layer inner stack always works standalone.

## Purpose

Define how SWE_Seed participates in the larger agentic capability loop (the SEA federation:
Context Kernel, SEA-Forge, GodSpeed-Agent) **without ever depending on it**. SWE_Seed is a
self-contained node. Federation happens only at the **semantic event envelope**, which
SWE_Seed can optionally take as input and produce as output. All external delegation is
**behind config flags, off by default.**

This spec reconciles specs 0002–0010 with the existing `agentic_capability_loop/` adapter
and defines its Rust port.

## Non-goals

- SWE_Seed does not require SEA-Forge, Context Kernel, or GodSpeed-Agent to function.
- Not a re-design of the SEA event contract — SWE_Seed conforms to the existing envelope.
- Federation does not move governance authority out of SWE_Seed; it lets SWE_Seed
  *optionally delegate or escalate*.

## The sovereign inner loop (hard invariant)

```
ingest → normalize → register → project → verify
```

With `[federation].enabled = false` (the default), **no external call is ever made**. Every
gate resolves locally: local authority (profiles + scan gate, 0006/0007), local context
(ContextBuild hook, 0005), local proof (doctor/verify, 0008). A standalone SWE_Seed
completes the full loop offline. **Any code path that can break the inner loop when the
federation is unreachable is a defect.**

## Prior-art evidence (sources removed, unnamed)

| Prior-art (removed) | File/path | Lines or section | Pattern observed | SWE_Seed interpretation |
|---|---|---|---|---|
| (first-party) | `agentic_capability_loop/adapters.py` | `emit_*` / `consume_*` | Canonical envelope `{event_id,event_type,namespace,occurred_at,payload}` w/ `domain_model_hash`; emit WorkRequested/ContextRequired/RouteSelected/ProofStarted/ProofCompleted; consume ContextPacketCreated/AuthorityChecked/SettlementRecorded | Port 1:1 to Rust `swe_seed::federation`; this is the only integration surface |
| (first-party) | `agentic_capability_loop/adapters.py:29-95` | `_load_hash()` | Resolve `sea_file_hash` from SEA manifest via `SEA_ROOT`/`SEA_MANIFEST_PATH`/marker walk, fallback hash + warn | Same resolution order in Rust; fallback keeps standalone mode working |
| (first-party) | `tests/test_agentic_capability_{loop,contracts,hardening}.py` | files | Contract + hardening tests for envelope | Rust port must keep these green (or port them) — cross-repo consistency |

> Note: `agentic_capability_loop/` is **first-party SWE_Seed code**, not a read-only
> reference repo. Porting/refactoring it is permitted; clean-room rules (0009) do not apply.

## SWE_Seed requirements

### 1. Canonical envelope (I/O only)

```rust
struct Envelope {
    event_id: Uuid,
    event_type: String,        // "WorkRequested", ...
    namespace: String,         // "agentic_capability_loop"
    occurred_at: String,       // RFC3339 UTC
    payload: Value,            // includes "domain_model_hash"
}
```

`domain_model_hash` resolved exactly as the Python adapter does (env → manifest → fallback
+ warn). In standalone mode the envelope is never constructed; it is purely a boundary
artifact for `emit_envelope`/`consume_envelope`.

### 2. Config flags (off by default)

```toml
[federation]
enabled         = false   # master switch; false ⇒ fully standalone, no external calls
emit_envelope   = false   # produce canonical events as output (stdout/bus/file sink)
consume_envelope = false  # accept canonical events as input

[federation.authority]    # SEA-Forge
mode = "local"            # local | delegate | hybrid
#   local    : SWE_Seed decides (profiles + scan gate). No external call.
#   delegate : require AuthorityChecked(allow|deny|escalate) before risky action.
#   hybrid   : local decision first; on local "escalate" or risk≥threshold, consult AuthorityChecked.

[federation.context]      # Context Kernel
mode = "local"            # local | external | hybrid
#   local    : ContextBuild builds locally (0005).
#   external : emit ContextRequired, consume ContextPacketCreated; fail-soft to local on timeout.
#   hybrid   : local context + external citations merged.

[federation.settlement]   # GodSpeed-Agent
mode = "off"              # off | emit | consume | both
```

`enabled=false` forces every sub-mode to behave as `local`/`off` regardless of their value
(master switch wins). When `enabled=true`, each plane is independently togglable.

### 3. Concept ↔ event mapping (unification)

| SWE_Seed local concept (spec) | Loop event | Direction | Gated by |
|---|---|---|---|
| Work intake / `TaskStart` (0005) | `WorkRequested` | emit | `emit_envelope` |
| `ContextBuild` hook (0005) | `ContextRequired` / `ContextPacketCreated` | emit / consume | `federation.context = external\|hybrid` |
| `Projection` + gateway route choice (0004/0006) | `RouteSelected` | emit | `emit_envelope` |
| `doctor`/verify proof run (0008) | `ProofStarted` / `ProofCompleted` | emit | `emit_envelope` |
| `SecurityGate`/`PreToolUse` decision (0005/0007) | `AuthorityChecked` (allow/deny/escalate) | consume | `federation.authority = delegate\|hybrid` |
| Final outcome record | `SettlementRecorded` | consume/emit | `federation.settlement` |

The local concept is always authoritative when the corresponding flag is off. When on, the
event becomes an additional input/output, never a replacement for the local fallback.

### 4. Proof semantics (resolves "capability" ambiguity)

The loop's "capability promotion" rule — *simulation results cannot promote capabilities* —
unifies with SWE_Seed activation gates:
- A capability (skill/MCP/etc., 0003) is **activated/projected** only on a **live** proof
  (`proof_type = "live"`, `result = "pass"`) — never on `"simulation"`.
- `doctor`/verify (0008) is the local proof engine; it emits `ProofStarted/ProofCompleted`
  when `emit_envelope = true`.
- This is the same fail-closed posture as the scan gate (0007).

To avoid the terminology collision: in SWE_Seed prose, "capability" = installable unit
(0003); the loop's "promotion of a capability" = SWE_Seed **activation** of that unit.
"route" (loop) ≠ gateway tool-routing (0006) — call the loop concept **route card**.

### 5. Authority delegation (resolves the authority conflict)

- `federation.authority = local` (default): SWE_Seed's profiles + scan gate are the sole
  authority. Specs 0006/0007 unchanged.
- `delegate`: before any action whose tool/profile is risk-tagged, SWE_Seed waits for an
  `AuthorityChecked` envelope; `deny` blocks, `escalate` requires human approval, `allow`
  proceeds. If no envelope arrives within timeout → fail-closed (treat as deny) for
  `dangerous`, fall back to local decision otherwise.
- `hybrid`: local decision runs first; a local `escalate`/threshold breach consults
  `AuthorityChecked`.

## CLI behavior, if applicable

```
swe-seed run --federation off            # explicit standalone (default)
swe-seed run --federation on             # enable envelope I/O per config
swe-seed emit --event WorkRequested ...   # produce an envelope (testing/integration)
swe-seed consume < envelope.json          # ingest an envelope
swe-seed federation status                # show flags + resolved domain_model_hash
```

Flags override config; absent flags use `[federation]` config; absent config = standalone.

## Generated files, if applicable

None new on the local side. When `emit_envelope=true`, events go to the configured sink
(stdout / file / bus URL) — never written into host config files (0004).

## Rust module boundaries

```
swe_seed::federation
  ├─ envelope      # Envelope struct, domain_model_hash resolution (port of _load_hash)
  ├─ emit          # emit_work_requested / context_required / route_selected / proof_*
  ├─ consume       # consume_context_packet_created / authority_checked / settlement_recorded
  └─ flags         # FederationConfig, master-switch precedence
```

`federation` is the **only** module that touches the SEA contract. Core modules
(`capability`, `hooks`, `gateway`, `doctor`, `security`) call into `federation` only
through narrow hooks guarded by flags; with flags off they never reference it. The Python
`agentic_capability_loop/` package is replaced by this module (port), keeping the same
event types/payload keys so existing contract tests carry over.

## Security and provenance considerations

- Federation cannot weaken a local gate: external `AuthorityChecked` can make a decision
  *stricter* (deny/escalate) or, only in `delegate`/`hybrid`, grant `allow` — but local
  `deny` from profile/scan is never overridden by an external `allow` unless config
  explicitly opts in (`authority.allow_external_override = false` default).
- `domain_model_hash` mismatch between SWE_Seed and an incoming envelope ⇒ reject the
  envelope (cross-repo drift guard) and warn.
- Emitted envelopes carry no secrets; payloads use ids/refs (e.g. `output_ref`), not raw
  content.

## Tests

- **Standalone invariant**: with `federation.enabled=false` and SEA unreachable, the full
  inner loop (add→sync→doctor→rollback) passes; assert zero external calls attempted.
- **Envelope round-trip**: emit → serialize → consume reproduces payload; `domain_model_hash`
  resolution matches the Python adapter on identical inputs (port-parity test).
- **Authority modes**: `delegate` blocks on `deny`, escalates on `escalate`, fail-closed on
  timeout for dangerous; `local` ignores envelopes entirely.
- **Proof gate**: a `simulation` proof never activates a capability; `live`+`pass` does.
- **Drift guard**: mismatched `domain_model_hash` envelope rejected.
- Port the existing `tests/test_agentic_capability_*.py` contracts to Rust (or keep them
  running against the Rust binary via a thin harness).

## Decisions

- **Envelope transport**: **file + stdout** in v0.1 (`emit` writes to a sink path and/or
  stdout; `consume` reads a file/stdin). A message bus is deferred — not needed to prove the
  contract.
- **`domain_model_hash` with no SEA present**: use the documented fallback
  `sha256("agentic_capability_loop")` **+ warn**. This is the existing adapter's behavior
  and is what keeps standalone mode working — it is the intended design, not a stopgap.
- **`SettlementRecorded`**: **consume / log-only** in v0.1. No feedback into SWE_Seed state
  (no reputation/scoring) until there is a concrete consumer for it.

> Note: the Rust port reproduces the *contract* (event types, payload keys, hash-resolution
> order) of the now-deleted Python `agentic_capability_loop/` — which is **first-party**
> SWE_Seed code, freely reimplementable. No reference-repo access is needed.

## Acceptance criteria

- [ ] `federation.enabled=false` ⇒ provably zero external dependencies; inner loop fully
      works.
- [ ] Every external delegation (authority/context/settlement) is behind a flag, default off.
- [ ] Canonical envelope emit/consume matches the existing adapter contract (port-parity).
- [ ] Concept↔event mapping and terminology (capability/route/proof) unified across
      specs 0002–0010.
- [ ] Python `agentic_capability_loop/` superseded by Rust `swe_seed::federation`.
