# Proof Command Map

Use this map when making or checking completion claims.

| Claim                        | Proof command                                |
| ---------------------------- | -------------------------------------------- |
| Harness files exist          | `bash tests/validate-harness.sh`             |
| Local CI passes              | `just ci`                                    |
| Tooling is installed         | `just doctor`                                |
| Formatting is clean          | `just format`                                |
| Static checks pass           | `just lint`                                  |
| Tests pass                   | `just test`                                  |
| Observability surface works  | `scripts/agent-hooks doctor --observability` |
| Secret workflow is available | `just secrets-edit <file>`                   |

## Config Files

- `tests/validate-harness.sh`
- `justfile`
- `.github/workflows/ci.yml`

## Ownership Boundaries

Use the narrowest proof command that supports the claim. Use `just ci` for general completion.

## Proof

```bash
bash tests/validate-harness.sh
```
