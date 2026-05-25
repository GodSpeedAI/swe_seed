# SWE SEED CI/CD Development Harness Specification v0.1.0

## Purpose

This specification defines a CI/CD development harness that gives humans and agents one stable way to install tools, run checks, manage secrets, debug failures, and prove work is ready.

The desired outcome is behavioral: fewer hidden commands, fewer environment surprises, and less completion theater. A contributor should know what to run, what the result means, and which document to update when the harness changes.

## Core Principle

```text
Remote CI is not a separate truth.
The local just interface is the developer and agent API.
GitHub Actions should call the same commands wherever practical.
```

## Required Command Interface

The project MUST provide a `justfile` with these recipes:

- `just bootstrap`: install or synchronize local tool dependencies.
- `just doctor`: report missing required or recommended tools.
- `just format`: run formatting checks.
- `just lint`: run static checks.
- `just test`: run harness and project tests.
- `just ci`: run the local mirror of remote CI.
- `just secrets-encrypt`: encrypt plaintext secret inputs.
- `just secrets-decrypt`: decrypt encrypted secrets into ignored local files.
- `just secrets-edit <file>`: edit a SOPS-managed secret.
- `just secrets-rotate-key`: rotate SOPS recipients or age keys.

`just ci` MUST be the proof command for ordinary completion claims.

## Required Scaffold

The harness MUST include:

- GitHub Actions workflow under `.github/workflows/ci.yml`.
- Local command scripts under `scripts/`.
- `.vscode/extensions.json` and `.vscode/settings.json`.
- `.editorconfig`.
- `.gitignore`.
- `.gitattributes`.
- `.env.example`.
- `.envrc`.
- `.mise.toml`.
- `devbox.json`.
- `package.json` and lockfile when Node-based tooling is used.
- `pyproject.toml` when Python tooling is used.
- SOPS and age configuration for secret handling.
- A validation script that checks required harness files and command contracts.

The scaffold SHOULD stay small. Add tools only when they remove a real ambiguity or make the proof path more reliable.

## CI Requirements

GitHub Actions MUST:

- Check out the repository.
- Install `just`.
- Install the declared toolchain.
- Restore dependency caches where the tool supports safe caching.
- Install dependencies from lockfiles.
- Run `just ci`.

The workflow MAY contain CI-specific setup, but it SHOULD NOT duplicate lint, test, or format logic already expressed through `just`.

## Dev Harness Documentation

The project MUST include concise operational documentation under `docs/dev-harness/`.

The documentation MUST explain how to use, maintain, and safely change the harness. It SHOULD NOT restate obvious command names unless the command contract matters. It MUST document intent, non-obvious behavior, required workflows, failure recovery, and references needed to make correct changes.

### Documentation Structure

`docs/dev-harness/README.md`
: Entry point. Explain what the harness is, what problem it solves, and the command contract: local `just` commands are the source of truth, and CI calls them where practical.

`docs/dev-harness/explanations/`
: Conceptual documentation for non-obvious design decisions.

Examples:

- `local-ci-parity.md`
- `toolchain-boundaries.md`
- `secrets-model.md`
- `agent-proof-commands.md`

`docs/dev-harness/howto/`
: Task-focused guides for work a developer or agent may need to perform.

Each how-to filename MUST use a verb phrase.

Examples:

- `initialize-dev-env.md`
- `run-local-ci.md`
- `add-a-ci-check.md`
- `change-node-version.md`
- `rotate-secrets-key.md`
- `debug-failing-ci.md`

`docs/dev-harness/references/`
: Stable reference material for configuration, schemas, command contracts, and external docs.

Examples:

- `just-recipes.md`
- `github-actions.md`
- `mise.md`
- `devbox.md`
- `sops-age.md`
- `proof-command-map.md`

### Documentation Rules

How-to documents MUST include:

- Purpose.
- Prerequisites.
- Steps.
- Verification command.
- Common failure modes, when useful.

Explanation documents MUST include:

- Why the design exists.
- What tradeoff it makes.
- What should not be changed casually.

Reference documents MUST include:

- Relevant config files.
- Important fields or schema links.
- Ownership boundaries.
- Commands that prove the config still works.

### Required Initial Documents

The scaffold MUST generate at least:

- `docs/dev-harness/README.md`
- `docs/dev-harness/howto/initialize-dev-env.md`
- `docs/dev-harness/howto/run-local-ci.md`
- `docs/dev-harness/howto/add-a-ci-check.md`
- `docs/dev-harness/howto/debug-failing-ci.md`
- `docs/dev-harness/explanations/local-ci-parity.md`
- `docs/dev-harness/explanations/secrets-model.md`
- `docs/dev-harness/references/just-recipes.md`
- `docs/dev-harness/references/proof-command-map.md`

`just ci` MUST verify that these required documentation files exist.

When a harness command, CI job, tool version, or secrets workflow changes, the matching documentation MUST be updated in the same change.

## Secrets Requirements

The harness MUST support SOPS with age.

Secret handling MUST follow these rules:

- Plaintext secret outputs MUST be ignored by git.
- Local age keys MUST be ignored by git.
- SOPS configuration MUST make intended encrypted paths explicit.
- Secret commands MUST fail clearly when `sops` or required key material is missing.

The spec MUST NOT require real project secrets in the scaffold.

## Proof Command Map

The documentation MUST include a proof-command map that tells humans and agents what command proves each kind of claim.

Minimum map:

| Claim                        | Proof command                    |
| ---------------------------- | -------------------------------- |
| Harness files exist          | `bash tests/validate-harness.sh` |
| Local CI passes              | `just ci`                        |
| Tooling is installed         | `just doctor`                    |
| Formatting is clean          | `just format`                    |
| Static checks pass           | `just lint`                      |
| Tests pass                   | `just test`                      |
| Secret workflow is available | `just secrets-edit <file>`       |

## Validation Requirements

The harness MUST include automated validation for:

- Required files.
- Required `just` recipes.
- CI calling `just ci`.
- Documentation files required by this spec.
- Git ignore rules for local secrets and generated cache files.

Validation SHOULD check contracts, not incidental formatting.

## Implementation Learning Loop

The harness MUST encode lessons that reduce future implementation friction. These lessons are generalizable and should be applied without adding ceremony:

- Make the command contract executable. If a required file, recipe, or doc matters, validate it.
- Separate facts by use. Put tasks in `howto/`, rationale in `explanations/`, and stable contracts in `references/`.
- Prefer one source of truth. CI should call local commands instead of reimplementing them.
- Keep optional tools optional in local diagnostics, but make required proof commands fail clearly.
- Use lockfiles when a command depends on package-installed tools.
- Avoid placeholder success. A completion claim needs a fresh proof command and observed output.
- Fix discovered ambiguity in the spec, not only in the scaffold.

These rules are not an invitation to expand the harness. Add structure only when it improves correctness, repeatability, or recovery from failure.

## Implementation Checklist

- Create the required scaffold files.
- Create the required documentation files.
- Add validation for the scaffold and documentation.
- Run the validation script and observe failure before creating missing required outputs.
- Run `just ci` after implementation.
- Update this spec when implementation reveals a reusable requirement or ambiguity.
