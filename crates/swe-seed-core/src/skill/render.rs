//! Render `SkillIR` to host files (spec 0007). Deterministic templates per
//! `render_target` kind. A blocking scan prevents render.

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::ingest::{discover, ingest_one};
use super::ir::SkillIR;
use crate::security::{exceptions::Exceptions, gate::scan_blocks_projection};

/// One rendered host file (repo-relative path + content).
#[derive(Debug, Clone)]
pub struct RenderTarget {
    pub kind: String,
    pub rel_path: PathBuf,
    pub content: String,
}

/// Reduce an ingested identifier to a single safe path component: drop any path
/// separators / traversal / absolute prefixes so a malicious `skill.id` or
/// `category` cannot escape the render-targets hierarchy. Falls back to a safe
/// placeholder when nothing safe remains.
fn safe_component(raw: &str) -> String {
    // Take the final segment after any separator (covers absolute + nested).
    let last = raw.rsplit(['/', '\\']).next().unwrap_or("");
    let cleaned: String = last
        .chars()
        .filter(|c| *c != '.' && !c.is_control())
        .collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "unsafe-skill".to_string()
    } else {
        cleaned
    }
}

/// A single-line, YAML-scalar-safe string for SKILL.md front matter: collapse
/// newlines/control chars and quote when needed so `:`, `---`, or leading
/// indicators cannot break the YAML block.
fn yaml_scalar(raw: &str) -> String {
    let one_line: String = raw
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let needs_quote = one_line.is_empty()
        || one_line.contains(':')
        || one_line.contains('#')
        || one_line.contains("---")
        || one_line.starts_with([
            '&', '*', '!', '|', '>', '%', '@', '`', '"', '\'', '{', '[', ',', '?',
        ])
        || one_line.starts_with('-');
    if needs_quote {
        format!(
            "\"{}\"",
            one_line.replace('\\', "\\\\").replace('"', "\\\"")
        )
    } else {
        one_line
    }
}

fn title(id: &str) -> String {
    id.replace('-', " ").to_case_titlelike()
}

// Tiny TitleCase helper without a dep.
trait CaseExt {
    fn to_case_titlelike(self) -> String;
}
impl CaseExt for String {
    fn to_case_titlelike(self) -> String {
        let mut out = String::with_capacity(self.len());
        let mut up = true;
        for ch in self.chars() {
            if ch.is_whitespace() {
                up = true;
                out.push(ch);
            } else if up {
                for u in ch.to_uppercase() {
                    out.push(u);
                }
                up = false;
            } else {
                out.push(ch);
            }
        }
        out
    }
}

/// Render a single `SkillIR` to its `render_targets`. Pure function —
/// deterministic for the same input (the basis of the golden test).
pub fn render_skill(skill: &SkillIR) -> Vec<RenderTarget> {
    let raw_id = &skill.id;
    let path_id = safe_component(&skill.id);
    let path_cat = safe_component(&skill.category);
    let header = format!(
        "Generated from Skill IR: {raw_id}@{version}\n\
         Do not edit this generated file directly. Update the Skill IR source instead.\n\n",
        version = skill.version
    );
    let title = title(raw_id);
    let triggers = skill.triggers.join(", ");
    let procedure = skill
        .procedure
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {s}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");
    let checklist = skill
        .procedure
        .iter()
        .map(|s| format!("- [ ] {s}"))
        .collect::<Vec<_>>()
        .join("\n");
    let evidence = skill
        .evidence_required
        .iter()
        .map(|s| format!("- {s}"))
        .collect::<Vec<_>>()
        .join("\n");
    let forbidden = skill
        .forbidden_behaviors
        .iter()
        .map(|s| format!("- {s}"))
        .collect::<Vec<_>>()
        .join("\n");
    let success = skill
        .success_criteria
        .iter()
        .map(|s| format!("- {s}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut out = Vec::new();
    for kind in &skill.render_targets {
        let (rel, content) = match kind.as_str() {
            "copilot_instruction" => (
                PathBuf::from(format!(".agent-harness/render-targets/copilot/{path_id}.instructions.md")),
                format!(
                    "{header}# {title}\n\n## Use this when\n\nUse this when triggers match: {triggers}.\n\nJob to be done: {jtbd}.\n\n## What to do\n\n{procedure}\n\n## Evidence required\n\n{evidence}\n\n## Forbidden behavior\n\n{forbidden}\n\n## Done when\n\n{success}\n",
                    jtbd = skill.jtbd
                ),
            ),
            "hook_prompt" => (
                PathBuf::from(format!(".agent-harness/render-targets/hooks/{path_id}.prompt.md")),
                format!(
                    "{header}# {title} Hook Prompt\n\n## Use this when\n\nUse this hook prompt when lifecycle context matches: {triggers}.\n\n## What to do\n\nRequire the agent to pursue: {jtbd}.\n\n{procedure}\n\n## Evidence required\n\n{evidence}\n\n## Forbidden behavior\n\n{forbidden}\n\n## Done when\n\n{success}\n\nHook should warn or block when evidence is skipped or completion is claimed without proof.\n",
                    jtbd = skill.jtbd
                ),
            ),
            "checklist" => (
                PathBuf::from(format!(".agent-harness/render-targets/checklists/{path_id}.md")),
                format!(
                    "{header}# {title} Checklist\n\n## Use this when\n\nUse this checklist when triggers match: {triggers}.\n\n## What to do\n\n{checklist}\n\n## Evidence required\n\n{evidence}\n\n## Forbidden behavior\n\n{forbidden}\n\n## Done when\n\n{success}\n",
                ),
            ),
            "claude_skill" => (
                PathBuf::from(format!(".agent-harness/render-targets/claude/{path_cat}/{path_id}/SKILL.md")),
                format!(
                    "---\nname: {name}\ndescription: {desc}\n---\n\n{header}# {title}\n\n## Use this when\n\nUse this when triggers match: {triggers}.\n\nJob to be done: {jtbd}.\n\n## What to do\n\n{procedure}\n\n## Evidence required\n\n{evidence}\n\n## Forbidden behavior\n\n{forbidden}\n\n## Done when\n\n{success}\n",
                    name = yaml_scalar(raw_id),
                    desc = yaml_scalar(&skill.description),
                    jtbd = skill.jtbd
                ),
            ),
            other => (
                PathBuf::from(format!(".agent-harness/render-targets/other/{path_id}.txt")),
                format!("{header}Unsupported render_target '{other}' for skill {raw_id}; no template.\n"),
            ),
        };
        out.push(RenderTarget {
            kind: kind.clone(),
            rel_path: rel,
            content,
        });
    }
    out
}

/// Render every non-blocked skill under `out_root`. Returns `(rendered,
/// blocked)`. Blocked skills (scan) are not rendered.
pub fn render_all(
    root: &Path,
    out_root: &Path,
    exceptions: &Exceptions,
) -> Result<(usize, Vec<String>)> {
    let mut rendered = 0usize;
    let mut blocked = Vec::new();
    for path in discover(root)? {
        let rec = ingest_one(&path)?;
        if scan_blocks_projection(&rec.scan, exceptions) {
            blocked.push(rec.ir.id.clone());
            continue;
        }
        for target in render_skill(&rec.ir) {
            let full = out_root.join(&target.rel_path);
            if let Some(p) = full.parent() {
                std::fs::create_dir_all(p).ok();
            }
            std::fs::write(&full, &target.content)?;
            rendered += 1;
        }
    }
    Ok((rendered, blocked))
}
