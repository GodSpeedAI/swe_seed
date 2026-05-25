#!/usr/bin/env bash
set -euo pipefail

required_files=(
  ".agent-harness/config.yaml"
  ".agent-harness/context/README.md"
  ".agent-harness/context/budget-policy.yaml"
  ".agent-harness/context/context-mode-normalization.md"
  ".agent-harness/evals/core-conformance.md"
  ".agent-harness/evals/negative-conformance.md"
  ".agent-harness/evals/route-conflicts.md"
  ".agent-harness/hooks/hook-router.sh"
  ".agent-harness/imports/README.md"
  ".agent-harness/imports/9arm-skills-normalization.md"
  ".agent-harness/imports/anthropic-skills-skill-creator-normalization.md"
  ".agent-harness/imports/hermes-agent-normalization.md"
  ".agent-harness/memory/constraints.md"
  ".agent-harness/memory/decisions.md"
  ".agent-harness/memory/failure-patterns.md"
  ".agent-harness/memory/glossary.md"
  ".agent-harness/memory/open-questions.md"
  ".agent-harness/memory/repo-map.md"
  ".agent-harness/memory/successful-patterns.md"
  ".agent-harness/playbooks/README.md"
  ".agent-harness/reflections/harness-improvement-proposals.md"
  ".agent-harness/reflections/learning-review-template.yaml"
  ".agent-harness/reflections/reflection-template.yaml"
  ".agent-harness/render-targets/checklists/debug-discipline.md"
  ".agent-harness/render-targets/copilot/debug-discipline.instructions.md"
  ".agent-harness/render-targets/hooks/debug-discipline.prompt.md"
  ".agent-harness/routes/bugfix.json"
  ".agent-harness/routes/documentation.json"
  ".agent-harness/routes/harness_improvement.json"
  ".agent-harness/routes/implementation.json"
  ".agent-harness/routes/refactor.json"
  ".agent-harness/routes/release.json"
  ".agent-harness/routes/research.json"
  ".agent-harness/routes/review.json"
  ".agent-harness/routes/skill_authoring.json"
  ".agent-harness/routes/spec.json"
  ".agent-harness/routes/test.json"
  ".agent-harness/skills/40-debug/debug-discipline.json"
  ".agent-harness/traces/traceability-template.yaml"
  ".agent-harness/traces/README.md"
  ".agent-harness/traces/route-decisions/README.md"
  ".github/workflows/ci.yml"
  ".github/copilot-instructions.md"
  ".vscode/extensions.json"
  ".vscode/settings.json"
  "AGENTS.md"
  ".editorconfig"
  ".env.example"
  ".envrc"
  ".gitattributes"
  ".gitignore"
  ".mise.toml"
  "devbox.json"
  "docs/dev-harness/README.md"
  "docs/dev-harness/explanations/local-ci-parity.md"
  "docs/dev-harness/explanations/secrets-model.md"
  "docs/dev-harness/howto/add-a-ci-check.md"
  "docs/dev-harness/howto/debug-failing-ci.md"
  "docs/dev-harness/howto/initialize-dev-env.md"
  "docs/dev-harness/howto/run-local-ci.md"
  "docs/dev-harness/references/just-recipes.md"
  "docs/dev-harness/references/proof-command-map.md"
  "docs/agent-harness/README.md"
  "docs/agent-harness/explanations/routing-and-proof.md"
  "docs/agent-harness/explanations/context-and-continuity.md"
  "docs/agent-harness/explanations/verification-and-conformance.md"
  "docs/agent-harness/explanations/hooks-memory-and-learning.md"
  "docs/agent-harness/howto/start-a-task.md"
  "docs/agent-harness/howto/run-harness-checks.md"
  "docs/agent-harness/howto/capture-traces.md"
  "docs/agent-harness/howto/debug-routing.md"
  "docs/agent-harness/howto/extend-the-harness.md"
  "docs/agent-harness/howto/add-a-route-card.md"
  "docs/agent-harness/howto/add-a-conformance-eval.md"
  "docs/agent-harness/howto/inspect-the-harness.md"
  "docs/agent-harness/howto/render-skills-and-targets.md"
  "docs/agent-harness/references/cli-recipes.md"
  "docs/agent-harness/references/route-map.md"
  "docs/agent-harness/references/artifact-map.md"
  "docs/agent-harness/references/eval-map.md"
  "docs/agent-harness/references/hook-events.md"
  "docs/agent-harness/references/memory-map.md"
  "docs/specs/agentic-swe-harness.md"
  "docs/specs/skill-ir.md"
  "docs/specs/verification-system.md"
  "docs/specs/memory-system.md"
  "docs/specs/hook-strategy.md"
  "justfile"
  "package.json"
  "pyproject.toml"
  "scripts/harness.py"
  "scripts/query-learning-store.sh"
  "scripts/eval-learning-retrieval.sh"
  "scripts/plan-learning-store.sh"
  "scripts/sync-learning-store.sh"
  "scripts/bootstrap.sh"
  "scripts/ci.sh"
  "scripts/doctor.sh"
  "scripts/secrets-decrypt.sh"
  "scripts/secrets-edit.sh"
  "scripts/secrets-encrypt.sh"
  "scripts/secrets-rotate-key.sh"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "Missing required file: $file" >&2
    exit 1
  fi
done

grep -qE '^ci:' justfile
grep -qE '^doctor:' justfile
grep -qE '^secrets-encrypt:' justfile
grep -qE '^secrets-decrypt:' justfile
grep -qE '^secrets-edit file:' justfile
grep -qE '^secrets-rotate-key:' justfile
grep -qE '^harness-validate:' justfile
grep -qE '^harness-doctor:' justfile
grep -qE '^harness-render-skills:' justfile
grep -qE '^harness-route task:' justfile
grep -qE '^harness-route-record task:' justfile
grep -qE '^harness-inspect item:' justfile
grep -qE '^harness-context-plan task:' justfile
grep -qE '^harness-trace-start task:' justfile
grep -qE '^harness-trace-distill trace:' justfile
grep -qE '^harness-sync-learning-store db_path="":' justfile
grep -qE '^harness-query-learning-store mode="summaries" limit="10" status="any" job_type="any" db_path="":' justfile
grep -qE '^harness-eval-learning-retrieval db_path="":' justfile
grep -qE '^harness-plan-learning-store backend="both":' justfile
grep -q 'just ci' .github/workflows/ci.yml
grep -q 'pnpm/action-setup' .github/workflows/ci.yml
grep -q 'astral-sh/setup-uv' .github/workflows/ci.yml
grep -q 'AGENTS.md' .github/copilot-instructions.md
grep -q '.agent-harness/routes/' AGENTS.md
grep -q 'sops' .gitignore
grep -q 'SWE_SEED' package.json
grep -q 'swe-seed' pyproject.toml
python scripts/harness.py validate
route_output=$(python scripts/harness.py route "fix a failing regression test")
grep -q '"job_type": "bugfix"' <<<"$route_output"
route_output=$(python scripts/harness.py route "the login form is broken")
grep -q '"route_card": ".agent-harness/routes/bugfix.json"' <<<"$route_output"
route_output=$(python scripts/harness.py route "implement HARNESS_SPEC.md semantic router")
grep -q '"route_card": ".agent-harness/routes/harness_improvement.json"' <<<"$route_output"
route_output=$(python scripts/harness.py route "incorporate context-mode mechanisms")
grep -q '"route_card": ".agent-harness/routes/harness_improvement.json"' <<<"$route_output"
route_output=$(python scripts/harness.py route "Let's make a react todo list")
grep -q '"route_card": ".agent-harness/routes/spec.json"' <<<"$route_output"
route_output=$(python scripts/harness.py route "review my recent auth changes for risk")
grep -q '"route_card": ".agent-harness/routes/review.json"' <<<"$route_output"
inspect_output=$(python scripts/harness.py inspect debug-discipline)
grep -q '"skills"' <<<"$inspect_output"
context_plan_output=$(python scripts/harness.py context-plan "implement a parser change")
grep -q '"context_budget"' <<<"$context_plan_output"
grep -q '## Bundled resources' .agent-harness/render-targets/claude/debug/debug-discipline/SKILL.md
grep -q '## Evaluation prompts' .agent-harness/render-targets/claude/debug/debug-discipline/SKILL.md
trace_id=$(python scripts/harness.py trace start "checkpoint smoke" | python -c 'import json,sys; print(json.load(sys.stdin)["trace_id"])')
trace_output=$(python scripts/harness.py trace checkpoint "$trace_id" --stage change --summary "spec delta captured" --next-action "run targeted validation" --artifact HARNESS_SPEC.md --risk "proof not run")
grep -q '"trace.checkpoint"' <<<"$trace_output"
trace_output=$(python scripts/harness.py trace resume "$trace_id")
grep -q '"latest_checkpoint"' <<<"$trace_output"
trace_output=$(python scripts/harness.py trace distill "$trace_id")
grep -q '"learning_review"' <<<"$trace_output"
grep -q '"provenance"' <<<"$trace_output"
learning_store_plan=$(bash scripts/plan-learning-store.sh both)
grep -q 'Phase 1: Optional rusql mirror' <<<"$learning_store_plan"
grep -q 'Phase 2: Optional ruvector retrieval' <<<"$learning_store_plan"
sync_output=$(bash scripts/sync-learning-store.sh)
grep -q 'learning_store_db' <<<"$sync_output"
grep -q 'backend' <<<"$sync_output"
query_output=$(bash scripts/query-learning-store.sh)
grep -q 'mode' <<<"$query_output"
grep -q 'results' <<<"$query_output"
retrieval_eval_output=$(bash scripts/eval-learning-retrieval.sh)
grep -q 'vector_readiness' <<<"$retrieval_eval_output"
grep -q 'recommendation' <<<"$retrieval_eval_output"

echo "Harness validation passed"
