//! Secret redaction (spec 0005/0009). Walks a JSON payload and redacts:
//! (a) any value whose key contains a `key_substring` (secret/token/...), and
//! (b) any string value matching a `value_pattern` regex. Mirrors agent_hooks.redact.

use serde_json::Value;

/// Redaction policy loaded from `.agent-hooks/config.yaml` `redaction:` section.
#[derive(Debug, Clone, Default)]
pub struct RedactionConfig {
    pub key_substrings: Vec<String>,
    pub value_patterns: Vec<String>,
}

/// Recursively redact a JSON value in place.
pub fn redact_value(value: &mut Value, cfg: &RedactionConfig) {
    match value {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                let key_match = cfg
                    .key_substrings
                    .iter()
                    .any(|sub| key.to_lowercase().contains(&sub.to_lowercase()));
                if key_match {
                    if let Some(v) = map.get_mut(&key) {
                        *v = Value::String("[REDACTED]".into());
                    }
                    continue;
                }
                if let Some(v) = map.get_mut(&key) {
                    redact_one(v, cfg);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_value(item, cfg);
            }
        }
        _ => {}
    }
}

fn redact_one(v: &mut Value, cfg: &RedactionConfig) {
    match v {
        Value::String(s) => {
            for pat in &cfg.value_patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    *s = re.replace_all(s, "[REDACTED]").into_owned();
                }
            }
        }
        Value::Object(_) | Value::Array(_) => redact_value(v, cfg),
        _ => {}
    }
}
