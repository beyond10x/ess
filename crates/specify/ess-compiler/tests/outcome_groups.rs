//! Outcome groups in the IR, and the stable codes their refusals compile to (beyond10x/ess#105).
//!
//! `docs/design/outcome-groups.md`: C1 (a group and its hand-copied twin compile to byte-identical
//! canonical IR) and the code half of C3 (each G1–G16 refusal is `ESS-COMMAND-<class>`, cited at the
//! group's own `name:` line). The fixture lives beside the `ess-domain` cases that share it.

use ess_compiler::resolve::{compile_locating, diagnose_locating};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const FIXTURE: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("../../ess-domain/tests/fixtures/outcome-groups/system.yaml"),
    ),
    (
        "domains/calls.yaml",
        include_str!("../../ess-domain/tests/fixtures/outcome-groups/domains/calls.yaml"),
    ),
    (
        "domains/session.yaml",
        include_str!("../../ess-domain/tests/fixtures/outcome-groups/domains/session.yaml"),
    ),
];

const TWIN: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("../../ess-domain/tests/fixtures/outcome-groups-twin/system.yaml"),
    ),
    (
        "domains/calls.yaml",
        include_str!("../../ess-domain/tests/fixtures/outcome-groups-twin/domains/calls.yaml"),
    ),
    (
        "domains/session.yaml",
        include_str!("../../ess-domain/tests/fixtures/outcome-groups-twin/domains/session.yaml"),
    ),
];

fn assemble(
    files: &[(&str, String)],
) -> Result<Specification, ess_primitives::error::ValidationErrors> {
    Specification::assemble(files.iter().map(|(label, text)| {
        (
            Source::new(*label),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}\n{text}")),
        )
    }))
}

fn owned(files: &[(&str, &str)]) -> Vec<(&'static str, String)> {
    files
        .iter()
        .map(|(label, text)| {
            let label: &'static str = match *label {
                "system.yaml" => "system.yaml",
                "domains/calls.yaml" => "domains/calls.yaml",
                _ => "domains/session.yaml",
            };
            (label, (*text).to_owned())
        })
        .collect()
}

fn sources(files: &[(&str, String)]) -> SourceMap {
    let mut sources = SourceMap::new();
    for (label, text) in files {
        sources.insert(*label, text.clone());
    }
    sources
}

fn canonical_ir(files: &[(&str, String)]) -> String {
    let spec = assemble(files).unwrap_or_else(|errors| panic!("{errors}"));
    let labels: Vec<&str> = files.iter().map(|(label, _)| *label).collect();
    compile_locating(&spec, &sources(files), &labels)
        .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"))
        .to_canonical_json()
}

#[test]
fn c1_a_group_and_its_hand_copy_compile_to_byte_identical_ir() {
    let grouped = canonical_ir(&owned(&FIXTURE));
    let copied = canonical_ir(&owned(&TWIN));
    assert_eq!(grouped, copied);
    assert!(grouped.contains("credential-rejected"), "{grouped}");
}

#[test]
fn c7_a_model_without_groups_compiles_as_it_did() {
    // The twin carries no group; the same text under the newest format it was valid in before
    // groups existed compiles to the same bytes, so the format bump alone moves nothing.
    let mut earlier = owned(&TWIN);
    earlier[0].1 = earlier[0].1.replace("format: ess/12", "format: ess/10");
    assert_eq!(canonical_ir(&earlier), canonical_ir(&owned(&TWIN)));
}

/// The calls file's groups replaced by `calls`, the session file's by `session`.
fn with_groups(calls: &str, session: &str) -> Vec<(&'static str, String)> {
    let mut files = owned(&FIXTURE);
    for (index, groups) in [(1, calls), (2, session)] {
        let head = files[index]
            .1
            .split_once("outcome_groups:\n")
            .expect("the fixture file carries groups")
            .0
            .to_owned();
        files[index].1 = if groups.is_empty() {
            head
        } else {
            format!("{head}outcome_groups:\n{groups}")
        };
    }
    files
}

/// The one diagnostic a document compiles to: its code, its path, its source and its line.
fn diagnosed(files: &[(&'static str, String)]) -> Vec<(String, String, String, Option<usize>)> {
    let errors = assemble(files).expect_err("the document must be refused");
    let labels: Vec<&str> = files.iter().map(|(label, _)| *label).collect();
    diagnose_locating(&errors, &sources(files), &labels)
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic.span.as_ref().expect("a span");
            (
                diagnostic.code.to_string(),
                span.path.clone(),
                span.source.clone(),
                span.located.map(|located| located.line),
            )
        })
        .collect()
}

/// The 1-based line of `needle` in `text`.
fn line_of(text: &str, needle: &str) -> usize {
    let index = text
        .find(needle)
        .unwrap_or_else(|| panic!("`{needle}` is not in:\n{text}"));
    text[..index].matches('\n').count() + 1
}

const OUTCOME: &str =
    "    outcomes:\n      - {name: lost, external: the service drops it, error: tel.session.Expired}\n";

/// A group named `flaky-link` in the calls file, with this membership and these outcomes.
fn flaky(body: &str) -> Vec<(&'static str, String)> {
    with_groups(&format!("  - name: flaky-link\n{body}"), "")
}

/// C3: the document compiles to exactly one diagnostic, `code` at `path`, cited at the line of the
/// group's own `name:` in the calls file.
fn assert_cited(files: &[(&'static str, String)], code: &str, path: &str) {
    let line = line_of(&files[1].1, "- name: flaky-link");
    assert_eq!(
        diagnosed(files),
        vec![(
            code.to_owned(),
            path.to_owned(),
            "domains/calls.yaml".to_owned(),
            Some(line)
        )]
    );
}

#[test]
fn c3_g2_no_membership() {
    assert_cited(
        &flaky(OUTCOME),
        "ESS-COMMAND-005",
        "outcome_groups.flaky-link",
    );
}

#[test]
fn c3_g3_two_membership_forms() {
    assert_cited(
        &flaky(&format!(
            "    actor: tel.calls.Agent
    domain: tel.calls
{OUTCOME}"
        )),
        "ESS-COMMAND-004",
        "outcome_groups.flaky-link",
    );
}

#[test]
fn c3_g4_an_undeclared_listed_command() {
    assert_cited(
        &flaky(&format!(
            "    commands: [tel.calls.Nope]
{OUTCOME}"
        )),
        "ESS-COMMAND-001",
        "outcome_groups.flaky-link.commands",
    );
}

#[test]
fn c3_g5_an_undeclared_actor() {
    assert_cited(
        &flaky(&format!(
            "    actor: tel.calls.Nobody
{OUTCOME}"
        )),
        "ESS-COMMAND-001",
        "outcome_groups.flaky-link",
    );
}

#[test]
fn c3_g6_an_undeclared_domain() {
    assert_cited(
        &flaky(&format!(
            "    domain: tel.nowhere
{OUTCOME}"
        )),
        "ESS-COMMAND-001",
        "outcome_groups.flaky-link",
    );
}

#[test]
fn c3_g7_except_beside_a_list() {
    assert_cited(
        &flaky(&format!(
            "    commands: [tel.calls.Hold]
    except: [tel.calls.Dial]
{OUTCOME}"
        )),
        "ESS-COMMAND-004",
        "outcome_groups.flaky-link.except",
    );
}

#[test]
fn c3_g8a_an_undeclared_exception() {
    assert_cited(
        &flaky(&format!(
            "    actor: tel.calls.Agent
    except: [tel.calls.Park, tel.calls.Nope]
{OUTCOME}"
        )),
        "ESS-COMMAND-001",
        "outcome_groups.flaky-link.except",
    );
}

#[test]
fn c3_g8b_an_exception_that_excludes_nothing() {
    assert_cited(
        &flaky(&format!(
            "    actor: tel.calls.Agent
    except: [tel.calls.Park, tel.calls.Dial]
{OUTCOME}"
        )),
        "ESS-COMMAND-004",
        "outcome_groups.flaky-link.except",
    );
}

#[test]
fn c3_g9_an_empty_membership() {
    assert_cited(
        &{
            let mut files = flaky(&format!(
                "    actor: tel.calls.Observer
{OUTCOME}"
            ));
            files[1].1 = files[1].1.replace(
                "actors:
",
                "actors:
  - name: tel.calls.Observer
",
            );
            files
        },
        "ESS-COMMAND-007",
        "outcome_groups.flaky-link",
    );
}

#[test]
fn c3_g10_no_outcomes() {
    assert_cited(
        &flaky(
            "    commands: [tel.calls.Hold]
",
        ),
        "ESS-COMMAND-007",
        "outcome_groups.flaky-link.outcomes",
    );
}

#[test]
fn c3_g11_a_repeated_outcome_name() {
    assert_cited(
        &flaky(
            "    commands: [tel.calls.Hold]
    outcomes:
      \
         - {name: lost, external: one, error: tel.session.Expired}
      \
         - {name: lost, external: two, error: tel.session.Expired}
",
        ),
        "ESS-COMMAND-006",
        "outcome_groups.flaky-link.outcomes.lost",
    );
}

#[test]
fn c3_g12_a_blank_cause() {
    assert_cited(
        &flaky(
            "    commands: [tel.calls.Hold]
    outcomes:
      \
         - {name: lost, external: ' ', error: tel.session.Expired}
",
        ),
        "ESS-COMMAND-012",
        "outcome_groups.flaky-link.outcomes.lost.external",
    );
}

#[test]
fn c3_g13_an_undeclared_error_once_for_three_members() {
    assert_cited(
        &flaky(
            "    commands: [tel.calls.Hold, tel.calls.Resume, tel.calls.Dial]
    outcomes:
      \
         - {name: lost, external: it drops, error: tel.session.Nope}
",
        ),
        "ESS-COMMAND-001",
        "outcome_groups.flaky-link.outcomes.lost.error",
    );
}

#[test]
fn c3_g14_a_member_declaring_the_same_outcome() {
    assert_cited(
        &flaky(
            "    commands: [tel.calls.Park]
    outcomes:
      \
         - {name: parked, external: it drops, error: tel.session.Expired}
",
        ),
        "ESS-COMMAND-006",
        "outcome_groups.flaky-link.outcomes.parked",
    );
}

#[test]
fn c3_g15_two_groups_giving_one_command_one_outcome() {
    assert_cited(
        &with_groups(
            "  - name: flaky-link
    commands: [tel.calls.Hold]
    outcomes:
      \
         - {name: lost, external: it drops, error: tel.session.Expired}
",
            "  - name: audited
    commands: [tel.calls.Hold]
    outcomes:
      \
         - {name: lost, external: it drops too, error: tel.session.Expired}
",
        ),
        "ESS-COMMAND-006",
        "outcome_groups.flaky-link.outcomes.lost",
    );
}

#[test]
fn c3_g16_a_group_below_the_format() {
    assert_cited(
        &{
            let mut files = flaky(&format!(
                "    commands: [tel.calls.Hold]
{OUTCOME}"
            ));
            files[0].1 = files[0].1.replace("format: ess/12", "format: ess/10");
            files
        },
        "ESS-COMMAND-009",
        "outcome_groups.flaky-link",
    );
}

#[test]
fn c3_g1_a_group_declared_twice_is_a_command_code_with_no_single_line() {
    let files = with_groups(
        &format!("  - name: twice\n    commands: [tel.calls.Hold]\n{OUTCOME}"),
        &format!("  - name: twice\n    commands: [tel.calls.Dial]\n{OUTCOME}"),
    );
    // `name: twice` is written twice, and the locator reports a line only for a needle written once.
    assert_eq!(
        diagnosed(&files),
        vec![(
            "ESS-COMMAND-006".to_owned(),
            "outcome_groups.twice".to_owned(),
            "<document>".to_owned(),
            None
        )]
    );
}
