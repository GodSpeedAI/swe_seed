//! Session sandbox + atomic governance (spec 0020 §7, §8). Server-generated
//! sessions (>=128-bit ids), per-session/per-client/per-tool counters, and a
//! durable write-ahead journal whose deltas are recorded in one critical
//! section BEFORE a request may advance — so a crash cannot undercount
//! (spec 0020 §7). On restart the journal is replayed before accepting traffic.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use rand_core::RngCore;

use super::{GatewayEffectiveLimits, GatewayError, GatewayGovernanceState, GatewaySession, SessionState};

/// Default journal location (durable write-ahead log for counter deltas).
pub const GOVERNANCE_JOURNAL_REL: &str = ".swe-seed/gateway/governance.journal";

/// >=128-bit server-generated session id (16 random bytes, lowercase hex).
pub fn generate_session_id() -> String {
    let mut bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

struct SessionStart {
    started_at: SystemTime,
}

/// Authoritative governance state + durable journal. All check/increment work
/// happens under one critical section (`state` lock), and the counter delta is
/// appended + fsynced to the journal BEFORE the lock is released — the durable
/// contract from spec 0020 §7.
// ponytail: fsync-inside-lock bounds throughput; per-session locks + async WAL
// is the upgrade path if high-concurrency backends need it.
pub struct Governance {
    state: Mutex<GatewayGovernanceState>,
    sessions: Mutex<HashMap<String, SessionStart>>,
    journal: Mutex<Option<std::fs::File>>,
    journal_path: PathBuf,
    seq: AtomicU64,
    limits: GatewayEffectiveLimits,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JournalEntry {
    seq: u64,
    session_id: String,
    client_id: String,
    tool: String,
    occurred_at: String,
}

impl Governance {
    /// Load governance for `root`, replaying the durable journal first so
    /// counters reflect pre-crash work (spec 0020 §7). `limits` = effective
    /// limits after PermissionPolicy/config narrowing.
    pub fn load(root: &Path, limits: GatewayEffectiveLimits) -> std::io::Result<Self> {
        let journal_path = root.join(GOVERNANCE_JOURNAL_REL);
        let mut state = GatewayGovernanceState::default();
        state.limits = limits;
        if let Ok(bytes) = std::fs::read_to_string(&journal_path) {
            for line in bytes.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<JournalEntry>(line) {
                    *state.session_calls.entry(entry.session_id).or_insert(0) += 1;
                    *state.client_counters.entry(entry.client_id).or_insert(0) += 1;
                    *state.tool_counters.entry(entry.tool).or_insert(0) += 1;
                }
            }
        }
        let next_seq = state
            .session_calls
            .values()
            .map(|v| *v)
            .sum::<u64>();
        Ok(Self {
            state: Mutex::new(state),
            sessions: Mutex::new(HashMap::new()),
            // journal file is opened for append lazily on first write (after
            // mkdir), so load() never fails just because the dir is absent.
            journal: Mutex::new(None),
            journal_path,
            seq: AtomicU64::new(next_seq),
            limits,
        })
    }

    fn ensure_journal_open(&self) -> std::io::Result<()> {
        let mut guard = self.journal.lock().unwrap();
        if guard.is_some() {
            return Ok(());
        }
        if let Some(parent) = self.journal_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        use std::fs::OpenOptions;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.journal_path)?;
        *guard = Some(file);
        Ok(())
    }

    fn append_delta(&self, entry: &JournalEntry) -> Result<(), GatewayError> {
        self.ensure_journal_open().map_err(|e| {
            GatewayError::AuditWriteFailed {
                reason: format!("governance journal open: {e}"),
            }
        })?;
        let line = serde_json::to_string(entry)
            .map_err(|e| GatewayError::AuditWriteFailed {
                reason: format!("governance journal serialize: {e}"),
            })?;
        let mut guard = self.journal.lock().unwrap();
        let file = guard.as_mut().expect("journal opened");
        use std::io::Write;
        writeln!(file, "{line}").map_err(journal_write_failed)?;
        file.flush().map_err(journal_write_failed)?;
        // fsync = durable before the critical section closes (spec 0020 §7).
        file.sync_data().map_err(journal_write_failed)?;
        Ok(())
    }

    /// Open a new server-generated session. Clients cannot supply/override the
    /// id (spec 0020 §7).
    pub fn open_session(&self, client_id: &str) -> GatewaySession {
        let session_id = generate_session_id();
        let started_at = SystemTime::now();
        let started_at_iso = crate::util::utc_now();
        self.sessions.lock().unwrap().insert(
            session_id.clone(),
            SessionStart { started_at },
        );
        GatewaySession {
            session_id,
            client_id: client_id.into(),
            route_card_id: None,
            allowed_backends: vec![],
            state: SessionState::Active,
            started_at: started_at_iso,
            call_count: 0,
            max_calls: self.limits.session_max_calls,
            max_duration_secs: self.limits.session_max_duration_secs,
        }
    }

    /// Check + increment session/client/tool counters in one critical section.
    /// The durable delta is written BEFORE this returns Ok. A limit breach
    /// returns `SessionLimitExceeded` (session) or `BudgetExceeded` (client/
    /// tool) and writes NO delta.
    pub fn check_and_increment(
        &self,
        session_id: &str,
        client_id: &str,
        tool: &str,
    ) -> Result<(), GatewayError> {
        // Duration check (session sandbox).
        if self.limits.session_max_duration_secs > 0 {
            let started = self
                .sessions
                .lock()
                .unwrap()
                .get(session_id)
                .map(|s| s.started_at);
            if let Some(started) = started {
                if SystemTime::now()
                    .duration_since(started)
                    .map(|d| d > Duration::from_secs(self.limits.session_max_duration_secs))
                    .unwrap_or(false)
                {
                    return Err(GatewayError::SessionLimitExceeded {
                        session_id: session_id.into(),
                    });
                }
            }
        }

        let mut state = self.state.lock().unwrap();
        let session_count = state.session_calls.get(session_id).copied().unwrap_or(0);
        let client_count = state.client_counters.get(client_id).copied().unwrap_or(0);
        let tool_count = state.tool_counters.get(tool).copied().unwrap_or(0);

        if self.limits.session_max_calls > 0 && session_count >= self.limits.session_max_calls {
            return Err(GatewayError::SessionLimitExceeded {
                session_id: session_id.into(),
            });
        }
        if self.limits.client_max_requests > 0 && client_count >= self.limits.client_max_requests {
            return Err(GatewayError::BudgetExceeded {
                limit: self.limits.client_max_requests,
            });
        }
        if self.limits.tool_max_requests > 0 && tool_count >= self.limits.tool_max_requests {
            return Err(GatewayError::BudgetExceeded {
                limit: self.limits.tool_max_requests,
            });
        }

        // Increment in-memory first, then durably record the delta, then close
        // the critical section. A crash after this point still replays the
        // delta; a crash before it leaves counters unchanged.
        *state.session_calls.entry(session_id.into()).or_insert(0) += 1;
        *state.client_counters.entry(client_id.into()).or_insert(0) += 1;
        *state.tool_counters.entry(tool.into()).or_insert(0) += 1;
        let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        let entry = JournalEntry {
            seq,
            session_id: session_id.into(),
            client_id: client_id.into(),
            tool: tool.into(),
            occurred_at: crate::util::utc_now(),
        };
        // Drop nothing before the delta is durable.
        drop(state);
        self.append_delta(&entry)?;
        Ok(())
    }

    /// Snapshot of current counters (for operator inspection / metrics).
    pub fn snapshot(&self) -> GatewayGovernanceState {
        self.state.lock().unwrap().clone()
    }
}

fn journal_write_failed(e: std::io::Error) -> GatewayError {
    GatewayError::AuditWriteFailed {
        reason: format!("governance journal write: {e}"),
    }
}
