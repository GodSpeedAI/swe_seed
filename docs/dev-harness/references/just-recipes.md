# Just Recipes

`justfile` is the stable command API for humans, agents, and CI.

| Recipe                                           | Purpose                                                                 |
| ------------------------------------------------ | ----------------------------------------------------------------------- |
| `just bootstrap`                                 | Install or synchronize local dependencies.                              |
| `just doctor`                                    | Report missing required or recommended tools.                           |
| `just format`                                    | Run formatting checks.                                                  |
| `just lint`                                      | Run static checks.                                                      |
| `just test`                                      | Run harness and project tests.                                          |
| `just ci`                                        | Run the local mirror of remote CI.                                      |
| `just harness-plan-learning-store <backend>`     | Print the staged plan for optional `rusql` and `ruvector` integration.  |
| `just harness-sync-learning-store <db_path>`     | Mirror distilled traces into an optional local SQLite learning store.   |
| `just harness-query-learning-store <args>`       | Query mirrored learning packets with structured filters before vectors. |
| `just harness-eval-learning-retrieval <db_path>` | Evaluate whether `ruvector` is justified yet.                           |
| `just agent-hooks-trace-last`                    | Show the most recent captured observability events.                     |
| `just agent-hooks-trace-session <session_id>`    | Filter captured observability events to one session.                    |
| `just agent-hooks-inspect <event_id>`            | Load one event with its normalized payload and captured artifacts.      |
| `just agent-hooks-replay <event_id>`             | Rehydrate a captured event for adapter debugging and recovery.          |
| `just agent-hooks-doctor`                        | Report observability layout health, counts, and derived index paths.    |
| `just agent-hooks-compact-logs`                  | Compact stale event logs without changing the durable event ledger.     |
| `just agent-hooks-index-rebuild`                 | Rebuild the optional SQLite-style observability index from JSONL logs.  |
| `just agent-hooks-export-otel <output>`          | Export captured events as OpenTelemetry-compatible JSON.                |
| `just agent-hooks-export-junit <output>`         | Export captured events as a JUnit-style XML report.                     |
| `just secrets-encrypt`                           | Encrypt plaintext secret inputs.                                        |
| `just secrets-decrypt`                           | Decrypt secrets into ignored local files.                               |
| `just secrets-edit <file>`                       | Edit one SOPS-managed secret.                                           |
| `just secrets-rotate-key`                        | Rotate SOPS recipients or age keys.                                     |

## Config Files

- `justfile`
- `scripts/*.sh`
- `.github/workflows/ci.yml`

## Ownership Boundaries

Put command orchestration in `justfile`. Put shell behavior in `scripts/`. Keep GitHub Actions focused on setup and `just ci`.

## Proof

```bash
just ci
```
