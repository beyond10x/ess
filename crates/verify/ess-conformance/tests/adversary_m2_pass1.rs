//! Adversary pass 1 for an external refusal answered before the row (`before_row: true`, ess/25,
//! beyond10x/ess#534).
//!
//! The design (`docs/design/cross-record-and-stored-field-guards.md`, "An external refusal answered
//! before the row") places the branch at step 2, before existence (step 3) and the held state
//! (step 4), and among the input-guarded refusals in declaration order. Each case here synthesizes a
//! suite from a model that marks the branch, and runs it on the model interpreter of an IR a
//! non-conforming target would implement: the suite must fail it somewhere.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// beyond10x/ess#534's `SignIn`.
const SIGN_IN: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/external-before-row.yaml");

/// A creation that authenticates its caller before it looks for the row it would create: the
/// credential answers before `existing_instance:` (step 3), so an unauthenticated caller does not
/// learn whether the seat already exists.
const REGISTER: &str = r#"format: ess/25
system: demo
version: v1
domain: demo.desk
entities:
  - name: demo.desk.Seat
    identity: {name: user_id, type: String}
    fields:
      - {name: call_ref, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: demo.desk.Client, may: [demo.desk.Register]}
commands:
  - name: demo.desk.Register
    input:
      - {name: user_id, type: String}
      - {name: secret, type: String}
    outcomes:
      - name: bad-credential
        external: secret is not the digest the service derives from user_id
        before_row: true
        error: demo.desk.BadCredential
      - name: taken
        existing_instance: true
        error: demo.desk.Taken
      - name: registered
        creates: demo.desk.Seat
        instance: user_id
        sets: {call_ref: ""}
        emits: [demo.desk.Registered]
        payload: {demo.desk.Registered: {user_id: input.user_id}}
errors:
  - name: demo.desk.BadCredential
    summary: The secret is not the one the service derives for the user.
  - name: demo.desk.Taken
    summary: A seat already carries the identity.
events:
  - {name: demo.desk.Registered, fields: [{name: user_id, type: String}]}
views:
  - name: demo.desk.Seats
    source: demo.desk.Seat
    consistency: read_your_writes
    fields:
      - {name: user_id, type: String}
      - {name: call_ref, type: String}
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble(vec![(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

/// Every scenario of `suite` run on the model interpreter of `ir`, by id.
fn run_on(ir: EssIr, suite: &ConformanceSuite) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// `ir` with `command` rewritten by `change`.
fn rewritten(ir: &EssIr, command: &str, change: impl Fn(&mut ResolvedCommand)) -> EssIr {
    ir.with_commands_rewritten(|resolved| {
        let mut resolved = resolved.clone();
        if resolved.name.to_string() == command {
            change(&mut resolved);
        }
        resolved
    })
}

/// A target that looks the row up before it asks the provider answers `taken` for a seat that
/// already exists, and so tells an unauthenticated caller the seat exists. The design's
/// `unknown_instance:` bullet says the external answers before existence, so "the answer says
/// nothing about existence"; the suite synthesized for the model must fail that target. Its only
/// witnesses of `bad-credential` send an identity no row carries, where a row-first target answers
/// the external too, so the suite passes it.
#[test]
fn before_row_external_is_witnessed_on_an_identity_a_row_already_carries() {
    let result = synthesis(REGISTER);
    let id = "demo.desk.Register/outcome/bad-credential";
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "the external refusal has a scenario: {:#?}",
        result.refusals
    );
    let statuses = run_on(ir(REGISTER), &result.suite);
    assert_eq!(not_passed(&statuses), Vec::<String>::new());

    // The target reading the row first: the external answered at step 6, after existence.
    let faulty = rewritten(&ir(REGISTER), "demo.desk.Register", |command| {
        for outcome in &mut command.outcomes {
            if outcome.name.as_str() == "bad-credential" {
                assert!(outcome.before_row, "`bad-credential` is marked in the IR");
                outcome.before_row = false;
            }
        }
    });
    let faulty = run_on(faulty, &result.suite);
    assert_ne!(
        not_passed(&faulty),
        Vec::<String>::new(),
        "a target answering `existing_instance:` before the provider passes every scenario: \
         {faulty:#?}"
    );
}

/// The design orders step 2 by declaration: the first declared step-2 branch that answers is an
/// input refusal whose guard holds or a `before_row` external its provider takes. With
/// `empty-secret` declared after `bad-credential`, a forced provider on an empty secret answers
/// `bad-credential`. A target that asks the provider only after every input refusal answers
/// `empty-secret` there; the synthesized suite must fail it.
#[test]
fn before_row_external_is_witnessed_before_an_input_refusal_declared_after_it() {
    let text = SIGN_IN
        .replace(
            "        error: demo.desk.BadCredential\n",
            "        error: demo.desk.BadCredential\n      - name: empty-secret\n        when: \
             secret == \"\"\n        error: demo.desk.EmptySecret\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.desk.EmptySecret\n    summary: The secret is empty.\n",
        );
    assert_ne!(text, SIGN_IN);
    let result = synthesis(&text);
    let statuses = run_on(ir(&text), &result.suite);
    assert_eq!(not_passed(&statuses), Vec::<String>::new());

    // The target asking the provider after the input refusals: `empty-secret` read first.
    let faulty = rewritten(&ir(&text), "demo.desk.SignIn", |command| {
        let at = command
            .outcomes
            .iter()
            .position(|outcome| outcome.name.as_str() == "empty-secret")
            .expect("`empty-secret` is declared");
        let refusal = command.outcomes.remove(at);
        command.outcomes.insert(0, refusal);
    });
    let faulty = run_on(faulty, &result.suite);
    assert_ne!(
        not_passed(&faulty),
        Vec::<String>::new(),
        "a target answering the input refusals before the `before_row` external passes every \
         scenario: {faulty:#?}"
    );
}
