# Eval Map

Use this map when you know the harness needs a deterministic check but you are not sure which eval file should hold it.

| Eval file                                      | Use when                                                   | Typical shape                              |
| ---------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------ |
| `.agent-harness/evals/core-conformance.md`     | A core positive behavior must keep working                 | command, expected output, why it matters   |
| `.agent-harness/evals/negative-conformance.md` | A known breakage should be rejected or detected            | breakage, expected failure, why it matters |
| `.agent-harness/evals/route-conflicts.md`      | Two plausible routes need a stable precedence rule         | prompt, expected route, precedence rule    |
| `just harness-validate`                    | The behavior is cheap to assert deterministically in shell | file presence or CLI output assertion      |

## Selection rule

- If the harness must prove a happy-path capability, start with core conformance.
- If the harness must catch a bad state, start with negative conformance.
- If the problem is route ambiguity, start with route conflicts.
- If the assertion is simple and cheap in shell, mirror it in `just harness-validate`.

Many important changes need both an eval entry and a shell assertion.
