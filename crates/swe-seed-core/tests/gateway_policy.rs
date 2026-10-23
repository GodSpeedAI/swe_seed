//! Gateway stage-3 integration proof: catalog projection + invoke gates compose
//! end-to-end (spec 0020 §8, §10). Run: `cargo test gateway_policy`.

use std::collections::HashMap;

use swe_seed_core::gateway::{
    evaluate_invoke, project_catalog, InvokeDecision, MCPServer,
};
use swe_seed_core::hooks::policy::PermissionPolicy;

fn server_with(entries: Vec<swe_seed_core::gateway::DeclaredCatalogEntry>) -> MCPServer {
    swe_seed_core::gateway::MCPServer {
        id: "fs".into(),
        namespace: "fs".into(),
        transport: swe_seed_core::gateway::GatewayTransport::Stdio,
        command_or_url: "echo".into(),
        policy_tags: vec![],
        source_hash: Some("sha256:src".into()),
        scan_status: Some("clear".into()),
        catalog: entries,
        ..Default::default()
    }
}

fn declared(name: &str, tags: &[&str], pinned: Option<&str>) -> swe_seed_core::gateway::DeclaredCatalogEntry {
    let mut pinned_hashes = HashMap::new();
    if let Some(p) = pinned {
        pinned_hashes.insert(format!("fs.{name}"), p.to_string());
    }
    swe_seed_core::gateway::DeclaredCatalogEntry {
        kind: swe_seed_core::gateway::CatalogKind::Tool,
        name: name.into(),
        description: "d".into(),
        input_schema: None,
        policy_tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn projected_readonly_entry_allows() {
    let (catalog, dropped) = project_catalog(&[server_with(vec![declared("read", &["readonly"], None)])]);
    assert!(dropped.is_empty());
    let entry = &catalog[0];
    let decision = evaluate_invoke(&PermissionPolicy::default(), entry, Some("clear"), true).unwrap();
    assert_eq!(decision, InvokeDecision::Allow);
}

#[test]
fn projected_pinned_hash_mismatch_blocks_after_projection() {
    // Pin a DIFFERENT hash than the projected definition_hash → call-time block.
    let mut s = server_with(vec![declared("read", &["readonly"], None)]);
    // compute real hash, then pin something else
    let (cat, _) = project_catalog(&[s.clone()]);
    let real = cat[0].definition_hash.clone();
    s.pinned_hashes
        .insert("fs.read".into(), format!("{real}-tampered"));
    let (catalog, _) = project_catalog(&[s]);
    let err = evaluate_invoke(&PermissionPolicy::default(), &catalog[0], Some("clear"), true).unwrap_err();
    assert_eq!(err.reason_code(), "capability_hash_mismatch");
}

#[test]
fn projected_risky_unscanned_fail_closed() {
    let mut s = server_with(vec![declared("write", &["write"], None)]);
    s.scan_status = None; // unscanned
    let (catalog, _) = project_catalog(&[s]);
    let err = evaluate_invoke(&PermissionPolicy::default(), &catalog[0], None, true).unwrap_err();
    assert_eq!(err.reason_code(), "scan_blocked");
}

#[test]
fn projected_approval_gated_tag_requires_approval() {
    let mut p = PermissionPolicy::default();
    p.approval_gated_actions = vec!["deploy".into()];
    let (catalog, _) = project_catalog(&[server_with(vec![declared("deploy", &["deploy"], None)])]);
    let decision = evaluate_invoke(&p, &catalog[0], Some("clear"), true).unwrap();
    assert_eq!(
        decision,
        InvokeDecision::ApprovalRequired {
            namespaced_name: "fs.deploy".into()
        }
    );
}
