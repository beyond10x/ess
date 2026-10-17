//! Adversary, pass 1, for ess/25 create-or-update selected by stored fields (beyond10x/ess#530).
//!
//! The unit's fixture gives every `SignIn` branch a subject of its own. A create-or-update command
//! may also carry a branch that names no subject: a refusal guarded by the stored row
//! (`when_subject:` with `error:`), or a refusal guarded by the input (`when:` with `error:`). For
//! such a branch the subject-fact strategy opens its scenario with the absent-subject witness
//! (`synthesize/subject_fact.rs`, `absent`): the command sent once for an identity no row carries,
//! asserting that no declared event is published and that no row appears under that identity. On
//! a command whose `unknown_instance:` branch creates the row, the specification says the
//! opposite: the creating branch answers that send, publishes its event and stores the row. The
//! interpreted target, executing the same model, then fails the scenario.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/create-or-update-stored-fields.yaml"
);

/// The fixture's `reopened-busy` move, replaced below by a refusal naming no subject.
const REOPENED_BUSY: &str = "      - name: reopened-busy
        when_subject: {predicate: {all: [state == Busy, call_ref != \"\"]}}
        moves: demo.desk.Seat.reopen
        instance: user_id
        emits: [demo.desk.Left]
        payload: {demo.desk.Left: {user_id: input.user_id}}
";

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    statuses
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// Synthesizes `text` and runs every scenario against the interpreter of the same model.
fn assert_interpreter_passes(label: &str, text: &str, refusal: &str) {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let id = format!("demo.desk.SignIn/outcome/{refusal}");
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "{label}: {id} is synthesized; refusals: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        not_passed(model, &synthesis.suite),
        Vec::<String>::new(),
        "{label}: the interpreted target, executing the model the suite was synthesized from, \
         passes every scenario"
    );
}

/// The stored-row refusal: a seat busy on a call refuses the sign-in, reading the call reference
/// from the row into its error.
#[test]
fn a_stored_field_refusal_beside_the_creating_branch_passes_the_interpreter() {
    let text = replaced(
        MODEL,
        REOPENED_BUSY,
        "      - name: busy
        when_subject: {predicate: {all: [state == Busy, call_ref != \"\"]}}
        error: demo.desk.InCall
        payload: {demo.desk.InCall: {call_ref: {subject: call_ref}}}
",
    );
    let text = replaced(
        &text,
        "errors:\n",
        "errors:
  - name: demo.desk.InCall
    summary: The seat is on a call.
    fields: [{name: call_ref, type: String}]
",
    );
    assert_interpreter_passes("stored-field refusal", &text, "busy");
}

/// The input refusal: an empty secret is refused before any row is read (step 2 of the precedence
/// order), whether or not a seat carries the identity.
#[test]
fn an_input_refusal_beside_the_creating_branch_passes_the_interpreter() {
    let text = replaced(
        MODEL,
        "    outcomes:\n      - name: reopened-busy\n",
        "    outcomes:
      - name: no-secret
        when: secret == \"\"
        error: demo.desk.NoSeat
      - name: reopened-busy\n",
    );
    assert_interpreter_passes("input refusal", &text, "no-secret");
}
