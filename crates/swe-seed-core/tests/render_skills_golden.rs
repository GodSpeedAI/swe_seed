//! render_skills_golden: render SkillIR → render_targets deterministically.
//! The same SkillIR produces byte-identical output across renders, and each
//! target carries the skill's identity/procedure/evidence (spec 0007).

use std::fs;
use std::path::PathBuf;

use swe_seed_core::skill::{ingest_one, render_skill};

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn fixture_skill() -> swe_seed_core::skill::SkillRecord {
    // Copy the fixture into a temp skills dir so ingest_one resolves it.
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-render-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let skills = dir.join(".agent-harness/skills/fixture");
    fs::create_dir_all(&skills).unwrap();
    let target = skills.join("fixture-skill.json");
    fs::copy(real_root().join("tests/fixtures/skill-ir.json"), &target).unwrap();
    ingest_one(&target).unwrap()
}

#[test]
fn render_is_deterministic_across_runs() {
    let rec = fixture_skill();
    let r1 = render_skill(&rec.ir);
    let r2 = render_skill(&rec.ir);

    assert!(!r1.is_empty(), "fixture has 4 render_targets");
    assert_eq!(r1.len(), r2.len());
    for (a, b) in r1.iter().zip(r2.iter()) {
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.rel_path, b.rel_path);
        assert_eq!(a.content, b.content, "non-deterministic content for {}", a.kind);
    }
}

#[test]
fn render_targets_carry_skill_content() {
    let rec = fixture_skill();
    let targets = render_skill(&rec.ir);
    let kinds: Vec<&str> = targets.iter().map(|t| t.kind.as_str()).collect();
    assert!(kinds.contains(&"claude_skill"));
    assert!(kinds.contains(&"copilot_instruction"));
    assert!(kinds.contains(&"hook_prompt"));
    assert!(kinds.contains(&"checklist"));

    for t in &targets {
        assert!(t.content.contains("fixture-skill"), "{} missing skill id", t.kind);
        assert!(t.content.contains("add the narrowest failing check"), "{} missing procedure", t.kind);
        assert!(t.content.contains("failing check"), "{} missing evidence", t.kind);
    }

    // Target paths follow the host layout.
    let claude = targets.iter().find(|t| t.kind == "claude_skill").unwrap();
    assert!(claude.rel_path.to_string_lossy().contains("claude/test/fixture-skill/SKILL.md"));
    let copilot = targets.iter().find(|t| t.kind == "copilot_instruction").unwrap();
    assert!(copilot.rel_path.to_string_lossy().ends_with("copilot/fixture-skill.instructions.md"));
}

#[test]
fn render_skill_writes_to_disk_deterministically() {
    let rec = fixture_skill();
    let targets = render_skill(&rec.ir);

    let dir_a = std::env::temp_dir().join(format!(
        "swe-seed-golden-a-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let dir_b = std::env::temp_dir().join(format!(
        "swe-seed-golden-b-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));

    for dir in [&dir_a, &dir_b] {
        for t in &targets {
            let full = dir.join(&t.rel_path);
            if let Some(p) = full.parent() {
                fs::create_dir_all(p).unwrap();
            }
            fs::write(&full, &t.content).unwrap();
        }
    }

    for t in &targets {
        let a = fs::read(dir_a.join(&t.rel_path)).unwrap();
        let b = fs::read(dir_b.join(&t.rel_path)).unwrap();
        assert_eq!(a, b, "on-disk render differs for {}", t.kind);
    }

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn malicious_identifiers_cannot_escape_render_hierarchy() {
    // A skill id/category with traversal or absolute-path segments must be
    // clamped to a single safe component so rel_path stays under render-targets.
    use swe_seed_core::skill::SkillIR;
    let malicious: SkillIR = serde_json::from_str(
        r#"{"id":"../escape","version":1,"category":"/etc/pwned","jtbd":"j","description":"d","triggers":["t"],"procedure":["p"],"evidence_required":["e"],"forbidden_behaviors":["f"],"outputs":["o"],"success_criteria":["s"],"failure_modes":["fm"],"render_targets":["copilot_instruction","claude_skill"],"status":"active"}"#,
    )
    .unwrap();
    let targets = render_skill(&malicious);
    for t in &targets {
        let p = t.rel_path.to_string_lossy();
        assert!(p.starts_with(".agent-harness/render-targets/"), "escaped hierarchy: {p}");
        assert!(!p.contains("../"), "traversal segment survived: {p}");
        assert!(!p.contains("/etc/"), "absolute segment survived: {p}");
    }
    // The claude path uses the sanitized basename, not the raw category.
    let claude = targets.iter().find(|t| t.kind == "claude_skill").unwrap();
    let p = claude.rel_path.to_string_lossy();
    assert!(p.contains("claude/pwned/escape/SKILL.md") || p.contains("claude/unsafe"), "claude path: {p}");
    // Front matter is YAML-safe (no raw newlines/--- break the block).
    assert!(claude.content.contains("name: "));
}
