# Outcome-to-Code Map: SWE_SEED

## 1. Executive Summary

- **Repo name:** SWE_SEED (`swe_seed`), GodSpeedAI org
- **Audit date:** 2026-06-14
- **Languages/frameworks:** Python 3.12 (CLI harnesses), Bash (CI/hooks/secrets), YAML/JSON (route cards, Skill IR, schemas, configs), BAML (typed contracts via `baml-py`), Jinja-like `{{placeholder}}` templates (custom renderer, not Jinja2), Markdown (specs/playbooks/evals/memory), Prettier/Ruff (format/lint), `just` (command surface), SOPS+age (secrets), SQLite/rusql (learning store + hook index)
- **Main purpose (evidenced):** A low-touch development harness for AI coding agents that routes work by outcome, loads bounded context, enforces proof-before-completion structurally, records auditable traces, and improves itself through evidence-backed proposals. Layered: core harness + agent-hooks observability + fabricator + strategy + (orphaned) agentic capability loop.
- **Outcomes found:** 22
- **Implemented:** 16
- **Partially implemented:** 3
- **Declared only:** 2
- **Test only:** 1
- **Contradicted:** 0 (but several overclaims — see section 6)
- **Unclear:** 0

Note: 12 of 22 outcomes have full detailed maps in section 4; the remaining 10 are covered by the inventory table (section 3), contradictions (section 6), missing proofs (section 7), and the frontier brief (section 8). All 22 carry repo evidence in the inventory.

### Top 5 risks

1. **Completion-gate is advisory, not enforced.** Spec mandates Anti-Delusion Gates that "MUST reject, block, or flag" unsupported completion claims and controls that "SHOULD NOT exist as advice that cannot affect agent action" (`HARNESS_SPEC.md:407-423`, `:364-365`). Runtime `trace_finish` stamps any claim without checking proof; hook-router only emits guidance text; `agent_hooks.py` has no blocking handler. An agent can claim completion without proof and the harness records it (classified "missing"/"partial", but not blocked).
2. **"Block unauthorized file writes" / "block premature completion" README claims have no enforcing implementation.** README (`README.md:23`, `:56`) promises tool-call safety checks and proof review at turn end; neither blocks anything.
3. **Fabrication proof is static pattern-matching hardcoded to one example game.** "BAML generation model" doc implies LLM/BAML-driven generation; actual `render_template` is `{{key}}` regex substitution and `static_proof_checks` is tailored to "Focus Runner" tokens.
4. **`agentic_capability_loop` + 5-file pytest suite are orphaned from CI and the harness CLI.** Real tested code (event adapters, cross-language contracts, hash consistency) that is not imported by any runtime module and not executed by `just ci`. Known weak links (naive placeholder hash, stubbed Rust services) are self-documented in `.agent-harness/reports/godspeed_stack_agentic_loop_wiring_report.md`.
5. **Eval executor is static-only.** `harness.py eval run` supports `file_exists` / `static_required_patterns` / `static_forbidden_patterns` / `manual_check` — no dynamic/runtime test execution. Evals verify artifact shape, not behavior.

## 2. Method

**Files/directories inspected (directly):**

- Root: `README.md`, `AGENTS.md`, `HARNESS_SPEC.md` (key sections), `pyproject.toml`, `justfile`, `package.json`, `.github/workflows/ci.yml`
- Core code: `scripts/harness.py` (2293 lines, all argparse handlers + key functions), `scripts/agent_hooks.py`, `scripts/fabricate.py`, `.strategy/strategy.py`, `agentic_capability_loop/adapters.py`
- Config: `.agent-harness/config.yaml`, `.agent-hooks/config.yaml`, `.fabricator/config.yaml`, `.strategy/config.yaml`
- Routes: `implementation.json`, `bugfix.json`, `review.json` (full); tree of all 11
- Skills: `implement-with-proof.json` (full); tree of all 7
- Specs/docs: `docs/specs/{verification,hook-strategy,memory-system}.md`, `docs/fabrication-layer/{README,proof-before-completion}.md`, `.agent-harness/baml/V0_1_RELEASE_CRITERIA.md`, route-map, evals
- Runtime evidence: sample route-decision JSON, `learning-store.sqlite3` schema+counts, `.agent-hooks/index/hooks.rusql`, sample normalized hook payload + result, `validate-harness.sh` (full)
- Prior review: `.agent-harness/reports/godspeed_stack_agentic_loop_wiring_report.md`

**Commands run:** `ls`, `find`, `grep`, `cat`, Python `sqlite3` introspection, structural greps for stubs/TODOs.

**Tools used:** Bash, Read, Grep, Glob (no AST-grep needed; Python CLI tools are linear).

**Limitations / areas not deeply inspected:**

- `HARNESS_SPEC.md` (2865 lines) and `SWE_SEED_SPEC_v0.2.0.md` (large) were sampled via grep + targeted reads, not read end-to-end. Findings about their normative claims are based on the sections cited.
- The 4 background `explore` agents launched for deep code analysis became stuck (~8min, no completion) and were cancelled; the equivalent evidence was gathered directly via targeted reads/greps.
- Live execution of `just ci` / `pytest` was not run during this audit (read-only audit). Status claims rest on code + captured runtime artifacts (75+ real traces, 520KB sqlite, 266KB rusql, hundreds of hook payloads) plus the exhaustive `validate-harness.sh` integration script.

## 3. Outcome Inventory

| ID      | Outcome                                                                                                                  | Actor/User                  | Status                | Confidence | Primary Evidence                                                                           |
| ------- | ------------------------------------------------------------------------------------------------------------------------ | --------------------------- | --------------------- | ---------- | ------------------------------------------------------------------------------------------ |
| OUT-001 | Route a task to a job-type-specific route card via deterministic token-overlap matching                                  | Agent / developer           | implemented           | high       | `harness.py::route`/`score_route`; 75+ real route-decision JSONs                           |
| OUT-002 | Validate the harness's own structural contract (files, routes, skills, memory, playbooks, evals, hooks, BAML)            | Operator / CI               | implemented           | high       | `harness.py::validate`; `tests/validate-harness.sh`                                        |
| OUT-003 | Reject incomplete-work markers (TODO/FIXME/stub/placeholder) in harness artifacts                                        | Operator / CI               | implemented           | high       | `harness.py::validate_no_incomplete_work_markers`                                          |
| OUT-004 | Render a canonical Skill IR into multi-target artifacts (Claude/Copilot/hook/checklist)                                  | Skill author / agent        | implemented           | high       | `harness.py::render_skill`; `.agent-harness/render-targets/**`                             |
| OUT-005 | Record an auditable route-decision trace for each routed task                                                            | Operator / reviewer         | implemented           | high       | `harness.py::write_route_decision`; `.agent-harness/traces/route-decisions/*.json`         |
| OUT-006 | Capture a per-task traceability/learning trace (start/checkpoint/resume/finish/distill) with verification classification | Agent / reviewer            | implemented           | high       | `harness.py::trace_*`/`classify_verification_status`                                       |
| OUT-007 | Require proof commands before agentic work is considered complete (completion gate)                                      | Agent / reviewer            | partially_implemented | high       | `trace_finish` records claims without proof check; spec mandates block                     |
| OUT-008 | Execute an EvalSpec and emit a machine-readable EvalResult                                                               | Operator / CI               | implemented           | high       | `harness.py::run_eval_spec`/`evaluate_check` (static check types only)                     |
| OUT-009 | Plan a bounded context budget per task                                                                                   | Agent                       | implemented           | medium     | `harness.py::context_plan`                                                                 |
| OUT-010 | Maintain a structured (sqlite) mirror of distilled traces and learning reviews                                           | Operator / agent            | implemented           | high       | `learning-store.sqlite3` (78+78 rows); `sync/query/plan/eval-learning-store.sh`            |
| OUT-011 | Connect agent lifecycle events to routing/context/evidence/proof/learning guidance                                       | Agent host                  | partially_implemented | high       | `.agent-harness/hooks/hook-router.sh` (advisory emit + optional capture)                   |
| OUT-012 | Block unauthorized/risky tool calls before execution                                                                     | Agent host / security       | declared_only         | high       | README + spec promise; `agent_hooks.py` has no PreToolUse/deny handler                     |
| OUT-013 | Block premature completion language at turn end unless proof recorded                                                    | Agent host                  | declared_only         | high       | `turn.stop` emits reminder; no scan/block in code                                          |
| OUT-014 | Capture an auditable evidence trail after governed events                                                                | Operator / reviewer         | implemented           | high       | `agent_hooks.py::command_capture`; `.agent-hooks/{payloads,artifacts,logs}` + rusql        |
| OUT-015 | Export observability data to OpenTelemetry and JUnit formats                                                             | Operator / CI               | implemented           | high       | `agent_hooks.py::command_export_otel`/`command_export_junit`                               |
| OUT-016 | Convert a bounded product seed into a fabrication packet                                                                 | Product engineer / agent    | implemented           | high       | `fabricate.py` new→generate (custom `{{placeholder}}` render)                              |
| OUT-017 | Require fabrication proof before marking a run complete and gate adaptation on it                                        | Product engineer / reviewer | partially_implemented | medium     | `static_proof_checks` (real but static, single-example); no hard handoff-before-proof gate |
| OUT-018 | Convert a strategic question into an evidence-backed decision                                                            | Strategist                  | implemented           | high       | `strategy.py` 11 subcommands; real decision records                                        |
| OUT-019 | Provide a typed, event-sourced agentic capability loop with cross-language contracts                                     | Platform integrator         | test_only             | medium     | `agentic_capability_loop/adapters.py` + 5 pytest files; not wired to CLI/CI                |
| OUT-020 | Provide one local CI command that is the default proof and mirrors remote CI                                             | Developer / CI              | implemented           | high       | `scripts/ci.sh`; `.github/workflows/ci.yml` runs `just ci`                                 |
| OUT-021 | Manage secrets (encrypt/decrypt/rotate) without leaking into traces/memory                                               | Operator                    | implemented           | medium     | `scripts/secrets-*.sh`; `.sops.yaml`; `.gitignore` rules                                   |
| OUT-022 | Define typed BAML contracts for harness/fabricator artifacts and regenerate client code                                  | Spec author / CI            | implemented           | high       | `.agent-harness/baml/baml_src/*.baml`; `baml-cli generate` in `validate-harness.sh`        |

## 4. Detailed Outcome Maps

### OUT-001 — Semantic routing to a route card

**Outcome statement:** Given a free-text task, the system selects exactly one of 11 job-type route cards (or a safe bootstrap default) and returns the required context, skills, work loop, artifacts, proof, and next action.

**Actor/User:** Agent / developer issuing a task.

**Settlement / success criterion:** `python scripts/harness.py route "<task>"` returns JSON with `job_type`, `route_card`, `work_loop`, `proof`, `done_when`, `next_action`, and the selection matches documented precedence rules (e.g. "fix a failing test" → bugfix, "implement HARNESS_SPEC.md ..." → harness_improvement).

**Status:** implemented. **Confidence:** high.

**Declared evidence:**

- `README.md::route` (Quick Start) — documents `harness.py route "task"`.
- `.agent-harness/config.yaml::routes` — `semantic_fallback: deterministic_token_overlap`.
- `docs/specs/agentic-swe-harness.md`, `.agent-harness/evals/route-conflicts.md` — precedence rules.
- `.agent-harness/routes/*.json` — 11 route cards with `semantic_triggers`, `positive_examples`, `negative_examples`.

**Implementation evidence:**

- `scripts/harness.py::score_route` (line 1528) — token-overlap scoring: triggers +8, positive examples +3, negative examples -4/-2x, job_type +5.
- `scripts/harness.py::infer_bootstrap_route` (1559) — conservative spec fallback when score ≤ 0.
- `scripts/harness.py::build_route_result` (1595) / `route` (1644) — assembles full result + next_action.

**Test evidence:**

- `tests/validate-harness.sh` — executes 6 routing assertions incl. conflict cases: bugfix-beats-test, harness_improvement-beats-implementation, spec for "make a react todo list", review.
- `.agent-harness/traces/route-decisions/20260525T111113Z-*.json` — real captured decision with `decision_basis: "deterministic token overlap against route-card triggers"`.

**Runtime path:** `main()` → `route()` → `build_route_result()` → `score_route()` over all `route_paths()` → write `write_route_decision()` JSON → stdout. With `--record`/`--capture-hook`, also emits to agent-hooks.

**Missing links / gaps:** none material. Router is deterministic keyword-based (no embeddings); matches the spec's stated `deterministic_token_overlap` fallback. `confidence` is honestly "medium"/"low" rather than a fabricated high.

**Reviewer notes:** Confirm the conflict-eval precedence rules still hold after any route-card edits (they are gated by `validate-harness.sh`).

---

### OUT-002 — Harness structural-contract validation

**Outcome statement:** The system fails locally (and in CI) if any required harness artifact, route card, skill, memory file, playbook, eval, hook, or BAML contract is missing, malformed, or too thin.

**Actor/User:** Operator / CI.

**Settlement / success criterion:** `python scripts/harness.py validate` (and therefore `just ci`) exits non-zero with a specific error list on any contract violation; exits 0 with "Harness validation passed" when conformant.

**Status:** implemented. **Confidence:** high.

**Declared evidence:** `HARNESS_SPEC.md::Enterprise Completion Policy` (line 36); `.agent-harness/config.yaml::validation`.

**Implementation evidence:**

- `scripts/harness.py::validate` (line 1167) — ~330 lines checking: root specs, BAML source + release criteria, required files/dirs, memory artifacts (min words + phrases), behavior-shaping phrases in `successful-patterns`/`constraints`/`AGENTS.md`, all 6 hook events, playbooks (min words + phrases + 9arm process phrases), reflection/learning-review/proposal templates, evals (min cases + phrases), render-target canonical rule.
- `validate_skill` (615), `validate_route` (723) — per-artifact validators.
- `tests/validate-harness.sh` — ~220 file-existence checks + ~60 justfile-recipe greps + BAML structural greps + live `harness.py validate`.

**Test evidence:** `tests/validate-harness.sh`; `.agent-harness/evals/core-conformance.md::Eval Case 1`.

**Missing links / gaps:** Validation is structural (presence + phrase counts). It does not verify semantic correctness of route precedence beyond the scripted cases.

---

### OUT-003 — Reject incomplete-work markers

**Outcome statement:** The system rejects TODO/FIXME/TBD/XXX/stub/placeholder markers in tracked harness artifacts so incomplete work cannot hide behind completion language.

**Actor/User:** Operator / CI.

**Settlement / success criterion:** `validate` fails with `incomplete-work marker 'X' in <path>:<line>` if any scanned artifact contains a marker (with documented policy-line exceptions).

**Status:** implemented. **Confidence:** high.

**Implementation evidence:**

- `scripts/harness.py::validate_no_incomplete_work_markers` (line 792) — scans `INCOMPLETE_SCAN_PATHS` (md/yaml/yml/json/py/sh/baml/toml), word-boundary regex for TODO/FIXME/TBD/XXX, substring for others; skips policy-mentioning lines and `harness.py` self-references.
- `INCOMPLETE_WORK_MARKERS` list (line ~248) includes `TODO`, `FIXME`, `placeholder for`, `stub`, etc.

**Test evidence:** wired into `validate` → `just ci`.

**Missing links / gaps:** Scope is limited to `INCOMPLETE_SCAN_PATHS`; markers in untracked paths or generated `baml_client/` are excluded. This is an anti-completion-theater gate but only for the harness's own surface, not for downstream adopter code.

---

### OUT-007 — Proof-before-completion gate (PARTIAL / key gap)

**Outcome statement:** The system prevents an agent from claiming a task complete unless the route's proof commands have run and passed (or skipped checks are justified).

**Actor/User:** Agent / reviewer.

**Settlement / success criterion:** A completion claim without recorded, passing verification is rejected or flagged operator-visibly before the final response.

**Status:** partially_implemented. **Confidence:** high.

**Declared evidence:**

- `HARNESS_SPEC.md::Enterprise Completion Policy` (36-48): "Completion theater is a blocking defect. A completion claim is allowed only when linked route, context, implementation, eval, proof, reflection, and adaptation artifacts satisfy the active route."
- `HARNESS_SPEC.md::Anti-Delusion Gates` (407-423): "The harness MUST reject, block, or flag ... unsupported completion claims ... confidence without proof."
- `HARNESS_SPEC.md::Affordance Control` (364-365): "These controls MUST be attached to executable behavior or validation. They SHOULD NOT exist as advice that cannot affect agent action."
- Route cards: each has `proof` + `done_when`.

**Implementation evidence:**

- `scripts/harness.py::classify_verification_status` (2013) — classifies a trace as `missing`/`partial`/`verified` from recorded verification entries + success markers.
- `scripts/harness.py::trace_finish` (1994) — appends `completion_claim` and an optional verification entry; **does NOT check whether proof passed before accepting the claim**.
- `.agent-harness/hooks/hook-router.sh::turn.stop` — prints "Compare the route card's proof ... If proof is missing, report the gap"; **emits text, does not scan or block**.

**Test evidence:** No test asserts that a completion claim is rejected when proof is absent.

**Runtime path:** Agent calls `trace finish <trace> --claim "done"` → `trace_finish` stamps claim unconditionally → `trace distill` later classifies status (which may be "missing"). The gate is classification-after-the-fact, not enforcement-before-the-claim.

**Missing links / gaps:**

- `missing runtime enforcement`: no code path rejects `trace_finish` (or blocks turn.stop) when verification is absent or failing.
- `architecture/code mismatch`: spec says controls "SHOULD NOT exist as advice"; the only turn.stop implementation is advisory text.
- `missing test coverage`: no negative test for "claim without proof is rejected".

**Reviewer notes:** This is the single highest-leverage gap. Decide whether completion-blocking is (a) delegated to the host agent's native hook system (then document that boundary explicitly, which `docs/specs/hook-strategy.md` partially does), or (b) implemented in `trace_finish`/a new gate. Either resolution closes the spec-vs-code mismatch.

---

### OUT-008 — EvalSpec execution (static)

**Outcome statement:** The system executes a declared EvalSpec (a list of checks) and emits a machine-readable EvalResult with per-check pass/fail and an aggregate status.

**Actor/User:** Operator / CI / fabrication layer.

**Settlement / success criterion:** `python scripts/harness.py eval run <spec> [--output <result>]` exits 0/1 and writes/prints an EvalResult JSON whose `status` reflects required checks.

**Status:** implemented (static check types only). **Confidence:** high.

**Implementation evidence:**

- `scripts/harness.py::run_eval_spec` (994) — validates required EvalSpec fields, runs each check, aggregates, writes JSON, exit 0/1.
- `scripts/harness.py::evaluate_check` (863) — supports `file_exists`, `static_required_patterns`, `static_forbidden_patterns`, `manual_check` (and likely `process_compliance` variants).

**Test evidence:** `tests/validate-harness.sh` runs `harness.py eval run` with a 3-check smoke EvalSpec and asserts `"status": "pass"`.

**Missing links / gaps:** `missing runtime enforcement` of dynamic behavior — check types are static (file existence, substring presence) or manual evidence; there is no check type that executes a command and parses its exit code/output as the assertion. Evals prove artifact shape, not runtime behavior.

---

### OUT-010 — Structured learning store

**Outcome statement:** The system mirrors distilled traces and learning reviews into a queryable local store so future agents can recover prior session state without replaying transcripts.

**Actor/User:** Operator / agent.

**Settlement / success criterion:** `sync-learning-store.sh` populates a sqlite DB; `query-learning-store.sh` returns summaries by status/job_type; the DB has non-empty `trace_records` and `learning_reviews`.

**Status:** implemented. **Confidence:** high.

**Declared evidence:** `docs/specs/memory-system.md` (DB explicitly optional/staged); `.agent-harness/config.yaml::observability`.

**Implementation evidence:**

- `.agent-harness/traces/learning-store.sqlite3` (520 KB) — tables `trace_records` (78 rows) and `learning_reviews` (78 rows); columns include `verification_status`, `latest_summary`, `unresolved_risks_json`, `candidate_memory_updates_json`, `provenance_json`.
- `scripts/{plan,sync,query,eval-learning-store}.sh` — plan/sync/query/eval-readiness scripts.

**Test evidence:** `validate-harness.sh` exercises all four learning-store scripts and asserts `learning_store_db`, `mode`, `results`, `vector_readiness`, `recommendation`.

**Missing links / gaps:** `missing persistence` for vector retrieval — `.agent-hooks/index/vectors/` is empty. This is by-design per `memory-system.md` (vector layer is a later phase adopted only when structured recall is insufficient), so it is a staged gap, not a defect.

---

### OUT-011 / OUT-012 / OUT-013 — Hook layer (advisory, not enforcing)

**Outcome statements:**

- OUT-011: Connect lifecycle events to routing/context/evidence/proof/learning.
- OUT-012: Block unauthorized/risky tool calls before execution.
- OUT-013: Block premature completion at turn end unless proof recorded.

**Actor/User:** Agent host (Claude Code / Copilot / Codex native hook runtime).

**Status:** OUT-011 partially_implemented; OUT-012 & OUT-013 declared_only. **Confidence:** high.

**Declared evidence:**

- `README.md:23` — "A tool call can trigger a safety check. ... A final response can trigger proof review."
- `README.md:56` — "check risky tool use before it happens ... block premature completion language at turn end."
- `HARNESS_SPEC.md::Anti-Delusion Gates` (407); `.agent-harness/config.yaml::hooks`.

**Implementation evidence:**

- `.agent-harness/hooks/hook-router.sh` — handles all 6 events (`session.start`, `prompt.submit`, `tool.pre`, `tool.post`, `turn.stop`, `session.end`); **`emit_or_capture` prints `purpose/action/boundary` text unless `--capture`**; `tool.pre` says "Check workspace, command intent ... destructive-operation risk" but performs no check; `turn.stop` says "Compare the route card's proof" but performs no comparison.
- `scripts/agent_hooks.py` — **observability layer only**: `command_capture`, `command_trace`, `command_inspect`, `command_replay`, `command_doctor`, `command_compact_logs`, `command_index_rebuild`, `command_export_otel`, `command_export_junit`. **No `PreToolUse`/`PostToolUse`/`UserPromptSubmit`/`Stop` handler; no `sys.exit(2)`/deny/approve logic anywhere in the file.**
- `docs/specs/hook-strategy.md` (line 16) — explicitly admits the binding contract is incomplete: "Another implementation must also define the binding contract ... which failures are advisory versus blocking."

**Test evidence:** `validate-harness.sh` exercises `hook-router.sh prompt.submit --capture` round-trip and all `agent-hooks` query/export commands. No test exercises a blocking/denial path.

**Missing links / gaps:**

- `missing runtime enforcement` (OUT-012, OUT-013): no code blocks tool calls or completion.
- `missing security boundary` (OUT-012): no denylist of risky commands; `security.secret_patterns` in config exist for redaction, not for blocking.
- `overclaimed maturity`: README's "block" language overstates advisory emit.
- `unclear ownership boundary`: which layer (host agent native hooks vs. hook-router vs. agent_hooks.py) is responsible for blocking is undefined.

**Reviewer notes:** Inspect whether a host agent (e.g. Claude Code) is configured elsewhere to invoke `hook-router.sh tool.pre` and act on its stdout as a block signal. Within this repo alone, nothing blocks.

---

### OUT-014 / OUT-015 — Evidence trail + exports

**Outcome statements:**

- OUT-014: Capture normalized payloads + stdout/stderr artifacts + JSONL logs + rusql index after governed events.
- OUT-015: Export to OpenTelemetry and JUnit.

**Status:** implemented. **Confidence:** high.

**Implementation evidence:**

- `scripts/agent_hooks.py::command_capture` (356) / `append_event` (255) / `write_json`/`write_text` — writes `.agent-hooks/payloads/<date>/*.{native,normalized,result}.json` and `.agent-hooks/artifacts/<date>/*.std{out,err}.txt`.
- `.agent-hooks/logs/events-*.jsonl` — date-rotated event logs (2026-05-25, -26, -06-11).
- `.agent-hooks/index/hooks.rusql` (266 KB) — `command_index_rebuild` (482) builds sqlite index.
- `command_export_otel` (546) / `command_export_junit` (584).
- `.agent-hooks/config.yaml::redaction` — key-substring + value-pattern redaction.

**Test evidence:** `validate-harness.sh` round-trips capture → trace → inspect → replay → doctor → compact → index rebuild → export otel/junit, all with assertions. Hundreds of real captured payloads exist.

---

### OUT-016 / OUT-017 — Fabrication layer

**Outcome statements:**

- OUT-016: Convert a bounded product seed into a fabrication packet (PRODUCT_SEED, JTBD, JOB_HYPOTHESIS, HYPOTHESIS, ADR, PRD, SDS, TDD, CONTEXT_PACK, AGENT_TASK, EVAL_CHECKLIST, EVAL_SPEC).
- OUT-017: Require fabrication proof before completion; gate adaptation/learning on proof pass.

**Status:** OUT-016 implemented; OUT-017 partially_implemented. **Confidence:** high / medium.

**Declared evidence:** `docs/fabrication-layer/README.md`; `docs/fabrication-layer/explanations/{baml-generation-model,proof-before-completion}.md`; `FABRICATOR_SPEC_v0.1.0.md`; `.fabricator/config.yaml`.

**Implementation evidence:**

- `scripts/fabricate.py` — `command_new/generate/validate/handoff/proof/reflect/status` (1039-1136).
- `render_template` (281) — **custom `{{placeholder}}` regex substitution** (`PLACEHOLDER_RE.sub`); templates use `.j2` extension but are NOT rendered by Jinja2 and generation does NOT call BAML/LLM despite the "BAML generation model" doc title.
- `static_proof_checks` (878) — REAL pattern assertions on generated `index.html` (checks for `<canvas`, `requestAnimationFrame`, `handleMovement`, `resolveCollisions`, `state.score += 10`, HUD ids, `restartGame`); BUT **descriptions and tokens are hardcoded to the "Focus Runner" game** (config `default_product_type: single_page_html5_game`).
- `write_proof_outputs` (897) — writes `EVAL_RESULT.json`, `PROOF_RECORD.md`, `REFLECTION.md`, `ADAPTATION_DECISION.yaml` with `status: candidate|blocked` gated on proof pass; reflection honestly notes "Risk: proof is static analysis and should be paired with manual play verification for release claims" (line 919).

**Test evidence:** `validate-harness.sh` runs the full `new→generate→validate→handoff→proof→reflect→status` pipeline and asserts each step's output. `.fabricator/runs/0001-focus-runner/` is a real completed run with a passing `EVAL_RESULT.json`.

**Missing links / gaps:**

- `overclaimed maturity`: "BAML generation model" doc implies LLM/BAML generation; actual renderer is deterministic template substitution. BAML contracts exist but drive type-checking/release gates, not generation.
- `architecture/code mismatch`: proof is static HTML scan tailored to one example; the layer is effectively a single-example reference, not a general product-to-prototype engine.
- `missing runtime enforcement`: nothing in `command_handoff` (1068) requires `proof` to have passed before handoff (the docs list proof as step 6, after handoff step 5, so handoff-before-proof is the documented order — but then "proof before completion" is weaker than the spec implies).

---

### OUT-018 — Strategy layer

**Outcome statement:** Convert a strategic question into an evidence-backed decision through brief → option → gaps → research prompts → tests → evidence → evaluation → decision, blocking commitment while critical evidence gaps remain.

**Actor/User:** Strategist.

**Status:** implemented. **Confidence:** high.

**Implementation evidence:**

- `.strategy/strategy.py` — all 11 subcommands (`new`, `capture`, `generate-options`, `identify-gaps`, `generate-research-prompts`, `ingest-research-report`, `design-tests`, `record-evidence`, `evaluate`, `decide`, `status`).
- Honest state: `.strategy/decisions/decision.agentic_dev_harness.md` records `Status: delegate_research`, `Reason: critical purchase behavior evidence remains unresolved.` — the layer refuses to commit while gaps are open.

**Test evidence:** `validate-harness.sh` runs the strategy pipeline in a temp dir (`new` → `generate-options` → `identify-gaps` → `generate-research-prompts` → `design-tests` → `decide` → `status`) and asserts each step + `"next_action": "review"`.

**Missing links / gaps:** `option_data` (line 293) contains pre-baked content for the `agentic_dev_harness` example; arbitrary new questions get scaffolded artifacts but the depth of generated option/evidence content depends on templates. This is honest scaffolding, not a defect.

---

### OUT-019 — Agentic capability loop (orphaned/test-only)

**Outcome statement:** Provide a typed, event-sourced agentic capability loop (WorkRequested → ContextRequired → ContextPacketCreated → AuthorityChecked → ProofCompleted → SettlementRecorded …) with cross-language contracts (Python TypedDict / TypeScript interface / Rust struct) generated from a canonical SEA manifest, with cross-repo hash consistency.

**Actor/User:** Platform integrator (cross-repo: SWE_SEED + Context Kernel + SEA-Forge + GodSpeed-Agent + DomainForge).

**Status:** test_only. **Confidence:** medium.

**Declared evidence:** NOT in `README.md`, `AGENTS.md`, `HARNESS_SPEC.md`, or `SWE_SEED_SPEC_v0.2.0.md` (only "godspeed" appears in README as the git org URL). Self-documented in `.agent-harness/reports/godspeed_stack_agentic_loop_wiring_report.md` (2026-06-10).

**Implementation evidence:**

- `agentic_capability_loop/adapters.py` — `emit_work_requested`, `emit_context_required`, `emit_route_selected`, `emit_proof_started`, `emit_proof_completed`, `consume_context_packet_created`, `consume_authority_checked`, `consume_settlement_recorded`. NOT imported by `harness.py`, `fabricate.py`, `agent_hooks.py`, or `strategy.py` (orphan from the main runtime).
- `tests/{test_agentic_capability_loop,test_agentic_capability_contracts,test_agentic_capability_hardening,test_feedback_loop,test_cross_repo_hash_consistency}.py` — real assertions on event shapes, contract materialization (Py/TS/Rust), SEA regeneration equivalence, authority deny + coherence-break, live proof event, ledger persistence, hash consistency.

**Test evidence:** `test_agentic_capability_hardening.py::test_live_swe_seed_harness_validate_emits_proof_completed_shape` (marked `@pytest.mark.live_proof`) runs `harness.py validate` and asserts a `ProofCompleted` event shape.

**Missing links / gaps:**

- `missing integration wiring`: adapters not imported by any main runtime module.
- `missing test coverage` (in CI): **`pytest` is NOT invoked by `just ci`, `ci.sh`, `validate-harness.sh`, `package.json`, or `.github/workflows/ci.yml`.** The 5 pytest files are real but ungated.
- `duplicated/conflicting implementation`: per the wiring report, `domain_model_hash` is a naive SHA-256 of the literal string `"agentic_capability_loop"` (adapters.py line ~89 fallback), not a hash of the `.sea` content; downstream Rust `ContextAgent` trait is defined but unimplemented and tests use Python stubs.

**Reviewer notes:** Decide whether this loop is in-scope for SWE_SEED. If yes: wire adapters into a harness command, add `pytest` to `just ci`, replace the placeholder hash with a real content hash, and document it in README/AGENTS. If no: move it out of this repo to avoid the impression of an integrated capability.

---

### OUT-020 / OUT-021 / OUT-022 — Dev harness, secrets, BAML

- **OUT-020 (local CI = proof, mirrors remote):** implemented, high. `scripts/ci.sh` (doctor + prettier + ruff + harness validate + validate-harness.sh); `.github/workflows/ci.yml` runs the identical `just ci`. Local-remote parity is real.
- **OUT-021 (secrets without leakage):** implemented, medium. `scripts/secrets-{encrypt,decrypt,edit,rotate-key}.sh` + `.sops.yaml` + `.gitignore` (`*.dec.*`, `*.plain.*`, `sops`, `.secrets/`); `secrets-encrypt.sh` guards against the placeholder age key. Redaction patterns configured in `.agent-harness/config.yaml::security` and `.agent-hooks/config.yaml::redaction`.
- **OUT-022 (BAML typed contracts + regeneration):** implemented, high. `.agent-harness/baml/baml_src/{swe_seed,harness,fabricator}.baml`; `validate-harness.sh` runs `uv run baml-cli generate` + ~25 structural greps (`class ProductHypothesis`, `enum EARSPattern`, `class JobStory`, `function GenerateEARSRequirements`, etc.); `.agent-harness/baml/V0_1_RELEASE_CRITERIA.md` lists release gates. BAML drives type contracts + release gates (NOT fabrication generation — see OUT-016 gap).

## 5. Cross-Cutting Architecture Map

- **Entrypoints:** `just` recipes (`justfile`) → `scripts/{ci,bootstrap,doctor}.sh`, `scripts/harness.py`, `scripts/fabricate.py`, `.strategy/strategy.py`, `scripts/agent-hooks` → `scripts/agent_hooks.py`, `.agent-harness/hooks/hook-router.sh`. CI: `.github/workflows/ci.yml` → `just ci`.
- **Core domain (harness):** `scripts/harness.py` (route/validate/render-skills/context-plan/trace/eval/inspect). `.agent-harness/{routes,skills,playbooks,memory,context,reflections,evals,traces}/`.
- **Policy/governance layer:** `HARNESS_SPEC.md` (normative MUST/SHOULD), `AGENTS.md` (operating contract), `.agent-harness/config.yaml` (validation rules, secret patterns, behavior-shaping phrases).
- **Persistence/ledger:** `.agent-harness/traces/route-decisions/*.json`, `.agent-harness/traces/records/`, `.agent-harness/traces/learning-store.sqlite3`.
- **Evidence/observability:** `.agent-hooks/{payloads,artifacts,logs,index/hooks.rusql}`; `agent_hooks.py` capture/trace/inspect/replay/export.
- **Schemas/contracts:** `.fabricator/schemas/*.yaml`, `.strategy/schemas/*.yaml`, `.agent-harness/baml/baml_src/*.baml`, Skill IR JSON.
- **Adapters/integrations:** `agentic_capability_loop/adapters.py` (event emit/consume — not wired to harness CLI); cross-repo GodSpeed stack (documented, partially stubbed).
- **Tests/proof paths:** `tests/validate-harness.sh` (exhaustive integration, in CI); `tests/test_*.py` (pytest, NOT in CI).
- **Build/deployment:** `just ci` local; GitHub Actions remote; no Docker/release pipeline beyond `release` route card.

## 6. Contradictions and Overclaims

| Claim / Outcome                                                                                                                         | Evidence for Claim                             | Evidence Against / Missing                                                                                                        | Risk                                                                   |
| --------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| "block premature completion language at turn end" (README:23,56) + "Anti-Delusion Gates MUST reject, block, or flag" (HARNESS_SPEC:407) | Spec + README mandate; `turn.stop` hook exists | `hook-router.sh turn.stop` only prints guidance; `agent_hooks.py` has no Stop handler; `trace_finish` stamps any claim            | High — core value prop ("completion requires proof") is advisory       |
| "check risky tool use before it happens" / "blocks unauthorized file writes" (README:23)                                                | README + `tool.pre` hook                       | `tool.pre` emits text; no denylist; no `exit(2)`; `agent_hooks.py` has no PreToolUse handler                                      | High — security boundary claimed but unenforced                        |
| "controls MUST be attached to executable behavior ... SHOULD NOT exist as advice" (HARNESS_SPEC:364)                                    | Normative spec                                 | The only hook implementation is advice (emit)                                                                                     | High — spec self-contradicted by implementation                        |
| "BAML generation model" (docs/fabrication-layer/explanations/baml-generation-model.md)                                                  | Doc title + `baml-py` dependency               | `fabricate.py::render_template` is `{{placeholder}}` regex sub; no Jinja2/BAML/LLM call in generation                             | Medium — generation mechanism overstated                               |
| Fabrication = general "product-to-prototype workflow" (docs/fabrication-layer/README.md)                                                | Full 7-step pipeline runs end-to-end           | `static_proof_checks` hardcoded to "Focus Runner" tokens; config `default_product_type: single_page_html5_game`                   | Medium — single-example reference sold as general engine               |
| `learning.auto_apply: false` + "improves without becoming reckless" (README:58)                                                         | Config + `reflections/` proposals              | Consistent — proposals ledger exists, no auto-apply code                                                                          | Low — honest                                                           |
| Vector retrieval (`.agent-hooks/index/vectors/`)                                                                                        | Config `vector_index_root`                     | Dir empty                                                                                                                         | Low — explicitly staged as future in `memory-system.md`                |
| Agentic capability loop as integrated capability                                                                                        | 5 pytest files + adapters + contracts          | Not in README/AGENTS/specs; not imported by runtime; not in CI; naive placeholder hash; stubbed Rust services (per wiring report) | Medium — impression of integration where there is parallel/orphan code |

## 7. Missing Proofs

| Outcome ID | Missing Proof                           | Suggested Minimal Test                                                                                                                        | Why It Matters                                  |
| ---------- | --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| OUT-007    | Completion claim accepted without proof | Negative test: `trace finish` (or the completion gate) on a trace with no verification entry → expect rejection/non-zero, not a stamped claim | Core anti-completion-theater value              |
| OUT-012    | No risky-tool-call block                | Test: invoke `tool.pre` with a destructive command (`rm -rf`) → expect deny/non-zero (currently nothing denies)                               | Claimed security boundary                       |
| OUT-013    | No premature-completion block           | Test: `turn.stop` after a "done" claim with no proof → expect block/flag                                                                      | Claimed turn-end gate                           |
| OUT-008    | No dynamic eval check type              | Add a `command_exit_code` check type that runs a command and asserts exit 0; test it                                                          | Evals currently prove shape, not behavior       |
| OUT-017    | Fabrication proof generality            | Test: `fabricate proof` on a seed whose prototype is NOT the Focus Runner → expect non-hardcoded checks                                       | Proof currently tied to one example             |
| OUT-019    | pytest not in CI                        | Add `uv run pytest` to `just ci` (or document why it is excluded)                                                                             | 5 real test files currently ungated             |
| OUT-016    | Generation mechanism                    | Test/doc clarifying that fabrication generation is deterministic template substitution (not BAML/LLM)                                         | Corrects the "BAML generation model" impression |

## 8. Frontier Model Audit Brief

### Most Important Outcomes

1. OUT-007 — proof-before-completion gate (partial; the central value proposition).
2. OUT-012 — block risky tool calls (declared only).
3. OUT-013 — block premature completion (declared only).
4. OUT-001 — semantic routing (implemented; the entrypoint).
5. OUT-002/OUT-003 — structural validation + anti-stub gate (implemented).
6. OUT-014/OUT-015 — evidence trail + exports (implemented).
7. OUT-016/OUT-017 — fabrication packet + proof (implemented / partial-single-example).
8. OUT-019 — agentic capability loop (test-only, orphaned, ungated).
9. OUT-008 — eval executor (static only).
10. OUT-022 — BAML typed contracts (implemented).

### Highest-Risk Gaps

1. Completion-gate is classification-after-the-fact, not enforcement (`trace_finish` + advisory `turn.stop`).
2. "Block" semantics in README/spec have zero enforcing code.
3. pytest suite (incl. capability-loop contracts) is outside `just ci`.
4. Fabrication proof is static and hardcoded to one game.
5. Spec self-contradiction: "controls SHOULD NOT exist as advice" while the only hook implementation is advice.

### First Files to Inspect

1. `scripts/harness.py` — `trace_finish` (1994), `classify_verification_status` (2013), `route` (1644), `validate` (1167), `run_eval_spec` (994).
2. `.agent-harness/hooks/hook-router.sh` — the `emit_or_capture` / `turn.stop` / `tool.pre` branches (advisory).
3. `scripts/agent_hooks.py` — confirm absence of any blocking handler (capture/query/export only).
4. `HARNESS_SPEC.md` — sections 3.7 (Affordance Control), 3.10 (Anti-Delusion Gates), Enterprise Completion Policy (line 36).
5. `scripts/fabricate.py` — `render_template` (281), `static_proof_checks` (878), `command_handoff` (1068).
6. `tests/validate-harness.sh` — what is actually gated vs. the pytest files that are not.
7. `agentic_capability_loop/adapters.py` + `.agent-harness/reports/godspeed_stack_agentic_loop_wiring_report.md`.

### Open Architectural Questions

1. Is completion-blocking intended to live in the host agent's native hook runtime (then the SWE_SEED repo only advises), or in SWE_SEED code? The boundary is undefined.
2. Should `pytest` be added to `just ci`, or are the capability-loop tests out-of-scope for this repo?
3. Is the fabrication layer a general engine or a single-example reference? The docs imply general; the proof code is specific.
4. Should BAML contracts drive fabrication generation (currently they drive only type/release gates)?
5. Who owns the "block" semantics — hook-router.sh, agent_hooks.py, or the host agent?

### Recommended Next Prompt

> Adversarially audit SWE_SEED's completion-gate and hook-enforcement claims. For each of these README/spec promises — "block unauthorized file writes before execution", "block premature completion language at turn end", "completion theater is a blocking defect", "controls SHOULD NOT exist as advice" — trace the exact code path that enforces it (file:line). Where no enforcing path exists, classify the gap (declared_only vs partially_implemented) and propose the smallest change that converts advisory emit into executable enforcement, including which layer should own it (hook-router.sh, agent_hooks.py, trace_finish, or the host agent native runtime). Do not modify code; produce a gap-closure plan with a falsifiable test for each converted gate. Use `.agents/reports/outcome_code_map_SWE_SEED_2026-06-14.md` as the evidence base.
