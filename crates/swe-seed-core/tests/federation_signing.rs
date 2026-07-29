//! federation_signing: Ed25519 envelope signing + the canonical signing string.
//! The committed test vector (`tests/fixtures/federation-sign-vector.json`) is
//! the cross-repo parity lock — SEA-Forge's Python verifier MUST reproduce the
//! same signature from the same vector.

use swe_seed_core::federation::{
    canonical_signing_string, private_key_path, public_key_b64, public_key_from_b64,
    public_key_path, sign_envelope, signing_key_from_bytes, verify_envelope, write_keypair,
    Envelope, SIGNING_ALGORITHM,
};

fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn fixture() -> std::path::PathBuf {
    root().join("tests/fixtures/federation-sign-vector.json")
}

/// Fixed test-only secret key (32 × 0x01). NOT a production key; obviously a
/// test vector. base64 = AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE=... (32 ones).
fn test_secret() -> [u8; 32] {
    [0x01; 32]
}

fn envelope() -> Envelope {
    Envelope {
        schema_version: "v1".into(),
        event_id: "ignored-in-canonical".into(),
        source_agent: "swe-seed".into(),
        event_type: "ProofCompleted".into(),
        occurred_at: "2026-06-30T00:00:00+00:00".into(),
        idempotency_key: None,
        payload: serde_json::json!({
            "domain_model_hash": "deadbeef",
            "namespace": "agentic_capability_loop",
            "trace_chain_root": "cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d",
            "result": "pass",
        }),
        provenance: None,
    }
}

#[test]
fn canonical_string_is_stable_and_documented() {
    // The exact canonical form: 4 lines, sorted compact JSON payload, signature
    // AND namespace excluded from the payload JSON (namespace is the header
    // line). This is the contract both Rust and Python reproduce.
    let e = envelope();
    let s = canonical_signing_string(
        e.namespace().unwrap_or(""),
        &e.event_type,
        &e.occurred_at,
        &e.payload,
    );
    assert_eq!(
        s,
        "agentic_capability_loop\n\
         ProofCompleted\n\
         2026-06-30T00:00:00+00:00\n\
         {\"domain_model_hash\":\"deadbeef\",\"result\":\"pass\",\"trace_chain_root\":\"cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d\"}"
    );
}

#[test]
fn signature_round_trips() {
    let sec = signing_key_from_bytes(&test_secret());
    let e = envelope();
    let sig = sign_envelope(&e, &sec, "test-key-2026q3");
    assert_eq!(sig.algorithm, SIGNING_ALGORITHM);
    // Verify with the corresponding public key.
    let pk = sec.verifying_key();
    assert!(verify_envelope(&e, &sig.value, &pk).is_ok());
    // A tampered payload fails verification.
    let mut bad = envelope();
    bad.payload["result"] = serde_json::json!("fail");
    assert_eq!(
        verify_envelope(&bad, &sig.value, &pk),
        Err(swe_seed_core::federation::VerifyError::InvalidSignature)
    );
}

#[test]
fn public_key_round_trips_base64() {
    let sec = signing_key_from_bytes(&test_secret());
    let pk = sec.verifying_key();
    let b64 = public_key_b64(&pk);
    let back = public_key_from_b64(&b64).unwrap();
    assert_eq!(back.to_bytes(), pk.to_bytes());
}

#[test]
fn key_paths_reject_unsafe_key_ids() {
    let root = root();
    for key_id in ["../escape", "/abs", "a/b", "..", "", "bad key"] {
        assert!(
            public_key_path(&root, key_id).is_err(),
            "public key accepted {key_id:?}"
        );
        assert!(
            private_key_path(&root, key_id).is_err(),
            "private key accepted {key_id:?}"
        );
    }
    assert!(public_key_path(&root, "swe-seed_2026.q3").is_ok());
}

#[test]
fn write_keypair_rejects_unsafe_key_ids() {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-unsafe-key-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let secret = signing_key_from_bytes(&test_secret());
    for key_id in ["../escape", "/tmp/escape", "a/b", "..", "", "bad key"] {
        assert!(write_keypair(&dir, key_id, &secret).is_err());
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn write_keypair_uses_private_key_owner_only_permissions() {
    let dir = std::env::temp_dir().join(format!(
        "swe-seed-keypair-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let sec = signing_key_from_bytes(&test_secret());
    let (_pub_path, priv_path) = write_keypair(&dir, "test-key", &sec).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&priv_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn write_keypair_corrects_existing_private_key_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!(
        "swe-seed-existing-key-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = private_key_path(&dir, "test-key").unwrap();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "old-key").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    write_keypair(&dir, "test-key", &signing_key_from_bytes(&test_secret())).unwrap();

    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn write_keypair_rejects_private_key_symlinks() {
    use std::os::unix::fs::symlink;

    let dir = std::env::temp_dir().join(format!(
        "swe-seed-symlink-key-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = private_key_path(&dir, "test-key").unwrap();
    let target = dir.join("target.key");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&target, "outside").unwrap();
    symlink(&target, &path).unwrap();

    assert!(write_keypair(&dir, "test-key", &signing_key_from_bytes(&test_secret())).is_err());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "outside");
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn write_keypair_rejects_symlinked_private_key_directories() {
    use std::os::unix::fs::symlink;

    let dir = std::env::temp_dir().join(format!(
        "swe-seed-symlink-key-dir-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let keys = dir.join(".swe-seed/federation/keys");
    let target = dir.join("outside");
    std::fs::create_dir_all(keys.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&target).unwrap();
    symlink(&target, &keys).unwrap();

    assert!(write_keypair(&dir, "test-key", &signing_key_from_bytes(&test_secret())).is_err());
    assert!(!target.join("test-key.key").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn write_keypair_rejects_symlinked_public_key_directories() {
    use std::os::unix::fs::symlink;

    let dir = std::env::temp_dir().join(format!(
        "swe-seed-symlink-public-key-dir-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let keys = dir.join(".agent-harness/federation/keys");
    let target = dir.join("outside");
    std::fs::create_dir_all(keys.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&target).unwrap();
    symlink(&target, &keys).unwrap();

    assert!(write_keypair(&dir, "test-key", &signing_key_from_bytes(&test_secret())).is_err());
    assert!(!target.join("test-key.pub").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn canonical_string_sorts_nested_payload_keys_and_strips_signature() {
    let payload = serde_json::json!({
        "z": [{"b": 1, "a": 2}],
        "signature": {"value": "ignored"},
        "a": {"d": 4, "c": 3}
    });
    let s = canonical_signing_string("ns", "Event", "t", &payload);
    assert_eq!(
        s,
        r#"ns
Event
t
{"a":{"c":3,"d":4},"z":[{"a":2,"b":1}]}"#
    );
}

#[test]
fn trace_chain_root_requires_object_payload() {
    let mut envelope = Envelope {
        schema_version: "v1".into(),
        event_id: "e".into(),
        source_agent: "swe-seed".into(),
        event_type: "ProofCompleted".into(),
        occurred_at: "t".into(),
        idempotency_key: None,
        payload: serde_json::json!("not-object"),
        provenance: None,
    };
    assert!(envelope.with_trace_chain_root("abc").is_err());
}

/// The vector committed to `tests/fixtures/federation-sign-vector.json`. Both
/// this Rust test and SEA-Forge's Python verifier load it and must reproduce
/// the same signature.
#[derive(serde::Deserialize, serde::Serialize)]
struct VectorFile {
    secret_key_b64: String,
    public_key_b64: String,
    namespace: String,
    event_type: String,
    occurred_at: String,
    payload: serde_json::Value,
    canonical_signing_string: String,
    signature_b64: String,
    key_id: String,
}

#[test]
fn rust_matches_committed_test_vector() {
    let v: VectorFile = serde_json::from_str(&std::fs::read_to_string(fixture()).unwrap_or_else(|e| {
        panic!(
            "test vector missing: {}: {e}. \
             Run `cargo test -p swe-seed-core --test federation_signing regen_vector -- --ignored`.",
            fixture().display()
        )
    }))
    .unwrap();

    // Reproduce the secret key from the vector.
    let sec_bytes = base64_decode(&v.secret_key_b64);
    let sec = signing_key_from_bytes(&sec_bytes);
    // The public key derived from the secret must match the vector's public key.
    assert_eq!(
        public_key_b64(&sec.verifying_key()),
        v.public_key_b64,
        "public key drift"
    );

    // The canonical string must match.
    let canon = canonical_signing_string(&v.namespace, &v.event_type, &v.occurred_at, &v.payload);
    assert_eq!(canon, v.canonical_signing_string, "canonical string drift");

    // Rust's signature MUST equal the committed vector. Build a v1 envelope,
    // injecting the vector's namespace into the payload so `namespace()` finds
    // it; the signing string strips namespace from the payload JSON, so the
    // canonical bytes match the (pre-v1) vector exactly.
    let mut payload = v.payload.clone();
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("namespace".into(), serde_json::json!(v.namespace));
    }
    let e = Envelope {
        schema_version: "v1".into(),
        event_id: String::new(),
        source_agent: "swe-seed".into(),
        event_type: v.event_type.clone(),
        occurred_at: v.occurred_at.clone(),
        idempotency_key: None,
        payload,
        provenance: None,
    };
    let sig = sign_envelope(&e, &sec, &v.key_id);
    assert_eq!(
        sig.value, v.signature_b64,
        "signature drift — Rust ≠ committed vector"
    );
    // And it verifies with the vector's public key.
    let pk = public_key_from_b64(&v.public_key_b64).unwrap();
    assert!(verify_envelope(&e, &v.signature_b64, &pk).is_ok());
}

/// Regenerate the test vector. Ignored by default; run on purpose.
#[test]
#[ignore = "vector regenerator — run with --ignored after an intentional signing change"]
fn regen_vector() {
    let sec = signing_key_from_bytes(&test_secret());
    let e = envelope();
    let canon = canonical_signing_string(
        e.namespace().unwrap_or(""),
        &e.event_type,
        &e.occurred_at,
        &e.payload,
    );
    let sig = sign_envelope(&e, &sec, "test-key-2026q3");
    // The vector stores namespace separately and payload without namespace,
    // matching the pre-v1 shape; canonical_signing_string strips namespace
    // from the payload JSON, so the stored canonical string is stable.
    let mut stored_payload = e.payload.clone();
    if let Some(obj) = stored_payload.as_object_mut() {
        obj.remove("namespace");
    }
    let v = VectorFile {
        secret_key_b64: base64_encode(&test_secret()),
        public_key_b64: public_key_b64(&sec.verifying_key()),
        namespace: e.namespace().unwrap_or("").to_string(),
        event_type: e.event_type.clone(),
        occurred_at: e.occurred_at.clone(),
        payload: stored_payload,
        canonical_signing_string: canon,
        signature_b64: sig.value,
        key_id: "test-key-2026q3".into(),
    };
    let out = format!("{}\n", serde_json::to_string_pretty(&v).unwrap());
    std::fs::write(fixture(), out).unwrap();
    println!("wrote {}", fixture().display());
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    STANDARD.encode(bytes)
}
fn base64_decode(s: &str) -> [u8; 32] {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let v = STANDARD.decode(s).unwrap();
    v.as_slice().try_into().unwrap()
}
