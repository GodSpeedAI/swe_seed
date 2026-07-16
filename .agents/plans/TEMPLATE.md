# {{PROJECT_NAME}} — Implementation Plan

<!--
  TEMPLATE USAGE
  - Replace every {{PLACEHOLDER}} with concrete values.
  - Keep the falsifiability discipline: every Outcome must have a matching Falsification.
  - Duplicate the "Phase N" block as many times as needed; renumber sequentially.
  - Delete any optional section that does not apply, but prefer leaving the heading
    with an explicit "N/A — <reason>" rather than silently dropping it.
-->

**Status:** {{STATUS}}  <!-- e.g. Awaiting approval | Approved | In progress -->
**Evidence basis:** {{EVIDENCE_BASIS}}  <!-- How claims are grounded, e.g. "Direct code inspection — report at <path>" -->
**Proof level target:** {{PROOF_LEVEL_TARGET}}  <!-- e.g. local-confidence → live-dev-proof for Phases 1–N; focused-slice for Phase X -->

---

## Design Decisions Applied

<!--
  Record the *resolved* ambiguities BEFORE the phases begin.
  One row per decision so reviewers can challenge a single choice without re-reading the plan.
-->

| Decision | Resolution |
|---|---|
| {{DECISION_1}} | {{RESOLUTION_1}} |
| {{DECISION_2}} | {{RESOLUTION_2}} |
| {{DECISION_N}} | {{RESOLUTION_N}} |

---

## Dependency Order

<!--
  Show the execution order and WHY each phase depends on the prior one.
  Keep the "← rationale" annotation; it is what makes the ordering reviewable.
-->

```
Phase 1: {{PHASE_1_NAME}}   ← {{PHASE_1_RATIONALE}}
Phase 2: {{PHASE_2_NAME}}   ← {{PHASE_2_RATIONALE}}
Phase 3: {{PHASE_3_NAME}}   ← {{PHASE_3_RATIONALE}}
...
Phase N: {{PHASE_N_NAME}}   ← {{PHASE_N_RATIONALE}}  <!-- final phase is typically end-to-end verification -->
```

{{SELF_CONTAINMENT_STATEMENT}}
<!-- e.g. "Each phase is self-contained: it can be implemented and verified independently once its prerequisites are met." -->

---

<!-- ============================================================= -->
<!-- REPEAT THIS BLOCK PER PHASE. Copy/paste and renumber.         -->
<!-- ============================================================= -->

## Phase {{N}} — {{PHASE_NAME}}

**Repo:** `{{REPO_NAME}}` (`{{REPO_PATH_PRIMARY}}` / `{{REPO_PATH_ALT}}`)
<!-- List both OS paths if the team works cross-platform; otherwise delete the alt path. -->
**Prerequisite:** {{PREREQUISITE}}  <!-- "None." or "Phase X done (<why>)." -->

### Outcome (falsifiable)

<!--
  Each numbered item must be OBSERVABLE and BINARY (true/false), not aspirational.
  Reference concrete files, functions, fields, and tests by name.
-->

1. {{OUTCOME_1}}
2. {{OUTCOME_2}}
3. {{OUTCOME_N}}

**Falsification:** {{FALSIFICATION}}
<!--
  State the exact action that would break each outcome and the resulting failure signal.
  e.g. "Remove field X → test_Y fails with KeyError. Change one byte of Z without regenerating → hash comparison fails."
-->

---

### Files to Modify

<!--
  Use one sub-block per file. Tag each with [MODIFY] or [NEW].
  For [MODIFY], anchor to line numbers or unambiguous markers and show BEFORE/AFTER.
  For [NEW], provide the full intended file (or a faithful skeleton).
-->

#### [{{MODIFY|NEW}}] `{{FILE_PATH}}`

{{CHANGE_DESCRIPTION}}  <!-- Where in the file and what to insert/replace. -->

```{{LANGUAGE}}
{{CODE_SNIPPET}}
```

<!--
  Optional callouts — keep only those that apply:
  - **Note:** existing imports / dependencies already satisfied?
  - **Schema/contract impact:** does this change require a schema or interface update elsewhere?
  - **BEFORE / AFTER** blocks for in-place edits.
-->
**Note:** {{IMPLEMENTATION_NOTE}}

<!-- ...repeat [MODIFY]/[NEW] file sub-blocks as needed... -->

---

### {{REGENERATION_OR_BUILD_STEP_TITLE}}   <!-- OPTIONAL: delete if no codegen/build step -->

<!--
  Include only if the phase requires regenerating artifacts (codegen, manifests, migrations)
  AFTER the source edits. Provide exact, copy-pasteable commands.
-->

```bash
{{REGENERATION_COMMANDS}}
```

{{REGENERATION_RESULT_STATEMENT}}  <!-- e.g. "The committed manifest now contains meta.X." -->

---

### Verification Commands

<!--
  Commands a reviewer can run verbatim to confirm the phase. Pin the working directory.
  Prefer a fast, targeted test first, then a spot-check, then (optionally) the broader suite.
-->

```bash
cd {{REPO_PATH}}
{{VERIFICATION_COMMAND_1}}

# Spot-check (optional):
{{VERIFICATION_COMMAND_2}}
```

**Expected:** {{EXPECTED_RESULT}}  <!-- e.g. "All N tests pass. Spot-check prints MATCH." -->

### Done Conditions (Phase {{N}})

<!-- Checklist mirroring the Outcome + verification. Each item independently checkable. -->

- [ ] {{DONE_CONDITION_1}}
- [ ] {{DONE_CONDITION_2}}
- [ ] {{DONE_CONDITION_N}}

---

<!-- ============================================================= -->
<!-- END PER-PHASE BLOCK                                           -->
<!-- ============================================================= -->


## Phase {{FINAL_N}} — End-to-End Verification

<!--
  The closing phase. Run only after all prior phases are complete.
  Chains the verification commands across every repo/component touched.
-->

### Command Sequence

```bash
# 1. {{E2E_STEP_1_DESCRIPTION}}
{{E2E_STEP_1_COMMANDS}}

# 2. {{E2E_STEP_2_DESCRIPTION}}
{{E2E_STEP_2_COMMANDS}}

# N. {{E2E_STEP_N_DESCRIPTION}}
{{E2E_STEP_N_COMMANDS}}
```

### Final Proof Artifact

<!--
  Write the single falsifiable claim that becomes TRUE once all commands pass.
  Quantify it (number of repos, tests, components) so it can be checked, not just asserted.
-->

> {{FINAL_PROOF_STATEMENT}}

---

## What Remains After This Plan

<!--
  Be explicit about scope boundaries. For each deferred item, name the boundary
  (what IS done) and a concrete path forward. This prevents scope creep and
  documents known gaps for the next plan.
-->

| Gap | Boundary | Path Forward |
|---|---|---|
| {{GAP_1}} | {{BOUNDARY_1}} | {{PATH_FORWARD_1}} |
| {{GAP_2}} | {{BOUNDARY_2}} | {{PATH_FORWARD_2}} |
| {{GAP_N}} | {{BOUNDARY_N}} | {{PATH_FORWARD_N}} |
