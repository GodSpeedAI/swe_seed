# EGM Drift Report - 2026-10-30

Target: the 24 EGM contracts generated/corrected by `egm-bootstrap` on this date
(root `README.md` Repository Blueprint block, `crates/README.md`, `crates/swe-seed{,/src}/README.md`,
`crates/swe-seed-core{,/src,+17 modules}/README.md`, `scripts/README.md`).

Method: `egm-drift-report` skill applied to the modified contracts. Each contract was
read in full and every cited symbol/invariant/side effect was verified against the
source files in the same directory. Three sequential drift agents (root + CLI + scripts;
core modules adapters through eval; core modules fabricator through trace). Divergences
only, per skill format. Findings were fixed in the contracts the same day; resolution
noted inline.

## /home/sprime01/projects/SWE_SEED/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed/README.md

- Overview claimed the directory "holds only the package manifest and the `src/` module
  tree", but the directory also contains `tests/` with 7 integration test files
  (`cli_golden.rs`, `cr_review_cli.rs`, `cr_review_learning_cli.rs`, `eval_proof_cli.rs`,
  `host_cli.rs`, `learning_cli.rs`, `trace_cli.rs`). Fixed: overview now names the
  `tests/` directory and its exclusion from the contracts.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed/src/README.md

- `federation keygen` side effect claimed the public key is written "alongside" the
  private key. Code writes the public key to `.agent-harness/federation/keys/<id>.pub`
  (`crates/swe-seed-core/src/federation/signing.rs:86-91`) and only the private key to
  `.swe-seed/federation/keys/<id>.key` (`signing.rs:94-99`), matching
  `federation_cli.rs:23`. Fixed.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/README.md

- Global Side Effects classified the gateway governance journal and audit log as SQLite
  databases. Both are JSONL append logs: `.swe-seed/gateway/governance.journal`
  (`gateway/governance.rs:17-18`, fsynced appends at `:122-125`) and the audit JSONL
  (`gateway/audit.rs:17-18`, `:116-125`); no rusqlite usage exists under `src/gateway/`.
  Fixed: journal and audit now described as fsynced/line-atomic JSONL, not SQLite.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/README.md

- Interface said "17 subsystem modules plus the 5 root-level files" with a 21-name
  parenthetical. `lib.rs:8-28` declares 21 `pub mod` items: 17 subsystem modules plus 4
  root-level modules (`harness_validate`, `routing_gate`, `trace_ledger`, `util`);
  `lib.rs` itself is not a `pub mod`. Fixed: count corrected to 4, parenthetical kept.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/adapters/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/config/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/context/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/contracts/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/doctor/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/eval/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/fabricator/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/gateway/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/hooks/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/learning/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/provenance/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/route/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/security/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/seed/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/skill/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/trace/README.md

- No drift detected.

## /home/sprime01/projects/SWE_SEED/scripts/README.md

- No drift detected.

## Summary

24 contracts checked, 4 drift findings (2 CLI-side, 2 core-side), all verified against
code and fixed in the contracts on the same day. 20 contracts clean on first pass.
