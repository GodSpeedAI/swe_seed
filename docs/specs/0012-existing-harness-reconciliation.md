# 0012 — Existing Harness Reconciliation (Authoritative Bridge)

## Purpose

Reconcile the **proven, in-use Python project** with specs 0002–0011 before the Rust
rewrite. The existing project is not a blank slate: it is the outer layer of a **3-layer,
spec-driven, proof-gated, self-improving agent harness** with an established vocabulary and
working CLIs. Specs 0002–0011 independently re-derived a *subset* of it under invented
names. This document makes the **proven model authoritative**, maps every existing artifact
to a spec, names the gaps the specs missed, and directs the Rust rewrite (build-to-specs,
superseding the Python).

Decisions driving this doc (confirmed with the maintainer):
- **Build to new specs** — the rewrite targets the spec design, superseding the Python
  prototype.
- **Contracts-as-data** — `.baml` files are canonical schema artifacts; Rust validates and
  renders them. **No LLM runtime** is ported (the current harness never calls LLMs at run
  time; `.baml` is consumed as data — confirmed at `scripts/harness.py:245,785`).
- **Reconcile then plan** — this doc precedes the revised plan.

## Non-goals

- Not re-porting BAML or any LLM client (contracts-as-data).
- Not preserving Python implementation details — only its *behavior, vocabulary, file
  formats, and proven concepts*.
- Not discarding specs 0002–0011 — they are amended by this doc.

## Evidence (first-party, in-repo)

| Source | Observed | Reconciliation impact |
|---|---|---|
| `scripts/harness.py` (2293 LOC) | Commands `validate, doctor, render-skills, route, inspect, context-plan, eval run, trace {start,append,checkpoint,resume,distill,finish}`; hand-rolled YAML parser; stdlib-only | These are the proven Harness-layer CLIs the Rust binary must reproduce |
| `scripts/agent_hooks.py` (661) | Hook event log (JSONL) + SQLite index, redaction, artifacts, OTEL/JUnit export, compaction | Proven hook *runtime* — richer than spec 0005 |
| `scripts/fabricate.py` (1169) | Bounded product→prototype runs | Fabricator layer — absent from specs |
| `.strategy/strategy.py` (701) | File-first strategy layer w/ template rendering | Strategy/doctrine projection — partial in specs |
| `agentic_capability_loop/adapters.py` (353) | SEA envelope emit/consume | Spec 0011 (already covered) |
| `.agent-harness/baml/baml_src/{swe_seed,harness,fabricator}.baml` | Full declarative data model (below) | **Canonical schema source** for Rust types |
| `AGENTS.md` | "Agent Operating Contract": Routing Mandatory, Required Job Types (11), Completion Rule, Context Discipline, Prose/Claim Discipline | Doctrine = the contract; 11 job types each need a RouteCard |
| `.agent-harness/context/budget-policy.yaml` | `raw_output_policy`, `tool_output_containment`, `session_continuity` | This **is** the context plane (spec 0005 ContextBuild + 0006) |
| `.agent-harness/traces/route-decisions/*.json` | Durable route-decision + trace records | Proven trace/proof model |

## The 3-layer model (specs missed two layers)

`LayerName { SweSeed, Harness, Fabricator }` (`swe_seed.baml`). The project is three nested
layers, outer governs inner:

```
SweSeed     (outer / centralization + governance)
  └─ Harness   (middle / routing, proof, context, hooks, learning)
       └─ Fabricator (inner / product → prototype pipeline)
```

- **SweSeed layer** — `ProjectSeed`, `LayerCapability`, `SeedPackageManifest`,
  `BoundaryReport`/`BoundaryFinding`, `ValidateLayerBoundaries`, `SelectLayerCapabilities`,
  `AssembleSeedPackage`, `ArtifactMetadata`, `SeedArtifactStatus`, `ReviewRequirement`,
  `SeedRegenerationPlan`. **Governs layer boundaries + assembles capability packages.**
  ⇒ This is what specs 0002/0003/0009 were reaching for.
- **Harness layer** — `RouteCard`, `SkillIR`, `EvalSpec`/`EvalCheck`/`EvalResult`,
  `ProofRecord`/`ProofDisposition`, `ContextBudget`/`ContextPack`, `HookPolicy`,
  `PermissionPolicy`, `TraceSchema`, `ReflectionTemplate`, `LearningRecord`/
  `LearningCandidate`/`LearningDisposition`, `SkillProposal`, `RegressionCase`,
  `AdaptationDecision`, `RegenerationPlan`, `HarnessNeed`, `HarnessADR`.
  ⇒ Covers specs 0004/0005/0006/0007/0008 — but proven and richer.
- **Fabricator layer** — `ProductSeed`→`JobStory`→`PRD`→`SDS`→`TDDPlan`→`AgentTask`→
  `FabricatorEvalSpec`→`FabricatorProofRecord`, EARS/Gherkin/ADR, `SemanticChainValidationReport`.
  ⇒ Entirely absent from specs 0002–0011.

**Spec correction:** SWE_Seed is the *outer* layer of this stack, not a standalone
installer. The Rust rewrite must implement all three layers (or explicitly stage them).

## Vocabulary alignment — adopt the proven terms

The proven `.baml`/CLI vocabulary is authoritative. Specs' invented terms are renamed.

| Spec term (invented) | Proven term (adopt) | Source |
|---|---|---|
| Projection / "route choice" | **RouteCard** + `route` command | `harness.baml` `RouteCard`; `harness.py:1644` |
| SkillPack (normalized) | **SkillIR** (intermediate rep) + `SkillProposal` | `harness.baml`; `render-skills` |
| DoctorCheck / verify | **EvalSpec/EvalCheck/EvalResult** + `ProofRecord` + `doctor`/`validate`/`eval run` | `harness.baml`; `harness.py:1167,1499` |
| ContextBuild hook + gateway context | **ContextBudget / ContextPack** + `context-plan` + `budget-policy.yaml` | `harness.baml`; `harness.py:1797` |
| Hook / PreToolUse gate | **HookPolicy + PermissionPolicy** (+ `agent_hooks` runtime) | `harness.baml`; `agent_hooks.py` |
| CapabilityProfile | **PermissionPolicy** + route `required_*` sets | `harness.baml` |
| ProvenanceRecord | **ArtifactMetadata + SourceRef + TraceabilityLink** (+ keep provenance hashing) | all `.baml` |
| Capability (registry entry) | **LayerCapability** (seed) / **RouteCard**+**SkillIR** (harness) | `swe_seed.baml` |
| SecurityGate / scan | keep as a **PermissionPolicy/EvalCheck** gate; SkillSpector = external `EvalCheck` | spec 0007 + `harness.baml` |
| (none) | **Trace / TraceSchema** (durable decisions, checkpoint/resume/distill) | `harness.py:1838+` |
| (none) | **Learning loop**: ReflectionTemplate→LearningRecord→SkillProposal→RegressionCase→AdaptationDecision | `harness.baml` |
| (none) | **BoundaryReport / ValidateLayerBoundaries** (3-layer governance) | `swe_seed.baml` |
| (none) | **RegenerationPlan** (idempotent artifact re-derivation) | all `.baml` |

## Coverage map — existing → spec → action

| Existing capability | Spec | Status | Action for Rust rewrite |
|---|---|---|---|
| `route` + RouteCard + route-decision traces | 0004/0006 | Partial (specs lacked route cards) | Adopt RouteCard as the core routing unit; 11 job types each need a card |
| `validate` (skill/route/incomplete-markers) + `doctor` | 0008 | Covered, rename | Doctor = validate + eval + boundary checks; reuse proven checks |
| `eval run` + EvalSpec/EvalResult + EvalClass/Status | 0008 | **Gap** | Add an Eval subsystem spec; proof gating runs evals |
| `render-skills` + SkillIR + SkillProposal | 0007 | Partial | Skill ingestion produces SkillIR; render projects to hosts |
| `context-plan` + ContextBudget/Pack + budget-policy.yaml | 0005/0006 | **Gap** | Context plane is first-class: budget + pack + raw-output policy |
| `trace {start,append,checkpoint,resume,distill,finish}` + TraceSchema | (none) | **Gap** | Add a Trace/durable-decision spec; ProofRecord ties to traces |
| `agent_hooks` runtime (JSONL+SQLite, redaction, OTEL/JUnit, compaction) | 0005 | Partial (specs had events, not runtime) | Port the hook *runtime* (logging/index/redaction/export) |
| HookPolicy + PermissionPolicy | 0005/0006 | Partial | Permission/hook policy gates tool use + edits |
| SweSeed layer: ProjectSeed/LayerCapability/SeedPackageManifest/BoundaryReport | 0002/0003 | Partial | Registry = SeedPackageManifest of LayerCapabilities; add boundary validation |
| Fabricator layer (ProductSeed→…→AgentTask, semantic chain) | (none) | **Gap (whole layer)** | Add a Fabricator-layer spec; stage after Harness layer |
| Learning loop (Reflection/Learning/Adaptation/Regression) | (none) | **Gap** | Add a learning-loop spec; this is the self-improving inner mechanism |
| RegenerationPlan / RegenerationInput | (none) | **Gap** | Add idempotent regeneration as a cross-cutting capability |
| `.baml` contracts | all | New decision | Treat `.baml` as canonical schema; derive/validate Rust types, no LLM runtime |
| AGENTS.md operating contract | 0002 doctrine | Partial | Doctrine = the Agent Operating Contract; routing is mandatory; 11 job types |
| `adapters.py` SEA envelope | 0011 | Covered | Port as Phase 6 (already planned) |
| strategy.py (file-first strategy layer) | 0002 | Partial | Strategy layer = doctrine/template projection; fold into SweSeed layer |

## Required spec amendments (directives)

These are amendments 0002–0011 must absorb (or be read with 0012 as override):

1. **0002** — reframe SWE_Seed as the **outer layer of a 3-layer stack**; add `LayerName`,
   `BoundaryReport`, and the `SweSeed→Harness→Fabricator` topology. Doctrine = AGENTS.md
   Agent Operating Contract; routing is mandatory.
2. **0003** — registry entry kinds adopt proven types: `LayerCapability`, `RouteCard`,
   `SkillIR`, `EvalSpec`, `ProofRecord`, `ContextBudget`, `HookPolicy`, `PermissionPolicy`,
   `TraceSchema`, `LearningRecord`. `SeedPackageManifest` is the assembled registry.
   `ArtifactMetadata` carries provenance.
3. **0004** — projection includes RouteCard rendering and SkillIR→host rendering
   (`render-skills`). Keep the host-adapter degradation contract.
4. **0005** — keep the normalized event vocabulary, but add the proven **hook runtime**
   (JSONL log + SQLite index + redaction + OTEL/JUnit export + compaction) and
   `HookPolicy`/`PermissionPolicy`.
5. **0006** — gateway/profile gating expressed via `PermissionPolicy`; context plane via
   `ContextBudget`/`ContextPack`/`budget-policy.yaml`.
6. **0007** — skill ingestion yields `SkillIR`; SkillSpector is an external `EvalCheck`.
7. **0008** — Doctor = `validate` + `doctor` + `eval run` + `ValidateLayerBoundaries`;
   ProofRecord/ProofDisposition is the proof artifact (unifies with 0011 live-proof rule).
8. **0009** — provenance via `ArtifactMetadata`/`SourceRef`/`TraceabilityLink`; keep hashing.
9. **0011** — unchanged (SEA envelope); note `ProofRecord` ↔ Proof* events.
10. **NEW specs to add** (follow-up): `0013-eval-and-proof`, `0014-trace-and-durable-decisions`,
    `0015-context-budget-plane`, `0016-learning-and-adaptation-loop`,
    `0017-fabricator-layer`, `0018-layer-boundary-governance`,
    `0019-baml-contracts-as-data`.

## Contracts-as-data (canonical schema)

`.baml` files are the **single source of truth for the data model**. The Rust rewrite:
- Parses `.baml` enums/classes and either (a) code-generates Rust structs from them, or
  (b) hand-writes structs with a CI test asserting parity against the `.baml` contracts.
- Validates/renders `.baml`-defined artifacts as files (as the Python harness does).
- **Never** invokes an LLM at runtime. Generation functions in `.baml` are contracts for an
  external/offline generation step, not a runtime dependency of the Rust binary.

> Recommendation: (b) hand-written structs + a `baml_parity` CI test. A `.baml`→Rust
> generator is more code than the contracts justify for v0.1 (YAGNI). Revisit if the
> contracts churn often.

## Plan impact

`plan 0001` must be revised to **build all three layers to the proven vocabulary**,
superseding the Python:
- Phase 1 (registry) → **SweSeed layer**: `LayerCapability`, `SeedPackageManifest`,
  `ArtifactMetadata`, `ValidateLayerBoundaries`, `.baml` parity.
- Phase 2 (adapter) → add RouteCard + SkillIR rendering (`route`, `render-skills`).
- Phase 3 (doctor) → `validate` + `doctor` + `eval run` + boundary checks + ProofRecord.
- Phase 4 (hooks/gateway) → proven hook runtime + HookPolicy/PermissionPolicy + ContextBudget/Pack.
- Phase 5 (skill ingestion) → SkillIR + SkillProposal + the learning loop hook points.
- Phase 6 (federation) → unchanged.
- **New phases**: Trace subsystem; Fabricator layer; Learning/Adaptation loop — staged
  after the Harness layer is proven.
- Each phase must reproduce the existing CLI surface and file formats it supersedes, with a
  golden-file parity test against current Python output where formats are stable.

## Open questions

- Stage the Fabricator layer in v0.1 or defer to v0.2? (It is the largest gap; recommend
  **defer** — ship SweSeed+Harness layers first.)
- `.baml`→Rust: generator vs. hand-written+parity-test? (Recommend hand-written.)
- Keep the hand-rolled YAML parser behavior, or adopt `serde_yaml`? (Recommend `serde_yaml`;
  add a parity test against existing config files.)
- Are the route-decision/trace JSON formats frozen (golden-file targets) or free to evolve?

## Acceptance criteria

- [ ] 3-layer model (`SweSeed/Harness/Fabricator`) is the authoritative architecture.
- [ ] Proven vocabulary adopted; no invented term shadows an existing `.baml`/CLI concept.
- [ ] Every existing command, runtime, and `.baml` class maps to a spec + an action.
- [ ] Gaps (eval, trace, context budget, learning loop, fabricator, boundary governance)
      have a home (new specs 0013–0019).
- [ ] `.baml` = canonical schema, contracts-as-data, no LLM runtime.
- [ ] Revised `plan 0001` builds to this reconciled model and supersedes the Python.
