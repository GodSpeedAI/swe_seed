set dotenv-load := true

default:
    @just --list

bootstrap:
    @bash scripts/bootstrap.sh

doctor:
    @bash scripts/doctor.sh

format:
    @bash scripts/ci.sh format

lint:
    @bash scripts/ci.sh lint

test:
    @bash scripts/ci.sh test

ci:
    @bash scripts/ci.sh

harness-validate:
    @python scripts/harness.py validate

harness-doctor:
    @python scripts/harness.py doctor

harness-render-skills:
    @python scripts/harness.py render-skills

harness-route task:
    @python scripts/harness.py route "{{task}}"

harness-route-record task:
    @python scripts/harness.py route --record "{{task}}"

harness-inspect item:
    @python scripts/harness.py inspect "{{item}}"

harness-context-plan task:
    @python scripts/harness.py context-plan "{{task}}"

harness-trace-start task:
    @python scripts/harness.py trace start "{{task}}"

harness-trace-append trace note:
    @python scripts/harness.py trace append "{{trace}}" "{{note}}"

harness-trace-distill trace:
    @python scripts/harness.py trace distill "{{trace}}"

harness-plan-learning-store backend="both":
    @bash scripts/plan-learning-store.sh "{{backend}}"

harness-sync-learning-store db_path="":
    @bash scripts/sync-learning-store.sh "{{db_path}}"

harness-query-learning-store mode="summaries" limit="10" status="any" job_type="any" db_path="":
    @bash scripts/query-learning-store.sh "{{mode}}" "{{limit}}" "{{status}}" "{{job_type}}" "{{db_path}}"

harness-eval-learning-retrieval db_path="":
    @bash scripts/eval-learning-retrieval.sh "{{db_path}}"

agent-hooks-trace-last:
    @scripts/agent-hooks trace --last

agent-hooks-trace-session session_id:
    @scripts/agent-hooks trace --session "{{session_id}}"

agent-hooks-inspect event_id:
    @scripts/agent-hooks inspect --event "{{event_id}}"

agent-hooks-replay event_id:
    @scripts/agent-hooks replay --event "{{event_id}}"

agent-hooks-doctor:
    @scripts/agent-hooks doctor --observability

agent-hooks-compact-logs:
    @scripts/agent-hooks compact-logs

agent-hooks-index-rebuild:
    @scripts/agent-hooks index rebuild

agent-hooks-export-otel output="":
    @if [ -n "{{output}}" ]; then scripts/agent-hooks export otel --output "{{output}}"; else scripts/agent-hooks export otel; fi

agent-hooks-export-junit output="":
    @if [ -n "{{output}}" ]; then scripts/agent-hooks export junit --output "{{output}}"; else scripts/agent-hooks export junit; fi

harness-trace-finish trace claim:
    @python scripts/harness.py trace finish "{{trace}}" --claim "{{claim}}"

secrets-encrypt:
    @bash scripts/secrets-encrypt.sh

secrets-decrypt:
    @bash scripts/secrets-decrypt.sh

secrets-edit file:
    @bash scripts/secrets-edit.sh "{{file}}"

secrets-rotate-key:
    @bash scripts/secrets-rotate-key.sh
