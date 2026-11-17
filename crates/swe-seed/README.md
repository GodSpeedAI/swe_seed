# swe-seed

## Overview

The `swe-seed` binary crate: the command-line entry point of the SWE Seed governance CLI
(SweSeed → Harness → Fabricator). `Cargo.toml` declares a single binary target
(`[[bin]] name = "swe-seed"`, path `src/main.rs`) and depends on the sibling path crate
`swe-seed-core` for all domain logic, plus `clap` (derive), `anyhow`, and `serde_json`.
This directory holds the package manifest, the `src/` module tree, and a `tests/`
directory of Rust integration tests (excluded from these contracts); every command
family (seed, provenance, route, trace, eval, doctor, host sync/rollback, hooks, skills,
learning, fabricate, run/federation, gateway, gate) is defined and dispatched in `src/`.

## Subdirectories & Boundaries

- src: [./src/README.md](./src/README.md) - CLI binary modules: clap command schema and
  dispatch (`cli.rs`) plus one module per command family; all behavior is delegated to
  `swe-seed-core`, so this crate defines the command surface and exit-code contract only.

## Verification

- Run: `cargo test -p swe-seed`
