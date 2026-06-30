# 0014 — Trace and Durable Decisions

## Purpose

Define the durable decision/trace system that survives context compaction and enables
session continuity: route-decision records and trace lifecycle (`start → append →
checkpoint → resume → distill → finish`). This is the proven mechanism behind
`.agent-harness/traces/`.

## Non-goals

- Not the hook event log (0005/agent_hooks runtime) — that is high-volume telemetry; traces
  are curated durable decisions.
- Not eval/proof (0013) — traces *reference* proof, finish requires a claim + verification.

## Evidence (first-party)

| Source | Observed |
|---|---|
| `harness.baml` `TraceSchema{required_fields,optional_fields,identifier_fields,proof_fields}` | Trace record contract |
| `scripts/harness.py:1838-2030` `trace_start/append/checkpoint/resume/distill/finish` | Trace lifecycle CLI |
| `.agent-harness/traces/route-decisions/*.json` | Record: `trace_id, created_at, task, route{job_type, route_card, confidence, assumption, required_context, required_skills, work_loop, required_artifacts, proof, done_when, next_action}` |
| `budget-policy.yaml` `session_continuity` | restart rule: read route decision, trace record, changed specs, unresolved risks first |

## SWE_Seed requirements

1. **Route decision is recorded** at routing time (`route --record`) with the selected
   RouteCard, confidence, assumption, required context/skills, work loop, proof, done-when,
   and `next_action`.
2. **Trace lifecycle**: `start` opens a trace; `append` adds a note; `checkpoint` records a
   stage/summary/next-action/artifact/risk (compaction-survivable); `resume` replays the
   latest state; `distill` condenses to a durable record; `finish` requires a `--claim` and
   optional `--command`/`--result` (ties to proof, 0013).
3. **TraceSchema** declares `required_fields`, `identifier_fields` (trace_id/session/span),
   and `proof_fields`. Records validated against it.
4. **Session continuity**: on restart, read the latest route decision + trace record +
   changed specs + unresolved risks before new exploration (enforced as a doctor hint).
5. Trace records are append-only and content-addressed; generated records are gitignored by
   default (`budget-policy.yaml`).

## Data model

`TraceSchema` (canonical, 0019). Record shape from the observed JSON (frozen golden format).

```json
{
  "trace_id": "<ts>-<slug>",
  "created_at": "<rfc3339>",
  "task": "...",
  "route": { "job_type": "...", "route_card": "...", "confidence": "low|medium|high",
             "assumption": "...", "required_context": [], "required_skills": [],
             "work_loop": [], "required_artifacts": [], "proof": [], "done_when": [],
             "next_action": "..." },
  "checkpoints": [ {"stage": "...", "summary": "...", "next_action": "...",
                    "artifact": "...", "risk": "..."} ],
  "finish": { "claim": "...", "verification_command": "...", "result": "..." }
}
```

- **Rust**: `swe_seed::trace` (`schema.rs`, `record.rs`, `lifecycle.rs`).
- **Validation**: identifier fields present; `finish` requires a claim; append-only.

## CLI behavior

```
swe-seed route <task> [--record] [--capture-hook] [--agent --agent-version --session-id --trace-id --span-id]
swe-seed trace start <task> | append <trace> --note | checkpoint <trace> --stage --summary --next-action [--artifact --risk]
swe-seed trace resume <trace> | distill <trace> | finish <trace> --claim [--command --result]
```

## Generated files

`.agent-harness/traces/route-decisions/*.json`, `.agent-harness/traces/records/*.json`
(formats frozen as golden-file targets for parity with the current Python).

## Rust module boundaries

`swe_seed::trace`; route command lives in `swe_seed::route` (0004/0012) and writes trace
records via this module.

## Security and provenance considerations

Traces may capture tool output — apply the redaction rules (0005 hook runtime) before
writing. Generated records gitignored to avoid leaking session content.

## Tests

- `start→checkpoint→resume` reproduces the latest stage/next-action.
- `finish` without `--claim` fails.
- Record validates against `TraceSchema`; missing identifier field rejected.
- Golden-file parity vs a captured current-Python route-decision record.

## Open questions

- Are the JSON formats frozen (golden targets) or free to evolve? (Recommend frozen for
  v0.1 parity, version the schema after.)

## Acceptance criteria

- [ ] Full trace lifecycle implemented; route decisions recorded.
- [ ] TraceSchema validation + append-only + redaction.
- [ ] Session-continuity restart rule surfaced by doctor.
