# Cognitive Representations

A focused set of diagrams and interactive explainers for the parts of SWE_SEED that are hardest to reason about from prose alone. Each entry states the question it answers, what makes that question hard, and the source evidence behind it.

These pages complement the canonical documentation. They do not replace it: the subsystem pages remain authoritative for behaviour, and every representation links back to them.

---

## Why these three

SWE_SEED has many subsystems. Three behaviours account for most of the confusion when someone first tries to predict what the tool will do:

| Behaviour             | Why it is hard to predict                                                       | Representation                                                             |
| --------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Route selection       | Two cards can tie, and a zero score can still yield `confidence: medium`        | [Routing Explorer](routing-explorer.html)                                  |
| What the gate detects | Two edits produce two different failures, and truncation is not detected at all | [Trace Chain Explorer](trace-chain-explorer.html)                          |
| Decision branches     | The fallback path is invisible in the emitted JSON                              | Diagram in [Routing Engine](../subsystems/routing-engine.md#decision-path) |

Everything else in the harness is either a straightforward file transformation or is already well served by the existing sequence diagrams.

---

## 1. Routing Explorer

**Reader question.** Why did this task route to that job type, and how close was the runner-up?

**Before this page.** The scoring rule lived only in `route/mod.rs`. The subsystem page listed the weights but did not show them applied, so readers could not tell whether a route won by a wide margin or by a tie-break, and could not see that ties resolve alphabetically by card id.

**What it makes perceptible.** All eleven cards score simultaneously for any task you type. Each bar is the card's total, split into trigger, example, and job-type contributions. Selecting a bar shows which triggers actually fired and whether they matched as a substring or as a token overlap.

**Medium.** Interactive HTML, because the interesting behaviour only appears across many inputs. A static diagram cannot show that `add a new host adapter for zed` resolves to `test` by tie-break.

**Evidence.** Weights and fallback conditions transcribed from `crates/swe-seed-core/src/route/mod.rs`. Card text copied from `.agent-harness/routes/*.json`. Output agreement checked against `swe-seed route` on nine inputs, including the tie, the fallback, and the no-match case.

**Success test.** After using the page, a reader can explain why two equal scores still produce a definite winner, and why `confidence: medium` does not imply a strong content match.

---

## 2. Trace Chain Explorer

**Reader question.** If one ledger entry is edited, which entry does the gate flag, and what does it miss?

**Before this page.** The hash formula was documented incorrectly (as `sha256(prev_hash + payload)`), and no page distinguished the two failure modes: an entry failing its own hash versus a later entry failing its link. The fact that trailing entries can be deleted without detection was not stated anywhere.

**What it makes perceptible.** A real four-entry chain, with every hash recomputed live in the browser. You can apply two different edits. Editing the payload alone fails at that entry with `event or payload tampered`. Editing the payload _and_ rewriting that entry's hash shifts the failure to the next entry with `prev_hash broken (chain links severed)`. A note records the third case: deleting the last entry passes.

**Medium.** Interactive HTML, because the distinction only becomes obvious when you can make both edits and compare which entry the gate names.

**Evidence.** The four entries were produced by `trace start`, `trace append`, `trace checkpoint`, and `trace finish` in a throwaway root. The gate wording and both block messages were captured from the binary. The truncation result was captured by deleting the last row and re-running `gate --verify`, which exited 0. The page's SHA-256 implementation reproduces all four stored hashes.

**Success test.** After using the page, a reader can say why rewriting a hash does not hide the edit, and why `gate --verify` proves consistency of the surviving rows rather than completeness.

---

## Conventions

- **Offline.** Both pages are single files with inline CSS, JavaScript, and data. They open from `file://` with no server, no CDN, and no network access.
- **Reduced motion.** Animations are disabled when the reader requests reduced motion.
- **Keyboard.** Score rows and ledger entries are focusable and respond to `Enter`, `Space`, and arrow keys.
- **No side effects.** Interaction state lives in memory only. Neither page writes files or calls the CLI.
- **Fidelity labels.** Each page states in its header what it models. The Routing Explorer runs an exact port of `score_route()`, `infer_bootstrap_route()`, and the ranking comparator. The Trace Chain Explorer runs an exact port of `verify_chain()`'s two checks and stops-at-first-failure order. Neither page models truncation, which the gate does not detect either.

---

## Maintaining these pages

The data each page depends on is small and stated in the page itself:

| Page                 | Depends on                                                                           | Re-verify when                                                      |
| -------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------- |
| Routing Explorer     | The eleven route cards and the weights in `score_route()`                            | A card's triggers, examples, or the scoring weights change          |
| Trace Chain Explorer | `hash_of()` and `verify_chain()` in `trace_ledger.rs`, and the gate's block messages | The hash inputs, the verification order, or the gate wording change |

If a dependency changes, re-run the agreement check described under each page's Evidence section before trusting the page again.
