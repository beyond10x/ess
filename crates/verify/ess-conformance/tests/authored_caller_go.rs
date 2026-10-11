//! The generated Go runtime sends an authored act's `caller:` (`ess-scenario/5`) and gives the
//! reference verdict.
//!
//! The authored suite in `fixtures/authored-caller-scenario.yaml` states each act's caller, a
//! literal and an instance arranged with `setup:`. The interpreted target's answers are recorded
//! once and replayed to the Go runtime, which must send every command as the caller the reference
//! runner sent it as, and pass.

mod support_go;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::{ConformanceSuite, SuiteProvenance};
use ess_domain::{spec::RawSpecFile, system::Source as SpecSource, Specification};

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(include_str!("fixtures/authored-caller.yaml"))
        .unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let ir = ir();
    let authoring = compile_authored(
        &ir,
        &[Source::new(
            "note.yaml",
            include_str!("fixtures/authored-caller-scenario.yaml"),
        )],
    );
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    suite.select_fresh_format_for(&ir);
    suite
}

#[test]
fn go_sends_the_authored_caller_and_passes_with_the_reference_runner() {
    let suite = suite();
    let verdicts =
        support_go::assert_parity("authored-caller", &suite, Interpreted::for_model(ir()));
    assert_eq!(verdicts.len(), 1, "{verdicts:?}");
    assert!(support_go::not_passed(&verdicts).is_empty(), "{verdicts:?}");
}
