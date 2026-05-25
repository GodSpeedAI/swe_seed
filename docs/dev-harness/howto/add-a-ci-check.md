# Add a CI Check

## Purpose

Add a new check without splitting local and remote truth.

## Prerequisites

- The check has a clear failure signal.
- The check can run locally without CI-only secrets.

## Steps

1. Add the tool or command to the local harness script.
2. Expose it through `just` if humans or agents need to run it directly.
3. Keep GitHub Actions calling `just ci` instead of duplicating the check.
4. Add or update validation when the new check changes the harness contract.
5. Update the matching documentation.

## Verification Command

```bash
just ci
```

## Common Failure Modes

- CI passes but local fails: move CI-only logic behind setup, not the check itself.
- Local passes but CI cannot find the tool: add installation or lockfile support.
- The check needs secrets: separate public validation from secret-backed deployment work.
