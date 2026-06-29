//! baml_parity: every hand-written swe_seed-layer Rust type matches its
//! `.baml` counterpart (field names **and types**, plus enum variants). Drift
//! fails CI (spec 0019).
//!
//! Coverage policy:
//! - swe_seed.baml: every class/enum MUST be registered (hard assert).
//! - harness/fabricator.baml: unmapped types are pending later phases. The gap
//!   set is asserted to live only in those two sources (so a surprise source
//!   or a dropped swe_seed type fails here), and the human-readable list is
//!   printed for visibility.

use std::collections::HashMap;
use std::path::PathBuf;

use swe_seed_core::contracts::{parse_baml_dir, BamlParity, BamlShape, BamlType};
use swe_seed_core::context::{ContextBudget, ContextPack};
use swe_seed_core::eval::{
    EvalCheck, EvalCheckResult, EvalClass, EvalResult, EvalSpec, EvalStatus, ProofRecord, SourceRef,
};
use swe_seed_core::fabricator::{
    AgentTask, EARSPattern, EARSRequirement, FabricatorArtifactStatus, FabricatorEvalCheck,
    FabricatorEvalClass, FabricatorEvalSpec, FabricatorProofRecord, FabricatorSourceRef,
    GherkinScenario, JobStory, PRD, ProductADR, ProductHypothesis, ProductSeed, SDS,
    SDSComponent, SemanticChainValidationReport, TDDPlan, TraceabilityLink, YStatement,
};
use swe_seed_core::hooks::{HookPolicy, PermissionPolicy};
use swe_seed_core::learning::{
    AdaptationDecision, LearningCandidate, LearningDisposition, LearningRecord, ReflectionTemplate,
    RegressionCase, SkillProposal,
};
use swe_seed_core::route::RouteCard;
use swe_seed_core::seed::{
    ArtifactMetadata, BoundaryFinding, BoundaryReport, LayerCapability, LayerName,
    ProjectSeed, ReviewRequirement, SeedArtifactStatus, SeedNeed, SeedPackageManifest,
    SeedRegenerationInput, SeedRegenerationPlan, SeedSourceRef, SeedValidationRequirement,
};
use swe_seed_core::skill::SkillIR;
use swe_seed_core::trace::TraceSchema;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn parse() -> Vec<BamlType> {
    parse_baml_dir(&root().join(".agent-harness/baml/baml_src")).expect("parse baml dir")
}

/// Fail fast if two parsed types share a name (across or within files). A name
/// collision must surface instead of silently overwriting a `BamlType` in a
/// name-keyed map and masking a mismatch.
fn assert_unique_names(types: &[BamlType]) {
    let mut seen: HashMap<&str, &str> = HashMap::new();
    for t in types {
        if let Some(prev) = seen.insert(t.name.as_str(), t.source_file.as_str()) {
            panic!(
                "duplicate .baml type name '{}' in both {} and {}",
                t.name, prev, t.source_file
            );
        }
    }
}

/// The full set of Rust types currently registered for parity.
fn registered() -> Vec<(&'static str, BamlShape)> {
    vec![
        (LayerName::baml_name(), LayerName::baml_shape()),
        (SeedArtifactStatus::baml_name(), SeedArtifactStatus::baml_shape()),
        (ReviewRequirement::baml_name(), ReviewRequirement::baml_shape()),
        (SeedSourceRef::baml_name(), SeedSourceRef::baml_shape()),
        (
            SeedValidationRequirement::baml_name(),
            SeedValidationRequirement::baml_shape(),
        ),
        (SeedNeed::baml_name(), SeedNeed::baml_shape()),
        (ArtifactMetadata::baml_name(), ArtifactMetadata::baml_shape()),
        (ProjectSeed::baml_name(), ProjectSeed::baml_shape()),
        (LayerCapability::baml_name(), LayerCapability::baml_shape()),
        (
            SeedPackageManifest::baml_name(),
            SeedPackageManifest::baml_shape(),
        ),
        (BoundaryFinding::baml_name(), BoundaryFinding::baml_shape()),
        (BoundaryReport::baml_name(), BoundaryReport::baml_shape()),
        (
            SeedRegenerationInput::baml_name(),
            SeedRegenerationInput::baml_shape(),
        ),
        (
            SeedRegenerationPlan::baml_name(),
            SeedRegenerationPlan::baml_shape(),
        ),
        (RouteCard::baml_name(), RouteCard::baml_shape()),
        (TraceSchema::baml_name(), TraceSchema::baml_shape()),
        (EvalClass::baml_name(), EvalClass::baml_shape()),
        (EvalStatus::baml_name(), EvalStatus::baml_shape()),
        (SourceRef::baml_name(), SourceRef::baml_shape()),
        (EvalCheck::baml_name(), EvalCheck::baml_shape()),
        (EvalSpec::baml_name(), EvalSpec::baml_shape()),
        (EvalCheckResult::baml_name(), EvalCheckResult::baml_shape()),
        (EvalResult::baml_name(), EvalResult::baml_shape()),
        (ProofRecord::baml_name(), ProofRecord::baml_shape()),
        (ContextBudget::baml_name(), ContextBudget::baml_shape()),
        (ContextPack::baml_name(), ContextPack::baml_shape()),
        (HookPolicy::baml_name(), HookPolicy::baml_shape()),
        (PermissionPolicy::baml_name(), PermissionPolicy::baml_shape()),
        (SkillIR::baml_name(), SkillIR::baml_shape()),
        (
            LearningDisposition::baml_name(),
            LearningDisposition::baml_shape(),
        ),
        (LearningRecord::baml_name(), LearningRecord::baml_shape()),
        (
            ReflectionTemplate::baml_name(),
            ReflectionTemplate::baml_shape(),
        ),
        (LearningCandidate::baml_name(), LearningCandidate::baml_shape()),
        (SkillProposal::baml_name(), SkillProposal::baml_shape()),
        (RegressionCase::baml_name(), RegressionCase::baml_shape()),
        (
            AdaptationDecision::baml_name(),
            AdaptationDecision::baml_shape(),
        ),
        // Fabricator layer (spec 0017).
        (
            FabricatorArtifactStatus::baml_name(),
            FabricatorArtifactStatus::baml_shape(),
        ),
        (
            FabricatorEvalClass::baml_name(),
            FabricatorEvalClass::baml_shape(),
        ),
        (EARSPattern::baml_name(), EARSPattern::baml_shape()),
        (
            FabricatorSourceRef::baml_name(),
            FabricatorSourceRef::baml_shape(),
        ),
        (TraceabilityLink::baml_name(), TraceabilityLink::baml_shape()),
        (JobStory::baml_name(), JobStory::baml_shape()),
        (ProductSeed::baml_name(), ProductSeed::baml_shape()),
        (
            ProductHypothesis::baml_name(),
            ProductHypothesis::baml_shape(),
        ),
        (EARSRequirement::baml_name(), EARSRequirement::baml_shape()),
        (YStatement::baml_name(), YStatement::baml_shape()),
        (ProductADR::baml_name(), ProductADR::baml_shape()),
        (SDSComponent::baml_name(), SDSComponent::baml_shape()),
        (GherkinScenario::baml_name(), GherkinScenario::baml_shape()),
        (PRD::baml_name(), PRD::baml_shape()),
        (SDS::baml_name(), SDS::baml_shape()),
        (TDDPlan::baml_name(), TDDPlan::baml_shape()),
        (AgentTask::baml_name(), AgentTask::baml_shape()),
        (
            FabricatorEvalCheck::baml_name(),
            FabricatorEvalCheck::baml_shape(),
        ),
        (
            FabricatorEvalSpec::baml_name(),
            FabricatorEvalSpec::baml_shape(),
        ),
        (
            FabricatorProofRecord::baml_name(),
            FabricatorProofRecord::baml_shape(),
        ),
        (
            SemanticChainValidationReport::baml_name(),
            SemanticChainValidationReport::baml_shape(),
        ),
    ]
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

fn assert_shape_match(name: &str, shape: &BamlShape, baml: &BamlType) {
    match shape {
        BamlShape::Enum { variants } => {
            let want = sorted(variants.iter().map(|s| s.to_string()).collect());
            let got = sorted(baml.variants.clone());
            assert_eq!(want, got, "variant drift for enum {name}");
        }
        BamlShape::Class {
            fields,
            field_types,
        } => {
            assert_eq!(
                fields.len(),
                field_types.len(),
                "shape decl for {name}: fields/field_types length mismatch"
            );
            let mut want: Vec<(String, String)> = fields
                .iter()
                .zip(field_types.iter())
                .map(|(f, t)| (f.to_string(), t.to_string()))
                .collect();
            want.sort();
            let mut got: Vec<(String, String)> = baml
                .fields
                .iter()
                .zip(baml.field_types.iter())
                .map(|(f, t)| (f.to_string(), t.to_string()))
                .collect();
            got.sort();
            assert_eq!(want, got, "field/type drift for class {name}");
        }
    }
}

#[test]
fn baml_parity_registered_match_baml() {
    let types = parse();
    assert_unique_names(&types);

    // Every registered type is matched against its .baml counterpart across ALL
    // files (swe_seed + harness + fabricator), not just one layer.
    let all: HashMap<&str, &BamlType> = types.iter().map(|t| (t.name.as_str(), t)).collect();

    let reg_list = registered();
    // Registered baml names must be unique; a silent HashMap overwrite would
    // otherwise hide a missing parity impl.
    let mut seen = std::collections::HashSet::new();
    for (n, _) in &reg_list {
        assert!(
            seen.insert(*n),
            "duplicate baml_name '{n}' returned by registered()"
        );
    }

    for (name, shape) in &reg_list {
        let baml = all
            .get(*name)
            .unwrap_or_else(|| panic!("registered type {name} has no .baml counterpart"));
        assert_shape_match(name, shape, baml);
    }
}

#[test]
fn baml_parity_swe_seed_layer_fully_covered() {
    let types = parse();
    assert_unique_names(&types);
    let by_file: HashMap<&str, Vec<&BamlType>> = {
        let mut m: HashMap<&str, Vec<&BamlType>> = HashMap::new();
        for t in &types {
            m.entry(t.source_file.as_str()).or_default().push(t);
        }
        m
    };
    let swe: HashMap<&str, &BamlType> = by_file["swe_seed"]
        .iter()
        .map(|t| (t.name.as_str(), *t))
        .collect();
    let reg: std::collections::HashSet<&str> =
        registered().into_iter().map(|(n, _)| n).collect();

    // Every swe_seed.baml class/enum must be registered (no coverage gap).
    let unregistered: Vec<&str> = swe.keys().filter(|n| !reg.contains(*n)).copied().collect();
    assert!(
        unregistered.is_empty(),
        "swe_seed.baml types without a Rust parity impl: {unregistered:?}"
    );
}

#[test]
fn baml_parity_parser_reads_all_files() {
    let types = parse();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for t in &types {
        *counts.entry(t.source_file.as_str()).or_default() += 1;
    }
    // Pinned counts: the .baml files are frozen inputs (spec 0019), so a
    // partial-parse regression changes one of these numbers and fails here.
    assert_eq!(counts.get("swe_seed").copied().unwrap_or(0), 14, "swe_seed.baml");
    assert_eq!(counts.get("harness").copied().unwrap_or(0), 29, "harness.baml");
    assert_eq!(counts.get("fabricator").copied().unwrap_or(0), 26, "fabricator.baml");
}

#[test]
fn baml_parity_dir_io_error_propagates() {
    // parse_baml_dir surfaces filesystem failures rather than masking them as
    // an empty (falsely-valid) schema set.
    let missing = root().join("definitely/not/a/dir");
    assert!(parse_baml_dir(&missing).is_err());
}

#[test]
fn baml_parity_coverage_gaps() {
    // Coverage gaps are spec-mandated to "warn" (spec 0019), but enforce that
    // they live only in the not-yet-implemented harness/fabricator layers: a
    // swe_seed gap or an unexpected source here fails instead of being missed.
    let types = parse();
    let reg: std::collections::HashSet<&str> = registered().into_iter().map(|(n, _)| n).collect();
    let mut gaps: Vec<String> = Vec::new();
    for t in &types {
        if t.source_file != "swe_seed" && !reg.contains(t.name.as_str()) {
            gaps.push(format!("{}:{}", t.source_file, t.name));
        }
    }
    for g in &gaps {
        let src = g.split(':').next().expect("gap source");
        assert!(
            src == "harness" || src == "fabricator",
            "unexpected coverage gap source (expected harness/fabricator only): {g}"
        );
    }
    eprintln!(
        "baml_parity: {} pending type(s) across harness/fabricator (later phases): {gaps:?}",
        gaps.len()
    );
}

#[test]
fn duplicate_baml_names_fail_fast() {
    use swe_seed_core::contracts::BamlKind;
    let mk = |name: &str, src: &str| BamlType {
        name: name.into(),
        source_file: src.into(),
        kind: BamlKind::Enum,
        fields: Vec::new(),
        field_types: Vec::new(),
        variants: Vec::new(),
    };
    // Cross-file duplicate name → fail.
    let cross = vec![mk("Dup", "swe_seed"), mk("Dup", "harness")];
    assert!(
        std::panic::catch_unwind(|| assert_unique_names(&cross)).is_err(),
        "cross-file duplicate name must fail fast"
    );
    // Within-file duplicate name → fail.
    let within = vec![mk("Dup", "swe_seed"), mk("Dup", "swe_seed")];
    assert!(
        std::panic::catch_unwind(|| assert_unique_names(&within)).is_err(),
        "within-file duplicate name must fail fast"
    );
    // No duplicates → ok.
    assert_unique_names(&[mk("A", "swe_seed"), mk("B", "harness")]);
}
