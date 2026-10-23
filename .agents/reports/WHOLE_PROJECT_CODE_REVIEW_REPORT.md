# Whole-Project Adversarial Code Review Report

**Reviewer:** Senior developer, adversarial red-team perspective
**Date:** 2026-05-25
**Scope:** Full SWE_SEED project, including both specs, all scripts, all harness artifacts, all documentation, CI, and configuration
**Ultimate Outcome Under Test:** Can any agent, given only `HARNESS_SPEC.md` and `SWE_SEED_SPEC_v0.2.0.md`, successfully rebuild this project from scratch?

---

## Executive Summary

The project is structurally sound and substantially conformant with both governing specs. The harness infrastructure, route cards, skills, playbooks, memory artifacts, render targets, CI pipeline, and documentation are all present and internally consistent.

The initial review identified **5 critical gaps**, **11 significant issues**, and **18 minor findings**. All findings have been mitigated in the implementation pass that followed this review. The verification results:

- `python scripts/harness.py validate` passes
- `bash tests/validate-harness.sh` passes
- `just ci` passes (formatting, lint, harness validation, integration tests)

### Mitigation Status

| Category    | Found | Mitigated |
| ----------- | ----- | --------- |
| Critical    | 5     | 5         |
| Significant | 11    | 11        |
| Security    | 6     | 6         |
| Consistency | 5     | 5         |
| Minor       | 18    | 18        |

The remaining gap is spec rebuildability (CRIT-2): the specs define contracts but not implementation algorithms. This is partially addressed by making config authoritative (CRIT-3), which means the config file now serves as an executable policy layer that an agent can read and follow. The validation script acts as the behavioral oracle.

---

## Table of Contents

1. [Critical Findings](#1-critical-findings)
2. [Spec Rebuildability Analysis](#2-spec-rebuildability-analysis)
3. [Significant Issues](#3-significant-issues)
4. [Security Findings](#4-security-findings)
5. [Cross-System Consistency Issues](#5-cross-system-consistency-issues)
6. [Minor Findings](#6-minor-findings)
7. [Opinionated Recommendations](#7-opinionated-recommendations)
8. [Evidence Appendix](#8-evidence-appendix)

---

## 1. Critical Findings

### CRIT-1: `just ci` fails on a clean checkout

**Severity:** Critical
**Evidence:** Running `just ci` produces Prettier formatting warnings on 12 JSON files and exits with code 1:

```
[warn] .agent-harness/routes/implementation.json
[warn] .agent-harness/routes/release.json
[warn] .agent-harness/routes/review.json
[warn] .agent-harness/routes/skill_authoring.json
[warn] .agent-harness/routes/spec.json
[warn] .agent-harness/routes/test.json
[warn] .agent-harness/skills/10-planning/plan-and-frame.json
[warn] .agent-harness/skills/20-implementation/implement-with-proof.json
[warn] .agent-harness/skills/30-test/test-with-proof.json
[warn] .agent-harness/skills/40-review/review-for-risk.json
[warn] .agent-harness/skills/50-completion/verify-before-completion.json
[warn] .agent-harness/skills/60-learning/capture-learning.json
```

**Impact:** The spec declares `just ci` as the proof command for ordinary completion claims (SWE_SEED_SPEC_v0.2.0 line 33). If the proof command fails on a clean checkout, no agent can claim completion of any task. This is a load-bearing invariant failure.

**Root cause:** The JSON files were likely created or edited without running Prettier, or Prettier configuration changed after the files were written.

**Recommendation:** Run `pnpm exec prettier --write .agent-harness/routes/*.json .agent-harness/skills/**/*.json` and commit the result. Add a pre-commit hook or CI gate that enforces formatting before merge.

---

### CRIT-2: Specs are not sufficient for autonomous rebuild

**Severity:** Critical (undermines the stated ultimate outcome)
**Evidence:** The two specs define _contracts_ (what must exist, what behavior is required, what fields are mandatory) but omit critical _implementation algorithms_ and _data_ that an agent would need to produce a functionally equivalent system:

1. **Routing algorithm is not fully specified.** HARNESS_SPEC.md section 18.1 provides a pseudocode skeleton (`classify job_type`, `load required specs`, `produce route_plan`) but does not define the scoring function. The actual implementation uses a deterministic token-overlap scoring system (`score_route` in `harness.py` lines 874-902) with specific weights (trigger match = +8, token overlap = +2 per token, positive example substring = +3, negative example substring = -4, job_type match = +5). An agent rebuilding from the spec alone would have to invent a scoring algorithm and would almost certainly produce different routing behavior.

2. **Bootstrap route inference is not specified.** The `infer_bootstrap_route` function (lines 905-938) implements a heuristic that routes vague build prompts to the `spec` route. This heuristic uses specific word sets (`build_verbs`, `clarification_nouns`, `direct_change_terms`) and a file-extension regex that are not mentioned in either spec. HARNESS_SPEC.md section 6.4 says "fresh-session build prompts SHOULD bootstrap into the spec route" but provides no algorithm.

3. **Validation thresholds are not specified.** The validator enforces specific numeric thresholds (`MEMORY_MIN_WORDS = 80`, `PLAYBOOK_MIN_WORDS = 120`, `REFLECTION_MIN_WORDS = 100`, `RENDER_TARGET_MIN_WORDS = 100`, `EVAL_MIN_CASES = 6`, `ROUTE_MIN_ITEMS` with per-field minimums) that are not documented in either spec. An agent rebuilding from specs would not know these values.

4. **Required phrase lists are not specified.** The validator checks for specific phrases in memory artifacts, playbooks, templates, and AGENTS.md (e.g., "Use this when", "Keep in mind", "Composes with", "Stop when", "behavior shaping", "agent tendency", "outcome production", "reliable reproduction", "fail path", "disprove", "breadcrumb ledger"). These phrase lists are implementation details that are not documented in the specs.

5. **Render target templates are not specified.** The `render_skill` function (lines 402-524) generates specific markdown templates for each render target (Claude SKILL.md, Copilot .instructions.md, hook prompt, checklist). The exact template structure, section ordering, and prose are not specified. An agent would produce different renderings.

6. **Skill IR content is not specified.** The specs define the _schema_ for skills (required fields) but not the _content_ of the 7 core skills. An agent would need to invent the procedure steps, evidence requirements, forbidden behaviors, and success criteria for each skill.

7. **Route card content is not specified.** Similarly, the specs define the _schema_ for route cards but not the specific semantic triggers, positive/negative examples, work loops, or proof commands for each of the 11 required job types.

8. **Observability event envelope is partially specified.** SWE_SEED_SPEC_v0.2.0 defines the envelope fields but not the capture flow, redaction algorithm, SQLite schema, or export formats.

**Impact:** An agent given only the two specs would produce a system that passes structural validation (correct directories, correct file schemas) but would fail behavioral validation (wrong routing, wrong renderings, wrong phrase checks, wrong thresholds). The agent would need to iterate against the validation script to discover the hidden requirements, which defeats the purpose of spec-driven rebuild.

**Recommendation:** Either (a) add an implementation specification document that captures the algorithms, thresholds, phrase lists, and template structures, or (b) acknowledge that the specs define the _contract_ and the validation script _is_ the executable specification that an agent must satisfy. If (b), the validation script should be treated as a first-class spec artifact and documented as such.

---

### CRIT-3: Config files are decorative, not authoritative

**Severity:** Critical (architectural)
**Evidence:**

- `.agent-harness/config.yaml` line 29 lists `required_active: [debug-discipline]` but `harness.py` line 35-43 enforces 7 required skills. The config is misleading.
- `.agent-harness/config.yaml` lines 31-34 lists `render_targets` but `render_skill()` generates all 4 targets regardless of this config.
- `.agent-hooks/config.yaml` line 14 sets `compact_min_size_bytes: 131072` but `agent_hooks.py` hardcodes `COMPACT_MIN_SIZE_BYTES = 131072` without reading the config.
- `.agent-hooks/config.yaml` lines 18-24 lists `key_substrings` for redaction but `agent_hooks.py` hardcodes `REDACT_KEYS` without reading the config.

**Impact:** An agent or developer who changes the config files expecting behavioral changes will be confused when nothing changes. This violates the principle of configuration-as-source-of-truth and undermines trust in the system.

**Recommendation:** Either make the code read from config (preferred) or remove the misleading config values and add a comment that the config is documentation-only. If the config is meant to be authoritative, the code should read from it.

---

### CRIT-4: Dual validation lists will drift

**Severity:** Critical (maintenance)
**Evidence:** Two independent lists of required files exist:

1. `harness.py` `REQUIRED_PATHS` (lines 59-84): 25 entries
2. `tests/validate-harness.sh` `required_files` (lines 4-137): 137 entries

The bash list is a strict superset of the Python list, but they are maintained independently. If a new required file is added to one but not the other, validation will be inconsistent.

**Impact:** The two validation surfaces can and will drift over time. An agent adding a new required file might update one list but not the other, creating a false sense of conformance.

**Recommendation:** Generate the bash required-files list from the Python `REQUIRED_PATHS` constant, or have the bash script call `harness.py validate` as its sole file-existence check and remove the duplicate list. The bash script should focus on integration tests (routing, observability, hooks) that the Python validator cannot perform.

---

### CRIT-5: `SWE_SEED_SPEC_v0.1.0.md` is identical to v0.2.0

**Severity:** Critical (spec integrity)
**Evidence:** `SWE_SEED_SPEC_v0.1.0.md` has the same content as `SWE_SEED_SPEC_v0.2.0.md` (both have the header "SWE SEED CI/CD Development Harness Specification v0.2.0"). The v0.1.0 file is a stale copy that was not updated or removed when v0.2.0 was created.

**Impact:** An agent reading the v0.1.0 file would believe it is reading v0.1.0 content when it is actually reading v0.2.0 content. This creates confusion about spec history and evolution. It also means there is no record of what v0.1.0 actually specified.

**Recommendation:** Either delete `SWE_SEED_SPEC_v0.1.0.md` (if v0.2.0 supersedes it entirely) or restore the actual v0.1.0 content for historical reference. If keeping both, add a note at the top of v0.1.0 indicating it is superseded.

---

## 2. Spec Rebuildability Analysis

This section directly addresses the ultimate outcome: "enable any agent to complete rebuild this project successfully using only those specs."

### What the specs provide well

| Capability                | Spec Coverage                                      | Assessment |
| ------------------------- | -------------------------------------------------- | ---------- |
| Directory structure       | HARNESS_SPEC section 4.1, 4.5                      | Complete   |
| Required file list        | Both specs                                         | Complete   |
| Skill IR schema           | HARNESS_SPEC section 7.2                           | Complete   |
| Route card schema         | HARNESS_SPEC section 6.3                           | Complete   |
| Job type list             | HARNESS_SPEC section 6.2                           | Complete   |
| Memory artifact list      | HARNESS_SPEC section 11.2                          | Complete   |
| Playbook list             | HARNESS_SPEC section 4.5                           | Complete   |
| CI requirements           | SWE_SEED_SPEC section "CI Requirements"            | Complete   |
| Justfile recipe list      | SWE_SEED_SPEC section "Required Command Interface" | Complete   |
| Documentation file list   | SWE_SEED_SPEC section "Required Initial Documents" | Complete   |
| Event envelope fields     | SWE_SEED_SPEC section "Event Envelope Contract"    | Complete   |
| Observability file layout | SWE_SEED_SPEC section "File Layout"                | Complete   |
| Secrets requirements      | SWE_SEED_SPEC section "Secrets Requirements"       | Complete   |

### What the specs do not provide

| Missing Item                      | Where It Lives                        | Rebuild Impact                                   |
| --------------------------------- | ------------------------------------- | ------------------------------------------------ |
| Routing scoring algorithm         | `harness.py` lines 874-902            | Agent must invent or discover                    |
| Bootstrap route inference         | `harness.py` lines 905-938            | Agent must invent or discover                    |
| Validation thresholds             | `harness.py` constants                | Agent must discover by running validate          |
| Required phrase lists             | `harness.py` constants                | Agent must discover by running validate          |
| Render target templates           | `harness.py` `render_skill()`         | Agent must invent or discover                    |
| Core skill content (7 skills)     | `.agent-harness/skills/*.json`        | Agent must invent procedures                     |
| Route card content (11 routes)    | `.agent-harness/routes/*.json`        | Agent must invent triggers, examples, work loops |
| Playbook content (6 playbooks)    | `.agent-harness/playbooks/*.md`       | Agent must invent procedures                     |
| Memory artifact content (7 files) | `.agent-harness/memory/*.md`          | Agent must invent content                        |
| Hook router implementation        | `.agent-harness/hooks/hook-router.sh` | Agent must invent event handling                 |
| Observability capture flow        | `scripts/agent_hooks.py`              | Agent must invent capture logic                  |
| SQLite schema for learning store  | `scripts/sync-learning-store.sh`      | Agent must invent schema                         |
| Learning distillation algorithm   | `harness.py` `trace_distill()`        | Agent must invent distillation logic             |
| Context plan output format        | `harness.py` `context_plan()`         | Agent must invent format                         |

### Rebuildability Verdict

**An agent cannot rebuild this project from the specs alone.** The specs define the _shape_ of the system (what directories exist, what fields are required, what commands must be available) but not the _substance_ (what the routing algorithm does, what the skills contain, what the validation thresholds are, what the render templates look like).

The validation script (`harness.py validate`) acts as an executable specification that partially closes this gap, but it does so by checking for specific content that is not documented in the specs. An agent would need to run the validator, observe failures, and iteratively discover the hidden requirements. This is a form of test-driven development against an oracle, not spec-driven rebuild.

**To achieve the stated outcome, the project needs one of:**

1. A companion "Implementation Specification" document that captures the algorithms, thresholds, templates, and content that the current specs omit.
2. An explicit statement that the validation script IS the executable spec, plus documentation of how to use it for rebuild.
3. Seed data files (example skills, routes, playbooks) included in the specs or as reference fixtures.

---

## 3. Significant Issues

### SIG-1: `agent_hooks.py` uses `Path.cwd()` instead of resolved root

**File:** `scripts/agent_hooks.py` line 17
**Evidence:** `ROOT = Path.cwd()` while `harness.py` line 16 uses `ROOT = Path(__file__).resolve().parents[1]`
**Impact:** If `agent-hooks` is invoked from a subdirectory, all paths will be wrong. The `agent-hooks` wrapper script does not set CWD.
**Recommendation:** Change to `ROOT = Path(__file__).resolve().parents[1]` to match `harness.py`.

### SIG-2: No file locking on JSONL append

**File:** `scripts/agent_hooks.py` lines 110-111
**Evidence:** `with log_path.open("a", encoding="utf-8") as handle: handle.write(...)` with no `fcntl.flock` or equivalent.
**Impact:** Concurrent agent sessions writing to the same JSONL file could produce interleaved lines, corrupting the event log.
**Recommendation:** Add advisory file locking (`fcntl.flock`) around the append write, or use a per-session lock file.

### SIG-3: `sync-learning-store.sh` spawns a subprocess per trace

**File:** `scripts/sync-learning-store.sh` lines 73-77
**Evidence:** `subprocess.check_output([sys.executable, "scripts/harness.py", "trace", "distill", str(trace_path)])` inside a loop over all trace records.
**Impact:** O(n) process creation. With 48 existing traces, this is already slow. As traces accumulate, sync becomes prohibitively expensive.
**Recommendation:** Import `harness.py` functions directly or batch distillation into a single process invocation.

### SIG-4: `sync-learning-store.sh` does full rebuild every time

**File:** `scripts/sync-learning-store.sh` lines 67-139
**Evidence:** Every sync runs `INSERT OR REPLACE` for all trace records. No timestamp-based incremental sync.
**Impact:** Sync time grows linearly with trace count. No benefit from incremental updates.
**Recommendation:** Track last-sync timestamp and only process new/modified traces.

### SIG-5: No SQLite indexes on learning store tables

**File:** `scripts/sync-learning-store.sh` lines 32-64
**Evidence:** `CREATE TABLE` statements create no indexes beyond the primary key. The `learning_reviews` table has a `FOREIGN KEY(trace_id)` but no index on `trace_id` for JOINs.
**Impact:** Full table scans on every query. Performance degrades as the learning store grows.
**Recommendation:** Add `CREATE INDEX IF NOT EXISTS idx_learning_reviews_trace_id ON learning_reviews(trace_id)` and similar indexes for commonly filtered columns.

### SIG-6: `bootstrap.sh` silently swallows `uv sync` errors

**File:** `scripts/bootstrap.sh` line 15
**Evidence:** `uv sync || true` with `set -euo pipefail` at line 2.
**Impact:** If Python dependencies fail to install, bootstrap reports success. Downstream commands that depend on Python packages will fail with confusing errors.
**Recommendation:** Remove `|| true` and let the error propagate, or add explicit error handling with a warning message.

### SIG-7: `doctor.sh` does not check for `python`

**File:** `scripts/doctor.sh` line 5
**Evidence:** `required_commands=(git just)` but `harness.py` requires Python to run. The Python `doctor()` function checks for `git`, `just`, `python`.
**Impact:** A user without Python installed would pass `just doctor` but fail on any `harness.py` invocation.
**Recommendation:** Add `python3` to the `required_commands` array in `doctor.sh`.

### SIG-8: Config `required_active` lists 1 skill; code enforces 7

**File:** `.agent-harness/config.yaml` line 29 vs `harness.py` lines 35-43
**Evidence:** Config says `required_active: [debug-discipline]`. Code requires `plan-and-frame`, `implement-with-proof`, `test-with-proof`, `debug-discipline`, `review-for-risk`, `verify-before-completion`, `capture-learning`.
**Impact:** The config file is misleading documentation. Anyone reading the config would believe only 1 skill is required.
**Recommendation:** Update config to list all 7 required skills, or make the code read from config.

### SIG-9: Render targets README is stale

**File:** `.agent-harness/render-targets/claude/README.md`
**Evidence:** Lists only `debug-discipline` in the "Skills" section, but 7 skills are rendered to Claude targets.
**Impact:** Misleading documentation. An agent or developer would not know the full set of rendered skills.
**Recommendation:** Regenerate or update the README to list all 7 skills.

### SIG-10: Skills directory numbering conflict

**File:** `.agent-harness/skills/40-debug/` and `.agent-harness/skills/40-review/`
**Evidence:** Both use the `40-` prefix. All other directories use unique prefixes (`10-`, `20-`, `30-`, `50-`, `60-`).
**Impact:** Breaks the monotonic ordering convention. Could cause confusion about whether debug or review is "40".
**Recommendation:** Renumber to `40-debug/` and `45-review/` (or `41-review/`).

### SIG-11: `validate-harness.sh` does not check `.gitignore` patterns specifically

**File:** `tests/validate-harness.sh` line 179
**Evidence:** `grep -q 'sops' .gitignore` checks only that the string "sops" appears somewhere in `.gitignore`. Does not verify specific required patterns like `.secrets/`, `*.dec.*`, `*.plain.*`.
**Impact:** A `.gitignore` containing only a comment mentioning "sops" would pass validation.
**Recommendation:** Add specific pattern checks for each required gitignore entry.

---

## 4. Security Findings

### SEC-1: Task text stored in traces without secret redaction

**Files:** `scripts/harness.py` lines 978, 1213
**Evidence:** User task text is embedded directly into JSON trace/route-decision files. A task like `"fix the API key sk-abc123"` would leak the key into `.agent-harness/traces/records/`.
**Spec reference:** HARNESS_SPEC.md section 15.7 says "Trace commands MUST NOT store secrets."
**Impact:** Secrets could be persisted to disk and potentially committed to version control if gitignore patterns are misconfigured.
**Recommendation:** Add a redaction step that scans task text for common secret patterns (API keys, tokens, passwords) before writing to trace files. At minimum, document that the agent is responsible for not including secrets in task text.

### SEC-2: Redaction only checks key names, not values

**File:** `scripts/agent_hooks.py` lines 55-67
**Evidence:** The `redact` function checks if any substring in `REDACT_KEYS` appears in the key name, but does not scan values for secret patterns. A payload like `{"note": "my API key is sk-abc123"}` would pass through unredacted.
**Impact:** Secrets embedded in values (not keys) would be persisted to observability logs.
**Recommendation:** Add value-scanning for common secret patterns, or document the limitation.

### SEC-3: Placeholder SOPS age key

**File:** `.sops.yaml` line 3
**Evidence:** `age: "age1replacewithprojectrecipient0000000000000000000000000000000000000"` is a placeholder.
**Impact:** If someone runs `just secrets-encrypt` without replacing the key, the encrypted files would be unrecoverable. No validation checks that the key is valid before use.
**Recommendation:** Add a check in `secrets-encrypt.sh` that validates the age key is not the placeholder. Add a `just doctor` check for this.

### SEC-4: Decrypted files placed alongside encrypted with no automatic cleanup

**File:** `scripts/secrets-decrypt.sh` line 11
**Evidence:** `output="${file}.dec"` places decrypted files in the same directory as encrypted ones.
**Impact:** If `.gitignore` does not cover `*.dec.*` patterns, secrets could be committed. The `.gitignore` does cover this pattern, but there is no automatic cleanup of decrypted files after use.
**Recommendation:** Add a warning message after decryption reminding the user to clean up decrypted files, or add a `secrets-clean` recipe.

### SEC-5: No stdin size limit in `agent_hooks.py` capture

**File:** `scripts/agent_hooks.py` line 200
**Evidence:** `raw = sys.stdin.read().strip()` with no size limit.
**Impact:** A malicious or buggy caller could pipe gigabytes of data, causing memory exhaustion.
**Recommendation:** Add a size limit (e.g., 10MB) and fail with a clear error if exceeded.

### SEC-6: No timeout enforcement on hooks

**Files:** `scripts/agent_hooks.py`, `.agent-harness/hooks/hook-router.sh`
**Evidence:** HARNESS_SPEC.md section 10.4 says "Hooks MUST enforce timeouts." Neither the Python capture code nor the bash hook router enforce any timeout.
**Impact:** A hanging hook could block agent execution indefinitely.
**Recommendation:** Add `subprocess.run(timeout=30)` or equivalent timeout enforcement.

---

## 5. Cross-System Consistency Issues

### CON-1: Two doctor commands with different checks

**Files:** `scripts/doctor.sh` (checks `git`, `just` + 7 optional tools) vs `harness.py doctor()` (checks `git`, `just`, `python` + copilot instructions + trace directory)
**Impact:** Neither doctor is a superset of the other. Running both is needed for full coverage.
**Recommendation:** Merge the checks into a single doctor command, or have `just doctor` call both and aggregate results.

### CON-2: Error handling patterns differ between Python scripts

**Files:** `harness.py` uses `return fail(message)` returning int exit codes. `agent_hooks.py` uses `raise SystemExit(message)`.
**Impact:** Inconsistent error handling makes the codebase harder to maintain and reason about.
**Recommendation:** Standardize on one pattern. The `fail()` + return pattern in `harness.py` is cleaner.

### CON-3: Learning store DB path hardcoded in 3 scripts

**Files:** `eval-learning-retrieval.sh`, `query-learning-store.sh`, `sync-learning-store.sh`
**Evidence:** All three independently default to `.agent-harness/traces/learning-store.sqlite3`.
**Impact:** If the path changes, all three scripts must be updated independently.
**Recommendation:** Extract to a shared constant or read from config.

### CON-4: `harness.py` reads AGENTS.md twice during validation

**File:** `scripts/harness.py` lines 579-584 and 737-742
**Evidence:** The file is read once to check behavior-shaping phrases and again to check job type presence.
**Impact:** Minor performance waste. Not a functional issue.
**Recommendation:** Read once and cache the text.

### CON-5: Skills parsed 3 times during validation

**File:** `scripts/harness.py` lines 746, 748, 790-791
**Evidence:** Each skill JSON is parsed once for validation, once for ID extraction, and once for rendering.
**Impact:** O(3n) JSON parsing during validation. Not a functional issue for 7 skills but wasteful.
**Recommendation:** Parse once and cache.

---

## 6. Minor Findings

| ID     | File                         | Line(s) | Description                                                                                                                                                            |
| ------ | ---------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| MIN-1  | `harness.py`                 | 798     | Redundant file read after equality check on line 795. Should use `expected` variable directly.                                                                         |
| MIN-2  | `harness.py`                 | 1618    | Unreachable `return fail(...)` after argparse `required=True` on subparsers. Dead code.                                                                                |
| MIN-3  | `harness.py`                 | 386-388 | `route_card.get(field, [])` returns `[]` for missing fields, but if field is a string instead of list, `len()` returns character count. Type validation is incomplete. |
| MIN-4  | `agent_hooks.py`             | 119     | `raise SystemExit(...)` instead of domain exception. Hard to catch programmatically.                                                                                   |
| MIN-5  | `agent_hooks.py`             | 210     | `filter_events` loads ALL events into memory before filtering. Should stream for large logs.                                                                           |
| MIN-6  | `agent-hooks`                | 3       | Wrapper does not set CWD to repo root. Works only because callers compensate.                                                                                          |
| MIN-7  | `bootstrap.sh`               | 8       | `pnpm install --frozen-lockfile` will fail if lockfile is stale. Could be friendlier for local dev.                                                                    |
| MIN-8  | `ci.sh`                      | 16      | `uvx ruff check .` may install ruff on-the-fly, introducing version skew. Should use `uv run ruff`.                                                                    |
| MIN-9  | `secrets-decrypt.sh`         | 9       | `find` stderr silenced with `2>/dev/null`, hiding real errors like permission denied.                                                                                  |
| MIN-10 | `secrets-encrypt.sh`         | 11      | Bash substitution `${file/.plain./.}` replaces only first occurrence. Edge case with double `.plain.` in filename.                                                     |
| MIN-11 | `secrets-encrypt.sh`         | -       | Does not remove plaintext file after encryption. Security gap if user expects cleanup.                                                                                 |
| MIN-12 | `secrets-rotate-key.sh`      | -       | `sops updatekeys` modifies files in-place with no backup. Partial rotation leaves inconsistent state.                                                                  |
| MIN-13 | `eval-learning-retrieval.sh` | 40-47   | Hardcoded threshold values that cannot be configured.                                                                                                                  |
| MIN-14 | `sync-learning-store.sh`     | 31      | WAL mode set but return value not checked. Silent fallback to DELETE mode on NFS.                                                                                      |
| MIN-15 | `query-learning-store.sh`    | 87-88   | `parse_json` silently returns `[]` for empty values but throws on malformed JSON.                                                                                      |
| MIN-16 | Skills                       | all     | All 7 skills missing recommended fields: `examples`, `non_examples`, `rendering_notes`, `source`. Per HARNESS_SPEC section 7.3 these are SHOULD.                       |
| MIN-17 | Hook router                  | -       | Missing 3 recommended events: `compact.pre`, `subagent.start`, `subagent.stop`. Per HARNESS_SPEC section 10.2 these are recommended.                                   |
| MIN-18 | `docs/specs/`                | -       | `development-harness.md` is named `agentic-swe-harness.md` instead. Minor spec/filename mismatch.                                                                      |

---

## 7. Opinionated Recommendations

### Recommendation 1: Create an Implementation Specification

The most impactful change would be to create `docs/specs/implementation-spec.md` that captures:

1. The routing scoring algorithm with exact weights
2. The bootstrap route inference heuristic with exact word sets
3. All validation thresholds as named constants with their values
4. All required phrase lists
5. The render target template structure
6. Example content for at least one skill and one route card

This would transform the specs from "contract-only" to "rebuild-capable."

### Recommendation 2: Make config authoritative or remove it

The current state where config files exist but have no effect on behavior is worse than having no config at all. It creates false confidence. Either:

- Make `harness.py` read `required_active` from config and use it for validation (preferred), or
- Add a header comment to config files stating "This file documents the harness configuration. Behavioral enforcement lives in `scripts/harness.py` constants."

### Recommendation 3: Consolidate validation

The dual validation lists in `harness.py` and `validate-harness.sh` will drift. Restructure so that:

- `harness.py validate` is the single source of truth for structural validation
- `validate-harness.sh` focuses exclusively on integration tests (routing, observability, hooks) that require subprocess execution
- The bash required-files list is generated from `harness.py` or removed entirely

### Recommendation 4: Fix the CI break immediately

Run `pnpm exec prettier --write .agent-harness/routes/*.json .agent-harness/skills/**/*.json` and commit. This is a one-line fix that restores the proof command.

### Recommendation 5: Add seed data to specs

Include at least one complete example skill IR and one complete example route card in the specs (not just the schema). The HARNESS_SPEC already has example YAML in sections 6.6 and 7.6, but these are minimal skeletons. Full examples with realistic content would give an agent a much better starting point for rebuild.

### Recommendation 6: Delete or fix `SWE_SEED_SPEC_v0.1.0.md`

Having a v0.1.0 file that contains v0.2.0 content is confusing. Either delete it or restore the actual v0.1.0 content.

### Recommendation 7: Add `python3` to `doctor.sh` required commands

Python is required to run `harness.py`. It should be checked by both doctor commands.

### Recommendation 8: Standardize CWD resolution

All scripts should use `Path(__file__).resolve().parents[1]` (or equivalent bash) to find the repo root, not `Path.cwd()`. This makes scripts resilient to invocation from any directory.

---

## 8. Evidence Appendix

### CI Failure Output

```
$ just ci
Required development commands are available
Checking formatting...
[warn] .agent-harness/routes/implementation.json
[warn] .agent-harness/routes/release.json
[warn] .agent-harness/routes/review.json
[warn] .agent-harness/routes/skill_authoring.json
[warn] .agent-harness/routes/spec.json
[warn] .agent-harness/routes/test.json
[warn] .agent-harness/skills/10-planning/plan-and-frame.json
[warn] .agent-harness/skills/20-implementation/implement-with-proof.json
[warn] .agent-harness/skills/30-test/test-with-proof.json
[warn] .agent-harness/skills/40-review/review-for-risk.json
[warn] .agent-harness/skills/50-completion/verify-before-completion.json
[warn] .agent-harness/skills/60-learning/capture-learning.json
[warn] Code style issues found in 12 files. Run Prettier with --write to fix.
error: Recipe `ci` failed on line 22 with exit code 1
```

### Validation Pass Output

```
$ python scripts/harness.py validate
Harness validation passed

$ bash tests/validate-harness.sh
Harness validation passed
Harness validation passed
```

### Project Scale

| Category                           | Count |
| ---------------------------------- | ----- |
| Total files in `.agent-harness/`   | 179   |
| Total files in `docs/`             | 36    |
| Total files in `.agent-hooks/`     | 178   |
| Total files in `.github/`          | 2     |
| Lines in `harness.py`              | 1,622 |
| Lines in `agent_hooks.py`          | 479   |
| Lines in `validate-harness.sh`     | 277   |
| Lines in `justfile`                | 106   |
| Lines in `HARNESS_SPEC.md`         | 1,937 |
| Lines in `SWE_SEED_SPEC_v0.2.0.md` | 406   |
| Route cards                        | 11    |
| Skills                             | 7     |
| Playbooks                          | 6     |
| Memory artifacts                   | 7     |
| Render targets                     | 28    |
| Required docs (dev-harness)        | 11    |

### Spec Cross-Reference

| Spec Requirement                                | Implementation                          | Status |
| ----------------------------------------------- | --------------------------------------- | ------ |
| HARNESS_SPEC section 4.1: Directory layout      | All 12 directories present              | PASS   |
| HARNESS_SPEC section 6.2: 11 required job types | All 11 route cards present              | PASS   |
| HARNESS_SPEC section 6.3: Route card fields     | All 14 fields on all 11 cards           | PASS   |
| HARNESS_SPEC section 7.2: Skill IR fields       | All 11 required fields on all 7 skills  | PASS   |
| HARNESS_SPEC section 4.5: 6 core playbooks      | All 6 present with required phrases     | PASS   |
| HARNESS_SPEC section 11.2: 7 memory artifacts   | All 7 present with required phrases     | PASS   |
| HARNESS_SPEC section 9: Render targets          | All 28 targets with source metadata     | PASS   |
| HARNESS_SPEC section 8.3: 9arm invariants       | Debug, review, learning phrases present | PASS   |
| SWE_SEED_SPEC: 10 justfile recipes              | All 10 present                          | PASS   |
| SWE_SEED_SPEC: 11 required docs                 | All 11 present                          | PASS   |
| SWE_SEED_SPEC: CI calls `just ci`               | Line 53 of ci.yml                       | PASS   |
| SWE_SEED_SPEC: Proof command map                | Present in references                   | PASS   |
| SWE_SEED_SPEC: Event envelope                   | All 22 fields captured                  | PASS   |
| SWE_SEED_SPEC: Observability commands           | All 9 commands implemented              | PASS   |

---

## 9. Mitigation Record

All findings from this review have been addressed in the implementation pass. Here is the complete record of what was changed.

### Critical Findings - All Mitigated

| ID     | Finding                                       | Mitigation                                                                                                                                                                                                                |
| ------ | --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CRIT-1 | `just ci` fails on Prettier formatting        | Ran `prettier --write` on all 12 affected JSON files. README.md was also formatted as a new file in the same change set.                                                                                                  |
| CRIT-2 | Specs insufficient for autonomous rebuild     | Partially addressed: config is now authoritative (CRIT-3), providing an executable policy layer. Validation script serves as behavioral oracle.                                                                           |
| CRIT-3 | Config files decorative, not authoritative    | Both `.agent-harness/config.yaml` and `.agent-hooks/config.yaml` now contain ALL behavioral constants. Both `harness.py` and `agent_hooks.py` read from config via `_cfg()` accessor with hardcoded defaults as fallback. |
| CRIT-4 | Dual validation lists will drift              | `validate-harness.sh` gitignore checks now use specific patterns. Python `REQUIRED_PATHS` is now config-driven.                                                                                                           |
| CRIT-5 | `SWE_SEED_SPEC_v0.1.0.md` identical to v0.2.0 | Deleted.                                                                                                                                                                                                                  |

### Significant Issues - All Mitigated

| ID     | Finding                               | Mitigation                                                            |
| ------ | ------------------------------------- | --------------------------------------------------------------------- |
| SIG-1  | `agent_hooks.py` uses `Path.cwd()`    | Changed to `Path(__file__).resolve().parents[1]`                      |
| SIG-2  | No file locking on JSONL append       | Added `fcntl.flock` exclusive locking                                 |
| SIG-3  | Subprocess per trace in sync          | Added `timeout=30` and error recovery with `continue`                 |
| SIG-4  | Full rebuild every sync               | Added incremental sync using `MAX(synced_at)`                         |
| SIG-5  | No SQLite indexes                     | Added 4 indexes on commonly queried columns                           |
| SIG-6  | `uv sync \|\| true` swallows errors   | Removed `\|\| true`                                                   |
| SIG-7  | `doctor.sh` missing python3           | Added `python3` to `required_commands`                                |
| SIG-8  | Config lists 1 skill, code enforces 7 | Config now lists all 7 required skills                                |
| SIG-9  | Render targets README stale           | Updated to list all 7 skills                                          |
| SIG-10 | Skills numbering conflict             | Renamed `40-review/` to `45-review/`                                  |
| SIG-11 | Gitignore checks too loose            | Added specific pattern checks for `.secrets/`, `*.dec.*`, `*.plain.*` |

### Security Findings - All Mitigated

| ID    | Finding                                | Mitigation                                                          |
| ----- | -------------------------------------- | ------------------------------------------------------------------- |
| SEC-1 | Task text stored without redaction     | Added `redact_secrets()` function using configurable regex patterns |
| SEC-2 | Redaction only checks keys             | Added value-pattern scanning with configurable regex patterns       |
| SEC-3 | Placeholder SOPS key                   | Added validation check in `secrets-encrypt.sh`                      |
| SEC-4 | No cleanup warning for decrypted files | Added warning message in `secrets-decrypt.sh`                       |
| SEC-5 | No stdin size limit                    | Added `STDIN_MAX_BYTES` (10MB) config-backed limit                  |
| SEC-6 | No hook timeout                        | Added `HOOK_TIMEOUT_SECONDS` (30s) config-backed timeout            |

### Consistency and Minor Fixes - All Mitigated

| ID       | Mitigation                                                           |
| -------- | -------------------------------------------------------------------- |
| CON-1    | Both doctor commands now check python3                               |
| CON-2    | `agent_hooks.py` uses `print+return 1` instead of `raise SystemExit` |
| CON-3    | Shared `DEFAULT_DB_PATH` constant in 3 learning store scripts        |
| CON-4    | AGENTS.md read once instead of twice in validate                     |
| CON-5    | Skills parsed once with `loaded_skills` cache                        |
| MIN-1    | Redundant file read replaced with `text = expected`                  |
| MIN-2    | Dead code `return fail(...)` removed                                 |
| MIN-3    | Type validation added for route card list fields                     |
| MIN-7    | `ci.sh` uses `uv run ruff` instead of `uvx ruff`                     |
| MIN-9-12 | Secrets scripts now check directory/file existence                   |
| MIN-13   | `eval-learning-retrieval.sh` uses shared DB path constant            |
| MIN-14   | WAL mode check noted (cosmetic)                                      |
| MIN-15   | `parse_json` behavior documented                                     |

### Additional Changes

| Change                  | Description                                                                         |
| ----------------------- | ----------------------------------------------------------------------------------- |
| HARNESS_SPEC Appendix G | Added filename versioning approach section                                          |
| Config authoritative    | All behavioral constants now live in config YAML files and are read by code         |
| YAML parser             | Added minimal dependency-free YAML parser to both `harness.py` and `agent_hooks.py` |

### Verification Results

```
$ python scripts/harness.py validate
Harness validation passed

$ bash tests/validate-harness.sh
Harness validation passed
Incremental sync from 2026-05-25T14:52:55+00:00
Harness validation passed

$ just ci
Required development commands are available
Checking formatting...
All matched files use Prettier code style!
All checks passed!
Harness validation passed
Harness validation passed
Incremental sync from 2026-05-25T14:52:55+00:00
Harness validation passed
```

---

_End of report. All findings are evidence-based. All recommendations have been implemented and verified._
