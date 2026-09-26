//! Outcome groups leave no trace in a suite (beyond10x/ess#105).
//!
//! `docs/design/outcome-groups.md`, C1's suite half: a specification with groups and the same
//! specification with each outcome copied by hand synthesize byte-identical suites, and a group of
//! N members adds N ordinary external-outcome scenarios.

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::synthesize::synthesize;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const FIXTURE: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("../../../specify/ess-domain/tests/fixtures/outcome-groups/system.yaml"),
    ),
    (
        "domains/calls.yaml",
        include_str!(
            "../../../specify/ess-domain/tests/fixtures/outcome-groups/domains/calls.yaml"
        ),
    ),
    (
        "domains/session.yaml",
        include_str!(
            "../../../specify/ess-domain/tests/fixtures/outcome-groups/domains/session.yaml"
        ),
    ),
];

const TWIN: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("../../../specify/ess-domain/tests/fixtures/outcome-groups-twin/system.yaml"),
    ),
    (
        "domains/calls.yaml",
        include_str!(
            "../../../specify/ess-domain/tests/fixtures/outcome-groups-twin/domains/calls.yaml"
        ),
    ),
    (
        "domains/session.yaml",
        include_str!(
            "../../../specify/ess-domain/tests/fixtures/outcome-groups-twin/domains/session.yaml"
        ),
    ),
];

fn suite_json(files: &[(&str, &str)]) -> String {
    let spec = Specification::assemble(files.iter().map(|(label, text)| {
        (
            Source::new(*label),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}")),
        )
    }))
    .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"));
    synthesize(&ir)
        .suite
        .to_canonical_json()
        .expect("the suite serializes")
}

#[test]
fn c1_a_group_and_its_hand_copy_synthesize_byte_identical_suites() {
    let grouped = suite_json(&FIXTURE);
    assert_eq!(grouped, suite_json(&TWIN));
    for scenario in [
        "tel.calls.Hold/outcome/credential-rejected",
        "tel.calls.Hold/outcome/session-expired",
        "tel.calls.Resume/outcome/credential-rejected",
        "tel.calls.Dial/outcome/session-expired",
    ] {
        assert!(
            grouped.contains(scenario),
            "{scenario} is not in:\n{grouped}"
        );
    }
    assert!(
        !grouped.contains("tel.calls.Dial/outcome/credential-rejected"),
        "the Supervisor's command is not an Agent's"
    );
}
