//! Set effects over filtered instances (ess/16, beyond10x/ess#167, #175) in the generated
//! artifacts: the documentation says which rows a branch changes, and every other artifact —
//! `OpenAPI` included — is generated as it is for a branch naming no instance, without a panic.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");

const COMPONENT: &str = "components:\n  - component: desk-service\n    owns:\n      domains: [demo.desk]\n    accepts:\n      commands: [demo.desk.Open, demo.desk.Park, demo.desk.Close, demo.desk.EndTeam, demo.desk.NoteTeam, demo.desk.Invite]\n    publishes:\n      events: [demo.desk.SessionOpened, demo.desk.SessionParked, demo.desk.SessionClosed, demo.desk.TeamEnded, demo.desk.TeamNoted, demo.desk.Invited]\n";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(&format!("{MODEL}{COMPONENT}")).unwrap();
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn artifacts(ir: &EssIr, keep: impl Fn(&str) -> bool) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| keep(path))
        .map(|(path, artifact)| format!("== {path}\n{}", artifact.contents))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_documentation_says_which_rows_a_set_effect_changes() {
    let docs = artifacts(&ir(), |path| {
        std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    });
    assert!(
        docs.contains("team == input.team"),
        "the filter is published: {docs}"
    );
    assert!(
        docs.contains("team == subject.team"),
        "the `affects:` filter is published: {docs}"
    );
    assert!(
        docs.contains("every `demo.desk.Session`"),
        "a set effect is described as changing every selected row: {docs}"
    );
}

#[test]
fn every_artifact_is_generated_without_a_panic() {
    let all = artifacts(&ir(), |_| true);
    assert!(all.contains("demo.desk.EndTeam"));
}

#[test]
fn issue_229_the_documentation_says_an_affects_entry_moves_its_rows() {
    let from = "            where: team == subject.team\n";
    assert!(MODEL.contains(from), "{from}");
    let model = MODEL
        .replacen("format: ess/16", "format: ess/22", 1)
        .replacen(
            from,
            "            where: team == subject.team\n            moves: demo.desk.Session.park\n",
            1,
        );
    let raw = RawSpecFile::parse(&format!("{model}{COMPONENT}")).unwrap();
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let docs = artifacts(&ir, |path| {
        std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    });
    assert!(
        docs.contains(
            "moves every `demo.desk.Session` the filter `team == subject.team` selects to `Parked`"
        ),
        "the entry's move is published: {docs}"
    );
}
