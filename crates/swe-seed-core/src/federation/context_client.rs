//! Minimal MCP stdio client for the Context Kernel (WP-5, F-04).
//!
//! SWE_SEED's context stage calls CK's `context_required` tool to obtain a
//! cited `ContextPacketCreated`, satisfying POL-ACL-001/003 from a real context
//! source instead of a hand-built packet. §14 decision: CK is reached over
//! **MCP only** (stdio), never NATS.
//!
//! Config-gated and offline-first: the client is a no-op (returns `None`)
//! unless `SWE_SEED_CONTEXT_KERNEL_BIN` is set. A CK call failure never breaks
//! the harness hot path — it logs and returns `None`, preserving current
//! behavior when CK is absent or unreachable.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

/// A citation-bearing context packet returned by CK's `context_required` tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextPacket {
    /// Raw `ContextPacketCreated` payload from CK.
    pub packet: Value,
    /// Raw `ContextAuthorized` payload (POL-ACL-002 result).
    pub authorization: Value,
}

impl ContextPacket {
    /// Number of citations in the packet (POL-ACL-003 requires ≥1).
    pub fn citation_count(&self) -> usize {
        self.packet
            .get("citations")
            .and_then(|c| c.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
    }

    /// True if CK authorized the corpus (POL-ACL-002).
    pub fn authorized(&self) -> bool {
        self.packet
            .get("authorized")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

/// A minimal MCP stdio client. Spawns the CK binary once and reuses the
/// connection for one or more `tools/call` requests. Drop closes the child.
pub struct ContextKernelClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl ContextKernelClient {
    /// Spawn the CK binary pointed at by `SWE_SEED_CONTEXT_KERNEL_BIN`, with an
    /// optional corpus root (`CK_CONTEXT_CORPUS_ROOT`). Returns `None` if the
    /// binary isn't configured — callers treat that as "CK unavailable".
    pub fn from_env() -> Option<std::io::Result<Self>> {
        let bin = std::env::var("SWE_SEED_CONTEXT_KERNEL_BIN").ok()?;
        Some(Self::spawn(
            &bin,
            std::env::var("SWE_SEED_CONTEXT_CORPUS_ROOT").ok().as_deref(),
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
        Ok(Self { child, stdin, stdout })
    }

    /// Call CK's `context_required` tool. Returns the cited packet on success.
    pub fn context_required(
        &mut self,
        work_request_id: &str,
        context_requirement_id: &str,
        corpus_id: &str,
        query: Option<&str>,
        is_private: bool,
        scope_claim: Option<&str>,
    ) -> std::io::Result<ContextPacket> {
        let arguments = json!({
            "work_request_id": work_request_id,
            "context_requirement_id": context_requirement_id,
            "corpus_id": corpus_id,
            "query": query,
            "max_results": 10,
            "is_private": is_private,
            "scope_claim": scope_claim,
        });
        let resp = self.call_tool("context_required", arguments)?;
        let packet = resp
            .get("context_packet")
            .cloned()
            .unwrap_or(Value::Null);
        let authorization = resp
            .get("authorization")
            .cloned()
            .unwrap_or(Value::Null);
        Ok(ContextPacket {
            packet,
            authorization,
        })
    }

    /// Generic JSON-RPC `tools/call`.
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> std::io::Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        });
        let line = serde_json::to_string(&request)?;
        self.stdin
            .write_all(line.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))?;
        self.stdin.flush()?;

        let mut out = String::new();
        self.stdout.read_line(&mut out)?;
        let v: Value = serde_json::from_str(out.trim())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if let Some(err) = v.get("error") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("CK error: {}", err),
            ));
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
