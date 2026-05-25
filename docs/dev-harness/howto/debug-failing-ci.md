# Debug Failing CI

## Purpose

Find whether a failure comes from setup, command wiring, tool behavior, or project code.

## Prerequisites

- Access to the failing CI log.
- Local checkout of the same branch or commit.

## Steps

1. Identify the failing GitHub Actions step.
2. If the failing step is `Run local CI mirror`, run `just ci` locally.
3. If setup failed before `just ci`, compare the workflow setup with `.mise.toml`, `package.json`, `pyproject.toml`, and lockfiles.
4. If the failure involves hook wiring, adapter behavior, or mismatched proof artifacts, inspect the latest captured observability event with `scripts/agent-hooks trace --last` and `scripts/agent-hooks inspect --event <event_id>`.
5. Fix the smallest contract mismatch.
6. Rerun the failing command locally.

## Verification Command

```bash
just ci
```

## Common Failure Modes

- Missing dependency cache is usually slow, not broken.
- Missing package-installed command usually means dependencies were not installed from the lockfile.
- A recipe exists locally but not in CI usually means the workflow skipped setup or is not running from the project root.
- A hook or wrapper failed without enough context usually means you need the captured payload and artifacts from `scripts/agent-hooks inspect --event <event_id>` before changing the adapter.
