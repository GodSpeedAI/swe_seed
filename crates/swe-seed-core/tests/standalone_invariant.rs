//! standalone_invariant: with `federation.enabled=false` and the SEA
//! unreachable, SWE_Seed is fully standalone — zero external calls are made
//! (no envelope dispatched, no sink touched), the authority gate resolves
//! locally, and the inner stack still functions (spec 0011, hard invariant).

use swe_seed_core::federation::{
    dispatch, emit_work_requested, resolve_from, Dispatch, FederationConfig, HashSource,
};

#[test]
fn default_config_is_fully_standalone() {
    let cfg = FederationConfig::default();
    assert!(!cfg.enabled);
    assert!(!cfg.emits());
    assert!(!cfg.consumes());
    assert!(cfg.is_standalone(), "default config must be standalone");
}

#[test]
fn disabled_federation_dispatches_nothing() {
    // The hard invariant: with federation off, dispatch is a no-op and the sink
    // is never touched (zero external calls). A sink file written here would be
    // an invariant violation.
    let cfg = FederationConfig::default();
    assert!(!cfg.emits());

    let sink = std::env::temp_dir().join(format!(
        "swe-seed-fed-sink-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!sink.exists(), "precondition: sink absent");

    let resolved = resolve_from(None, None, false); // SEA unreachable → fallback
    let envelope = emit_work_requested(&resolved.hash, "wr", "actor", "op", "res", "low", None);
    let outcome = dispatch(&cfg, &envelope, Some(&sink));

    assert_eq!(
        outcome,
        Dispatch::Suppressed,
        "disabled federation must not dispatch"
    );
    assert!(
        !sink.exists(),
        "disabled federation must not touch the sink (zero external calls)"
    );
}

#[test]
fn dead_sea_root_resolves_to_fallback_without_panic() {
    // SEA_ROOT pointing at a nonexistent path must not break resolution or the
    // inner loop — it falls back (with warn). This is the standalone contract.
    let resolved = resolve_from(Some("/nonexistent"), None, false);
    assert_eq!(resolved.source, HashSource::Fallback);
    assert!(resolved.warned);
    // The fallback hash is the documented standalone value.
    assert_eq!(
        resolved.hash,
        "dc144cbd71a483431301ca1bf32e95c014af4edba8dbcc525f505310e30a107e"
    );
}

#[test]
fn enabled_but_emit_off_still_dispatches_nothing() {
    // Even with the master switch on, emit_envelope=false → no dispatch.
    let mut cfg = FederationConfig::default();
    cfg.enabled = true;
    assert!(!cfg.emits(), "emit_envelope gates dispatch independently");

    let sink = std::env::temp_dir().join(format!(
        "swe-seed-fed-sink2-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let envelope = emit_work_requested("h", "wr", "a", "o", "r", "low", None);
    assert_eq!(dispatch(&cfg, &envelope, Some(&sink)), Dispatch::Suppressed);
    assert!(!sink.exists());
}

#[test]
fn inner_stack_works_with_federation_off_and_sea_unreachable() {
    // The inner stack (seed/eval/doctor concepts) has no external dependency.
    // Prove the federation boundary doesn't entangle it: a standalone run
    // resolves a hash, builds an envelope in memory, and suppresses dispatch —
    // the inner stack's inputs/outputs are untouched.
    use swe_seed_core::seed::assemble_default;

    // Inner-stack artifacts are constructed purely locally.
    let manifest = assemble_default();
    assert!(
        !manifest.capabilities.is_empty(),
        "inner stack must produce capabilities standalone"
    );

    let cfg = FederationConfig::default();
    let resolved = resolve_from(Some("/nonexistent"), None, false);
    let envelope = emit_work_requested(&resolved.hash, "wr", "actor", "route", "task", "low", None);

    // The envelope is never dispatched under the default (standalone) config.
    assert_eq!(dispatch(&cfg, &envelope, None), Dispatch::Suppressed);
    assert!(cfg.is_standalone());
}
