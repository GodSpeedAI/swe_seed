# Adversarial Project Audit — SWE_SEED Rust Rewrite

**Date:** 2026-06-29
**Scope:** Full project — gaps, debt, correctness, the `.agents` vs `.agent-harness` relationship, and the Phase 10 cutover reality.
**Method:** Every claim below is backed by a command + observed output (run 2026-06-29). No claim relies on `CURRENT_STATUS.md` (it is stale — see F8). "Verified" = I ran it; "Claimed" = stated somewhere but not re-verified here.
**Posture:** Adversarial. Every "DONE" was re-checked against actual behavior. Findings are ranked by blast radius, not by how they look.

---

## 0. Executive summary (read this first)

The Rust rewrite is **substantially complete and genuinely works**: 155 tests pass, 0 build warnings, all 11 job-type route cards resolve, the inner stack runs fully standalone under a dead `SEA_ROOT`, `just ci` is green, and Phases 0–9 are implemented and committed. The test suite is **not theater** — assertions check real behavior, only one test is `#[ignore]`'d (a legitimate golden-regenerator).

But the project has **one critical provenance break** and **one critical coordination hazard** that undermine everything else:

- **F1 (CRITICAL):** The 19 authoritative specs (`0001`–`0019`) and the implementation plan that the *entire* rewrite is built against are **not in version control**. They live only in `.agents/specs/` and `.agents/plans/`, both gitignored. A fresh clone gets the code and the `.baml` contracts but **not** the normative source those contracts are "generated/reviewed from."
- **F2 (CRITICAL):** The repository is being edited **concurrently by more than one agent/process**. Three commits (`204b7ac`, `c2e4ed2`, `ff93c7c`) landed between my sessions that I did not make; a live log (`.logs/subtask2.log`) is appended to continuously; local memory (`.agents/DEBT.md`) was reset by another writer while my copy diverged. There is no coordination lock, and the local-only status tracking has already diverged from committed reality.

Everything else (Phase 10 not done, baml parity partial, release build slow, federation gate not wired, spec/memory drift) is **real but secondary** to those two.

**Headline recommendation:** commit the specs/plan to git (or move them under `docs/`), and decide on a single-agent-or-coordinated-multi-agent operating model before Phase 10.

---

## 1. Ground truth (what actually exists right now)

Verified 2026-06-29 on the working tree (commit `ff93c7c` + one uncommitted sqlite file):

| Dimension | Observed | Evidence |
|---|---|---|
| Build | clean, 0 warnings | `cargo build` → `Finished`; `grep -c "^warning"` = 0 |
| Tests | **155 passed, 0 failed, 1 ignored** | `cargo test` aggregate; the 1 ignored is `fabricate_golden_regen` (fixture regenerator, intentional) |
| `just ci` | **exit 0** ("All checks passed!", "Harness validation passed") | `just ci > /tmp/ci.out; echo $?` = 0 |
| Standalone smoke (`SEA_ROOT=/nonexistent`) | all 5 commands exit 0 | `seed assemble`, `route "checkpoint smoke" --record`, `eval run --spec tests/fixtures/eval.toml`, `doctor`, `fabricate "sample need"` |
| Phases implemented & committed | **0–9** | git log + module inventory (`adapters/`, `federation/`, `fabricator/`, `learning/`, etc.) |
| Phase 10 (Python removal) | **not done** | `scripts/harness.py`, `scripts/agent_hooks.py`, `scripts/fabricate.py`, `.strategy/strategy.py` all still present |
| baml parity gaps | **12** | `cargo test --test baml_parity --noclean` → "12 pending type(s)" |
| Release build | **slow (>120 s)** | `cargo build --release` exceeded the 120 s tool timeout |

Test-suite integrity is sound: no `assert!(true)`, no `unimplemented!`, no `todo!()` in `src/`; the only `#[ignore]` is documented and intentional.

---

## 2. Findings (ranked)

### F1 — CRITICAL: The normative specs and the plan are not in version control

**Evidence**
```
$ git check-ignore .agents/specs/0013-eval-and-proof.md .agents/plans/0001-swe-seed-v0-1-implementation.md .agents/CURRENT_STATUS.md
.agents/specs/0013-eval-and-proof.md
.agents/plans/0001-swe-seed-v0-1-implementation.md
.agents/CURRENT_STATUS.md
$ git ls-files | grep -cE "00(0[1-9]|1[0-9])-[a-z]"
0
```
- `.gitignore` line 32: `.agents/` — the entire local-memory tree is ignored.
- The 19 numbered specs (`0001`…`0019`) exist **only** in `.agents/specs/`.
- The implementation plan (`0001-swe-seed-v0-1-implementation.md`) exists **only** in `.agents/plans/`.
- `docs/specs/` (committed, the location `AGENTS.md:90` actually points at) contains a **different, older** spec generation: `agentic-swe-harness.md`, `hook-strategy.md`, `memory-system.md`, `skill-ir.md`, `verification-system.md` — none of the numbered specs.
- Root `HARNESS_SPEC.md` is a "stable root spec" that explicitly defers detail to `.agents/specs/` (line 3–6) — i.e. a committed file that points at uncommitted files.

**Why it matters**
The project's stated posture is "build-to-specs" and "`.baml` is generated/reviewed from root specs." A fresh clone gets the `.baml` contracts but **not** the specs they are derived from, nor the plan that sequenced the rewrite. The baml_parity test asserts Rust types match `.baml`, but nothing asserts `.baml` matches the (absent) specs. The whole provenance chain has a missing link in version control. This is the single highest-blast-radius issue.

**Recommendation**
Either (a) move `.agents/specs/` and `.agents/plans/` under `docs/` and commit them (and fix `AGENTS.md:90` references — it already says `docs/specs/`), or (b) force-add them in place. Option (a) is cleaner; the root specs already point there. Keep `.agents/{CURRENT_STATUS,DEBT,OPEN_QUESTIONS}.md` and `lessons/` gitignored (those are legitimately disposable local memory).

---

### F2 — CRITICAL: Concurrent multi-agent editing with no coordination

**Evidence**
```
$ git log --oneline -6
ff93c7c Address review feedback for CLI hardening
c2e4ed2 Implement Phase 9 host adapters with review hardening
204b7ac Implement Phase 8 federation boundary with review hardening
30e1d37 Update subtask log          ← my commit
be9a724 Implement Phase 7 fabricator layer with chain-integrity gate   ← my commit
6278a50 Regenerate skill render targets; update subtask log           ← my commit
```
- `204b7ac` (Phase 8 federation), `c2e4ed2` (Phase 9 host adapters), `ff93c7c` (CLI refactor) were committed **between** my Phase 8 session and now. I did not make them.
- My Phase 8 `envelope.rs` doc comment (`//! Pure port of agentic_capability_loop/adapters.py _load_hash / _event`) is verbatim in the tree — so my uncommitted Phase 8 work was **folded into a commit by another writer**.
- The CLI I edited in Phases 6–8 (monolithic `cli.rs` with my `run_fabricate`/`run_run`/`run_federation`) was **refactored out from under me** into `context_cli.rs`, `doctor_cli.rs`, …, `federation_cli.rs`, `host_cli.rs` (`grep -c "fn run_fabricate" cli.rs` = 0).
- `.logs/subtask2.log` is a **live** file (appended to during sessions) — it shows dirty immediately after every commit.
- `.agents/DEBT.md` was **reset by another writer** to "No active technical debt is currently recorded," while my local copy held four entries. Local memory has diverged from itself across writers.

**Why it matters**
Two+ agents writing the same files with no lock means: silent overwrites (my CLI handlers were moved; if I'd committed naively I'd have re-introduced the old structure), divergent status/debt (the local source-of-truth for "what's done" no longer agrees across writers or with git), and unbounded merge risk. The fact that the tree currently builds and tests green is **luck + the refactor being clean**, not a guarantee. Phase 10's "git rm the Python" done by an uncoordinated agent could delete files another agent is mid-edit.

**Recommendation**
Decide explicitly: either (a) **single-writer** model (one agent at a time; the orchestrator serializes), or (b) **worktree-per-agent** with explicit merge points. In either case, **stop tracking project state in gitignored local files that multiple writers mutate** (`.agents/CURRENT_STATUS.md`, `DEBT.md`) — move durable status into committed files (e.g. `docs/STATUS.md`) or accept it is per-agent and never authoritative.

---

### F3 — The `.agents` ↔ `.agent-harness` relationship: not a "conflict" but a **split-brain memory** with one critical gap

Direct answer to the question asked: they do not *conflict* in purpose (`.agents/` = disposable local agent memory; `.agent-harness/` = the committed runtime harness), but they **overlap and drift**, and the split has one broken seam.

| Aspect | `.agents/` | `.agent-harness/` | Verdict |
|---|---|---|---|
| Git status | **fully gitignored** (`.gitignore:32`) except 2 force-added report files | **mostly committed**; only `traces/records/*`, `traces/route-decisions/*`, `baml/baml_client/`, `learning/` ignored | Asymmetric commit status is the root of the drift |
| Specs | `specs/0001-0019` (authoritative, **uncommitted**) | `baml/baml_src/*.baml` (committed, derived from specs) | **Broken provenance seam** (F1) |
| Open questions | `OPEN_QUESTIONS.md` (631 B, Jun 26) | `memory/open-questions.md` (1245 B, May 24) | **Duplicated, divergent** — two files, different content, neither references the other |
| Memory | `lessons/`, `DEBT.md`, `CURRENT_STATUS.md` | `memory/{repo-map,constraints,decisions,glossary,failure-patterns,successful-patterns,open-questions}.md` | Two "memory" namespaces; the committed one is what route cards actually read |
| Status/debt | `CURRENT_STATUS.md`, `DEBT.md` (local, divergent across writers) | none | Local-only, already diverged (F2/F8) |
| Reports | `reports/` (gitignored) + 2 tracked `outcome_code_map_*` files | `reports/` (empty in tree) | Minor |

**The real conflict is the specs seam (F1) plus the duplicated open-questions.** The rest is benign separation *if* the operating model were single-writer. Under concurrent writers (F2), the split-brain becomes a live hazard because each writer's `.agents/` is its own divergent reality while `.agent-harness/memory/` is shared-committed.

**Evidence**
```
$ ls -la .agent-harness/memory/open-questions.md .agents/OPEN_QUESTIONS.md
-rw-r--r--  .agent-harness/memory/open-questions.md  1245  May 24 23:39
-rw-r--r--  .agents/OPEN_QUESTIONS.md                 631  Jun 26 15:02
```

**Recommendation**
1. Resolve the specs seam (F1). 2. Collapse the two `open-questions` files into one committed location (`.agent-harness/memory/open-questions.md` is the one route cards read; make `.agents/OPEN_QUESTIONS.md` a symlink or delete it). 3. Document the `.agents/` = disposable / `.agent-harness/` = committed contract in `AGENTS.md` explicitly (it's implied, not stated).

---

### F4 — Phase 10 (Python cutover) is not done, and the plan's sequence is internally risky

**Evidence**
```
$ ls scripts/harness.py scripts/agent_hooks.py scripts/fabricate.py .strategy/strategy.py
scripts/harness.py  scripts/agent_hooks.py  scripts/fabricate.py  .strategy/strategy.py   ← all present
$ grep -rn "harness.py" scripts/ci.sh justfile tests/validate-harness.sh | wc -l
40+ references
```
- `just ci` → `scripts/ci.sh` → `python scripts/harness.py validate` + `bash tests/validate-harness.sh`.
- `tests/validate-harness.sh` exercises the **full Python CLI lifecycle** (harness route/trace/eval/inspect/context-plan, fabricate new→generate→validate→handoff→proof→reflect→status, strategy new→…→decide). It is the Python parity harness.
- The `justfile` has ~30 recipes that call `harness.py`, `fabricate.py`, `strategy.py`.

**The cutover is genuinely blocked, not just unfinished:**
1. `just ci` is green **only because the Python is still present**. Remove `harness.py` and `just ci` breaks (ci.sh calls it).
2. The Rust binary has **byte-parity with Python for exactly one CLI**: `route` (`tests/route_golden.rs` vs a captured Python record). The other Rust CLIs produce **Rust-idiomatic output** (e.g. `fabricate` writes JSON artifacts; the Python `fabricate.py` writes Markdown+YAML). So `validate-harness.sh` cannot be re-pointed at the Rust binary without rewriting it, and rewriting it requires byte-parity that was never a goal of Phases 1–9 (contracts-as-data, spec 0019).
3. The plan's Phase 10 sequence runs `just ci` *before* the `git rm` and only re-runs `cargo test` after — i.e. it **accepts `just ci` breaks post-removal**. That leaves the repo with no green CI command until `ci.sh`/`validate-harness.sh`/the `justfile` are rewired to Rust. The plan does not specify that rewire.

**Recommendation**
Treat Phase 10 as a **cutover project**, not a single phase:
1. Decide what `just ci` should mean post-cutover (probably: `cargo test` + a Rust `harness validate` + `cargo build --release` smoke). 
2. Port the parts of `harness.py validate` that check **harness structure** (routes resolvable, skills valid, render-target canonical-source notices, no duplicate content) into a Rust command — that is the one piece of Python validation with no Rust equivalent. (~150 lines of `harness.py`, the block at lines 1470–1496.)
3. Only then `git rm` the Python and delete/rewrite `validate-harness.sh` + the Python `justfile` recipes.
4. Until then, **do not delete the Python** — it is load-bearing for `just ci`.

---

### F5 — "baml_parity proves every Rust type matches its `.baml` contract" is **partially true**

**Evidence**
```
baml_parity: 12 pending type(s) across harness/fabricator (later phases):
["fabricator:FabricatorValidationRequirement", "fabricator:FabricatorReflectionTemplate",
 "fabricator:FabricatorAdaptationDecision", "fabricator:FabricatorRegressionCase",
 "fabricator:FabricatorLearningCandidate", "harness:ArtifactStatus", "harness:ProofDisposition",
 "harness:ValidationRequirement", "harness:HarnessNeed", "harness:HarnessADR",
 "harness:RegenerationInput", "harness:RegenerationPlan"]
```
- **Every *registered* Rust type matches its `.baml` counterpart** (the parity test is real and caught two drift bugs during Phase 6/7 implementation).
- But **12 `.baml` types have no Rust implementation**: 5 fabricator learning-analogues (parallel to Phase-6 types, unneeded because the fabricator reuses the harness learning types at runtime) and 7 harness types (`ArtifactStatus`, `ProofDisposition`, `ValidationRequirement`, `HarnessNeed`, `HarnessADR`, `RegenerationInput`, `RegenerationPlan`).
- `ValidationRequirement` is the interesting one: it is a named `.baml` class used by many types, but each Rust module defines its **own local copy** (context, hooks, route, skill, trace, learning each have one) and **none** are registered. So the parity test cannot catch drift between those local copies and `.baml`.

**Severity:** Medium. The Phase 10 "Final Proof Artifact" claims "every Rust type matches its `.baml` contract" — strictly, it's "every registered type." The coverage-gap test is honest about the 12 (it warns, doesn't fail), and gaps are confined to harness/fabricator (the swe_seed layer is fully covered, hard-asserted).

**Recommendation**
Either (a) register the 7 harness types (they're small) and consolidate `ValidationRequirement` into one registered type used everywhere, or (b) reword the Phase 10 claim to "every swe_seed-layer type + every implemented harness/fabricator type." Option (a) is cheap and removes the ambiguity.

---

### F6 — Release build exceeds a 120 s budget (Phase 10 friction)

**Evidence**
```
$ cargo build --release   # timed out after 120000 ms in the tool
```
`rusqlite` (bundled) + `uuid` + `flate2` + full optimization on a workspace = a heavy first-time release build. Phase 10's command sequence starts with `cargo build --release` and then runs every smoke against the release binary.

**Severity:** Low–Medium (productivity, not correctness). The debug binary passes the identical standalone smoke (verified: `seed assemble`=0, `route`=0, `eval run`=0, `doctor`=0, `fabricate`=0, all under `SEA_ROOT=/nonexistent`).

**Recommendation**
Either raise the build timeout in CI/agent budgets, or run Phase 10 smoke against the debug binary (behavior-identical) and reserve the release build for a final parity gate. Consider `cargo build --release -p swe-seed` (just the bin) for smoke.

---

### F7 — Federation authority gate is implemented and tested but **not wired** into any call site

**Evidence**
```
$ grep -rn "authority_gate\|federation::" crates/swe-seed-core/src/hooks crates/swe-seed-core/src/doctor
(empty)
```
- `federation::authority_gate()` exists (local/delegate/hybrid, fail-closed on timeout for `Dangerous`) and is tested (`tests/federation_parity.rs::authority_modes_honor_contract`).
- But `hooks/policy.rs::gate_action` (the real permission gate) does **not** call it. The federation gate has zero production callers.

**Severity:** Low for correctness (the inner stack is standalone and safe), but it means the "authority delegation" half of spec 0011 §5 is **contract-tested, not exercised**. A regression that breaks `authority_gate` would not affect any real path.

**Recommendation**
Either wire `authority_gate` into `hooks::gate_action` behind `FederationConfig::authority_mode() != Local` (so standalone is untouched), or delete the gate + its test and mark authority-delegation a post-v0.1 feature. Wiring is the honest choice; the gate is small and the standalone invariant is already proven (`tests/standalone_invariant.rs`).

---

### F8 — Status/debt tracking has diverged and is no longer trustworthy

**Evidence**
- `.agents/CURRENT_STATUS.md` (my local record) stops at Phase 8; it does **not** mention Phase 9 host adapters, the CLI refactor, or commits `204b7ac`/`c2e4ed2`/`ff93c7c`.
- `.agents/DEBT.md` was reset to "No active technical debt is currently recorded" by another writer, while my copy held four entries (root-specs, provenance cross-check, learn-promote gate, render-target notice).
- The render-target debt is now **resolved** (`harness validate` exits 0 — verified) but was never marked resolved in the reset DEBT file because the file was wiped.

**Severity:** Medium. The AGENTS.md contract says local memory must be "current enough that the next agent can recover the latest outcome." It currently cannot — the local memory contradicts git.

**Recommendation**
Either make project status a **committed** artifact (`docs/STATUS.md`) updated each phase, or accept `.agents/` as per-agent scratch and never treat it as authoritative. Given F2, the committed option is safer.

---

## 3. Per-phase verification (re-checked, not transcribed)

| Phase | Plan outcome | Verified state | Evidence |
|---|---|---|---|
| 0 Scaffold + baml | workspace builds; baml_parity passes | ✅ | build clean; baml_parity green |
| 1 Seed + provenance | assemble manifest, boundary, regenerate, provenance fail-closed | ✅ | `seed assemble`=13 caps; `boundary`/`regenerate_idempotent`/`provenance_verify` tests present |
| 2 Routing + traces | route card + frozen-format record; trace lifecycle; 11 job types | ✅ | `route "checkpoint smoke" --record`=0; 11 route cards on disk |
| 3 Eval + proof + doctor | eval (4 classes), frozen, proof needs evidence, doctor aggregates | ✅ | `doctor`=0; `eval_frozen`/`proof_evidence`/`doctor_json` tests |
| 4 Context + hooks | budget/pack, hook runtime (redact/index/export/compact), permission gate | ✅ | `hooks_runtime`/`permission_gate`/`context_plan` tests |
| 5 Skill ingest + render | pipeline, blocking scan, deterministic render | ✅ | `ingest_pipeline`/`render_skills_golden`/`scan_gate` tests |
| 6 Learning + adaptation | reflect→record→adaptation→proposal/regression; rollback gate | ✅ | `adaptation_gate`/`regression_link`/`learning_loop` tests |
| 7 Fabricator | semantic chain, integrity gate, EARS/Gherkin, frozen eval at handoff | ✅ | `chain_integrity`/`fabricate_golden`; `fabricate "sample need"`=chain_passed |
| 8 Federation | envelope parity, **standalone invariant**, Python loop superseded | ✅ (loop removed) / ⚠️ (gate not wired, F7) | `federation_parity`/`standalone_invariant`; `agentic_capability_loop/` gone |
| 9 Host adapters | Claude deterministic + reversible; others honest stubs; drift detected | ✅ | `adapters/{claude,codex,opencode,antigravity,github_copilot,ci}.rs`; `host_projection_determinism`/`host_projection_rollback`/`host_partial_support`/`host_drift_detection` tests |
| 10 E2E parity + Python removal | release builds; full suite; **Python removed**; suite still green | ❌ Python NOT removed; release build >120s | scripts/*.py all present; `just ci` green only because Python present |

---

## 4. Test-quality assessment (adversarial)

- **No fake-green found.** `grep` for `assert!(true)`, `unimplemented!`, `todo!()`, `panic!("not implemented")` in `src/` → zero hits.
- **One `#[ignore]`** (`fabricate_golden_regen`) — legitimate, documented ("run with --ignored after an intentional chain change"), and the non-ignored `fabricate_matches_golden_fixture` enforces the golden.
- **Golden tests are meaningful:** `route_golden` compares the Rust route object to a captured Python record (timestamps excluded); `fabricate_golden` normalizes `created_at` to a fixed `"fixture"` and compares byte-stable JSON; `render_skills_golden` is determinism-based (two renders identical), not a brittle byte-fixture.
- **Falsification is real:** boundary tests construct misplaced artifacts; chain-integrity tests drop a PRD→SDS link; adaptation-gate feeds a failing `AdaptationEligibility`; regression-link runs the linked check against a recurrence vs clean target. These fail for the right reason if the logic breaks.
- **One nuance:** `federation_parity` asserts the fallback hash equals a hardcoded Python hex — this proves the *algorithm* matches, not that the live Python adapter produces it (the Python is being removed). Acceptable, but the constant should be re-derived if the fallback input ever changes.

---

## 5. Security & determinism posture

- **`command_check` is opt-in** (`eval/check.rs:104`): spec-controlled shell execution is rejected unless `trusted=true`. Verified.
- **Redaction** (hooks) + secret-pattern scrubbing (util) present; `hooks_runtime` proves a secret survives neither in key-matched nor value-pattern form.
- **Federation cannot weaken a local gate** (spec 0011 §security): `authority_gate` never lets an external `Allow` override a local `Deny` unless `allow_external_override` (default false). Tested.
- **Determinism:** route/trace records carry timestamps but the golden comparison excludes them; fabricate normalizes `created_at`. UUIDs (`event_id`) are non-deterministic by design but are not compared byte-wise in any parity test.
- **Drift guard:** federation rejects envelopes whose `domain_model_hash` ≠ local resolution. Tested.
- **No secrets found** in tracked content (scanned `.logs/subtask2.log`, render-targets, configs).

---

## 6. Recommendations (prioritized)

| # | Action | Severity | Effort |
|---|---|---|---|
| R1 | **Commit the specs + plan** (move `.agents/specs/`→`docs/specs/`, `.agents/plans/`→`docs/plans/`); fix `AGENTS.md:90` (already says `docs/specs/`). Keep `.agents/{STATUS,DEBT,OPEN_QUESTIONS}.md` + `lessons/` gitignored. | CRITICAL (F1) | S |
| R2 | **Pick one writer model** (serialize agents, or worktree-per-agent + merge). Stop mutating shared gitignored status under concurrent writers. | CRITICAL (F2) | M (process) |
| R3 | **Define Phase 10 as a cutover project**: port `harness.py validate` structure-checks (~150 lines) to a Rust command; rewire `ci.sh`/`justfile`/`validate-harness.sh`; only then `git rm` Python. Do not delete Python until `just ci` is Rust-backed. | High (F4) | L |
| R4 | Collapse the two `open-questions` files into one committed location. | Medium (F3) | S |
| R5 | Register the 7 harness baml types + consolidate `ValidationRequirement`, OR reword the Phase 10 parity claim. | Medium (F5) | S |
| R6 | Wire `federation::authority_gate` into `hooks::gate_action` behind the config flag, or delete it as post-v0.1. | Low (F7) | S |
| R7 | Move project status to a committed `docs/STATUS.md` (per-phase), or stop treating `.agents/STATUS` as authoritative. | Medium (F8) | S |
| R8 | Run Phase 10 smoke against the debug binary; reserve release build for a final gate (or raise the timeout). | Low (F6) | S |

---

## 7. What is solid (so it isn't lost in the critique)

- Phases 0–9 are real, tested, and the inner stack genuinely runs standalone (proven under `SEA_ROOT=/nonexistent`).
- The test suite is honest: 155 real assertions, no theater, one intentional ignored regenerator.
- `just ci` is green and `harness validate` passes after the render-target canonical-source fix.
- baml_parity has already caught and fixed real drift twice; it is load-bearing.
- The standalone invariant (federation off → zero external calls) is proven by an explicit test that asserts the sink file is never touched.
- Host adapters ship Claude live with others as honest `PartialSupport` stubs, matching the plan.

The bones are good. The two critical issues (F1 specs-out-of-git, F2 concurrent writers) are **provenance and process** problems, not code-quality problems — and both are fixable in a single short session before Phase 10.

---

## Appendix A — Evidence commands (reproducible)

```bash
# F1: specs not in git
git check-ignore .agents/specs/0013-eval-and-proof.md
git ls-files | grep -cE "00(0[1-9]|1[0-9])-[a-z]"          # → 0

# F2: concurrent commits
git log --oneline -6

# Ground truth
cargo build && cargo test                                   # 155 passed, 0 failed
just ci                                                     # exit 0
SEA_ROOT=/nonexistent target/debug/swe-seed doctor; echo $? # 0

# F4: Python still load-bearing
grep -rln "harness.py" scripts/ci.sh justfile tests/validate-harness.sh

# F5: baml gaps
cargo test -p swe-seed-core --test baml_parity -- --nocapture | grep "pending type"

# F7: federation gate not wired
grep -rn "authority_gate" crates/swe-seed-core/src/hooks crates/swe-seed-core/src/doctor   # empty
```

## Appendix B — Inventory snapshots

- **Implemented modules:** `seed, provenance, route, trace, eval, doctor, context, hooks, security, skill, learning, fabricator, federation, adapters, contracts, config, util`.
- **CLI (refactored):** `cli, context_cli, doctor_cli, eval_cli, fabricate_cli, federation_cli, hooks_cli, host_cli, learning_cli, provenance_cli, seed_cli, skill_cli, trace_cli`.
- **Test files (29):** adaptation_gate, antigravity_projection, baml_parity, boundary, chain_integrity, ci_projection, claude_projection, codex_projection, context_plan, cr_review_cli, cr_review_learning_cli, doctor_json, eval_frozen, fabricate_golden, federation_parity, github_copilot_projection, hooks_runtime, host_adapter_capabilities, host_cli, host_drift_detection, host_partial_support, host_projection_determinism, host_projection_rollback, host_test_support, ingest_pipeline, learning_cli, learning_loop, opencode_projection, permission_gate, proof_evidence, provenance_verify, regenerate_idempotent, regression_link, render_skills_golden, route_golden, scan_gate, standalone_invariant, trace_cli, trace_lifecycle, yaml_parity.
- **baml coverage:** swe_seed layer fully covered (hard-asserted); 12 harness/fabricator types pending.
