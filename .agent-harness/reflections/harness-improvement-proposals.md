# Harness Improvement Proposals

## Use this when

Use this ledger when evidence suggests a material harness change: routing behavior, route-card schema, Skill IR, hooks, validation, memory policy, or completion gates. The ledger separates learning from mutation. It lets the harness improve without rewriting stable operating rules from a single preference.

## Do Not Use This For

- Routine implementation notes.
- One-off user preferences.
- Changes already requested explicitly by the user in the current task.
- Formatting, typo, or wording-only edits that do not change behavior.
- Ideas without evidence.

## Evidence Required

A proposal needs at least one concrete signal:

- failed validation or missing validation,
- user correction that exposes a general harness gap,
- repeated friction from memory, traces, or reviews,
- route output that does not produce a useful next action,
- proof command that cannot support the claim it is meant to support.

## Proposal States

- `proposed`: captured with evidence, not accepted yet.
- `accepted`: approved for implementation.
- `implemented`: change landed and proof passed.
- `rejected`: not worth changing; preserve reason.
- `superseded`: replaced by a better proposal.

## Proposal States Template

```yaml
id: hip-0001
status: proposed
observed_problem: ""
evidence:
  - type: ""
    value: ""
proposed_change:
  target: "" # route | skill | hook | memory | spec | validation | docs | playbook
  files:
    - ""
  summary: ""
expected_benefit: ""
risk: ""
rollback_plan: ""
validation_plan:
  - command: ""
    expected: ""
decision:
  accepted_by: null
  decided_at: null
  notes: ""
```

## Proposal States Rules

Proposal states are not progress theater. A proposal is useful only if it can be accepted, rejected, or validated. If the proposed change would not make the next agent action clearer, reduce failure recovery time, or improve proof quality, do not create it.

## Not enough evidence

If evidence is weak, do one of these instead:

- add an open question to `.agent-harness/memory/open-questions.md`,
- keep a local reflection using `reflection-template.yaml`,
- wait for a second occurrence,
- ask the user for the desired policy.

Do not mutate routing, skills, hooks, or specs only because an idea sounds cleaner.

## Active Proposals

No active proposals.
