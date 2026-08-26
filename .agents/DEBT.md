# Debt

No active technical debt is currently recorded.

## Resolved (2026-07-29)

- ~~gateway serve: single-worker concurrency~~ → promoted to spec 0020 §15 "Concurrency and
  Overload" and implemented as a bounded worker pool (`DEFAULT_WORKERS=4`, `DEFAULT_QUEUE=16`)
  with HTTP 503 on overflow. Proven: overlap, bounded-queue 503, exact counts.
- ~~gateway serve: discovery is declared-catalog only~~ → promoted to spec 0020 §7 "Live Catalog
  Discovery" and implemented: `tools/list`/`resources/list`/`prompts/list` fan-out at
  startup/reload, declared-wins merge, per-backend isolation, drop-and-report. Proven.

