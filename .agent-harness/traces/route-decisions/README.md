# Route Decision Ledger

## Use this when

Use this when the semantic router's choice matters to the task outcome or when a later agent may need to audit why a route was selected.

## Route decision

A Route decision record is written by:

```bash
python scripts/harness.py route --record "task text"
```

The record stores the selected job type, route card path, confidence, required context, required skills, work loop, proof commands, and next action. It does not replace execution. It starts execution.

## Trace record relationship

`python scripts/harness.py trace start "task text"` creates both a route decision and a Trace record. The trace file points back to the decision so routing, work notes, and verification remain connected.

## Do not store secrets

Do not store secrets, tokens, private keys, or sensitive customer data here. If a secret affected the route, record only the safe fact, such as "secret unavailable" or "encrypted env file missing."

## Done when

The ledger is useful when route ambiguity, conflict, or fallback can be explained from the record without guessing.
