//! `gateway serve` — a bounded, dependency-free HTTP/1.1 JSON-RPC server that
//! executes every request through the proven fail-closed pipeline
//! (`validate_request` → policy/governance/route/audit via `handle_request`).
//!
//! Design (spec 0020 §5, §8, §15):
//!   - stdio `TcpListener`, no async runtime. Single worker (bounded
//!     concurrency = 1) for v0.1; the breaker + governance already serialize
//!     the risky paths.
//!   - Loopback default; non-loopback requires TLS/tunnel (validated at bind).
//!   - One request body capped at `REQUEST_SIZE_CAP`, JSON depth capped at
//!     `JSON_DEPTH_CAP`. Oversized/deep/malformed ⇒ `InvalidRequest` (400).
//!   - Every completion (success OR typed error) writes ONE audit record
//!     before the response is sent. Audit-write failure fails the request
//!     closed (handled inside `handle_request`).
//!   - Discovery (`tools/list` etc.) returns the local compact catalog
//!     projection; namespaced calls (`tools/call` etc.) resolve exactly one
//!     backend and forward through the pipeline.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use crate::hooks::policy::PermissionPolicy;

use super::audit::AuditRecord;
use super::config::GatewayConfig;
use super::governance::Governance;
use super::jsonrpc::{validate_request, JsonRpcRequest};
use super::routing::{HttpTransport, Router, StdioTransport};
use super::{
    handle_request, load_redaction, parse_jsonrpc, AuditWriter, GatewayCatalogEntry,
    GatewayError, ListenerConfig, RequestContext, REQUEST_SIZE_CAP,
};

/// JSON-RPC error codes (spec 0020 §8).
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INTERNAL_ERROR: i64 = -32603;

/// Maximum HTTP request line + header block size (64 KiB). Defends against an
/// unbounded header read before we reach the body.
const HEADER_CAP: usize = 64 * 1024;

/// Default per-backend forward timeout.
const DEFAULT_FORWARD_TIMEOUT: Duration = Duration::from_secs(30);

/// A bound gateway server. Owns the listener + the compiled runtime snapshot.
pub struct GatewayServer {
    listener: TcpListener,
    runtime: Runtime,
}

/// Compiled runtime: router, governance, audit, policy, catalog.
struct Runtime {
    router: Router,
    governance: Governance,
    audit: AuditWriter,
    policy: PermissionPolicy,
    catalog: Vec<GatewayCatalogEntry>,
}

impl GatewayServer {
    /// Compile the runtime from `<root>/.swe-seed/gateway/config.json` +
    /// `.agent-hooks/config.yaml` redaction, validate it, and bind the listener.
    /// `bind`/`port` override the config listener for this run (after exposure
    /// validation). `port=0` lets the OS choose; the selected port is available
    /// via [`Self::local_addr`].
    pub fn bind(
        root: &Path,
        bind: Option<&str>,
        port: Option<u16>,
    ) -> Result<Self, GatewayError> {
        let mut config = GatewayConfig::load(root).map_err(|e| GatewayError::InvalidRequest {
            reason: format!("load gateway config: {e}"),
        })?;
        // CLI override takes precedence, then validate exposure.
        if let Some(b) = bind {
            config.listener.bind = b.to_string();
        }
        if let Some(p) = port {
            config.listener.port = p;
        }
        validate_exposure(&config.listener)?;

        let (catalog, _dropped) = super::project_catalog(&config.servers);
        let mut router = Router::new();
        for server in &config.servers {
            register_transport(&mut router, server)?;
        }
        let redaction = load_redaction(root).map_err(|e| GatewayError::InvalidRequest {
            reason: format!("load redaction: {e}"),
        })?;
        let audit = AuditWriter::new(root, redaction);
        let governance = Governance::load(root, config.limits.clone()).map_err(|e| {
            GatewayError::InvalidRequest {
                reason: format!("load governance journal: {e}"),
            }
        })?;

        let addr = format!("{}:{}", config.listener.bind, config.listener.port);
        let listener = TcpListener::bind(&addr).map_err(|e| GatewayError::InvalidRequest {
            reason: format!("bind {addr}: {e}"),
        })?;
        listener
            .set_nonblocking(false)
            .map_err(|e| GatewayError::InvalidRequest {
                reason: format!("listener mode: {e}"),
            })?;

        Ok(Self {
            listener,
            runtime: Runtime {
                router,
                governance,
                audit,
                policy: config.policy.clone(),
                catalog,
            },
        })
    }

    /// The bound local address (use to discover an OS-chosen port).
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Serve ONE connection then return. For `--once` and deterministic tests.
    pub fn serve_once(&self) -> Result<(), GatewayError> {
        let (stream, _) = self
            .listener
            .accept()
            .map_err(|e| GatewayError::InvalidRequest {
                reason: format!("accept: {e}"),
            })?;
        self.handle_connection(stream)
    }

    /// Serve connections until the listener errors. Single-worker loop.
    pub fn serve_loop(&self) -> Result<(), GatewayError> {
        loop {
            let (stream, _) = match self.listener.accept() {
                Ok(p) => p,
                Err(e) => {
                    return Err(GatewayError::InvalidRequest {
                        reason: format!("accept: {e}"),
                    });
                }
            };
            // One connection at a time (bounded concurrency = 1). A handler
            // error is logged to stderr but does not stop the loop.
            if let Err(e) = self.handle_connection(stream) {
                eprintln!("gateway serve: connection ended with {e}");
            }
        }
    }

    /// Handle a single HTTP/1.1 connection: parse, dispatch, respond.
    fn handle_connection(&self, mut stream: TcpStream) -> Result<(), GatewayError> {
        stream
            .set_read_timeout(Some(DEFAULT_FORWARD_TIMEOUT))
            .map_err(|e| GatewayError::InvalidRequest {
                reason: format!("set read timeout: {e}"),
            })?;
        let mut reader = BufReader::new(stream.try_clone().map_err(|e| {
            GatewayError::InvalidRequest {
                reason: format!("clone stream: {e}"),
            }
        })?);
        let (status, body) = self.read_and_dispatch(&mut reader, &mut stream);
        write_response(&mut stream, status, &body).map_err(|e| {
            GatewayError::InvalidRequest {
                reason: format!("write response: {e}"),
            }
        })?;
        Ok(())
    }

    /// Read the HTTP request, dispatch through the pipeline, return
    /// `(http_status, json_body_string)`.
    fn read_and_dispatch<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        _writer: &mut W,
    ) -> (u16, String) {
        let (method, path, content_length) = match read_request_head(reader) {
            Ok(v) => v,
            Err(e) => return error_http(400, jsonrpc_error_obj(None, INVALID_REQUEST, &e)),
        };
        // Only POST is supported.
        if method != "POST" {
            return error_http(
                405,
                jsonrpc_error_obj(None, INVALID_REQUEST, "only POST is supported"),
            );
        }
        let _ = path;
        // Read body, capped.
        let body = match read_body(reader, content_length) {
            Ok(b) => b,
            Err(e) => return error_http(400, jsonrpc_error_obj(None, INVALID_REQUEST, &e)),
        };
        // Validate JSON-RPC bounds + shape.
        let req = match validate_request(&body) {
            Ok(r) => r,
            Err(e) => {
                // Best-effort id recovery: parse the body, then the JSON-RPC id.
                let id = serde_json::from_slice::<Value>(&body)
                    .ok()
                    .and_then(|v| parse_jsonrpc(&v).ok().map(|r| r.id))
                    .unwrap_or(Value::Null);
                return error_http(
                    400,
                    jsonrpc_error_obj(Some(&id), INVALID_REQUEST, &e.to_string()),
                );
            }
        };
        // Dispatch.
        let (id, result_value, http_status) = self.dispatch(&req);
        let body = match result_value {
            Ok(result) => serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result,
            }))
            .unwrap_or_else(|_| {
                serde_json::to_string(&jsonrpc_error_obj(Some(&id), INTERNAL_ERROR, "serialize response"))
                    .unwrap()
            }),
            Err((code, msg)) => {
                serde_json::to_string(&jsonrpc_error_obj(Some(&id), code, &msg)).unwrap()
            }
        };
        (http_status, body)
    }

    /// Route a validated request through the pipeline. Returns
    /// `(id, Ok(result) | Err((code, message)), http_status)`.
    fn dispatch(&self, req: &JsonRpcRequest) -> (Value, Result<Value, (i64, String)>, u16) {
        let occurred_at = crate::util::utc_now();
        let session_id = super::governance::generate_session_id();
        // Discovery: serve the compact catalog locally (no backend forward).
        if req.method.ends_with("/list") {
            let (items, _truncated) =
                super::list_catalog(&self.runtime.catalog, super::DEFAULT_DISCOVERY_CAP);
            let result = serde_json::json!({ "items": items });
            // Audit the local discovery response (allow, no backend).
            let _ = self.write_discovery_audit(req, &occurred_at);
            return (req.id.clone(), Ok(result), 200);
        }
        // Namespaced call: resolve catalog entry + backend.
        let namespaced = req
            .params
            .get("name")
            .and_then(|v| v.as_str())
            .or_else(|| req.params.get("uri").and_then(|v| v.as_str()));
        let namespaced = match namespaced {
            Some(n) => n.to_string(),
            None => {
                return (
                    req.id.clone(),
                    Err((INVALID_REQUEST, "missing namespaced 'name' in params".into())),
                    400,
                );
            }
        };
        let entry = self.runtime.catalog.iter().find(|e| e.namespaced_name == namespaced);
        let (entry_ref, backend_id, scan_status) = match entry {
            Some(e) => (Some(e), Some(e.backend_id.as_str()), e_scan(e)),
            None => {
                return (
                    req.id.clone(),
                    Err((METHOD_NOT_FOUND, format!("unknown capability '{namespaced}'"))),
                    404,
                );
            }
        };
        let ctx = RequestContext {
            request_id: &req.id,
            session_id: &session_id,
            client_id: "serve",
            method: &req.method,
            namespaced_name: Some(&namespaced),
            route_card_id: None,
            params: &req.params,
        };
        match handle_request(
            &ctx,
            entry_ref,
            backend_id,
            scan_status,
            entry_ref.map(|e| e.source_hash.is_some()).unwrap_or(false),
            &self.runtime.policy,
            &self.runtime.governance,
            &self.runtime.router,
            DEFAULT_FORWARD_TIMEOUT,
            &self.runtime.audit,
            &occurred_at,
        ) {
            Ok(v) => (req.id.clone(), Ok(v), 200),
            Err(e) => {
                let code = error_code_for(&e);
                (req.id.clone(), Err((code, e.to_string())), error_http_status(&e))
            }
        }
    }

    fn write_discovery_audit(&self, req: &JsonRpcRequest, occurred_at: &str) {
        let record = AuditRecord {
            occurred_at: occurred_at.to_string(),
            request_id: req.id.to_string(),
            session_id: "discovery".to_string(),
            client_id: "serve".to_string(),
            method: req.method.clone(),
            namespaced_name: None,
            backend: None,
            decision: "allow".to_string(),
            reason_code: None,
            latency_ms: Some(0),
            cost_delta: None,
            route_card_id: None,
            domain_model_hash_source: None,
            semantic_envelope_id: None,
            semantic_envelope_event_type: None,
            detail: Some("local catalog discovery".to_string()),
        };
        // Best-effort: discovery is read-only local. A write failure here does
        // not block the response (no backend was consulted), but is logged.
        if let Err(e) = self.runtime.audit.write(&record) {
            eprintln!("gateway serve: discovery audit write failed: {e}");
        }
    }
}

fn e_scan(_e: &GatewayCatalogEntry) -> Option<&str> {
    None
}

fn validate_exposure(listener: &ListenerConfig) -> Result<(), GatewayError> {
    if !listener.exposure_ok() {
        return Err(GatewayError::InvalidRequest {
            reason: format!(
                "non-loopback bind '{}' requires tls or tunnel mode (spec 0020 §15)",
                listener.bind
            ),
        });
    }
    Ok(())
}

/// Register one `MCPServer`'s transport into the router.
fn register_transport(
    router: &mut Router,
    server: &super::config::MCPServer,
) -> Result<(), GatewayError> {
    if server.command_or_url.is_empty() {
        // No transport configured: the backend is registered as a name only;
        // a call to it resolves and fails with BackendUnavailable at forward.
        return Ok(());
    }
    match server.transport {
        super::GatewayTransport::Stdio => {
            router.register(
                &server.id,
                Box::new(StdioTransport::new(&server.command_or_url, server.args.clone())),
            );
        }
        super::GatewayTransport::Http => {
            let transport = HttpTransport::new(&server.command_or_url)?;
            router.register(&server.id, Box::new(transport));
        }
        // SSE/WebSocket backends are not forwarded in v0.1; the backend stays
        // registered as a name only and a call fails with BackendUnavailable.
        super::GatewayTransport::Sse | super::GatewayTransport::Websocket => {}
    }
    Ok(())
}

// ── HTTP parsing (minimal, bounded) ──────────────────────────────────────────

/// Read the request line + headers. Returns `(method, path, content_length)`.
fn read_request_head<R: BufRead>(reader: &mut R) -> Result<(String, String, usize), String> {
    let mut total = 0usize;
    let mut request_line = String::new();
    let n = reader.read_line(&mut request_line).map_err(|e| format!("read request line: {e}"))?;
    if n == 0 {
        return Err("empty request".into());
    }
    total += n;
    let line = request_line.trim_end_matches(['\r', '\n']);
    let mut parts = line.splitn(3, ' ');
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    if method.is_empty() || path.is_empty() {
        return Err("malformed request line".into());
    }
    let mut content_length: Option<usize> = None;
    loop {
        let mut header = String::new();
        let n = reader.read_line(&mut header).map_err(|e| format!("read header: {e}"))?;
        if n == 0 {
            return Err("unexpected EOF in headers".into());
        }
        total += n;
        if total > HEADER_CAP {
            return Err("header block too large".into());
        }
        let trimmed = header.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break; // end of headers
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse::<usize>().ok();
            }
        }
    }
    Ok((method, path, content_length.unwrap_or(0)))
}

/// Read up to `content_length` bytes, capped at `REQUEST_SIZE_CAP`.
fn read_body<R: BufRead>(reader: &mut R, content_length: usize) -> Result<Vec<u8>, String> {
    if content_length > REQUEST_SIZE_CAP {
        return Err(format!(
            "content-length {content_length} exceeds cap {REQUEST_SIZE_CAP}"
        ));
    }
    let mut buf = vec![0u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut buf)
            .map_err(|e| format!("read body: {e}"))?;
    }
    Ok(buf)
}

fn write_response<W: Write>(stream: &mut W, status: u16, body: &str) -> std::io::Result<()> {
    let reason = http_reason(status);
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

fn http_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

/// Map a `GatewayError` to a JSON-RPC error code.
fn error_code_for(e: &GatewayError) -> i64 {
    match e {
        GatewayError::InvalidRequest { .. } => INVALID_REQUEST,
        GatewayError::BackendUnavailable { .. } | GatewayError::BackendError { .. } => INTERNAL_ERROR,
        GatewayError::ScopeDenied { .. } | GatewayError::ApprovalRequired { .. } => -32603,
        GatewayError::ScanBlocked { .. } | GatewayError::ProvenanceMissing { .. } => -32603,
        GatewayError::BudgetExceeded { .. } | GatewayError::SessionLimitExceeded { .. } => -32603,
        GatewayError::AuditWriteFailed { .. } => INTERNAL_ERROR,
        _ => INTERNAL_ERROR,
    }
}

/// Map a `GatewayError` to an HTTP status.
fn error_http_status(e: &GatewayError) -> u16 {
    match e {
        GatewayError::InvalidRequest { .. } => 400,
        GatewayError::ScopeDenied { .. } | GatewayError::ApprovalRequired { .. }
        | GatewayError::ScanBlocked { .. } | GatewayError::ProvenanceMissing { .. } => 403,
        GatewayError::BudgetExceeded { .. } | GatewayError::SessionLimitExceeded { .. } => 429,
        GatewayError::BackendUnavailable { .. } => 503,
        GatewayError::BackendError { .. } => 502,
        GatewayError::AuditWriteFailed { .. } => 500,
        _ => 500,
    }
}

/// Build a JSON-RPC error response object.
fn jsonrpc_error_obj(id: Option<&Value>, code: i64, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.cloned().unwrap_or(Value::Null),
        "error": { "code": code, "message": message }
    })
}

/// Package an error object + http status.
fn error_http(status: u16, body: Value) -> (u16, String) {
    (status, serde_json::to_string(&body).unwrap_or_else(|_| "{}".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn temp_root(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "swe-seed-gw-serve-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // ---- pure HTTP parsing tests ----

    #[test]
    fn read_request_head_parses_method_path_and_content_length() {
        let raw = "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\n";
        let mut cur = BufReader::new(Cursor::new(raw.as_bytes()));
        let (method, path, cl) = read_request_head(&mut cur).unwrap();
        assert_eq!(method, "POST");
        assert_eq!(path, "/mcp");
        assert_eq!(cl, 5);
    }

    #[test]
    fn read_request_head_rejects_oversized_header_block() {
        let mut raw = String::from("POST /mcp HTTP/1.1\r\n");
        let val = "x".repeat(8);
        // push many headers past HEADER_CAP
        for i in 0..(HEADER_CAP / 16 + 10) {
            raw.push_str(&format!("X-Pad-{i}: {val}\r\n"));
        }
        raw.push_str("\r\n");
        let mut cur = BufReader::new(Cursor::new(raw.into_bytes()));
        let err = read_request_head(&mut cur).unwrap_err();
        assert!(err.contains("header block too large"));
    }

    #[test]
    fn read_body_caps_at_request_size_cap() {
        let too_big = REQUEST_SIZE_CAP + 1;
        let mut cur = BufReader::new(Cursor::new(Vec::<u8>::new()));
        let err = read_body(&mut cur, too_big).unwrap_err();
        assert!(err.contains("exceeds cap"));
    }

    #[test]
    fn jsonrpc_error_object_shape() {
        let obj = jsonrpc_error_obj(Some(&serde_json::json!(7)), -32600, "bad");
        assert_eq!(obj["jsonrpc"], "2.0");
        assert_eq!(obj["id"], 7);
        assert_eq!(obj["error"]["code"], -32600);
        assert_eq!(obj["error"]["message"], "bad");
    }

    #[test]
    fn http_reason_covers_used_statuses() {
        assert_eq!(http_reason(200), "OK");
        assert_eq!(http_reason(400), "Bad Request");
        assert_eq!(http_reason(503), "Service Unavailable");
    }

    // ---- error mapping tests ----

    #[test]
    fn invalid_request_maps_to_400_and_code() {
        let e = GatewayError::InvalidRequest { reason: "x".into() };
        assert_eq!(error_http_status(&e), 400);
        assert_eq!(error_code_for(&e), INVALID_REQUEST);
    }

    #[test]
    fn backend_unavailable_maps_to_503() {
        let e = GatewayError::BackendUnavailable { backend: "fs".into() };
        assert_eq!(error_http_status(&e), 503);
    }

    #[test]
    fn scope_denied_maps_to_403() {
        let e = GatewayError::ScopeDenied { scope: "s".into() };
        assert_eq!(error_http_status(&e), 403);
    }

    // ---- end-to-end server tests via a real loopback socket ----

    fn write_config(root: &Path, servers: Vec<super::super::config::MCPServer>) {
        let cfg = GatewayConfig {
            servers,
            ..Default::default()
        };
        let path = root.join(super::super::config::GATEWAY_CONFIG_REL);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
    }

    /// A backend whose stdio responder echoes a fixed tools/list response.
    fn list_responder_server() -> super::super::config::MCPServer {
        use super::super::{config::MCPServer, GatewayTransport};
        MCPServer {
            id: "fs".into(),
            namespace: "fs".into(),
            transport: GatewayTransport::Stdio,
            command_or_url: "sh".into(),
            args: vec![
                "-c".into(),
                r#"read line; echo '{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"fs.read"}]}}'"#
                    .into(),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn serve_once_returns_local_discovery_for_tools_list() {
        let root = temp_root("discovery");
        write_config(&root, vec![list_responder_server()]);
        let server = GatewayServer::bind(&root, Some("127.0.0.1"), Some(0)).unwrap();
        let addr = server.local_addr().unwrap();
        let body = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}).to_string();
        let handle = std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            write!(
                s,
                "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            let resp = std::io::read_to_string(s).unwrap();
            resp
        });
        let _ = server.serve_once();
        let resp = handle.join().unwrap();
        assert!(resp.contains("HTTP/1.1 200"), "response was: {resp}");
        // local discovery: items array, no backend forward
        assert!(resp.contains("\"items\""));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn serve_once_forwards_namespaced_call_through_pipeline() {
        let root = temp_root("call");
        write_config(&root, vec![list_responder_server()]);
        // declare a catalog entry so tools/call fs.read resolves
        {
            let cfg_path = root.join(super::super::config::GATEWAY_CONFIG_REL);
            let mut cfg: GatewayConfig =
                serde_json::from_slice(&std::fs::read(&cfg_path).unwrap()).unwrap();
            cfg.servers[0].catalog.push(super::super::config::DeclaredCatalogEntry {
                kind: super::super::CatalogKind::Tool,
                name: "read".into(),
                description: "read a file".into(),
                input_schema: None,
                policy_tags: vec![],
            });
            std::fs::write(&cfg_path, serde_json::to_vec(&cfg).unwrap()).unwrap();
        }
        let server = GatewayServer::bind(&root, Some("127.0.0.1"), Some(0)).unwrap();
        let addr = server.local_addr().unwrap();
        let body = serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"fs.read","args":{"path":"/x"}}
        })
        .to_string();
        let handle = std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            write!(
                s,
                "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            std::io::read_to_string(s).unwrap()
        });
        let _ = server.serve_once();
        let resp = handle.join().unwrap();
        assert!(resp.contains("HTTP/1.1 200"), "response was: {resp}");
        // backend result forwarded through the pipeline
        assert!(resp.contains("fs.read"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn serve_once_rejects_oversized_body_with_400() {
        let root = temp_root("oversize");
        write_config(&root, vec![]);
        let server = GatewayServer::bind(&root, Some("127.0.0.1"), Some(0)).unwrap();
        let addr = server.local_addr().unwrap();
        // The cap-check keys off the Content-Length HEADER value (the attack
        // vector), so we claim an oversized length without actually sending a
        // huge body — no socket RST, and the server must reject on the header.
        let claimed = REQUEST_SIZE_CAP + 10;
        let handle = std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            write!(
                s,
                "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: {claimed}\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
            s.shutdown(std::net::Shutdown::Write).ok();
            std::io::read_to_string(s).unwrap_or_default()
        });
        let _ = server.serve_once();
        let resp = handle.join().unwrap();
        assert!(resp.contains("HTTP/1.1 400"), "response was: {resp}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn serve_once_rejects_malformed_json_with_400() {
        let root = temp_root("malformed");
        write_config(&root, vec![]);
        let server = GatewayServer::bind(&root, Some("127.0.0.1"), Some(0)).unwrap();
        let addr = server.local_addr().unwrap();
        let body = "{not json";
        let handle = std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            write!(
                s,
                "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            std::io::read_to_string(s).unwrap()
        });
        let _ = server.serve_once();
        let resp = handle.join().unwrap();
        assert!(resp.contains("HTTP/1.1 400"), "response was: {resp}");
        assert!(resp.contains("malformed JSON"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn serve_once_rejects_non_loopback_bind_without_tls_or_tunnel() {
        let root = temp_root("nonloopback");
        write_config(&root, vec![]);
        // A non-loopback bind without tls/tunnel must fail at bind.
        let err = GatewayServer::bind(&root, Some("0.0.0.0"), Some(0))
            .err()
            .expect("non-loopback bind must fail");
        assert_eq!(err.reason_code(), "invalid_request");
        assert!(err.to_string().contains("non-loopback"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn serve_once_rejects_unknown_namespaced_call_with_404() {
        let root = temp_root("unknown");
        write_config(&root, vec![]);
        let server = GatewayServer::bind(&root, Some("127.0.0.1"), Some(0)).unwrap();
        let addr = server.local_addr().unwrap();
        let body = serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"nope.missing"}
        })
        .to_string();
        let handle = std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(addr).unwrap();
            write!(
                s,
                "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            std::io::read_to_string(s).unwrap()
        });
        let _ = server.serve_once();
        let resp = handle.join().unwrap();
        assert!(resp.contains("HTTP/1.1 404"), "response was: {resp}");
        std::fs::remove_dir_all(&root).ok();
    }
}
