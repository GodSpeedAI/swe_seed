# gateway

## Overview

MCPGate runtime models, gates, and server (spec 0020). The gateway forwards
JSON-RPC 2.0 to registered MCP backends through a fail-closed pipeline
(`validate_request` -> policy/integrity gates -> governance counters -> route ->
audit), projecting the registry (`MCPServer`) into runtime views without
becoming a second capability store. Every request completes with a backend
response OR a typed `GatewayError` carrying a stable `reason_code`, and one
redacted audit record is written before a request is considered complete.

## Interface

### Domain model and typed failure model

- `GatewayTransport { Stdio, Http, Sse, Websocket }`, `HealthState { Healthy,
Degraded, Down }`, `BreakerState { Closed, Open, HalfOpen }`, `CatalogKind {
Tool, Resource, Prompt }`, `SessionState { Active, LimitExceeded, Terminated }`
  [mod.rs:GatewayTransport], [mod.rs:HealthState], [mod.rs:BreakerState],
  [mod.rs:CatalogKind], [mod.rs:SessionState]
- `GatewayBackend`, `GatewayCatalogEntry` (with `definition_hash` SHA-256 over
  canonical `wire_definition` and operator `pinned_hash`),
  `GatewaySession`, `GatewayEffectiveLimits`, `GatewayGovernanceState`
  [mod.rs:GatewayBackend], [mod.rs:GatewayCatalogEntry], [mod.rs:GatewaySession],
  [mod.rs:GatewayEffectiveLimits], [mod.rs:GatewayGovernanceState]
- `pub enum GatewayError` (15 variants incl. `BackendUnavailable`,
  `CapabilityHashMismatch`, `ScopeDenied`, `ApprovalRequired`, `ScanBlocked`,
  `ProvenanceMissing`, `BudgetExceeded`, `SessionLimitExceeded`, `InvalidRequest`,
  `InvalidReload`, `AuditWriteFailed`, `FederationUnavailable`,
  `NamespacedRerouteBlocked`) with `reason_code()` snake_case codes frozen for
  the life of each variant; `GatewayResult<T>` alias [mod.rs:GatewayError],
  [mod.rs:reason_code], [mod.rs:GatewayResult]

### Config and registry

- `pub const GATEWAY_CONFIG_REL: &str = ".swe-seed/gateway/config.json"`
  [config.rs:GATEWAY_CONFIG_REL]
- `MCPServer` (the registry artifact: namespace, transport, command_or_url,
  args, env, profile/policy tags, `source_hash`, `scan_status`, `provenance_ref`,
  declared `catalog`, `pinned_hashes`) and `DeclaredCatalogEntry`
  [config.rs:MCPServer], [config.rs:DeclaredCatalogEntry]
- `GatewayConfig::load(root)` (missing file = standalone default),
  `GatewayConfig::validate()` (namespaces + listener exposure),
  `GatewayConfig::validate_namespaces()` [config.rs:load],
  [config.rs:validate], [config.rs:validate_namespaces]
- `ListenerConfig` with `is_loopback()` and `exposure_ok()` (non-loopback bind
  requires TLS or tunnel) [config.rs:ListenerConfig], [config.rs:exposure_ok]
- `project_backend(server) -> GatewayBackend` (fresh backends start
  Healthy/Closed) [config.rs:project_backend]

### JSON-RPC validation

- `pub const REQUEST_SIZE_CAP: usize = 1024 * 1024`, `JSON_DEPTH_CAP: usize =
64` [jsonrpc.rs:REQUEST_SIZE_CAP], [jsonrpc.rs:JSON_DEPTH_CAP]
- `validate_request(body) -> Result<JsonRpcRequest, GatewayError>`,
  `parse_jsonrpc(value)`, `is_allowed_method(method)`, `json_depth(value)`
  [jsonrpc.rs:validate_request], [jsonrpc.rs:parse_jsonrpc],
  [jsonrpc.rs:is_allowed_method], [jsonrpc.rs:json_depth]

### Catalog projection and discovery

- `validate_namespace(ns)` (`a-z0-9_-`, 1..=63 chars), `validate_local_name(name)`
  (MUST NOT contain `.`) with `NamespaceError { Invalid, Duplicate }`
  [catalog.rs:validate_namespace], [catalog.rs:validate_local_name],
  [catalog.rs:NamespaceError]
- `canonical_json(value)`: sorted keys, no whitespace; byte-stable hashing is
  load-bearing for pinned-hash integrity [catalog.rs:canonical_json]
- `definition_hash(wire_definition) -> "sha256:<hex>"`
  [catalog.rs:definition_hash]
- `project_catalog(servers) -> (Vec<GatewayCatalogEntry>, Vec<DroppedEntry>)`
  (invalid local names are dropped and reported, not fatal)
  [catalog.rs:project_catalog]
- `merge_live(declared, live)`: declared entries win conflicts; conflicting and
  duplicate live entries are dropped and reported [catalog.rs:merge_live]
- `CompactCatalogItem`, `pub const DEFAULT_DISCOVERY_CAP: usize = 64`,
  `list_catalog(catalog, cap)`, `search_catalog(catalog, query, cap)`
  [catalog.rs:CompactCatalogItem], [catalog.rs:DEFAULT_DISCOVERY_CAP],
  [catalog.rs:list_catalog], [catalog.rs:search_catalog]
- `discover_live(router, servers) -> (Vec<GatewayCatalogEntry>,
Vec<DiscoveryNote>)`: fan-out `tools/list`/`resources/list`/`prompts/list`
  per backend with a 5-second `DISCOVERY_TIMEOUT` [discover.rs:discover_live],
  [discover.rs:DiscoveryNote]

### Policy, redaction, audit, governance

- `evaluate_invoke(policy, entry, scan_status, provenance_present) ->
Result<InvokeDecision, GatewayError>` with `InvokeDecision { Allow,
ApprovalRequired }`; `is_risky(policy_tags)` over the fixed `RISKY_TAGS` set
  (`dangerous`, `destructive`, `write`, `exec`, `shell`, `network`,
  `untrusted`); `scan_is_clear(scan_status)`
  [policy.rs:evaluate_invoke], [policy.rs:InvokeDecision], [policy.rs:is_risky],
  [policy.rs:scan_is_clear]
- `builtin_redaction()` and `load_redaction(root)` (reads
  `.agent-hooks/config.yaml`; falls back to the conservative built-in only when
  the file is absent; malformed policy fails closed)
  [redaction.rs:builtin_redaction], [redaction.rs:load_redaction]
- `pub const AUDIT_DIR_REL: &str = ".swe-seed/gateway/audit"`, `AUDIT_FILE:
&str = "audit.jsonl"`; `AuditRecord` (fixed schema, `detail` redacted before
  write) with `decision_for_error()`; `AuditWriter::new(root, redaction)`,
  `file_path()`, `write(record)` [audit.rs:AUDIT_DIR_REL], [audit.rs:AUDIT_FILE],
  [audit.rs:AuditRecord], [audit.rs:decision_for_error], [audit.rs:AuditWriter],
  [audit.rs:write]
- `pub const GOVERNANCE_JOURNAL_REL: &str =
".swe-seed/gateway/governance.journal"`; `generate_session_id()` (16 random
  bytes hex, >=128 bits); `Governance::load(root, limits)` (replays the journal),
  `open_session(client_id)`, `check_and_increment(session_id, client_id, tool)`,
  `snapshot()` [governance.rs:GOVERNANCE_JOURNAL_REL],
  [governance.rs:generate_session_id], [governance.rs:Governance],
  [governance.rs:check_and_increment]

### Routing, health, reload

- `pub const ALLOWED_METHODS: &[&str]` = `tools/list`, `tools/call`,
  `resources/list`, `resources/read`, `prompts/list`, `prompts/get`
  [routing.rs:ALLOWED_METHODS]
- `pub trait BackendTransport { send_request(request, timeout) }` with
  `StdioTransport` (spawns the backend process, one request line, always reaps
  the child), `HttpTransport` (loopback plain HTTP/1.1 POST; HTTPS rejected),
  and `MockTransport` (test fake with `call_count()`)
  [routing.rs:BackendTransport], [routing.rs:StdioTransport],
  [routing.rs:HttpTransport], [routing.rs:MockTransport]
- `Router::register(backend_id, transport)` and `Router::invoke(backend_id,
request_id, method, params, timeout)` [routing.rs:Router],
  [routing.rs:invoke]
- `BreakerConfig { failure_threshold = 3, cooldown = 5s }`; `HealthBook` with
  `register`, `record_outcome`, `can_forward`, `snapshot`
  [health.rs:BreakerConfig], [health.rs:HealthBook],
  [health.rs:record_outcome], [health.rs:can_forward]
- `Metrics` (`record`, `snapshot` -> `MetricsSnapshot`), `BackendMetrics`
  [health.rs:Metrics], [health.rs:MetricsSnapshot], [health.rs:BackendMetrics]
- `SnapshotManager::install(initial)` (must validate), `reload(new_cfg)`
  (invalid reload keeps last-known-good), `current()`, `rollback()`
  [health.rs:SnapshotManager]

### Federation projection and evidence links

- `project_route_selected(cfg, ...)`, `project_settlement_recorded(cfg, ...)`,
  `consume_authority(cfg, domain_model_hash, local_allows, incoming)`,
  `reject_on_drift(expected_hash, incoming)` with `Disposition { event_type,
envelope_id, dispatched, suppressed_reason }`
  [federation_proj.rs:project_route_selected],
  [federation_proj.rs:project_settlement_recorded],
  [federation_proj.rs:consume_authority], [federation_proj.rs:reject_on_drift],
  [federation_proj.rs:Disposition]
- `cite_gateway_evidence(proof, gateway_ref)` (dedup append of a gateway
  audit/proof id onto `FabricatorProofRecord.evidence_refs`) and
  `evidence_link(upstream_id, downstream_id)` (relation `"proves"`)
  [fabricator.rs:cite_gateway_evidence], [fabricator.rs:evidence_link]

### OpenAPI import

- `import_openapi(namespace, doc) -> Result<MCPServer, GatewayError>`:
  idempotent import; operations become `<namespace>.<operationId>` tools with
  merged path/query/body input schemas and policy tags `http`,
  `openapi-imported` [import.rs:import_openapi]

### Request orchestration and server

- `RequestContext` (caller-supplied identity + target) and `handle_request(...)`
  composing gates + governance + forward + audit
  [request.rs:RequestContext], [request.rs:handle_request]
- `pub const DEFAULT_WORKERS: usize = 4`, `DEFAULT_QUEUE: usize = 16`;
  `GatewayServer::bind(root, bind, port)`, `local_addr()`, `serve_once()`,
  `serve_loop()`, `serve_n(workers, queue, n)`
  [serve.rs:DEFAULT_WORKERS], [serve.rs:DEFAULT_QUEUE], [serve.rs:GatewayServer],
  [serve.rs:bind], [serve.rs:serve_once], [serve.rs:serve_loop]

## Invariants

- Every request completes with a response OR a typed `GatewayError`; each
  variant's `reason_code` is snake_case and frozen for the life of the variant
  [mod.rs:GatewayError], [mod.rs:reason_code]
- Audit is part of request completion: exactly one redacted JSONL record is
  written before a request is complete; if the audit write fails the request
  fails closed with `AuditWriteFailed`, dominating any underlying result
  [audit.rs:write], [request.rs:handle_request]
- A bad input never consumes a budget counter or touches a backend: malformed,
  oversized (over 1 MiB), or too-deep (over 64) requests fail as
  `InvalidRequest` during validation [jsonrpc.rs:validate_request]
- Deny overrides allow: `evaluate_invoke` reuses the Harness `PermissionPolicy`
  semantics so the gateway is a faithful `PreToolUse` fallback; a forbidden tag
  blocks like a forbidden action [policy.rs:evaluate_invoke]
- Risky capabilities fail closed: a tagged capability with a non-clear scan or
  missing provenance is blocked before approval or hash outcomes are considered
  [policy.rs:evaluate_invoke], [policy.rs:RISKY_TAGS]
- A namespaced call routes ONLY to its owning backend: a missing or down
  backend is `BackendUnavailable`, never a reroute; a backend echoing a
  different JSON-RPC id is `BackendError` [routing.rs:invoke]
- Governance counters are durable before forward: the check-and-increment
  critical section appends + fsyncs the journal delta before returning Ok, and
  on restart the journal is replayed so a crash cannot undercount
  [governance.rs:check_and_increment], [governance.rs:load]
- Session ids are server-generated with >=128 bits of entropy; clients MUST NOT
  supply or override them [governance.rs:generate_session_id],
  [mod.rs:GatewaySession]
- The breaker is single-probe in half-open: exactly one probe may be in flight;
  success closes, failure re-opens [health.rs:can_forward],
  [health.rs:record_outcome]
- Reload validates before applying: an invalid snapshot keeps last-known-good
  and returns `InvalidReload`; non-loopback binds require TLS or tunnel mode
  [health.rs:reload], [config.rs:exposure_ok]
- Catalog construction has ONE owner (`merge_live`); declared entries win
  conflicts and dropped entries are recorded deterministically
  [catalog.rs:merge_live], [catalog.rs:project_catalog]
- Federation standalone invariant: with `federation.enabled=false` the
  projection module makes ZERO external calls, constructs no envelope, and
  touches no sink; local authority mode never inspects a (possibly spoofed)
  incoming envelope [federation_proj.rs:project_route_selected],
  [federation_proj.rs:consume_authority]
- Gateway request routing MUST NOT mutate Fabricator semantic-chain artifacts;
  gateway ids enter the chain only as evidence refs
  [fabricator.rs:cite_gateway_evidence]
- Redaction is never zero: an absent `.agent-hooks/config.yaml` falls back to
  the conservative built-in policy; a present-but-malformed policy fails closed
  [redaction.rs:load_redaction], [redaction.rs:builtin_redaction]
- Audit appends are line-atomic: one buffered write per record under a mutex,
  so concurrent appends never fuse or drop records [audit.rs:write]
- The server is bounded: fixed worker pool with a bounded queue; overflow is
  rejected with HTTP 503 after draining the request, never unbounded work
  [serve.rs:serve_bounded], [serve.rs:DEFAULT_QUEUE]

## Side Effects

- `GatewayConfig::load` reads `.swe-seed/gateway/config.json`
  [config.rs:load]
- `AuditWriter::write` creates `.swe-seed/gateway/audit/` and appends redacted
  JSONL records to `audit.jsonl` [audit.rs:write]
- `Governance::load` replays `.swe-seed/gateway/governance.journal`;
  `check_and_increment` appends + fsyncs one JSONL delta per namespaced call
  [governance.rs:load], [governance.rs:append_delta]
- `load_redaction` reads `.agent-hooks/config.yaml` when present
  [redaction.rs:load_redaction]
- `StdioTransport::send_request` spawns the backend process per call and
  always kills + reaps it; `HttpTransport::send_request` opens a TCP
  connection to the backend [routing.rs:StdioTransport],
  [routing.rs:HttpTransport]
- `GatewayServer::bind` binds a `TcpListener` (loopback default) and runs live
  discovery; discovery notes and dropped live entries are logged to stderr
  [serve.rs:bind], [serve.rs:serve_bounded]
- `project_route_selected` / `project_settlement_recorded` dispatch envelopes
  to the sink file or stdout when the federation master switch AND
  `emit_envelope` are on [federation_proj.rs:project_route_selected]
- `import_openapi` computes the source hash over the canonical OpenAPI
  document (no disk I/O itself; the caller supplies the parsed doc)
  [import.rs:import_openapi]
- `GatewayServer::dispatch` serves `/list` methods from the local compact
  catalog without forwarding, writing a best-effort discovery audit record
  [serve.rs:dispatch], [serve.rs:write_discovery_audit]

## Verification

- Run: `cargo test -p swe-seed-core`
