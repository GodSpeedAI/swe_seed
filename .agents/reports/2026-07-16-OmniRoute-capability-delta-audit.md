# Comparative Outcome-Capability Delta Audit — OmniRoute vs. SWE_SEED

Date: 2026-07-16

## 1. Executive Verdict

- **Most consequential delta:** OmniRoute's *evidence discipline for probabilistic behavior* — golden-set quality tests with key-phrase preservation checks, LLM-judge scoring, savings benchmarks, and chaos/failure-injection testing. SWE_SEED has an eval plane (spec 0013, `eval_cli.rs`, `.agent-harness/evals/`) but its evals are markdown conformance documents, not data-driven golden sets with quantitative gates.
- **Acquire first:** the golden-set eval *pattern* (JSONL golden data + programmatic quality/parity assertions wired into `just harness-eval-run`), not any OmniRoute code.
- **Do not copy:** the provider gateway itself (250-provider routing, quota tracking, compression proxy, Electron/dashboard/i18n machinery). It is a different product domain. If SWE_SEED users want free-tier LLM failover, the correct relationship is **protocol-level**: point host agents' base URL at a running OmniRoute instance. Zero code transfer.
- **Is the target strategically valuable?** Moderately, and almost entirely as **evidence discipline and operational practice**, not code or architecture. The two projects route different things: OmniRoute routes *inference requests to providers*; SWE_SEED routes *work requests to route cards*. The nouns collide ("routing", "fallback", "policy") but the semantics do not transfer.

## 2. Inspection Substrate

| Item | Value |
|---|---|
| Current project | SWE_SEED (`/home/sprime01/projects/SWE_SEED`) |
| Current branch | `deploy-prep` |
| Current commit | `a55d14b377538226b0727a80db06d3509f37d857` |
| Worktree | dirty: 1 modified doc (`docs/agent-harness/explanations/federation-signing-and-traces.md`) |
| Target repo | https://github.com/diegosouzapw/OmniRoute |
| Target ref | default branch, `--depth 50` clone at `.tmp/ref/OmniRoute/` |
| Target commit | `bc6cd2a8068a8f252e059e0e854d7464c6251872` (2026-07-16) |
| Target version | `omniroute` npm package v3.8.49, MIT license (© 2026 diegosouzapw) |
| Tools | git 2.x, python3 for manifest parsing |

Note: the context-mode MCP sandbox was unavailable this session (better-sqlite3 NODE_MODULE_VERSION mismatch — `npm rebuild` needed in the plugin cache). Inspection used truncated Bash reads instead.

## 3. Method and Evidence Standard

Current project inspected first (README, `justfile` recipe surface, `crates/swe-seed/src/*_cli.rs` module list, `tests/` and `crates/swe-seed/tests/`, `docs/specs/0001–0020`, `.agent-harness/` layout). Target then inspected: manifest, `src/domain/` and `src/lib/` module inventory, `tests/` inventory (3,164 `*.test.*` files), and traced two claims into implementation: fallback chains (`src/domain/fallbackPolicy.ts` → SQLite via `src/lib/db/domainState`) and golden-set compression quality (`tests/golden-set/compression-quality.test.ts` → `open-sse/services/compression/caveman.ts` + `tests/golden-set/data/prompts.jsonl`). Maturity ratings: *proven* = traced entry point → mechanism → test/fixture; *implemented but unproven* = code found, evidence not traced; *declared* = README only. Given the size of both repos and a shallow clone, tracing was sampled, not exhaustive; matrix rows below state their evidence basis.

## 4. Current-Project Capability Baseline (abridged)

| Capability | Outcome | Entry point | Evidence | Maturity |
|---|---|---|---|---|
| Prompt→route-card routing | A work request deterministically resolves to a route with required context, loop, artifacts, proof | `swe-seed` CLI (`harness_cli.rs`), `just harness-route` | `crates/swe-seed/tests/cli_golden.rs`, `.agent-harness/routes/` | Proven |
| Trace lifecycle (start/append/checkpoint/resume/distill/finish) | Work is recoverable after interruption; completion claims carry proof commands | `trace_cli.rs`, `just harness-trace-*` | `trace_cli.rs` tests, `.agent-harness/traces/` | Proven |
| Proof gating | Completion claims blocked without proof | `gate_cli.rs`, hooks | hook wiring in `.agent-harness/hooks/` | Implemented, sampled not fully traced |
| Eval plane | Conformance of routes/harness checked | `eval_cli.rs`, `just harness-eval-run` | `.agent-harness/evals/*.md` (markdown conformance docs + `schemas/`) | Partial — no data-driven golden sets |
| Learning store | Lessons persisted/queried (files + SQLite) | `learning_cli.rs`, `just harness-*-learning-store` | `learning_cli.rs` tests, spec 0016 | Proven |
| Doctor/drift detection | Install/config drift diagnosed | `doctor_cli.rs`, spec 0008 | golden baseline fixtures (recent commits) | Proven |
| Federation, provenance, secrets | Signed cross-repo evidence exchange | `federation_cli.rs`, `provenance_cli.rs`, `just federation-*` | spec 0011, 0009; tests present | Implemented |
| Fabricator layer | Need → fabricated artifact chain with validation | `fabricate_cli.rs`, spec 0017 | `just fabricate-validate-chain` | Implemented |
| Hooks/OTel export | Lifecycle events exported (OTel, JUnit) | `hooks_cli.rs`, `just agent-hooks-export-*` | recipes present | Implemented |
| **LLM provider access** | — | **None. Zero LLM client code in the crate** (no anthropic/openai hits in `crates/swe-seed/src/`). Host agents bring their own model access. | — | Absent **by design** (host-adapter boundary, spec 0004) |

**Target inspection checklist derived from baseline:** (a) anything that would strengthen the eval plane's quantitative evidence; (b) resilience/recovery testing practice; (c) whether provider routing/fallback has any translation into route-card routing; (d) operational health monitoring vs. doctor; (e) anything relevant to the host-adapter/MCP-gateway specs (0004, 0006, 0020).

## 5. Target Capability Map (material capabilities only)

| Capability | Outcome | Mechanism | Evidence | Maturity |
|---|---|---|---|---|
| Multi-provider gateway with auto-fallback | An agent CLI keeps working when a provider dies or hits quota | `src/domain/fallbackPolicy.ts` (priority-ordered chains, SQLite-persisted via `lib/db/domainState`), `policyEngine.ts`, `comboResolver.ts`, `tagRouter.ts`, `degradation.ts`, `lockoutPolicy.ts`, `quotaCache.ts` | Traced to code + SQLite persistence; unit tests exist in `tests/unit` | Proven (core), sampled |
| Token compression (RTK/caveman) with quality guarantees | Prompts shrink 15–95% without losing key content | `open-sse/services/compression/caveman.ts`; judge client `src/lib/compression/judgeModelClient.ts` | `tests/golden-set/compression-{quality,savings,caveman-v2,upstream-parity}.test.ts` + `data/prompts.jsonl` (key-phrase preservation asserts) | Proven |
| Golden-set + benchmark eval harness | A probabilistic transformation is gated by measurable quality/savings thresholds | JSONL golden data, key-phrase extraction, parity tests, `bench:compression`/`eval:compression` npm scripts | Same test files | Proven |
| Chaos/failure injection | Failure modes are exercised deliberately | `src/lib/chaos/{chaosConfig,chaosExecutor}.ts` | executor traced; dedicated test dir not confirmed | Implemented but unproven |
| Credential health scheduling | Dead API keys detected before they break a session | `src/lib/credentialHealth/{cache,scheduler}.ts` | not traced to tests | Implemented but unproven |
| Test-suite scale & taxonomy | Regression confidence across unit/integration/e2e/security/load/llm-security/homolog/boundary/golden-set/snapshots | `tests/` directory taxonomy, 3,164 test files | inventory observed | Proven as practice |
| MCP/A2A/ACP bridges, Electron app, dashboard, 42-language docs pipeline | Distribution & UX reach | `src/lib/{a2a,acp}`, `electron/`, `docs/i18n` | not traced | Declared/implemented |

## 6. Comparative Delta Matrix

| Target outcome | Current state | Delta class | Target mechanism | Evidence strength | Strategic value | Acquisition burden | Disposition |
|---|---|---|---|---|---|---|---|
| Quantitative golden-set evals gating probabilistic behavior | Markdown conformance evals only (`.agent-harness/evals/`) | **PARTIAL / MATURITY DELTA** | JSONL golden data + key-phrase/parity assertions + judge model | Strong (traced) | High — SWE_SEED's whole thesis is proof-gated completion; its own eval plane is its weakest proof surface | Low (pattern only; substrate exists in `eval_cli.rs`) | **Acquire now** (pattern, clean-room) |
| Chaos/failure-injection of the harness lifecycle | Recovery exists (trace resume) but is not adversarially tested | MATURITY DELTA | `chaosConfig`/`chaosExecutor` fault injection | Medium | Medium — hardens the resume/recovery claims in spec 0014 | Medium | **Acquire after prerequisite** (golden evals first) |
| Provider failover so agents never stall on quota | Absent by design (host brings model) | INCOMPATIBLE at code level / **LATENT via protocol** | Gateway + fallback chains | Strong | Medium for users, zero for the codebase | Low as protocol integration (docs only) | **Protocol-level integration**: document OmniRoute as an optional host-side gateway in the host-adapter docs |
| Token compression | User-level RTK already deployed globally; harness doesn't own prompts | NON-CONSEQUENTIAL | caveman compressor | Strong | Low | — | Reject |
| Credential health scheduler | `doctor_cli.rs` covers install/config drift; API keys out of scope | EQUIVALENT (different scope) | scheduler + cache | Weak | Low | — | Reject |
| Degradation/lockout policy semantics | Route conflicts handled differently (`route-conflicts.md` eval) | NON-CONSEQUENTIAL | policy engine | Medium | Low | — | Reject |
| Dashboard/Electron/i18n/MCP bridges | Out of scope for a spec-first harness | NON-CONSEQUENTIAL | — | — | Low | High | Reject |

Burden ratings reflect: golden-eval pattern = low implementation time, no new dependencies, no semantic migration; chaos = medium implementation time + medium test burden; everything rejected = high maintenance/opportunity cost against near-zero outcome delta.

## 7. Recommended Capability Acquisition Dossiers

### Dossier 1 — Data-driven golden-set eval gate

- **A. Desired outcome:** `just harness-eval-run` can execute a *data-backed* eval spec: a JSONL golden set of inputs with expected invariants (e.g., route decisions, context-plan contents, trace-distill key phrases), assert them programmatically, and fail CI on regression with a quantitative report.
- **B. Current limitation:** `.agent-harness/evals/core-conformance.md`, `negative-conformance.md`, `route-conflicts.md` are prose conformance documents plus `schemas/`. `eval_cli.rs` runs specs, but there is no golden dataset format, no threshold gating, and no per-case pass/fail evidence artifact. The harness gates *agents* on proof but does not gate *itself* with the same rigor.
- **C. Target implementation:** `tests/golden-set/compression-quality.test.ts` loads `data/prompts.jsonl` ({prompt, keyPhrases}), runs the transformation (`cavemanCompress`), extracts code blocks, and asserts key-phrase preservation; sibling tests assert savings thresholds (`compression-savings.test.ts`) and upstream parity (`compression-upstream-parity.test.ts`); `judgeModelClient.ts` adds optional LLM-judge scoring; npm scripts `bench:compression` / `eval:compression` expose it as a proof command.
- **D. Essential mechanism:** *versioned golden dataset + invariant assertions + threshold gate exposed as a single proof command.* Incidental: TypeScript, node:test, JSONL-of-prompts schema, LLM judge, compression as the subject.
- **E. Acquisition options → recommendation:** option 5, **reimplement against current abstractions** (extend `eval_cli.rs` to accept a golden-set spec referencing a JSONL/YAML dataset under `.agent-harness/evals/golden/`). No code copied; MIT attribution not required for a pattern.
- **F. Integration points (confirmed):** `crates/swe-seed/src/eval_cli.rs`, `.agent-harness/evals/` (+ `schemas/`), `justfile` (`harness-eval-run`), `crates/swe-seed/tests/cli_golden.rs`, `docs/specs/0013-eval-and-proof.md`. Probable: CI `ci.yml`, `.agent-harness/reports/`.
- **G. Semantic translation:** target "golden set" → SWE_SEED "eval spec with golden dataset"; target "judge model" → do **not** import (SWE_SEED makes no LLM calls; a judge would violate the host-adapter boundary — deterministic invariants only, or delegate judging to the host agent as a documented proof command). Do not import "compression", "provider", or "combo" vocabulary.
- **H. Prerequisites:** none missing — `eval_cli.rs`, schemas dir, and justfile recipe already exist. Optional: a report emitter into `.agent-harness/reports/`.
- **I. Risks/debt:** golden datasets rot if routes change — mitigate by regenerating via a `--bless` flow like the existing doctor golden baseline (recent commit `74da38a` shows the project already handles committed golden fixtures). Low lock-in, no new dependencies.
- **J. Minimum vertical slice:** one golden dataset of ~10 route-decision cases (`prompt → expected route id + required context keys`), one `eval_cli` subcommand or spec type that runs them, exits non-zero on any mismatch, and writes a per-case JSON report; wired as `just harness-eval-golden`.
- **K. Acceptance/proof:** `just harness-eval-golden` passes on HEAD; mutating a route card breaks exactly the affected cases (failure test); report artifact lists per-case verdicts; CI job runs it. Failure behavior: non-zero exit + named failing cases, never silent pass.

### Dossier 2 — Chaos-style recovery testing of trace resume (secondary)

- **A. Outcome:** trace `resume`/`checkpoint` guarantees (spec 0014) are proven under injected failure: killed process mid-append, truncated trace file, missing checkpoint.
- **B. Limitation:** `trace_cli.rs` tests exercise the happy path; no adversarial corpus found.
- **C/D. Target implementation & essence:** OmniRoute's `chaosExecutor.ts` injects configured faults at runtime. Essential mechanism: *deliberate fault injection at persistence boundaries with asserted recovery outcome.* Incidental: its config format and HTTP context.
- **E. Recommendation:** option 8, adopt the pattern only — Rust tests that corrupt/truncate fixture traces and assert `harness-trace-resume` behavior.
- **F. Integration:** `crates/swe-seed/tests/trace_cli.rs`, `.agent-harness/traces/` fixtures.
- **J. Slice:** one test: write trace, truncate last record, run resume, assert defined recovery or defined error.
- **K. Proof:** test in `cargo test`; failure behavior = explicit diagnostic, not panic.

## 8. Rejected or Deferred Capabilities

- **Provider gateway / fallback / quota routing:** architecture and product conflict — SWE_SEED deliberately owns no model access (host-adapter contract, spec 0004). Importing it would erode the layer boundary spec 0018 exists to protect. Value is captured instead by one docs paragraph (see sequence step 3).
- **Token compression:** duplicative — RTK already deployed at the user level; the harness does not own prompt bytes.
- **Credential health, degradation/lockout, arena Elo sync, cloud sync, tunnels, Electron, dashboard, i18n pipeline:** target-specific operational surface for a consumer gateway; no outcome delta for a spec-first dev harness; high maintenance cost.
- **LLM-judge evaluation:** semantically attractive but violates the no-LLM-in-harness boundary; defer — revisit only as a host-delegated proof command.

## 9. Recommended Acquisition Sequence

1. **Golden-set eval gate** (Dossier 1). Prerequisite: none. Slice: 10-case route-decision golden set + `harness-eval-golden`. Proof: CI-gated pass + mutation-breaks-case demo. Opens: quantitative gating for context plans, skill rendering, and trace distillation next.
2. **Chaos recovery tests** (Dossier 2). Prerequisite: step 1's report format (reuse for evidence). Slice: one truncated-trace resume test. Proof: `cargo test` failure test. Opens: hardened claims in spec 0014 and federation evidence integrity.
3. **Protocol-level OmniRoute note** in host-adapter docs (spec 0004 area): document that host agents may run behind an OmniRoute (or any OpenAI-compatible) gateway; no code change. Proof: doc lint passes.

## 10. Implementation Specification Inputs (for Dossier 1)

- **Problem:** the harness gates agents on proof but its own eval plane is prose-based; regressions in routing/context planning are caught only by CLI golden tests, not by declared eval specs.
- **Scope boundary:** eval plane only; no changes to routing, traces, hooks, federation; no LLM calls; no new dependencies.
- **Components:** `eval_cli.rs` (new spec type or subcommand), `.agent-harness/evals/golden/*.jsonl` + schema in `evals/schemas/`, `justfile` recipe, `ci.yml` job, spec 0013 amendment.
- **Data model:** golden case = `{id, prompt, expected: {route, context_keys[], …}}`; report = `{case_id, verdict, expected, actual}` JSON lines.
- **Contract changes:** none external; new CLI surface is additive.
- **Migration:** none; existing markdown evals remain valid.
- **Acceptance:** `just harness-eval-golden` green on HEAD; deliberate route-card mutation fails the mapped cases; report artifact emitted; CI enforces.
- **Failure cases:** malformed dataset → schema error naming the line; missing route → per-case FAIL, non-zero exit.
- **Unresolved decisions:** dataset format (JSONL vs. YAML to match existing eval schemas); whether reports land in `.agent-harness/reports/` or `target/`; `--bless` regeneration policy.
- **Work packages:** (1) schema + loader, (2) runner + report, (3) justfile/CI wiring + spec 0013 update, (4) failure tests.

## 11. Open Questions

1. Should self-eval golden reports feed the federation evidence stream (`ProofCompleted` events) or stay CI-local? Changes report format and signing requirements.
2. Is a host-delegated LLM-judge proof command ever desired, or is the no-LLM boundary absolute? Determines whether Dossier 1's schema reserves a judge field.

## 12. Evidence Appendix

**Current project:** `README.md` (boundary section, F-08), `justfile` (recipe surface incl. `harness-eval-run`, `harness-trace-*`, `federation-*`), `crates/swe-seed/src/{eval_cli,trace_cli,gate_cli,doctor_cli,federation_cli,provenance_cli,learning_cli,fabricate_cli}.rs`, `crates/swe-seed/tests/{cli_golden,trace_cli,learning_cli}.rs`, `.agent-harness/{routes,evals,traces,hooks}/`, `docs/specs/0004,0008,0011,0013,0014,0016,0017,0018`, commit `74da38a` (committed route golden fixture precedent). Zero anthropic/openai references in crate sources.

**Target (at `bc6cd2a8`):** `src/domain/fallbackPolicy.ts` (priority chains, SQLite hydration via `src/lib/db/domainState`), `src/domain/{policyEngine,comboResolver,tagRouter,degradation,lockoutPolicy,quotaCache}.ts`, `tests/golden-set/compression-{quality,savings,caveman-v2,upstream-parity}.test.ts`, `tests/golden-set/data/prompts.jsonl`, `open-sse/services/compression/caveman.ts`, `src/lib/compression/judgeModelClient.ts`, `src/lib/chaos/{chaosConfig,chaosExecutor}.ts`, `src/lib/credentialHealth/{cache,scheduler}.ts`, `package.json` v3.8.49 (bin `omniroute`), MIT `LICENSE`, 3,164 `*.test.*` files across a unit/integration/e2e/security/load/golden-set/homolog taxonomy.

---

## Conclusion

**READY FOR IMPLEMENTATION SPEC**

The recommended acquisitions are pattern-level (clean-room reimplementation against existing `eval_cli.rs`/test substrate), require no new dependencies, carry no MIT obligation beyond none-for-patterns, and each has a small provable vertical slice. The two open questions affect schema details, not feasibility.
