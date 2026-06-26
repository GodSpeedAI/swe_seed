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
use swe_seed_core::seed::{
    ArtifactMetadata, BoundaryFinding, BoundaryReport, LayerCapability, LayerName,
    ProjectSeed, ReviewRequirement, SeedArtifactStatus, SeedNeed, SeedPackageManifest,
    SeedRegenerationInput, SeedRegenerationPlan, SeedSourceRef, SeedValidationRequirement,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn parse() -> Vec<BamlType> {
    parse_baml_dir(&root().join(".agent-harness/baml/baml_src")).expect("parse baml dir")
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
fn baml_parity_swe_seed_layer() {
    let types = parse();

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
    let reg: HashMap<&str, BamlShape> = reg_list.into_iter().map(|(n, s)| (n, s)).collect();

    // 1. Every swe_seed.baml type must be registered (no coverage gap).
    let unregistered: Vec<&str> = swe.keys().filter(|n| !reg.contains_key(*n)).copied().collect();
    assert!(
        unregistered.is_empty(),
        "swe_seed.baml types without a Rust parity impl: {unregistered:?}"
    );

    // 2. Every registered type maps to a swe_seed.baml type.
    for name in reg.keys() {
        assert!(
            swe.contains_key(name),
            "registered type {name} has no swe_seed.baml counterpart"
        );
    }

    // 3. Shape (field names + types, enum variants) parity for every type.
    for (name, shape) in &reg {
        assert_shape_match(name, shape, swe[name]);
    }
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
