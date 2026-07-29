# Debt

## gateway serve: single-worker concurrency (2026-07-29)

`gateway/serve.rs` `serve_loop` handles ONE connection at a time (bounded concurrency = 1).
The breaker + governance already serialize the risky paths, so this is safe and correct for
v0.1, but it caps throughput. Evidence: `serve_loop` accepts then fully handles one connection
before looping. Impact: a slow backend blocks all other in-flight requests. Upgrade path: spawn
one thread per accepted connection (or a small bounded pool), keeping `Governance`/`AuditWriter`
(both `Send + Sync` via internal locks) shared. Add a load test asserting N concurrent calls each
complete under a parallel slow backend before raising the cap. Add when: real multi-client load
appears or a latency budget is set for the gateway.

## gateway serve: discovery is declared-catalog only (2026-07-29)

`tools/list` returns the local compact catalog projection (`project_catalog` over declared
`MCPServer.catalog`); it does not yet issue live `tools/list` to each backend to augment the
catalog (spec 0020 §7 mentions live discovery). Impact: dynamically-added backend tools are not
visible until declared in config. Evidence: `serve.rs::dispatch` `tools/list` arm calls
`list_catalog` locally. Upgrade path: fan-out live discovery to registered stdio/HTTP backends,
merge with the declared baseline, dedupe by namespaced name. Add when: a backend advertises tools
not pre-declared in its config.

