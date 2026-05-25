# Core Conformance Evals

These evals are deterministic checks for the harness itself. They are not substitutes for project tests. Each case proves one behavior that helps agents move toward verified outcomes.

Run the full suite through:

```bash
just ci
```

## Eval Case 1: Harness Contract Validates

**Command**

```bash
python scripts/harness.py validate
```

**Expected**

`Harness validation passed`

**Why it matters**

This proves the scaffold has required files, route cards, Skill IR, render targets, memory, hooks, playbooks, reflections, and eval structure.

## Eval Case 2: Semantic Router Emits an Executable Bugfix Plan

**Command**

```bash
python scripts/harness.py route "fix a failing regression test"
```

**Expected**

Output includes:

- `"job_type": "bugfix"`
- `"route_card": ".agent-harness/routes/bugfix.json"`
- `"work_loop"`
- `"next_action"`
- `reliable reproduction`
- `breadcrumb ledger`

**Why it matters**

This proves routing produces action, not only a job-type label.

## Eval Case 3: Semantic Router Selects Harness Improvement for Harness Work

**Command**

```bash
python scripts/harness.py route "implement HARNESS_SPEC.md semantic router"
```

**Expected**

Output includes:

- `"job_type": "harness_improvement"`
- `"route_card": ".agent-harness/routes/harness_improvement.json"`
- `"proof"`
- `"done_when"`

**Why it matters**

This proves harness changes route through the spec-first, validation-first workflow.

## Eval Case 4: Render Targets Are Generated and Fresh

**Command**

```bash
python scripts/harness.py render-skills && python scripts/harness.py validate
```

**Expected**

Validation passes after rendering.

Generated targets include:

- `.agent-harness/render-targets/claude/debug/debug-discipline/SKILL.md`
- `.agent-harness/render-targets/copilot/debug-discipline.instructions.md`
- `.agent-harness/render-targets/hooks/debug-discipline.prompt.md`
- `.agent-harness/render-targets/checklists/debug-discipline.md`

**Why it matters**

This proves Skill IR remains canonical and agent-specific surfaces do not drift.

## Eval Case 5: Hook Router Emits Purpose, Action, and Boundary

**Command**

```bash
.agent-harness/hooks/hook-router.sh prompt.submit
```

**Expected**

Output includes:

- `event: prompt.submit`
- `purpose:`
- `action:`
- `boundary:`

**Why it matters**

This proves hooks guide lifecycle behavior without replacing agent reasoning.

## Eval Case 6: Dev Harness Mirror Passes

**Command**

```bash
just ci
```

**Expected**

Formatting passes, static checks pass, harness validation passes, and scaffold validation passes.

**Why it matters**

This is the default completion proof for ordinary implementation claims.

## Eval Case 7: 9arm-Derived Debug Process Is Preserved

**Command**

```bash
rg -n "reliable reproduction|fail path|disprove|breadcrumb ledger" .agent-harness/skills .agent-harness/routes .agent-harness/playbooks .agent-harness/render-targets
```

**Expected**

Matches appear in Skill IR, bugfix route, debug playbook, and generated render targets.

**Why it matters**

This proves imported process invariants live in local executable artifacts, not only in notes.

## Eval Case 8: No Duplicate Render Target Bytes

**Command**

```bash
python scripts/harness.py validate
```

**Expected**

Validation passes without duplicate render-target content errors.

**Why it matters**

This enforces the canonical-file rule: identical files should be symlinked, while different target surfaces should be generated from Skill IR.
