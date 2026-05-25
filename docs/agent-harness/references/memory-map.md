# Memory Map

These are the required durable memory artifacts under `.agent-harness/memory/`.

| File                     | Use when                                         | What it should contain                                        |
| ------------------------ | ------------------------------------------------ | ------------------------------------------------------------- |
| `repo-map.md`            | You need to orient quickly to the right layer    | where to start for routing, validation, memory, or docs tasks |
| `decisions.md`           | A stable architectural choice may be reopened    | decisions that should not be casually undone                  |
| `constraints.md`         | A change might broaden scope or add weight       | boundaries on dependencies, memory size, and validation style |
| `failure-patterns.md`    | The harness looks busy but may not move work     | recurring ways the harness fails to produce outcomes          |
| `successful-patterns.md` | A reusable working pattern should guide new work | compact examples of what reliably improves outcomes           |
| `glossary.md`            | Terms need stable operational meaning            | definitions that help route work or choose proof              |
| `open-questions.md`      | A decision is not ripe for contract change yet   | unresolved issues that need evidence or policy                |

## Keep in mind

- Memory is for durable operational guidance, not transcripts.
- Material lessons should cite evidence when possible.
- If a memory artifact does not change future behavior, it is probably not memory.
