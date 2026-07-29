//! OpenAPI import (spec 0020 §16). Maps OpenAPI operations into registry-
//! backed gateway catalog entries. Import is idempotent: the same source
//! document yields the same operation names, namespaced names, definition
//! hashes, and source hash — no semantic diff on re-import (spec 0020 §16).
//! The source document hash is carried as the `MCPServer.source_hash`
//! provenance link (spec 0009 vocabulary).

use serde_json::Value;

use super::config::{DeclaredCatalogEntry, MCPServer};
use super::{canonical_json, CatalogKind, GatewayError, GatewayTransport};

const METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "head"];

/// Import an OpenAPI 3.x document as a gateway `MCPServer` (one HTTP backend).
/// `namespace` is the operator-chosen backend namespace; operation ids become
/// local tool names (`<namespace>.<operationId>`).
pub fn import_openapi(namespace: &str, doc: &Value) -> Result<MCPServer, GatewayError> {
    // Source hash over canonical JSON → provenance link (idempotency anchor).
    let source_hash = crate::provenance::hash::content_hash(canonical_json(doc).as_bytes());

    let server_url = doc
        .get("servers")
        .and_then(|s| s.get(0))
        .and_then(|s| s.get("url"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| GatewayError::InvalidRequest {
            reason: "openapi doc missing servers[0].url".into(),
        })?
        .to_string();

    let paths = doc
        .get("paths")
        .and_then(|p| p.as_object())
        .ok_or_else(|| GatewayError::InvalidRequest {
            reason: "openapi doc missing paths object".into(),
        })?;

    let mut entries: Vec<DeclaredCatalogEntry> = Vec::new();
    for (path, path_item) in paths {
        let path_item = match path_item.as_object() {
            Some(o) => o,
            None => continue,
        };
        // Path-level parameters apply to every operation under this path.
        let path_params = path_item.get("parameters").cloned().unwrap_or(Value::Array(vec![]));
        for &method in METHODS {
            let op = match path_item.get(method) {
                Some(o) => o,
                None => continue,
            };
            let local_name = operation_local_name(op, method, path);
            let description = op
                .get("summary")
                .and_then(|s| s.as_str())
                .or_else(|| op.get("description").and_then(|s| s.as_str()))
                .unwrap_or("")
                .to_string();
            let input_schema = build_input_schema(op, &path_params);
            entries.push(DeclaredCatalogEntry {
                kind: CatalogKind::Tool,
                name: local_name,
                description,
                input_schema,
                policy_tags: vec!["http".into(), "openapi-imported".into()],
            });
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(MCPServer {
        id: namespace.into(),
        namespace: namespace.into(),
        transport: GatewayTransport::Http,
        command_or_url: server_url,
        source_hash: Some(source_hash),
        policy_tags: vec!["openapi-imported".into()],
        catalog: entries,
        ..Default::default()
    })
}

/// operationId if present; otherwise derive deterministically from method+path
/// (spec 0020 §16: deterministic operation naming).
fn operation_local_name(op: &Value, method: &str, path: &str) -> String {
    if let Some(id) = op.get("operationId").and_then(|v| v.as_str()) {
        return slug(id);
    }
    let p = path
        .trim_matches('/')
        .replace('/', "_")
        .replace(['{', '}'], "");
    format!("{}_{}", method, p)
}

fn slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Build a JSON Schema `object` from operation parameters (path/query/header)
/// and the request body schema. Deterministic given the same operation def.
fn build_input_schema(op: &Value, path_params: &Value) -> Option<Value> {
    let mut properties = serde_json::Map::new();
    let mut required: Vec<String> = Vec::new();
    // merge path-level params then op-level params
    for params_src in [path_params.clone(), op.get("parameters").cloned().unwrap_or(Value::Null)] {
        if let Some(arr) = params_src.as_array() {
            for p in arr {
                let name = match p.get("name").and_then(|n| n.as_str()) {
                    Some(n) => n,
                    None => continue,
                };
                let schema = p.get("schema").cloned().unwrap_or(Value::Null);
                properties.insert(name.into(), schema);
                if p.get("required").and_then(|r| r.as_bool()).unwrap_or(false) {
                    required.push(name.into());
                }
            }
        }
    }
    // request body
    if let Some(body_schema) = op
        .get("requestBody")
        .and_then(|rb| rb.get("content"))
        .and_then(|c| c.get("application/json"))
        .and_then(|j| j.get("schema"))
        .cloned()
    {
        if let Some(obj) = body_schema.get("properties").and_then(|p| p.as_object()) {
            for (k, v) in obj {
                properties.insert(k.clone(), v.clone());
            }
        }
        if let Some(req) = body_schema.get("required").and_then(|r| r.as_array()) {
            for r in req {
                if let Some(s) = r.as_str() {
                    required.push(s.into());
                }
            }
        }
    }
    if properties.is_empty() {
        return None;
    }
    required.sort();
    required.dedup();
    Some(serde_json::json!({
        "type": "object",
        "properties": Value::Object(properties),
        "required": required,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_doc() -> Value {
        json!({
            "openapi": "3.0.0",
            "servers": [{"url": "https://api.example.com"}],
            "paths": {
                "/users/{id}": {
                    "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "get": {
                        "operationId": "getUser",
                        "summary": "get a user",
                        "parameters": [{"name": "verbose", "in": "query", "schema": {"type": "boolean"}}]
                    },
                    "delete": {"operationId": "deleteUser", "summary": "delete"}
                },
                "/health": {
                    "get": { "summary": "health check" } // no operationId → derived
                }
            }
        })
    }

    #[test]
    fn imports_operations_as_catalog_entries() {
        let server = import_openapi("api", &sample_doc()).unwrap();
        assert_eq!(server.namespace, "api");
        assert_eq!(server.transport, GatewayTransport::Http);
        assert_eq!(server.command_or_url, "https://api.example.com");
        assert!(server.source_hash.as_deref().unwrap().starts_with("sha256:"));
        let names: Vec<&str> = server.catalog.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"getuser"));
        assert!(names.contains(&"deleteuser"));
        // derived name from method+path when operationId absent
        assert!(names.iter().any(|n| n.contains("health")));
    }

    #[test]
    fn input_schema_merges_path_query_and_body() {
        let server = import_openapi("api", &sample_doc()).unwrap();
        let get_user = server
            .catalog
            .iter()
            .find(|e| e.name == "getuser")
            .unwrap();
        let schema = get_user.input_schema.as_ref().unwrap();
        let props = schema.get("properties").unwrap().as_object().unwrap();
        assert!(props.contains_key("id")); // path param
        assert!(props.contains_key("verbose")); // query param
        assert!(schema
            .get("required")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some("id")));
    }

    #[test]
    fn import_is_idempotent() {
        let a = import_openapi("api", &sample_doc()).unwrap();
        let b = import_openapi("api", &sample_doc()).unwrap();
        assert_eq!(a.source_hash, b.source_hash);
        assert_eq!(a.catalog.len(), b.catalog.len());
        let anames: Vec<String> = a.catalog.iter().map(|e| e.name.clone()).collect();
        let bnames: Vec<String> = b.catalog.iter().map(|e| e.name.clone()).collect();
        assert_eq!(anames, bnames);
        // definition hashes (computed downstream) would also match because the
        // wire definitions are byte-identical for identical inputs.
        assert_eq!(
            super::canonical_json(&serde_json::to_value(&a.catalog).unwrap()),
            super::canonical_json(&serde_json::to_value(&b.catalog).unwrap()),
        );
    }

    #[test]
    fn missing_servers_url_is_invalid_request() {
        let err = import_openapi("api", &json!({"paths": {}})).unwrap_err();
        assert_eq!(err.reason_code(), "invalid_request");
    }
}
