//! Harness structure validation — Rust port of the load-bearing subset of the
//! former `scripts/harness.py validate` (spec: CI integrity). Checks that the
//! harness tree is structurally sound: root specs/baml present, required dirs
//! present, route cards resolve for every job type with resolvable skills,
//! skills are well-formed with no duplicate ids and all required active skills
//! present, render-targets carry the canonical-source notice and have no
//! duplicate content, and source specs/scripts carry no incomplete-work
//! markers.
//!
//! The doctrine phrase/word-count checks (behavior-shaping phrases in AGENTS.md,
//! playbook min-word counts, 9arm process phrases, etc.) are intentionally NOT
//! ported here — they are content-quality nudges, not structural integrity, and
//! are tracked for a follow-up.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::Value;

/// Root layer spec files that MUST exist.
const ROOT_SPECS: &[&str] = &[
    "SWE_SEED_SPEC_v0.2.0.md",
    "HARNESS_SPEC.md",
    "FABRICATOR_SPEC_v0.1.0.md",
];

/// `.baml` source contracts that MUST exist.
const BAML_CONTRACTS: &[&str] = &[
    ".agent-harness/baml/baml_src/swe_seed.baml",
    ".agent-harness/baml/baml_src/harness.baml",
    ".agent-harness/baml/baml_src/fabricator.baml",
];

/// Required harness directories.
const REQUIRED_DIRS: &[&str] = &[
    "skills",
    "render-targets",
    "hooks",
    "memory",
    "context",
    "traces",
    "evals",
    "reflections",
    "playbooks",
    "imports",
    "routes",
];

/// The 11 required job types (AGENTS.md); each MUST have a route card.
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

/// Required active skill ids.
const REQUIRED_ACTIVE_SKILLS: &[&str] = &[
    "plan-and-frame",
    "implement-with-proof",
    "test-with-proof",
    "debug-discipline",
    "review-for-risk",
    "verify-before-completion",
    "capture-learning",
];

/// Required fields on every skill JSON.
const REQUIRED_SKILL_FIELDS: &[&str] = &[
    "id",
    "version",
    "category",
    "jtbd",
    "description",
    "triggers",
    "procedure",
    "evidence_required",
    "forbidden_behaviors",
    "outputs",
    "success_criteria",
];

/// Valid skill status values.
const VALID_SKILL_STATUSES: &[&str] = &[
    "draft",
    "active",
    "deprecated",
    "experimental",
    "imported",
    "candidate",
];

/// Removed proof command surfaces that route cards must not depend on.
const REMOVED_ROUTE_PROOF_COMMANDS: &[&str] =
    &["python scripts/harness.py", "bash tests/validate-harness.sh"];

/// Incomplete-work markers that must not appear in source specs/scripts.
const INCOMPLETE_MARKERS: &[&str] = &[
    "TODO",
    "FIXME",
    "TBD",
    "XXX",
    "coming soon",
    "stub implementation",
    "mock implementation",
    "placeholder for",
];

/// Roots scanned for incomplete-work markers.
const INCOMPLETE_SCAN_ROOTS: &[&str] = &[
    "AGENTS.md",
    "SWE_SEED_SPEC_v0.2.0.md",
    "HARNESS_SPEC.md",
    "FABRICATOR_SPEC_v0.1.0.md",
    ".agent-harness",
    "tests",
    "docs/specs",
];

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn harness_root(root: &Path) -> PathBuf {
    root.join(".agent-harness")
}

/// Run the harness structure validation. Returns the list of errors (empty = pass).
pub fn validate(root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    for spec in ROOT_SPECS {
        if !root.join(spec).is_file() {
            errors.push(format!("missing root spec: {spec}"));
        }
    }
    for c in BAML_CONTRACTS {
        if !root.join(c).is_file() {
            errors.push(format!("missing BAML source contract: {c}"));
        }
    }
    let h = harness_root(root);
    for d in REQUIRED_DIRS {
        if !h.join(d).is_dir() {
            errors.push(format!("missing required directory: .agent-harness/{d}"));
        }
    }

    errors.extend(validate_no_incomplete_markers(root));

    // ---- skills ----
    let mut seen_skills: HashSet<String> = HashSet::new();
    for path in skill_paths(root) {
        let r = rel(root, &path);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => {
                errors.push(format!("{r}: cannot read"));
                continue;
            }
        };
        let skill: Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{r} is invalid JSON: {e}"));
                continue;
            }
        };
        let missing: Vec<&&str> = REQUIRED_SKILL_FIELDS
            .iter()
            .filter(|f| !skill.get(**f).is_some_and(|v| !v.is_null()))
            .collect();
        if !missing.is_empty() {
            let names: Vec<&str> = missing.iter().map(|m| **m).collect();
            errors.push(format!("{r} missing fields: {}", names.join(", ")));
        }
        let desc_empty = skill
            .get("description")
            .and_then(Value::as_str)
            .map(|s| s.trim().is_empty())
            .unwrap_or(true);
        if desc_empty {
            errors.push(format!("{r} description must be a non-empty string"));
        }
        let status_ok = skill
            .get("status")
            .and_then(Value::as_str)
            .map(|s| VALID_SKILL_STATUSES.contains(&s))
            .unwrap_or(false);
        if !status_ok {
            errors.push(format!(
                "{r} has invalid status: {}",
                skill
                    .get("status")
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            ));
        }
        if let Some(id) = skill.get("id").and_then(Value::as_str) {
            if id != id.to_lowercase() || id.contains('_') || id.contains(' ') {
                errors.push(format!("{r} id must be lowercase kebab-case"));
            }
            if !seen_skills.insert(id.to_string()) {
                errors.push(format!("duplicate skill id: {id}"));
            }
        }
    }
    for required in REQUIRED_ACTIVE_SKILLS {
        if !seen_skills.contains(*required) {
            errors.push(format!("missing required active skill: {required}"));
        }
    }

    // ---- routes ----
    let mut seen_routes: HashSet<String> = HashSet::new();
    let mut seen_job_types: HashSet<String> = HashSet::new();
    for path in route_paths(root) {
        let r = rel(root, &path);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => {
                errors.push(format!("{r}: cannot read"));
                continue;
            }
        };
        let route: Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{r} is invalid JSON: {e}"));
                continue;
            }
        };
        if let Some(id) = route.get("id").and_then(Value::as_str) {
            if !seen_routes.insert(id.to_string()) {
                errors.push(format!("duplicate route id: {id}"));
            }
            if id != path.file_stem().and_then(|s| s.to_str()).unwrap_or("") {
                errors.push(format!("{r} id must match filename"));
            }
        }
        let job_type = route.get("job_type").and_then(Value::as_str).unwrap_or("");
        if !REQUIRED_JOB_TYPES.contains(&job_type) {
            errors.push(format!(
                "{r} job_type is not a required job type: {job_type}"
            ));
        } else {
            seen_job_types.insert(job_type.to_string());
        }
        // required_skills must resolve to a known skill id.
        if let Some(reqs) = route.get("required_skills").and_then(Value::as_array) {
            for s in reqs {
                if let Some(sid) = s.as_str() {
                    if !seen_skills.contains(sid) {
                        errors.push(format!("{r} required_skill not found: {sid}"));
                    }
                }
            }
        }
        if let Some(proof) = route.get("proof").and_then(Value::as_array) {
            for command in proof.iter().filter_map(Value::as_str) {
                for removed in REMOVED_ROUTE_PROOF_COMMANDS {
                    if command.contains(removed) {
                        errors.push(format!(
                            "{r} proof references removed command surface: {command}"
                        ));
                    }
                }
            }
        }
    }
    for jt in REQUIRED_JOB_TYPES {
        if !seen_job_types.contains(*jt) {
            errors.push(format!("missing route card for job type: {jt}"));
        }
    }

    // ---- render targets: canonical-source notice + duplicate content ----
    errors.extend(validate_render_targets(root));

    errors
}

fn validate_no_incomplete_markers(root: &Path) -> Vec<String> {
    let policy_phrases = [
        "strictly forbidden",
        "prohibited",
        "must reject",
        "must not rely",
        "cannot rely",
        "forbidden release evidence",
        "incomplete_work_markers",
    ];
    let word_markers = ["TODO", "FIXME", "TBD", "XXX"];
    let word_res: Vec<(&str, Regex)> = word_markers
        .iter()
        .map(|m| {
            (
                *m,
                Regex::new(&format!(r"\b{}\b", regex::escape(m))).unwrap(),
            )
        })
        .collect();

    let mut errors = Vec::new();
    let mut scan: Vec<PathBuf> = Vec::new();
    for rel_root in INCOMPLETE_SCAN_ROOTS {
        let p = root.join(rel_root);
        if p.is_file() {
            scan.push(p);
        } else if p.is_dir() {
            collect_scan_files(&p, &mut scan);
        }
    }
    for path in scan {
        let rel_path = rel(root, &path);
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        // filename marker check
        if name.contains("TODO") || name.contains("FIXME") || name.contains("TBD") {
            errors.push(format!("incomplete-work marker in filename: {rel_path}"));
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        for (line_no, raw) in text.lines().enumerate() {
            let stripped = raw.trim();
            if stripped.is_empty() {
                continue;
            }
            let lower = stripped.to_lowercase();
            let policy_line = policy_phrases.iter().any(|p| lower.contains(p));
            for marker in INCOMPLETE_MARKERS {
                let found = if matches!(*marker, "TODO" | "FIXME" | "TBD" | "XXX") {
                    word_res
                        .iter()
                        .find(|(m, _)| m == marker)
                        .map_or(false, |(_, re)| re.is_match(stripped))
                } else {
                    lower.contains(&marker.to_lowercase())
                };
                if found && !policy_line {
                    errors.push(format!(
                        "incomplete-work marker {marker:?} in {rel_path}:{}",
                        line_no + 1
                    ));
                }
            }
        }
    }
    errors
}

fn collect_scan_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_symlink() {
            continue;
        } else if p.is_dir() {
            // skip baml_client + noise/cache dirs
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(
                name,
                "baml_client" | "__pycache__" | ".git" | "target" | "node_modules"
            ) {
                continue;
            }
            collect_scan_files(&p, out);
        } else {
            let ext = p
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if matches!(
                ext.as_str(),
                "md" | "yaml" | "yml" | "json" | "py" | "sh" | "baml" | "toml"
            ) {
                out.push(p);
            }
        }
    }
}

fn validate_render_targets(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let rt = harness_root(root).join("render-targets");
    let mut files: Vec<PathBuf> = Vec::new();
    collect_all(&rt, &mut files);

    // duplicate content among non-symlinks
    let mut seen: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    for path in &files {
        if path.is_symlink() {
            continue;
        }
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if let Some(prev) = seen.get(&text) {
            errors.push(format!(
                "duplicate render target content must use symlink: {} and {}",
                rel(root, prev),
                rel(root, path)
            ));
        } else {
            seen.insert(text, path.clone());
        }
    }

    // canonical-source notice on generated .md
    for path in &files {
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if text.contains("Generated from Skill IR:")
            && !text.contains("Update the Skill IR source instead.")
        {
            errors.push(format!(
                "generated render target missing canonical-source notice: {}",
                rel(root, path)
            ));
        }
    }

    errors
}

fn collect_all(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_symlink() {
            continue;
        } else if p.is_dir() {
            collect_all(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn skill_paths(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_json(&harness_root(root).join("skills"), &mut out);
    out.sort();
    out
}

fn route_paths(root: &Path) -> Vec<PathBuf> {
    let dir = harness_root(root).join("routes");
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn collect_json(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_json(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(p);
        }
    }
}
