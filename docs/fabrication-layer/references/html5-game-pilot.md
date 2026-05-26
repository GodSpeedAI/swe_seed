# HTML5 Game Pilot

The reference pilot product type is `single_page_html5_game`.

The generator emits:

- a semantic-chain packet under `.fabricator/runs/<run_id>/generated/`
- a playable local prototype at `.fabricator/runs/<run_id>/prototype/index.html`
- static proof results under `.fabricator/runs/<run_id>/proof/`

Current proof checks cover:

- single local file with no network calls
- keyboard movement logic
- collision and scoring logic
- HUD and timer rendering
- end-state and restart flow

This is intentionally a v0.1 reference implementation: small enough to regenerate from the root
specs, strong enough to prove the semantic chain and bounded handoff are executable.
