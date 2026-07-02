# Route Map

Each job type matters only because it selects a route card.

| Route                 | Use when                                                           | First move                                                                 | Default proof                             |
| --------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------------- | ----------------------------------------- |
| `bugfix`              | A defect or broken behavior must be diagnosed and fixed            | establish reliable reproduction or document the missing artifact or access | `just ci`                                 |
| `documentation`       | A reader-facing document must help someone act correctly           | identify the reader and action the doc must support                        | `just ci`                                 |
| `harness_improvement` | The harness itself must change                                     | identify the harness gap and desired behavior                              | `just harness-validate`      |
| `implementation`      | A stated requirement must become behavior                          | identify the governing requirement                                         | `just harness-validate`      |
| `refactor`            | Structure should improve without changing intended behavior        | identify preserved behavior and the reason structure should change         | `just ci`                                 |
| `release`             | A versioned release must be prepared or published                  | confirm release scope and release target                                   | `just ci`                                 |
| `research`            | Information is needed before choosing a change                     | define the decision or question research must support                      | `just harness-validate`      |
| `review`              | Changes need risk analysis, not immediate edits                    | identify the changed contract or intended outcome                          | `just harness-validate`      |
| `skill_authoring`     | A reusable behavioral skill must be created or revised             | define the JTBD and trigger conditions                                     | `just harness-render-skills` |
| `spec`                | The source-of-truth contract must be defined before implementation | identify the desired outcome                                               | `just ci`                                 |
| `test`                | Verification must be added or repaired                             | identify behavior under test                                               | `just ci`                                 |

Read the full route card under `.agent-harness/routes/` before material work. The table is an index, not the procedure.
