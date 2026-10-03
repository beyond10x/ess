//! Adversary pass 2 for beyond10x/ess#186: the emitter's new refusal of suite versions its own
//! runner does not admit (`go::refuse_unadmitted`, `NEWEST_ADMITTED_SUITE_MAJOR`).
//!
//! Three questions, each answered by a program rather than a paragraph:
//!
//! - a fresh direct-return suite (`/34`, `/35`) emits on both targets;
//! - the new refusal itself names the version, the limit and the target, at
//!   `$.provenance.suite_version`, for `emit` on both targets;
//! - the emit boundary agrees with the admission limit written in each emitted runtime, read as
//!   text, with no Go or Node toolchain on the machine (`generated_docs.rs` holds the same
//!   agreement by running both runners, and cannot run where they are absent).

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::scenario::{SuiteFormat, SuiteProvenance};
use ess_conformance::{authored, AdmissionError, ConformanceSuite};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::evidence::SpecDigest;

/// The direct-return model `tests/direct_returns.rs` uses: one command whose outcome returns.
const MODEL: &str = r"format: ess/17
system: library
version: v1
domain: library.api
types:
  - name: library.api.Item
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: ordinal, type: Integer}
commands:
  - name: library.api.Read
    response:
      - {name: value, type: String}
      - {name: sequence, type: 'List<Integer>'}
      - {name: item, type: library.api.Item}
    outcomes:
      - name: returned
        returns: true
";

const SCENARIO: &str = r"type: ess-scenario/4
domain: library.api
scenario: pure-return
summary: The actual return contains the declared literal and ordered values.
timeline:
  - at: 2026-09-28T00:00:00Z
    command: library.api.Read
    outcome: returned
    response: {value: actual, sequence: [1, 2, 2], item: {label: nested, ordinal: 9007199254740993}}
";

fn ir() -> ess_compiler::EssIr {
    let spec = Specification::assemble([(
        Source::new("library.yaml"),
        RawSpecFile::parse(MODEL).expect("direct-return source parses"),
    )])
    .expect("direct-return source validates");
    compile(&spec, &SourceMap::new()).expect("direct-return source compiles")
}

fn direct_return_suite() -> ConformanceSuite {
    let model = ir();
    let authored = authored::compile(&model, &[authored::Source::new("return.yaml", SCENARIO)]);
    assert!(authored.refusals.is_empty(), "{:?}", authored.refusals);
    let mut suite = ess_conformance::synthesize(&model).suite;
    suite.scenarios = authored.scenarios;
    suite.select_fresh_format();
    suite
}

fn direct_return_input() -> ess_conformance::coverage::AdmittedInput {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::{build, CoverageSource},
    };
    build(
        &ir(),
        &[CoverageSource::new("return.yaml", SCENARIO).unwrap()],
        Scope::System,
        Origins::GeneratedAndAuthored,
    )
    .unwrap()
}

fn empty_suite(major: u32) -> ConformanceSuite {
    ConformanceSuite::new(SuiteProvenance {
        suite_version: SuiteFormat::parse(&format!("ess-conformance/{major}")).expect("a version"),
        system: "billing".to_owned(),
        specification_version: "v3".to_owned(),
        spec_digest: SpecDigest::new("ab".repeat(32)).expect("a digest"),
        contract_digest: SpecDigest::new("cd".repeat(32)).expect("a digest"),
        component: None,
        scenario_initial_state: None,
    })
}

fn go(suite: &ConformanceSuite) -> Result<Vec<(String, String)>, AdmissionError> {
    ess_conformance::go::emit(suite)
        .map(|files| files.into_iter().map(|f| (f.path, f.contents)).collect())
}

fn typescript(suite: &ConformanceSuite) -> Result<Vec<(String, String)>, AdmissionError> {
    ess_conformance::ts::emit(suite)
        .map(|files| files.into_iter().map(|f| (f.path, f.contents)).collect())
}

/// `(reason, path, message)` of an emitter refusal with exactly one issue.
fn issue(error: &AdmissionError) -> (String, String, String) {
    assert_eq!(error.issues.len(), 1, "one issue: {error}");
    let issue = &error.issues[0];
    (
        issue.reason.to_string(),
        issue.path.clone(),
        issue.detail.clone(),
    )
}

/// Both ordinary and coverage direct-return suites reach their executing runtime.
#[test]
fn adversary2_direct_return_suite_emits_in_both_supported_runtimes() {
    let suite = direct_return_suite();
    assert_eq!(suite.provenance.suite_version.major(), 34, "precondition");
    let input = direct_return_input();
    assert_eq!(
        input.selected().suite().provenance.suite_version.major(),
        35,
        "precondition"
    );
    ess_conformance::go::emit(&suite).expect("Go executes direct responses");
    ess_conformance::ts::emit(&suite).expect("TypeScript executes direct responses");
    ess_conformance::go::emit_input(&input).expect("Go executes direct response coverage");
    ess_conformance::ts::emit_input(&input).expect("TypeScript executes direct response coverage");
}

/// Future majors still name the version, newest admitted major and target in their refusal.
#[test]
fn adversary2_unadmitted_version_refusal_names_the_version_and_the_limit() {
    for major in [36, 37] {
        for (target, error) in [
            ("Go", go(&empty_suite(major)).unwrap_err()),
            ("TypeScript", typescript(&empty_suite(major)).unwrap_err()),
        ] {
            let (reason, path, message) = issue(&error);
            assert_eq!(reason, "UnsupportedTarget", "{target} /{major}: {error}");
            assert_eq!(
                path, "$.provenance.suite_version",
                "{target} /{major}: {error}"
            );
            for needle in [
                format!("generated {target} runner"),
                "`ess-conformance/35`".to_owned(),
                format!("`ess-conformance/{major}`"),
                "regenerate using a supported suite version".to_owned(),
            ] {
                assert!(
                    message.contains(&needle),
                    "{target} /{major}: the refusal does not say {needle}: {message}"
                );
            }
        }
    }
}

fn file<'a>(package: &'a [(String, String)], suffix: &str) -> &'a str {
    &package
        .iter()
        .find(|(path, _)| path.ends_with(suffix))
        .unwrap_or_else(|| panic!("the package carries `{suffix}`"))
        .1
}

/// `const newestSuiteMajor = N` in the emitted Go runtime.
fn go_newest(runtime: &str) -> u32 {
    let rest = runtime
        .split("const newestSuiteMajor = ")
        .nth(1)
        .expect("runtime.go declares newestSuiteMajor");
    rest.split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|n| n.parse().ok())
        .expect("a number")
}

/// The majors of `SUITE_MAJORS` in the emitted TypeScript runtime.
fn ts_majors(runtime: &str) -> Vec<u32> {
    let table = runtime
        .split("const SUITE_MAJORS: { [version: string]: number } = {")
        .nth(1)
        .and_then(|rest| rest.split("};").next())
        .expect("runtime.ts declares SUITE_MAJORS");
    table
        .lines()
        .filter_map(|line| line.trim().strip_prefix("'ess-conformance/"))
        .map(|rest| {
            rest.split('\'')
                .next()
                .and_then(|n| n.parse().ok())
                .expect("a major")
        })
        .collect()
}

/// The emit boundary is the admission limit each emitted runtime declares, and the two runtimes
/// declare the same one: every major up to it emits on both targets, the next does not.
#[test]
fn adversary2_emit_boundary_is_the_limit_both_runtimes_declare() {
    let go_runtime = go(&empty_suite(4)).expect("suite/4 emits");
    let go_newest = go_newest(file(&go_runtime, "/runtime.go"));
    let ts_runtime = typescript(&empty_suite(4)).expect("suite/4 emits");
    let ts_majors = ts_majors(file(&ts_runtime, "/runtime.ts"));
    assert_eq!(
        ts_majors,
        (1..=go_newest).collect::<Vec<_>>(),
        "the TypeScript runtime admits other majors than the Go runtime's 1..={go_newest}"
    );
    for major in 1..=go_newest {
        let suite = empty_suite(major);
        assert!(
            go(&suite).is_ok(),
            "Go refuses /{major}, which runtime.go admits"
        );
        assert!(
            typescript(&suite).is_ok(),
            "TypeScript refuses /{major}, which runtime.ts admits"
        );
    }
    let next = empty_suite(go_newest + 1);
    assert!(
        go(&next).is_err() && typescript(&next).is_err(),
        "a package is emitted for /{}, which neither runtime admits",
        go_newest + 1
    );
}
