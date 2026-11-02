//! Adversary pass on story:authored-success-step-asserts-no-error (wave W1, unit RUN1).
//!
//! `crates/edge/ess-cli/tests/coverage_producers.rs` builds each structural case of
//! `tests/fixtures/coverage-producers/semantic-plan.json` with `coverage_build::build` and compares
//! every scenario's full definition with the case's `scenario_definition_oracle`
//! (`support/coverage_producer.rs`, `check_structure`). This file runs that same comparison for the
//! two authored-only cases, S03-authored and S08-unknown, from this crate, against the oracles
//! the unit moved, without building the CLI.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    coverage::{Origins, Scope},
    coverage_build::{self, CoverageSource},
    ConformanceSuite,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source as SpecSource,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../edge/ess-cli/tests/fixtures/coverage-producers")
}

fn files(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            result.extend(files(&entry.path()));
        } else if kind.is_file() {
            result.push(entry.path());
        }
    }
    result.sort();
    result
}

fn model(path: &Path) -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for file in files(path) {
        let label = file.strip_prefix(path).unwrap().to_str().unwrap();
        let original = fs::read_to_string(&file).unwrap();
        let raw = RawSpecFile::parse(&original).unwrap();
        sources.insert(label, original);
        parsed.push((SpecSource::new(label), raw));
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}

fn authored(relative: &str) -> Vec<CoverageSource> {
    let root = fixtures().join(relative);
    files(&root)
        .iter()
        .map(|file| {
            CoverageSource::new(
                file.strip_prefix(&root).unwrap().to_str().unwrap(),
                fs::read_to_string(file).unwrap(),
            )
            .unwrap()
        })
        .collect()
}

/// The scenario ids whose fresh definition differs from the oracle's, as `check_structure` would
/// report them.
fn drifted(authored_input: &str, oracle: &str) -> Vec<String> {
    let ir = model(&fixtures().join("inputs/models/billing"));
    let built = coverage_build::build(
        &ir,
        &authored(authored_input),
        Scope::System,
        Origins::Authored,
    )
    .expect("the authored structure builds");
    let oracle = ConformanceSuite::from_json(
        &fs::read_to_string(fixtures().join(oracle)).expect("the oracle is readable"),
    )
    .expect("the oracle reads");
    built
        .selected()
        .suite()
        .scenarios
        .iter()
        .filter(|(id, scenario)| oracle.scenarios.get(*id) != Some(*scenario))
        .map(|(id, _)| id.to_string())
        .collect()
}

/// S08-unknown and S09-unknown-child: `inputs/authored/single`, one `outcome: accepted` act.
#[test]
fn the_cli_coverage_producer_oracle_for_the_single_authored_act_still_matches() {
    assert_eq!(
        drifted(
            "inputs/authored/single",
            "legacy-observations/author-single-suite4.json"
        ),
        Vec::<String>::new(),
        "ess-cli coverage_producers check_structure compares this definition in full; a success \
         act's new `expect_no_error` is not in the oracle"
    );
}

/// S03-authored: `inputs/authored/rank`, four success acts.
#[test]
fn the_cli_coverage_producer_oracle_for_the_rank_authored_acts_still_matches() {
    assert_eq!(
        drifted(
            "inputs/authored/rank",
            "legacy-observations/author-rank-suite4.json"
        ),
        Vec::<String>::new(),
        "ess-cli coverage_producers check_structure compares this definition in full; a success \
         act's new `expect_no_error` is not in the oracle"
    );
}

/// Control for the two oracle cases above: with every `expect_no_error` step removed from both
/// the fresh definitions and the moved oracles, the two still match, so that step is the whole
/// of what the oracles gained (story:authored-success-step-asserts-no-error moved them).
#[test]
fn without_the_new_step_both_authored_oracles_match() {
    let ir = model(&fixtures().join("inputs/models/billing"));
    let strip = |scenario: &ess_conformance::ConformanceScenario| {
        let mut stripped = scenario.clone();
        stripped
            .steps
            .retain(|step| !matches!(step, ess_conformance::ScenarioStep::ExpectNoError));
        stripped
    };
    for (input, oracle) in [
        (
            "inputs/authored/single",
            "legacy-observations/author-single-suite4.json",
        ),
        (
            "inputs/authored/rank",
            "legacy-observations/author-rank-suite4.json",
        ),
    ] {
        let built = coverage_build::build(&ir, &authored(input), Scope::System, Origins::Authored)
            .expect("the authored structure builds");
        let oracle =
            ConformanceSuite::from_json(&fs::read_to_string(fixtures().join(oracle)).unwrap())
                .unwrap();
        for (id, scenario) in &built.selected().suite().scenarios {
            let stripped = strip(scenario);
            assert_ne!(
                stripped.steps.len(),
                scenario.steps.len(),
                "story:authored-success-step-asserts-no-error: {id} is a success act"
            );
            assert_eq!(
                Some(stripped),
                oracle.scenarios.get(id).map(strip),
                "story:authored-success-step-asserts-no-error: {id}"
            );
        }
    }
}
