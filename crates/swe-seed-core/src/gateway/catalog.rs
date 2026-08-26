//! Registry-derived catalog + compact discovery (spec 0020 §6, §7, §11).
//!
//! The catalog is a runtime *projection* of the registry (`MCPServer` declared
//! entries); live `tools/list` discovery (stage 5) augments it. Discovery must
//! stay compact: `list`/`search` return `CompactCatalogItem` (name + short
//! description), never full backend schemas dumped into model context (§11).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::config::{DeclaredCatalogEntry, MCPServer};
use super::{CatalogKind, GatewayCatalogEntry};

/// Namespace / local-name validation error (spec 0020 §7: namespace MUST be
/// unique and namespaced calls MUST route only to their owning backend).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamespaceError {
    /// Empty or illegal characters (allowed: `a-z 0-9 _ -`, 1..=63 chars).
    Invalid(String),
    /// Two servers share a namespace.
    Duplicate(String),
}

impl std::fmt::Display for NamespaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceError::Invalid(ns) => write!(
                f,
                "invalid namespace '{ns}': must be 1..=63 chars of [a-z0-9_-]"
            ),
            NamespaceError::Duplicate(ns) => {
                write!(f, "duplicate namespace '{ns}'")
            }
        }
    }
}
impl std::error::Error for NamespaceError {}

/// Validate a backend namespace: non-empty, `a-z0-9_-` only, 1..=63 chars.
/// Deterministic and independent of locale (spec 0020 §6).
pub fn validate_namespace(ns: &str) -> Result<(), NamespaceError> {
    let len = ns.len();
    if !(1..=63).contains(&len) {
        return Err(NamespaceError::Invalid(ns.into()));
    }
    if !ns
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
    {
        return Err(NamespaceError::Invalid(ns.into()));
    }
    Ok(())
}

/// Validate a local catalog name: same charset as a namespace, plus it MUST NOT
/// contain `.` (the namespace separator) so `<ns>.<name>` is unambiguous.
pub fn validate_local_name(name: &str) -> Result<(), NamespaceError> {
    validate_namespace(name)?;
    // validate_namespace already forbids '.', but be explicit and defensive.
    if name.contains('.') {
        return Err(NamespaceError::Invalid(name.into()));
    }
    Ok(())
}

/// Canonical JSON for hashing: object keys sorted ascending, no whitespace.
/// Deterministic regardless of serde_json map features. The pinned-hash and
/// call-time integrity contracts (spec 0020 §7, §19) depend on byte-stable
/// hashes, so this is load-bearing.
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).unwrap_or_else(|_| String::from("\"\"")));
                out.push(':');
                write_canonical(&map[*k], out);
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(x, out);
            }
            out.push(']');
        }
        _ => out.push_str(&serde_json::to_string(v).unwrap_or_else(|_| "null".into())),
    }
}

/// SHA-256 over the canonical JSON of a wire definition, as `sha256:<hex>`
/// (matches `ProvenanceRecord.source_hash` formatting).
pub fn definition_hash(wire_definition: &Value) -> String {
    crate::provenance::hash::content_hash(canonical_json(wire_definition).as_bytes())
}

/// A catalog entry that was dropped during projection, with a deterministic
/// reason (spec 0020 §6/§19: dropped/filtered entries recorded deterministically).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DroppedEntry {
    pub server_id: String,
    pub namespace: String,
    pub name: String,
    pub reason: String,
}

/// Build the wire definition JSON for a declared entry. Canonical fields so the
/// `definition_hash` is stable across reloads.
pub fn wire_definition_for(namespace: &str, entry: &DeclaredCatalogEntry) -> Value {
    let namespaced = format!("{namespace}.{}", entry.name);
    serde_json::json!({
        "name": namespaced,
        "local_name": entry.name,
        "description": entry.description,
        "input_schema": entry.input_schema,
    })
}

/// Project all declared catalog entries across servers into runtime
/// `GatewayCatalogEntry`s. Returns `(catalog, dropped)`. Namespace uniqueness
/// is assumed already checked by `GatewayConfig::validate_namespaces`; this
/// function drops entries with invalid local names rather than failing the whole
/// projection (one bad entry must not poison the catalog).
pub fn project_catalog(servers: &[MCPServer]) -> (Vec<GatewayCatalogEntry>, Vec<DroppedEntry>) {
    let mut catalog = Vec::new();
    let mut dropped = Vec::new();
    for server in servers {
        for entry in &server.catalog {
            if let Err(_) = validate_local_name(&entry.name) {
                dropped.push(DroppedEntry {
                    server_id: server.id.clone(),
                    namespace: server.namespace.clone(),
                    name: entry.name.clone(),
                    reason: "invalid_local_name".into(),
                });
                continue;
            }
            let namespaced_name = format!("{}.{}", server.namespace, entry.name);
            let wire = wire_definition_for(&server.namespace, entry);
            let definition_hash = definition_hash(&wire);
            let mut policy_tags = server.policy_tags.clone();
            policy_tags.extend(entry.policy_tags.iter().cloned());
            catalog.push(GatewayCatalogEntry {
                kind: entry.kind,
                namespaced_name: namespaced_name.clone(),
                backend_id: server.id.clone(),
                wire_definition: wire,
                definition_hash,
                source_hash: server.source_hash.clone(),
                policy_tags,
                pinned_hash: server.pinned_hashes.get(&namespaced_name).cloned(),
                input_schema: entry.input_schema.clone(),
            });
        }
    }
    catalog.sort_by(|a, b| a.namespaced_name.cmp(&b.namespaced_name));
    dropped.sort_by(|a, b| {
        (a.namespace.as_str(), a.name.as_str(), a.reason.as_str()).cmp(&(
            b.namespace.as_str(),
            b.name.as_str(),
            b.reason.as_str(),
        ))
    });
    (catalog, dropped)
}

/// Merge declared catalog entries with live-discovered entries (spec 0020 §7
/// "Live Catalog Discovery"). Declared entries win any conflict on
/// `namespaced_name`; a conflicting or duplicate live entry is dropped and
/// reported. Catalog construction has ONE owner — this function — so the
/// declared/live boundary never produces two sources of truth.
pub fn merge_live(
    declared: Vec<GatewayCatalogEntry>,
    live: Vec<GatewayCatalogEntry>,
) -> (Vec<GatewayCatalogEntry>, Vec<DroppedEntry>) {
    use std::collections::HashSet;
    let declared_names: HashSet<String> =
        declared.iter().map(|e| e.namespaced_name.clone()).collect();
    let mut seen_live: HashSet<String> = HashSet::new();
    let mut merged = declared;
    let mut dropped = Vec::new();
    for entry in live {
        if declared_names.contains(&entry.namespaced_name) {
            dropped.push(dropped_from_entry(&entry, "conflicts_with_declared"));
            continue;
        }
        if !seen_live.insert(entry.namespaced_name.clone()) {
            dropped.push(dropped_from_entry(&entry, "duplicate_live"));
            continue;
        }
        merged.push(entry);
    }
    merged.sort_by(|a, b| a.namespaced_name.cmp(&b.namespaced_name));
    dropped.sort_by(|a, b| {
        (a.namespace.as_str(), a.name.as_str(), a.reason.as_str()).cmp(&(
            b.namespace.as_str(),
            b.name.as_str(),
            b.reason.as_str(),
        ))
    });
    (merged, dropped)
}

fn dropped_from_entry(entry: &GatewayCatalogEntry, reason: &str) -> DroppedEntry {
    let (namespace, name) = entry
        .namespaced_name
        .split_once('.')
        .unwrap_or((&entry.namespaced_name, ""));
    DroppedEntry {
        server_id: entry.backend_id.clone(),
        namespace: namespace.to_string(),
        name: name.to_string(),
        reason: reason.to_string(),
    }
}

/// Compact discovery item (spec 0020 §11): name + kind + short description only.
/// Full schemas stay out of model context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactCatalogItem {
    pub kind: CatalogKind,
    pub namespaced_name: String,
    #[serde(default)]
    pub description: String,
}

impl CompactCatalogItem {
    pub fn from_entry(entry: &GatewayCatalogEntry) -> Self {
        let description = entry
            .wire_definition
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();
        Self {
            kind: entry.kind,
            namespaced_name: entry.namespaced_name.clone(),
            description,
        }
    }
}

/// Default compact-discovery cap (spec 0020 §6: small catalog limit).
pub const DEFAULT_DISCOVERY_CAP: usize = 64;

/// List the catalog as compact items, capped at `cap`. Returns the items plus
/// the count of entries beyond the cap (deterministic dropped-entries logging).
pub fn list_catalog<'a>(
    catalog: &'a [GatewayCatalogEntry],
    cap: usize,
) -> (Vec<CompactCatalogItem>, usize) {
    let cap = if cap == 0 { DEFAULT_DISCOVERY_CAP } else { cap };
    let total = catalog.len();
    let items: Vec<CompactCatalogItem> = catalog
        .iter()
        .take(cap)
        .map(CompactCatalogItem::from_entry)
        .collect();
    let dropped = total.saturating_sub(items.len());
    (items, dropped)
}

/// Search namespaced names + descriptions (case-insensitive substring), capped.
pub fn search_catalog<'a>(
    catalog: &'a [GatewayCatalogEntry],
    query: &str,
    cap: usize,
) -> (Vec<CompactCatalogItem>, usize) {
    let cap = if cap == 0 { DEFAULT_DISCOVERY_CAP } else { cap };
    let q = query.to_ascii_lowercase();
    let matches_desc = |e: &&GatewayCatalogEntry| {
        e.namespaced_name.to_ascii_lowercase().contains(&q)
            || e.wire_definition
                .get("description")
                .and_then(|d| d.as_str())
                .map(|d| d.to_ascii_lowercase().contains(&q))
                .unwrap_or(false)
    };
    let total_matched = catalog.iter().filter(matches_desc).count();
    let mut items: Vec<CompactCatalogItem> = catalog
        .iter()
        .filter(matches_desc)
        .take(cap)
        .map(CompactCatalogItem::from_entry)
        .collect();
    items.sort_by(|a, b| a.namespaced_name.cmp(&b.namespaced_name));
    let dropped = total_matched.saturating_sub(items.len());
    (items, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_validation_rules() {
        assert!(validate_namespace("fs").is_ok());
        assert!(validate_namespace("a_b-c9").is_ok());
        assert!(validate_namespace("").is_err());
        assert!(validate_namespace("FS").is_err()); // uppercase forbidden
        assert!(validate_namespace("has.dot").is_err());
        assert!(validate_namespace(&"x".repeat(64)).is_err());
    }

    #[test]
    fn canonical_json_sorts_keys_and_is_stable() {
        let a = serde_json::json!({"b": 1, "a": {"y": 2, "x": [3, 4]}});
        let b = serde_json::json!({"a": {"x": [3, 4], "y": 2}, "b": 1});
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(canonical_json(&a), r#"{"a":{"x":[3,4],"y":2},"b":1}"#);
    }
}
