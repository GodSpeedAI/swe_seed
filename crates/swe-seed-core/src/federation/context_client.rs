//! Minimal MCP stdio client for the Context Kernel (WP-5, F-04).
//!
//! SWE_SEED's context stage calls CK's `context_required` tool to obtain a
//! cited `ContextPacketCreated`, satisfying POL-ACL-001/003 from a real context
//! source instead of a hand-built packet. §14 decision: CK is reached over
//! **MCP only** (stdio), never NATS.
//!
//! Convergence T02: the E2 request is a canonical v1 envelope produced by the
//! exclusive authoritative producer (`swe_seed` for `ContextRequired`), and the
//! E3 response is adjudicated through the composed boundary gate
//! (`validate_envelope`) plus correlation/citation checks. Required-context
//! absence is an explicit governed outcome — never a silent success.
//!
//! Config-gated and offline-first: the client is a no-op unless
//! `SWE_SEED_CONTEXT_KERNEL_BIN` is set.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Map, Value};

use super::envelope::{make_event_verified, Envelope};
use super::identity::VerifiedDomainIdentity;
use super::{validate_envelope, ConsumeError};

/// Why acquiring context failed or produced no acquisition.
#[derive(Debug)]
pub enum ContextClientError {
    Io(std::io::Error),
    /// CK answered with a JSON-RPC error or a malformed body.
    Transport(String),
    /// The response failed the canonical boundary gate (producer authority,
    /// identity shape, drift) or its causality record.
    BoundaryRejected(ConsumeError),
    /// The response envelope/packet correlates to a different work request or
    /// requirement than the one we asked for (frozen falsifier: cross-wired
    /// packets are rejected).
    CrossWired {
        field: &'static str,
        expected: String,
        got: String,
    },
    /// The causal parent we sent (the E2 event id) was not recorded on the E3
    /// envelope — derived evidence lost its provenance.
    CausalityBroken {
        expected_parent: String,
        recorded: Vec<String>,
    },
    /// A REQUIRED acquisition returned zero citations: the explicit governed
    /// no-context outcome. Callers must handle this as non-success.
    GovernedNoContext {
        reason: Option<String>,
    },
    /// The response violated the outcome protocol itself.
    InvalidResponse(String),
}

impl std::fmt::Display for ContextClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "context client io error: {e}"),
            Self::Transport(m) => write!(f, "context kernel transport error: {m}"),
            Self::BoundaryRejected(e) => write!(f, "context packet rejected at boundary: {e}"),
            Self::CrossWired {
                field,
                expected,
                got,
            } => write!(f, "cross-wired context packet: {field} expected {expected:?}, got {got:?}"),
            Self::CausalityBroken {
                expected_parent,
                recorded,
            } => write!(
                f,
                "packet does not cite our E2 request as causal parent: expected {expected_parent}, recorded {recorded:?}"
            ),
            Self::GovernedNoContext { reason } => write!(
                f,
                "governed no-context outcome (explicitly NOT successful acquisition): {}",
                reason.as_deref().unwrap_or("<unspecified>")
            ),
            Self::InvalidResponse(m) => write!(f, "invalid context response: {m}"),
        }
    }
}

impl std::error::Error for ContextClientError {}

impl From<std::io::Error> for ContextClientError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// A citation-bearing `ContextPacketCreated` envelope returned by CK, already
/// adjudicated: producer authority, identity, drift, causality, and
/// correlation all verified at the boundary.
#[derive(Debug, Clone)]
pub struct ContextPacket {
    /// The canonical E3 envelope (producer: `context-kernel`).
    pub envelope: Envelope,
}

impl ContextPacket {
    /// Number of citations in the packet (POL-ACL-003 requires ≥1).
    pub fn citation_count(&self) -> usize {
        self.envelope
            .payload
            .get("citations")
            .and_then(|c| c.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
    }

    pub fn work_request_id(&self) -> Option<&str> {
        self.envelope.work_request_id()
    }

    pub fn context_requirement_id(&self) -> Option<&str> {
        self.envelope
            .payload
            .get("context_requirement_id")
            .and_then(Value::as_str)
    }

    pub fn domain_model_hash(&self) -> Option<&str> {
        self.envelope.domain_model_hash()
    }

    /// Pass-through authority REFERENCES (opaque strings). CK cannot create
    /// authority; these only point at SEA-Forge decisions (I4).
    pub fn authority_reference(&self) -> Option<&str> {
        self.envelope
            .payload
            .get("authority_reference")
            .and_then(Value::as_str)
    }

    pub fn citations(&self) -> &[Value] {
        self.envelope
            .payload
            .get("citations")
            .and_then(Value::as_array)
            .map(|a| a.as_slice())
            .unwrap_or(&[])
    }
}

/// What the caller expects back — used to adjudicate the response against the
/// exact request (correlation + drift + causality). Public so contract tests
/// exercise the same adjudicator the live client uses.
pub struct ExpectedContext<'a> {
    pub work_request_id: &'a str,
    pub context_requirement_id: &'a str,
    pub domain_model_hash: &'a str,
    pub request_event_id: &'a str,
    pub required: bool,
}

/// Adjudicate CK's raw tool response into a verified [`ContextPacket`].
///
/// Pure function over the response JSON so the full rejection battery is
/// unit-testable without spawning the CK binary.
pub fn adjudicate_context_response(
    resp: &Value,
    expected: &ExpectedContext<'_>,
) -> Result<ContextPacket, ContextClientError> {
    let env_val = resp.get("context_envelope").cloned().unwrap_or(Value::Null);
    if env_val.is_null() {
        return Err(ContextClientError::InvalidResponse(
            "response missing context_envelope".into(),
        ));
    }
    let envelope: Envelope = serde_json::from_value(env_val)
        .map_err(|e| ContextClientError::InvalidResponse(format!("envelope decode: {e}")))?;

    // 1. Composed canonical boundary gate: exclusive producer authority
    //    (context-kernel owns ContextPacketCreated), identity shape/placeholder
    //    rules, drift against OUR declared model identity, causality records.
    validate_envelope(&envelope, expected.domain_model_hash)
        .map_err(ContextClientError::BoundaryRejected)?;

    // 2. Causality: the E3 envelope must cite OUR E2 request event id.
    let parents = envelope.causal_parents();
    if !parents.contains(&expected.request_event_id) {
        return Err(ContextClientError::CausalityBroken {
            expected_parent: expected.request_event_id.to_string(),
            recorded: parents.into_iter().map(str::to_string).collect(),
        });
    }

    // 3. Correlation: cross-wired packets are rejected (frozen falsifier).
    let got_wr = envelope.work_request_id().unwrap_or("");
    if got_wr != expected.work_request_id {
        return Err(ContextClientError::CrossWired {
            field: "work_request_id",
            expected: expected.work_request_id.to_string(),
            got: got_wr.to_string(),
        });
    }
    let got_cr = envelope
        .payload
        .get("context_requirement_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    if got_cr != expected.context_requirement_id {
        return Err(ContextClientError::CrossWired {
            field: "context_requirement_id",
            expected: expected.context_requirement_id.to_string(),
            got: got_cr.to_string(),
        });
    }

    // 4. Outcome protocol: only `cited` is successful acquisition; zero
    //    citations under `required` is the explicit governed outcome.
    let outcome = resp.get("outcome").and_then(Value::as_str).unwrap_or("");
    let citation_count = envelope
        .payload
        .get("citations")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    match outcome {
        "cited" => {
            if citation_count == 0 {
                return Err(ContextClientError::InvalidResponse(
                    "outcome 'cited' but zero citations".into(),
                ));
            }
            Ok(ContextPacket { envelope })
        }
        "no_context_governed" | "empty_optional" => {
            if expected.required && outcome != "no_context_governed" {
                return Err(ContextClientError::InvalidResponse(format!(
                    "required acquisition resolved to {outcome:?}"
                )));
            }
            Err(ContextClientError::GovernedNoContext {
                reason: resp.get("reason").and_then(Value::as_str).map(String::from),
            })
        }
        other => Err(ContextClientError::InvalidResponse(format!(
            "unknown acquisition outcome {other:?}"
        ))),
    }
}

/// A minimal MCP stdio client. Spawns the CK binary once and reuses the
/// connection for one or more `tools/call` requests. Drop closes the child.
pub struct ContextKernelClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

/// Arguments for one canonical E2 acquisition.
pub struct ContextRequest<'a> {
    pub work_request_id: &'a str,
    pub context_requirement_id: &'a str,
    pub corpus_id: &'a str,
    pub query: Option<&'a str>,
    /// Canonical DomainForge identity — construction type-gated, so a
    /// fallback/placeholder pseudo-hash cannot reach this boundary.
    pub domain_identity: &'a VerifiedDomainIdentity,
    /// Optional SEA-Forge authority references (pass-through only).
    pub authority_decision_id: Option<&'a str>,
    pub authority_reference: Option<&'a str>,
    /// When true (default semantics), zero citations yield the explicit
    /// governed no-context outcome instead of silent success.
    pub required: bool,
}

impl ContextKernelClient {
    /// Spawn the CK binary pointed at by `SWE_SEED_CONTEXT_KERNEL_BIN`, with an
    /// optional corpus root (`CK_CONTEXT_CORPUS_ROOT`). Returns `None` if the
    /// binary isn't configured — callers treat that as "CK unavailable".
    pub fn from_env() -> Option<std::io::Result<Self>> {
        let bin = std::env::var("SWE_SEED_CONTEXT_KERNEL_BIN").ok()?;
        Some(Self::spawn(
            &bin,
            std::env::var("SWE_SEED_CONTEXT_CORPUS_ROOT")
                .ok()
                .as_deref(),
        ))
    }

    /// Spawn a CK binary at `bin`, passing `corpus_root` via the env var CK
    /// reads (`CK_CONTEXT_CORPUS_ROOT`).
    pub fn spawn(bin: &str, corpus_root: Option<&str>) -> std::io::Result<Self> {
        let dir = std::env::temp_dir().join(format!(
            "swe-seed-ck-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir)?;
        let db = dir.join("ck.db");
        let data_dir = dir.join("data");
        std::fs::create_dir_all(&data_dir)?;

        let mut cmd = Command::new(bin);
        cmd.args([
            "serve",
            "--database",
            db.to_str().unwrap(),
            "--data-dir",
            data_dir.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
        if let Some(root) = corpus_root {
            cmd.env("CK_CONTEXT_CORPUS_ROOT", root);
        }

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().expect("stdin piped");
        let stdout = BufReader::new(child.stdout.take().expect("stdout piped"));
        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    /// Acquire context over the frozen E2/E3 boundary.
    ///
    /// Sends the canonical E2 envelope (identity-gated construction) and
    /// adjudicates the E3 response through the composed boundary gate before
    /// returning it. Zero-citation results surface as
    /// [`ContextClientError::GovernedNoContext`] when `required` — explicitly
    /// not a success.
    pub fn acquire_context(
        &mut self,
        req: ContextRequest<'_>,
    ) -> Result<ContextPacket, ContextClientError> {
        // Canonical E2 envelope: make_event_verified stamps the VERIFIED hash
        // and the swe-seed producer (exclusive authoritative producer of
        // ContextRequired per the frozen topology).
        let mut payload: Map<String, Value> = Map::new();
        payload.insert(
            "work_request_id".into(),
            Value::String(req.work_request_id.to_string()),
        );
        payload.insert(
            "context_requirement_id".into(),
            Value::String(req.context_requirement_id.to_string()),
        );
        payload.insert("corpus_id".into(), Value::String(req.corpus_id.to_string()));
        if let Some(q) = req.query {
            payload.insert("query".into(), Value::String(q.to_string()));
        }
        if let Some(a) = req.authority_decision_id {
            payload.insert("authority_decision_id".into(), Value::String(a.to_string()));
        }
        if let Some(a) = req.authority_reference {
            payload.insert("authority_reference".into(), Value::String(a.to_string()));
        }
        let e2 = make_event_verified("ContextRequired", payload, req.domain_identity);

        let mut arguments = serde_json::to_value(&e2)
            .map_err(|e| ContextClientError::Transport(format!("encode E2: {e}")))?;
        arguments["required"] = Value::Bool(req.required);

        let resp = self.call_tool("context_required", arguments)?;
        adjudicate_context_response(
            &resp,
            &ExpectedContext {
                work_request_id: req.work_request_id,
                context_requirement_id: req.context_requirement_id,
                domain_model_hash: req.domain_identity.as_str(),
                request_event_id: &e2.event_id,
                required: req.required,
            },
        )
    }

    /// Generic JSON-RPC `tools/call`.
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, ContextClientError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        });
        let line = serde_json::to_string(&request)
            .map_err(|e| ContextClientError::Transport(format!("encode request: {e}")))?;
        self.stdin
            .write_all(line.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))?;
        self.stdin.flush()?;

        let mut out = String::new();
        self.stdout.read_line(&mut out)?;
        let v: Value = serde_json::from_str(out.trim())
            .map_err(|e| ContextClientError::Transport(format!("malformed response: {e}")))?;
        if let Some(err) = v.get("error") {
            // Surface structured rejection reasons from the boundary gates.
            let msg = err.get("message").and_then(Value::as_str).unwrap_or("?");
            if msg.contains("identity rejected") {
                return Err(ContextClientError::BoundaryRejected(
                    ConsumeError::Identity(super::IdentityError::MalformedHash {
                        got: msg.to_string(),
                    }),
                ));
            }
            return Err(ContextClientError::Transport(msg.to_string()));
        }
        Ok(v.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl Drop for ContextKernelClient {
    fn drop(&mut self) {
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
