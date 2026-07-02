# Golden fixtures - Rust harness output

Captured from the Rust `swe-seed` CLI after the Python harness cutover. These serve as
golden targets for stable CLI behavior, modulo the normalization notes below.

## Files

| Fixture                 | Command                                      | Notes                                                                                                                                                                               |
| ----------------------- | -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `route_test.out`        | `swe-seed route "checkpoint smoke"`        | Stable (no `--record`, so no timestamps).                                                                                                                                           |
| `context-plan_test.out` | `swe-seed context-plan "checkpoint smoke"` | Stable.                                                                                                                                                                             |
| `render-skills.out`     | `swe-seed render-skills`                   | Stable.                                                                                                                                                                             |
| `inspect_test.out`      | `swe-seed route "review my recent auth changes for risk"` | Stable route-card inspection fixture.                                                                                                                               |
| `doctor.out`            | `swe-seed doctor`                          | Stable (short).                                                                                                                                                                     |
| `validate.out`          | `swe-seed harness`                         | **Exits 0** ("Harness validation passed") since the three root layer specs now exist.                                                                                                |

## Normalization rules for parity tests

- **Timestamps / trace ids**: any `--record`/`trace`/`eval`/`fabricate` output embeds
  `created_at` (RFC3339) and timestamped `trace_id`s. Parity tests MUST normalize these
  (replace with a fixed token) before comparing. The fixtures here deliberately use the
  non-`--record` forms to stay byte-stable.
- **Absolute paths**: none captured; keep fixtures repo-relative.

## Not yet captured (need setup; capture during the build)

- `eval run --spec <path>` — needs an eval-spec fixture.
- `trace start|checkpoint|finish` — needs a lifecycle; normalize timestamps.
- `fabricate <need>` — needs `.fabricator` config/templates; normalize timestamps.
- `agent-hooks` runtime - needs event payloads on stdin.

## Re-capture

```bash
cargo run -q -p swe-seed -- route "checkpoint smoke" > tests/fixtures/golden/route_test.out
# ...etc per table above
```
