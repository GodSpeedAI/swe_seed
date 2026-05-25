# Initialize Dev Env

## Purpose

Prepare a workstation or agent environment to run the same commands used by CI.

## Prerequisites

- `git`
- `just`
- Recommended: `mise`, `uv`, `pnpm`, `direnv`, `devbox`, `sops`, and `age`

## Steps

1. Copy `.env.example` to `.env` if local overrides are needed.
2. Run `just bootstrap`.
3. Run `just doctor`.
4. If using `direnv`, run `direnv allow`.

## Verification Command

```bash
just doctor
```

## Common Failure Modes

- Missing `just`: install it before running harness commands.
- Missing recommended tools: install them when you need that workflow.
- `direnv` does not load `.envrc`: run `direnv allow` from the project root.
