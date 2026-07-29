//! Gateway stage-2 proof (spec 0020 §6, §7, §11): registry-derived catalog,
//! deterministic namespace validation, canonical hashing, compact discovery
//! with size caps + dropped-entry accounting. Run: `cargo test gateway_catalog`.

use std::collections::HashMap;
use std::fs;

use swe_seed_core::gateway::{
    canonical_json, definition_hash, list_catalog, project_backend, project_catalog,
    search_catalog, validate_namespace, CatalogKind, DeclaredCatalogEntry, GatewayConfig,
    GatewayTransport, MCPServer, NamespaceError, DEFAULT_DISCOVERY_CAP,
};

fn server(namespace: &str, entries: Vec<DeclaredCatalogEntry>) -> MCPServer {
    MCPServer {
        id: namespace.into(),
        namespace: namespace.into(),
        transport: GatewayTransport::Stdio,
        command_or_url: "echo".into(),
        policy_tags: vec!["base".into()],
        source_hash: Some("sha256:src".into()),
        catalog: entries,
        pinned_hashes: HashMap::new(),
        ..Default::default()
    }
}

fn declared(name: &str, desc: &str) -> DeclaredCatalogEntry {
    DeclaredCatalogEntry {
        kind: CatalogKind::Tool,
        name: name.into(),
        description: desc.into(),
        input_schema: Some(serde_json::json!({"type":"object"})),
        policy_tags: vec!["extra".into()],
    }
}

#[test]
fn namespace_format_and_uniqueness() {
    assert!(validate_namespace("fs").is_ok());
    assert!(validate_namespace("UPPER").is_err());
    assert!(validate_namespace("a.b").is_err());
    let mut cfg = GatewayConfig::default();
    cfg.servers = vec![server("fs", vec![]), server("fs", vec![])];
    assert_eq!(
        cfg.validate_namespaces(),
        Err(NamespaceError::Duplicate("fs".into()))
    );
}

#[test]
fn project_catalog_builds_namespaced_entries_with_hashes_and_merged_tags() {
    let servers = vec![server(
        "fs",
        vec![declared("read", "read a file"), declared("write", "write a file")],
    )];
    let (catalog, dropped) = project_catalog(&servers);
    assert!(dropped.is_empty());
    assert_eq!(catalog.len(), 2);
    assert_eq!(catalog[0].namespaced_name, "fs.read");
    assert_eq!(catalog[0].backend_id, "fs");
    // server tags ++ entry tags
    assert!(catalog[0].policy_tags.contains(&"base".to_string()));
    assert!(catalog[0].policy_tags.contains(&"extra".to_string()));
    assert_eq!(catalog[0].source_hash.as_deref(), Some("sha256:src"));
    // hash is sha256:<hex> over canonical wire JSON
    assert!(catalog[0].definition_hash.starts_with("sha256:"));
    // catalog is sorted by namespaced name
    assert_eq!(catalog[0].namespaced_name, "fs.read");
    assert_eq!(catalog[1].namespaced_name, "fs.write");
}

#[test]
fn definition_hash_is_byte_stable_across_key_order() {
    let a = serde_json::json!({"name":"fs.read","local_name":"read","description":"d","input_schema":null});
    let b = serde_json::json!({"input_schema":null,"description":"d","local_name":"read","name":"fs.read"});
    assert_eq!(definition_hash(&a), definition_hash(&b));
    // canonical form is sorted + whitespace-free
    assert_eq!(canonical_json(&a), canonical_json(&b));
    assert!(!canonical_json(&a).contains(' '));
}

#[test]
fn pinned_hash_from_registry_is_attached_to_entry() {
    let mut s = server("fs", vec![declared("read", "d")]);
    s.pinned_hashes
        .insert("fs.read".into(), "sha256:expected".into());
    let (catalog, _) = project_catalog(&[s]);
    assert_eq!(catalog[0].pinned_hash.as_deref(), Some("sha256:expected"));
}

#[test]
fn invalid_local_name_is_dropped_deterministically() {
    let servers = vec![server(
        "fs",
        vec![declared("good", "ok"), declared("bad.name", "dot forbidden")],
    )];
    let (catalog, dropped) = project_catalog(&servers);
    assert_eq!(catalog.len(), 1);
    assert_eq!(dropped.len(), 1);
    assert_eq!(dropped[0].name, "bad.name");
    assert_eq!(dropped[0].reason, "invalid_local_name");
    assert_eq!(dropped[0].server_id, "fs");
}

#[test]
fn list_catalog_caps_and_reports_truncation() {
    let entries: Vec<DeclaredCatalogEntry> = (0..(DEFAULT_DISCOVERY_CAP + 5))
        .map(|i| declared(&format!("t{i}"), "desc"))
        .collect();
    let (catalog, _) = project_catalog(&[server("fs", entries)]);
    let (items, dropped) = list_catalog(&catalog, DEFAULT_DISCOVERY_CAP);
    assert_eq!(items.len(), DEFAULT_DISCOVERY_CAP);
    assert_eq!(dropped, 5);
    // compact items carry no schema
    let json = serde_json::to_string(&items[0]).unwrap();
    assert!(!json.contains("input_schema"));
    assert!(json.contains("\"namespaced_name\""));
}

#[test]
fn search_catalog_filters_and_caps() {
    let servers = vec![server(
        "fs",
        vec![
            declared("read", "read a file"),
            declared("write", "write a file"),
            declared("list", "list directory"),
        ],
    )];
    let (catalog, _) = project_catalog(&servers);
    let (hits, dropped) = search_catalog(&catalog, "file", 64);
    assert_eq!(hits.len(), 2);
    assert_eq!(dropped, 0);
    // matches are by description ("file"); the two are read + write
    let names: Vec<&str> = hits.iter().map(|h| h.namespaced_name.as_str()).collect();
    assert!(names.contains(&"fs.read"));
    assert!(names.contains(&"fs.write"));
    // cap behavior
    let (_, capped_dropped) = search_catalog(&catalog, "file", 1);
    assert_eq!(capped_dropped, 1);
}

#[test]
fn config_load_missing_file_is_standalone_default() {
    let root = temp_root();
    let cfg = GatewayConfig::load(&root).unwrap();
    assert!(cfg.servers.is_empty());
    assert!(cfg.federation.is_standalone());
    assert!(cfg.listener.is_loopback());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn config_load_parses_written_file() {
    let root = temp_root();
    fs::create_dir_all(root.join(".swe-seed/gateway")).unwrap();
    let json = serde_json::json!({
        "servers": [{
            "id": "fs", "namespace": "fs", "transport": "stdio",
            "command_or_url": "echo",
            "catalog": [{"kind":"tool","name":"read","description":"d"}]
        }],
        "listener": {"bind": "0.0.0.0", "tls": true}
    });
    fs::write(
        root.join(".swe-seed/gateway/config.json"),
        serde_json::to_string_pretty(&json).unwrap(),
    )
    .unwrap();
    let cfg = GatewayConfig::load(&root).unwrap();
    assert_eq!(cfg.servers.len(), 1);
    assert_eq!(cfg.servers[0].namespace, "fs");
    assert!(!cfg.listener.is_loopback());
    assert!(cfg.listener.exposure_ok()); // tls on
    assert!(cfg.validate_namespaces().is_ok());
    let backends: Vec<_> = cfg.servers.iter().map(project_backend).collect();
    assert_eq!(backends[0].namespace, "fs");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn config_rejects_duplicate_namespace_at_load_validation() {
    let cfg = GatewayConfig {
        servers: vec![server("fs", vec![]), server("fs", vec![])],
        ..Default::default()
    };
    assert_eq!(
        cfg.validate_namespaces(),
        Err(NamespaceError::Duplicate("fs".into()))
    );
}

fn temp_root() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-gw-cat-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
