# MCPGate in SWE_SEED

Status: Draft v3.0, reconciled into the SWE_SEED architecture.

Owner: SWE_Seed capability and gateway plane.

Scope: Project-native contract for the MCP gateway delta. This spec folds the useful
standalone MCPGate outcomes into the existing SWE_SEED stack instead of defining a separate
product.

Related specs: `0002-swe-seed-centralization-layer.md`, `0003-capability-registry.md`,
`0004-host-adapter-contract.md`, `0005-normalized-hook-runtime.md`,
`0006-mcp-gateway-integration.md`, `0007-skill-ingestion-and-scan-gate.md`,
`0011-sea-loop-federation.md`, `0012-existing-harness-reconciliation.md`,
`0013-eval-and-proof.md`, `0015-context-budget-plane.md`,
`0017-fabricator-layer.md`, and `0018-layer-boundary-governance.md`.

## Normative Language

The key words `MUST`, `MUST NOT`, `REQUIRED`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`,
`MAY`, and `OPTIONAL` in this document are to be interpreted as described in RFC 2119.

`Implementation-defined` means the behavior is part of the implementation contract, but this
specification does not prescribe one universal policy. Implementations MUST document the
selected behavior.

## 1. Position in the Stack

MCPGate is the strengthened runtime form of spec 0006. It belongs to the SWE_Seed layer as
the capability/tool plane:

```text
SWE_Seed: capability registry, gateway, host adapters, provenance, boundary governance
  -> Harness: route cards, permission policy, context budget, hooks, eval, proof, traces
      -> Fabricator: product-to-prototype semantic chain and handoff artifacts
```

MCPGate is not a fourth layer. It consumes the SWE_Seed registry, enforces Harness policy,
emits Harness proof evidence, and can feed Fabricator only through traceable artifacts. When
federation is online, every cross-system interaction goes through the semantic event
envelope from spec 0011. Local audit is the durable local copy; the envelope is the
federated integration contract.

The existing local gateway concept remains the base: one SWE_SEED endpoint fronts registered
MCP servers, exposes compact discovery meta-tools, and routes invocations through
PermissionPolicy and approval gates. This spec adds the delta from the standalone MCPGate
draft that the current project does not yet express strongly enough: atomic governance,
session sandboxing, hot reload, health and breaker state, strict call-time integrity
pinning, OpenAPI import, and auditable request outcomes.

## 2. Non-Goals

- MCPGate is not an LLM or chat-completions gateway.
- MCPGate does not replace the capability registry. `MCPServer` and imported tool
  definitions remain registry artifacts governed by specs 0003, 0007, 0009, and 0018.
- MCPGate does not own SEA federation. The only external SEA boundary is
  `swe_seed::federation` from spec 0011.
- MCPGate does not make Context Kernel, SEA-Forge, or GodSpeed-Agent mandatory. With
  `federation.enabled=false`, MCPGate must run fully locally and make zero external calls.
- MCPGate does not move Fabricator into the request path. Fabricator consumes proven
  outcomes and generated artifacts, not live tool traffic.
- MCPGate does not persist backend plaintext secrets. It resolves configured references and
  redacts all audit and hook output.

## 3. Existing Project Baseline

The project already has these native surfaces:

| Surface                          | Existing owner            | Contract MCPGate must use                                                                         |
| -------------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------- |
| Capability inventory             | SWE_Seed registry         | `MCPServer`, `LayerCapability`, `SeedPackageManifest`, `ArtifactMetadata`                         |
| Gateway discovery and invocation | Spec 0006                 | `swe_seed_list_servers`, `swe_seed_list_tools`, `swe_seed_search_tools`, `swe_seed_invoke`        |
| Tool gating                      | Harness hooks and gateway | `PermissionPolicy`, `HookPolicy`, approval gates, `PreToolUse` fallback to gateway                |
| Context discipline               | Harness context plane     | `ContextBudget`, `ContextPack`, `budget-policy.yaml`                                              |
| Federation                       | `swe_seed::federation`    | `Envelope`, `domain_model_hash`, `AuthorityChecked`, `ContextPacketCreated`, `SettlementRecorded` |
| Proof                            | Harness eval subsystem    | `EvalSpec`, `EvalResult`, `ProofRecord`, live-pass-only promotion                                 |
| Product handoff                  | Fabricator                | `AgentTask`, `FabricatorEvalSpec`, `FabricatorProofRecord`, `TraceabilityLink`                    |

MCPGate MUST extend these surfaces. It MUST NOT introduce parallel names for the same
concepts.

## 4. Delta to Incorporate

The standalone MCPGate draft contributes these project-useful outcomes:

| Delta                                                | Current project gap                                                                                  | Project-native destination                                                                                                |
| ---------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Response-or-typed-error plus audit for every request | 0006 says route and gate, but does not define request completion evidence or online outcome eventing | Local gateway request outcome, Harness `ProofRecord` evidence, and semantic-envelope projection when federation is online |
| Atomic budget and session enforcement                | PermissionPolicy exists, but no request counter model is specified                                   | Gateway governance state, checked by an `EvalCheck` proof                                                                 |
| Per-session sandboxing                               | Hook payloads include session ids, but gateway sessions are underspecified                           | Gateway `GatewaySession` state and audit correlation                                                                      |
| Hot reload with last-known-good rollback             | Registry hot reload is deferred and gateway reload semantics are absent                              | Gateway config snapshot contract; registry remains source of truth                                                        |
| Backend health, circuit breaker, bounded retry       | Doctor checks reachability, but runtime breaker state is absent                                      | Gateway runtime state plus doctor and metrics                                                                             |
| Deterministic namespacing and call-time hash pin     | 0006 requires namespacing, 0007 requires source hashes                                               | `GatewayCatalogEntry` derives from registry and verifies pinned definition hashes at call time                            |
| Operator metrics and health endpoints                | Doctor exists, but live gateway observability is absent                                              | Gateway runtime endpoints and hook/audit logs                                                                             |
| OpenAPI import as MCP tools                          | Skill ingestion exists; REST/OpenAPI import is not specified                                         | Capability ingestion extension producing `MCPServer` plus `GatewayCatalogEntry` projections                               |
| Tunnel/private exposure                              | Host adapters generate configs, but network exposure policy is absent                                | Gateway listener policy with loopback default and explicit tunnel mode                                                    |

These deltas are in scope only where they strengthen SWE_SEED's existing capability/tool
plane. They do not authorize a standalone daemon that bypasses the registry, PermissionPolicy,
context budget, proof gate, or federation flags.

## 5. Outcome Contract

For each inbound MCP or JSON-RPC request, MCPGate MUST produce exactly one of:

1. the owning backend's response, with the original request id preserved, or
2. a typed gateway error.

In both cases MCPGate MUST write a local audit record before the request is considered
complete. Notifications that do not require a response still require a local audit record.

When `federation.enabled=true` and the matching plane is enabled, MCPGate MUST also project
the same decision through the semantic envelope. The local audit record is not a substitute
for the envelope; it is the local evidence copy that cites the envelope id or records that
dispatch was suppressed.

The audit record MUST include:

- `request_id`
- `session_id`
- `client_id`
- `method`
- `namespaced_name`, when the request targets a tool, resource, or prompt
- `backend`
- `decision`
- `reason_code`
- `latency_ms`
- `cost_delta`
- `route_card_id`, when invoked during routed harness work
- `domain_model_hash_source`, when federation is enabled or an envelope was consumed
- `semantic_envelope_id`, when an online outcome was emitted or consumed
- `semantic_envelope_event_type`, when an online outcome was emitted or consumed

Secrets MUST be redacted using the hook runtime redaction rules. A request is incomplete if
the response was returned but the audit record was not written.

## 6. Capability Claim

After repeated successful use, SWE_SEED operators should be able to add, govern, observe, and
verify heterogeneous MCP and REST tool backends through one project-native policy surface,
without editing each host's config and without depending on SEA services.

The capability claim is proven only when:

- a backend can be added or removed through the registry and projected through host adapters
  without hand-editing every client;
- `swe_seed_invoke` applies PermissionPolicy, scan status, approval flags, session limits,
  and budget limits before forwarding;
- discovery returns a compact, namespaced catalog filtered by the caller's effective policy;
- a live proof shows concurrent calls cannot overrun session or budget counters;
- every allow, deny, and backend failure produces local audit evidence and, when online,
  the configured semantic-envelope outcome event;
- with `federation.enabled=false`, the same local request flow runs with zero external calls.

The claim is not proven by a single backend demo, by config fields without enforced checks, or
by successful forwarding without audit and proof evidence.

## 7. Domain Model

MCPGate adds gateway-runtime models that derive from the existing registry and Harness
contracts.

### GatewayBackend

`GatewayBackend` is the live runtime view of a registered `MCPServer`.

Required fields:

- `id`: registry capability id.
- `namespace`: stable prefix used in catalog names.
- `transport`: `stdio`, `http`, `sse`, or `websocket`.
- `command_or_url`: resolved from the registry entry.
- `profile_tags`: inherited from the registry entry.
- `health_state`: `healthy`, `degraded`, or `down`.
- `breaker_state`: `closed`, `open`, or `half_open`.
- `provenance_ref`: `ArtifactMetadata` or provenance record id.

The namespace MUST be unique. A namespaced call MUST route only to its owning backend.
MCPGate MUST NOT fail over a stateful namespaced tool call to another backend.

### GatewayCatalogEntry

`GatewayCatalogEntry` is the runtime projection of a tool, resource, or prompt.

Required fields:

- `kind`: `tool`, `resource`, or `prompt`.
- `namespaced_name`: `<backend_namespace>.<local_name>`.
- `backend_id`: owning `GatewayBackend`.
- `wire_definition`: canonical JSON definition sent to clients.
- `definition_hash`: SHA-256 over canonical JSON.
- `source_hash`: source hash from provenance when available.
- `policy_tags`: tags used by PermissionPolicy and approval gates.

Optional fields:

- `pinned_hash`: an operator-pinned expected `definition_hash`.
- `input_schema`: JSON Schema for tool parameters.

MCPGate MUST verify `pinned_hash` at call time when it is present. A hash mismatch returns a
typed error and blocks forwarding.

### GatewaySession

`GatewaySession` is the per-client runtime sandbox.

Required fields:

- `session_id`: server-generated id with at least 128 bits of entropy.
- `client_id`: resolved caller identity.
- `route_card_id`: optional active Harness route.
- `allowed_backends`: backend namespaces allowed for this session.
- `state`: `active`, `limit_exceeded`, or `terminated`.
- `started_at`
- `call_count`
- `max_calls`
- `max_duration_secs`

Clients MUST NOT supply or override `session_id`. `limit_exceeded` is a governance state,
not a backend failure.

### GatewayGovernanceState

`GatewayGovernanceState` stores authoritative in-memory counters.

Required fields:

- per-session call counts.
- per-client request or byte counters.
- per-tool request or byte counters.
- UTC window start.
- effective limits after PermissionPolicy and config narrowing.

The gateway MUST check and increment session and budget counters in one critical section for
the affected session, client, and tool. Before a request can advance, the counter delta MUST
be durably recorded in a write-ahead log, synchronous journal, or equivalent handoff that can
be replayed after a crash. Snapshot writes remain asynchronous and atomic, and MUST NOT sit
on the request path; recovery MUST replay the durable counter log before accepting traffic so
session, client, and tool counters cannot be undercounted.

## 8. Request Flow

The gateway request flow is:

```text
frame request
  -> validate JSON-RPC and size/depth limits
  -> resolve GatewaySession and client identity
  -> resolve GatewayCatalogEntry by namespaced name
  -> apply PermissionPolicy and approval gates
  -> apply local scan/provenance status
  -> optionally consult SEA-Forge authority through federation
  -> check and increment session and budget counters atomically
  -> verify call-time definition hash pin
  -> route to the owning backend through breaker and concurrency gates
  -> return backend response or typed gateway error
  -> write local audit, hook event, metrics, and proof evidence refs
  -> if federation online, emit or consume the configured semantic-envelope event
```

Every gate has a blocked path. A blocked path returns a typed error, writes audit evidence,
and does not forward the request.

## 9. Semantic Envelope Projection

MCPGate MUST treat the semantic envelope as the only online integration surface. It MUST NOT
call Context Kernel, SEA-Forge, or GodSpeed-Agent through side channels.

When federation is disabled, this section is inactive and no envelope is constructed,
dispatched, consumed, or awaited.

When federation is enabled, MCPGate maps local gateway concepts to envelope events:

| Gateway concept                            | Envelope event                              | Direction       | Purpose                                                |
| ------------------------------------------ | ------------------------------------------- | --------------- | ------------------------------------------------------ |
| Gateway request enters routed work         | `WorkRequested` or existing work request id | emit or attach  | Correlate gateway traffic to the agentic loop          |
| Backend or tool route selected             | `RouteSelected`                             | emit            | Publish the selected backend/tool route                |
| Context needed for discovery or invocation | `ContextRequired` / `ContextPacketCreated`  | emit / consume  | Request and receive Context Kernel citations           |
| Risky tool authority decision              | `AuthorityChecked`                          | consume         | Receive SEA-Forge allow, deny, or escalate             |
| Gateway proof command starts/completes     | `ProofStarted` / `ProofCompleted`           | emit            | Publish live proof state for gateway capability claims |
| Final request outcome                      | `SettlementRecorded`                        | emit or consume | Publish or reconcile the auditable request outcome     |

The local audit record MUST include the envelope id for every emitted or consumed event. If
the configured sink suppresses or fails to dispatch an event, the audit record MUST record
that disposition. A federated request outcome is incomplete for online integration purposes
until the required envelope disposition is recorded.

Envelope payloads MUST carry ids and evidence references, not raw tool arguments, raw backend
responses, credentials, or full context packets. `domain_model_hash` drift rejects incoming
envelopes before their payloads affect gateway routing, authority, context, proof, or
settlement.

## 10. Permission, Hooks, and Authority

MCPGate is the enforcement fallback for hosts that cannot run `PreToolUse` hooks. It MUST use
the same `PermissionPolicy` semantics as the Harness hook runtime:

- deny overrides allow;
- dangerous tool groups require explicit approval;
- unscanned or unproven risky capabilities fail closed when policy requires scan or live
  proof;
- external authority can only participate through `swe_seed::federation`.

When `federation.authority=local`, MCPGate decides locally and ignores incoming
`AuthorityChecked` envelopes.

When `federation.authority=delegate`, MCPGate waits for `AuthorityChecked` before risky
actions. `deny` blocks, `escalate` requires human approval, and timeout fails closed for
dangerous actions.

When `federation.authority=hybrid`, MCPGate applies local policy first and consults
SEA-Forge only for configured escalation cases. External `allow` MUST NOT override a local
deny unless `authority.allow_external_override=true`.

## 11. Context Kernel Integration

MCPGate MUST NOT create a separate context plane. It uses `ContextBudget`, `ContextPack`, and
`budget-policy.yaml`.

In local mode, discovery and invocation context comes from the routed `ContextPack` and the
compact gateway catalog. Tool schemas MUST NOT be dumped wholesale into the model context.

When `federation.context=external`, MCPGate emits `ContextRequired` and consumes
`ContextPacketCreated` through `swe_seed::federation`. That pair is the only Context Kernel
interaction. Timeout or unavailable envelope delivery MUST fail soft to the local
`ContextPack`, with the timeout recorded in local audit.

When `federation.context=hybrid`, MCPGate may merge citations from a
`ContextPacketCreated` envelope with local salient facts. The merged context MUST cite its
sources and stay within the context budget.

Incoming context envelopes MUST pass the `domain_model_hash` drift guard before use.

## 12. SEA Envelope Boundary

MCPGate never builds its own federation protocol. It calls only the existing federation
module:

- emit `RouteSelected` when a gateway route choice matters to the agentic loop;
- emit `ContextRequired` only for context federation modes;
- consume `AuthorityChecked` only for authority delegation modes;
- emit `SettlementRecorded` for completed gateway outcomes only when settlement federation is
  enabled;
- consume `SettlementRecorded` only to reconcile an external settlement with a local audit or
  proof record;
- emit `ProofStarted` and `ProofCompleted` only for live proof commands that run through the
  Harness proof subsystem.

With `federation.enabled=false`, MCPGate MUST NOT construct, dispatch, consume, or wait for
any SEA envelope.

## 13. GodSpeed-Agent Settlement

GodSpeed-Agent is a settlement and learning consumer, not a runtime dependency.

When `federation.settlement=off`, MCPGate records local audit and proof evidence only.

When settlement is enabled, a completed request outcome emits `SettlementRecorded` through the
federation module. If GodSpeed-Agent returns or later produces settlement state, MCPGate
consumes it only as another `SettlementRecorded` envelope. Settlement MAY annotate later
learning or capability quality records, but it MUST NOT retroactively change the request
result.

GodSpeed settlement can support the capability claim only after a live proof exists. A
simulation or settlement-only success cannot activate or promote a capability.

## 14. Fabricator Integration

MCPGate interacts with Fabricator through artifacts, not live control.

Allowed interactions:

- A Fabricator `AgentTask` may require tools served by MCPGate.
- Gateway audit records may become evidence refs in `FabricatorProofRecord`.
- Imported OpenAPI or MCP tools may be represented as requirements, scenarios, or SDS
  dependencies only when linked by `TraceabilityLink`.
- Gateway proof checks may be embedded in `FabricatorEvalSpec` when the prototype depends on
  those tools.

Forbidden interactions:

- MCPGate MUST NOT mutate Fabricator chain artifacts during request routing.
- MCPGate MUST NOT treat a backend response as a satisfied product requirement unless a
  Fabricator proof record cites it.
- MCPGate MUST NOT bypass `ValidateSemanticChain`.

## 15. Configuration Contract

The registry remains the source of truth. A gateway runtime config is a compiled view of the
registry, profile policy, and local operator settings.

Config precedence:

1. CLI flags for the current run.
2. Project gateway config under `.swe-seed/`.
3. Registry and manifest data.
4. Built-in defaults.

Required defaults:

- Listener binds to `127.0.0.1`.
- Non-loopback bind requires TLS or explicit tunnel mode.
- Federation is disabled.
- Content inspection is off unless enabled per policy.
- Unknown or unscanned dangerous tools are blocked.

Live reload is a post-v0.1 MCPGate runtime requirement. Reload MUST parse and validate a new
snapshot before applying it. In-flight requests continue on the snapshot they started with.
Invalid reload keeps the last-known-good snapshot and emits `invalid_reload_error`.

## 16. Import Contract

MCPGate may import two categories into the registry:

1. MCP backends, as `MCPServer` entries.
2. OpenAPI operations, as generated gateway catalog entries backed by HTTP targets.

OpenAPI import MUST map:

- `operationId` to local tool name;
- parameter and request body schemas to `input_schema`;
- server URL to backend endpoint;
- source document hash to provenance metadata;
- generated tool names to deterministic namespaced names.

Import MUST be idempotent. Re-importing unchanged source documents MUST produce the same
hashes and no semantic diff.

Imported skills remain governed by SkillIR and the skill ingestion pipeline. If a skill
exposes MCP tools, those tools enter the same gateway path as all other catalog entries.

## 17. Observability and Proof

MCPGate MUST expose or produce:

- local audit records for every decision;
- semantic-envelope dispositions for every online decision whose federation plane is enabled;
- hook runtime events for `PreToolUse` and `PostToolUse` equivalents;
- gateway metrics for request count, request latency, backend health, breaker state, active
  sessions, and governance counters;
- health status using `healthy`, `degraded`, and `down`;
- proof artifacts that can be cited by `ProofRecord` and Fabricator proof records.

Required proof commands for an implementation:

```text
swe-seed gateway doctor
swe-seed gateway proof --routing
swe-seed gateway proof --concurrency 64
swe-seed gateway proof --reload
swe-seed validate
swe-seed doctor
```

The existing Python/Rust compatibility commands may stand in until the target CLI exists:

```text
just harness-validate
just harness-validate
just ci
```

Proof passes only when command exit code is 0 and the output shows no counter overrun, no
silent audit omission, no missing online envelope disposition, no namespaced reroute, no
secret leak, and no external call in standalone mode.

## 18. Failure Model

MCPGate failure behavior is fail-safe and scoped.

Backend failure:

- timeout, connection refused, process exit, or malformed backend response opens or advances
  the backend breaker;
- affected tools return `BackendUnavailable`;
- other backends keep serving;
- namespaced calls are never rerouted.

Policy failure:

- failed auth, denied scope, missing approval, scan block, hash mismatch, budget exceeded, or
  session limit exceeded returns a typed error;
- the request is not forwarded;
- audit records the reason.

Config failure:

- startup config failure exits non-zero;
- reload failure preserves last-known-good config;
- missing required credential reference blocks only affected backend startup when possible.

Evidence failure:

- if the audit record cannot be written, the request MUST fail closed with a typed internal
  error;
- if online envelope dispatch is required and no disposition can be recorded, the local
  request may still complete only when the plane's fail-soft rule allows it; otherwise it
  fails closed with a typed federation error;
- metrics failure is logged and retried, but does not by itself mark an allowed request
  failed.

## 19. Test Matrix

| Area                 | Required test                                 | Expected result                                                                      |
| -------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------ |
| Compact discovery    | list/search tools under a small catalog limit | filtered namespaced catalog; dropped entries logged deterministically                |
| PermissionPolicy     | readonly invokes a write tool                 | typed denial, no forward, audit reason                                               |
| Atomic governance    | concurrent calls at budget edge               | counters never exceed effective limits                                               |
| Session sandbox      | max call count reached                        | session enters `limit_exceeded`; later calls denied                                  |
| Routing              | backend down for `backend.tool`               | `BackendUnavailable`; no alternate backend receives call                             |
| Hash pin             | catalog definition changes after list         | call-time `CapabilityHashMismatch`                                                   |
| Hot reload           | valid then invalid reload under load          | valid applies to future calls; invalid keeps last-known-good; in-flight calls finish |
| Semantic envelope    | online request emits configured outcome event | local audit cites envelope id and event type                                         |
| Context Kernel       | `federation.context=external` timeout         | local ContextPack used, envelope timeout recorded                                    |
| SEA Forge            | external deny in delegate mode                | `AuthorityChecked` denial blocks risky request                                       |
| GodSpeed             | settlement enabled after live proof           | `SettlementRecorded` cites proof; request result unchanged                           |
| Standalone invariant | federation disabled and SEA unreachable       | zero external calls; local gateway still works                                       |
| Fabricator evidence  | gateway result cited by prototype proof       | `FabricatorProofRecord` has evidence ref and chain link                              |

## 20. Definition of Done

MCPGate is implemented in SWE_SEED only when:

- the gateway runtime reads from the registry or manifest and does not maintain a competing
  capability source of truth;
- compact discovery, namespaced routing, PermissionPolicy, approval gates, and scan/provenance
  gates work together;
- request completion requires response-or-typed-error plus local audit;
- online request completion records the required semantic-envelope disposition;
- session and budget counters are atomic under concurrency;
- per-backend health and circuit breakers prevent unbounded retries and never reroute
  namespaced tools;
- hot reload uses validated snapshots and last-known-good rollback;
- context uses `ContextBudget` and optional Context Kernel federation only through spec 0011;
- SEA-Forge and GodSpeed integration remain flag-gated and default-off;
- Fabricator integration happens only through traceable proof artifacts;
- standalone mode is proven to make zero external calls;
- proof commands pass or skipped checks are justified with recorded risks.

## 21. Migration Notes

Spec 0006 remains the v0.1 minimum: generated host MCP config plus a minimal local router.
This spec defines the stronger MCPGate target. Implementation should stage it as follows:

1. Preserve 0006 meta-tools and PermissionPolicy behavior.
2. Add local audit records, typed errors, and request completion proof.
3. Add GatewaySession and atomic governance counters.
4. Add backend health, breaker state, and metrics.
5. Add validated snapshot reload.
6. Add OpenAPI import.
7. Add semantic-envelope projection for online request outcomes through
   `swe_seed::federation`.
8. Add Fabricator evidence links where product prototypes depend on gateway tools.

Each stage must add or update an `EvalCheck` or deterministic test that can fail before the
implementation exists.
