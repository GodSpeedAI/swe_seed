# Run Local CI

## Purpose

Run the local mirror of remote CI before claiming work is complete.

## Prerequisites

- Development environment initialized.
- Project dependencies installed through `just bootstrap` or the package manager required by the changed area.

## Steps

1. Run the full proof command.
2. Read the complete output.
3. Fix failures before reporting completion.

```bash
just ci
```

## Verification Command

```bash
just ci
```

## Common Failure Modes

- Formatter fails: run the formatter or edit the reported files.
- Linter fails: fix the reported issue, then rerun `just ci`.
- Validation fails: update the scaffold, docs, or validation script so the contract and implementation match.
