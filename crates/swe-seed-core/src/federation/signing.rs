//! Envelope signing (Phase B). Ed25519 signatures over a canonical signing
//! string so SEA-Forge can verify a SWE_Seed envelope (and its `trace_chain_root`)
//! was produced by SWE_Seed and not altered.
//!
//! ## Canonical signing string (the cross-repo parity contract)
//!
//! Signed bytes = the canonical signing string:
//! ```text
//! {namespace}\n{event_type}\n{occurred_at}\n{compact_sorted_payload_json}
//! ```
//! - `compact_sorted_payload_json`: `serde_json` of the payload with **sorted
//!   keys** (BTreeMap order) and **no whitespace**; the `signature` field is
//!   excluded from the payload when computing the string.
//! - `event_id` is intentionally **excluded** (it is an envelope identifier,
//!   not content; including it would prevent re-signing and adds no integrity).
//!
//! Ed25519 signs these UTF-8 bytes directly (no pre-hash). Both SWE_Seed
//! (Rust, `ed25519-dalek`) and SEA-Forge (Python, `cryptography`) MUST produce
//! byte-identical canonical strings; the committed test vector pins this.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::io::{Error, ErrorKind};

use super::envelope::Envelope;
use super::SIGNING_ALGORITHM;

/// Generate a fresh Ed25519 signing key (for `federation keygen`).
pub fn generate_signing_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

/// Where the public key lives (committed, safe to publish).
pub fn public_key_path(
    root: &std::path::Path,
    key_id: &str,
) -> std::io::Result<std::path::PathBuf> {
    let key_id = validate_key_id(key_id)?;
    Ok(root
        .join(".agent-harness/federation/keys")
        .join(format!("{key_id}.pub")))
}

/// Where the private key lives (gitignored `.swe-seed/`; encrypt via SOPS/age
/// before any committed use — see AGENTS / federation keygen output).
pub fn private_key_path(
    root: &std::path::Path,
    key_id: &str,
) -> std::io::Result<std::path::PathBuf> {
    let key_id = validate_key_id(key_id)?;
    Ok(root
        .join(".swe-seed/federation/keys")
        .join(format!("{key_id}.key")))
}

fn validate_key_id(key_id: &str) -> std::io::Result<&str> {
    let safe = !key_id.is_empty()
        && key_id != "."
        && key_id != ".."
        && key_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'));
    if safe {
        Ok(key_id)
    } else {
        Err(Error::new(
            ErrorKind::InvalidInput,
            "key_id must be a basename using only ASCII letters, digits, '.', '_', or '-'",
        ))
    }
}

/// Write a keypair: public key (base64 raw) to the committed path, private key
/// (base64 raw) to the gitignored path. Returns (pub_path, priv_path).
pub fn write_keypair(
    root: &std::path::Path,
    key_id: &str,
    secret: &SigningKey,
) -> std::io::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let root = root.canonicalize()?;
    let key_id = validate_key_id(key_id)?;
    let public_key = public_key_b64(&secret.verifying_key());
    let pub_path = crate::util::secure_write(
        &root,
        &[".agent-harness", "federation", "keys"],
        &format!("{key_id}.pub"),
        public_key.as_bytes(),
        0o644,
    )?;
    let private_key = B64.encode(secret.to_bytes());
    let priv_path = crate::util::secure_write(
        &root,
        &[".swe-seed", "federation", "keys"],
        &format!("{key_id}.key"),
        private_key.as_bytes(),
        0o600,
    )?;
    Ok((pub_path, priv_path))
}

/// Load a signing key from a private-key file. Accepts either:
/// - plaintext base64 (32 raw bytes), or
/// - a SOPS/age-encrypted file (decrypted via `sops -d` if `sops` is installed).
///
/// This makes production key management transparent: keygen writes plaintext
/// to gitignored `.swe-seed/`, `just federation-encrypt-key` SOPS-encrypts it
/// in place, and this loader decrypts on read.
pub fn load_signing_key(path: &std::path::Path) -> Result<SigningKey, LoadError> {
    let text = std::fs::read_to_string(path)
        .map_err(|_| LoadError::NotFound(path.display().to_string()))?;
    decode_key_bytes(text.trim(), path)
}

fn decode_key_bytes(text: &str, path: &std::path::Path) -> Result<SigningKey, LoadError> {
    // First try plaintext base64.
    if let Ok(bytes) = B64.decode(text.as_bytes()) {
        if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
            return Ok(SigningKey::from_bytes(&arr));
        }
    }
    // Otherwise assume SOPS-encrypted; decrypt via `sops -d`.
    let decrypted = std::process::Command::new("sops")
        .arg("-d")
        .arg(path)
        .output()
        .map_err(|e| LoadError::SopsUnavailable(e.to_string()))?;
    if !decrypted.status.success() {
        return Err(LoadError::SopsFailed(
            String::from_utf8_lossy(&decrypted.stderr).to_string(),
        ));
    }
    let plain = String::from_utf8_lossy(&decrypted.stdout);
    let bytes = B64
        .decode(plain.trim().as_bytes())
        .map_err(|_| LoadError::Malformed)?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| LoadError::Malformed)?;
    Ok(SigningKey::from_bytes(&arr))
}

#[derive(Debug)]
pub enum LoadError {
    NotFound(String),
    Malformed,
    SopsUnavailable(String),
    SopsFailed(String),
}
impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::NotFound(p) => write!(f, "private key not found: {p}"),
            LoadError::Malformed => write!(f, "malformed private key (expected base64 32 bytes)"),
            LoadError::SopsUnavailable(e) => {
                write!(f, "key is SOPS-encrypted but `sops` is unavailable: {e}")
            }
            LoadError::SopsFailed(e) => write!(f, "SOPS decrypt failed: {e}"),
        }
    }
}
impl std::error::Error for LoadError {}

/// Construct the canonical signing string for an envelope. `payload` must be
/// the payload *minus* any `signature` field (the verifier strips it before
/// calling this).
///
/// `namespace` is signed once, as the header line. In the v1 layout namespace
/// rides inside `payload`; we strip it from the payload-JSON portion so the
/// canonical bytes are identical whether namespace is carried top-level
/// (family A) or inside payload (v1) — the port does not invalidate any
/// committed signature/vector.
pub fn canonical_signing_string(
    namespace: &str,
    event_type: &str,
    occurred_at: &str,
    payload: &Value,
) -> String {
    // The federation contract is compact JSON with recursively sorted keys,
    // excluding the envelope's attached signature AND the namespace (which is
    // signed as the header line above).
    let mut p = sorted_without_signature(payload);
    if let Some(obj) = p.as_object_mut() {
        obj.remove("namespace");
    }
    let payload_json = serde_json::to_string(&p).unwrap_or_default();
    format!("{namespace}\n{event_type}\n{occurred_at}\n{payload_json}")
}

/// Compact JSON of a payload with recursively sorted keys and the `signature`
/// field removed. This is the canonical payload representation used by the
/// content-derived idempotency key (F-10). Unlike the signing string it
/// intentionally KEEPS `namespace` (idempotency is over full producer content,
/// matching SEA/GSA's `json.dumps(payload, sort_keys=True)`).
pub fn canonical_payload_json(payload: &Value) -> String {
    let p = sorted_without_signature(payload);
    serde_json::to_string(&p).unwrap_or_default()
}

fn sorted_without_signature(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();
            for key in keys {
                if key == "signature" {
                    continue;
                }
                sorted.insert(key.clone(), sorted_without_signature(&obj[key]));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted_without_signature).collect()),
        other => other.clone(),
    }
}

/// The signature attached to a signed envelope.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignaturePayload {
    pub algorithm: String,
    pub key_id: String,
    /// Base64 (standard) Ed25519 signature over the canonical signing string.
    pub value: String,
}

/// A 32-byte Ed25519 secret key, loaded from bytes (e.g. SOPS-decrypted).
pub fn signing_key_from_bytes(bytes: &[u8; 32]) -> SigningKey {
    SigningKey::from_bytes(bytes)
}

/// Sign the canonical signing string of `envelope` and return the attached
/// signature payload (algorithm, key_id, base64 signature value).
pub fn sign_envelope(envelope: &Envelope, secret: &SigningKey, key_id: &str) -> SignaturePayload {
    let namespace = envelope.namespace().unwrap_or("");
    let msg = canonical_signing_string(
        namespace,
        &envelope.event_type,
        &envelope.occurred_at,
        &envelope.payload,
    );
    let sig: Signature = secret.sign(msg.as_bytes());
    SignaturePayload {
        algorithm: SIGNING_ALGORITHM.into(),
        key_id: key_id.into(),
        value: B64.encode(sig.to_bytes()),
    }
}

/// Verify an envelope's signature against a public key. Recomputes the
/// canonical string (stripping the signature field) and checks the Ed25519
/// signature. `signature_b64` is the base64 signature value.
pub fn verify_envelope(
    envelope: &Envelope,
    signature_b64: &str,
    public_key: &VerifyingKey,
) -> Result<(), VerifyError> {
    let sig_bytes = B64
        .decode(signature_b64.as_bytes())
        .map_err(|_| VerifyError::MalformedSignature)?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|_| VerifyError::MalformedSignature)?;
    let namespace = envelope.namespace().unwrap_or("");
    let msg = canonical_signing_string(
        namespace,
        &envelope.event_type,
        &envelope.occurred_at,
        &envelope.payload,
    );
    public_key
        .verify_strict(msg.as_bytes(), &sig)
        .map_err(|_| VerifyError::InvalidSignature)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    MalformedSignature,
    InvalidSignature,
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::MalformedSignature => write!(f, "malformed signature (bad base64/length)"),
            VerifyError::InvalidSignature => write!(f, "invalid signature"),
        }
    }
}

impl std::error::Error for VerifyError {}

/// Encode a verifying (public) key as base64.
pub fn public_key_b64(pk: &VerifyingKey) -> String {
    B64.encode(pk.to_bytes())
}

/// Decode a base64 verifying (public) key.
pub fn public_key_from_b64(b64: &str) -> Result<VerifyingKey, VerifyError> {
    let bytes = B64
        .decode(b64.as_bytes())
        .map_err(|_| VerifyError::MalformedSignature)?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| VerifyError::MalformedSignature)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| VerifyError::MalformedSignature)
}
