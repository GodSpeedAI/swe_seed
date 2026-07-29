//! JSON-RPC request validation + bounds (spec 0020 §8). The gateway accepts
//! only well-formed JSON-RPC 2.0 requests under a size cap and a nesting-depth
//! cap. Anything malformed, oversized, or too deeply nested fails closed as
//! `InvalidRequest` BEFORE it reaches policy/governance/forward — a bad input
//! never consumes a budget counter or touches a backend.

use serde_json::Value;

use super::{GatewayError, ALLOWED_METHODS};

/// Hard cap on a single JSON-RPC request body (1 MiB). Generous for tool
/// arguments, small enough to reject unbounded reads (spec 0020 §8).
pub const REQUEST_SIZE_CAP: usize = 1024 * 1024;

/// Maximum nesting depth of a JSON-RPC request (spec 0020 §8). 64 is well
/// above any legitimate tool argument shape and rejects stack-exhaustion
/// attempts.
pub const JSON_DEPTH_CAP: usize = 64;

/// A validated JSON-RPC 2.0 request.
#[derive(Debug, Clone)]
pub struct JsonRpcRequest {
    pub id: Value,
    pub method: String,
    pub params: Value,
}

/// Validate a raw request body: size cap, JSON parse, depth cap, JSON-RPC
/// shape (jsonrpc=="2.0", method is a non-empty string, id is string|number|null).
/// `params` defaults to `{}` when absent. Method allow-listing is deferred to
/// the router (the server boundary checks `ALLOWED_METHODS`).
pub fn validate_request(body: &[u8]) -> Result<JsonRpcRequest, GatewayError> {
    if body.len() > REQUEST_SIZE_CAP {
        return Err(GatewayError::InvalidRequest {
            reason: format!("request body {} bytes exceeds cap {}", body.len(), REQUEST_SIZE_CAP),
        });
    }
    let value: Value = serde_json::from_slice(body).map_err(|e| GatewayError::InvalidRequest {
        reason: format!("malformed JSON: {e}"),
    })?;
    if json_depth(&value) > JSON_DEPTH_CAP {
        return Err(GatewayError::InvalidRequest {
            reason: format!("json depth exceeds cap {JSON_DEPTH_CAP}"),
        });
    }
    parse_jsonrpc(&value)
}

/// Validate an already-parsed JSON value as a JSON-RPC 2.0 request.
pub fn parse_jsonrpc(value: &Value) -> Result<JsonRpcRequest, GatewayError> {
    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            return Err(GatewayError::InvalidRequest {
                reason: "request must be a JSON object".into(),
            });
        }
    };
    let v = match obj.get("jsonrpc") {
        Some(v) => v,
        None => {
            return Err(GatewayError::InvalidRequest {
                reason: "missing jsonrpc version".into(),
            })
        }
    };
    if v.as_str() != Some("2.0") {
        return Err(GatewayError::InvalidRequest {
            reason: "jsonrpc must be \"2.0\"".into(),
        });
    }
    let method = match obj.get("method").and_then(|m| m.as_str()) {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => {
            return Err(GatewayError::InvalidRequest {
                reason: "method must be a non-empty string".into(),
            })
        }
    };
    let id = match obj.get("id") {
        Some(id) if id.is_string() || id.is_number() || id.is_null() => id.clone(),
        Some(_) => {
            return Err(GatewayError::InvalidRequest {
                reason: "id must be string, number, or null".into(),
            })
        }
        None => Value::Null,
    };
    let params = obj.get("params").cloned().unwrap_or(Value::Object(Default::default()));
    Ok(JsonRpcRequest { id, method, params })
}

/// True iff `method` is one of the gateway's allowed JSON-RPC methods.
pub fn is_allowed_method(method: &str) -> bool {
    ALLOWED_METHODS.contains(&method)
}

/// Recursive nesting depth of a JSON value (a scalar = 0, an object/array
/// containing only scalars = 1, ...).
pub fn json_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => 1 + map.values().map(json_depth).max().unwrap_or(0),
        Value::Array(items) => 1 + items.iter().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid(body: &str) -> JsonRpcRequest {
        validate_request(body.as_bytes()).expect("should validate")
    }

    #[test]
    fn validates_minimal_request() {
        let r = valid(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
        assert_eq!(r.method, "tools/list");
        assert_eq!(r.id, json!(1));
        assert_eq!(r.params, json!({}));
    }

    #[test]
    fn params_default_to_object_when_absent() {
        let r = valid(r#"{"jsonrpc":"2.0","id":"a","method":"tools/list"}"#);
        assert_eq!(r.params, json!({}));
    }

    #[test]
    fn params_pass_through_when_present() {
        let r = valid(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fs.read","args":{"path":"/x"}}}"#);
        assert_eq!(r.params["name"], json!("fs.read"));
    }

    #[test]
    fn rejects_oversized_body() {
        let mut big = String::from(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":""#);
        big.push_str(&"x".repeat(REQUEST_SIZE_CAP + 10));
        big.push_str(r#""}"#);
        let err = validate_request(big.as_bytes()).unwrap_err();
        assert_eq!(err.reason_code(), "invalid_request");
        assert!(err.to_string().contains("exceeds cap"));
    }

    #[test]
    fn rejects_malformed_json() {
        let err = validate_request(b"{not json").unwrap_err();
        assert_eq!(err.reason_code(), "invalid_request");
        assert!(err.to_string().contains("malformed JSON"));
    }

    #[test]
    fn rejects_excessive_depth() {
        // Build a deeply nested object: {"a":{"a":...}}
        let mut s = String::new();
        for _ in 0..(JSON_DEPTH_CAP + 5) {
            s.push_str(r#"{"a":"#);
        }
        s.push_str("1");
        for _ in 0..(JSON_DEPTH_CAP + 5) {
            s.push('}');
        }
        let err = validate_request(s.as_bytes()).unwrap_err();
        assert_eq!(err.reason_code(), "invalid_request");
        assert!(err.to_string().contains("depth"));
    }

    #[test]
    fn rejects_missing_jsonrpc_version() {
        let err = validate_request(br#"{"id":1,"method":"tools/list"}"#).unwrap_err();
        assert!(err.to_string().contains("jsonrpc"));
    }

    #[test]
    fn rejects_wrong_version() {
        let err = validate_request(br#"{"jsonrpc":"1.0","id":1,"method":"tools/list"}"#).unwrap_err();
        assert!(err.to_string().contains("2.0"));
    }

    #[test]
    fn rejects_non_string_method() {
        let err = validate_request(br#"{"jsonrpc":"2.0","id":1,"method":5}"#).unwrap_err();
        assert!(err.to_string().contains("method"));
    }

    #[test]
    fn rejects_empty_method() {
        let err = validate_request(br#"{"jsonrpc":"2.0","id":1,"method":""}"#).unwrap_err();
        assert!(err.to_string().contains("method"));
    }

    #[test]
    fn rejects_non_object_request() {
        let err = validate_request(b"[1,2,3]").unwrap_err();
        assert!(err.to_string().contains("JSON object"));
    }

    #[test]
    fn rejects_bad_id_type() {
        let err = validate_request(br#"{"jsonrpc":"2.0","id":true,"method":"tools/list"}"#).unwrap_err();
        assert!(err.to_string().contains("id"));
    }

    #[test]
    fn null_id_is_accepted() {
        let r = valid(r#"{"jsonrpc":"2.0","id":null,"method":"tools/list"}"#);
        assert_eq!(r.id, Value::Null);
    }

    #[test]
    fn is_allowed_method_recognizes_known_methods() {
        assert!(is_allowed_method("tools/list"));
        assert!(is_allowed_method("tools/call"));
        assert!(!is_allowed_method("bogus/method"));
    }

    #[test]
    fn json_depth_scalars_zero_object_one() {
        assert_eq!(json_depth(&json!("x")), 0);
        assert_eq!(json_depth(&json!({"a":1})), 1);
        assert_eq!(json_depth(&json!({"a":{"b":1}})), 2);
        assert_eq!(json_depth(&json!([1,[2,[3]]])), 3);
    }
}
