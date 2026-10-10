//! Adversary pass 1 on `caller:` in authored acts (`ess-scenario/5`).
//!
//! Authoring requires an act to state only the caller attributes its command reads that are not
//! `Optional` for its actor: an act may leave an optional one unstated and is admitted. The run
//! has to agree, reading the unstated optional attribute as absent rather than reporting the
//! scenario `unsupported` for a caller nobody was asked to state.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ConformanceReport, ConformanceStatus};
use ess_conformance::{AdmittedSuite, ConformanceSuite, SuiteProvenance};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

/// `Sign`, granted to `Guest` alone, publishes the caller's `nickname`, which `Guest` declares
/// `Optional<String>` and which the event declares `Optional<String>` too.
const MODEL: &str = r"format: ess/19
system: ledger
version: v1
domain: ledger.notes
actors:
  - name: ledger.notes.Guest
    attributes:
      - {name: nickname, type: 'Optional<String>'}
    may: [ledger.notes.Sign]
commands:
  - name: ledger.notes.Sign
    input: [{name: title, type: String}]
    outcomes:
      - name: signed
        emits: [ledger.notes.Signed]
        payload:
          ledger.notes.Signed: {nickname: {caller: nickname}}
events:
  - name: ledger.notes.Signed
    fields: [{name: nickname, type: 'Optional<String>'}]
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn authoring(text: &str) -> Authoring {
    compile_authored(&ir(), &[Source::new("sign.yaml", text)])
}

fn suite(authoring: Authoring) -> ConformanceSuite {
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    let ir = ir();
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    suite.select_fresh_format_for(&ir);
    suite
}

fn run(suite: &ConformanceSuite) -> ConformanceReport {
    let admitted = AdmittedSuite::from_suite(suite).expect("admitted");
    ess_conformance::Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir()))
        .into_report()
}

fn sign(caller: &str) -> String {
    format!(
        "type: ess-scenario/5\ndomain: ledger.notes\nscenario: sign\nsummary: A guest signs.\n\
         timeline:\n  - at: 2026-01-05T09:00:00Z\n    command: ledger.notes.Sign\n    actor: \
         ledger.notes.Guest\n{caller}    input: {{title: first}}\n    outcome: signed\n    \
         events:\n      - event: ledger.notes.Signed\n"
    )
}

/// The act states no `caller:`; authoring admits it because `nickname` is optional for `Guest`.
/// The interpreted target then has to run it the way it runs `caller: {nickname: null}` (the
/// control below), not report it `unsupported` for a caller attribute nothing required.
#[test]
fn an_optional_attribute_an_act_may_leave_unstated_runs_on_the_interpreted_target() {
    let control = run(&suite(authoring(&sign("    caller: {nickname: null}\n"))));
    assert_eq!(control.status, ConformanceStatus::Passed, "{control:#?}");

    let authoring = authoring(&sign(""));
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    let report = run(&suite(authoring));
    assert_eq!(report.status, ConformanceStatus::Passed, "{report:#?}");
}
