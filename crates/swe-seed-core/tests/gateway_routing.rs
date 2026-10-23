//! Gateway stage-5 end-to-end proof: real stdio + HTTP transports forward
//! JSON-RPC, preserve ids, and reach the backend. Run: `cargo test gateway_routing`.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use swe_seed_core::gateway::{HttpTransport, Router, StdioTransport};

#[test]
fn stdio_transport_forwards_and_returns_result() {
    // A shell responder: read one request line, echo a fixed-id JSON-RPC reply.
    let script = r#"read line; echo '{"jsonrpc":"2.0","id":1,"result":{"forwarded":true}}'"#;
    let transport = StdioTransport::new("sh", vec!["-c".into(), script.into()]);
    let router = Router::with("fs", Box::new(transport));
    let result = router
        .invoke(
            "fs",
            &json!(1),
            "tools/list",
            json!({}),
            Duration::from_secs(3),
        )
        .unwrap();
    assert_eq!(result, json!({"forwarded":true}));
}

#[test]
fn stdio_transport_timeout_when_backend_silent() {
    // `sleep` never replies → the bounded timeout yields BackendUnavailable.
    let transport = StdioTransport::new("sleep", vec!["5".into()]);
    let router = Router::with("fs", Box::new(transport));
    let err = router
        .invoke("fs", &json!(1), "tools/list", json!({}), Duration::from_millis(300))
        .unwrap_err();
    assert_eq!(err.reason_code(), "backend_unavailable");
}

#[test]
fn http_transport_forwards_method_and_params_and_preserves_id() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let received: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
    let received_clone = received.clone();

    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_http_request(&mut stream);
        *received_clone.lock().unwrap() = request;
        let body = r#"{"jsonrpc":"2.0","id":42,"result":{"items":["a","b"]}}"#;
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(resp.as_bytes()).unwrap();
    });

    // wait briefly for the listener to be ready (bind is already done, so ok)
    let transport = HttpTransport::new(&format!("http://127.0.0.1:{}", addr.port())).unwrap();
    let router = Router::with("api", Box::new(transport));
    let result = router
        .invoke(
            "api",
            &json!(42),
            "tools/call",
            json!({"name":"api.op","arguments":{"x":1}}),
            Duration::from_secs(3),
        )
        .unwrap();
    assert_eq!(result, json!({"items":["a","b"]}));

    // the backend actually received the forwarded method + params
    let got = received.lock().unwrap().clone().unwrap();
    assert_eq!(got["method"], "tools/call");
    assert_eq!(got["params"]["name"], "api.op");
    assert_eq!(got["id"], 42);
}

#[test]
fn http_transport_connection_refused_is_unavailable() {
    // pick a port that's almost certainly closed
    let transport = HttpTransport::new("http://127.0.0.1:9").unwrap();
    let router = Router::with("dead", Box::new(transport));
    let err = router
        .invoke("dead", &json!(1), "tools/list", json!({}), Duration::from_millis(300))
        .unwrap_err();
    assert_eq!(err.reason_code(), "backend_unavailable");
}

/// Read one HTTP request from the stream and return its parsed JSON body.
fn read_http_request(stream: &mut TcpStream) -> Option<Value> {
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = rest.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}
