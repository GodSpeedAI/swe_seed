Replaying 35 AI prompts from your last review on registry-provenance-core.

## Completion ledger

- ~~Reject unsafe federation `key_id` path values before constructing key paths.~~ Fixed and covered by `federation_signing`.
- ~~Treat route-gate audit append failures as hook blocks instead of process errors.~~ Fixed.
- ~~Make `trace_chain_root` insertion fail for non-object envelope payloads.~~ Fixed and covered by `federation_signing`.
- ~~Make the genesis `RouteSelected` ledger append mandatory.~~ Fixed.
- ~~Verify trace ledgers before route-gate genesis checks.~~ Fixed and covered by `routing_gate`.
- ~~Fix root `SWE_SEED_SPEC_v0.2.0.md` broken detailed-spec pointer.~~ Fixed.
- ~~Move shared `SourceRef` into the contracts layer.~~ Fixed.
- ~~Stop federation signing from emitting placeholder passing proof data.~~ Fixed.
- ~~Move clean-room boundary authority out of `.agents/` references.~~ Fixed.
- ~~Remove duplicate `docs/specs/` wording in `README.md`.~~ Fixed.
- ~~Fix root `HARNESS_SPEC.md` broken detailed-spec pointer.~~ Fixed.
- ~~Serialize trace ledger append head lookup and insert in one write transaction.~~ Fixed.
- ~~Create private signing keys with owner-only permissions.~~ Fixed and covered by `federation_signing`.
- ~~Normalize stale context warnings in CLI golden tests.~~ Fixed.
- ~~Skip symlinked directories before validator recursion.~~ Fixed.
- ~~Remove/defer missing module exports in `lib.rs`.~~ Verified no longer valid: the module files are present in this cohort.
- ~~Replace placeholder SOPS age recipient wording with the real operator recipient.~~ Fixed using the SEA-documented team recipient already present.
- ~~Make canonical signing JSON recursively sorted and signature-free.~~ Fixed and covered by `federation_signing`.
- ~~Rewrite invalid AgentPet `just` cleanup, dependency, and uninstall recipes.~~ Fixed.
- ~~Use unified and platform-specific AgentPet build commands accurately.~~ Fixed.
- ~~Align AgentPet IPC `nix::unistd::Uid` example with documented dependencies.~~ Fixed.
- ~~Move release-gating provenance evidence to committed authoritative paths.~~ Fixed.
- ~~Align `no_change_allowed` semantics across learning specs.~~ Fixed.
- ~~Add release parity coverage to default CI.~~ Fixed.
- ~~Use one authoritative capability-registry vocabulary.~~ Fixed.
- ~~Update SWE_Seed CLI overview to the Rust workspace and current command contract.~~ Fixed.
- ~~Update context-plan golden read order to a numbered authoritative spec.~~ Fixed.
- ~~Seal `event_type` into trace ledger hashes.~~ Fixed and covered by `trace_ledger`/`routing_gate`.
- ~~Define scan severity-to-score threshold mapping.~~ Fixed.
- ~~Resolve competing registry shape in reconciliation specs.~~ Fixed.
- ~~Remove/defer missing `contracts::harness` exports.~~ Verified no longer valid: `src/contracts/harness.rs` is present in this cohort.
- ~~Require crash-safe MCPGate counter persistence before requests advance.~~ Fixed.
- ~~Include `docs/specs/0020-mcpgate.md` in the authoritative spec range.~~ Fixed.
- ~~Prevent federated allow from overriding local forbidden actions.~~ Fixed.
- ~~Clarify `SeedPackageManifest` versus `registry.toml`.~~ Fixed.

Proof: `just ci` passed after these updates.

────────────────────────────────────────────────────────────────────────
  major [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/signing.rs:36crates/swe-seed-core/src/federation/signing.rs:36-43]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/federation/signing.rs around lines 36 - 43,
  Reject unsafe key_id values in public_key_path and private_key_path before
  constructing the PathBuf. Validate that key_id is basename-safe (no path
  separators, no absolute paths, no traversal like .., and only an allowed
  character set) so federation keygen cannot escape the keys directory or
  overwrite arbitrary files. Apply the check directly in the signing.rs
  helpers that build the .agent-harness/federation/keys and
  .swe-seed/federation/keys paths.


────────────────────────────────────────────────────────────────────────
  minor [Stability & Availability]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed/src/hooks_cli.rs:134crates/swe-seed/src/hooks_cli.rs:134-162]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed/src/hooks_cli.rs around lines 134 - 162, The RouteGate
  branch in hooks_cli currently lets append_event failures escape as a
  process error, which breaks the 0/1 hook contract. Update the
  HooksAction::RouteGate handling so audit-log failures are treated as a
  block: catch the append_event error, write a clear stderr message, and
  return ExitCode::from(1) instead of propagating the error. Keep the
  existing allow/block logic intact around route_gate,
  RouteGate::Allow/Block, and the event_id print path.


────────────────────────────────────────────────────────────────────────
  minor [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/envelope.rs:201crates/swe-seed-core/src/federation/envelope.rs:201-204]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/federation/envelope.rs around lines 201 -
  204, Make trace chain root assignment fail when the payload is not a JSON
  object instead of silently no-oping. Update
  Envelope::with_trace_chain_root in envelope.rs to return a fallible
  result or otherwise surface an error when payload is not an object, and
  ensure callers cannot proceed with signing unless trace_chain_root was
  actually inserted. If you prefer enforcing object payloads earlier, add
  that validation at Envelope construction or builder time so
  with_trace_chain_root only runs on object payloads.


────────────────────────────────────────────────────────────────────────
  major [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/trace/lifecycle.rs:13crates/swe-seed-core/src/trace/lifecycle.rs:13-33]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/trace/lifecycle.rs around lines 13 - 33, The
  genesis RouteSelected append is currently treated as best-effort in
  ledger_append, which lets start() return a trace_id even when the required
  proof was never persisted. Update the flow around start() and
  ledger_append so the first RouteSelected write is mandatory: propagate the
  append failure instead of only warning, while keeping later ledger events
  best-effort. Use the trace_ledger::append/open path and the RouteSelected
  genesis write site to separate required genesis persistence from optional
  follow-up logging.


────────────────────────────────────────────────────────────────────────
  major [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/routing_gate.rs:25crates/swe-seed-core/src/routing_gate.rs:25-40]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/routing_gate.rs around lines 25 - 40,
  route_gate() is only checking for a RouteSelected genesis row, so invalid
  ledgers can still pass; update routing_gate::route_gate to fail closed by
  calling trace_ledger::verify_chain on the opened connection before
  has_genesis, and if verification returns any error, immediately return
  RouteGate::Block with an appropriate message instead of allowing routing.
  Keep the existing allow/block behavior in route_gate, but ensure the
  verification step is the gatekeeper so only intact chains can reach the
  genesis check.


────────────────────────────────────────────────────────────────────────
  minor [Maintainability & Code Quality]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/SWE_SEED_SPEC_v0.2.0.md:67SWE_SEED_SPEC_v0.2.0.md:67-69]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @SWE_SEED_SPEC_v0.2.0.md around lines 67 - 69, The Semantic
  Specification Chain reference in the root SWE_SEED_SPEC_v0.2.0.md points
  to a non-existent docs/specs/SWE_SEED_SPEC_v0.2.0.md path. Update that
  pointer to the actual detailed-spec location used by the numbered docs
  under docs/specs/, keeping the authoritative root spec consistent with the
  new spec tree and ensuring the referenced target exists.


────────────────────────────────────────────────────────────────────────
  major [Maintainability & Code Quality]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/contracts/harness.rs:11crates/swe-seed-core/src/contracts/harness.rs:11]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/contracts/harness.rs at line 11, Move the
  shared SourceRef type out of crate::eval and into the contracts layer so
  contracts::harness stays independent of the eval subsystem. Update the
  SourceRef import in harness and any related references in the harness
  contract code, then have eval, route, and skill reuse or re-export the
  contracts version instead of depending on crate::eval::SourceRef. Preserve
  the existing SourceRef symbol name so the contract APIs remain stable
  while removing the layering dependency.


────────────────────────────────────────────────────────────────────────
  major [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed/src/federation_cli.rs:150crates/swe-seed/src/federation_cli.rs:150-168]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed/src/federation_cli.rs around lines 150 - 168, The
  federation signing path in federation_cli::sign currently emits a signed
  ProofCompleted envelope using placeholder proof data ("pass" and empty
  evidence) after only checking trace_root existence. Before calling
  swe_seed_core::federation::emit_proof_completed, load and validate the
  actual proof result for the trace, and only populate the envelope with
  real proof outcome/evidence when that validation succeeds. Use the
  existing sign flow around chain_root and resolve_from_root to locate the
  change, and ensure the signing branch cannot produce a passing proof
  unless proof evidence has been verified.


────────────────────────────────────────────────────────────────────────
  minor [Maintainability & Code Quality]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0009-license-and-provenance-boundaries.md:46docs/specs/0009-license-and-provenance-boundaries.md:46-47]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0009-license-and-provenance-boundaries.md around lines 46 -
  47, The clean-room boundary rule currently references a non-authoritative
  .agents/ location, so move or mirror that boundary text into
  docs/specs/ and update the spec in
  0009-license-and-provenance-boundaries to point to the stable source of
  truth. Keep the rule content aligned with the existing clean-room guidance
  from clean-room-boundaries while ensuring the authoritative version
  lives under docs/specs/, not .agents/.


────────────────────────────────────────────────────────────────────────
  minor [Maintainability & Code Quality]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/README.md:41README.md:41]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @README.md at line 41, The repo-layout sentence in README.md repeats
  the same docs/specs reference twice, so update that description to mention
  the correct distinct source-of-truth location only once. Fix the text
  around the root-layer contracts mention by editing the README sentence so
  it clearly distinguishes the root contracts from the detailed design docs,
  using the duplicated docs/specs reference as the location cue.


────────────────────────────────────────────────────────────────────────
  minor [Maintainability & Code Quality]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/HARNESS_SPEC.md:50HARNESS_SPEC.md:50-51]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @HARNESS_SPEC.md around lines 50 - 51, The root HARNESS_SPEC.md now
  points to a nonexistent docs/specs/HARNESS_SPEC.md reference, so update
  the migrated wording to point readers to the actual root HARNESS_SPEC.md
  and keep the existing 0013-eval-and-proof.md reference intact. Make the
  change in the contract text that mentions Evaluation and adaptation so it
  no longer refers to the broken target and instead directs users to the
  correct document location.


────────────────────────────────────────────────────────────────────────
  major [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/trace_ledger.rs:84crates/swe-seed-core/src/trace_ledger.rs:84-111]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/trace_ledger.rs around lines 84 - 111, The
  append logic in append() reads the current chain head and then inserts the
  next entry without any transaction, which can let concurrent writers pick
  the same seq and lose entries. Wrap the head lookup and insert in a single
  write transaction on the Connection so the read-modify-write sequence is
  serialized, and keep the existing hash/prev_hash computation inside that
  transaction; use the append() path in trace_ledger.rs as the place to
  enforce this.


────────────────────────────────────────────────────────────────────────
  major [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/signing.rs:48crates/swe-seed-core/src/federation/signing.rs:48-55]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/federation/signing.rs around lines 48 - 55,
  The private key is currently written with std::fs::write in write_keypair,
  which can leave it with the process default permissions. Update
  write_keypair so the secret file is created with owner-only permissions
  from the start (0600-style), while keeping the public key write unchanged.
  Use the existing public_key_path/private_key_path flow and apply the fix
  specifically to the private key creation path.


────────────────────────────────────────────────────────────────────────
  major [Stability & Availability]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed/tests/cli_golden.rs:41crates/swe-seed/tests/cli_golden.rs:41-47]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed/tests/cli_golden.rs around lines 41 - 47, The golden
  normalization in normalize() is still letting age-based
  stale_context_warnings through, which makes context-plan output unstable.
  Update the CLI golden test handling in normalize() so
  stale_context_warnings are either stripped or normalized to a stable
  placeholder alongside the existing path, timestamp, and run_id scrubbing.
  Use the normalize() helper in crates/swe-seed/tests/cli_golden.rs and the
  context-plan golden assertions to keep the output deterministic when
  referenced files change or staleness crosses the threshold.


────────────────────────────────────────────────────────────────────────
  minor [Stability & Availability]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/harness_validate.rs:331crates/swe-seed-core/src/harness_validate.rs:331-353]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/harness_validate.rs around lines 331 - 353,
  The directory walkers in collect_scan_files and collect_all are recursing
  into symlinked directories because is_dir() is checked before symlink
  handling, which can create cycles and stack overflows. Update both
  functions to detect symlinks before descending, and skip any path that is
  a symlink even when it points to a directory; keep the existing
  file-extension filtering unchanged. Use the collect_scan_files and
  collect_all traversal logic as the places to apply the guard consistently.


────────────────────────────────────────────────────────────────────────
  critical [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/lib.rs:16crates/swe-seed-core/src/lib.rs:16-26]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/lib.rs around lines 16 - 26, Remove or defer
  the new pub mod exports from lib.rs until their corresponding source files
  are present, since harness_validate, routing_gate, and trace_ledger are
  not available in this cohort. Update the module wiring in
  crate::swe_seed_core::lib only when the cohort lands the actual module
  files, or add temporary stub files for those module names so the crate can
  compile independently.


────────────────────────────────────────────────────────────────────────
  major [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/.sops.yaml:11.sops.yaml:11-12]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @.sops.yaml around lines 11 - 12, The age recipient in the SOPS rule is
  still a placeholder TODO value, so replace it with the real SWE_Seed
  operator age public key before merging. Update the recipient entry in the
  SOPS configuration that contains the commented placeholder and the
  age1mq5... value so federation signing keys are encrypted only to the
  intended operator key. Ensure no sample or unrelated recipient remains in
  the final rule.


────────────────────────────────────────────────────────────────────────
  major [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/signing.rs:118crates/swe-seed-core/src/federation/signing.rs:118-127]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/federation/signing.rs around lines 118 - 127,
  The canonical_signing_string function currently relies on serde_json’s
  internal Map ordering, which can change under feature unification and
  break signature stability. Update canonical_signing_string to explicitly
  normalize the payload into a recursively sorted JSON object before calling
  serde_json::to_string, and keep the signature field removal in that
  normalization step. Add a concise comment or note in the signing module
  documenting that sorted-key JSON is the intended contract, so the behavior
  is explicit rather than implementation-dependent.


────────────────────────────────────────────────────────────────────────
  major [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/agentpet.md:215docs/specs/agentpet.md:215-253]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/agentpet.md around lines 215 - 253, The cleanup,
  install-deps, and uninstall sections use invalid just syntax with if os()
  branching, so rewrite them into valid justfile recipes before publishing.
  Update the recipes in the agentpet spec to use proper just platform
  handling via separate recipes or conditional execution supported by just,
  and make sure the named targets clean, install-deps, and uninstall remain
  usable across windows, macos, and linux without verbatim shell-style if
  blocks.


────────────────────────────────────────────────────────────────────────
  major [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/agentpet.md:155docs/specs/agentpet.md:155-161]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/agentpet.md around lines 155 - 161, The documentation
  change is replacing build-script references with the macOS-only `just
  build-macos`, which would misdirect non-macOS readers. Update the affected
  AgentPet spec text to use just build as the unified entrypoint where
  appropriate, and keep platform-specific build instructions accurate by
  referencing the matching commands for Linux and Windows instead of
  substituting just build-macos globally; check the quick start and any
  build-app references in the AgentPet markdown for consistency.


────────────────────────────────────────────────────────────────────────
  major [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0001-reference-analysis-protocol.md:30docs/specs/0001-reference-analysis-protocol.md:30-31]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0001-reference-analysis-protocol.md around lines 30 - 31,
  The Traceability requirement currently points release-gating evidence at
  .agents/provenance/*, which conflicts with AGENTS.md treating .agents/ as
  gitignored scratch. Update the reference-analysis protocol so the
  provenance matrix lives in a committed, authoritative location (or clearly
  mark it as non-gating), and make the related Traceability / provenance
  requirements consistent with that decision across the spec.


────────────────────────────────────────────────────────────────────────
  major [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0016-learning-and-adaptation-loop.md:23docs/specs/0016-learning-and-adaptation-loop.md:23-25]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0016-learning-and-adaptation-loop.md around lines 23 - 25,
  The no_change_allowed meaning is inconsistent between
  ReflectionTemplate and LearningRecord; choose one polarity and make
  both references match so the reflection gate behavior is unambiguous.
  Update the spec wording in the affected sections to align the explicit “no
  change” requirement with the learning-skip behavior, and use the same
  no_change_allowed semantics wherever the flag is described.


────────────────────────────────────────────────────────────────────────
  major [Stability & Availability]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/scripts/ci.sh:22scripts/ci.sh:22-27]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @scripts/ci.sh around lines 22 - 27, The default CI path in run_test()
  currently stops at cargo test -q, so it misses the release-only parity
  checks covered by just parity. Update the CI script so the run_test() flow
  also invokes the parity gate (or otherwise folds in the same release
  coverage used by just parity) after the existing harness and test steps,
  keeping the default proof path aligned with just ci and preventing
  release-only CLI regressions from slipping through.


────────────────────────────────────────────────────────────────────────
  major [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0003-capability-registry.md:3docs/specs/0003-capability-registry.md:3-9]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0003-capability-registry.md around lines 3 - 9, The spec
  currently mixes two active type systems: the reconciliation note declares
  SkillIR, PermissionPolicy, ArtifactMetadata, LayerCapability, and
  SeedPackageManifest as authoritative, but the canonical-concepts sections
  still describe SkillPack, CapabilityProfile, and ProvenanceRecord as if
  they were primary registry types. Update the body of the document to use
  the reconciled names consistently, and if the legacy terms must remain,
  label them explicitly as historical aliases only. Apply this cleanup
  across the canonical-concepts sections and any referenced type rules so
  the spec has one authoritative vocabulary.


────────────────────────────────────────────────────────────────────────
  major [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/agentpet.md:99docs/specs/agentpet.md:99-107]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/agentpet.md around lines 99 - 107, The IPC example
  currently references nix::unistd::Uid::current() without declaring the nix
  dependency or its needed feature set in the desktop/Cargo.toml guidance.
  Update the spec’s dependency list in the desktop/Cargo.toml section to add
  nix with the required Unix feature(s), or remove that fallback from the
  example if it is not meant to depend on nix. Make sure the IPC example and
  the dependency instructions stay aligned so the sample compiles as
  written.


────────────────────────────────────────────────────────────────────────
  major [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0002-swe-seed-centralization-layer.md:89docs/specs/0002-swe-seed-centralization-layer.md:89-93]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0002-swe-seed-centralization-layer.md around lines 89 - 93,
  The CLI overview is still describing the old monolithic swe_seed::*
  layout and pre-rewrite command shape, which conflicts with the current
  workspace split. Update the authoritative wording in the spec to match the
  Rust workspace model with swe-seed-core and the separate swe-seed CLI
  crate, and align the listed verbs with the current command contract; use
  the existing “CLI behavior” section in this spec as the place to revise so
  readers are pointed at the right modules and commands.


────────────────────────────────────────────────────────────────────────
  major [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/tests/fixtures/golden/context-plan.json:7tests/fixtures/golden/context-plan.json:7-14]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @tests/fixtures/golden/context-plan.json around lines 7 - 14, The
  golden context plan is still pinning a non-authoritative spec path, so
  update the read_order in context-plan.json to use the numbered
  authoritative spec under docs/specs/ instead of
  docs/specs/verification-system.md. Make the change where the
  read_order array is defined so the context policy reflects the new
  design source, and keep the path aligned with the numbered subsystem spec
  naming used elsewhere in the repo.


────────────────────────────────────────────────────────────────────────
  critical [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/trace_ledger.rs:34crates/swe-seed-core/src/trace_ledger.rs:34-38]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/trace_ledger.rs around lines 34 - 38, The
  hash in hash_of() does not cover the gate-relevant event_type field, so
  route_gate() can be fooled by changing the genesis row to RouteSelected
  without breaking verify_chain(). Update the hashing inputs and any callers
  that build the hashed payload so the genesis record’s event_type is
  included in the sealed bytes, and make sure trace_ledger::verify_chain()
  and route_gate() still agree on the exact data being authenticated.


────────────────────────────────────────────────────────────────────────
  major [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0007-skill-ingestion-and-scan-gate.md:53docs/specs/0007-skill-ingestion-and-scan-gate.md:53-57]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0007-skill-ingestion-and-scan-gate.md around lines 53 - 57,
  Clarify the scan gate severity contract by defining how low, medium,
  high, and critical map to the existing score thresholds used in the
  risk/activation rules. Update the spec sections around `Risk scores →
  activation` and the severity-based policy in the later gate logic so
  adapter authors have a single, unambiguous mapping for scan findings and
  make consistent block/exception decisions.


────────────────────────────────────────────────────────────────────────
  minor [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0012-existing-harness-reconciliation.md:42docs/specs/0012-existing-harness-reconciliation.md:42-43]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0012-existing-harness-reconciliation.md around lines 42 -
  43, The spec currently presents two competing registry shapes, with
  SeedPackageManifest called the top-level artifact but another open
  question later reintroducing a second registry contract. Resolve this in
  the authoritative spec by choosing one shape: either fold the 0003 fields
  into SeedPackageManifest and make that the single registry definition, or
  remove the later question entirely. Update the relevant sections in the
  spec so the registry contract is consistent throughout.


────────────────────────────────────────────────────────────────────────
  critical [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/contracts/mod.rs:8crates/swe-seed-core/src/contracts/mod.rs:8-15]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/contracts/mod.rs around lines 8 - 15, The
  contracts::harness module is being exported before its implementation
  exists in this cohort, which makes this layer uncompilable. Remove or
  defer the pub mod harness and pub use harness::{...} exports from
  contracts::mod until the harness cohort adds src/contracts/harness.rs, or
  include the harness module in the same cohort if that is intended. Use the
  symbols contracts::mod, pub mod harness, and the pub use harness block to
  locate the change.


────────────────────────────────────────────────────────────────────────
  major [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0020-mcpgate.md:234docs/specs/0020-mcpgate.md:234-236]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0020-mcpgate.md around lines 234 - 236, The snapshot
  persistence flow is not crash-safe, so a restart can lose in-memory
  counter progress and undercount active sessions. Update the
  gateway/session-budget handling spec around the counter critical section
  to require a durable WAL/replay or an equivalent synchronous handoff
  before requests can advance, while keeping snapshot writes off the request
  path. Reference the session, client, and tool counter update flow and the
  async snapshot persistence behavior so the recovery requirement is
  explicit.


────────────────────────────────────────────────────────────────────────
  major [Functional Correctness]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/AGENTS.md:90AGENTS.md:90-92]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @AGENTS.md around lines 90 - 92, Update the normative spec range in
  AGENTS.md so it explicitly includes docs/specs/0020-mcpgate.md as
  authoritative alongside 0001–0019. Adjust the “Specs” guidance to
  reference the numbered subsystem specs through 0020, keeping the rest of
  the contract unchanged and still directing normative specs to docs/specs/
  rather than .agents/specs/.


────────────────────────────────────────────────────────────────────────
  major [Security & Privacy]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/hooks/policy.rs:143crates/swe-seed-core/src/hooks/policy.rs:143-150]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @crates/swe-seed-core/src/hooks/policy.rs around lines 143 - 150, The
  policy.rs flow in the local/federation decision path currently lets
  GateOutcome::Allow override a local ActionGate::Forbidden, which
  breaks the fail-closed behavior. Update the gate_action/authority_gate
  handling so the local decision remains authoritative for forbidden cases:
  only accept GateOutcome::Allow when the local gate is not forbidden, and
  otherwise return ActionGate::Forbidden before mapping the federation
  result.


────────────────────────────────────────────────────────────────────────
  minor [Data Integrity & Integration]
  → ]8;;vscode://file//home/sprime01/projects/SWE_SEED/docs/specs/0018-layer-boundary-governance.md:42docs/specs/0018-layer-boundary-governance.md:42-43]8;;

  ▶ Prompt for AI agent
  Verify each finding against current code. Fix only still-valid issues,
  skip the rest with a brief reason, keep changes minimal, and validate.

  In @docs/specs/0018-layer-boundary-governance.md around lines 42 - 43,
  Clarify the authoritative relationship between SeedPackageManifest and
  registry.toml in the spec so there is no registry/file ambiguity. Update
  the section around SeedPackageManifest to state normatively whether it is
  the same on-disk file as registry.toml or a higher-level artifact that
  contains it, and remove the question-style wording from the authoritative
  text in 0018. Keep the canonical definition tied to SeedPackageManifest
  and align the later registry discussion to that same symbol so the layout
  is fully specified.
