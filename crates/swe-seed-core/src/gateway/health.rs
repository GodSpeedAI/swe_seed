//! Backend health, circuit breakers, metrics, and validated reload
//! (spec 0020 §7, §15, §17, §18). Per-backend `healthy`/`degraded`/`down` and
//! breaker `closed`/`open`/`half_open` bound retries and prevent unbounded
//! hammering of a failing backend. Reload validates a new snapshot before
//! applying it; an invalid snapshot keeps the last-known-good (spec 0020 §15).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use super::config::GatewayConfig;
use super::{BreakerState, GatewayError, HealthState};

/// Breaker thresholds. Defaults give a simple, predictable failure-driven cycle.
#[derive(Debug, Clone, Copy)]
pub struct BreakerConfig {
    pub failure_threshold: u32,
    pub cooldown: Duration,
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            cooldown: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BackendHealth {
    health: HealthState,
    breaker: BreakerState,
    consecutive_failures: u32,
    opened_at: Option<SystemTime>,
    /// Half-open single-probe slot: claimed by the first `can_forward` that
    /// enters half-open, released by `record_outcome` on transition out.
    probe_in_flight: bool,
}

impl BackendHealth {
    fn fresh() -> Self {
        Self {
            health: HealthState::Healthy,
            breaker: BreakerState::Closed,
            consecutive_failures: 0,
            opened_at: None,
            probe_in_flight: false,
        }
    }
}

/// Per-backend health + breaker book. `record_outcome` drives state transitions;
/// `can_forward` gates forwarding (the half-open probe is allowed once cooldown
/// elapses, then record_outcome closes or re-opens the breaker).
pub struct HealthBook {
    states: Mutex<HashMap<String, BackendHealth>>,
    cfg: BreakerConfig,
}

impl HealthBook {
    pub fn new(cfg: BreakerConfig) -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
            cfg,
        }
    }
    pub fn register(&self, backend_id: &str) {
        self.states
            .lock()
            .unwrap()
            .entry(backend_id.into())
            .or_insert_with(BackendHealth::fresh);
    }

    /// Record a forwarding outcome. Success closes a half-open breaker and
    /// clears failures; failure increments and may open the breaker (spec §18).
    pub fn record_outcome(&self, backend_id: &str, success: bool, now: SystemTime) {
        let mut states = self.states.lock().unwrap();
        let h = states.entry(backend_id.into()).or_insert_with(BackendHealth::fresh);
        let was_half_open = h.breaker == BreakerState::HalfOpen;
        if success {
            h.consecutive_failures = 0;
            h.breaker = BreakerState::Closed;
            h.opened_at = None;
            h.health = HealthState::Healthy;
            // Release the single-probe slot on success → closed.
            if was_half_open {
                h.probe_in_flight = false;
            }
        } else {
            h.consecutive_failures = h.consecutive_failures.saturating_add(1);
            if h.consecutive_failures >= self.cfg.failure_threshold || was_half_open {
                h.breaker = BreakerState::Open;
                h.opened_at = Some(now);
                h.health = HealthState::Down;
                // Release the single-probe slot on failure → re-open.
                if was_half_open {
                    h.probe_in_flight = false;
                }
            } else {
                h.health = HealthState::Degraded;
            }
        }
    }

    /// Gate: may we forward to this backend right now? An open breaker allows a
    /// single probe after cooldown (half-open); a closed breaker always allows.
    /// A blocked forward returns `false` (caller maps to BackendUnavailable).
    pub fn can_forward(&self, backend_id: &str, now: SystemTime) -> bool {
        let mut states = self.states.lock().unwrap();
        let h = match states.get_mut(backend_id) {
            Some(h) => h,
            None => return true, // unknown backend — let the router resolve/err
        };
        match h.breaker {
            BreakerState::Closed => true,
            BreakerState::Open => {
                let cooled = h
                    .opened_at
                    .map(|o| now.duration_since(o).map(|d| d >= self.cfg.cooldown).unwrap_or(false))
                    .unwrap_or(true);
                if cooled {
                    h.breaker = BreakerState::HalfOpen;
                    h.health = HealthState::Degraded;
                    h.probe_in_flight = true;
                    true // one probe
                } else {
                    false
                }
            }
            BreakerState::HalfOpen => {
                // Single-probe-at-a-time: the probe slot was claimed when the
                // breaker entered half-open. Block additional probes until
                // record_outcome closes or re-opens the breaker and releases
                // the slot. (The defensive claim below handles any future code
                // path that reaches half-open without claiming.)
                if h.probe_in_flight {
                    false
                } else {
                    h.probe_in_flight = true;
                    true
                }
            }
        }
    }

    /// Read-only health snapshot for operator inspection (spec 0020 §17).
    pub fn snapshot(&self) -> HashMap<String, (HealthState, BreakerState, u32)> {
        self.states
            .lock()
            .unwrap()
            .iter()
            .map(|(k, h)| (k.clone(), (h.health, h.breaker, h.consecutive_failures)))
            .collect()
    }
}

/// Per-backend metrics counters (spec 0020 §17).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendMetrics {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub latency_sum_us: u128,
}

/// Gateway-wide metrics: total + per-backend request/latency + active sessions.
pub struct Metrics {
    total_requests: AtomicU64,
    active_sessions: AtomicU64,
    per_backend: Mutex<HashMap<String, BackendMetrics>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            active_sessions: AtomicU64::new(0),
            per_backend: Mutex::new(HashMap::new()),
        }
    }
    pub fn inc_active_session(&self) {
        self.active_sessions.fetch_add(1, Ordering::SeqCst);
    }
    pub fn dec_active_session(&self) {
        self.active_sessions.fetch_sub(1, Ordering::SeqCst);
    }
    pub fn record(&self, backend_id: &str, success: bool, latency: Duration) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        let mut map = self.per_backend.lock().unwrap();
        let m = map.entry(backend_id.into()).or_default();
        m.requests += 1;
        if success {
            m.successes += 1;
        } else {
            m.failures += 1;
        }
        m.latency_sum_us += latency.as_micros();
    }
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::SeqCst),
            active_sessions: self.active_sessions.load(Ordering::SeqCst),
            per_backend: self.per_backend.lock().unwrap().clone(),
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub active_sessions: u64,
    pub per_backend: HashMap<String, BackendMetrics>,
}

/// Validated snapshot reload with last-known-good rollback (spec 0020 §15).
/// Reload parses + validates a new config before applying it to future
/// requests; an invalid reload preserves the current snapshot. In-flight
/// requests continue on the snapshot they started with.
pub struct SnapshotManager {
    current: RwLock<GatewayConfig>,
    last_known_good: RwLock<GatewayConfig>,
}

impl SnapshotManager {
    /// Initial install. The first config MUST validate (namespaces + listener
    /// exposure) — startup failure exits non-zero per spec §18; here we surface
    /// the typed error to the caller.
    pub fn install(initial: GatewayConfig) -> Result<Self, GatewayError> {
        initial.validate()?;
        Ok(Self {
            current: RwLock::new(initial.clone()),
            last_known_good: RwLock::new(initial),
        })
    }

    /// Validate + apply a new snapshot for FUTURE requests. On invalid reload
    /// keeps last-known-good and returns `InvalidReload` (spec 0020 §15).
    pub fn reload(&self, new_cfg: GatewayConfig) -> Result<(), GatewayError> {
        new_cfg.validate()?;
        let mut cur = self.current.write().unwrap();
        // prior current becomes last-known-good for future rollback
        *self.last_known_good.write().unwrap() = cur.clone();
        *cur = new_cfg;
        Ok(())
    }

    /// A consistent point-in-time snapshot for a request to run against.
    pub fn current(&self) -> GatewayConfig {
        self.current.read().unwrap().clone()
    }

    /// Roll back to the last-known-good snapshot (operator recovery).
    pub fn rollback(&self) {
        let good = self.last_known_good.read().unwrap().clone();
        *self.current.write().unwrap() = good;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn now() -> SystemTime {
        SystemTime::now()
    }

    #[test]
    fn breaker_opens_after_threshold_and_blocks() {
        let book = HealthBook::new(BreakerConfig {
            failure_threshold: 3,
            cooldown: Duration::from_secs(60),
        });
        book.register("fs");
        for _ in 0..3 {
            book.record_outcome("fs", false, now());
        }
        assert!(!book.can_forward("fs", now())); // open
        let snap = book.snapshot();
        assert_eq!(snap.get("fs").unwrap().1, BreakerState::Open);
    }

    #[test]
    fn breaker_half_open_after_cooldown_then_closes_on_success() {
        let book = HealthBook::new(BreakerConfig {
            failure_threshold: 2,
            cooldown: Duration::from_millis(10),
        });
        book.register("fs");
        book.record_outcome("fs", false, now());
        book.record_outcome("fs", false, now()); // open
        assert!(!book.can_forward("fs", now()));
        std::thread::sleep(Duration::from_millis(20));
        assert!(book.can_forward("fs", now())); // half-open probe allowed
        book.record_outcome("fs", true, now()); // success → closed
        assert_eq!(book.snapshot().get("fs").unwrap().1, BreakerState::Closed);
    }

    #[test]
    fn half_open_failure_reopens_breaker() {
        let book = HealthBook::new(BreakerConfig {
            failure_threshold: 1,
            cooldown: Duration::from_millis(5),
        });
        book.register("fs");
        book.record_outcome("fs", false, now()); // open (threshold 1)
        std::thread::sleep(Duration::from_millis(10));
        assert!(book.can_forward("fs", now())); // half-open
        book.record_outcome("fs", false, now()); // fail again → reopen
        assert_eq!(book.snapshot().get("fs").unwrap().1, BreakerState::Open);
    }

    #[test]
    fn success_in_closed_resets_degraded_state() {
        let book = HealthBook::new(BreakerConfig {
            failure_threshold: 3,
            cooldown: Duration::from_secs(60),
        });
        book.register("fs");
        book.record_outcome("fs", false, now()); // 1 failure → degraded
        assert_eq!(book.snapshot().get("fs").unwrap().0, HealthState::Degraded);
        book.record_outcome("fs", true, now()); // success → healthy, failures reset
        let snap = book.snapshot();
        let s = snap.get("fs").unwrap();
        assert_eq!(s.0, HealthState::Healthy);
        assert_eq!(s.2, 0);
    }

    #[test]
    fn metrics_record_per_backend() {
        let m = Metrics::new();
        m.inc_active_session();
        m.record("fs", true, Duration::from_micros(1500));
        m.record("fs", false, Duration::from_micros(500));
        m.dec_active_session();
        let snap = m.snapshot();
        assert_eq!(snap.total_requests, 2);
        assert_eq!(snap.active_sessions, 0);
        let b = snap.per_backend.get("fs").unwrap();
        assert_eq!(b.requests, 2);
        assert_eq!(b.successes, 1);
        assert_eq!(b.failures, 1);
        assert_eq!(b.latency_sum_us, 2000);
    }

    #[test]
    fn half_open_allows_single_probe_concurrently() {
        // During half-open, exactly ONE probe may be in flight: the first
        // concurrent caller claims the slot, all others are blocked.
        let book = HealthBook::new(BreakerConfig {
            failure_threshold: 2,
            cooldown: Duration::from_millis(10),
        });
        book.register("fs");
        book.record_outcome("fs", false, now());
        book.record_outcome("fs", false, now()); // open
        std::thread::sleep(Duration::from_millis(20)); // past cooldown
        let now = now();
        let trues = AtomicU64::new(0);
        std::thread::scope(|s| {
            for _ in 0..16 {
                s.spawn(|| {
                    if book.can_forward("fs", now) {
                        trues.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
        });
        let t = trues.load(Ordering::SeqCst);
        assert!(
            t <= 1,
            "half-open must allow at most one probe, got {t}"
        );
    }
}
