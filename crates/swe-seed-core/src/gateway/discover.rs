//! Live catalog discovery fan-out (spec 0020 §7 "Live Catalog Discovery").
//! At gateway startup and on each validated reload, this module asks each
//! registered backend for its `tools`/`resources`/`prompts` and parses the
//! responses into namespaced, hashed catalog entries. The MERGE with the
//! declared catalog (declared-wins) lives in `catalog::merge_live` — catalog
//! construction has one owner.
//!
//! Per-backend isolation (spec 0020 §7): a timeout, connection failure, or
//! malformed response from one backend yields a `DiscoveryNote` and continues;
//! it never fails discovery for the other backends, and it never removes that
//! backend's declared entries (those are already in the catalog via
//! `project_catalog`).

use serde_json::Value;

use super::catalog::{definition_hash, validate_local_name};
use super::routing::Router;
use super::{CatalogKind, GatewayCatalogEntry, MCPServer};

/// Per-backend discovery note: a backend that contributed no live entries for a
/// method, with a deterministic reason. Not a catalog entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryNote {
    pub backend_id: String,
    pub method: String,
    pub reason: String,
}

/// Bounded discovery timeout per backend. A slow backend yields a note and is
/// skipped; it does not block the gateway from coming up.
const DISCOVERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Fan out discovery to every registered backend. Returns live catalog entries
/// (namespaced + hashed, NOT yet merged with declared) plus per-backend notes.
///
/// Per-backend isolation holds: any error/timeout/empty-response from one
/// backend yields a note and the loop continues to the next backend.
pub fn discover_live(
    router: &Router,
    servers: &[MCPServer],
) -> (Vec<GatewayCatalogEntry>, Vec<DiscoveryNote>) {
    let mut live = Vec::new();
    let mut notes = Vec::new();
    for server in servers {
        if !router.has(&server.id) {
            // No transport registered (SSE/Websocket not forwarded in v0.1, or
            // an empty command). Declared entries, if any, remain via
            // project_catalog; this backend simply contributes no live entries.
            notes.push(DiscoveryNote {
                backend_id: server.id.clone(),
                method: "*".into(),
                reason: "no_transport".into(),
            });
            continue;
        }
        discover_one(
            router,
            server,
            "tools/list",
            "tools",
            CatalogKind::Tool,
            &mut live,
            &mut notes,
        );
        discover_one(
            router,
            server,
            "resources/list",
            "resources",
            CatalogKind::Resource,
            &mut live,
            &mut notes,
        );
        discover_one(
            router,
            server,
            "prompts/list",
            "prompts",
            CatalogKind::Prompt,
            &mut live,
            &mut notes,
        );
    }
    (live, notes)
}

fn discover_one(
    router: &Router,
    server: &MCPServer,
    method: &str,
    result_key: &str,
    kind: CatalogKind,
    live: &mut Vec<GatewayCatalogEntry>,
    notes: &mut Vec<DiscoveryNote>,
) {
    let resp = router.invoke(
        &server.id,
        &Value::Null,
        method,
        Value::Object(Default::default()),
        DISCOVERY_TIMEOUT,
    );
    let result = match resp {
        Ok(r) => r,
        Err(e) => {
            notes.push(DiscoveryNote {
                backend_id: server.id.clone(),
                method: method.into(),
                reason: e.reason_code().into(),
            });
            return;
        }
    };
    let items = match result.get(result_key).and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return, // shape we don't recognize — nothing to add
    };
    for item in items {
        let local = match item.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => continue,
        };
        if validate_local_name(local).is_err() {
            notes.push(DiscoveryNote {
                backend_id: server.id.clone(),
                method: method.into(),
                reason: format!("invalid_local_name:{local}"),
            });
            continue;
        }
        let namespaced_name = format!("{}.{}", server.namespace, local);
        let schema = item
            .get("inputSchema")
            .or_else(|| item.get("input_schema"))
            .cloned();
        let wire = serde_json::json!({
            "name": namespaced_name,
            "local_name": local,
            "description": item.get("description").cloned().unwrap_or(Value::Null),
            "input_schema": schema.clone().unwrap_or(Value::Null),
        });
        live.push(GatewayCatalogEntry {
            kind,
            namespaced_name,
            backend_id: server.id.clone(),
            definition_hash: definition_hash(&wire),
            wire_definition: wire,
            source_hash: server.source_hash.clone(),
            policy_tags: server.policy_tags.clone(),
            pinned_hash: None, // live-only entries have no operator pin
            input_schema: schema,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::{merge_live, MockTransport, Router};
    use serde_json::json;

    fn server(id: &str, ns: &str) -> MCPServer {
        MCPServer {
            id: id.into(),
            namespace: ns.into(),
            ..Default::default()
        }
    }

    #[test]
    fn discover_adds_live_tools_namespaced() {
        let t = MockTransport::responding(
            r#"{"jsonrpc":"2.0","id":null,"result":{"tools":[{"name":"search","description":"d","inputSchema":{"type":"object"}}]}}"#,
        );
        let router = Router::with("fs", Box::new(t));
        let (live, notes) = discover_live(&router, &[server("fs", "fs")]);
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].namespaced_name, "fs.search");
        assert_eq!(live[0].backend_id, "fs");
        assert!(live[0].definition_hash.starts_with("sha256:"));
        assert!(
            notes.is_empty(),
            "no notes for a healthy backend: {notes:?}"
        );
    }

    #[test]
    fn backend_failure_is_isolated_and_noted() {
        let down = MockTransport {
            drop_request: true,
            ..MockTransport::responding("")
        };
        let up = MockTransport::responding(
            r#"{"jsonrpc":"2.0","id":null,"result":{"tools":[{"name":"ping"}]}}"#,
        );
        let mut router = Router::new();
        router.register("a", Box::new(down));
        router.register("b", Box::new(up));
        let (live, notes) = discover_live(&router, &[server("a", "a"), server("b", "b")]);
        // backend b's tool survived despite a's failure
        assert!(live.iter().any(|e| e.namespaced_name == "b.ping"));
        // backend a produced per-method failure notes (isolated, not fatal)
        assert!(notes.iter().all(|n| n.backend_id == "a"));
        assert!(notes.iter().any(|n| n.method == "tools/list"));
    }

    #[test]
    fn no_transport_backend_is_noted_not_fatal() {
        let up = MockTransport::responding(
            r#"{"jsonrpc":"2.0","id":null,"result":{"tools":[{"name":"ping"}]}}"#,
        );
        let router = Router::with("b", Box::new(up));
        // "a" has no registered transport; "b" does.
        let (live, notes) = discover_live(&router, &[server("a", "a"), server("b", "b")]);
        assert!(live.iter().any(|e| e.namespaced_name == "b.ping"));
        let a_note = notes.iter().find(|n| n.backend_id == "a").unwrap();
        assert_eq!(a_note.reason, "no_transport");
    }

    #[test]
    fn merge_declared_wins_over_live_conflict() {
        let declared = vec![GatewayCatalogEntry {
            kind: CatalogKind::Tool,
            namespaced_name: "fs.read".into(),
            backend_id: "fs".into(),
            wire_definition: json!({"name":"fs.read"}),
            definition_hash: "sha256:declared".into(),
            source_hash: None,
            policy_tags: vec![],
            pinned_hash: Some("sha256:declared".into()),
            input_schema: None,
        }];
        let live = vec![
            // conflicts with declared → dropped
            GatewayCatalogEntry {
                kind: CatalogKind::Tool,
                namespaced_name: "fs.read".into(),
                backend_id: "fs".into(),
                wire_definition: json!({"name":"fs.read","live":true}),
                definition_hash: "sha256:live".into(),
                source_hash: None,
                policy_tags: vec![],
                pinned_hash: None,
                input_schema: None,
            },
            // live-only → added
            GatewayCatalogEntry {
                kind: CatalogKind::Tool,
                namespaced_name: "fs.search".into(),
                backend_id: "fs".into(),
                wire_definition: json!({"name":"fs.search"}),
                definition_hash: "sha256:search".into(),
                source_hash: None,
                policy_tags: vec![],
                pinned_hash: None,
                input_schema: None,
            },
        ];
        let (merged, dropped) = merge_live(declared, live);
        assert_eq!(merged.len(), 2);
        // declared entry retained its hash (live did not replace it)
        let read = merged
            .iter()
            .find(|e| e.namespaced_name == "fs.read")
            .unwrap();
        assert_eq!(read.definition_hash, "sha256:declared");
        assert!(merged.iter().any(|e| e.namespaced_name == "fs.search"));
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].reason, "conflicts_with_declared");
    }

    #[test]
    fn merge_drops_duplicate_live_entries() {
        let live = vec![
            entry("fs.a", "sha256:1"),
            entry("fs.a", "sha256:2"), // duplicate live name → dropped
            entry("fs.b", "sha256:3"),
        ];
        let (merged, dropped) = merge_live(vec![], live);
        assert_eq!(merged.len(), 2);
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].reason, "duplicate_live");
        // first live entry wins
        assert_eq!(
            merged
                .iter()
                .find(|e| e.namespaced_name == "fs.a")
                .unwrap()
                .definition_hash,
            "sha256:1"
        );
    }

    fn entry(name: &str, hash: &str) -> GatewayCatalogEntry {
        GatewayCatalogEntry {
            kind: CatalogKind::Tool,
            namespaced_name: name.into(),
            backend_id: "fs".into(),
            wire_definition: json!({"name":name}),
            definition_hash: hash.into(),
            source_hash: None,
            policy_tags: vec![],
            pinned_hash: None,
            input_schema: None,
        }
    }
}
