# Golden fixtures — current Python harness output

Captured from the **current Python harness** (`scripts/harness.py`) on the reconciliation
date, to serve as **parity targets** for the Rust rewrite (see `.agents/plans/0001`). Each
Rust CLI that supersedes a Python command must reproduce the corresponding `*.out` (and exit
behavior) — modulo the normalization notes below.

## Files

| Fixture | Command | Notes |
|---|---|---|
| `route_test.out` | `harness.py route "checkpoint smoke"` | Stable (no `--record`, so no timestamps). |
| `context-plan_test.out` | `harness.py context-plan "checkpoint smoke"` | Stable. |
| `render-skills.out` | `harness.py render-skills` | Stable. |
| `inspect_test.out` | `harness.py inspect test` | Stable. |
| `doctor.out` | `harness.py doctor` | Stable (short). |
| `validate.out` | `harness.py validate` | **Exits 0** ("Harness validation passed") since the three root layer specs now exist. The Rust `validate` must reproduce this green state and the same root-spec / artifact checks. |

## Normalization rules for parity tests

- **Timestamps / trace ids**: any `--record`/`trace`/`eval`/`fabricate` output embeds
  `created_at` (RFC3339) and timestamped `trace_id`s. Parity tests MUST normalize these
  (replace with a fixed token) before comparing. The fixtures here deliberately use the
  non-`--record` forms to stay byte-stable.
- **Absolute paths**: none captured; keep fixtures repo-relative.

## Not yet captured (need setup; capture during the build)

- `eval run --spec <path>` — needs an eval-spec fixture.
- `trace start|checkpoint|finish` — needs a lifecycle; normalize timestamps.
- `fabricate <need>` — needs `.fabricator` config/templates; normalize timestamps.
- `agent-hooks` runtime (from `scripts/agent_hooks.py`) — needs event payloads on stdin.

## Re-capture

```bash
PY=.venv/bin/python3
$PY scripts/harness.py route "checkpoint smoke" > tests/fixtures/golden/route_test.out
# ...etc per table above
```
