# federation

## Overview

SEA federation boundary (spec 0011). The ONLY module that touches the SEA
contract. With `enabled=false` (the default) SWE_Seed is fully standalone: no
external call is ever made and every gate resolves locally. Ports the
first-party Python `agentic_capability_loop/adapters.py` 1:1 (event types,
payload keys, hash resolution order). Covers the canonical v1 event envelope,
flag-gated dispatch, exclusive-producer authority, domain-identity and
semantic-world gates, Ed25519 envelope signing, durable idempotency, the
Context Kernel MCP client, and the canonical E4 GovernedWorkRequest / E6
OperationalSettlement / E7 ProofCompleted surfaces.

## Interface

### Envelope and hash resolution

- `pub const NAMESPACE: &str = "agentic_capability_loop"`, `SCHEMA_VERSION:
&str = "v1"`, `SOURCE_AGENT: &str = "swe-seed"`, `CAUSALITY_PREFIX: &str =
"caused_by:"` [envelope.rs:NAMESPACE], [envelope.rs:SCHEMA_VERSION],
  [envelope.rs:SOURCE_AGENT], [envelope.rs:CAUSALITY_PREFIX]
- `pub struct Envelope { schema_version, event_id, source_agent, event_type,
occurred_at, idempotency_key, payload, provenance }`; `namespace` rides
  inside `payload` (v1 schema `additionalProperties:false`)
  [envelope.rs:Envelope]
- `Envelope` accessors: `domain_model_hash()`, `namespace()`,
  `with_trace_chain_root()`, `trace_chain_root()`, `work_request_id()`,
  `causal_parents()` [envelope.rs:domain_model_hash],
  [envelope.rs:with_trace_chain_root], [envelope.rs:causal_parents]
- `pub enum HashSource { SeaRoot, SeaManifestPath, MarkerWalk, Fallback }` and
  `ResolvedHash { hash, source, warned }` [envelope.rs:HashSource],
  [envelope.rs:ResolvedHash]
- `fallback_hash()` = `sha256("agentic_capability_loop")` [envelope.rs:fallback_hash]
- `resolve_domain_model_hash()`: env order `SEA_ROOT` -> `SEA_MANIFEST_PATH` ->
  marker walk -> fallback (+ warn) [envelope.rs:resolve_domain_model_hash]
- `resolve_from_root(root)` and pure `resolve_from(sea_root, sea_manifest, walk)`
  [envelope.rs:resolve_from_root], [envelope.rs:resolve_from]
- `make_event(event_type, payload, hash)`: canonical v1 envelope builder;
  injects `domain_model_hash` and `namespace` as first payload keys and a
  `{origin, chain}` provenance record [envelope.rs:make_event]
- `make_event_verified(event_type, payload, identity)`: hash-gated construction
  that accepts only a `VerifiedDomainIdentity` [envelope.rs:make_event_verified]
- `derive_event(parents, event_type, payload, hash, correlation)`: child
  envelope derivation recording `caused_by:` parents and enforcing correlation
  stability; `DeriveError { NoParents, DuplicateParent, CorrelationMismatch }`
  [envelope.rs:derive_event], [envelope.rs:DeriveError]
- `idempotency_key(event_type, payload, correlation_id)`: content-derived
  sha256 over `"{correlation_id}|{event_type}|{canonical_payload_json}"`
  [envelope.rs:idempotency_key]

### Flags and the authority gate

- `pub enum AuthorityMode { Local, Delegate, Hybrid }`, `ContextMode { Local,
External, Hybrid }`, `SettlementMode { Off, Emit, Consume, Both }` (all
  default local/off, lowercase serde) [flags.rs:AuthorityMode],
  [flags.rs:ContextMode], [flags.rs:SettlementMode]
- `pub enum Risk { Low, Medium, High, Dangerous }` with `at_least_high()` /
  `is_dangerous()` [flags.rs:Risk]
- `pub struct FederationConfig { enabled, emit_envelope, consume_envelope,
authority, context, settlement }` (default = fully standalone);
  `standalone()`, `authority_mode()`, `context_mode()`, `settlement_mode()`,
  `emits()`, `consumes()`, `is_standalone()` [flags.rs:FederationConfig],
  [flags.rs:standalone], [flags.rs:emits], [flags.rs:is_standalone]
- `authority_gate(cfg, risk, local_allows, incoming) -> GateOutcome` with
  `GateOutcome { Allow, Deny, Escalate, NeedsHuman }` and
  `AuthorityVerdict { Allow, Deny, Escalate }` [flags.rs:authority_gate],
  [flags.rs:GateOutcome]

### Emit and dispatch

- `emit_work_requested`, `emit_context_required`, `emit_route_selected`,
  `emit_proof_started`, `emit_proof_completed`, `emit_settlement_recorded`:
  per-event envelope builders mirroring the Python payload keys 1:1
  [emit.rs:emit_work_requested], [emit.rs:emit_context_required],
  [emit.rs:emit_route_selected], [emit.rs:emit_proof_started],
  [emit.rs:emit_proof_completed], [emit.rs:emit_settlement_recorded]
- `pub const PROOF_TYPE_LIVE: &str = "live"`, `PROOF_TYPE_SIMULATION: &str =
"simulation"` [emit.rs:PROOF_TYPE_LIVE], [emit.rs:PROOF_TYPE_SIMULATION]
- `dispatch(cfg, envelope, sink) -> Dispatch { Written(PathBuf), Suppressed }`:
  writes only when `cfg.emits()`; file sink or stdout
  [emit.rs:dispatch], [emit.rs:Dispatch]

### Producer authority (invariant I3)

- `pub mod agent_id` (canonical component ids: `swe_seed`, `godspeed_agent`,
  `context_kernel`, `sea_forge`, `realitytrace`, `memory_ledger`,
  `execution_environment`, `external_environment`) [producers.rs:agent_id]
- `canonical_agent(source_agent) -> Option<&'static str>`: unknown agents are
  `None` (fail closed) [producers.rs:canonical_agent]
- `authoritative_producer(event_type)`: frozen edge topology E0-E10 (e.g.
  `WorkRequested` -> `godspeed_agent`, `ContextPacketCreated` ->
  `context_kernel`, `GovernedWorkRequest` / `ProofCompleted` / `RouteSelected` /
  `ProofStarted` -> `swe_seed`, `OperationalSettlement` / `AuthorityChecked` ->
  `sea_forge`) [producers.rs:authoritative_producer]
- `validate_producer(envelope)` with `ProducerAuthorityError { UnknownEventType,
NotAuthoritative, UnknownAgent }` [producers.rs:validate_producer],
  [producers.rs:ProducerAuthorityError]

### Consume and identity gates

- `check_drift(envelope, expected_hash)`: rejects absent/mismatched
  `domain_model_hash` [consume.rs:check_drift]
- `consume_context_packet_created`, `consume_authority_checked`,
  `consume_settlement_recorded`, `authority_verdict(payload)`
  [consume.rs:consume_context_packet_created],
  [consume.rs:consume_authority_checked],
  [consume.rs:consume_settlement_recorded], [consume.rs:authority_verdict]
- `validate_envelope(envelope, expected_hash)`: the composed T01 boundary gate
  (producer authority, non-placeholder identity, drift, well-formed causal
  parent ids); `ConsumeError` covers `WrongType`, `MissingPayload`,
  `WrongNamespace`, `HashDrift`, `ProducerAuthority`, `Identity`, `Causality`,
  `DestinationOnly`, `World` [consume.rs:validate_envelope],
  [consume.rs:ConsumeError]
- `check_conformance(envelope) -> ConformanceReport`: pure structural
  conformance against the v1 shape; returns data, never effects, never
  evaluates the truth of the claim [consume.rs:check_conformance],
  [consume.rs:ConformanceReport]
- `verify_declared_hash(declared)`, `verify_against_artifact(declared,
artifact)`, `strict_resolve(sea_root, sea_manifest)`,
  `VerifiedDomainIdentity::from_resolved(resolved)` with `IdentityError`
  [identity.rs:verify_declared_hash], [identity.rs:verify_against_artifact],
  [identity.rs:strict_resolve], [identity.rs:VerifiedDomainIdentity],
  [identity.rs:IdentityError]
- `WorldRef::parse(candidate)` (syntax gate for
  `world:<name>@sha256:<64 lowercase hex>`), `WorldRef::require_same`,
  `Envelope::world_ref()`, `Envelope::verified_world_ref()`, `WorldRefError`
  [world.rs:WorldRef], [world.rs:WorldRefError], [world.rs:verified_world_ref]
- `WorkRequestContract { work_request_id, affordance_id, desired_outcome,
settlement_criteria, world_ref }` and `accept_work_requested(envelope,
expected_hash)` (canonical E1 ingress) [work_ingress.rs:WorkRequestContract],
  [work_ingress.rs:accept_work_requested]

### Signing (Phase B)

- `generate_signing_key()`, `signing_key_from_bytes(&[u8; 32])`
  [signing.rs:generate_signing_key], [signing.rs:signing_key_from_bytes]
- `public_key_path(root, key_id)` = `.agent-harness/federation/keys/<id>.pub`;
  `private_key_path(root, key_id)` = `.swe-seed/federation/keys/<id>.key`
  [signing.rs:public_key_path], [signing.rs:private_key_path]
- `write_keypair(root, key_id, secret)`: pub 0o644, private 0o600 via
  `util::secure_write` [signing.rs:write_keypair]
- `load_signing_key(path)`: plaintext base64 or SOPS-encrypted (decrypts via
  `sops -d`); `LoadError { NotFound, Malformed, SopsUnavailable, SopsFailed }`
  [signing.rs:load_signing_key], [signing.rs:LoadError]
- `canonical_signing_string(namespace, event_type, occurred_at, payload)`:
  `{namespace}\n{event_type}\n{occurred_at}\n{compact_sorted_payload_json}`;
  `signature`, `namespace` (payload copy), and `event_id` are excluded from the
  signed payload JSON [signing.rs:canonical_signing_string]
- `canonical_payload_json(payload)`: sorted compact JSON keeping `namespace`
  (the F-10 idempotency canonicalization) [signing.rs:canonical_payload_json]
- `sign_envelope(envelope, secret, key_id) -> SignaturePayload { algorithm,
key_id, value }` and `verify_envelope(envelope, signature_b64, public_key)`
  (Ed25519 `verify_strict`) with `VerifyError { MalformedSignature,
InvalidSignature }` [signing.rs:sign_envelope], [signing.rs:SignaturePayload],
  [signing.rs:verify_envelope], [signing.rs:VerifyError]
- `public_key_b64(pk)`, `public_key_from_b64(b64)`
  [signing.rs:public_key_b64], [signing.rs:public_key_from_b64]

### Idempotency and settlement

- `IdempotencyLedger::open(path)`, `contains(key)`, `len()`, `is_empty()`,
  `admit(key) -> Admission { First, Duplicate }`; append-only JSONL with
  fsync; `AdmitError { Io, MalformedKey }` [idempotency.rs:IdempotencyLedger],
  [idempotency.rs:admit], [idempotency.rs:Admission], [idempotency.rs:AdmitError]
- `OperationalSettlementAdjudicator::open(path)`, `adjudicate(envelope,
originating_work_request_id, expected_chain, local_model_sha256,
originating_world_ref) -> Adjudication { First(facts), DuplicateDelivery }`
  with `AdjudicationError`; `OperationalOutcome { Accepted, Rejected }`;
  `OperationalSettlementFacts` [operational_settlement.rs:OperationalSettlementAdjudicator],
  [operational_settlement.rs:adjudicate], [operational_settlement.rs:Adjudication],
  [operational_settlement.rs:OperationalOutcome],
  [operational_settlement.rs:OperationalSettlementFacts],
  [operational_settlement.rs:AdjudicationError]
- `pub const EVENT_TYPE: &str = "OperationalSettlement"` and `REQUIRED_FIELDS`
  (work_request_id, authority_decision_id, execution_status, observed_effects,
  operational_settlement_status, evidence_refs, domain_model_ref)
  [operational_settlement.rs:EVENT_TYPE],
  [operational_settlement.rs:REQUIRED_FIELDS]

### Governed submission and proof (E4 / E7)

- `ProofContract { criterion, command, proof_type }`: deliberately a distinct
  type from settlement criteria [governed_submission.rs:ProofContract]
- `GovernedSubmission` (typed inputs: contract, actor, intent, proof_contract,
  route_id, artifact_expectations, authority_context, payment_budget,
  context_bundle) [governed_submission.rs:GovernedSubmission]
- `context_packet_ref(context_packet)`: the packet's `context_packet_id` else
  its event id [governed_submission.rs:context_packet_ref]
- `build_governed_work_request(work_requested, context_packet, sub, identity)`:
  canonical E4 envelope derived from the E1 + E3 parents
  [governed_submission.rs:build_governed_work_request]
- `pub const EVENT_TYPE: &str = "ProofCompleted"`, `PROOF_STATUSES: [&str; 2] =
["passed", "failed"]`, `REQUIRED_FIELDS` (7 fields)
  [proof_completed.rs:EVENT_TYPE], [proof_completed.rs:PROOF_STATUSES],
  [proof_completed.rs:REQUIRED_FIELDS]
- `operational_settlement_ref(settlement)`: content-addressed
  `sha256:<hex>` of the settlement envelope's canonical JSON (ENV-I7)
  [proof_completed.rs:operational_settlement_ref]
- `ProofCompletion` (typed E7 inputs incl. verbatim `expected_outcome` and
  `context_bundle_ref`) and `emit_proof_completed_verified(settlement, facts,
additional_parents, completion, identity)`
  [proof_completed.rs:ProofCompletion],
  [proof_completed.rs:emit_proof_completed_verified]
- `SubmissionError` and `EmissionError`: every refusal fires BEFORE any
  canonical envelope exists [governed_submission.rs:SubmissionError],
  [proof_completed.rs:EmissionError]

### Context Kernel client (WP-5, F-04)

- `ContextKernelClient::from_env()` (config-gated: `SWE_SEED_CONTEXT_KERNEL_URL`
  service first, `SWE_SEED_CONTEXT_KERNEL_BIN` embedded spawn second, `None`
  when neither is set), `connect(base)`, `spawn(bin, corpus_root)`,
  `negotiate() -> KernelService { name, version }`, `acquire_context(req)`,
  `call_tool(name, arguments)` [context_client.rs:from_env],
  [context_client.rs:connect], [context_client.rs:spawn],
  [context_client.rs:negotiate], [context_client.rs:acquire_context],
  [context_client.rs:call_tool]
- `ContextRequest` (typed E2 inputs incl. `VerifiedDomainIdentity` and
  `WorldRef`), `ExpectedContext`, `KernelService`
  [context_client.rs:ContextRequest], [context_client.rs:ExpectedContext],
  [context_client.rs:KernelService]
- `adjudicate_context_response(resp, expected) -> Result<ContextPacket,
ContextClientError>`: pure adjudication of the E3 response
  [context_client.rs:adjudicate_context_response]
- `ContextPacket` (envelope + optional CEP bundle) with `citation_count()`,
  `retrieval_completeness()`, `omissions()`, `bundle_content_hash()`,
  `authority_reference()`, `citations()`
  [context_client.rs:ContextPacket], [context_client.rs:citation_count],
  [context_client.rs:retrieval_completeness]
- `pub enum RetrievalCompleteness { Complete, Partial, None, Unknown }`
  [context_client.rs:RetrievalCompleteness]

### Context bundle (CEP-0008)

- `pub const PROFILE_ID: &str = "godspeed.context_bundle"`, `PROFILE_VERSION:
&str = "1.0.0"` [context_bundle.rs:PROFILE_ID], [context_bundle.rs:PROFILE_VERSION]
- `bundle_content_hash(envelope)`: sha256 over canonical JSON of the envelope
  minus the `integrity` block, floats normalized to 6 decimal places
  [context_bundle.rs:bundle_content_hash]
- `verify_context_bundle(bundle, work_request_id, world_ref,
context_requirement_id) -> Result<BundleFacts, String>`: six-step battery
  (kind, profile, world, correlation, truthful completeness, integrity hash)
  [context_bundle.rs:verify_context_bundle]
- `BundleFacts { envelope_id, content_hash, world_ref, work_request_ref,
retrieval_completeness }` with `to_ref()`; `bundle_from_response(resp)`,
  `ref_matches(facts, stored)`, `completeness_of(facts)`
  [context_bundle.rs:BundleFacts], [context_bundle.rs:bundle_from_response],
  [context_bundle.rs:ref_matches], [context_bundle.rs:completeness_of]

## Invariants

- Standalone by default: with `enabled=false` every plane resolves
  `local`/`off`, `emits()`/`consumes()` are false, and `dispatch` touches
  nothing (zero external calls, no sink write) [flags.rs:authority_mode],
  [emit.rs:dispatch]
- Master switch wins: with `enabled=false`, sub-mode configuration is ignored
  and `authority_mode()`/`context_mode()`/`settlement_mode()` return
  Local/Local/Off [flags.rs:authority_mode]
- A local `deny` is never overridden by an external `allow` unless
  `allow_external_override` is set; delegate timeouts are fail-closed for
  `Dangerous` actions [flags.rs:authority_gate],
  [flags.rs:AuthorityConfig]
- `namespace` is never a top-level envelope field (v1 schema
  `additionalProperties:false`); it rides in `payload` and is signed as the
  canonical string's header line [envelope.rs:Envelope],
  [signing.rs:canonical_signing_string]
- Hash resolution order is `SEA_ROOT` -> `SEA_MANIFEST_PATH` -> marker walk ->
  fallback (+ warn), matching the Python adapter [envelope.rs:resolve_from]
- Fallback/placeholder pseudo-hashes never become identity: `VerifiedDomainIdentity`
  refuses `HashSource::Fallback` and the all-zero digest, and construction of a
  `VerifiedDomainIdentity` is the only way to stamp a hash through the T01 API
  [identity.rs:from_resolved], [identity.rs:reject_placeholders],
  [envelope.rs:make_event_verified]
- Exclusive producer authority: each canonical event type has exactly one
  authoritative producer per the frozen E0-E10 topology; unknown event types
  and forged source agents are rejected, never coerced
  [producers.rs:validate_producer], [producers.rs:authoritative_producer]
- Derived envelopes must identify their parents: `derive_event` requires at
  least one parent, rejects duplicates, records `caused_by:` entries, and
  refuses a correlation that disagrees with agreeing parents (ENV-I4/I3)
  [envelope.rs:derive_event]
- Every durable envelope write is idempotent (ENV-I6): `admit` appends + fsyncs
  on `First` only, duplicates write nothing, and a malformed ledger line is a
  hard error (corrupt ledger fails closed) [idempotency.rs:admit],
  [idempotency.rs:open]
- The first settlement stands: a new OperationalSettlement claiming an
  already-adjudicated chain (work_request_id + authority_decision_id) with
  different content is a `ConflictingResettlement` refusal; duplicate delivery
  is consequence-free [operational_settlement.rs:adjudicate]
- Operational settlement facts carry nothing capability- or
  developmental-shaped and no proof verdict; there is no path from the E6
  surface to `SettlementRecorded`/`CapabilityUpdated` promotion (I6/I7)
  [operational_settlement.rs:OperationalSettlementFacts]
- Proof completion emits only against a REAL adjudicated settlement: the facts
  must name the settlement's event id, agree on work_request_id and world_ref,
  and the settlement must pass the composed boundary gate under our identity
  [proof_completed.rs:emit_proof_completed_verified]
- `expected_outcome` is carried verbatim (immutable projection); optional
  fields are typed or refused, never coerced
  [proof_completed.rs:emit_proof_completed_verified]
- Destination-only work inputs are rejected: `accept_work_requested` requires
  non-empty `affordance_id`, `desired_outcome`, and `settlement_criteria`
  (frozen E1 falsifier) [work_ingress.rs:accept_work_requested]
- `world_ref` is a syntax-and-equality gate only: SWE_SEED never verifies the
  digest, never derives a `world_ref` from the legacy `domain_model_hash`, and
  refuses mutable aliases/labels [world.rs:WorldRef], [world.rs:parse]
- A partial context packet is never `complete`; `require_complete` refuses
  non-complete packets, and zero citations under `required` is the explicit
  `GovernedNoContext` outcome, never silent success
  [context_client.rs:adjudicate_context_response],
  [context_client.rs:GovernedNoContext]
- Truthful bundle accounting: a `complete` bundle must be
  `complete_for_context` with references and no omissions; `partial` and `none`
  bundles must keep their omissions/questions and never read complete
  [context_bundle.rs:verify_context_bundle]
- The signing string excludes `event_id`, the `signature` field, and the
  payload's `namespace` copy; Rust and Python implementations MUST produce
  byte-identical canonical strings [signing.rs:canonical_signing_string]
- CK is reached over MCP `tools/call` only (stdio spawn or HTTP service),
  never NATS; the client refuses services whose `get_status` does not report
  `context-kernel` [context_client.rs:call_tool], [context_client.rs:negotiate]

## Side Effects

- `dispatch` writes the envelope JSON to the sink file (creating parent
  directories) or prints to stdout, only when `cfg.emits()`
  [emit.rs:dispatch]
- `resolve_domain_model_hash` reads `SEA_ROOT`/`SEA_MANIFEST_PATH` env vars and
  manifest files from disk; the marker walk reads the current directory tree
  [envelope.rs:resolve_domain_model_hash], [envelope.rs:marker_walk_hash]
- `write_keypair` writes the public key under
  `.agent-harness/federation/keys/` (0o644) and the private key under
  `.swe-seed/federation/keys/` (0o600) [signing.rs:write_keypair]
- `load_signing_key` spawns `sops -d <path>` when the key file is not
  plaintext base64 [signing.rs:decode_key_bytes]
- `IdempotencyLedger` and `OperationalSettlementAdjudicator` read and append to
  their JSONL ledger file with fsync [idempotency.rs:admit],
  [idempotency.rs:open]
- `ContextKernelClient::spawn` launches the CK binary as a child process with a
  temp database (killed on drop); `connect`/`call_tool` opens a TCP connection
  to the CK service (HTTP/1.1 POST `{base}/mcp`) [context_client.rs:spawn],
  [context_client.rs:http_post_json_rpc], [context_client.rs:Drop]
- `verify_against_artifact` reads the referenced model artifact bytes
  [identity.rs:verify_against_artifact]
- `read_manifest_hash` reads SEA manifest JSON files
  [envelope.rs:read_manifest_hash]

## Verification

- Run: `cargo test -p swe-seed-core`
