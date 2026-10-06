//! Adversary pass 1 on bulk removal and deleting `affects:` entries (beyond10x/ess#452): admitted
//! ess/23 shapes beyond the unit's fixture, each synthesized without a refusal and passed whole by
//! the interpreted model, which is the honest target the suite must admit.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const DELETES: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-deletes.yaml");

const TOKEN_ENTRY: &str = "        affects:
          - entity: demo.auth.Token
            where: user_id == subject.user_id
            deletes: demo.auth.Token
";

fn edited(from: &str, to: &str) -> String {
    assert!(DELETES.contains(from), "the fixture holds:\n{from}");
    DELETES.replacen(from, to, 1)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("set-deletes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn run(suite: &ConformanceSuite, ir: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir),
        )
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// The `DeleteUser` scenario is synthesized without a refusal, reads `absent` rows of `view`
/// absent at least `at_least` times, and the interpreted model passes the whole suite.
fn honest_passes(text: &str, view: &str, at_least: usize) {
    let ir = ir_of(text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    let deleted = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.auth.DeleteUser/outcome/deleted")
        .map(|(_, scenario)| scenario)
        .expect("the deleting branch has a scenario");
    let absent = deleted
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExpectSubjectAbsent { view: read, .. }
                if read.to_string() == view)
        })
        .count();
    assert!(
        absent >= at_least,
        "{at_least} rows of {view} read absent, found {absent}: {:#?}",
        deleted.steps
    );
    let statuses = run(&synthesis.suite, ir);
    let failed: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(
        failed.is_empty(),
        "the interpreted model passes every scenario: {failed:#?}"
    );
}

/// A deleting entry over the subject's own entity: delete a user and every teammate.
#[test]
fn deleting_entry_over_the_subjects_own_entity_is_witnessed_and_passed_honestly() {
    honest_passes(
        &edited(
            TOKEN_ENTRY,
            "        affects:
          - entity: demo.auth.User
            where: team == subject.team
            deletes: demo.auth.User
",
        ),
        "demo.auth.Users",
        2,
    );
}

/// A deleting entry beside a subject that updates.
#[test]
fn deleting_entry_beside_an_updating_subject_is_witnessed_and_passed_honestly() {
    honest_passes(
        &edited(
            "        deletes: demo.auth.User\n        instance: user_id\n",
            "        updates: demo.auth.User\n        instance: user_id\n        sets: {team: gone}\n",
        ),
        "demo.auth.Tokens",
        1,
    );
}

/// Two deleting entries over two entities: the tokens a user owns and its teammates.
#[test]
fn two_deleting_entries_over_two_entities_are_witnessed_and_passed_honestly() {
    honest_passes(
        &edited(
            TOKEN_ENTRY,
            &format!(
                "{TOKEN_ENTRY}          - entity: demo.auth.User
            where: team == subject.team
            deletes: demo.auth.User
"
            ),
        ),
        "demo.auth.Tokens",
        1,
    );
}

/// A setting entry beside a `deletes:` subject: the tokens a deleted user owned are re-scoped.
#[test]
fn setting_entry_beside_a_deleting_subject_is_witnessed_and_passed_honestly() {
    honest_passes(
        &edited(
            "            deletes: demo.auth.Token\n",
            "            sets: {scope: orphaned}\n",
        ),
        "demo.auth.Users",
        1,
    );
}
