# Strategy Layer Specification v0.1.0

## Purpose

This specification defines a standalone Strategy Layer for converting need signals, JTBDs, market observations, customer evidence, existing purchase behavior, capability assumptions, competitive context, and research findings into testable strategic choices.

The Strategy Layer exists to help an operator decide:

- what is worth pursuing,
- where to play,
- how to win,
- what must be true,
- what information is missing,
- what research must be delegated,
- what evidence is needed,
- what should be tested,
- what should be rejected,
- what should be fabricated,
- and when to commit, defer, or stop.

The Strategy Layer does not build prototypes by default. It decides which strategic possibilities deserve additional research, testing, customer discovery, fabrication, or commitment.

The Strategy Layer may generate prompts for dedicated research agents when the available information is insufficient for a sound strategic decision.

The desired outcome is decision quality under uncertainty.

The Strategy Layer MUST be self-contained within `.strategy/`.

It MUST be possible for another agent to regenerate and use the layer from the strategy-folder
artifacts alone, without depending on the harness, fabricator, or SWE Seed runtime.

Outer layers MAY accommodate the Strategy Layer through `just` aliases, wrappers, or optional
metadata fields, but the Strategy Layer itself MUST remain usable when copied into another
repository with only its own folder contents.

A contributor or agent should know:

- what strategic question is being answered,
- what options are being compared,
- what assumptions each option depends on,
- what evidence supports or weakens each option,
- what information is missing,
- what research prompt should be delegated,
- what test should be run next,
- what decision is currently justified,
- and what level of confidence is warranted.

---

## Core Principle

```text
Strategy is not a vision, plan, goal, or narrative.
Strategy is an integrated set of choices under uncertainty.
```

A strategy option is not valid unless it states:

- the winning aspiration,
- the chosen arena,
- the way to win,
- the required capabilities,
- the required management systems,
- the assumptions that must hold,
- the evidence currently available,
- the evidence missing,
- the research required,
- the tests required,
- the decision status.

A strategy with no trade-offs is not a strategy.

A strategy with no evidence is a hypothesis.

A strategy with no test plan is speculation.

A strategy with missing information must produce a research request before pretending to know.

A strategy with no decision record is theater.

## Self-Containment And Runtime Independence

The reference implementation for v0.1 SHOULD live under `.strategy/` itself.

Preferred runtime surface:

```text
python .strategy/strategy.py <command>
```

Outer layers MAY add `just strategy-*` aliases, but those aliases are accommodation only and MUST
NOT become the Strategy Layer's sole execution surface.

The Strategy Layer SHOULD use the project's canonical Python environment when one already exists.
If no canonical environment exists and the implementation requires a virtual environment, it MAY
create one inside `.strategy/`.

If the implementation can run with the Python standard library alone, it SHOULD avoid creating a
dedicated environment.

---

## Required Command Interface

A conforming implementation SHOULD expose strategy commands through `just`, a project CLI, or simple scripts.

Preferred command surface:

```text
just strategy-new <question>
just strategy-capture <source>
just strategy-generate-options <brief_id>
just strategy-identify-gaps <option_id>
just strategy-generate-research-prompts <option_id>
just strategy-ingest-research-report <report_path>
just strategy-design-tests <option_id>
just strategy-record-evidence <test_id>
just strategy-evaluate <option_id>
just strategy-decide <option_id>
just strategy-status <option_id>
```

Equivalent commands are allowed, but the implementation MUST document the selected command names and proof obligations.

At minimum, v0.1 MUST support this file-first workflow:

```text
NeedSignal / StrategicQuestion
→ StrategyOption
→ ConditionsToWin
→ EvidenceGapReport
→ ResearchPromptPacket
→ ResearchReport
→ StrategyEvidence
→ StrategyTests
→ StrategyDecisionRecord
```

The command or manual workflow MUST fail clearly when required artifacts are missing or incomplete.

---

## Required Scaffold

A conforming standalone Strategy Layer SHOULD use this folder structure:

```text
.strategy/
  STRATEGY_LAYER_SPEC_v0.1.0.md
  config.yaml
  strategy.py

  briefs/
    <brief-id>.md

  need-signals/
    <need-signal-id>.yaml

  options/
    <strategy-option-id>.yaml

  gaps/
    <evidence-gap-report-id>.md

  research-prompts/
    <research-prompt-packet-id>.md

  research-reports/
    <research-report-id>.md

  tests/
    <strategy-test-id>.yaml

  evidence/
    <evidence-id>.md

  decisions/
    <decision-id>.md

  research/
    <research-synthesis-id>.md

  backlog/
    strategy-experiment-backlog.md

  schemas/
    need-signal.schema.yaml
    strategic-question.schema.yaml
    strategy-option.schema.yaml
    evidence-gap-report.schema.yaml
    research-prompt-packet.schema.yaml
    research-report.schema.yaml
    strategy-test.schema.yaml
    strategy-evidence.schema.yaml
    strategy-eval-spec.schema.yaml
    strategy-decision-record.schema.yaml
    ugc-signal.schema.yaml
    research-synthesis.schema.yaml
    purchase-evidence.schema.yaml

  templates/
    STRATEGY_BRIEF.md
    NEED_SIGNAL.yaml
    STRATEGY_OPTION.yaml
    EVIDENCE_GAP_REPORT.md
    RESEARCH_PROMPT_PACKET.md
    RESEARCH_REPORT.md
    STRATEGY_TEST.yaml
    STRATEGY_EVIDENCE.md
    STRATEGY_DECISION_RECORD.md
    RESEARCH_SYNTHESIS.md
    STRATEGY_EXPERIMENT_BACKLOG.md
```

Equivalent paths are allowed, but the implementation MUST document them.

The self-contained layer MUST NOT import helper code from outer-layer runtime files merely because
they exist in the current repository. Copying or re-implementing the minimal local helper logic is
preferred over taking a hidden dependency on external project files.

The scaffold MUST remain lightweight. Do not add dashboards, databases, hosted tools, vector memory, or multi-agent orchestration in v0.1.

---

## Strategy Artifact Chain

A conforming strategy process MUST preserve this chain:

```text
Need Signals
→ Strategic Question
→ Winning Aspiration
→ Where-to-Play Options
→ How-to-Win Options
→ Capability Map
→ Management System Map
→ Conditions That Must Be True
→ Evidence Gap Report
→ Research Prompt Packet
→ Research Report
→ Research Synthesis
→ Strategy Tests
→ Strategy Evidence
→ Strategy Eval
→ Strategy Decision Record
→ ProductSeed / Fabrication Candidate / No-Build Decision
```

A downstream artifact is invalid if it cannot trace to upstream evidence, assumption, condition, research gap, or explicit waiver.

---

## Canonical Artifacts

### NeedSignal

A `NeedSignal` is a raw observation of a human, market, organizational, operational, or technical need.

It SHOULD capture:

- exact language,
- situation,
- struggle,
- current workaround,
- desired outcome,
- constraint,
- system friction,
- emotional intensity,
- frequency,
- cost,
- evidence source,
- confidence.

### StrategicQuestion

A `StrategicQuestion` defines the decision being answered.

Examples:

```text
Which first market wedge should this system pursue?
Should we target solo builders, consulting clients, or internal AI teams first?
Is this problem strong enough to fabricate a prototype?
```

### StrategyOption

A `StrategyOption` combines aspiration, where to play, how to win, capabilities, management systems, assumptions, tests, evidence, evidence gaps, research requests, and decision status.

### EvidenceGapReport

An `EvidenceGapReport` states what is missing before a sound strategic decision can be made.

It MUST include:

- missing information,
- affected strategy option,
- affected condition that must be true,
- why the information matters,
- decision blocked by the gap,
- recommended research type,
- evidence level required,
- research prompt needed.

### ResearchPromptPacket

A `ResearchPromptPacket` is a set of prompts that the operator can give to dedicated research agents.

It MUST include:

- research objective,
- strategic context,
- condition being tested,
- specific questions,
- source priorities,
- evidence requirements,
- exclusion criteria,
- anti-bias instructions,
- required output structure,
- citations requirement,
- decision use.

### ResearchReport

A `ResearchReport` is an output produced by a dedicated research agent.

It SHOULD include:

- executive summary,
- methods,
- sources,
- findings,
- evidence supporting,
- evidence weakening,
- confidence,
- limitations,
- recommended next research,
- implications for strategy.

### ResearchSynthesis

A `ResearchSynthesis` converts one or more ResearchReports into strategic implications.

It MUST separate:

- facts,
- interpretations,
- assumptions,
- evidence strength,
- unanswered questions,
- decision implications.

---

## Strategic Choice Cascade

Every complete strategy option MUST answer five questions:

1. **Winning Aspiration**
   What does winning mean? Why does this strategy exist?

2. **Where to Play**
   Which market, segment, user, channel, geography, use case, workflow, or value-chain stage will be targeted?

3. **How to Win**
   What unique approach creates advantage in the chosen arena?

4. **Core Capabilities**
   What must be done exceptionally well to deliver the how-to-win choice?

5. **Management Systems**
   What processes, metrics, cadence, governance, and resource allocation will reinforce the strategy?

Rules:

- A how-to-win choice without a where-to-play choice is incomplete.
- A where-to-play choice without a how-to-win choice is incomplete.
- A strategy without capabilities is wishful thinking.
- A strategy without management systems is not operational.
- A strategy with no explicit trade-offs is incomplete.
- All five choices must reinforce one another.

---

## StrategyOption Schema

```yaml
strategy_option:
  id:
  title:
  status: draft | research_needed | testing | committed | rejected | deferred | fabrication_candidate

  strategic_question:
  winning_aspiration:

  where_to_play:
    segment:
    user_or_buyer:
    use_case:
    channel:
    geography:
    value_chain_stage:
    constraints:
    excluded_segments:

  how_to_win:
    value_proposition:
    differentiation_or_cost_position:
    unfair_advantage:
    proof_of_value:
    tradeoffs:
    alternatives_rejected:

  capabilities_required:
    - capability:
      importance: high | medium | low
      current_strength: strong | adequate | weak | missing
      obtain_by: build | partner | outsource | acquire | avoid
      evidence:

  management_systems_required:
    - system:
      purpose:
      metric:
      cadence:
      owner:
      evidence:

  conditions_that_must_be_true:
    - id:
      condition:
      category: customer | competition | capability | cost | channel | timing | regulatory | operational | purchase_behavior
      confidence: high | medium | low
      evidence_currently_available:
      evidence_missing:
      test_required:
      research_required:

  evidence_gaps:
    - gap_id:
      missing_information:
      affected_condition_id:
      decision_blocked:
      recommended_research_type:
      required_evidence_level:

  tests:
    - id:
      condition_id:
      method: analytical_research | ugc_social_listening | jtbd_interview | design_research | competitor_analysis | category_mapping | purchase_behavior_research | prototype_pilot | sales_commitment | cost_model | capability_audit
      pass_criteria:
      fail_criteria:
      owner:
      due_date:

  evidence:
    - source:
      evidence_type:
      summary:
      supports:
      weakens:
      confidence:

  decision:
    status: commit | test | defer | reject | fabricate | collect_more_evidence | delegate_research
    reason:
    next_action:
```

---

## Reverse Engineering Strategic Options

The Strategy Layer uses reverse engineering to prevent premature commitment.

Process:

1. Generate strategic possibilities.
2. Specify conditions that must be true.
3. Identify barriers and concerns.
4. Identify missing information.
5. Generate research prompts for missing information.
6. Ingest research reports.
7. Design tests for the highest-risk conditions.
8. Run tests.
9. Record evidence.
10. Decide: commit, test more, defer, reject, fabricate, collect more evidence, or delegate additional research.

The goal is not to prove the favorite option right.

The goal is to discover which option survives evidence.

---

## Evidence Gap Detection

The Strategy Layer MUST identify missing information before making or recommending a strategic decision.

An evidence gap exists when:

- a condition that must be true lacks evidence,
- the available evidence is too weak for the proposed decision,
- a claim depends on unstated assumptions,
- purchase behavior is inferred but not shown,
- customer need is assumed but not observed,
- willingness to pay is claimed without commitment or comparable purchase evidence,
- competitor response is ignored,
- capability feasibility is unknown,
- channel access is unproven,
- strategy depends on a market category that may not exist.

### EvidenceGapReport Schema

```yaml
evidence_gap_report:
  id:
  strategy_option_id:
  strategic_question:
  created_at:

  gaps:
    - gap_id:
      missing_information:
      affected_condition_id:
      why_it_matters:
      decision_blocked:
      current_evidence:
      current_evidence_level:
      required_evidence_level:
      recommended_research_type:
      recommended_research_prompt_id:
      urgency: high | medium | low

  decision_impact:
    allowed_decisions:
      - explore
      - collect_more_evidence
      - delegate_research
    blocked_decisions:
      - fabricate
      - commit
      - price
      - scale

  summary:
```

Rules:

- A strategy option with unresolved critical gaps MUST NOT move to `committed`.
- A strategy option MAY move to `fabrication_candidate` if the missing evidence is best obtained through prototype behavior.
- A strategy option SHOULD move to `delegate_research` when missing evidence can be gathered through desk research, UGC/social listening, purchase behavior research, competitor analysis, or category mapping.

---

## Research Prompt Generation

When critical information is missing, the Strategy Layer MAY generate prompts for dedicated research agents.

A research prompt is not the research result. It is a task packet.

The prompt MUST be specific enough for a research agent to produce a useful report and constrained enough to avoid confirmation bias.

### ResearchPromptPacket Schema

```yaml
research_prompt_packet:
  id:
  strategy_option_id:
  evidence_gap_report_id:
  title:
  research_type: analytical_research | ugc_social_listening | jtbd_interview_design | design_research | competitor_analysis | category_mapping | purchase_behavior_research | pricing_research | capability_research | regulatory_research

  strategic_context:
  condition_being_tested:
  missing_information:
  research_objective:

  research_questions:
    - question:

  source_priorities:
    - source_type:
      examples:
      reason:

  inclusion_criteria:
  exclusion_criteria:

  required_outputs:
    - executive_summary
    - methodology
    - findings
    - supporting_evidence
    - weakening_evidence
    - confidence_assessment
    - limitations
    - implications_for_strategy
    - recommended_next_tests
    - bibliography_or_source_list

  anti_bias_instructions:
    - "Do not search only for supporting evidence."
    - "Actively look for contradictory evidence."
    - "Separate facts from interpretation."
    - "Do not infer willingness to pay from interest alone."
    - "Distinguish direct, adjacent, and analogical evidence."
    - "Mark uncertainty clearly."

  final_report_format:
  decision_use:
```

### Research Prompt Rule

```text
If the Strategy Layer lacks critical evidence, it should generate a research prompt instead of inventing certainty.
```

---

## Dedicated Research Agent Prompt Template

A generated research prompt SHOULD use this structure:

```text
You are a neutral research analyst.

Research objective:
[State the specific missing information needed.]

Strategic context:
[Briefly explain the strategy option and why this research matters.]

Condition being tested:
[Condition that must be true.]

Research questions:
1. [Question]
2. [Question]
3. [Question]

Evidence to prioritize:
- Direct evidence where available.
- Real purchase behavior over stated preference.
- Primary sources over summaries.
- Recent sources where market conditions may have changed.
- Contradictory evidence, not only supporting evidence.

Required distinctions:
- Separate facts, interpretations, and assumptions.
- Distinguish direct, adjacent, and analogical evidence.
- Distinguish category-level willingness to pay from willingness to pay for this specific offer.
- Mark confidence and limitations.

Required output:
1. Executive summary.
2. Methodology.
3. Key findings.
4. Evidence supporting the condition.
5. Evidence weakening the condition.
6. Existing purchase behavior, if relevant.
7. Comparable solutions and price anchors, if relevant.
8. Open questions.
9. Strategic implications.
10. Recommended next test.
11. Bibliography/source list.

Do not try to prove the strategy correct.
Evaluate whether the evidence supports, weakens, or leaves unresolved the condition being tested.
```

---

## Research Report Ingestion

The Strategy Layer MAY ingest research reports generated by dedicated research agents.

Ingestion means converting the report into structured evidence.

The ingestion process SHOULD produce:

- `StrategyEvidence`,
- updated `ResearchSynthesis`,
- updated `EvidenceGapReport`,
- updated `StrategyEvalSpec`,
- updated `StrategyDecisionRecord`.

### ResearchReport Ingestion Rules

- Do not treat a report as truth.
- Extract evidence claims.
- Extract uncertainty.
- Extract contradictions.
- Extract purchase behavior evidence separately.
- Extract source quality.
- Map findings to conditions that must be true.
- Identify whether the report closes, weakens, or leaves open each evidence gap.
- Recommend the next decision or next research prompt.

### ResearchEvidenceExtraction Schema

```yaml
research_evidence_extraction:
  id:
  research_report_id:
  strategy_option_id:

  extracted_claims:
    - claim:
      source:
      evidence_type:
      supports_condition_id:
      weakens_condition_id:
      confidence:
      limitations:

  purchase_behavior_findings:
    - solution:
      evidence_type: direct | adjacent | analogical
      buyer:
      job_served:
      price_anchor:
      implication:

  gaps_closed:
    - gap_id:

  gaps_remaining:
    - gap_id:

  new_gaps_identified:
    - missing_information:

  recommended_next_action:
```

---

## Research-Centered Strategy Tests

A `StrategyTest` may be any disciplined evidence-gathering method that validates or weakens a condition that must be true.

Supported v0.1 test types:

```yaml
test_types:
  analytical_research:
    purpose: "Frame assumptions, compare explanations, map categories, and identify plausible strategic options."

  ugc_social_listening:
    purpose: "Mine public user-generated content for repeated pains, workarounds, language, objections, and category signals."

  jtbd_interview:
    purpose: "Understand real situations, switching behavior, current workarounds, constraints, and desired outcomes."

  design_research:
    purpose: "Map workflows, user journeys, service blueprints, friction points, and environmental constraints."

  competitor_analysis:
    purpose: "Identify alternatives, substitutes, positioning, pricing, capabilities, and likely responses."

  category_mapping:
    purpose: "Map how the market names, buys, and compares solutions."

  purchase_behavior_research:
    purpose: "Study real purchases of comparable or adjacent solutions to validate category-level willingness to pay and price anchors."

  prototype_pilot:
    purpose: "Test whether a proposed solution actually helps users perform the job."

  sales_commitment:
    purpose: "Test whether a target buyer will commit money, time, access, or authority."

  cost_model:
    purpose: "Test whether unit economics, delivery cost, or operating assumptions can work."

  capability_audit:
    purpose: "Test whether required capabilities exist or can be obtained."
```

---

## Evidence Ladder

The Strategy Layer uses an evidence ladder to prevent overclaiming.

```text
Level 1: Analytical inference
Level 2: UGC / social listening signal
Level 3: Existing purchase behavior in comparable or adjacent solutions
Level 4: Direct customer conversation / JTBD interview
Level 5: Prototype or workflow behavior
Level 6: Commitment signal to our offer
Level 7: Paid usage of our offer
Level 8: Repeated paid usage / retention
```

Rules:

- Lower levels may justify further research or testing.
- Higher levels are required for commitment.
- Existing purchase behavior may validate category-level willingness to pay.
- Existing purchase behavior does not automatically validate willingness to pay for our specific offer.
- Commitment or paid usage is required for offer-level willingness-to-pay validation.

---

## Purchase Behavior Research

Existing purchases are first-class strategic evidence.

Observed real purchases validate that people are willing to pay for some version of a job-to-be-done solution, at some price, under some conditions.

They do not automatically validate our specific offer, positioning, channel, pricing, or ability to win.

### Purchase Evidence Types

```yaml
purchase_evidence:
  type: direct | adjacent | analogical

  direct:
    meaning: "Users already buy the same class of solution for the same job."

  adjacent:
    meaning: "Users buy related solutions around the same workflow or pain."

  analogical:
    meaning: "Users buy solutions with a similar structure but different domain."
```

### Purchase Evidence Rule

```text
Existing purchases validate category-level willingness to pay when the purchased solution serves the same or adjacent JTBD.

They do not validate offer-level willingness to pay unless the evidence involves our offer, a close substitute, or a credible commitment from the target buyer.
```

---

## StrategyDecisionRecord

A `StrategyDecisionRecord` records the justified strategic decision.

```yaml
strategy_decision_record:
  id:
  strategy_option_id:
  decision_status: commit | test | defer | reject | fabricate | collect_more_evidence | delegate_research

  strategic_question:
  decision_summary:

  highest_evidence_level_reached:

  evidence_sufficient_for:
    - explore
    - delegate_research
    - interview
    - fabricate
    - pilot
    - commit
    - reject

  evidence_not_sufficient_for:
    - commit
    - pricing
    - retention
    - scale
    - channel_fit

  evidence_gaps:
    - gap_id:

  research_prompts_generated:
    - research_prompt_packet_id:

  research_reports_ingested:
    - research_report_id:

  supporting_evidence:
    - evidence_id:

  weakening_evidence:
    - evidence_id:

  unresolved_assumptions:
    - assumption:

  tradeoffs_accepted:
    - tradeoff:

  next_action:
  owner:
  review_date:
```

Decision rules:

- `commit` requires high-confidence evidence and explicit trade-offs.
- `fabricate` requires enough evidence to justify a prototype, not enough evidence to justify full commitment.
- `delegate_research` means the next best action is to generate or run a research prompt.
- `collect_more_evidence` means the option is plausible but under-supported.
- `test` means the next test is known.
- `defer` means insufficient timing, capability, or evidence.
- `reject` means the option failed a critical condition or is strategically incoherent.

---

## StrategyEvalSpec

A `StrategyEvalSpec` defines how to evaluate a strategy option.

It SHOULD check:

- customer need strength,
- segment attractiveness,
- where-to-play specificity,
- how-to-win clarity,
- trade-off clarity,
- purchase behavior evidence,
- willingness-to-pay evidence,
- evidence gaps,
- research sufficiency,
- capability fit,
- channel access,
- competitive response risk,
- execution feasibility,
- management system readiness,
- strategic coherence,
- decision readiness.

Example:

```yaml
strategy_eval_spec:
  id:
  strategy_option_id:

  checks:
    - id: evidence.gaps_identified
      question: "Have critical evidence gaps been identified before decision?"
      pass_criteria:
      evidence_required:

    - id: research.prompts_generated
      question: "Were research prompts generated for unresolved critical gaps?"
      pass_criteria:
      evidence_required:

    - id: customer.need_strength
      question: "Is the need painful, frequent, and already producing workarounds?"
      pass_criteria:
      evidence_required:

    - id: purchase.category_spend
      question: "Does real purchase behavior show money already moves around this job or adjacent solutions?"
      pass_criteria:
      evidence_required:

    - id: decision.evidence_sufficiency
      question: "Is the evidence strong enough for the proposed decision status?"
      pass_criteria:
      evidence_required:
```

Rule:

No StrategyOption may move to `committed` without a StrategyDecisionRecord and supporting StrategyEvidence.

No StrategyOption may move to `committed` while critical evidence gaps remain unresolved.

---

## Agency and Anti-Persuasion Safeguards

The Strategy Layer must support judgment, not replace it.

Required safeguards:

- Separate claim, assumption, evidence, and recommendation.
- Present competing options using comparable structure.
- Do not use urgency language unless time sensitivity is evidenced.
- Do not hide uncertainty.
- Do not over-rank weak evidence.
- Do not convert founder enthusiasm into market proof.
- Do not treat a coherent narrative as validation.
- Do not treat research volume as evidence quality.
- Do not treat existing market size as proof that we can win.
- Preserve the option to reject all generated strategies.
- Ask what outcome the operator is optimizing for before recommending.
- Mark speculative recommendations as speculative.
- Generate research prompts when evidence is missing instead of inventing certainty.

### Claim/Evidence Separation

Every strategic recommendation SHOULD be decomposed as:

```yaml
recommendation:
  claim:
  assumptions:
  evidence_supporting:
  evidence_weakening:
  evidence_missing:
  research_prompt_needed:
  confidence:
  test_required:
  decision_allowed:
  decision_not_allowed:
```

---

## Standalone Operating Mode

The Strategy Layer MUST be usable without the fabrication or harness system.

Minimum standalone workflow:

```text
1. Capture NeedSignal.
2. Create StrategicQuestion.
3. Generate StrategyOptions.
4. Define ConditionsToWin.
5. Identify EvidenceGaps.
6. Generate ResearchPromptPackets.
7. Run or delegate research.
8. Ingest ResearchReports.
9. Record StrategyEvidence.
10. Evaluate options.
11. Create StrategyDecisionRecord.
12. Add next experiments to backlog.
```

The standalone layer MUST NOT require:

- hosted tools,
- vector memory,
- databases,
- external APIs,
- multi-agent orchestration,
- prototype generation.

---

## Optional Harness / Fabrication Integration

The Strategy Layer MAY later hand off approved options to a fabrication or harness system.

Allowed handoff outputs:

- `ProductSeed`
- `FabricationCandidate`
- `MarketExperimentTask`
- `CustomerDiscoveryTask`
- `MessagingBrief`
- `ConsultingOfferBrief`
- `ResearchTask`
- `PrototypePilotTask`

Integration is optional in v0.1.

When strategy outputs are carried into other layers, the receiving layer MAY preserve optional
upstream provenance such as:

- `strategy_option_id`
- `strategy_decision_record_id`
- `strategy_evidence_ids`
- `strategy_evidence_gap_ids`
- `strategy_question`

These fields are advisory provenance only. Their presence MUST improve traceability, but their
absence MUST NOT invalidate standalone strategy operation.

A strategy handoff is valid only if it includes:

- strategy option ID,
- decision record,
- evidence summary,
- evidence gaps,
- research reports used,
- constraints,
- non-goals,
- assumptions,
- required proof,
- next test.

---

## BAML Support

If the repository uses BAML or typed generation, the Strategy Layer MAY define BAML functions for strategy artifacts.

Suggested functions:

```text
GenerateStrategicQuestion
GenerateNeedSignal
GenerateWinningAspiration
GenerateWhereToPlayOptions
GenerateHowToWinOptions
GenerateCapabilityMap
GenerateManagementSystemMap
GenerateConditionsToWin
IdentifyEvidenceGaps
GenerateResearchPromptPacket
IngestResearchReport
GenerateResearchSynthesis
GenerateStrategyTests
GenerateUGCSignals
GeneratePurchaseBehaviorResearch
GenerateStrategyOption
GenerateStrategyEvalSpec
GenerateStrategyDecisionRecord
GenerateProductSeedFromStrategyDecision
```

Rules:

- BAML may draft strategy artifacts.
- BAML may generate research prompts.
- BAML may ingest research reports into structured evidence.
- BAML must not commit strategy.
- BAML must preserve uncertainty.
- BAML must separate claims, assumptions, evidence, and missing evidence.
- BAML must not invent market proof.
- BAML must not promote a strategy option without a StrategyDecisionRecord.
- BAML outputs remain drafts until validated.

---

## Validation Requirements

The Strategy Layer SHOULD validate:

- every StrategyOption has a StrategicQuestion,
- every StrategyOption answers the five choice cascade questions,
- every StrategyOption includes conditions that must be true,
- every StrategyOption identifies evidence gaps,
- every critical evidence gap has a research prompt or explicit waiver,
- every StrategyTest maps to a condition,
- every StrategyEvidence maps to a test, condition, or research report,
- every ResearchReport maps to a ResearchPromptPacket,
- every StrategyDecisionRecord maps to evidence,
- every commit/fabricate decision states evidence sufficiency,
- every rejected option states why,
- every deferred option states what is missing,
- every purchase behavior claim distinguishes direct, adjacent, and analogical evidence,
- every recommendation separates claim, assumption, evidence, missing evidence, and confidence.

Validation SHOULD check strategic completeness, not formatting perfection.

Self-sufficiency lessons learned for v0.1:

- the layer should validate its own config and schemas with its own loader,
- generated artifacts should remain inside `.strategy/` during normal operation,
- example artifacts should be sufficient for another agent to understand the expected output shapes,
- external wrappers should be treated as optional convenience, not hidden requirements.

---

## Non-Goals

The Strategy Layer MUST NOT become:

- automatic strategy commitment,
- automatic product building,
- fake market validation,
- a replacement for operator judgment,
- large enterprise planning bureaucracy,
- strategy theater,
- a dashboard-first system,
- a hosted-tool dependency,
- a vector-memory dependency,
- a persuasive narrative generator,
- a way to avoid talking to customers,
- a way to treat research as proof beyond its evidence level.

A strategy option must produce a decision, a test, a research prompt, a rejection, or a justified deferment.

---

## Minimal v0.1 Implementation

v0.1 SHOULD include only:

- Strategy Layer spec.
- Self-contained strategy runtime under `.strategy/strategy.py` or equivalent.
- `.strategy/` folder structure.
- StrategyOption schema/template.
- EvidenceGapReport template.
- ResearchPromptPacket template.
- ResearchReport template.
- StrategyTest schema/template.
- StrategyEvalSpec schema/template.
- StrategyDecisionRecord template.
- UGCSignal schema/template.
- ResearchSynthesis template.
- PurchaseBehaviorResearch template.
- StrategyExperimentBacklog template.
- One example strategy option.
- One example evidence gap report.
- One example research prompt packet.
- One example strategy eval.
- One example reverse-engineering worksheet.
- Optional BAML function outline if BAML exists.

v0.1 SHOULD NOT include:

- databases,
- dashboards,
- autonomous strategy ranking,
- autonomous product commitment,
- vector memory,
- hosted research automation,
- multi-agent orchestration,
- automatic fabrication handoff.

---

## Example Evidence Gap

```yaml
evidence_gap_report:
  id: gap.agentic_dev_harness.purchase_behavior
  strategy_option_id: option.agentic_dev_harness
  strategic_question: "Should we pursue a first wedge around proof-backed agentic development harnesses?"

  gaps:
    - gap_id: gap.purchase_behavior.001
      missing_information: "Whether target users already spend money on comparable or adjacent solutions around AI-assisted development reliability."
      affected_condition_id: condition.agentic_dev_harness.purchase_behavior
      why_it_matters: "Without purchase behavior evidence, we cannot distinguish real commercial demand from intellectual interest."
      decision_blocked: "commit"
      current_evidence: "General knowledge that AI coding tools and dev productivity tools exist."
      current_evidence_level: 1
      required_evidence_level: 3
      recommended_research_type: purchase_behavior_research
      recommended_research_prompt_id: prompt.agentic_dev_purchase_behavior
      urgency: high

  decision_impact:
    allowed_decisions:
      - explore
      - delegate_research
    blocked_decisions:
      - commit
      - price
      - scale

  summary: >
    Purchase behavior must be researched before this option can be treated as commercially validated.
```

---

## Example Research Prompt Packet

```text
You are a neutral research analyst.

Research objective:
Determine whether target users already spend money on comparable or adjacent solutions related to AI-assisted software development reliability, agent drift reduction, code verification, developer productivity, or proof-backed implementation workflows.

Strategic context:
We are evaluating a strategy option around a proof-backed agentic development harness for users of Claude Code, OpenClaw-style agents, and similar AI coding tools.

Condition being tested:
Target users already spend money to improve AI-assisted software development reliability or adjacent workflows.

Research questions:
1. What paid tools or services do target users currently buy to improve AI-assisted development, developer productivity, software delivery reliability, code quality, or workflow governance?
2. Which solutions are direct, adjacent, or analogical comparables?
3. What jobs do those solutions serve?
4. What price anchors are visible?
5. What evidence suggests users want proof, evals, traceability, or reduced agent drift?
6. What evidence weakens the case that users would pay for a separate harness?
7. Would a proof-backed harness replace existing spend, augment existing spend, or require a new budget category?

Evidence to prioritize:
- Real purchases and paid products over stated interest.
- Pricing pages, public subscriptions, paid communities, marketplaces, consulting offers, and user reports of paid tools.
- Direct or adjacent developer workflows.
- Contradictory evidence showing users prefer free/manual workarounds.

Required distinctions:
- Separate facts, interpretations, and assumptions.
- Distinguish direct, adjacent, and analogical purchase evidence.
- Distinguish category-level willingness to pay from willingness to pay for this specific offer.
- Mark confidence and limitations.

Required output:
1. Executive summary.
2. Methodology.
3. Comparable paid solutions.
4. Price anchors.
5. Purchase evidence by direct/adjacent/analogical type.
6. Evidence supporting the condition.
7. Evidence weakening the condition.
8. Strategic implications.
9. Recommended next test.
10. Source list.

Do not try to prove the strategy correct.
Evaluate whether the evidence supports, weakens, or leaves unresolved the condition being tested.
```

---

## Advisor Constraint

No new Strategy Layer abstraction may be added unless it improves at least one of:

- decision quality,
- evidence clarity,
- research delegation,
- missing information detection,
- testability,
- trade-off clarity,
- strategy rejection speed,
- market learning,
- purchase behavior understanding,
- fabrication selection,
- commitment discipline.

If an abstraction does not improve one of those outcomes, it belongs in a parking-lot note, not the active spec.

---

## Acceptance Criteria

The Strategy Layer is acceptable when:

- it works standalone,
- it is regenerable from the contents of `.strategy/` itself,
- it defines strategy as integrated choices,
- it includes the Strategic Choice Cascade,
- it includes reverse engineering and conditions that must be true,
- it identifies missing information before overclaiming,
- it generates research prompts for dedicated research agents,
- it ingests research reports into structured evidence,
- it supports research-centered strategy tests,
- it treats existing purchase behavior as first-class evidence,
- it distinguishes direct, adjacent, and analogical purchase evidence,
- it prevents overclaiming from weak evidence,
- it separates claims, assumptions, evidence, missing evidence, tests, and decisions,
- it includes anti-persuasion and agency safeguards,
- it can optionally hand off to fabrication later,
- v0.1 remains lightweight and implementable,
- it avoids strategy theater.
