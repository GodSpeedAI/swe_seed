//! Minimal MCP client for the Context Kernel (WP-5, F-04).
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
//! `SWE_SEED_CONTEXT_KERNEL_BIN` (embedded spawn) or
//! `SWE_SEED_CONTEXT_KERNEL_URL` (persistent service) is set.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Map, Value};

use super::context_bundle::{bundle_from_response, verify_context_bundle, BundleFacts};
use super::envelope::{make_event_verified, Envelope};
use super::identity::VerifiedDomainIdentity;
use super::world::WorldRef;
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
    /// The packet names no world, or a different world than the request.
    WorldMismatch(super::world::WorldRefError),
    /// `require_complete` was set and the packet is not `complete`.
    IncompleteContext {
        completeness: RetrievalCompleteness,
        omissions: Vec<String>,
    },
    /// The canonical CEP godspeed.context_bundle present in the response
    /// was forged, modified, cross-wired, or untruthful -- refused before
    /// anything is handed to the caller.
    BundleRejected(String),
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
            Self::WorldMismatch(e) => write!(f, "context packet world rejected: {e}"),
            Self::IncompleteContext {
                completeness,
                omissions,
            } => write!(
                f,
                "context packet is {} (omissions: {:?}); complete context was required",
                completeness.as_str(),
                omissions
            ),
            Self::BundleRejected(m) => {
                write!(f, "context bundle rejected at boundary: {m}")
            }
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
    /// The canonical E3 envelope (producer: `context-kernel`), read during the
    /// bounded dual-read migration window.
    pub envelope: Envelope,
    /// The canonical CEP `godspeed.context_bundle` adjudicated alongside it.
    /// This is the cross-system artifact; the packet references ITS identity
    /// in the governed lineage.
    pub bundle: Option<Value>,
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

    /// The world the packet claims to be retrieved for (unvalidated string).
    pub fn world_ref(&self) -> Option<&str> {
        self.envelope.world_ref()
    }

    /// How much of the matching corpus the packet carries, as CK stated it.
    /// An absent or unrecognised value is [`RetrievalCompleteness::Unknown`],
    /// never `Complete`.
    pub fn retrieval_completeness(&self) -> RetrievalCompleteness {
        RetrievalCompleteness::from_payload(&self.envelope.payload)
    }

    /// CK's machine-readable omission reasons, empty when none were stated.
    pub fn omissions(&self) -> Vec<&str> {
        self.envelope
            .payload
            .get("omissions")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }

    /// The canonical CEP context bundle adjudicated alongside this packet
    /// (`None` only during the bounded legacy window when none was sent).
    pub fn bundle(&self) -> Option<&Value> {
        self.bundle.as_ref()
    }

    /// CEP `envelope_id` of the bundle -- occurrence identity for lineage.
    pub fn bundle_envelope_id(&self) -> Option<&str> {
        self.bundle
            .as_ref()?
            .get("envelope_id")
            .and_then(Value::as_str)
    }

    /// CEP `integrity.content_hash` of the bundle -- exact content identity.
    pub fn bundle_content_hash(&self) -> Option<&str> {
        self.bundle
            .as_ref()?
            .get("integrity")
            .and_then(|i| i.get("content_hash"))
            .and_then(Value::as_str)
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

/// What Context Kernel says about how much of the matching corpus a packet
/// carries. Relative to the query and corpus, never to reality. Only
/// `Complete` means "everything that matched is here, in full"; a missing or
/// unrecognised statement is `Unknown`, which is not complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalCompleteness {
    Complete,
    Partial,
    None,
    Unknown,
}

impl RetrievalCompleteness {
    fn from_payload(payload: &Value) -> Self {
        match payload.get("retrieval_completeness").and_then(Value::as_str) {
            Some("complete") => Self::Complete,
            Some("partial") => Self::Partial,
            Some("none") => Self::None,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::None => "none",
            Self::Unknown => "unknown",
        }
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
    /// The world the request was pinned to. The packet must name exactly this
    /// world (CEP-0008).
    pub world_ref: &'a str,
    /// When true, a packet whose `retrieval_completeness` is not `complete`
    /// is refused rather than surfaced. Off by default: a partial packet is
    /// reported honestly through [`ContextPacket::retrieval_completeness`].
    pub require_complete: bool,
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

    // 3b. World: the packet must be pinned to the request's world. Absent,
    //     malformed, or different is refused (CEP-0008 `world_ref`).
    envelope
        .verified_world_ref()
        .map_err(ContextClientError::WorldMismatch)?;
    WorldRef::parse(expected.world_ref)
        .map_err(ContextClientError::WorldMismatch)?
        .require_same(envelope.world_ref())
        .map_err(ContextClientError::WorldMismatch)?;

    // 3c. The canonical CEP bundle (dual-read: preferred when present). A
    //     forged, modified, cross-wired, or untruthful bundle is refused
    //     BEFORE any outcome is interpreted.
    if let Some(bundle) = bundle_from_response(resp) {
        let facts = verify_context_bundle(
            bundle,
            expected.work_request_id,
            expected.world_ref,
            Some(expected.context_requirement_id),
        )
        .map_err(ContextClientError::BundleRejected)?;
        // Cross-representation consistency: the bundle and the legacy packet
        // must describe the SAME bounded retrieval.
        let packet_id = envelope
            .payload
            .get("context_packet_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let bundle_packet_id = bundle
            .get("extensions")
            .and_then(|e| e.get("godspeed.context_bundle"))
            .and_then(|b| b.get("context_packet_id"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !packet_id.is_empty() && !bundle_packet_id.is_empty() && packet_id != bundle_packet_id {
            return Err(ContextClientError::BundleRejected(format!(
                "bundle names packet {bundle_packet_id:?}, response carries {packet_id:?}"
            )));
        }
        let packet_level = RetrievalCompleteness::from_payload(&envelope.payload);
        if packet_level != RetrievalCompleteness::Unknown
            && packet_level.as_str() != facts.retrieval_completeness
        {
            return Err(ContextClientError::BundleRejected(format!(
                "bundle states {:?}, packet states {:?}",
                facts.retrieval_completeness,
                packet_level.as_str()
            )));
        }
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
            let packet = ContextPacket {
                envelope,
                bundle: bundle_from_response(resp).cloned(),
            };
            let completeness = packet.retrieval_completeness();
            if expected.require_complete && completeness != RetrievalCompleteness::Complete {
                return Err(ContextClientError::IncompleteContext {
                    completeness,
                    omissions: packet.omissions().into_iter().map(String::from).collect(),
                });
            }
            Ok(packet)
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

/// A minimal MCP client for the Context Kernel. Two transports:
///
/// - `spawn` (stdio): embedded/dev/test mode. CK is launched per client with
///   an ephemeral database; the child is killed on drop.
/// - `connect` (HTTP): production/service mode. CK is a persistent service
///   (its `serve --sse` endpoint keeps the corpus indexed across requests);
///   the client owns no process.
///
/// Either way CK is reached over MCP `tools/call` only, never NATS.
pub struct ContextKernelClient {
    transport: ClientTransport,
}

enum ClientTransport {
    Spawned {
        child: Child,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    },
    /// Persistent CK service: `http://host:port[/prefix]`, MCP over POST /mcp.
    Service { base: String },
}

/// The negotiated service facts: the client refuses to treat anything else
/// as a Context Kernel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelService {
    pub name: String,
    pub version: String,
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
    /// The semantic world the request is pinned to; sent in the E2 payload.
    pub world_ref: &'a WorldRef,
    /// Refuse (rather than surface) a packet that is not `complete`.
    pub require_complete: bool,
    /// Optional SEA-Forge authority references (pass-through only).
    pub authority_decision_id: Option<&'a str>,
    pub authority_reference: Option<&'a str>,
    /// When true (default semantics), zero citations yield the explicit
    /// governed no-context outcome instead of silent success.
    pub required: bool,
}

impl ContextKernelClient {
    /// Configure from the environment. Production path first:
    /// `SWE_SEED_CONTEXT_KERNEL_URL` reaches a persistent CK service (whose
    /// corpus index survives across requests). Embedded/dev/test second:
    /// `SWE_SEED_CONTEXT_KERNEL_BIN` spawns CK with an ephemeral store.
    /// Returns `None` when neither is configured — callers treat that as
    /// explicit "CK unavailable" (never silently fabricated context).
    pub fn from_env() -> Option<std::io::Result<Self>> {
        if let Ok(base) = std::env::var("SWE_SEED_CONTEXT_KERNEL_URL") {
            return Some(Ok(Self::connect(&base)));
        }
        let bin = std::env::var("SWE_SEED_CONTEXT_KERNEL_BIN").ok()?;
        Some(Self::spawn(
            &bin,
            std::env::var("SWE_SEED_CONTEXT_CORPUS_ROOT")
                .ok()
                .as_deref(),
        ))
    }

    /// Production/service mode: reach an already-running persistent CK over
    /// HTTP MCP (`base` like `http://127.0.0.1:8765`).
    pub fn connect(base: &str) -> Self {
        Self {
            transport: ClientTransport::Service {
                base: base.trim_end_matches('/').to_string(),
            },
        }
    }

    /// Explicit capability negotiation over the surrounding MCP boundary: the
    /// other end must report `get_status` as the `context-kernel` service
    /// before any `context_required` call is made.
    pub fn negotiate(&mut self) -> Result<KernelService, ContextClientError> {
        let status = self.call_tool("get_status", json!({}))?;
        let service = status.get("service");
        let name = service
            .and_then(|s| s.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let version = service
            .and_then(|s| s.get("version"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if name != "context-kernel" {
            return Err(ContextClientError::Transport(format!(
                "expected a context-kernel service, got {name:?}"
            )));
        }
        if version.trim().is_empty() {
            return Err(ContextClientError::Transport(
                "context-kernel service reported no version".to_string(),
            ));
        }
        Ok(KernelService { name, version })
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
            transport: ClientTransport::Spawned {
                child,
                stdin,
                stdout,
            },
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
        payload.insert(
            "world_ref".into(),
            Value::String(req.world_ref.to_string()),
        );
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
                world_ref: req.world_ref.as_str(),
                require_complete: req.require_complete,
            },
        )
    }

    /// Generic JSON-RPC `tools/call` over whichever transport the client owns.
    /// HTTP transport is plain HTTP/1.1 POST to the service's MCP endpoint
    /// (no extra dependencies: the std-only client speaks just enough HTTP).
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, ContextClientError> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        });
        let line = serde_json::to_string(&request)
            .map_err(|e| ContextClientError::Transport(format!("encode request: {e}")))?;
        let v: Value = match &mut self.transport {
            ClientTransport::Spawned { stdin, stdout, .. } => {
                stdin
                    .write_all(line.as_bytes())
                    .and_then(|_| stdin.write_all(b"\n"))?;
                stdin.flush()?;

                let mut out = String::new();
                stdout.read_line(&mut out)?;
                serde_json::from_str(out.trim()).map_err(|e| {
                    ContextClientError::Transport(format!("malformed response: {e}"))
                })?
            }
            ClientTransport::Service { base } => http_post_json_rpc(base, &line)
                .map_err(|e| {
                    ContextClientError::Transport(format!("http transport: {e}"))
                })?,
        };
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
        if let ClientTransport::Spawned { stdin, child, .. } = &mut self.transport {
            let _ = stdin.flush();
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Minimal HTTP/1.1 JSON-RPC POST to `{base}/mcp`. Plain HTTP (localhost
/// service); TLS/underlay features are out of scope for the client.
fn http_post_json_rpc(base: &str, body: &str) -> std::io::Result<Value> {
    fn fail(msg: String) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
    }
    let (host, port, path) = split_http_base(base)?;
    let mut stream = std::net::TcpStream::connect_timeout(
        &format!("{host}:{port}").parse().map_err(|_| {
            fail(format!("unparsable http address in {base:?}"))
        })?,
        std::time::Duration::from_secs(5),
    )?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
    let endpoint = if path.is_empty() { "/mcp".to_string() } else { format!("{path}/mcp") };
    let request = format!(
        "POST {endpoint} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes())?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp)?;
    let body_start = resp
        .find("\r\n\r\n")
        .ok_or_else(|| fail(format!("malformed http response from {base:?}")))?;
    serde_json::from_str(resp[body_start + 4..].trim())
        .map_err(|e| fail(format!("malformed json-rpc response: {e}")))
}

/// Split `http://host:port[/prefix]` (HTTP only).
fn split_http_base(base: &str) -> std::io::Result<(String, u16, String)> {
    let fail = |msg: String| std::io::Error::new(std::io::ErrorKind::InvalidInput, msg);
    let rest = base
        .strip_prefix("http://")
        .ok_or_else(|| fail(format!("only plain http CK services are supported: {base:?}")))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], rest[i..].to_string()),
        None => (rest, String::new()),
    };
    let (host, port) = match authority.rfind(':') {
        Some(i) => (
            authority[..i].to_string(),
            authority[i + 1..]
                .parse::<u16>()
                .map_err(|_| fail(format!("bad port in {base:?}")))?,
        ),
        None => (authority.to_string(), 80),
    };
    if host.trim().is_empty() {
        return Err(fail(format!("empty host in {base:?}")));
    }
    Ok((host, port, path))
}
