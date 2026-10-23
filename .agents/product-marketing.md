# SWE_SEED product marketing context

**Status:** Working canonical product-marketing context
**Product:** SWE_SEED
**Company:** GodSpeed AI
**Primary use:** README copy, website copy, launch materials, developer documentation, product strategy, enterprise explanations, proof demonstrations, and AI-generated marketing content
**Last updated:** July 29, 2026

---

## 1. Source-of-truth hierarchy

When sources disagree, use this order:

1. The current SWE_SEED repository, tests, schemas, fixtures, commands, and license.
2. The current SWE_SEED README, harness specification, implementation plans, and architecture decisions.
3. This product-marketing context.
4. The GodSpeed AI Brand Voice Style Guide.
5. Current SEA Forge integration specifications.
6. GodSpeed AI strategic architecture documents.
7. Older language describing SWE_SEED as a general agent harness or settlement system.

Never let positioning override current behavior, implementation boundaries, technical limitations, or license terms.

Separate claims into four levels:

* **Evidence-backed:** supported by current code, tests, commands, fixtures, generated artifacts, or repository documentation.
* **Partially proven:** meaningful implementation exists, but complete cross-host, end-to-end, or production proof remains incomplete.
* **Strategic interpretation:** follows from the architecture but should be presented as positioning rather than shipped capability.
* **Roadmap:** planned, specified, or intended but not yet proven.

---

## 2. The framing hierarchy

Use these descriptions at different levels.

### Implementation description

SWE_SEED is a CLI harness.

It projects shared work contracts, policies, hooks, skills, and host-specific instructions into supported coding-agent environments.

It records work artifacts under the repository’s harness state rather than requiring a standalone network service.

### Product description

SWE_SEED is a verified agentic software-work harness.

It gives coding agents:

* a route;
* bounded context;
* an expected artifact;
* proof commands;
* a trace.

### Category description

SWE_SEED is a proof-bearing execution protocol for agentic software work.

The protocol turns a coding request into a falsifiable work contract:

```text
requested change
→ selected route
→ required context
→ expected artifact
→ required proof
→ recorded trace
```

### Market role

SWE_SEED is the low-friction adoption wedge for trustworthy agentic software delivery.

It improves the work performed by coding agents teams already use rather than requiring them to replace those agents with a new proprietary assistant.

### Canonical formulation

> SWE_SEED turns agentic coding from a conversation into a verifiable work contract.

---

## 3. Product summary

### Plain-language definition

SWE_SEED gives AI coding agents a route, the context they need, a required output, proof commands, and a trace before anyone gets to call the work done.

It works with the coding agents developers already use.

The agent can propose, edit, run tools, and report progress. SWE_SEED keeps the job tied to the repository, its acceptance criteria, and evidence that can be independently judged.

### Technical definition

SWE_SEED is a local-first CLI harness that projects canonical work contracts, route logic, policies, hooks, skills, and proof requirements into supported host coding agents.

It records route, trace, artifact, proof, evaluation, and learning-candidate data in repository-local harness state.

### Smallest useful explanation

> SWE_SEED makes coding agents show their work.

### One-sentence position

> SWE_SEED makes agentic software work route-based, context-loaded, artifact-producing, proof-checked, and trace-bearing.

### Sharp public version

> The agent’s final message ends the conversation. It does not close the work.

### Controlling product truth

> In software, “done” is not what the developer says. It is what the artifact and the proof can support.

Coding agents should not receive a lower standard because they type faster.

---

## 4. The market wedge

SWE_SEED’s first market is not every software-development workflow.

The first wedge is:

> Teams using AI coding agents for real repository changes who cannot reliably tell whether the requested work was actually completed.

The smallest useful deployment should include:

* one repository;
* one coding agent;
* one work request;
* one selected route;
* one bounded context package;
* one required artifact or diff;
* one or more proof commands;
* one trace showing what happened;
* one explicit pass or fail result.

The buyer does not need to adopt the entire GodSpeed stack.

SWE_SEED can provide standalone value by improving the rigor of work performed through existing coding agents.

### Immediate customer problem

```text
The agent received a prompt.
It changed some files.
It ran some commands.
It wrote a convincing summary.
Nobody can quickly prove whether it solved the requested problem.
```

### Recommended wedge line

> Keep your coding agent. Make the work prove itself.

### Recommended product category

> Verified agentic software-work harness.

### Broader category

> Proof-bearing execution protocol for AI-assisted software delivery.

### GodSpeed category

> The software-work proof layer of syntelligent infrastructure.

Use these categories in sequence. Do not lead with syntelligent infrastructure before the reader understands the failed-completion problem.

---

## 5. The core analogy

A coding agent should be treated like a developer whose work must survive the repository rather than merely sound credible in a status update.

### Ordinary software work

```text
task
→ implementation
→ artifact or diff
→ tests and checks
→ review
→ accepted or rejected result
```

### Unstructured agentic work

```text
prompt
→ agent activity
→ completion message
→ manual investigation
```

### SWE_SEED work

```text
work request
→ route
→ context contract
→ required artifact
→ proof commands
→ trace
→ evidence handoff
```

### Controlling analogy

> A completion summary is a status report. It is not a build result.

### Boundary of the analogy

SWE_SEED does not determine every meaning of “correct.”

Some software outcomes require:

* human review;
* business judgment;
* runtime observation;
* security approval;
* policy authority;
* production evidence;
* settlement by another system.

SWE_SEED makes the work and its proof legible. It does not claim that every passing command establishes final truth.

---

## 6. Why now

AI coding tools have lowered the cost of producing code.

They have not lowered the cost of proving that the code is correct, relevant, safe, complete, or maintainable.

### Generation is becoming cheap

Coding agents can quickly:

* inspect repositories;
* write code;
* generate tests;
* modify configuration;
* run commands;
* explain changes;
* produce plausible pull requests.

### Verification remains expensive

Teams must still determine:

* whether the agent understood the actual request;
* whether it used the right repository context;
* whether it changed the right files;
* whether required artifacts exist;
* whether relevant tests were run;
* whether omitted checks would have failed;
* whether the change respects architectural boundaries;
* whether the summary matches the diff;
* whether the result can be reproduced.

### The current failure pattern

Most coding-agent workflows treat the final response as the end of the task.

The developer then becomes the forensic layer:

* inspect the diff;
* rediscover the task;
* determine which context the agent missed;
* run the forgotten checks;
* identify unstated assumptions;
* reconstruct why the agent made the change;
* decide whether to trust the summary.

The agent saves typing while transferring verification burden to the human.

### Why SWE_SEED matters now

As models become more capable and interchangeable, the durable advantage moves into the harness around the work:

* how tasks are routed;
* which context is loaded;
* which artifacts are required;
* which proof must run;
* which evidence is retained;
* which failures improve the next attempt.

> Better models produce more candidate work. Better harnesses produce more trustworthy work.

---

## 7. Primary audiences

### 7.1 Primary technical user

**Who they are**

* software engineers using coding agents;
* staff and principal engineers;
* technical founders;
* maintainers of large repositories;
* platform engineers;
* developer-experience engineers;
* AI-assisted development teams.

**Current situation**

They use one or more of:

* Claude Code;
* Codex;
* OpenCode;
* GitHub Copilot;
* Antigravity;
* CI-hosted coding agents;
* other host agents supported by the current repository.

They often maintain separate:

* instruction files;
* skills;
* hooks;
* prompt templates;
* test conventions;
* proof expectations;
* memory files;
* repository policies.

**What they want**

* to keep using their preferred agent;
* one consistent work protocol across hosts;
* less repeated prompting;
* context selected for the actual task;
* explicit expected artifacts;
* proof commands declared before completion;
* records that make failures diagnosable;
* confidence that switching agents does not erase the workflow.

**What they distrust**

* another proprietary coding assistant;
* agent orchestration platforms that require replacing their current tools;
* heavy dashboards;
* generated status reports;
* brittle prompt frameworks;
* “autonomous development” claims without repository proof.

### 7.2 Engineering and platform leader

**Who they are**

* VP or Head of Engineering;
* director of platform engineering;
* developer-productivity leader;
* internal developer-platform owner;
* AI engineering leader;
* engineering quality leader.

**What they need**

* repeatable agentic work across teams;
* consistent proof expectations;
* host-independent policy;
* lower review burden;
* fewer false completion claims;
* visibility into which tasks, routes, checks, and agents work;
* adoption without forcing one model or vendor;
* a practical path from experimentation to controlled software delivery.

**Emotional state**

They want the productivity upside of coding agents.

They do not want every team inventing a different prompt ritual and calling it an operating model.

### 7.3 Quality, security, and regulated-software audience

**Who they are**

* quality engineers;
* application-security engineers;
* software assurance teams;
* regulated-software leaders;
* release managers;
* compliance and audit stakeholders.

**What they need**

* predeclared acceptance criteria;
* traceability from request to artifact to proof;
* explicit missing-proof failures;
* integration with existing scanners and CI;
* evidence that required checks ran;
* clear boundaries between software proof and runtime authority;
* evidence packs suitable for review.

SWE_SEED must not claim that a proof command automatically establishes regulatory compliance.

### 7.4 Secondary audiences

* open-source maintainers;
* consultants working across many repositories;
* Forward-Deployed Engineers;
* Delivery Engineers;
* AI Delivery Engineers;
* agent-tooling builders;
* internal platform teams;
* teams evaluating several coding agents;
* organizations building private or local-first development environments.

For the README:

* lead for the working developer;
* demonstrate value to the platform leader;
* preserve enough precision for quality and security review.

---

## 8. Audience knowledge and emotional state

Assume the reader understands:

* source control;
* repositories;
* diffs;
* tests;
* linting;
* static analysis;
* CI;
* coding agents;
* prompts;
* hooks;
* instruction files.

Do not assume the reader understands:

* route artifacts;
* harness projections;
* BAML-backed contracts;
* host adapters;
* proof events;
* Context Kernel;
* SEA Forge;
* GodSpeed-Agent;
* settlement;
* capability metabolization;
* syntelligent infrastructure.

The reader is:

* interested but skeptical;
* experienced with coding-agent inconsistency;
* tired of writing the same instructions repeatedly;
* unwilling to adopt a complex platform before seeing one task work;
* alert to “framework” language that conceals more process than value;
* likely to judge the product through its repository and CLI before reading strategy.

The first screen must explain the failed-completion problem and the smallest proof loop.

---

## 9. Core jobs to be done

### Functional jobs

When I delegate software work to a coding agent, help me:

1. convert the request into a bounded work contract;
2. select the correct route for the task;
3. identify required context before implementation;
4. project consistent instructions into the active host agent;
5. state which artifact or diff must exist;
6. state which proof commands must run;
7. prevent a missing route from silently becoming improvisation;
8. keep host-specific instructions derived from one canonical harness state;
9. record what context, tools, and route influenced the work;
10. preserve a trace from request to artifact to proof;
11. distinguish a passing check from a convincing summary;
12. fail explicitly when required proof is absent or fails;
13. compare behavior across coding-agent hosts;
14. detect drift between canonical harness policy and host projections;
15. preserve rollback and recovery information;
16. produce evidence another system or reviewer can judge;
17. turn failures into reviewed improvements to the harness.

### Emotional jobs

Help me feel:

* less dependent on agent confidence;
* able to switch agents without rebuilding my operating method;
* confident that the relevant checks were not quietly skipped;
* less burdened by forensic review after every run;
* able to understand why a task failed;
* comfortable delegating larger work because the proof boundary is clear.

### Social jobs

Help me:

* introduce coding agents without lowering engineering standards;
* show reviewers the evidence instead of repeating the agent’s explanation;
* give teams one shared work protocol;
* support agent adoption without committing to one model vendor;
* make AI-assisted work look disciplined rather than improvised;
* prove that developer productivity did not come from hiding verification debt.

---

## 10. Core customer problems

### 10.1 The final message is being mistaken for the result

Coding agents produce polished summaries.

The summary may say:

* implemented;
* fixed;
* tested;
* complete;
* production ready.

The repository may show:

* missing files;
* incomplete behavior;
* omitted tests;
* unrelated changes;
* failed checks;
* TODOs;
* architectural drift.

Underlying problem:

> Agent narration is being allowed to close a claim the repository has not proven.

### 10.2 Every agent receives a different operating system

Teams maintain different instructions for:

* Claude Code;
* Codex;
* OpenCode;
* Copilot;
* CI agents;
* local tools.

Each host receives a slightly different version of:

* repository rules;
* test expectations;
* workflows;
* tool policies;
* context references;
* completion criteria.

Underlying problem:

> The organization has several agent instruction surfaces but no canonical harness state.

### 10.3 Context selection is a lottery

Agents can read a large repository without understanding which files carry the decisive constraints.

More context can increase:

* noise;
* contradiction;
* token cost;
* stale assumptions;
* misplaced confidence.

Underlying problem:

> Access to the repository is being confused with possession of the right context.

### 10.4 Plans do not bind execution

An agent may create a plausible plan, then:

* ignore it;
* change direction silently;
* solve a nearby problem;
* omit required artifacts;
* run easier checks;
* stop after partial progress.

Underlying problem:

> The plan exists as prose, not as an execution contract connected to evidence.

### 10.5 Proof is decided after the work

Developers often decide which tests matter only after reviewing the diff.

The agent can then:

* choose convenient checks;
* avoid expensive tests;
* report a narrow pass;
* omit commands that would expose failure.

Underlying problem:

> The actor performing the work is also choosing what counts as sufficient proof.

SWE_SEED should bind proof obligations to the work contract before completion.

### 10.6 CI arrives too late

CI remains valuable, but it often runs after the agent has:

* consumed time;
* modified many files;
* declared completion;
* handed the work to a human;
* opened a pull request.

Underlying problem:

> The first real verification happens after the agent has already optimized for completion.

SWE_SEED brings relevant proof into the work loop while still preserving CI as an independent downstream check.

### 10.7 Agent behavior cannot be compared

When one agent succeeds and another fails, teams often cannot distinguish:

* model capability;
* context quality;
* route quality;
* instruction quality;
* proof selection;
* repository state;
* tool availability.

Underlying problem:

> The work lacks a stable experimental frame.

### 10.8 Failures disappear into transcripts

A failed agent session may contain useful information:

* a missing file;
* an invalid assumption;
* an ineffective route;
* a proof command that should become mandatory;
* a repeated context gap.

Without structured residue, the next session starts from the same ignorance.

Underlying problem:

> The organization pays for failure but does not preserve what the failure taught the harness.

### 10.9 Prompt improvements do not compound

A developer learns that a certain instruction, context file, or proof command matters.

That knowledge often remains:

* in chat history;
* in one agent’s memory;
* in the developer’s head;
* in an unreviewed prompt fragment.

Underlying problem:

> The harness does not improve at the same rate as the humans operating it.

### 10.10 Host configuration drifts

Generated host instructions, hooks, policies, memory views, and briefs can diverge from the canonical harness contracts.

Underlying problem:

> Derived agent configuration is being edited as though it were the source.

---

## 11. Status quo and alternatives

SWE_SEED competes first with unstructured agentic coding.

It also complements several existing tools.

### 11.1 Prompting the agent directly

**Why developers use it**

* no setup;
* flexible;
* immediate;
* works with any conversational agent.

**Where it fails**

* task structure varies by user;
* context expectations remain implicit;
* proof requirements are easy to omit;
* completion depends on narration;
* lessons do not become shared infrastructure.

**SWE_SEED position**

> Prompts start conversations. SWE_SEED defines work contracts.

### 11.2 `AGENTS.md`, `CLAUDE.md`, and repository instruction files

**Why teams use them**

* repository local;
* easy to review;
* supported by major coding agents;
* useful for durable rules.

**Where they fall short**

* mostly static;
* host formats differ;
* cannot always express route-specific context;
* proof and trace behavior may remain advisory;
* duplicated files can drift.

**SWE_SEED position**

> Repository instructions remain useful. SWE_SEED projects them from a canonical harness and binds them to task routes, artifacts, proof, and trace.

### 11.3 Agent-specific skills, rules, and hooks

**Why teams use them**

* powerful host integration;
* pre-action enforcement;
* customizable workflows;
* close to the agent.

**Where they fall short**

* host lock-in;
* inconsistent semantics;
* duplicated maintenance;
* difficult comparison across agents;
* weak common evidence format.

**SWE_SEED position**

> Keep host-native mechanisms. Generate and govern them from one portable work protocol.

### 11.4 CI pipelines

**Why teams use them**

* independent execution;
* mature tooling;
* required release gates;
* reliable repeatability.

**Where they fall short**

* often run after the agent stops;
* do not determine which context or route the agent used;
* may not require the full task artifact;
* do not preserve the reasoning path or route decision;
* a green pipeline may still prove the wrong task.

**SWE_SEED position**

> CI proves repository checks. SWE_SEED connects the request, route, context, artifact, and checks into one evidence chain.

SWE_SEED should use CI outputs rather than claim to replace CI.

### 11.5 Human code review

**Why teams use it**

* judgment;
* architecture awareness;
* accountability;
* contextual understanding.

**Where it fails**

* reviewers become forensic investigators;
* routine proof collection consumes attention;
* missing context is discovered late;
* evidence varies by author and reviewer.

**SWE_SEED position**

> Human review should judge the change, not reconstruct whether basic proof happened.

### 11.6 Coding-agent platforms

**Why teams use them**

* integrated planning;
* tool use;
* interfaces;
* memory;
* orchestration;
* task tracking.

**Where they fall short**

* may require adopting one agent or platform;
* work protocols can remain proprietary;
* evidence formats vary;
* switching tools can erase workflow investment.

**SWE_SEED position**

> SWE_SEED is the harness around the agent, not a replacement for the agent.

### 11.7 Spec-driven development tools

**Why teams use them**

* clearer requirements;
* structured plans;
* better decomposition;
* reusable templates.

**Where they fall short**

* specifications may stop before proof;
* execution can diverge from the spec;
* host integration may be narrow;
* evidence may not return to the specification.

**SWE_SEED position**

> A specification becomes operational when it controls route, context, artifact, proof, and trace.

### 11.8 Workflow orchestrators

**Why teams use them**

* repeatable stages;
* automation;
* dependencies;
* retries;
* visible status.

**Where they fall short**

* orchestrate tasks without understanding software correctness;
* agent work may remain a black box;
* route and context selection are separate custom logic;
* workflow completion can be confused with software proof.

**SWE_SEED position**

> SWE_SEED defines the evidence-bearing software-work unit that an orchestrator may invoke.

### 11.9 AI evaluation platforms

**Why teams use them**

* model comparison;
* benchmarks;
* scoring;
* regression detection.

**Where they fall short**

* often evaluate generated outputs outside the live repository workflow;
* may not capture route, context, and tool-use conditions;
* benchmark success may not prove real task completion.

**SWE_SEED position**

> SWE_SEED generates structured work evidence that evaluation systems can score.

It does not replace broader evaluation infrastructure.

---

## 12. Positioning

### Positioning statement

For software teams using AI coding agents in real repositories, SWE_SEED is a proof-bearing execution protocol that converts each coding request into a route, bounded context, required artifact, proof obligations, and trace.

Unlike ad hoc prompts, duplicated instruction files, agent-specific workflows, or CI used alone, SWE_SEED keeps the work contract portable across supported agents and prevents the agent’s final message from standing in for repository evidence.

### Category ladder

Use categories in this order:

1. **Immediate problem:** coding agents claim completion before the work is proven;
2. **Technical category:** local CLI harness for coding-agent hosts;
3. **Product category:** verified agentic software-work harness;
4. **Market category:** proof-bearing execution protocol;
5. **GodSpeed category:** software-work proof layer of syntelligent infrastructure.

### Core contrast

```text
Typical agentic coding

prompt
→ agent changes files
→ agent reports completion
→ human investigates
```

```text
With SWE_SEED

work request
→ route
→ bounded context
→ required artifact
→ proof commands
→ trace
→ evidence handoff
```

### Product promise

> Keep the agent you prefer. Standardize the work it must perform and the proof it must leave behind.

### Strategic value

SWE_SEED helps teams move from:

* conversational tasks to work contracts;
* repository access to bounded context;
* improvised agent behavior to route-based execution;
* arbitrary output to required artifacts;
* completion summaries to proof records;
* host-specific instructions to canonical harness projections;
* transcript archaeology to structured trace;
* one-off prompt improvements to reviewed harness learning;
* vendor-specific workflow investment to portable software-work discipline.

---

## 13. The work contract

The work contract is SWE_SEED’s core object.

It should answer:

```text
What work was requested?
Which route applies?
Which context is required?
What artifact must exist?
Which proof commands must run?
What evidence must remain?
```

### Work request

The requested change should be specific enough to route.

It may include:

* intent;
* scope;
* constraints;
* risk;
* expected behavior;
* exclusions;
* acceptance criteria.

### Route

The route determines the required work pattern.

A route may specify:

* task type;
* applicable policies;
* context requirements;
* required skills;
* allowed tools;
* expected artifacts;
* proof set;
* trace obligations;
* escalation behavior.

### Context contract

The context contract identifies what the agent needs to know.

Context should be:

* task-relevant;
* source-addressed;
* versioned or hash-bound where supported;
* bounded;
* inspectable;
* reproducible.

Access to the repository is not itself a context contract.

### Artifact contract

The artifact contract states what the work must produce.

Examples:

* a code diff;
* a new module;
* a migration;
* an ADR;
* a test;
* a generated contract;
* a fixed configuration;
* a benchmark result;
* a documentation update.

### Proof contract

The proof contract states what evidence must be produced.

Examples:

* unit tests;
* integration tests;
* lint;
* type checks;
* static analysis;
* dependency checks;
* policy tests;
* security checks;
* replay tests;
* target-native validation;
* artifact hashes.

### Trace contract

The trace links:

* request;
* route;
* context;
* actions;
* artifacts;
* proof;
* result;
* failure or handoff.

The trace should make the work inspectable without making a raw conversation the only source of truth.

---

## 14. The core mechanism

SWE_SEED’s differentiation is not any individual instruction or hook. It is the complete proof-oriented work loop.

### 14.1 Normalize the request

Convert the task into a stable work contract.

Reject or escalate requests that are too ambiguous to route safely.

### 14.2 Select the route

Choose the route that determines:

* relevant policies;
* context;
* skills;
* artifacts;
* proof;
* trace requirements.

A route is not merely a suggested plan. It changes what evidence the task must produce.

### 14.3 Resolve context

Determine which repository and external references the task requires.

In integrated deployments, SWE_SEED requests context and consumes a context packet from the designated context authority.

SWE_SEED should not silently become a second competing context source.

### 14.4 Project to the active host

Render the canonical harness contract into the mechanisms supported by the selected coding agent:

* instruction files;
* skills;
* hooks;
* policy;
* memory views;
* task briefs;
* host configuration.

Host projections are derived artifacts.

They should not become independent sources of truth.

### 14.5 Enforce pre-action requirements

Where the host supports it, hooks and gates can prevent work from proceeding without required route or context state.

The exact enforcement strength depends on the host.

Do not market advisory instructions as equivalent to enforceable hooks.

### 14.6 Produce the artifact

The coding agent performs the requested work.

The required output remains explicit.

A plausible explanation without the artifact is not success.

### 14.7 Run proof

Execute the proof commands bound to the route and task.

Record:

* command;
* environment;
* result;
* relevant output;
* artifact association;
* failure state.

A missing required proof is a failed proof obligation, not an empty field to ignore.

### 14.8 Record the trace

Preserve structured evidence of:

* route selection;
* context references;
* agent host;
* artifact production;
* proof start;
* proof completion;
* failures;
* relevant learning candidates.

### 14.9 Hand off for judgment

SWE_SEED produces proof-bearing evidence.

It does not automatically own final settlement, authority, promotion, or capability classification.

Another reviewer or system may determine whether the evidence is sufficient to accept the outcome.

### Canonical loop

```text
Route the work.
Load the context.
Produce the artifact.
Run the proof.
Leave the trace.
```

---

## 15. Evidence is not settlement

This distinction must remain explicit.

### SWE_SEED proves

SWE_SEED can produce or record:

* route selection;
* context requirements;
* expected artifacts;
* proof obligations;
* proof results;
* traces;
* evaluation outputs;
* learning candidates.

### A settlement authority judges

Settlement determines whether:

* the declared outcome was actually achieved;
* the proof is sufficient;
* the evidence is reliable;
* the result can be accepted;
* the work qualifies for capability promotion.

### Canonical distinction

> SWE_SEED makes the claim falsifiable. It does not grant the claim final standing.

### Bad framing

> SWE_SEED decides whether the work is complete.

This collapses proof production and settlement authority.

### Better framing

> SWE_SEED produces the route, artifact, proof, and trace needed to judge whether the work is complete.

### GodSpeed stack boundary

> DomainForge defines the domain.
> SEA Forge governs the work.
> SWE_SEED proves the change.
> GodSpeed-Agent compounds the capability.

Expanded:

* DomainForge supplies executable domain meaning.
* Context Kernel supplies bounded, cited context.
* SEA Forge decides authority before consequential action.
* SWE_SEED structures execution and produces software proof.
* GodSpeed-Agent records settlement and capability change.
* Developmental memory preserves what repeated evidence justifies.

---

## 16. Product and architecture boundaries

### SWE_SEED owns

* canonical software-work contracts;
* route logic;
* context requirements;
* host projections;
* harness policies;
* hooks and host adapters;
* expected artifact definitions;
* proof obligations;
* proof execution records;
* traces;
* harness drift checks;
* rollback state;
* learning candidates and regression links;
* repository-local harness state.

### SWE_SEED does not own

* the coding model;
* the host agent runtime;
* runtime authorization for all side effects;
* final settlement classification;
* capability promotion;
* organizational memory as a whole;
* canonical domain meaning;
* a universal context database;
* production deployment authority;
* every CI or security system.

### Host coding agents own

* conversation and local interaction;
* model calls;
* tool invocation;
* implementation behavior;
* host-specific UI;
* host-specific permission surfaces.

### Context Kernel owns

In the intended integrated architecture:

* real-time context authority;
* context packet creation;
* citation and provenance;
* context continuity;
* context retrieval policy.

SWE_SEED requests and consumes context.

It should not independently create a competing canonical context path.

### SEA Forge owns

* pre-action authority;
* governed side effects;
* approval and escalation;
* sandbox and execution boundaries;
* evidence ledgering;
* runtime integrity;
* governed delegation;
* policy enforcement.

### GodSpeed-Agent owns

* settlement classification;
* developmental navigation;
* payment and burden comparison;
* capability updates;
* repetition planning;
* metabolization;
* horizon change.

### CI and scanners own

* their specialized checks;
* independent execution environments;
* release gates;
* security findings;
* build and deployment verification.

SWE_SEED binds and records their outputs as proof where appropriate.

### ACP integration

In integrated SEA Forge deployments, an SWE_SEED-harnessed coding agent may be driven through ACP.

SEA Forge can harvest SWE_SEED route and proof artifacts and cross-link them to a governed run.

This integration does not turn SWE_SEED into the agent runtime or authority layer.

---

## 17. Canonical and derived state

SWE_SEED should maintain one canonical harness state.

### Canonical

Depending on the current repository architecture, canonical state may include:

* BAML-backed work contracts;
* route definitions;
* policy definitions;
* skills;
* hook contracts;
* proof definitions;
* repository harness objects;
* host adapter configuration;
* approved learning promotions.

### Derived

Derived outputs may include:

* `AGENTS.md`;
* host instruction files;
* memory views;
* briefs;
* hook configuration;
* generated skills;
* benchmark outputs;
* rendered host policies;
* adapter-specific files.

### Core rule

> Edit the harness source. Regenerate the host projections.

Do not permit every host file to become an independently maintained version of the operating model.

### Drift

SWE_SEED should detect when:

* a generated host file changes;
* a projection no longer matches canonical state;
* a content hash changes unexpectedly;
* a hook configuration is stale;
* a required host projection is missing;
* host behavior diverges from parity expectations.

### Why this matters

A portable harness only remains portable when host configuration is derived rather than copied and forgotten.

---

## 18. Standalone and integrated modes

### Standalone mode

SWE_SEED should remain useful without the full GodSpeed stack.

In standalone mode it can:

* route work;
* project host configuration;
* require context references;
* define artifacts;
* run proof;
* record trace;
* detect harness drift;
* produce reviewable evidence.

Standalone use is the lowest-burden adoption path.

### Integrated context mode

SWE_SEED requests a bounded context packet from Context Kernel or another declared context provider.

### Governed mode

SEA Forge mediates consequential agent actions and records authority and evidence.

SWE_SEED supplies the software-work route and proof artifacts.

### Developmental mode

GodSpeed-Agent consumes evidence and settlement results to determine:

* what worked;
* what burden was paid;
* what should repeat;
* what may become reusable capability.

### Federation

Federation should remain opt-in.

Do not imply that standalone SWE_SEED requires a network service, centralized database, or enterprise control plane.

---

## 19. Feature-to-outcome translation

Do not market features without their consequence.

| Capability                 | Reader consequence                                                                                              |
| -------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Route selection            | The agent follows the work pattern required by the task rather than improvising from one prompt.                |
| Bounded context contract   | The agent receives the decisive repository information without treating the whole codebase as equally relevant. |
| Host projection            | Teams keep one work protocol across different coding agents.                                                    |
| Canonical harness state    | Agent-specific configuration does not become several competing operating models.                                |
| Required artifact          | A convincing explanation cannot substitute for the requested file, diff, test, or contract.                     |
| Predeclared proof commands | The agent cannot quietly choose only the easiest checks after seeing the result.                                |
| Proof events               | The team can see which commands ran, what passed, and what failed.                                              |
| Route gate                 | Missing work structure fails explicitly instead of silently becoming improvisation.                             |
| Hooks                      | Host-supported pre-action rules can move beyond advisory prose.                                                 |
| Trace                      | Reviewers can follow the work from request through evidence without reconstructing the session manually.        |
| Drift checks               | Generated host configuration cannot diverge unnoticed from the canonical harness.                               |
| Rollback snapshots         | A failed projection or configuration change can be recovered.                                                   |
| Learning candidates        | Repeated failures can propose improvements instead of disappearing into transcripts.                            |
| Regression links           | A new harness rule can remain connected to the failure that justified it.                                       |
| Cross-host parity checks   | Teams can distinguish host differences from task and model differences.                                         |
| Local-first operation      | Source and proof can remain close to the repository rather than requiring a hosted coding platform.             |

---

## 20. Differentiators

### 20.1 It improves the agent you already use

SWE_SEED does not require one model or assistant to become the company standard.

The harness is designed to project into supported host environments.

### 20.2 It treats software work as a contract

The task has an explicit:

* route;
* context;
* artifact;
* proof;
* trace.

This makes the work falsifiable.

### 20.3 It separates narration from evidence

The agent’s summary remains useful.

It does not receive automatic standing as proof.

### 20.4 It brings proof into the work loop

Proof is not only a downstream CI event.

The work contract declares what must be checked before the claim can advance.

### 20.5 It keeps host configuration derived

Agent-specific files are projections of the harness rather than several manually maintained sources.

### 20.6 It remains local and lightweight

SWE_SEED is a CLI harness, not a hosted agent platform.

It does not need to become the repository’s network control plane.

### 20.7 It creates structured residue

Routes, context references, artifacts, proofs, traces, and learning candidates can survive the individual session.

### 20.8 It preserves product boundaries

SWE_SEED does not pretend:

* proof is authority;
* evidence is settlement;
* passing tests equal capability;
* context access equals context quality;
* agent output equals repository truth.

### 20.9 It supports evidence-based harness improvement

Harness changes can be tied to observed failures and regression proof rather than prompt folklore.

---

## 21. Messaging hierarchy

Explain SWE_SEED in this order.

### Message 1: The familiar failure

> Coding agents can write a convincing completion summary before the repository can support the claim.

### Message 2: The correction

> The final message is not the work product, and a diff without required proof is not a verified change.

### Message 3: The mechanism

> SWE_SEED gives each task a route, bounded context, required artifact, proof commands, and trace.

### Message 4: The practical result

> Developers can use their preferred coding agents without rebuilding the work protocol or investigating every claim from scratch.

### Message 5: The platform value

> Teams can compare agentic work, standardize proof, and improve the harness from evidence.

### Message 6: The larger stack

> SWE_SEED supplies the proof layer between governed action and capability settlement.

Do not begin with Message 6.

---

## 22. Approved message formulations

### Primary headline

> Make agentic coding prove its work.

### Alternative headline

> The agent can claim anything. The repository has to prove it.

### Product-led headline

> A work contract for every coding-agent task.

### Primary subhead

> SWE_SEED gives coding agents a route, bounded context, required artifact, proof commands, and trace before anyone gets to call the work done.

### Platform subhead

> Keep Claude Code, Codex, OpenCode, Copilot, or another supported host. Project one verified software-work protocol across them.

Use only hosts supported by the current repository.

### Plain one-liner

> SWE_SEED makes coding agents show what they changed and how they proved it.

### Technical one-liner

> A local-first CLI harness for route-based, context-bound, artifact-producing, proof-checked agentic software work.

### Enterprise one-liner

> SWE_SEED standardizes how coding agents receive work, produce evidence, and hand changes into review.

### Developer one-liner

> Route the task. Load the context. Produce the artifact. Run the proof. Leave the trace.

### Quality one-liner

> A completion claim without the required proof remains an open claim.

### Host-portability one-liner

> Change the agent without throwing away the engineering method.

### Sharp lines

* The final message is not the build result.
* A passing summary is not a passing test.
* The agent did not finish because it ran out of things to say.
* Repository access is not the same as relevant context.
* A plan that does not constrain evidence is a suggestion.
* Proof chosen after the diff is vulnerable to convenience.
* If the artifact is missing, the explanation is irrelevant.
* If the required check did not run, “tested” has no standing.
* Agent memory is not team infrastructure.
* The model can change. The work contract should survive.
* Stop grading coding agents on confidence.
* Prompts describe the job. Evidence closes the claim.
* SWE_SEED proves. GodSpeed-Agent settles.

Use sharp lines sparingly. Do not make every paragraph a manufactured punchline.

---

## 23. Voice and tone

SWE_SEED copy should sound:

* direct;
* developer-native;
* procedural;
* skeptical of completion theater;
* respectful of coding agents as useful tools;
* grounded in repository evidence;
* lightweight rather than enterprise-heavy;
* precise about boundaries.

The voice should sound like:

> A senior engineer who is happy to use coding agents but still checks the diff, runs the tests, and refuses to promote confidence into evidence.

### Use Speed Mode for

* headlines;
* README openers;
* completion-claim contrasts;
* developer launch content;
* agent comparison content.

### Use Journey Mode for

* architecture;
* host support;
* context contracts;
* proof semantics;
* integrations;
* evidence boundaries;
* maturity claims.

### Humor boundary

Mock the operating absurdity, not developers adopting the tools.

Good:

> Asking the same system to write the code, choose the tests, and certify completion is an efficient way to automate optimism.

Bad:

> Developers trusting coding agents are careless.

---

## 24. Language rules

### Prefer

* work contract;
* route;
* context;
* artifact;
* proof;
* trace;
* harness;
* host;
* projection;
* requirement;
* acceptance criterion;
* command;
* evidence;
* diff;
* repository;
* explicit;
* missing;
* passing;
* failing;
* derived;
* canonical;
* drift;
* rollback;
* learning candidate.

### Explain before relying on

* BAML;
* harness projection;
* route artifact;
* proof event;
* Context Kernel;
* ACP;
* settlement;
* capability metabolization;
* federation;
* syntelligent infrastructure.

### Avoid

* autonomous software engineer;
* replaces developers;
* fully autonomous coding;
* guarantees correct code;
* self-improving without qualification;
* zero-review development;
* seamless;
* revolutionary;
* supercharge;
* unlock;
* effortless;
* AI-powered development platform;
* complete software factory;
* universal agent support;
* verified when only agent narration exists;
* settled when only proof has been produced.

### Canonical distinctions

Do not cycle among these terms as though they mean the same thing:

* **Request:** the work someone wants performed.
* **Route:** the work pattern selected for the request.
* **Context:** the bounded information required for the route.
* **Artifact:** the output the task must produce.
* **Proof:** evidence generated by running declared checks.
* **Trace:** the structured record connecting the work stages.
* **Evaluation:** analysis of the evidence against a rubric or check.
* **Settlement:** the later judgment that the outcome counts.
* **Capability:** a pattern proven through repeated settlement under variation.

---

## 25. Common objections

### “My agent already runs tests.”

Response:

That is useful, but the agent may choose which tests to run after seeing the change.

SWE_SEED binds the required proof to the task route, records what ran, and treats missing proof as a visible failure.

### “We already use `AGENTS.md`.”

Response:

Keep it.

SWE_SEED can treat host instruction files as generated projections of a broader harness contract that also includes route, context, artifacts, proof, hooks, drift checks, and trace.

### “We already have CI.”

Response:

Keep CI.

CI provides independent repository verification. SWE_SEED connects the task request, selected route, context, artifact, and required proof before the work reaches CI.

The two reinforce each other.

### “This sounds like more process.”

Response:

It is more structure than a raw prompt.

The question is whether that structure costs less than repeated prompting, false completion claims, forensic code review, reruns, and inconsistent agent behavior.

The smallest deployment should begin with one task class and one proof set rather than attempting to model the entire SDLC.

### “Will it slow the agent down?”

Response:

Required context selection and proof consume time.

They should replace wasted exploration, irrelevant edits, skipped checks, and human reconstruction.

Do not promise net speed improvement without comparative evidence.

### “Why not build this into one agent?”

Response:

A host-specific solution may work well for one agent.

SWE_SEED preserves the software-work method outside any one model or vendor and projects it into supported hosts.

### “Does SWE_SEED verify that the software is correct?”

Response:

It verifies declared proof obligations and produces evidence.

No finite proof set establishes universal correctness.

The strength of the claim depends on:

* the route;
* the artifact contract;
* the selected checks;
* the environment;
* the evidence;
* later review or settlement.

### “Does SWE_SEED approve or reject the final outcome?”

Response:

SWE_SEED records proof and evaluation evidence.

Final settlement belongs to a reviewer or settlement authority such as GodSpeed-Agent in the integrated stack.

### “Does SWE_SEED govern file writes and tool calls?”

Response:

SWE_SEED can project policies and host hooks where supported.

Universal runtime authority belongs to SEA Forge.

Do not describe host instructions as equivalent to a separate authority boundary.

### “Does it require the full GodSpeed stack?”

Response:

No.

SWE_SEED should remain useful standalone.

Context Kernel, SEA Forge, and GodSpeed-Agent add stronger context authority, runtime governance, settlement, and capability learning.

### “Does it require a server?”

Response:

The current architecture is a CLI harness with repository-local state and no required network surface of its own.

Integrated services may add networked capabilities, but they are not required for the basic harness.

### “Will it work with every coding agent?”

Response:

Only claim support for hosts with current adapters, parity tests, and verified projection behavior.

Do not convert architectural portability into universal compatibility.

### “Can it learn automatically?”

Response:

SWE_SEED can record learning candidates and connect improvements to evidence.

Durable promotion should require review and regression proof.

Do not market unreviewed automatic self-modification as a feature.

### “Is a passing proof enough to merge?”

Response:

Not necessarily.

Authority, human review, security policy, release controls, and settlement may still block promotion.

Software proof is necessary for many changes. It is not the only control.

---

## 26. Proof inventory

Marketing may point to these surfaces when they exist and pass on the current branch.

### Canonical contract proof

* BAML-backed harness contracts;
* parity between generated and runtime types;
* route schemas;
* context requirements;
* artifact requirements;
* proof definitions;
* canonical repository objects.

### Host projection proof

* supported host adapters;
* `sync` or projection commands;
* generated instruction files;
* generated hooks;
* generated skills;
* deterministic projection output;
* projection hashes;
* projection drift detection;
* rollback snapshots.

### Route proof

* route selection;
* fail-closed route gate;
* route artifact;
* route-to-policy mapping;
* route-to-proof mapping;
* route regression fixtures.

### Context proof

* context request;
* context manifest or packet;
* citations or source references;
* content-hash bindings where implemented;
* missing-context failure behavior;
* Context Kernel contract tests where integrated.

### Artifact proof

* expected artifact declaration;
* artifact existence checks;
* diff capture;
* file or output hashes;
* generated-contract classification;
* target-native validation where applicable.

### Proof proof

* `ProofStarted`;
* `ProofCompleted`;
* command details;
* exit result;
* check output;
* test, lint, type-check, static-analysis, dependency, policy, security, or replay proof;
* required-proof missing behavior;
* stale-proof detection where implemented.

### Trace proof

* request-to-route link;
* route-to-context link;
* context-to-artifact link;
* artifact-to-proof link;
* host and tool metadata;
* failure trace;
* reviewable evidence bundle.

### Harness-control proof

* doctor or environment checks;
* drift detection;
* deterministic projection lock;
* rollback;
* source-of-truth checks;
* content-hash bindings;
* composite pre-action enforcement;
* parity vectors across implementations.

### Learning proof

* learning candidates;
* review workflow;
* durable promotion controls;
* regression links;
* repeat-failure benchmarks;
* evidence connecting a new harness rule to the failure that justified it.

### Integration proof

When implemented and passing:

* Context Kernel context packets;
* SEA Forge authority decisions;
* ACP-driven host sessions;
* SWE_SEED artifact harvesting;
* evidence cross-linking;
* GodSpeed-Agent settlement;
* capability update;
* end-to-end event ownership.

Only market proof that can be reproduced from the current repository.

---

## 27. Claims and maturity boundaries

### Safe evidence-backed positioning

Use when confirmed by the current repository:

* SWE_SEED is a CLI harness.
* SWE_SEED projects harness behavior into supported coding-agent hosts.
* SWE_SEED records repository-local harness artifacts.
* SWE_SEED supports route, trace, proof, and evaluation workflows.
* SWE_SEED treats host configurations as derived projections.
* SWE_SEED supports implemented hooks, skills, policies, drift checks, and rollback behavior.
* SWE_SEED is usable without replacing the host coding agent.
* SWE_SEED does not own final settlement or the capability lifecycle.

### Claims requiring qualification

* verified software work;
* fail closed;
* host independent;
* model agnostic;
* portable across agents;
* learning harness;
* self-improving;
* complete traceability;
* local and private;
* reproducible;
* ACP compatible;
* Context Kernel integrated;
* SEA Forge integrated;
* enterprise ready;
* regulated-software ready.

State the precise supported host, path, proof, and integration boundary.

### Roadmap or strategic claims

Do not present these as complete without current proof:

* universal coding-agent support;
* complete cross-host behavioral parity;
* automatic context authority;
* fully automatic learning promotion;
* independent settlement;
* enterprise-wide harness control plane;
* universal pre-action enforcement;
* production-proven ACP integration across all hosts;
* benchmark-driven automatic route recomposition;
* complete software-development lifecycle orchestration;
* autonomous capability compounding.

---

## 28. Licensing

Use the current SWE_SEED repository license as the legal source of truth.

Do not infer SWE_SEED’s license from DomainForge or SEA Forge.

Before publishing licensing copy, verify:

* license name;
* redistribution terms;
* commercial-use terms;
* contribution terms;
* generated-artifact treatment;
* bundled third-party components.

Until verified, approved marketing language is limited to:

> See the repository license for current use, modification, redistribution, and commercial terms.

Do not call SWE_SEED open source, source available, permissive, copyleft, or commercially restricted unless the current license supports that exact statement.

---

## 29. Adoption path

### Stage 1: Prove one task

Choose:

* one repository;
* one supported agent;
* one common task type;
* one route;
* one context set;
* one required artifact;
* one proof command.

Run the task and inspect the trace.

### Stage 2: Prove failure behavior

Test:

* missing route;
* missing context;
* missing artifact;
* failing proof;
* agent completion claim despite failed proof.

The harness is credible when failure is explicit.

### Stage 3: Project into a second host

Run the same work contract through another supported coding agent.

Compare:

* projection;
* route;
* context;
* artifact;
* proof;
* trace.

### Stage 4: Add real proof sets

Add:

* tests;
* lint;
* type checks;
* static analysis;
* dependency checks;
* policy checks;
* security checks;
* replay or recovery tests.

### Stage 5: Integrate existing CI

Use CI and scanner outputs as additional proof rather than creating parallel replacements.

### Stage 6: Add context authority

Connect Context Kernel or another declared context provider.

Require cited or hash-bound context.

### Stage 7: Add runtime authority

Connect SEA Forge for consequential file, command, API, commit, merge, secret, or deployment actions.

### Stage 8: Add settlement and capability learning

Connect GodSpeed-Agent or another settlement authority.

Compare predicted and actual burden.

Promote reusable patterns only after reviewed settlement.

### Stage 9: Standardize across teams

Create shared route and proof templates with controlled repository- or team-specific variation.

Do not begin with every repository and every agent.

---

## 30. Primary use cases

### Repository feature work

Route a feature task, load relevant architecture and domain context, require the implementation artifact, and run the proof set.

### Bug repair

Bind the task to:

* reproduction;
* failing test;
* repair;
* passing regression;
* relevant recovery checks.

### Refactoring

Require:

* preserved behavior;
* architecture constraints;
* dependency checks;
* benchmark or complexity proof where relevant.

### Dependency changes

Require:

* manifest update;
* lockfile update;
* license and advisory checks;
* build proof;
* affected integration tests.

### Generated code or contracts

Require:

* source change;
* deterministic regeneration;
* generated-zone integrity;
* target-native validation;
* stable output hashes where appropriate.

### Security-sensitive work

Require:

* bounded context;
* security policy;
* protected-path handling;
* static analysis;
* secret scanning;
* dependency review;
* explicit authority integration where actions are consequential.

### Regulated software changes

Require:

* policy and provenance context;
* artifact traceability;
* quality and security proof;
* review handoff;
* evidence package.

Do not claim the evidence package is itself regulatory certification.

### Cross-agent comparison

Run the same work contract through several supported hosts to compare actual performance under the same route, context, artifact, and proof constraints.

### Harness improvement

Turn a repeated failure into:

* a proposed route change;
* a new context requirement;
* a mandatory proof;
* a host projection fix;
* a regression test.

---

## 31. The verified agentic change evidence pack

A strong market artifact is a **Verified Agentic Change Evidence Pack**.

It should show:

### Work contract

* task;
* route;
* acceptance criteria;
* required context;
* expected artifact;
* proof obligations.

### Execution record

* agent host;
* relevant tools;
* route decision;
* context references;
* artifact or diff;
* errors and interruptions.

### Proof record

* commands;
* results;
* output references;
* missing or failed proof;
* evidence hashes where supported.

### Decision boundary

* what SWE_SEED proved;
* what remained for review;
* whether authority was separately granted;
* whether settlement occurred;
* whether the result qualified for promotion.

### Comparative proof

Where useful, run the same task:

* without SWE_SEED;
* with SWE_SEED;
* through a second agent host.

Measure:

* missing proof;
* review effort;
* irrelevant changes;
* retries;
* task completion;
* reproducibility;
* evidence completeness.

Do not claim superiority before running comparative proof.

---

## 32. README-specific guidance

The README must not begin with architecture terminology or a feature inventory.

Its upper section should answer:

1. What is broken about ordinary coding-agent completion?
2. Why is the agent’s final message insufficient?
3. What five things does SWE_SEED add?
4. Does it replace the coding agent?
5. What can the reader run in one repository?
6. What artifact proves the harness did something useful?

### Recommended upper-page sequence

1. problem-led headline;
2. plain-language explanation;
3. final-message-versus-repository contrast;
4. route-context-artifact-proof-trace loop;
5. smallest working example;
6. supported hosts;
7. generated harness artifacts;
8. failure demonstration;
9. standalone and integrated architecture;
10. technical reference.

### Ten-second outcome

The reader should understand:

> SWE_SEED gives coding agents a work contract and requires repository proof before their completion claim can advance.

### Sixty-second outcome

The reader should understand:

* SWE_SEED is not another coding agent;
* it works through supported host agents;
* the task receives a route;
* context is bounded;
* an artifact is required;
* proof commands are predeclared;
* trace is preserved;
* missing proof fails visibly;
* final settlement belongs elsewhere.

### Primary developer action

> Run one routed task and inspect the route, artifact, proof, and trace.

### Secondary actions

* project into a supported host;
* run the doctor or drift check;
* compare two host projections;
* create a route;
* add a proof set;
* inspect a failed task;
* review the integration boundaries.

---

## 33. Content acceptance tests

Before publishing SWE_SEED copy, verify that the reader can answer:

1. What failure does SWE_SEED prevent?
2. Why is the agent’s final message not sufficient?
3. What is a work contract?
4. What does a route control?
5. What is bounded context?
6. What artifact must the task produce?
7. Who chooses the proof commands?
8. What happens when proof is missing or fails?
9. What does the trace contain?
10. Does SWE_SEED replace the coding agent?
11. Does it require a server?
12. Does it replace CI?
13. Does it govern runtime authority?
14. Does it settle the final outcome?
15. How does it relate to Context Kernel?
16. How does it relate to SEA Forge?
17. How does it relate to GodSpeed-Agent?
18. Which hosts are currently supported?
19. Which claims are implemented and which remain roadmap?
20. What should the reader run next?

Apply five final tests.

### So what?

Does every capability connect to lower verification burden, fewer false completion claims, portable agent workflows, better traceability, or improved harness learning?

### Prove it

Can every product claim be reproduced from the current repository?

### Boundary

Does the copy separate proof from authority, settlement, and capability?

### Portability

Does the copy explain how canonical harness state differs from generated host configuration?

### Compression

Can a developer explain SWE_SEED in one sentence after reading the opening?

---

## 34. Final strategic summary

SWE_SEED should not be marketed as another coding agent, autonomous developer, workflow orchestrator, CI replacement, or settlement system.

The immediate problem is simple:

```text
agent receives task
→ agent changes code
→ agent reports completion
→ human reconstructs whether the claim is true
```

SWE_SEED changes the work:

```text
request
→ route
→ bounded context
→ required artifact
→ proof commands
→ trace
→ evidence handoff
```

Its smallest useful promise is:

> Keep your coding agent. Make every task leave a route, artifact, proof, and trace.

Its product category is:

> Verified agentic software-work harness.

Its stronger strategic category is:

> A proof-bearing execution protocol for agentic software delivery.

Its role in the GodSpeed stack is precise:

> DomainForge defines the domain.
> SEA Forge governs the work.
> SWE_SEED proves the change.
> GodSpeed-Agent compounds the capability.

Lead with the false completion claim.

Bind the work.

Run the proof.

Leave the trace.
