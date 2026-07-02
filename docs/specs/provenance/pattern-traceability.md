# Pattern Traceability Matrix

This committed matrix records high-level pattern provenance and confirms no code was copied.

| Source category | Pattern observed | SWE_Seed requirement | Code copied? | License concern? |
|---|---|---|---|---|
| gateway prior-art core | Single gateway over many tool backends | MCP gateway integration and compact discovery surface | No | No, MIT core |
| gateway prior-art core | Declarative capability definitions with hashing | SeedPackageManifest, ArtifactMetadata, and provenance hashing | No | No, MIT core |
| gateway prior-art core | Projection into host/client config | HostAdapter projection and drift detection | No | No, MIT core |
| gateway prior-art core | Doctor-style verification commands | EvalSpec, EvalCheck, ProofRecord, and doctor proof flow | No | No, MIT core |
| gateway prior-art EE list | Security modules existed in restricted files | EE files are out of bounds; derive security from public OWASP/MCP material | No | Yes, do not inspect |
| multi-host runtime prior-art | Per-host loaders and adapters | HostAdapter separation and SkillIR rendering | No | Yes, architecture only |
| multi-host runtime prior-art | AGENTS.md and rule discovery concepts | AGENTS.md operating contract and Rule/doctrine references | No | Yes, architecture only |
| multi-host runtime prior-art | Skill runtime versus portable distinction | SkillIR ingestion and render-target model | No | Yes, architecture only |
