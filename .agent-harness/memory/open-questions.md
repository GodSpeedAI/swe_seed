# Open Questions

## Use this when

Read this when a task stalls because the harness lacks policy, scope, or prioritization. Use it to decide whether to ask the user, record an assumption, or make a small reversible choice.

## Keep in mind

Open questions are not a parking lot for every thought. Keep only questions that can change future implementation, routing, validation, or developer experience.

## Active Questions

- Which real agent runtimes should receive first-class rendered instruction targets beyond Copilot, hook prompts, and checklists?
- Should traces remain file-based, or should they move to SQLite after real usage creates enough retrieval pressure?
- Should route cards remain JSON for dependency-light validation, or move to YAML for authoring ergonomics once a parser dependency is acceptable?
- What minimum traceability artifact should agents write after implementation work in this repository?
- Which route cards deserve dedicated skills next: `implementation`, `harness_improvement`, or `documentation`?

## Resolution Rule

When an answer becomes clear through implementation, move the result to `decisions.md` or `successful-patterns.md`. If an answer reveals a repeatable trap, move it to `failure-patterns.md`.
