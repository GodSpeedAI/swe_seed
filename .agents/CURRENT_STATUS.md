# Current Status

Last updated: 2026-07-29
Note: `.agents/` is non-authoritative scratch (AGENTS.md). Verify against committed sources.

Outcome: Promoted the two recorded DEBT items into spec 0020 as normative requirements and
implemented both. (1) Live catalog discovery (spec §7 "Live Catalog Discovery"): at startup and
on each validated reload, the gateway fans out `tools/list`/`resources/list`/`prompts/list` to
registered backends, namespaces the live entries, and merges them declared-wins; per-backend
isolation (one unavailable backend contributes no live entries but doesn't break the others);
dropped live entries + per-backend notes are reported to stderr (spec §7 "dropped and reported").
Catalog construction has one owner: `catalog::merge_live`; the I/O fan-out lives in `discover.rs`.
(2) Bounded concurrency (spec §15 "Concurrency and Overload"): a finite worker pool
(`DEFAULT_WORKERS=4`, `DEFAULT_QUEUE=16`) pulls accepted connections from a bounded
`sync_channel`; on overflow the accept thread drains the bounded request body and rejects with
HTTP 503. Runtime is `Arc`-shared; governance/audit/breaker locks remain exact under concurrency.

## Current proof

- `cargo test -q --workspace` → 337 passed, 0 failed.
- New: 5 discovery tests (live namespacing, backend-failure isolation, no-transport note,
  declared-wins conflict, duplicate-live drop) + 1 end-to-end live-discovery-through-real-bind
  + 3 concurrency tests (two slow calls overlap not serially, 503 on queue-full overload with
  bounded drain, governance/audit counts exact under 4-way concurrency).
- `cargo build --release -q -p swe-seed` → 0 warnings. `just ci` → exit 0.
- neatcode critique (Deep depth): scored correctness 4 · fit 5 · semantics 4 · restraint 5 ·
  operations 4 · evidence 5. One spec-MUST finding (§7 "reported") fixed mid-critique: discovery
  drops/notes now eprintln'd at bind rather than discarded.

## Current next action

Both promoted requirements are implemented and proven. Remaining Minor (not blocking): discovery
notes/drops go to stderr only — surface via `gateway doctor`/`list` if operator observability is
wanted. The neatcode skill lives at `.agents/skills/neatcode/` (not in the opencode skill index;
loaded by path). The working tree still carries unrelated filesystem-integrity hardening changes
that should be committed separately from the gateway work.

Last updated: 2026-07-02
Note: `.agents/` is non-authoritative scratch (AGENTS.md). Verify against committed sources.

Outcome: Revised `.agents/plans/mcp-gate.md` from a standalone MCP gateway spec into a
SWE_SEED-specific MCPGate delta implementation plan. The plan now defers to
`docs/specs/0020-mcpgate.md`, removes duplicate or irrelevant standalone-tool scope, keeps
federation behind `swe_seed_core::federation`, and targets a lightweight Rust daemon mode in
the existing `swe-seed` binary with no Docker requirement.

## Current proof

- `cargo run -q -p swe-seed -- route "review and revise .agents/plans/mcp-gate.md for SWE_SEED delta capabilities without Docker, preserving federation"` selected `review`.
- `cargo run -q -p swe-seed -- context-plan "review and revise .agents/plans/mcp-gate.md for SWE_SEED delta capabilities without Docker, preserving federation"` passed and selected the review context budget.
- `just harness-validate` passed.
- `cargo run -q -p swe-seed -- harness` passed.
- Targeted content check found the revised plan explicitly excludes Docker, a separate standalone registry, and a standalone setup wizard; it requires `swe_seed_core::federation`, `ContextBudget`/`ContextPack`, `SeedPackageManifest`, and `FabricatorProofRecord` evidence links.

## Current next action

Implement MCPGate from `.agents/plans/mcp-gate.md` stage 1: add the `gateway` module and
CLI command family with typed errors and model serialization tests.

## Previous status

Outcome: PR #1 is open for the portability/install remediation from `.agents/reports/portability_installation_inspection_2026-07-02.md`; CI fixes are being applied on branch `registry-provenance-core`.

## Previous proof

- Fixed the Rust compile blocker at `crates/swe-seed/src/federation_cli.rs:252` (`envelope.namespace()`).
- Replaced active Python harness proof/install references with Rust `swe-seed`/`just` commands in route cards, README, package scripts, docs, tests, and CI adapter templates.
- Added harness validation that rejects route-card proof commands referencing `python scripts/harness.py` or `bash tests/validate-harness.sh`.
- Removed tracked local metadata under `.omc/` and `.serena/`; added both directories to `.gitignore` and `.prettierignore`.
- `cargo check -q` passed.
- `cargo test -q -p swe-seed-core --test harness_validate` passed.
- `just harness-validate` passed.
- `just ci` passed.
- `just parity` passed.
- `just harness-route "follow all recommended portability installation steps from report"` and `just harness-context-plan "follow all recommended portability installation steps from report"` passed.
- Trace recipe smoke passed for `just harness-trace-start`, `just harness-trace-checkpoint`, `just harness-trace-resume`, and `just harness-trace-finish` using ignored trace artifacts.
- PR #1 CI exposed a fresh-clone failure in `route_matches_golden_checkpoint_smoke`; fixed by reading the committed `tests/fixtures/golden/route_test.out` fixture instead of ignored `.agent-harness/traces/...`.
- `cargo test -q -p swe-seed-core --test route_golden` passed.
- `just ci` passed after the route golden fixture fix.

## Previous next action

Commit and push the route golden fixture fix, then watch PR #1 checks and merge/delete branch after all jobs pass.

## Earlier status

Outcome: Created `.agents/reports/syntelligent_infrastructure_fable.md` as a non-authoritative resolved-decision addendum for the Syntelligent Infrastructure open questions. This is scratch documentation only; no committed specs, route cards, validation, or runtime behavior changed.

## Previous proof

- `just harness-route "update .agents/reports/syntelligent_infrastructure_fable.md to incorporate answers to open questions"` passed and selected `harness_improvement`.
- `just harness-context-plan "update .agents/reports/syntelligent_infrastructure_fable.md to incorporate answers to open questions"` passed.
- Focused content proof passed: `rg` found all 13 resolved decision headings in `.agents/reports/syntelligent_infrastructure_fable.md`; `wc -l` reported 137 lines.
- `just harness-validate` passed.

## Previous next action

Promote any decision that should be binding from `.agents/reports/` scratch into the relevant committed spec, AGENTS.md, or service documentation. Do not claim executable harness behavior changed from this addendum alone.

## Earlier status

Outcome: Addressed `.tmp/cr.md` review replay items for `registry-provenance-core`; added a stricken completion ledger for all 35 items in `.tmp/cr.md`; `just ci` passed after targeted fixes and golden updates.

## Previous proof

- `cargo test -p swe-seed-core --test federation_signing --test trace_ledger --test routing_gate` passed.
- `cargo test -p swe-seed --test cli_golden context_plan_matches_golden` passed.
- `cargo test -p swe-seed-core --test route_golden route_matches_golden_checkpoint_smoke` passed.
- `just ci` passed.

## Previous status

Outcome: Implement the 6 audit-remediation items (audit: `.agents/reports/AUDIT_2026-06-29_*.md`). All 6 done + verified. **158 tests, 0 warnings, `just ci` green, Python fully removed.**

## Done (all verified 2026-06-29)

1. **Specs → `docs/specs/`.** 19 numbered specs (`0001`–`0019`) moved from gitignored `.agents/specs/` to committed `docs/specs/`; all code/test/doc/spec self-refs repointed; `.agents/specs/` removed. `AGENTS.md` now states `.agents/specs/` MUST NOT exist.
2. **`.agent-harness/memory/` canonical.** AGENTS.md documents the split: `.agent-harness/memory/` (committed) = durable harness memory; `.agents/{reports,plans,OPEN_QUESTIONS.md,CURRENT_STATUS.md,DEBT.md}` = non-authoritative scratch.
3. **AGENTS.md source-of-truth rules** added (Source of Truth + Local Agent Memory sections).
4. **7 harness baml types registered + ValidationRequirement consolidated.** New `crates/swe-seed-core/src/contracts/harness.rs` (ArtifactStatus, ProofDisposition, ValidationRequirement, HarnessNeed, HarnessADR, RegenerationInput, RegenerationPlan). 6 module-local aliases removed (`HookValidation`, `ContextValidation`, `RouteValidationEntry`, `SkillValidation`, `TraceSchemaValidation`, learning's `ValidationRequirement`) → shared type. baml gaps **12 → 5** (remaining 5 = fabricator learning-analogues, genuinely separate types). skill's lowercase `ArtifactStatus` kept as a load-tolerant variant.
5. **Federation authority wired.** `hooks::gate_action_with_authority(policy, action, cfg, risk, incoming)` calls `authority_gate` only when `authority_mode() != Local`; `gate_action` unchanged. Tested in `permission_gate.rs`.
6. **Phase 10 cutover (Python removed).** `scripts/harness.py validate` load-bearing subset ported to Rust `swe-seed harness` (`crates/swe-seed-core/src/harness_validate.rs` + `harness_cli.rs`); `scripts/ci.sh` rewired to `cargo run -p swe-seed -- harness` + `cargo test`; `tests/validate-harness.sh` deleted; justfile rewired to Rust; `scripts/agent-hooks` launcher + 4 Python files removed (`harness.py`, `agent_hooks.py`, `fabricate.py`, `.strategy/strategy.py`). `.prettierignore` added for normative docs. `just ci` green post-removal.

## Proof

- `cargo test` → 158 passed, 0 failed; `cargo build` → 0 warnings.
- `just ci` → exit 0 (doctor + format + lint + Rust validate + cargo test).
- `cargo run -p swe-seed -- harness` → "Harness validation passed" (behavior-preserving vs the former Python validate).
- `SEA_ROOT=/nonexistent` standalone smoke still green (seed/route/eval/doctor/fabricate).

## Surfaced gaps (deferred, not silently dropped)

- The **doctrine phrase/word-count checks** from `harness.py validate` (behavior-shaping phrases in AGENTS.md, playbook min-word counts, 9arm phrases, memory phrases, reflection/eval phrase checks) were NOT ported — they are content-quality nudges, not structural integrity, and are config-driven. `swe-seed harness` covers the integrity spine (specs/baml/dirs/skills/routes/render-targets/incomplete-markers). Port the doctrine checks in a follow-up if that behavior-shaping coverage is wanted.
- `scripts/sync-learning-store.sh` repointed to `cargo run -p swe-seed -- trace distill`; its JSON-shape assumption (`learning_review` key) was not re-verified against the Rust distill output.
- `docs/**/*.md` and root specs still prose-reference the old Python CLIs (stale documentation, not load-bearing).
- Audit F2 (concurrent multi-agent editing) is a **process** decision, not code — needs an operating-model choice (single-writer vs worktree-per-agent).

## Next

Decide on concurrent-writer model (audit F2); optionally port the doctrine checks; Phase 10 final parity gate (release build + golden CLI parity beyond `route`).
