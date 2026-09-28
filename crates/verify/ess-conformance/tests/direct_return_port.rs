//! Direct returns reserve new formats beside the released round-three vocabulary.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::authored;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

#[test]
fn direct_returns_use_fresh_source_and_suite_versions() {
    assert_eq!(ess_conformance::direct_response::ORDINARY, 28);
    assert_eq!(ess_conformance::direct_response::COVERAGE, 29);
    let raw = RawSpecFile::parse(
        "format: ess/17\nsystem: library\nversion: v1\ndomain: library.api\ncommands:\n  - name: library.api.Read\n    response:\n      - {name: value, type: Integer}\n    outcomes:\n      - {name: returned, returns: true}\n",
    )
    .expect("the new direct-return declaration parses");
    let spec = Specification::assemble([(Source::new("library.yaml"), raw)])
        .expect("ess/17 admits direct returns");
    let model = compile(&spec, &SourceMap::new()).unwrap();
    let authored = authored::compile(
        &model,
        &[authored::Source::new(
            "return.yaml",
            "type: ess-scenario/4\ndomain: library.api\nscenario: fresh-return\nsummary: A direct return uses a fresh suite envelope.\ntimeline:\n  - at: 2026-09-28T00:00:00Z\n    command: library.api.Read\n    outcome: returned\n    response: {value: 9007199254740993}\n",
        )],
    );
    assert!(authored.refusals.is_empty(), "{:?}", authored.refusals);
    let mut suite = ess_conformance::synthesize(&model).suite;
    suite.scenarios = authored.scenarios;
    suite.select_fresh_format();
    assert_eq!(suite.provenance.suite_version.major(), 28);
    assert!(suite
        .to_canonical_json()
        .unwrap()
        .contains("expect_direct_response"));
}

#[test]
fn released_round_three_suites_keep_exact_bytes_and_meaning() {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::build,
        AdmittedSuite,
    };
    let raw = RawSpecFile::parse(include_str!(
        "../../../specify/ess-compiler/tests/fixtures/absent-input.yaml"
    ))
    .unwrap();
    let spec = Specification::assemble([(Source::new("absent-input.yaml"), raw)]).unwrap();
    let model = compile(&spec, &SourceMap::new()).unwrap();
    let ordinary = ess_conformance::synthesize(&model).suite;
    let covered = build(&model, &[], Scope::System, Origins::Generated).unwrap();
    for (expected, actual, major) in [
        (
            include_str!("fixtures/released-0.38/suite26.json"),
            ordinary.to_canonical_json().unwrap(),
            26,
        ),
        (
            include_str!("fixtures/released-0.38/suite27.json"),
            covered.selected().original_json().to_owned(),
            27,
        ),
    ] {
        let admitted = AdmittedSuite::from_json(expected).unwrap();
        assert_eq!(admitted.original_json(), expected);
        assert_eq!(admitted.suite().provenance.suite_version.major(), major);
        assert_eq!(actual, expected);
        assert!(expected.contains("execute_command_without_input"));
        assert!(!expected.contains("expect_direct_response"));
    }
}
