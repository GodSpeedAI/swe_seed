# Context Discipline

## Use this when

Use this when a task could pull broad files, logs, search results, tool output, web pages, or prior-session state into the conversation. The goal is to spend attention on decisions and proof, not raw data.

## context budget

Start with the route card and the required context it names. If the task needs more, add the smallest file, command, or excerpt that changes the next action. Stop reading when the next edit or proof command is clear.

Run:

```bash
python scripts/harness.py context-plan "task"
```

The plan orders route context before optional exploration and reminds the agent where to store durable state.

## tool-output containment

High-volume tools should return a summary, focused excerpt, count, path list, or computed JSON instead of raw output. Keep full logs in files or trace notes only when they are needed as proof. Do not paste secrets, raw environment dumps, or noisy logs into trace records.

## think in code

For bulk analysis, write or run a small script, shell pipeline, parser, or query. Let code count, filter, rank, diff, or validate. Bring back the result and the command. This keeps the model from manually processing thousands of tokens that a deterministic command can reduce.

## session continuity

Session continuity comes from durable artifacts: route decision records, trace records, memory updates, and spec changes. After compaction or restart, a new agent should recover state from those artifacts without needing the full transcript.

## Done when

Context discipline is working when the agent can name the governing route, the files it read, the reason each extra artifact was needed, the proof command, and the unresolved risks without replaying raw tool output.
