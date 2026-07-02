//! Small shared helpers ported from the Python harness so route/trace output
//! stays format-compatible (timestamps, slugs, redaction).

use chrono::Utc;
use regex::Regex;

/// `2026-06-17T00:23:16+00:00` — `datetime.now(utc).replace(microsecond=0).isoformat()`.
pub fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
}

/// `20260617T002316Z` — `strftime('%Y%m%dT%H%M%SZ')`, used in trace ids.
pub fn utc_stamp() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

/// Lowercase, non-[a-z0-9] runs → `-`, trimmed, capped at 48 chars, default `trace`.
pub fn slugify(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut out = String::new();
    let mut last = false;
    for ch in lower.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            last = false;
        } else if !last {
            out.push('-');
            last = true;
        }
    }
    let trimmed = out.trim_matches('-');
    let capped = if trimmed.len() > 48 {
        &trimmed[..48]
    } else {
        trimmed
    };
    if capped.is_empty() {
        "trace".to_string()
    } else {
        capped.to_string()
    }
}

/// Replace known secret patterns with `[REDACTED]` (mirrors harness.redact_secrets).
pub fn redact_secrets(text: &str) -> String {
    const PATTERNS: &[&str] = &[
        r"sk-[a-zA-Z0-9]{20,}",
        r"ghp_[a-zA-Z0-9]{36}",
        r"gho_[a-zA-Z0-9]{36}",
        r"xox[bpras]-[a-zA-Z0-9-]+",
        r"AKIA[0-9A-Z]{16}",
        r"-----BEGIN (?:RSA |EC |DSA )?PRIVATE KEY-----",
    ];
    let mut out = text.to_string();
    for pat in PATTERNS {
        let re = Regex::new(pat).expect("static regex");
        out = re.replace_all(&out, "[REDACTED]").into_owned();
    }
    out
}
