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
  ".agent-harness/evals/schemas/eval-artifacts.schema.yaml"
  ".agent-harness/baml/MIGRATION_NOTE.md"
  ".agent-harness/baml/SEMANTIC_CHAIN_V0_1_BACKLOG.md"
  ".agent-harness/baml/V0_1_RELEASE_CRITERIA.md"
  ".agent-harness/baml/baml_src/harness.baml"
  ".agent-harness/baml/baml_src/swe_seed.baml"
  ".agent-harness/baml/baml_src/fabricator.baml"
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
  ".agent-harness/render-targets/checklists/plan-and-frame.md"
  ".agent-harness/render-targets/checklists/implement-with-proof.md"
  ".agent-harness/render-targets/checklists/test-with-proof.md"
  ".agent-harness/render-targets/checklists/review-for-risk.md"
  ".agent-harness/render-targets/checklists/verify-before-completion.md"
  ".agent-harness/render-targets/checklists/capture-learning.md"
  ".agent-harness/render-targets/checklists/debug-discipline.md"
  ".agent-harness/render-targets/copilot/plan-and-frame.instructions.md"
  ".agent-harness/render-targets/copilot/implement-with-proof.instructions.md"
  ".agent-harness/render-targets/copilot/test-with-proof.instructions.md"
  ".agent-harness/render-targets/copilot/review-for-risk.instructions.md"
  ".agent-harness/render-targets/copilot/verify-before-completion.instructions.md"
  ".agent-harness/render-targets/copilot/capture-learning.instructions.md"
  ".agent-harness/render-targets/copilot/debug-discipline.instructions.md"
  ".agent-harness/render-targets/hooks/plan-and-frame.prompt.md"
  ".agent-harness/render-targets/hooks/implement-with-proof.prompt.md"
  ".agent-harness/render-targets/hooks/test-with-proof.prompt.md"
  ".agent-harness/render-targets/hooks/review-for-risk.prompt.md"
  ".agent-harness/render-targets/hooks/verify-before-completion.prompt.md"
  ".agent-harness/render-targets/hooks/capture-learning.prompt.md"
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
  ".agent-harness/skills/10-planning/plan-and-frame.json"
  ".agent-harness/skills/20-implementation/implement-with-proof.json"
  ".agent-harness/skills/30-test/test-with-proof.json"
  ".agent-harness/skills/40-debug/debug-discipline.json"
  ".agent-harness/skills/45-review/review-for-risk.json"
  ".agent-harness/skills/50-completion/verify-before-completion.json"
  ".agent-harness/skills/60-learning/capture-learning.json"
  ".agent-harness/traces/traceability-template.yaml"
  ".agent-harness/traces/README.md"
  ".agent-harness/traces/route-decisions/README.md"
  ".github/workflows/ci.yml"
  ".github/copilot-instructions.md"
  ".vscode/extensions.json"
  ".vscode/settings.json"
  "AGENTS.md"
  "SWE_SEED_SPEC_v0.2.0.md"
  "HARNESS_SPEC.md"
  "FABRICATOR_SPEC_v0.1.0.md"
  ".editorconfig"
  ".env.example"
  ".envrc"
  ".gitattributes"
  ".gitignore"
  ".agent-hooks/config.yaml"
  ".fabricator/config.yaml"
  ".fabricator/runs/README.md"
  ".fabricator/schemas/product-seed.schema.yaml"
  ".fabricator/schemas/jtbd.schema.yaml"
  ".fabricator/schemas/job-hypothesis.schema.yaml"
  ".fabricator/schemas/hypothesis.schema.yaml"
  ".fabricator/schemas/adr.schema.yaml"
  ".fabricator/schemas/prd.schema.yaml"
  ".fabricator/schemas/sds.schema.yaml"
  ".fabricator/schemas/tdd-plan.schema.yaml"
  ".fabricator/schemas/context-pack.schema.yaml"
  ".fabricator/schemas/agent-task.schema.yaml"
  ".fabricator/schemas/eval-checklist.schema.yaml"
  ".fabricator/schemas/eval-spec.schema.yaml"
  ".fabricator/schemas/eval-result.schema.yaml"
  ".fabricator/schemas/proof-record.schema.yaml"
  ".fabricator/schemas/reflection.schema.yaml"
  ".fabricator/schemas/adaptation-decision.schema.yaml"
  ".fabricator/schemas/skill-proposal.schema.yaml"
  ".fabricator/templates/PRODUCT_SEED.md.j2"
  ".fabricator/templates/JTBD.md.j2"
  ".fabricator/templates/JOB_HYPOTHESIS.md.j2"
  ".fabricator/templates/HYPOTHESIS.md.j2"
  ".fabricator/templates/ADR.md.j2"
  ".fabricator/templates/PRD.md.j2"
  ".fabricator/templates/SDS.md.j2"
  ".fabricator/templates/TDD.md.j2"
  ".fabricator/templates/CONTEXT_PACK.md.j2"
  ".fabricator/templates/AGENT_TASK.md.j2"
  ".fabricator/templates/EVAL_CHECKLIST.md.j2"
  ".fabricator/templates/PROOF_RECORD.md.j2"
  ".fabricator/templates/REFLECTION.md.j2"
  ".fabricator/templates/NO_SKILL_PROPOSED.md.j2"
  ".fabricator/templates/ADAPTATION_DECISION.yaml.j2"
  "scripts/fabricate.py"
  "docs/fabrication-layer/README.md"
  "docs/fabrication-layer/howto/create-a-product-seed.md"
  "docs/fabrication-layer/howto/generate-a-fabrication-packet.md"
  "docs/fabrication-layer/howto/hand-off-to-claude-code.md"
  "docs/fabrication-layer/howto/verify-a-prototype.md"
  "docs/fabrication-layer/howto/capture-run-learning.md"
  "docs/fabrication-layer/explanations/artifact-chain.md"
  "docs/fabrication-layer/explanations/baml-generation-model.md"
  "docs/fabrication-layer/explanations/proof-before-completion.md"
  "docs/fabrication-layer/references/artifact-schemas.md"
  "docs/fabrication-layer/references/command-contract.md"
  "docs/fabrication-layer/references/html5-game-pilot.md"
  ".mise.toml"
  "devbox.json"
  "docs/dev-harness/README.md"
  "docs/dev-harness/explanations/local-ci-parity.md"
  "docs/dev-harness/explanations/observability-model.md"
  "docs/dev-harness/explanations/secrets-model.md"
  "docs/dev-harness/howto/add-a-ci-check.md"
  "docs/dev-harness/howto/debug-failing-ci.md"
  "docs/dev-harness/howto/initialize-dev-env.md"
  "docs/dev-harness/howto/run-local-ci.md"
  "docs/dev-harness/references/just-recipes.md"
  "docs/dev-harness/references/observability-contract.md"
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
  "scripts/agent-hooks"
  "scripts/agent_hooks.py"
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
grep -qE '^harness-eval-run spec output="":' justfile
grep -qE '^fabricate-new seed:' justfile
grep -qE '^fabricate-generate run_id:' justfile
grep -qE '^fabricate-validate run_id:' justfile
grep -qE '^fabricate-handoff run_id:' justfile
grep -qE '^fabricate-proof run_id:' justfile
grep -qE '^fabricate-reflect run_id:' justfile
grep -qE '^fabricate-status run_id:' justfile
grep -qE '^harness-plan-learning-store backend="both":' justfile
grep -qE '^agent-hooks-trace-last:' justfile
grep -qE '^agent-hooks-trace-session session_id:' justfile
grep -qE '^agent-hooks-inspect event_id:' justfile
grep -qE '^agent-hooks-replay event_id:' justfile
grep -qE '^agent-hooks-doctor:' justfile
grep -qE '^agent-hooks-compact-logs:' justfile
grep -qE '^agent-hooks-index-rebuild:' justfile
grep -qE '^agent-hooks-export-otel output="":' justfile
grep -qE '^agent-hooks-export-junit output="":' justfile
grep -q 'just ci' .github/workflows/ci.yml
grep -q 'pnpm/action-setup' .github/workflows/ci.yml
grep -q 'astral-sh/setup-uv' .github/workflows/ci.yml
grep -q 'AGENTS.md' .github/copilot-instructions.md
grep -q '.agent-harness/routes/' AGENTS.md
grep -q '\.secrets/' .gitignore
grep -q '\*\.dec\.\*' .gitignore
grep -q '\*\.plain\.\*' .gitignore
grep -q 'sops' .gitignore
grep -q 'SWE_SEED' package.json
grep -q 'swe-seed' pyproject.toml
python scripts/fabricate.py --help >/tmp/swe-seed-fabricate-help.log
grep -q 'fabricate' /tmp/swe-seed-fabricate-help.log
python scripts/harness.py validate
obsolete_fabrication_alias="FABRICATION_LAYER""_SPEC_v0.1.0.md"
if [[ -f "$obsolete_fabrication_alias" ]]; then
  echo "Obsolete fabrication-layer alias still exists" >&2
  exit 1
fi
uv run baml-cli generate --from .agent-harness/baml/baml_src >/tmp/swe-seed-baml-generate.log
grep -q 'Generated' /tmp/swe-seed-baml-generate.log
grep -q 'class ProductHypothesis' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'target_job string' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'value_proposition string' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'class JobStory' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'enum EARSPattern' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'class EARSRequirement' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'class YStatement' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'class SDSComponent' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'class GherkinScenario' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'class TraceabilityLink' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'requirements EARSRequirement\[\]' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'y_statements YStatement\[\]' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'components SDSComponent\[\]' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'scenarios GherkinScenario\[\]' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateProductHypothesis' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateJobStory' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateEARSRequirements' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateYStatementADR' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateSDSComponents' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateGherkinScenarios' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateEvalSpecFromSemanticChain' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function GenerateAgentTaskFromSemanticChain' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function ValidateSemanticChain' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'function ProposeTraceabilityRepair' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'Every requirement MUST use EARS format' .agent-harness/baml/baml_src/fabricator.baml
grep -q 'Semantic Specification Chain' SWE_SEED_SPEC_v0.2.0.md
grep -q 'Semantic Specification Chain' FABRICATOR_SPEC_v0.1.0.md
grep -q 'just fabricate-new <seed>' SWE_SEED_SPEC_v0.2.0.md
grep -q 'scripts/fabricate.py' SWE_SEED_SPEC_v0.2.0.md
grep -q 'eval-spec.schema.yaml' FABRICATOR_SPEC_v0.1.0.md
grep -q 'NO_SKILL_PROPOSED.md.j2' FABRICATOR_SPEC_v0.1.0.md
grep -q 'EVAL_RESULT.json' FABRICATOR_SPEC_v0.1.0.md
grep -q 'Deterministic Serialization' FABRICATOR_SPEC_v0.1.0.md
grep -q 'Semantic Chain Validation' SWE_SEED_SPEC_v0.2.0.md
grep -q '## Job Story Syntax' FABRICATOR_SPEC_v0.1.0.md
grep -q '## Y-Statement ADR Syntax' FABRICATOR_SPEC_v0.1.0.md
grep -q '## SDS Structural Requirements' FABRICATOR_SPEC_v0.1.0.md
grep -q '## Gherkin Behavioral Syntax' FABRICATOR_SPEC_v0.1.0.md
grep -q 'No Gherkin scenario may be considered satisfied' FABRICATOR_SPEC_v0.1.0.md
fabricator_seed=$(mktemp --suffix=.fabricator-seed.yaml)
cat >"$fabricator_seed" <<'YAML'
run_id: "0001-focus-runner"
product_type: "single_page_html5_game"
user: "casual player with 60 seconds"
situation: "player opens a local browser game with no setup"
desired_outcome: "player experiences how distractions compete with focus"
prototype_goal: "create a playable one-file HTML5 canvas game"
constraints:
  - "index.html only"
  - "vanilla HTML/CSS/JS"
  - "no backend"
  - "no external assets"
  - "no network calls"
non_goals:
  - "no multiplayer"
  - "no account system"
  - "no build system"
success_conditions:
  - "game opens locally"
  - "player can move"
  - "score changes"
  - "win/loss condition works"
  - "restart works"
proof_expectations:
  - "manual browser checklist passes"
  - "no console errors on load"
YAML
python scripts/fabricate.py new "$fabricator_seed" >/tmp/swe-seed-fabricate-new.log
grep -q '0001-focus-runner' /tmp/swe-seed-fabricate-new.log
python scripts/fabricate.py generate 0001-focus-runner >/tmp/swe-seed-fabricate-generate.log
grep -q 'generated' /tmp/swe-seed-fabricate-generate.log
python scripts/fabricate.py validate 0001-focus-runner >/tmp/swe-seed-fabricate-validate.log
grep -q '"status": "pass"' /tmp/swe-seed-fabricate-validate.log
python scripts/fabricate.py handoff 0001-focus-runner >/tmp/swe-seed-fabricate-handoff.log
grep -q 'AGENT_TASK.md' /tmp/swe-seed-fabricate-handoff.log
python scripts/fabricate.py proof 0001-focus-runner >/tmp/swe-seed-fabricate-proof.log
grep -q '"status": "pass"' /tmp/swe-seed-fabricate-proof.log
python scripts/fabricate.py reflect 0001-focus-runner >/tmp/swe-seed-fabricate-reflect.log
grep -q 'REFLECTION.md' /tmp/swe-seed-fabricate-reflect.log
python scripts/fabricate.py status 0001-focus-runner >/tmp/swe-seed-fabricate-status.log
grep -q '"run_id": "0001-focus-runner"' /tmp/swe-seed-fabricate-status.log
eval_spec_path=$(mktemp --suffix=.EVAL_SPEC.yaml)
eval_result_path=$(mktemp --suffix=.EVAL_RESULT.json)
cat >"$eval_spec_path" <<'JSON'
{
  "id": "harness-smoke-eval",
  "version": "0.1.0",
  "run_id": "validate-harness",
  "target_type": "repo",
  "target_path": "HARNESS_SPEC.md",
  "purpose": "Verify local EvalSpec and EvalResult execution.",
  "eval_classes": [
    "product_outcome",
    "process_compliance",
    "learning_quality",
    "adaptation_eligibility"
  ],
  "checks": [
    {
      "id": "root-spec-exists",
      "class": "process_compliance",
      "type": "file_exists",
      "target": "HARNESS_SPEC.md",
      "required": true,
      "rule": "file exists",
      "evidence_required": ["observed file"]
    },
    {
      "id": "root-spec-has-eval-layer",
      "class": "process_compliance",
      "type": "static_required_patterns",
      "target": "HARNESS_SPEC.md",
      "required": true,
      "rule": ["Evaluation and Adaptation Layer"],
      "evidence_required": ["observed required section"]
    },
    {
      "id": "release-evidence-recorded",
      "class": "learning_quality",
      "type": "manual_check",
      "target": "HARNESS_SPEC.md",
      "required": true,
      "rule": "operator evidence recorded",
      "evidence": ["tests/validate-harness.sh executed local eval runner"],
      "evidence_required": ["manual evidence"]
    }
  ],
  "pass_condition": "all required checks pass",
  "outputs": ["EVAL_RESULT.json"]
}
JSON
python scripts/harness.py eval run "$eval_spec_path" --output "$eval_result_path" >/tmp/swe-seed-eval-run.log
grep -q '"status": "pass"' "$eval_result_path"
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
route_capture_output=$(python scripts/harness.py route --capture-hook --agent copilot --agent-version test --session-id route-session --trace-id route-trace --span-id route-span "implement parser change")
grep -q '"hook_capture"' <<<"$route_capture_output"
route_capture_event_id=$(python -c 'import json,sys; print(json.load(sys.stdin)["hook_capture"]["event_id"])' <<<"$route_capture_output")
route_capture_inspect=$(scripts/agent-hooks inspect --event "$route_capture_event_id")
grep -q 'route-trace' <<<"$route_capture_inspect"
grep -q 'route-session' <<<"$route_capture_inspect"
grep -q 'route-span' <<<"$route_capture_inspect"
grep -q 'route_preview' <<<"$route_capture_inspect"
inspect_output=$(python scripts/harness.py inspect debug-discipline)
grep -q '"skills"' <<<"$inspect_output"
context_plan_output=$(python scripts/harness.py context-plan "implement a parser change")
grep -q '"context_budget"' <<<"$context_plan_output"
grep -q '## Bundled resources' .agent-harness/render-targets/claude/debug/debug-discipline/SKILL.md
grep -q '## Evaluation prompts' .agent-harness/render-targets/claude/debug/debug-discipline/SKILL.md
grep -q '## Bundled resources' .agent-harness/render-targets/claude/planning/plan-and-frame/SKILL.md
grep -q '## Evaluation prompts' .agent-harness/render-targets/claude/implementation/implement-with-proof/SKILL.md
grep -q '## Evaluation prompts' .agent-harness/render-targets/claude/test/test-with-proof/SKILL.md
grep -q '## Evaluation prompts' .agent-harness/render-targets/claude/review/review-for-risk/SKILL.md
grep -q '## Evaluation prompts' .agent-harness/render-targets/claude/completion/verify-before-completion/SKILL.md
grep -q '## Evaluation prompts' .agent-harness/render-targets/claude/learning/capture-learning/SKILL.md
trace_start_output=$(python scripts/harness.py trace start --capture-hook --agent copilot --agent-version test --session-id trace-session --span-id trace-start-span "checkpoint smoke")
grep -q '"hook_capture"' <<<"$trace_start_output"
trace_id=$(python -c 'import json,sys; print(json.load(sys.stdin)["trace_id"])' <<<"$trace_start_output")
trace_start_event_id=$(python -c 'import json,sys; print(json.load(sys.stdin)["hook_capture"]["event_id"])' <<<"$trace_start_output")
trace_start_inspect=$(scripts/agent-hooks inspect --event "$trace_start_event_id")
grep -q 'trace-session' <<<"$trace_start_inspect"
grep -q "$trace_id" <<<"$trace_start_inspect"
trace_output=$(python scripts/harness.py trace checkpoint --capture-hook --agent copilot --agent-version test --session-id trace-session --span-id trace-checkpoint-span "$trace_id" --stage change --summary "spec delta captured" --next-action "run targeted validation" --artifact HARNESS_SPEC.md --risk "proof not run")
grep -q '"hook_capture"' <<<"$trace_output"
trace_checkpoint_event_id=$(python -c 'import json,sys; print(json.load(sys.stdin)["hook_capture"]["event_id"])' <<<"$trace_output")
trace_checkpoint_inspect=$(scripts/agent-hooks inspect --event "$trace_checkpoint_event_id")
grep -q 'trace-checkpoint-span' <<<"$trace_checkpoint_inspect"
grep -q 'spec delta captured' <<<"$trace_checkpoint_inspect"
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

event_output=$(printf '{"native_event":"turn.stop","event":"turn.stop","session_id":"validate-session","turn_id":"validate-turn","profile":"test","hook_id":"validate-hook","script":"tests/validate-harness.sh","status":"ok","result":{"message":"validation event"}}' | scripts/agent-hooks capture --agent copilot --agent-version test)
grep -q 'event_id' <<<"$event_output"
event_id=$(python -c 'import json,sys; print(json.load(sys.stdin)["event_id"])' <<<"$event_output")
trace_output=$(scripts/agent-hooks trace --last)
grep -q 'events' <<<"$trace_output"
session_trace_output=$(scripts/agent-hooks trace --session validate-session)
grep -q 'validate-session' <<<"$session_trace_output"
inspect_event_output=$(scripts/agent-hooks inspect --event "$event_id")
grep -q 'normalized_payload' <<<"$inspect_event_output"
replay_event_output=$(scripts/agent-hooks replay --event "$event_id")
grep -q 'result_payload' <<<"$replay_event_output"
doctor_output=$(scripts/agent-hooks doctor --observability)
grep -q 'status' <<<"$doctor_output"
compact_output=$(scripts/agent-hooks compact-logs)
grep -q 'compacted_files' <<<"$compact_output"
index_output=$(scripts/agent-hooks index rebuild)
grep -q 'index_db' <<<"$index_output"
otel_output=$(scripts/agent-hooks export otel)
grep -q 'resource' <<<"$otel_output"
junit_output=$(scripts/agent-hooks export junit)
grep -q '<testsuite' <<<"$junit_output"

hook_output=$(.agent-harness/hooks/hook-router.sh prompt.submit --task "implement parser change" --agent copilot --agent-version test --session-id hook-session --trace-id hook-trace --span-id hook-span --capture)
grep -q 'event_id' <<<"$hook_output"
hook_event_id=$(python -c 'import json,sys; print(json.load(sys.stdin)["event_id"])' <<<"$hook_output")
hook_inspect_output=$(scripts/agent-hooks inspect --event "$hook_event_id")
grep -q 'hook-trace' <<<"$hook_inspect_output"
grep -q 'hook-session' <<<"$hook_inspect_output"
grep -q 'hook-span' <<<"$hook_inspect_output"

echo "Harness validation passed"
