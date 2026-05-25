#!/usr/bin/env bash
set -euo pipefail

backend="${1:-both}"

case "$backend" in
  rusql|ruvector|both)
    ;;
  *)
    echo "usage: bash scripts/plan-learning-store.sh [rusql|ruvector|both]" >&2
    exit 1
    ;;
esac

print_common_header() {
  cat <<'EOF'
Optional Learning Store Plan

Outcome to improve:
- faster recovery from prior traces and learning-review packets,
- cheaper lookup than transcript archaeology,
- optional indexing that never replaces the filesystem source of truth.

Guardrails:
- keep the current filesystem-first baseline intact,
- make database-backed indexing optional,
- add `rusql` before `ruvector`, because exact and structured recall pays off earlier than semantic retrieval,
- only turn on retrieval layers that reduce recovery time or improve proof quality.

EOF
}

print_rusql_plan() {
  cat <<'EOF'
Phase 1: Optional rusql mirror

Outcomes enabled:
- query finished traces, checkpoints, and learning-review packets without scanning many JSON files,
- faster local search over route, job type, verification status, unresolved risk, and provenance,
- a stable substrate for later retrieval and reporting without changing the canonical filesystem artifacts.

Implementation shape:
1. Keep `.agent-harness/traces/records/*.json` and reflection templates as the canonical source.
2. Add a mirror command that indexes trace records and learning-review packets into a local SQLite-compatible store.
3. Index only durable metadata first: trace id, task, route, verification status, checkpoint summary, unresolved risks, and provenance paths.
4. Add proof that the mirror can be rebuilt from the filesystem at any time.

Proof gates before enabling:
- mirror rebuild succeeds from the repository artifacts alone,
- queries return the same identifiers the filesystem scan would return,
- no completion or routing logic depends on the database being present.

Suggested first command after adoption:
- `just harness-sync-learning-store`

EOF
}

print_ruvector_plan() {
  cat <<'EOF'
Phase 2: Optional ruvector retrieval

Outcomes enabled:
- semantic lookup over distilled learning packets when exact matching is too weak,
- better recall for similar failures, reusable lessons, and related skill-update candidates,
- lower recovery time once the learning corpus is large enough to create search pressure.

Implementation shape:
1. Build on the optional `rusql` mirror or an equivalent local index; do not skip the structured layer.
2. Embed only distilled artifacts: learning-review summaries, candidate updates, and selected memory notes.
3. Keep retrieval advisory. The agent still reads canonical files before mutating the harness.
4. Validate that semantic retrieval improves selection quality instead of just adding another store.

Proof gates before enabling:
- exact or structured search is already showing pressure,
- semantic retrieval returns better candidate artifacts than filename or grep search on a representative eval set,
- missing vector index does not break routing, verification, or learning capture.

Suggested first command after adoption:
- `just harness-eval-learning-retrieval`

EOF
}

print_rollout_plan() {
  cat <<'EOF'
Rollout order

1. Start with filesystem-only trace and learning-review distillation.
2. Add optional `rusql` mirroring only when local retrieval pressure is real.
3. Add optional `ruvector` retrieval only when structured search no longer covers the failure modes.
4. Keep every layer rebuildable from files and removable without breaking the harness contract.

Current recommendation:
- plan `rusql` first,
- defer `ruvector` until the stored learning corpus is large enough to justify semantic retrieval.
EOF
}

print_common_header

case "$backend" in
  rusql)
    print_rusql_plan
    ;;
  ruvector)
    print_ruvector_plan
    ;;
  both)
    print_rusql_plan
    print_ruvector_plan
    print_rollout_plan
    ;;
esac
