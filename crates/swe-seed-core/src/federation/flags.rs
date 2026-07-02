//! Federation config flags + the authority delegation gate (spec 0011 §2, §5).
//! `enabled=false` (the default) forces every sub-mode to behave as `local`/`off`
//! — the master switch wins. All external delegation is behind a flag.

use serde::{Deserialize, Serialize};

/// Authority plane mode (SEA-Forge).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthorityMode {
    #[default]
    Local,
    Delegate,
    Hybrid,
}

/// Context plane mode (Context Kernel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContextMode {
    #[default]
    Local,
    External,
    Hybrid,
}

/// Settlement plane mode (GodSpeed-Agent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SettlementMode {
    #[default]
    Off,
    Emit,
    Consume,
    Both,
}

/// Action risk tagging (drives authority fail-closed behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    Low,
    Medium,
    High,
    Dangerous,
}

impl Risk {
    pub fn at_least_high(self) -> bool {
        matches!(self, Risk::High | Risk::Dangerous)
    }
    pub fn is_dangerous(self) -> bool {
        matches!(self, Risk::Dangerous)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuthorityConfig {
    #[serde(default)]
    pub mode: AuthorityMode,
    /// If false (default), a local `deny` is never overridden by an external
    /// `allow` (spec 0011 §security).
    #[serde(default)]
    pub allow_external_override: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ContextConfig {
    #[serde(default)]
    pub mode: ContextMode,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SettlementConfig {
    #[serde(default)]
    pub mode: SettlementMode,
}

/// Federation configuration. Default = fully standalone (master switch off).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FederationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub emit_envelope: bool,
    #[serde(default)]
    pub consume_envelope: bool,
    #[serde(default)]
    pub authority: AuthorityConfig,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub settlement: SettlementConfig,
}

/// The fully-standalone config: master switch off, every plane local/off.
pub fn standalone() -> FederationConfig {
    FederationConfig::default()
}

impl FederationConfig {
    /// Master switch wins: with `enabled=false` every plane is local/off.
    pub fn authority_mode(&self) -> AuthorityMode {
        if self.enabled {
            self.authority.mode
        } else {
            AuthorityMode::Local
        }
    }
    pub fn context_mode(&self) -> ContextMode {
        if self.enabled {
            self.context.mode
        } else {
            ContextMode::Local
        }
    }
    pub fn settlement_mode(&self) -> SettlementMode {
        if self.enabled {
            self.settlement.mode
        } else {
            SettlementMode::Off
        }
    }
    /// Envelopes are emitted only when the master switch AND emit_envelope are on.
    pub fn emits(&self) -> bool {
        self.enabled && self.emit_envelope
    }
    /// Envelopes are consumed only when the master switch AND consume_envelope are on.
    pub fn consumes(&self) -> bool {
        self.enabled && self.consume_envelope
    }
    /// True iff the config can attempt ANY external interaction.
    pub fn is_standalone(&self) -> bool {
        !self.emits()
            && !self.consumes()
            && self.authority_mode() == AuthorityMode::Local
            && self.context_mode() == ContextMode::Local
            && self.settlement_mode() == SettlementMode::Off
    }
}

/// The verdict carried by a consumed `AuthorityChecked` envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityVerdict {
    Allow,
    Deny,
    Escalate,
}

/// Outcome of the authority gate for a proposed action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateOutcome {
    Allow,
    Deny,
    Escalate,
    /// A delegate/hybrid call timed out and the action needs a human decision.
    NeedsHuman,
}

/// The authority delegation gate (spec 0011 §5).
///
/// - `local` (default / master switch off): the local decision is sole; incoming
///   envelopes are ignored.
/// - `delegate`: wait for an `AuthorityChecked` envelope. `deny` blocks,
///   `escalate` escalates, `allow` proceeds. A timeout (no envelope) is
///   fail-closed for `Dangerous` actions, else falls back to local.
/// - `hybrid`: local decision first; on a local deny, or a high-risk allow,
///   consult the envelope.
///
/// A local `deny` is never overridden by an external `allow` unless
/// `allow_external_override` is set.
pub fn authority_gate(
    cfg: &FederationConfig,
    risk: Risk,
    local_allows: bool,
    incoming: Option<AuthorityVerdict>,
) -> GateOutcome {
    use AuthorityMode::*;
    use AuthorityVerdict as V;
    use GateOutcome as G;

    let mode = cfg.authority_mode();

    // Local deny is sticky unless explicitly overridable.
    let local = if local_allows { G::Allow } else { G::Deny };
    let override_local_deny = cfg.authority.allow_external_override;

    match mode {
        Local => local,
        Delegate | Hybrid => {
            // Hybrid: a clean local allow on low/medium risk need not consult.
            if mode == Hybrid && local == G::Allow && !risk.at_least_high() {
                return G::Allow;
            }
            // Local deny sticks unless overridable + an external allow arrives.
            if local == G::Deny {
                if override_local_deny && incoming == Some(V::Allow) {
                    return G::Allow;
                }
                return G::Deny;
            }
            // Local allows; consult the envelope.
            match incoming {
                Some(V::Allow) => G::Allow,
                Some(V::Deny) => G::Deny,
                Some(V::Escalate) => G::Escalate,
                None => {
                    // Timeout: fail-closed for dangerous, else local fallback.
                    if risk.is_dangerous() {
                        G::Deny
                    } else {
                        local
                    }
                }
            }
        }
    }
}
