//! D1 correspondence controls: full original typed diagnostics are a coordinator decision.
//! The general suite/5 binding only requires nonempty text including the original cause.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    authored,
    coverage::{Origins, Scope},
    coverage_build::{self, CoverageSource},
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../edge/ess-cli/tests/fixtures/coverage-producers/inputs")
}
fn model(name: &str) -> EssIr {
    let root = fixtures().join("models").join(name);
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for name in [
        "system.yaml",
        "components.yaml",
        "topology.yaml",
        "domains/invoice.yaml",
        "domains/email.yaml",
    ] {
        let original = fs::read_to_string(root.join(name)).unwrap();
        let raw = RawSpecFile::parse(&original).unwrap();
        sources.insert(name, original);
        parsed.push((Source::new(name), raw));
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}

#[test]
fn d1_generated_inventory_preserves_full_original_typed_refusal_rendering() {
    let ir = model("billing-repeated-gap");
    let legacy = ess_conformance::synthesize(&ir);
    assert_eq!(
        legacy.refusals.len(),
        8,
        "independently observed repeated gap occurrences"
    );
    let mut expected: Vec<_> = legacy
        .refusals
        .iter()
        .map(|r| {
            (
                r.scenario.clone(),
                r.cause.code().to_string(),
                r.to_string(),
            )
        })
        .collect();
    expected.sort();
    let input = coverage_build::build(&ir, &[], Scope::System, Origins::Generated).unwrap();
    let inventory = input.selected().coverage().unwrap();
    assert_eq!(inventory.refused.len(), 8);
    let mut actual: Vec<_> = inventory
        .refused
        .iter()
        .map(|r| (r.scenario.clone(), r.code.clone(), r.message.clone()))
        .collect();
    actual.sort();
    let evidence = std::env::temp_dir().join(format!(
        "ess-coverage-adversary-d1-generated-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
    fs::write(
        evidence.join("comparison.json"),
        serde_json::to_string_pretty(&json!({"expected":expected,"actual":actual})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        actual, expected,
        "D1 selected full Refusal::to_string, including original prefix and help"
    );
}

#[test]
fn d1_authored_inventory_preserves_full_original_typed_refusal_rendering() {
    let ir = model("billing");
    let mut comparisons = Vec::new();
    for name in ["accepted-duplicate", "refused-then-accepted"] {
        let root = fixtures().join("authored").join(name);
        let sources: Vec<_> = ["a.yaml", "b.yaml"]
            .iter()
            .map(|identity| {
                CoverageSource::new(*identity, fs::read_to_string(root.join(identity)).unwrap())
                    .unwrap()
            })
            .collect();
        let legacy_sources: Vec<_> = sources
            .iter()
            .map(|s| authored::Source::new(s.identity().as_str(), s.text()))
            .collect();
        let legacy = authored::compile(&ir, &legacy_sources);
        assert_eq!(legacy.scenarios.len(), 1);
        assert_eq!(
            legacy.refusals.len(),
            if name == "accepted-duplicate" { 1 } else { 2 }
        );
        let input = coverage_build::build(&ir, &sources, Scope::System, Origins::Authored).unwrap();
        let mut expected: Vec<_> = legacy
            .refusals
            .iter()
            .map(|r| {
                (
                    r.origin.clone(),
                    r.scenario.clone(),
                    r.code().to_string(),
                    r.to_string(),
                )
            })
            .collect();
        expected.sort();
        let mut actual: Vec<_> = input
            .selected()
            .coverage()
            .unwrap()
            .refused
            .iter()
            .map(|r| {
                (
                    r.source.as_ref().unwrap().as_str().to_owned(),
                    r.scenario.clone(),
                    r.code.clone(),
                    r.message.clone(),
                )
            })
            .collect();
        actual.sort();
        comparisons.push((name, expected, actual));
    }
    let evidence = std::env::temp_dir().join(format!(
        "ess-coverage-adversary-d1-authored-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
    fs::write(
        evidence.join("comparison.json"),
        serde_json::to_string_pretty(&comparisons).unwrap(),
    )
    .unwrap();
    for (name, expected, actual) in comparisons {
        assert_eq!(
            actual, expected,
            "D1 {name}: selected full Refusal::to_string, with root-relative source and help"
        );
    }
}
