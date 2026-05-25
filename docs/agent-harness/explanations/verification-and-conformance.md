# Verification and Conformance

The harness distinguishes work performed, proof gathered, and completion claimed.

That distinction matters because agents are good at producing activity that looks like progress: file edits, long summaries, route labels, and completion-adjacent prose. None of those prove the requested outcome. The harness counters that by attaching each route to explicit proof and by keeping deterministic conformance checks for the harness itself.

There are two related layers here.

The first layer is task proof. Each route card defines required evidence, proof commands, and done conditions. For this repository the default project proof command is `just ci`, but some routes also use focused harness checks such as `python scripts/harness.py validate` or `python scripts/harness.py render-skills`. Route decisions and traces support proof. They do not replace it.

The second layer is harness conformance. These checks prove the harness contract still exists and still moves agents toward outcomes. They are stored under `.agent-harness/evals/` and mirrored by deterministic shell assertions in `tests/validate-harness.sh`.

The main conformance classes are:

- core behavior that must keep working,
- negative cases the harness must reject or catch,
- route conflicts where precedence must stay stable.

This is why a harness change is not done when the docs read well or the route looks plausible. It is done when the harness proves the behavior with executable checks and the repository proof chain passes.
