//! Redaction config loader for the gateway (spec 0020 §5, §11). The gateway
//! audit writer redacts every record before it touches disk. The redaction
//! policy comes from `.agent-hooks/config.yaml` `redaction:` section so the
//! gateway and the hooks runtime share ONE policy. When that file is absent
//! (standalone gateway, no agent-hooks install) we fall back to a conservative
//! built-in policy that still redacts the obvious secret-bearing keys and the
//! common high-signal token shapes — never zero redaction.

use std::path::Path;

use crate::config::{load_yaml, HooksConfig};
use crate::hooks::redact::RedactionConfig;

/// Conservative built-in redaction used when `.agent-hooks/config.yaml` is
/// absent. Mirrors the project's own committed `redaction:` section so a
/// standalone gateway still redacts the obvious secrets.
pub fn builtin_redaction() -> RedactionConfig {
    RedactionConfig {
        key_substrings: vec![
            "secret".into(),
            "token".into(),
            "password".into(),
            "api_key".into(),
            "authorization".into(),
            "cookie".into(),
        ],
        value_patterns: vec![
            "sk-[a-zA-Z0-9]{20,}".into(),
            "ghp_[a-zA-Z0-9]{36}".into(),
            "gho_[a-zA-Z0-9]{36}".into(),
            "xox[bpras]-[a-zA-Z0-9-]+".into(),
            "AKIA[0-9A-Z]{16}".into(),
            "-----BEGIN (?:RSA |EC |DSA )?PRIVATE KEY-----".into(),
        ],
    }
}

/// Load the redaction policy for gateway audit records. Reads
/// `.agent-hooks/config.yaml` when present; otherwise returns the conservative
/// built-in. A present-but-unparseable file is an error (fail closed on a
/// malformed policy rather than silently redacting less).
pub fn load_redaction(root: &Path) -> anyhow::Result<RedactionConfig> {
    let cfg_path = root.join(".agent-hooks/config.yaml");
    match load_yaml::<HooksConfig>(&cfg_path) {
        Ok(hooks_cfg) => Ok(RedactionConfig {
            key_substrings: hooks_cfg.redaction.key_substrings,
            value_patterns: hooks_cfg.redaction.value_patterns,
        }),
        Err(e) => {
            // Missing file ⇒ standalone gateway: use the built-in policy.
            if e.to_string().contains("No such file") || e.chain().any(|c| c.to_string().contains("No such file")) {
                Ok(builtin_redaction())
            } else {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_redaction_covers_secret_keys_and_token_shapes() {
        let cfg = builtin_redaction();
        assert!(cfg.key_substrings.iter().any(|s| s == "secret"));
        assert!(cfg.key_substrings.iter().any(|s| s == "token"));
        assert!(cfg.value_patterns.iter().any(|s| s.contains("ghp_")));
    }

    #[test]
    fn load_redaction_falls_back_to_builtin_when_config_absent() {
        let root = std::env::temp_dir().join(format!(
            "swe-seed-gw-redact-absent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        // No .agent-hooks/config.yaml here.
        let cfg = load_redaction(&root).unwrap();
        assert!(!cfg.key_substrings.is_empty(), "fallback must redact, not zero");
        assert!(!cfg.value_patterns.is_empty());
    }

    #[test]
    fn load_redaction_reads_agent_hooks_config_when_present() {
        let root = std::env::temp_dir().join(format!(
            "swe-seed-gw-redact-present-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join(".agent-hooks")).unwrap();
        std::fs::write(
            root.join(".agent-hooks/config.yaml"),
            "version: 1\nredaction:\n  key_substrings:\n    - secret\n    - customkey\n  value_patterns:\n    - \"CUSTOM-[0-9]+\"\n",
        )
        .unwrap();
        let cfg = load_redaction(&root).unwrap();
        assert!(cfg.key_substrings.iter().any(|s| s == "customkey"));
        assert!(cfg.value_patterns.iter().any(|s| s == "CUSTOM-[0-9]+"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn load_redaction_fails_closed_on_malformed_config() {
        let root = std::env::temp_dir().join(format!(
            "swe-seed-gw-redact-bad-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join(".agent-hooks")).unwrap();
        std::fs::write(root.join(".agent-hooks/config.yaml"), "version: : not yaml :\n  - [\n").unwrap();
        // A malformed policy must NOT silently fall back to less redaction.
        let res = load_redaction(&root);
        assert!(res.is_err(), "malformed redaction config must fail closed");
        std::fs::remove_dir_all(&root).ok();
    }
}
