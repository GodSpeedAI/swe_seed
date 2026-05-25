# Successful Patterns

## Use this when

Read this before adding structure. It helps choose simple changes that create real leverage without bloating the harness.

## Keep in mind

Good harness work reduces choices at the moment of action. It should make the next correct move easier than the wrong move.

## Patterns To Reuse

- Make command contracts executable through validation.
- Prefer one source of truth: local `just` recipes define checks; CI calls them.
- Turn abstract categories into route cards with context, work loop, proof, and done conditions.
- Add deterministic fallback behavior even when a future semantic or model-based option may be better.
- Use behavior shaping deliberately: exploit an agent tendency only when it improves outcome production.
- Prefer cheap levers: small validation checks, default commands, generated surfaces, route-card fields, proof gates, and concise memory.
- Keep entry instructions short. Put procedure in route cards, Skill IR, and docs.
- Use how-to, explanation, and reference docs for different reader needs.
- Require generated artifacts to be fresh, not merely present.
- Update the spec before implementation when the contract changes.
- Keep memory small and operational: when to read, what to remember, what to do differently.

## Low-Hanging Fruit

When improving the harness, first look for missing validation, missing next action, ambiguous proof, duplicated command logic, or thin memory. These changes are small and usually improve both agent reliability and developer experience.

Useful behavior-shaping examples:

- Agents over-trust headings, so headings must point to action.
- Agents like completion language, so completion requires proof.
- Agents follow nearby examples, so route cards need positive and negative examples.
- Agents drift into broad context, so routes name required context.
- Agents tolerate hidden drift, so generated artifacts must be freshness-checked.
