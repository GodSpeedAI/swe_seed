# Core Conformance Evals

These evals are deterministic checks for the harness itself. They are not substitutes for project tests. Each case proves one behavior that helps agents move toward verified outcomes.

Run the full suite through:

```bash
just ci
```

## Eval Case 1: Harness Contract Validates

### Eval 11 Command

```bash
just harness-validate
```

### Eval 11 Expected

`Harness validation passed`

### Eval 11 Why it matters

This proves the scaffold has required files, route cards, Skill IR, render targets, memory, hooks, playbooks, reflections, and eval structure.

## Eval Case 2: Semantic Router Emits an Executable Bugfix Plan

**Command**

```bash
just harness-route "fix a failing regression test"
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
just harness-route "implement HARNESS_SPEC.md semantic router"
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
just harness-render-skills && just harness-validate
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
just harness-validate
```

**Expected**

Validation passes without duplicate render-target content errors.

**Why it matters**

This enforces the canonical-file rule: identical files should be symlinked, while different target surfaces should be generated from Skill IR.

## Eval Case 9: Trace Checkpoint Resume Packet Is Recoverable

**Command**

```bash
trace_json=$(just harness-trace-start "checkpoint smoke" | python -c 'import json,sys; print(json.load(sys.stdin)["trace_id"])')
just harness-trace-checkpoint "$trace_json" change "spec delta captured" "run targeted validation"
just harness-trace-resume "$trace_json"
```

**Expected**

Output includes:

- `"latest_checkpoint"`
- `"stage": "change"`
- `"next_action": "run targeted validation"`
- `"unresolved_risks"`

**Why it matters**

This proves the harness can emit a compact handoff packet for restart or agent turnover without depending on transcript replay.

## Eval Case 10: Canonical Build Prompt Bootstraps To Spec

**Command**

```bash
just harness-route "Let's make a react todo list"
```

**Expected**

Output includes:

- `"job_type": "spec"`
- `"route_card": ".agent-harness/routes/spec.json"`
- `"next_action"`

**Why it matters**

This proves the harness does real first-step routing on a fresh build prompt instead of defaulting to an unrelated maintenance route when semantic evidence is weak.

## Eval Case 11: Broad Bug Prompt Bootstraps To Bugfix

### Eval 12 Command

```bash
just harness-route "the login form is broken"
```

### Eval 12 Expected

Output includes:

- `"job_type": "bugfix"`
- `"route_card": ".agent-harness/routes/bugfix.json"`
- `"next_action"`

### Eval 12 Why it matters

This proves a broad user-reported failure enters the debugging workflow instead of stalling in a generic route.

## Eval Case 12: Broad Review Prompt Bootstraps To Review

### Command

```bash
just harness-route "review my recent auth changes for risk"
```

### Expected

Output includes:

- `"job_type": "review"`
- `"route_card": ".agent-harness/routes/review.json"`
- `"next_action"`

### Why it matters

This proves risk-focused review requests enter the review workflow directly instead of being mistaken for implementation work.
