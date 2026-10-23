# Portability And Installation Inspection - 2026-07-02

Scope: inspect whether this repository is portable and determine the current installation process.

Route: intended `review`/inspection route. `swe-seed route` was unavailable on `PATH`; `just harness-route` and `just harness-context-plan` were attempted as the local wrapper path.

## Verdict

The project is not currently verifiably portable because the Rust CLI does not compile. The intended installation path is a source checkout with `just bootstrap`, then `just doctor`, then `just ci`, but the documented command surface and route proof commands still reference removed Python files.

## Installation Process Observed

1. Install required tools:
   - `git`
   - `just`
   - Rust/Cargo, workspace `rust-version = "1.75"`
2. Recommended tools:
   - `mise`, `pnpm`, `uv`, `age`, `direnv`, `devbox`, `sops`
3. Clone repository.
4. Run `just bootstrap`.
   - `scripts/bootstrap.sh` runs `mise install` when available.
   - It runs `pnpm install --frozen-lockfile` when `pnpm-lock.yaml` exists.
   - It runs `uv sync` when `uv` exists.
5. Run `just doctor`.
6. Run `just ci`.
7. For direct CLI use, the current Rust surface is `cargo run -q -p swe-seed -- <command>` or a release binary from `cargo build --release -p swe-seed`.

## Findings

1. Build blocker: `cargo check -q` fails in `crates/swe-seed/src/federation_cli.rs:252` because `Envelope::namespace` is a method, not a field. This prevents `just harness-route`, `just harness-context-plan`, `cargo run -p swe-seed -- ...`, and release builds.

2. Documentation drift: `README.md` still documents `python scripts/harness.py ...`, but `scripts/harness.py` is no longer present. The same removed command appears throughout docs, route cards, eval notes, and generated adapter code.

3. Route proof drift: `.agent-harness/routes/review.json` and `.agent-harness/routes/research.json` still list `python scripts/harness.py validate` as proof. That command cannot run because the file is absent.

4. Package script drift: `package.json` has `"test": "bash tests/validate-harness.sh"`, but `tests/validate-harness.sh` is absent.

5. Full local CI is currently masked by formatting failure before Rust tests: `just ci` stops on Prettier warnings for `.omc/sessions/1cea5bed-7171-4ef2-8b8d-5d715e0bd880.json`, `.serena/project.yml`, and `README.md`. `.omc/sessions/...` and `.serena/project.yml` are tracked, so this can affect a clean clone.

## Proof Commands Run

- `just doctor` passed: required commands are available.
- `just harness-route "inspect the codebase to verify the project is portable and determine the installation process"` failed during Rust compile with E0615.
- `just harness-context-plan "inspect the codebase to verify the project is portable and determine the installation process"` failed during Rust compile with E0615.
- `cargo check -q` failed with E0615 at `crates/swe-seed/src/federation_cli.rs:252`.
- `just ci` failed at Prettier before reaching Rust tests.
- `find scripts ...` confirmed `scripts/harness.py` is absent.
- `find tests ...` confirmed `tests/validate-harness.sh` is absent.

## Recommended Next Actions

1. Fix the Rust compile error at `crates/swe-seed/src/federation_cli.rs:252`, likely by using `envelope.namespace()`.
2. Replace stale `python scripts/harness.py ...` and `tests/validate-harness.sh` references in README, route cards, package scripts, docs, evals, and generated adapter templates with the Rust CLI or `just` recipes.
3. Decide whether tracked local metadata under `.omc/` and `.serena/` should be formatted, ignored by Prettier, or removed from tracked source.
4. Re-run `just ci` and `just parity`.
