//! A paged view (`paging:`, ess/16, beyond10x/ess#174) in generated seams: every code target serves
//! a view as every row its projection holds, with no page and no total, so each refuses a paged view
//! by name, as it refuses `Json`, rather than emitting a handler that ignores the page.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str = include_str!("../../../verify/ess-conformance/tests/fixtures/view-paging.yaml");

const PAGED: &str = "views.demo.jobs.JobList.paging";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("view-paging.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_code_target_refuses_a_paged_view_by_name() {
    let ir = ir(MODEL);
    for target in [Target::Rust, Target::Go] {
        let failure = synthesize_for(&ir, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses a paged view"));
        let text = format!("{failure:?}");
        assert!(text.contains(PAGED), "{target:?} names {PAGED}: {text}");
        assert!(
            text.contains(&format!("{:?}", TargetFailureCode::MissingRepresentation)),
            "{text}"
        );
    }
    let plan = ess_synth::SynthesisPlan::of(&ir);
    let failures = [
        ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
        ("clap", ess_synth::clap::workspace(&ir, &plan).err()),
    ];
    for (target, failure) in failures {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses a paged view"));
        assert!(
            failure.to_canonical_json().contains(PAGED),
            "{target} names {PAGED}: {}",
            failure.to_canonical_json()
        );
    }
}

#[test]
fn an_unpaged_view_is_not_refused_for_paging() {
    let text = MODEL
        .replace("    paging: {page: page, size: size, total: true}\n", "")
        .replace(
            "      - {name: page, type: Integer}\n      - {name: size, type: Integer}\n",
            "",
        );
    let ir = ir(&text);
    for target in [Target::Rust, Target::Go] {
        if let Err(failure) = synthesize_for(&ir, target) {
            let text = format!("{failure:?}");
            assert!(!text.contains(".paging"), "{target:?}: {text}");
        }
    }
}
