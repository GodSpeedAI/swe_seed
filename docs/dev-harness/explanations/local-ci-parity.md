# Local CI Parity

Remote CI should not define a second test system. The local harness defines the checks, and CI runs the harness.

This design makes failures reproducible. A developer or agent can run `just ci`, see the same command surface as GitHub Actions, and fix failures before pushing.

The tradeoff is that CI setup still exists. GitHub Actions must install tools and dependencies before it can call `just ci`. That setup should stay thin. Do not copy lint, format, or test logic into workflow YAML when the same logic can live behind `just`.

Change this carefully. If a check runs only in CI, document why local execution is impossible and add a local substitute when practical.
