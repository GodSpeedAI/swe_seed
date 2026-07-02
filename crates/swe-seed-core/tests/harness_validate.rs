//! harness_validate: the Rust structure validator reports concrete errors on
//! an incomplete tree (CI integrity; port of the former `harness.py validate`).

use std::fs;

use swe_seed_core::harness_validate::validate;

fn temp_root() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-hv-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn missing_structure_produces_errors() {
    // An empty tree fails every structural check.
    let root = temp_root();
    let errors = validate(&root);
    assert!(
        !errors.is_empty(),
        "an empty tree must produce validation errors"
    );
    // The errors name the missing load-bearing structure.
    let joined = errors.join("\n");
    assert!(
        joined.contains("missing root spec"),
        "expected root-spec error: {joined}"
    );
    assert!(
        joined.contains("missing BAML source contract"),
        "expected baml error: {joined}"
    );
    assert!(
        joined.contains("missing required directory"),
        "expected dir error: {joined}"
    );
    assert!(
        joined.contains("missing route card for job type"),
        "expected job-type error: {joined}"
    );
    assert!(
        joined.contains("missing required active skill"),
        "expected active-skill error: {joined}"
    );
    fs::remove_dir_all(&root).ok();
}

#[test]
fn duplicate_skill_id_and_unresolved_route_skill_are_caught() {
    // A tree with a duplicate skill id and a route citing a missing skill is flagged.
    let root = temp_root();
    let h = root.join(".agent-harness");
    fs::create_dir_all(h.join("skills/x")).unwrap();
    fs::create_dir_all(h.join("routes")).unwrap();
    // minimal skill JSON (valid shape) duplicated under two paths with the same id.
    let skill = r#"{"id":"dup","version":"1","category":"c","jtbd":"j","description":"d","triggers":["t"],"procedure":["p"],"evidence_required":["e"],"forbidden_behaviors":["f"],"outputs":["o"],"success_criteria":["s"],"status":"active"}"#;
    fs::write(h.join("skills/x/a.json"), skill).unwrap();
    fs::write(h.join("skills/x/b.json"), skill).unwrap();
    // a route that requires a skill that does not exist.
    fs::write(
        h.join("routes/research.json"),
        r#"{"id":"research","job_type":"research","required_skills":["ghost-skill"],"proof":["python scripts/harness.py validate"]}"#,
    )
    .unwrap();

    let errors = validate(&root);
    let joined = errors.join("\n");
    assert!(
        joined.contains("duplicate skill id: dup"),
        "expected duplicate-id error: {joined}"
    );
    assert!(
        joined.contains("required_skill not found: ghost-skill"),
        "expected unresolved skill: {joined}"
    );
    assert!(
        joined.contains("proof references removed command surface"),
        "expected removed proof command error: {joined}"
    );
    fs::remove_dir_all(&root).ok();
}
