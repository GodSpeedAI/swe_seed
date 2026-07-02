# Run Harness Checks

Use these checks in order from narrowest to broadest.

## Validate harness structure and contracts

Run:

```bash
just harness-validate
```

Use this when you changed route cards, trace handling, docs required by validation, render targets, memory artifacts, or core harness files.

## Run the scaffold validation script

Run:

```bash
just harness-validate
```

Use this when you want deterministic end-to-end checks for required files, route behavior, and core CLI surfaces.

## Run the full project proof command

Run:

```bash
just ci
```

Use this before completion claims unless the governing route explicitly allows narrower proof.

## Read the output

The harness is strict about this point. Running checks is not enough. Read the output and tie the result back to the route's proof and done conditions.

## Suggested order

1. `just harness-validate`
2. `just harness-validate`
3. `just ci`

## Done when

The relevant checks passed, or skipped checks are justified with evidence.
