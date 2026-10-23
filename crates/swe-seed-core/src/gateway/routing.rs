//! Backend routing (spec 0020 §7, §8, §18). Forwards JSON-RPC to the owning
//! backend over stdio or HTTP and returns a typed `GatewayError` on any
//! failure. Invariants:
//!   - JSON-RPC request id is preserved (a backend that returns a different id
//!     is treated as malformed → `BackendError`);
//!   - a namespaced call routes ONLY to its owning backend; a failure there is
//!     `BackendUnavailable`, NEVER a reroute to another backend (spec 0020 §7);
//!   - bounded timeouts; a timeout or refused connection opens the path to
//!     breaker state (stage 7) and returns `BackendUnavailable` here.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;

use super::GatewayError;

/// Initial JSON-RPC method set the gateway forwards (spec 0020 stage 5).
pub const ALLOWED_METHODS: &[&str] = &[
    "tools/list",
    "tools/call",
    "resources/list",
    "resources/read",
    "prompts/list",
    "prompts/get",
];

/// A backend transport. Implementations: `StdioTransport`, `HttpTransport`,
/// and `MockTransport` (tests + stage 6/7 wiring). Returns the FULL JSON-RPC
/// response object; the router extracts `result`/`error` and checks id.
pub trait BackendTransport: Send + Sync {
    fn send_request(&self, request: Value, timeout: Duration) -> Result<Value, GatewayError>;
}

/// The router owns one transport per backend id. It resolves a namespaced call
/// to exactly one backend and never reroutes.
pub struct Router {
    transports: HashMap<String, Box<dyn BackendTransport>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            transports: HashMap::new(),
        }
    }
    pub fn with(backend_id: &str, transport: Box<dyn BackendTransport>) -> Self {
        let mut r = Self::new();
        r.register(backend_id, transport);
        r
    }
    pub fn register(&mut self, backend_id: &str, transport: Box<dyn BackendTransport>) {
        self.transports.insert(backend_id.into(), transport);
    }
    pub fn has(&self, backend_id: &str) -> bool {
        self.transports.contains_key(backend_id)
    }

    /// Forward a JSON-RPC call to `backend_id`. `request_id` is the inbound id
    /// preserved verbatim. Validates the method allowlist, checks the backend
    /// echoed our id, and maps JSON-RPC error objects to `BackendError`.
    pub fn invoke(
        &self,
        backend_id: &str,
        request_id: &Value,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, GatewayError> {
        if !ALLOWED_METHODS.contains(&method) {
            return Err(GatewayError::InvalidRequest {
                reason: format!("unsupported method '{method}'"),
            });
        }
        // Namespaced no-reroute: resolve exactly one backend. A missing/down
        // backend is BackendUnavailable — we never try another.
        let transport = self.transports.get(backend_id).ok_or_else(|| {
            GatewayError::BackendUnavailable {
                backend: backend_id.into(),
            }
        })?;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        });
        let resp = transport.send_request(request, timeout)?;
        // id preservation (spec 0020 §5/§19)
        if resp.get("id") != Some(request_id) {
            return Err(GatewayError::BackendError {
                backend: backend_id.into(),
                reason: "json-rpc id not preserved by backend".into(),
            });
        }
        if let Some(err) = resp.get("error") {
            let message = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("backend json-rpc error")
                .to_string();
            return Err(GatewayError::BackendError {
                backend: backend_id.into(),
                reason: message,
            });
        }
        Ok(resp.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

// ── stdio transport ──────────────────────────────────────────────────────────

/// stdio JSON-RPC transport: spawns the backend process, writes one request
/// line, reads one response line, bounded by `timeout`.
pub struct StdioTransport {
    pub program: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

impl StdioTransport {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            env: HashMap::new(),
        }
    }
}

impl BackendTransport for StdioTransport {
    fn send_request(&self, request: Value, timeout: Duration) -> Result<Value, GatewayError> {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args);
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = cmd.spawn().map_err(io_to_unavailable(&self.program))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| GatewayError::BackendUnavailable {
                backend: self.program.clone(),
            })?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GatewayError::BackendUnavailable {
                backend: self.program.clone(),
            })?;
        let body = serde_json::to_string(&request)
            .map_err(|e| GatewayError::BackendError {
                backend: self.program.clone(),
                reason: format!("serialize: {e}"),
            })?;
        let program = self.program.clone();
        let (tx, rx) = std::sync::mpsc::channel::<Result<Value, GatewayError>>();
        std::thread::spawn(move || {
            let res = (|| -> Result<Value, GatewayError> {
                if stdin
                    .write_all(format!("{body}\n").as_bytes())
                    .is_err()
                {
                    return Err(GatewayError::BackendUnavailable {
                        backend: program.clone(),
                    });
                }
                drop(stdin);
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                reader
                    .read_line(&mut line)
                    .map_err(io_to_unavailable(&program))?;
                let value: Value = serde_json::from_str(line.trim())
                    .map_err(|e| GatewayError::BackendError {
                        backend: program.clone(),
                        reason: format!("malformed response: {e}"),
                    })?;
                Ok(value)
            })();
            let _ = tx.send(res);
        });
        let result = rx.recv_timeout(timeout);
        let _ = child.kill(); // always reap the child
        let _ = child.wait();
        match result {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(GatewayError::BackendUnavailable {
                backend: self.program.clone(),
            }),
        }
    }
}

// ── http transport (loopback plain HTTP; HTTPS/TLS deferred — spec §15) ──────

/// Minimal HTTP/1.1 JSON-RPC POST transport for loopback MCP backends. Non-
/// loopback requires TLS (spec 0020 §15), which is out of scope for v0.1.
pub struct HttpTransport {
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl HttpTransport {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        // ponytail: hand-rolled URL split for http://host[:port][/path]; no new
        // crate. HTTPS URLs are rejected (TLS is a stage-7+ concern).
        let raw = url.strip_prefix("http://").ok_or_else(|| GatewayError::InvalidRequest {
            reason: format!("http transport needs http:// url (got '{url}')"),
        })?;
        let (authority, path) = raw.split_once('/').unwrap_or((raw, ""));
        let (host, port) = match authority.split_once(':') {
            Some((h, p)) => (h.to_string(), p.parse::<u16>().map_err(|_| GatewayError::InvalidRequest {
                reason: format!("bad port in {url}"),
            })?),
            None => (authority.to_string(), 80),
        };
        Ok(Self {
            host,
            port,
            path: if path.is_empty() { "/".into() } else { format!("/{path}") },
        })
    }
}

impl BackendTransport for HttpTransport {
    fn send_request(&self, request: Value, timeout: Duration) -> Result<Value, GatewayError> {
        let body = serde_json::to_string(&request).map_err(|e| GatewayError::BackendError {
            backend: format!("http://{}:{}", self.host, self.port),
            reason: format!("serialize: {e}"),
        })?;
        // ponytail: connect_timeout bounds the connect (a filtered port would
        // otherwise hang for the OS default ~60s); no async runtime needed.
        let addr = (self.host.as_str(), self.port)
            .to_socket_addrs()
            .map_err(io_to_unavailable(&self.host))?
            .next()
            .ok_or_else(|| GatewayError::BackendUnavailable {
                backend: self.host.clone(),
            })?;
        let mut stream = TcpStream::connect_timeout(&addr, timeout)
            .map_err(io_to_unavailable(&self.host))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(io_to_unavailable(&self.host))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(io_to_unavailable(&self.host))?;
        let req = format!(
            "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
            path = self.path,
            host = self.host,
            len = body.len()
        );
        stream
            .write_all(req.as_bytes())
            .map_err(io_to_unavailable(&self.host))?;
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .map_err(io_to_unavailable(&self.host))?;
        let text = String::from_utf8_lossy(&bytes);
        let body_start = text
            .find("\r\n\r\n")
            .ok_or_else(|| GatewayError::BackendError {
                backend: self.host.clone(),
                reason: "malformed http response (no header/body split)".into(),
            })?
            + 4;
        // Connection: close ⇒ read-to-end captured the full body; if a
        // Content-Length was sent we trust close-delimited framing here.
        let body_text = &text[body_start..];
        let value: Value = serde_json::from_str(body_text.trim()).map_err(|e| {
            GatewayError::BackendError {
                backend: self.host.clone(),
                reason: format!("malformed json body: {e}"),
            }
        })?;
        Ok(value)
    }
}

// ── mock transport (tests + stage 6/7 wiring) ────────────────────────────────

/// Test/fake transport. `response` is a canned JSON-RPC response string;
/// `delay` simulates latency (exceeds timeout → BackendUnavailable);
/// `drop_request` makes send_request always fail as unavailable.
pub struct MockTransport {
    pub response: String,
    pub delay: Option<Duration>,
    pub drop_request: bool,
    /// Counts calls so no-reroute tests can assert a backend was NOT called.
    pub calls: std::sync::atomic::AtomicU64,
}

impl MockTransport {
    pub fn responding(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            delay: None,
            drop_request: false,
            calls: 0.into(),
        }
    }
    pub fn call_count(&self) -> u64 {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl BackendTransport for MockTransport {
    fn send_request(&self, _request: Value, timeout: Duration) -> Result<Value, GatewayError> {
        self.calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.drop_request {
            return Err(GatewayError::BackendUnavailable {
                backend: "mock".into(),
            });
        }
        if let Some(d) = self.delay {
            std::thread::sleep(d.min(timeout));
            if d > timeout {
                return Err(GatewayError::BackendUnavailable {
                    backend: "mock".into(),
                });
            }
        }
        serde_json::from_str(&self.response).map_err(|e| GatewayError::BackendError {
            backend: "mock".into(),
            reason: format!("malformed mock response: {e}"),
        })
    }
}

fn io_to_unavailable(backend: &str) -> impl Fn(std::io::Error) -> GatewayError + '_ {
    move |_e: std::io::Error| GatewayError::BackendUnavailable {
        backend: backend.into(),
    }
    // NOTE: the io::ErrorKind is collapsed to BackendUnavailable; stage 7's
    // breaker uses the outcome (Err here) rather than the kind.
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn forwards_and_preserves_id_extracting_result() {
        let mock = MockTransport::responding(r#"{"jsonrpc":"2.0","id":7,"result":{"tools":[]}}"#);
        let router = Router::with("fs", Box::new(mock));
        let id = json!(7);
        let result = router
            .invoke("fs", &id, "tools/list", json!({}), Duration::from_secs(1))
            .unwrap();
        assert_eq!(result, json!({"tools":[]}));
    }

    #[test]
    fn id_mismatch_is_backend_error() {
        let mock = MockTransport::responding(r#"{"jsonrpc":"2.0","id":999,"result":{}}"#);
        let router = Router::with("fs", Box::new(mock));
        let err = router
            .invoke("fs", &json!(1), "tools/list", json!({}), Duration::from_secs(1))
            .unwrap_err();
        assert_eq!(err.reason_code(), "backend_error");
    }

    #[test]
    fn jsonrpc_error_object_maps_to_backend_error() {
        let mock =
            MockTransport::responding(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"method not found"}}"#);
        let router = Router::with("fs", Box::new(mock));
        let err = router
            .invoke("fs", &json!(1), "tools/list", json!({}), Duration::from_secs(1))
            .unwrap_err();
        match err {
            GatewayError::BackendError { reason, .. } => assert_eq!(reason, "method not found"),
            other => panic!("expected BackendError, got {other:?}"),
        }
    }

    #[test]
    fn unsupported_method_is_invalid_request() {
        let router = Router::with("fs", Box::new(MockTransport::responding("{}")));
        let err = router
            .invoke("fs", &json!(1), "evil/method", json!({}), Duration::from_secs(1))
            .unwrap_err();
        assert_eq!(err.reason_code(), "invalid_request");
    }

    #[test]
    fn missing_backend_is_unavailable_never_reroutes() {
        // Two backends; call to A whose transport is absent. B must NOT be tried.
        let b = MockTransport::responding(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#);
        let router = Router::with("b", Box::new(b));
        let err = router
            .invoke("a", &json!(1), "tools/list", json!({}), Duration::from_secs(1))
            .unwrap_err();
        assert_eq!(err.reason_code(), "backend_unavailable");
    }

    #[test]
    fn down_backend_is_unavailable() {
        // A backend whose transport drops the request returns unavailable.
        // The router resolves ONLY the named backend (structurally no reroute),
        // so a second registered backend is never consulted.
        let a = MockTransport {
            drop_request: true,
            ..MockTransport::responding("")
        };
        let b_calls = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let b = TallyingTransport {
            inner: MockTransport::responding(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#),
            tally: b_calls.clone(),
        };
        let mut router = Router::new();
        router.register("a", Box::new(a));
        router.register("b", Box::new(b));
        let err = router
            .invoke("a", &json!(1), "tools/list", json!({}), Duration::from_millis(100))
            .unwrap_err();
        assert_eq!(err.reason_code(), "backend_unavailable");
        // backend "b" was never called — no reroute.
        assert_eq!(b_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn timeout_is_unavailable() {
        let mock = MockTransport {
            delay: Some(Duration::from_millis(200)),
            ..MockTransport::responding(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#)
        };
        let router = Router::with("fs", Box::new(mock));
        let err = router
            .invoke("fs", &json!(1), "tools/list", json!({}), Duration::from_millis(20))
            .unwrap_err();
        assert_eq!(err.reason_code(), "backend_unavailable");
    }

    #[test]
    fn malformed_response_is_backend_error() {
        let mock = MockTransport::responding("not json at all");
        let router = Router::with("fs", Box::new(mock));
        let err = router
            .invoke("fs", &json!(1), "tools/list", json!({}), Duration::from_secs(1))
            .unwrap_err();
        assert_eq!(err.reason_code(), "backend_error");
    }

    pub struct TallyingTransport {
        pub inner: MockTransport,
        pub tally: std::sync::Arc<std::sync::atomic::AtomicU64>,
    }
    impl BackendTransport for TallyingTransport {
        fn send_request(&self, r: Value, t: Duration) -> Result<Value, GatewayError> {
            self.tally.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.inner.send_request(r, t)
        }
    }
}
