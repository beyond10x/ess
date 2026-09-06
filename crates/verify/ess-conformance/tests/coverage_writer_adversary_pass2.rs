//! Final independently compiled batch ownership and full D1 diagnostic preservation.
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
fn final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection() {
    let ir = model("billing");
    let original = fs::read_to_string(fixtures().join("authored/single/a.yaml")).unwrap();
    let identities = ["a/a.yaml", "a/b.yaml", "é/a.yaml"];
    let sources: Vec<_> = identities
        .iter()
        .map(|identity| CoverageSource::new(*identity, &original).unwrap())
        .collect();
    let old_sources: Vec<_> = sources
        .iter()
        .map(|source| authored::Source::new(source.identity().as_str(), source.text()))
        .collect();
    let legacy = authored::compile(&ir, &old_sources);
    assert_eq!(legacy.scenarios.len(), 1);
    assert_eq!(legacy.refusals.len(), 2);
    let mut expected: Vec<_> = legacy
        .refusals
        .iter()
        .map(|refusal| {
            (
                refusal.origin.clone(),
                refusal.code().to_string(),
                refusal.to_string(),
            )
        })
        .collect();
    expected.sort();
    let batches: Vec<_> = sources
        .iter()
        .map(|source| coverage_build::compile_sources(&ir, std::slice::from_ref(source)).unwrap())
        .collect();
    for batch in &batches {
        assert_eq!(batch.accepted(), 1);
        assert_eq!(batch.refusals().count(), 0);
    }
    let mut observations = Vec::new();
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let ordered: Vec<_> = order.iter().map(|index| batches[*index].clone()).collect();
        let input =
            coverage_build::merge_batches(&ir, &ordered, Scope::System, Origins::Authored).unwrap();
        let inventory = input.selected().coverage().unwrap();
        let mut actual: Vec<_> = inventory
            .refused
            .iter()
            .map(|refusal| {
                (
                    refusal.source.as_ref().unwrap().as_str().to_owned(),
                    refusal.code.clone(),
                    refusal.message.clone(),
                )
            })
            .collect();
        actual.sort();
        assert_eq!(
            actual, expected,
            "full original diagnostic depends on sorted identities, not batch arrival"
        );
        assert_eq!(inventory.authored.len(), 1);
        assert_eq!(inventory.authored_sources.len(), 3);
        for refusal in &inventory.refused {
            assert_eq!(
                refusal
                    .retained
                    .as_ref()
                    .unwrap()
                    .source
                    .as_ref()
                    .unwrap()
                    .as_str(),
                identities[0]
            );
            assert_eq!(refusal.subject, None);
        }
        let empty = input.select(&[]).unwrap();
        let filtered = empty.selected().coverage().unwrap();
        assert!(empty.selected().suite().is_empty());
        assert_eq!(filtered.refused, inventory.refused);
        assert_eq!(filtered.authored_sources, inventory.authored_sources);
        assert_eq!(filtered.outside.len(), 1);
        let original_input = empty.document().to_canonical_json().unwrap();
        let admitted =
            ess_conformance::coverage::AdmittedInput::from_json(&original_input).unwrap();
        assert_eq!(admitted.selected().coverage().unwrap(), filtered);
        observations.push(
            json!({"order":order,"expected":expected,"actual":actual,"input":original_input}),
        );
    }
    record_comparisons(&observations);
    println!("six real final-merge permutations; two exact legacy duplicate diagnostics each; empty selection retains all original source/refusal records");
}

fn record_comparisons(observations: &[serde_json::Value]) {
    let evidence =
        std::env::temp_dir().join(format!("ess-review2-final-merge-{}", std::process::id()));
    fs::create_dir_all(&evidence).unwrap();
    fs::write(
        evidence.join("comparisons.json"),
        serde_json::to_string_pretty(observations).unwrap(),
    )
    .unwrap();
}
