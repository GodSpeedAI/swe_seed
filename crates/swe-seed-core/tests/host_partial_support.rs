use swe_seed_core::adapters::{adapter_for, AdapterSupport, CanonicalHookEvent, HostId};

#[test]
fn unsupported_runtime_capabilities_are_not_silent_success() {
    // Given: hosts with known weaker native mechanisms.
    let codex = adapter_for(HostId::Codex);
    let antigravity = adapter_for(HostId::Antigravity);
    let ci = adapter_for(HostId::Ci);

    // When: unsupported or advisory hooks are mapped.
    let codex_prompt = codex.support_for(CanonicalHookEvent::UserPromptSubmit);
    let antigravity_stop = antigravity.support_for(CanonicalHookEvent::Stop);
    let ci_prompt = ci.support_for(CanonicalHookEvent::UserPromptSubmit);

    // Then: they surface as PartialSupport or Unsupported with reasons.
    for support in [codex_prompt, antigravity_stop, ci_prompt] {
        match support {
            AdapterSupport::Partial { reason } | AdapterSupport::Unsupported { reason } => {
                assert!(!reason.trim().is_empty());
            }
            AdapterSupport::Full => panic!("weak runtime support must not be reported as full"),
        }
    }
}
