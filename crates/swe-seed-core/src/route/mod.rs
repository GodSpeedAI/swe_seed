//! Routing (specs 0004/0012). Loads the frozen route cards, scores a task
//! against each via deterministic token overlap, and selects a card. Route-
//! decision records are written byte-compatible with the captured Python output.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::contracts::parity::{BamlParity, BamlShape};
use crate::util::{redact_secrets, slugify, utc_now, utc_stamp};

/// The 11 required job types (AGENTS.md). Each must resolve to a route card.
pub const REQUIRED_JOB_TYPES: &[&str] = &[
    "research",
    "spec",
    "implementation",
    "bugfix",
    "refactor",
    "test",
    "review",
    "release",
    "documentation",
    "harness_improvement",
    "skill_authoring",
];

/// Element of `RouteCard.validation` (baml `ValidationRequirement`).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RouteValidationEntry {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default)]
    pub evidence: String,
}

/// Element of `RouteCard.sources` (baml `SourceRef`).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RouteSourceRef {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub summary: String,
}

/// A route card. Mirrors `class RouteCard` in `harness.baml` (16 fields). The
/// runtime JSON cards omit `validation`/`sources`, so they default to empty.
#[derive(Debug, Clone, Deserialize)]
pub struct RouteCard {
    pub id: String,
    pub job_type: String,
    pub purpose: String,
    pub semantic_triggers: Vec<String>,
    pub positive_examples: Vec<String>,
    pub negative_examples: Vec<String>,
    pub required_context: Vec<String>,
    pub required_skills: Vec<String>,
    pub work_loop: Vec<String>,
    pub required_artifacts: Vec<String>,
    pub proof: Vec<String>,
    pub done_when: Vec<String>,
    pub failure_modes: Vec<String>,
    pub fallback_policy: String,
    #[serde(default)]
    pub validation: Vec<RouteValidationEntry>,
    #[serde(default)]
    pub sources: Vec<RouteSourceRef>,
}

impl BamlParity for RouteCard {
    fn baml_name() -> &'static str {
        "RouteCard"
    }
    fn baml_shape() -> BamlShape {
        BamlShape::Class {
            fields: vec![
                "id",
                "job_type",
                "purpose",
                "semantic_triggers",
                "positive_examples",
                "negative_examples",
                "required_context",
                "required_skills",
                "work_loop",
                "required_artifacts",
                "proof",
                "done_when",
                "failure_modes",
                "fallback_policy",
                "validation",
                "sources",
            ],
            field_types: vec![
                "string",
                "string",
                "string",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string[]",
                "string",
                "ValidationRequirement[]",
                "SourceRef[]",
            ],
        }
    }
}

/// The derived routing decision (the `route` object in records). Field order
/// matches the captured Python output for format parity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub job_type: String,
    pub route_card: String,
    pub confidence: String,
    pub assumption: Option<String>,
    pub required_context: Vec<String>,
    pub required_skills: Vec<String>,
    pub work_loop: Vec<String>,
    pub required_artifacts: Vec<String>,
    pub proof: Vec<String>,
    pub done_when: Vec<String>,
    pub next_action: String,
}

pub fn harness_dir(root: &Path) -> PathBuf {
    root.join(".agent-harness")
}
pub fn routes_dir(root: &Path) -> PathBuf {
    harness_dir(root).join("routes")
}
pub fn route_decisions_dir(root: &Path) -> PathBuf {
    harness_dir(root).join("traces").join("route-decisions")
}

/// Load all `*.json` route cards, sorted by filename (matches Python route_paths).
pub fn load_routes(root: &Path) -> Result<Vec<RouteCard>> {
    let dir = routes_dir(root);
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let p = entry?.path();
        if p.extension().is_some_and(|x| x == "json") {
            files.push(p);
        }
    }
    files.sort();
    let mut cards = Vec::with_capacity(files.len());
    for f in files {
        let bytes = std::fs::read(&f).with_context(|| format!("read {}", f.display()))?;
        let card: RouteCard =
            serde_json::from_slice(&bytes).with_context(|| format!("parse {}", f.display()))?;
        cards.push(card);
    }
    Ok(cards)
}

/// `{token for token in [a-z0-9_]+ if len > 1}` over the lowercased text.
fn tokenize(text: &str) -> std::collections::BTreeSet<String> {
    let re = Regex::new(r"[a-z0-9_]+").expect("static regex");
    re.find_iter(&text.to_lowercase())
        .map(|m| m.as_str().to_string())
        .filter(|t| t.len() > 1)
        .collect()
}

/// Deterministic token-overlap score (port of harness.score_route).
pub fn score_route(task: &str, card: &RouteCard) -> i64 {
    let task_text = task.to_lowercase();
    let task_tokens = tokenize(task);
    let mut score: i64 = 0;

    for phrase in &card.semantic_triggers {
        if task_text.contains(&phrase.to_lowercase()) {
            score += 8;
        }
        let overlap = task_tokens.intersection(&tokenize(phrase)).count() as i64;
        score += overlap * 2;
    }

    for example in &card.positive_examples {
        let overlap = task_tokens.intersection(&tokenize(example)).count() as i64;
        if task_text.contains(&example.to_lowercase()) {
            score += 3;
        } else if overlap >= 2 {
            score += overlap;
        }
    }

    for example in &card.negative_examples {
        let overlap = task_tokens.intersection(&tokenize(example)).count() as i64;
        if task_text.contains(&example.to_lowercase()) {
            score -= 4;
        } else if overlap >= 2 {
            score -= overlap * 2;
        }
    }

    if task_text.contains(&card.job_type) {
        score += 5;
    }

    score
}

/// Bootstrap heuristic (port of harness.infer_bootstrap_route). Returns the
/// `spec` card when the task is a build verb with no clarifying/direct-change
/// terms and no file extension.
fn infer_bootstrap_route<'a>(task: &str, cards: &'a [RouteCard]) -> Option<&'a RouteCard> {
    let task_tokens = tokenize(task);
    let task_text = task.to_lowercase();
    let build_verbs: &[&str] = &["build", "create", "make", "add", "start", "scaffold"];
    let clarification_nouns: &[&str] = &[
        "spec",
        "requirements",
        "contract",
        "acceptance",
        "criteria",
        "plan",
    ];
    let direct_change_terms: &[&str] = &[
        "fix", "debug", "review", "audit", "refactor", "release", "document", "docs", "test",
        "tests", "harness", "router", "skill",
    ];
    let has = |set: &[&str]| set.iter().any(|t| task_tokens.contains(*t));
    if !has(build_verbs) || has(clarification_nouns) || has(direct_change_terms) {
        return None;
    }
    if Regex::new(r"\.(py|ts|tsx|js|jsx|md|json|yaml|yml|sh)\b")
        .unwrap()
        .is_match(&task_text)
    {
        return None;
    }
    cards.iter().find(|c| c.job_type == "spec")
}

/// Build the routing decision for a task (port of harness.build_route_result).
pub fn build_route_result(root: &Path, task: &str) -> Result<RouteResult> {
    let cards = load_routes(root)?;
    if cards.is_empty() {
        anyhow::bail!("no route cards found");
    }
    // Sort by (score desc, id desc) — matches Python sorted(..., reverse=True).
    let mut ranked: Vec<&RouteCard> = cards.iter().collect();
    ranked.sort_by(|a, b| {
        (score_route(task, b), b.id.as_str()).cmp(&(score_route(task, a), a.id.as_str()))
    });
    let mut selected = ranked[0];
    let mut selected_score = score_route(task, selected);
    if selected_score <= 0 {
        if let Some(boot) = infer_bootstrap_route(task, &cards) {
            selected = boot;
            selected_score = 1;
        }
    }
    let confidence = if selected_score > 0 { "medium" } else { "low" };
    let assumption = if selected_score > 0 {
        None
    } else {
        Some("No strong semantic match; selected safest default route.".to_string())
    };
    let next_first = selected.work_loop.first().map(|s| s.as_str()).unwrap_or("");
    Ok(RouteResult {
        job_type: selected.job_type.clone(),
        route_card: rel(
            root,
            &routes_dir(root).join(format!("{}.json", selected.id)),
        ),
        confidence: confidence.to_string(),
        assumption,
        required_context: selected.required_context.clone(),
        required_skills: selected.required_skills.clone(),
        work_loop: selected.work_loop.clone(),
        required_artifacts: selected.required_artifacts.clone(),
        proof: selected.proof.clone(),
        done_when: selected.done_when.clone(),
        next_action: format!("Read required context, then execute work_loop[0]: {next_first}"),
    })
}

/// Repo-relative path string (port of harness.rel).
fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

/// Write a route-decision record; returns its repo-relative path string.
pub fn write_route_decision(root: &Path, task: &str, result: &RouteResult) -> Result<String> {
    let dir = route_decisions_dir(root);
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let trace_id = format!("{}-{}", utc_stamp(), slugify(&redact_secrets(task)));
    let decision = serde_json::json!({
        "trace_id": trace_id,
        "created_at": utc_now(),
        "task": redact_secrets(task),
        "route": result,
        "decision_basis": "deterministic token overlap against route-card triggers, examples, and job type",
    });
    let path = dir.join(format!("{trace_id}.json"));
    std::fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&decision)?),
    )
    .with_context(|| format!("write {}", path.display()))?;
    Ok(rel(root, &path))
}
