//! context_plan: the budget is enforced — excluded files are removed, raw
//! output is capped, and stale context is warned (spec 0015).

use std::fs::{self, FileTimes};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use swe_seed_core::context::{
    build_pack, cap_lines, context_plan, parse_line_cap, ContextBudget, DEFAULT_STALE_THRESHOLD,
};

fn real_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-ctx-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn budget(required: &[&str], excluded: &[&str]) -> ContextBudget {
    ContextBudget {
        id: "b".into(),
        max_context_size: "200".into(),
        required_files: required.iter().map(|s| s.to_string()).collect(),
        optional_files: vec![],
        excluded_files: excluded.iter().map(|s| s.to_string()).collect(),
        freshness_requirements: vec![],
        relevance_rules: vec![],
        summarization_rules: vec![],
        validation: vec![],
    }
}

#[test]
fn excluded_files_are_removed_from_the_pack() {
    let root = temp_root();
    fs::write(root.join("keep.md"), "x").unwrap();
    fs::write(root.join("secret.env"), "x").unwrap();
    let b = budget(&["keep.md", "secret.env"], &["secret.env"]);
    let pack = build_pack(
        &root,
        &b,
        "card",
        DEFAULT_STALE_THRESHOLD,
        SystemTime::now(),
    );
    assert!(pack.included_files.contains(&"keep.md".to_string()));
    assert!(!pack.included_files.contains(&"secret.env".to_string()));
    assert!(pack.excluded_files.contains(&"secret.env".to_string()));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn raw_output_is_capped() {
    // A 300-line blob is capped to the budget's line cap (200) + a note.
    let b = budget(&[], &[]);
    let cap = parse_line_cap(&b, 200);
    assert_eq!(cap, 200);
    let big: String = (0..300)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let capped = cap_lines(&big, cap);
    assert_eq!(capped.lines().count(), 201);
    assert!(capped.contains("truncated"));
}

#[test]
fn stale_context_is_warned() {
    let root = temp_root();
    let fresh = root.join("fresh.md");
    let stale = root.join("stale.md");
    fs::write(&fresh, "x").unwrap();
    fs::write(&stale, "x").unwrap();
    // Force stale.md's mtime 60 days in the past.
    let old = SystemTime::now() - Duration::from_secs(60 * 24 * 3600);
    let times = FileTimes::new().set_modified(old).set_accessed(old);
    let f = std::fs::File::open(&stale).unwrap();
    f.set_times(times).unwrap();
    drop(f);

    let b = budget(&["fresh.md", "stale.md"], &[]);
    let pack = build_pack(
        &root,
        &b,
        "card",
        DEFAULT_STALE_THRESHOLD,
        SystemTime::now(),
    );
    assert!(
        pack.stale_context_warnings
            .iter()
            .any(|w| w.contains("stale.md")),
        "stale.md should be warned: {:?}",
        pack.stale_context_warnings
    );
    assert!(
        !pack
            .stale_context_warnings
            .iter()
            .any(|w| w.contains("fresh.md")),
        "fresh.md must not be warned"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn context_plan_routes_and_emits_read_order() {
    // End-to-end against the real repo: routes "checkpoint smoke", loads the
    // budget policy, and emits a read order capped at the policy's 200 lines.
    let plan = context_plan(&real_root(), "checkpoint smoke").expect("context-plan");
    assert_eq!(plan["route"]["job_type"], "test");
    let read_order = plan["context_budget"]["read_order"].as_array().unwrap();
    assert!(!read_order.is_empty());
    assert!(
        read_order
            .iter()
            .any(|v| v.as_str().unwrap_or("").contains("budget-policy.yaml")),
        "read order should include the budget policy: {read_order:?}"
    );
    assert_eq!(plan["context_budget"]["raw_output_cap_lines"], 200);
}
