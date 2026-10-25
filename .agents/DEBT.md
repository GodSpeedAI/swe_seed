# Technical Debt & Observations

## Gateway Concurrency Test Timing Race
- Evidence: `cargo test -p swe-seed-core --lib gateway::serve::tests::pool_keeps_governance_counts_exact_under_concurrency` intermittently observes 3 lines instead of 4 if concurrent threads flush to stdout/log file concurrently without newline sync.
- Impact: Flaky CI test on high-concurrency or slow I/O machines.
- Suggested Follow-up: Ensure thread-safe newline delimited writing in the test harness or flush synchronization.

### .gitignore `reference/` pattern hides `docs/reference/`
- Date: 2026-10-24
- Location: `.gitignore:50`
- Debt: The bare `reference/` pattern matches any directory named `reference` at any depth, so `docs/reference/` (four documentation reference pages, including `cli.md`) is untracked. Edits to those pages are invisible to `git status` and cannot be committed without `git add -f`. Documentation fixes silently fail to ship.
- Evidence: `git check-ignore -v docs/reference/cli.md` returns `.gitignore:50:reference/`. `git ls-files docs/reference/` returns nothing. `docs/documentation-map.md` still catalogs `reference/cli.md` as a canonical page.
- Confidence: high
- Closure: Change `.gitignore:50` from `reference/` to `/reference/` (or whatever the intended scope was), then `git add docs/reference/`. Verify no unintended files appear. Files touched: `.gitignore`, plus four staged additions under `docs/reference/`. Size S.

### Flaky concurrency test in gateway serve
- Date: 2026-10-24
- Location: `crates/swe-seed-core/src/gateway/serve.rs` (`gateway::serve::tests::pool_keeps_governance_counts_exact_under_concurrency`)
- Debt: The test fails intermittently when run as part of the full `scripts/ci.sh test` suite, which makes `just ci` non-deterministic. A flaky proof command undermines the harness rule that a completion claim requires a passing proof gate.
- Evidence: `bash scripts/ci.sh test` reported `test result: FAILED. 87 passed; 1 failed` for this test; re-running the single test alone passed immediately. No source under test changed between runs.
- Confidence: high
- Closure: Identify the shared-state or timing assumption in the pool test and make it deterministic, or serialize it. File touched: `gateway/serve.rs` tests. Size M.

### `gate --verify` does not detect ledger truncation
- Date: 2026-10-24
- Status: Resolved. Each append writes a head anchor (`length`, `head`) to `ledger.anchors.json`, and verification compares the chain to it. Anchors are Ed25519-signed with a key outside the ledger directory, and appends are two-phase (pending before COMMIT, finalized after). Remaining limits: a process running as the key's uid can read the key and forge anchors; a trailing row committed but not yet finalized can be removed undetected until the next append; legacy traces with no anchor report `unanchored`, and an unsigned legacy anchor file reports `unsigned (legacy)` and is signed only by the operator command `swe-seed trace adopt-anchors` (trust-on-first-use), never automatically.
- Location: `crates/swe-seed-core/src/trace_ledger.rs` (`verify_chain`)
- Debt: `verify_chain` has no expected-length or head anchor, so deleting trailing `chain_entries` rows leaves a shorter prefix that still verifies and `gate --verify` exits 0. A trace can therefore be silently shortened to hide later events, which weakens the tamper-evidence claim the gate is relied on for.
- Evidence: Deleting the last row of a three-entry chain in a throwaway root, then running `swe-seed gate <id> --verify`, printed `allow: trace '<id>' routed + chain verifies` and exited 0.
- Confidence: high
- Closure: Record the expected entry count or chain root outside the ledger and compare on verify, or anchor the root via federation signing. Files touched: `trace_ledger.rs`, `routing_gate.rs`, possibly the trace record schema. Size M.
