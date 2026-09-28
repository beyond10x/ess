//! The emitted Go and TypeScript packages describe what their runner does (beyond10x/ess#186).
//!
//! Two sentences an adopter follows literally, and both were wrong:
//!
//! - `CommandResult.Outcome` said "empty when it refused", while the runner compares `Outcome` for
//!   refusals exactly as for any other branch, so a target that followed it failed every refusal
//!   scenario.
//! - The README put `ESS_REPORT_FORMAT=2` under "The report", as if it chose a report shape, while
//!   the runner refuses to execute the suite versions that need it and stops before the first
//!   scenario.
//!
//! The README's claim is checked against the runtime the same package carries: the versions the
//! runner refuses without `ESS_REPORT_FORMAT=2` are read out of the emitted runtime source, so a
//! runtime that changes the rule turns this red until the README says the same.

use std::collections::BTreeSet;

use ess_conformance::scenario::{SuiteFormat, SuiteProvenance, SUPPORTED_SUITE_FORMATS};
use ess_conformance::ConformanceSuite;
use ess_primitives::evidence::SpecDigest;

fn suite(major: u32) -> ConformanceSuite {
    ConformanceSuite::new(SuiteProvenance {
        suite_version: SuiteFormat::parse(&format!("ess-conformance/{major}")).expect("a version"),
        system: "billing".to_owned(),
        specification_version: "v3".to_owned(),
        spec_digest: SpecDigest::new("ab".repeat(32)).expect("a digest"),
        contract_digest: SpecDigest::new("cd".repeat(32)).expect("a digest"),
        component: None,
    })
}

/// One emitted package as `(path, contents)` pairs.
type Package = Vec<(String, String)>;

fn go(major: u32) -> Package {
    ess_conformance::go::emit(&suite(major))
        .expect("an empty suite emits")
        .into_iter()
        .map(|file| (file.path, file.contents))
        .collect()
}

fn typescript(major: u32) -> Package {
    ess_conformance::ts::emit(&suite(major))
        .expect("an empty suite emits")
        .into_iter()
        .map(|file| (file.path, file.contents))
        .collect()
}

fn file<'a>(package: &'a Package, suffix: &str) -> &'a str {
    let Some((_, contents)) = package.iter().find(|(path, _)| path.ends_with(suffix)) else {
        panic!("the package carries `{suffix}`")
    };
    contents
}

/// The suite majors the runtime refuses to execute without `ESS_REPORT_FORMAT=2`, read from the
/// version gate between suite admission and execution adaptation.
fn refused_without_format_2(runtime: &str, from: &str, to: &str) -> BTreeSet<u32> {
    let start = runtime
        .find(from)
        .unwrap_or_else(|| panic!("the runtime admits the suite with `{from}`"));
    let end = start
        + runtime[start..]
            .find(to)
            .unwrap_or_else(|| panic!("the runtime adapts the suite with `{to}`"));
    let gate = &runtime[start..end];
    assert!(
        gate.contains("require explicit ESS_REPORT_FORMAT=2 before execution"),
        "the version gate moved: {gate}"
    );
    let prefix = "ess-conformance/";
    gate.match_indices(prefix)
        .map(|(at, _)| {
            gate[at + prefix.len()..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .expect("a suite major")
        })
        .collect()
}

fn go_rule() -> BTreeSet<u32> {
    refused_without_format_2(
        file(&go(4), "/runtime.go"),
        "suite, err := admitRunInput(suiteJSON)",
        "suite, err = executionSuite(suite)",
    )
}

fn typescript_rule() -> BTreeSet<u32> {
    refused_without_format_2(
        file(&typescript(4), "/runtime.ts"),
        "const version = suite.provenance.suite_version;",
        "suite = executionSuite(suite);",
    )
}

/// The README before and after its `## The report` heading.
fn split_at_report(readme: &str) -> (&str, &str) {
    readme
        .split_once("## The report")
        .unwrap_or_else(|| panic!("the README has a report section:\n{readme}"))
}

fn check_readme(target: &str, readme: &str, major: u32, required: bool, run: &str) {
    let (instructions, report) = split_at_report(readme);
    if required {
        assert!(
            instructions.contains(&format!("ESS_REPORT_FORMAT=2 {run}")),
            "{target} suite/{major}: the run instructions do not run it with \
             `ESS_REPORT_FORMAT=2`:\n{readme}"
        );
        assert!(
            instructions.contains("stops before the first scenario"),
            "{target} suite/{major}: the run instructions do not say what happens without it:\n{readme}"
        );
        assert!(
            instructions.contains(&format!("`ess-conformance/{major}`")),
            "{target} suite/{major}: the run instructions do not name this suite's version:\n{readme}"
        );
        assert!(
            !report.contains("Select `ESS_REPORT_FORMAT=2` explicitly"),
            "{target} suite/{major}: the requirement still reads as a report option:\n{readme}"
        );
    } else {
        assert!(
            !instructions.contains("ESS_REPORT_FORMAT"),
            "{target} suite/{major}: the run instructions require a variable the runtime does \
             not:\n{readme}"
        );
    }
}

#[test]
fn generated_docs_go_readme_states_the_report_format_requirement_where_the_run_is() {
    let rule = go_rule();
    assert!(
        rule.contains(&8),
        "the Go runtime gate reads as {rule:?}; the parse is wrong"
    );
    for &major in SUPPORTED_SUITE_FORMATS {
        let package = go(major);
        check_readme(
            "Go",
            file(&package, "/README.md"),
            major,
            rule.contains(&major),
            "go test ./...",
        );
    }
}

#[test]
fn generated_docs_typescript_readme_states_the_report_format_requirement_where_the_run_is() {
    let rule = typescript_rule();
    assert!(
        rule.contains(&8),
        "the TypeScript runtime gate reads as {rule:?}; the parse is wrong"
    );
    for &major in SUPPORTED_SUITE_FORMATS {
        let package = typescript(major);
        check_readme(
            "TypeScript",
            file(&package, "README.md"),
            major,
            rule.contains(&major),
            "npm test",
        );
    }
}

#[test]
fn generated_docs_go_command_result_outcome_says_a_refusal_returns_its_outcome() {
    let runtime = go(4);
    let runtime = file(&runtime, "/runtime.go");
    assert!(
        !runtime.contains("empty when it refused"),
        "the Go `CommandResult.Outcome` comment still says a refusal leaves it empty"
    );
    assert!(
        runtime.contains("A refusal takes a branch too: return the refusing outcome's name"),
        "the Go `CommandResult.Outcome` comment does not say what a refusal returns"
    );
}

#[test]
fn generated_docs_typescript_command_result_outcome_says_a_refusal_returns_its_outcome() {
    let runtime = typescript(4);
    let runtime = file(&runtime, "/runtime.ts");
    assert!(
        !runtime.contains("empty when it refused"),
        "the TypeScript `CommandResult.outcome` doc still says a refusal leaves it empty"
    );
    assert!(
        runtime.contains("A refusal takes a branch too: return the refusing outcome's name"),
        "the TypeScript `CommandResult.outcome` doc does not say what a refusal returns"
    );
}
