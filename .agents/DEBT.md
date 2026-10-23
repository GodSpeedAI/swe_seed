# Technical Debt & Observations

## Gateway Concurrency Test Timing Race
- Evidence: `cargo test -p swe-seed-core --lib gateway::serve::tests::pool_keeps_governance_counts_exact_under_concurrency` intermittently observes 3 lines instead of 4 if concurrent threads flush to stdout/log file concurrently without newline sync.
- Impact: Flaky CI test on high-concurrency or slow I/O machines.
- Suggested Follow-up: Ensure thread-safe newline delimited writing in the test harness or flush synchronization.
