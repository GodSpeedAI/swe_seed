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
