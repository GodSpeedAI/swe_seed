# crates

## Overview

Cargo workspace members for the SWE_SEED governance harness. The workspace splits the CLI surface from the domain library: `swe-seed` is a thin clap-based binary that parses commands, resolves the repository root, and delegates every command family to `swe-seed-core`, which owns the routing, trace, eval, hook, learning, seed, gateway, and federation logic. Neither directory holds direct test code; integration tests live under each crate's `tests/` directory and are intentionally outside these contracts.

## Subdirectories & Boundaries

- swe-seed: [./swe-seed/README.md](./swe-seed/README.md) - Binary crate (`[[bin]] swe-seed`). 23 clap subcommands with per-family CLI modules; all domain logic delegates to the sibling core crate.
- swe-seed-core: [./swe-seed-core/README.md](./swe-seed-core/README.md) - Library crate. 17 subsystem modules (adapters, config, context, contracts, doctor, eval, fabricator, federation, gateway, hooks, learning, provenance, route, security, seed, skill, trace) plus the routing gate, hash-chained trace ledger, harness validation, and shared utilities.
