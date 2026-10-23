//! Gateway stage-7 reload proof (spec 0020 §15): valid snapshot applies to
//! future requests; invalid reload keeps last-known-good; a snapshot taken
//! before reload is unaffected (in-flight isolation). Run: `cargo test gateway_reload`.

use swe_seed_core::gateway::{GatewayConfig, MCPServer, NamespaceError, SnapshotManager};

fn cfg(namespaces: &[&str]) -> GatewayConfig {
    let mut c = GatewayConfig::default();
    c.servers = namespaces
        .iter()
        .map(|ns| MCPServer {
            id: (*ns).into(),
            namespace: (*ns).into(),
            ..Default::default()
        })
        .collect();
    c
}

#[test]
fn install_validates_initial_config() {
    assert!(SnapshotManager::install(cfg(&["fs", "db"])).is_ok());
}

#[test]
fn install_rejects_invalid_initial_config() {
    // duplicate namespace → startup config failure
    let mut bad = cfg(&["fs"]);
    bad.servers.push(MCPServer {
        id: "fs2".into(),
        namespace: "fs".into(),
        ..Default::default()
    });
    assert!(SnapshotManager::install(bad).is_err());
}

#[test]
fn valid_reload_applies_to_future_requests() {
    let mgr = SnapshotManager::install(cfg(&["fs"])).unwrap();
    // take a snapshot before reload
    let before = mgr.current();
    assert_eq!(before.servers.len(), 1);

    mgr.reload(cfg(&["fs", "db"])).unwrap();
    let after = mgr.current();
    assert_eq!(after.servers.len(), 2, "future requests see the new snapshot");
    // the pre-reload snapshot is unaffected (in-flight isolation)
    assert_eq!(before.servers.len(), 1);
}

#[test]
fn invalid_reload_keeps_last_known_good() {
    let mgr = SnapshotManager::install(cfg(&["fs"])).unwrap();
    // valid reload first, so last-known-good advances
    mgr.reload(cfg(&["fs", "db"])).unwrap();
    assert_eq!(mgr.current().servers.len(), 2);

    // invalid reload: duplicate namespace → must keep current (last-known-good)
    let mut invalid = cfg(&["fs"]);
    invalid.servers.push(MCPServer {
        id: "x".into(),
        namespace: "fs".into(),
        ..Default::default()
    });
    let err = mgr.reload(invalid).unwrap_err();
    assert_eq!(err.reason_code(), "invalid_reload");
    // current snapshot unchanged
    assert_eq!(mgr.current().servers.len(), 2);
}

#[test]
fn rollback_restores_last_known_good() {
    let mgr = SnapshotManager::install(cfg(&["fs"])).unwrap();
    mgr.reload(cfg(&["fs", "db", "cache"])).unwrap();
    assert_eq!(mgr.current().servers.len(), 3);
    mgr.rollback();
    // last-known-good was the pre-reload current (1 server)
    assert_eq!(mgr.current().servers.len(), 1);
}

#[test]
fn namespace_validation_surfaces_duplicate_error() {
    let mut c = cfg(&["fs"]);
    c.servers.push(MCPServer {
        id: "fs2".into(),
        namespace: "fs".into(),
        ..Default::default()
    });
    assert_eq!(
        c.validate_namespaces(),
        Err(NamespaceError::Duplicate("fs".into()))
    );
}

#[test]
fn non_loopback_bind_without_tls_or_tunnel_is_rejected() {
    // spec 0020 §15: non-loopback requires TLS or tunnel mode.
    let mut c = cfg(&["fs"]);
    c.listener.bind = "0.0.0.0".into();
    // install fails closed
    let err = SnapshotManager::install(c.clone())
        .err()
        .expect("expected invalid config");
    assert_eq!(err.reason_code(), "invalid_reload");
    // reload of an unsafe snapshot into an existing manager also fails
    let mgr = SnapshotManager::install(cfg(&["fs"])).unwrap();
    let err = mgr.reload(c).err().expect("expected invalid reload");
    assert_eq!(err.reason_code(), "invalid_reload");
}

#[test]
fn non_loopback_bind_with_tls_is_accepted() {
    let mut c = cfg(&["fs"]);
    c.listener.bind = "0.0.0.0".into();
    c.listener.tls = true;
    assert!(SnapshotManager::install(c).is_ok());
}
