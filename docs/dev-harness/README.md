# Dev Harness

The dev harness defines how this project installs tools, runs checks, manages secrets, captures file-first observability, and proves work is ready.

The command contract is simple: use `just`. Local commands are the source of truth, and GitHub Actions calls the same commands where practical.

## Start Here

- New environment: [Initialize Dev Env](howto/initialize-dev-env.md)
- Before claiming work is ready: [Run Local CI](howto/run-local-ci.md)
- CI is failing: [Debug Failing CI](howto/debug-failing-ci.md)
- Add or change checks: [Add a CI Check](howto/add-a-ci-check.md)

## Why It Exists

- [Local CI Parity](explanations/local-ci-parity.md)
- [Observability Model](explanations/observability-model.md)
- [Secrets Model](explanations/secrets-model.md)

## References

- [Just Recipes](references/just-recipes.md)
- [Observability Contract](references/observability-contract.md)
- [Proof Command Map](references/proof-command-map.md)
