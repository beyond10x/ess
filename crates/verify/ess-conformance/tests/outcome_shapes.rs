//! Each outcome shape of `ess/15` has a synthesized scenario that passes against the behaviour its
//! issue describes, and fails against the behaviour the issue reports as wrong
//! (`docs/design/outcome-shapes.md`; beyond10x/ess#144, #145, #150, #151, #152).
//!
//! One model carries all five, and one in-memory target implements it the way the issues describe
//! a correct implementation: a call announced as already ringing, removed at the end of its
//! lifecycle, an unknown identity answered by its own branch, a request accepted with nothing
//! changing, and every command run inside an open session whose user row exists. Each mode of the
//! target then breaks exactly one of those, and exactly the scenario about it fails.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::CommandRef;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/outcome-shapes.yaml");

const PRECONDITION: &str = "preconditions:
  - command: example.call.OpenSession
    as: example.call.Agent
    input: {user_id: 00000000-0000-4000-8000-000000000152}
";

const CART: &str = r"
format: ess/20
system: repro
version: v1
domain: repro.cart
entities:
  - name: repro.cart.Cart
    identity: {name: cart_id, type: String}
    fields: [{name: rev, type: Integer}]
    invariants: ['rev >= 1']
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: repro.cart.Shopper
    may: [repro.cart.OpenCart, repro.cart.CloseCart]
errors:
  - name: repro.cart.CartNotFound
  - name: repro.cart.StaleRev
commands:
  - name: repro.cart.OpenCart
    input: [{name: cart_id, type: String}]
    outcomes:
      - name: opened
        creates: repro.cart.Cart
        instance: cart_id
        emits: [repro.cart.CartOpened]
        payload: {repro.cart.CartOpened: {cart_id: input.cart_id}}
        sets: {rev: '1'}
  - name: repro.cart.CloseCart
    input: [{name: cart_id, type: String}, {name: rev, type: Integer}]
    outcomes:
      - name: missing
        unknown_instance: true
        error: repro.cart.CartNotFound
      - name: stale
        when_subject: {predicate: 'rev != input.rev'}
        error: repro.cart.StaleRev
      - name: closed
        deletes: repro.cart.Cart
        instance: cart_id
        emits: [repro.cart.CartClosed]
        payload: {repro.cart.CartClosed: {cart_id: input.cart_id}}
events:
  - name: repro.cart.CartOpened
    fields: [{name: cart_id, type: String}]
  - name: repro.cart.CartClosed
    fields: [{name: cart_id, type: String}]
views:
  - name: repro.cart.CartById
    source: repro.cart.Cart
    consistency: read_your_writes
    fields:
      - {name: cart_id, type: String}
      - {name: state, type: repro.cart.Cart.State}
      - {name: rev, type: Integer}
";

const STALE_CART: &str = "      - name: stale\n        when_subject: {predicate: 'rev != input.rev'}\n        error: repro.cart.StaleRev\n";

/// A cart stores its revision until deletion. The mutant changes only deletion's row removal.
struct Carts {
    rows: RefCell<BTreeMap<String, Node>>,
    guarded: bool,
    keep_deleted: bool,
}

impl ConformanceTarget for Carts {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("cart", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let id = request.input["cart_id"].as_text().unwrap();
        let mut rows = self.rows.borrow_mut();
        let (outcome, event, error) = match request.command.to_string().as_str() {
            "repro.cart.OpenCart" => {
                rows.insert(id.to_owned(), Node::Number(1_i64.into()));
                ("opened", Some("repro.cart.CartOpened"), None)
            }
            "repro.cart.CloseCart" => match rows.get(id) {
                None => ("missing", None, Some("repro.cart.CartNotFound")),
                Some(rev) if self.guarded && request.input.get("rev") != Some(rev) => {
                    ("stale", None, Some("repro.cart.StaleRev"))
                }
                Some(_) => {
                    if !self.keep_deleted {
                        rows.remove(id);
                    }
                    ("closed", Some("repro.cart.CartClosed"), None)
                }
            },
            other => panic!("unexpected cart command: {other}"),
        };
        let mut result = took(&request.command, outcome);
        if let Some(event) = event {
            emitted(&mut result, event, "cart_id", id);
        }
        result.error = error.map(|name| DeclaredErrorValue::new(name.parse().unwrap()));
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        assert_eq!(request.view.to_string(), "repro.cart.CartById");
        Ok(SemanticViewResult {
            rows: self
                .rows
                .borrow()
                .iter()
                .map(|(id, rev)| {
                    BTreeMap::from([
                        ("cart_id".into(), Node::Text(id.clone())),
                        ("rev".into(), rev.clone()),
                        ("state".into(), Node::Text("Open".into())),
                    ])
                })
                .collect(),
            total: None,
        })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "unused"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported("invocations", "unused"))
    }
}

fn cart_report(model: &str, keep_deleted: bool) -> ess_conformance::report::ConformanceReport {
    let synthesis = synthesis_of(model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &Carts {
                rows: RefCell::default(),
                guarded: model.contains(STALE_CART),
                keep_deleted,
            },
        )
        .into_report()
}

#[test]
fn issue_342_guarded_delete_passes_an_honest_cart_target() {
    let report = cart_report(CART, false);
    assert!(
        report
            .scenarios
            .iter()
            .all(|result| result.status == Status::Passed),
        "{report:#?}"
    );
    assert!(report
        .scenarios
        .iter()
        .any(|result| result.scenario.to_string() == "repro.cart.CloseCart/outcome/stale"));
}

#[test]
fn issue_342_guarded_delete_never_asserts_the_removed_subject() {
    let synthesis = synthesis_of(CART);
    let closed = scenario(&synthesis.suite, "repro.cart.CloseCart/outcome/closed");
    let absent = closed
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExpectSubjectAbsent { .. }))
        .unwrap();
    assert!(
        !closed.steps[absent + 1..]
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectView { .. })),
        "{}",
        text(closed)
    );
    assert!(!synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == "repro.cart.Cart/invariant/after/repro.cart.CloseCart/closed"));
    assert!(closed.steps[absent + 1..].iter().any(|step| matches!(step, ScenarioStep::ExpectOutcome { outcome } if outcome.to_string() == "repro.cart.CloseCart/missing")));
    // Either form: the event also carries the closed cart's identity (beyond10x/ess#273).
    assert!(closed.steps.iter().any(|step| matches!(
        step,
        ScenarioStep::ExpectEvent { .. } | ScenarioStep::ExpectEventValues { .. }
    )));
}

#[test]
fn issue_342_unguarded_control_passes_and_retaining_rows_fails_absence() {
    let control = CART.replace(STALE_CART, "");
    for model in [CART, control.as_str()] {
        let report = cart_report(model, false);
        assert!(
            report
                .scenarios
                .iter()
                .all(|result| result.status == Status::Passed),
            "{report:#?}"
        );
        let mutant = cart_report(model, true);
        let closed = mutant
            .scenarios
            .iter()
            .find(|result| result.scenario.to_string() == "repro.cart.CloseCart/outcome/closed")
            .unwrap();
        assert_eq!(closed.status, Status::Failed, "{closed:#?}");
        assert!(
            closed
                .checks
                .iter()
                .any(|check| check.status == Status::Failed
                    && check.about == "the removed subject is absent from repro.cart.CartById"),
            "{closed:#?}"
        );
    }
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("outcome-shapes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesis_of(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds:\n{}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            |(_, scenario)| scenario,
        )
}

fn text(scenario: &ConformanceScenario) -> String {
    serde_json::to_string_pretty(&scenario.steps).unwrap()
}

/// The steps after the preconditions every scenario opens with.
fn after_preconditions(scenario: &ConformanceScenario) -> &[ScenarioStep] {
    let steps = &scenario.steps;
    match (&steps[0], &steps[1]) {
        (ScenarioStep::ExecuteCommand { command, .. }, ScenarioStep::ExpectOutcome { outcome })
            if command.to_string() == "example.call.OpenSession"
                && outcome.to_string() == "example.call.OpenSession/opened" =>
        {
            &steps[2..]
        }
        _ => panic!(
            "the scenario opens with the precondition:\n{}",
            text(scenario)
        ),
    }
}

// ---- the reference behaviour, and one way to break each construct ---------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// The behaviour every issue describes as correct.
    Correct,
    /// #145 as reported: an id the system never held answers the `wrong_state` branch.
    FoldsUnknownIntoWrongState,
    /// #151 as reported: an ended call stays, in the state it had.
    KeepsEndedRows,
    /// #150 as reported: an offered call starts where the lifecycle starts.
    OffersIntoInitial,
    /// #144 as reported: the accepted no-op writes a row.
    TouchWrites,
}

struct Calls {
    mode: Mode,
    users: RefCell<BTreeMap<String, String>>,
    rows: RefCell<BTreeMap<String, String>>,
}

impl Calls {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            users: RefCell::new(BTreeMap::new()),
            rows: RefCell::new(BTreeMap::new()),
        }
    }
}

fn took(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
    let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
        command.clone(),
        outcome.parse().unwrap(),
    ));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    result
}

fn emitted(result: &mut SemanticCommandResult, event: &str, field: &str, id: &str) {
    let mut observed = ObservedEvent::new(event.parse().unwrap());
    observed
        .payload
        .insert(field.to_owned(), Node::Text(id.to_owned()));
    result.direct_events.push(observed);
}

impl ConformanceTarget for Calls {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("outcome-shapes", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.users.borrow_mut().clear();
        self.rows.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    // One arm per command of the model, as the issues describe it.
    #[allow(clippy::too_many_lines)]
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let name = request.command.to_string();
        let id = |field: &str| {
            request
                .input
                .get(field)
                .and_then(Node::as_text)
                .map(str::to_owned)
                .unwrap_or_default()
        };
        // #152: the implementation cannot take a call command outside an open session.
        if name != "example.call.OpenSession" && self.users.borrow().is_empty() {
            return Ok(SemanticCommandResult::undeclared());
        }
        let command = request.command.clone();
        let mut calls = self.rows.borrow_mut();
        let result = match name.as_str() {
            "example.call.OpenSession" => {
                self.users
                    .borrow_mut()
                    .insert(id("user_id"), "Active".into());
                let mut result = took(&command, "opened");
                emitted(
                    &mut result,
                    "example.call.SessionOpened",
                    "user_id",
                    &id("user_id"),
                );
                result
            }
            "example.call.PlaceCall" => {
                calls.insert(id("call_id"), "Dialing".into());
                let mut result = took(&command, "placed");
                emitted(
                    &mut result,
                    "example.call.CallPlaced",
                    "call_id",
                    &id("call_id"),
                );
                result
            }
            "example.call.OfferCall" => {
                let state = if self.mode == Mode::OffersIntoInitial {
                    "Dialing"
                } else {
                    "Ringing"
                };
                calls.insert(id("call_id"), state.into());
                let mut result = took(&command, "offered");
                emitted(
                    &mut result,
                    "example.call.CallOffered",
                    "call_id",
                    &id("call_id"),
                );
                result
            }
            "example.call.RingCall" => match calls.get(&id("call_id")).map(String::as_str) {
                Some("Dialing") => {
                    calls.insert(id("call_id"), "Ringing".into());
                    let mut result = took(&command, "rang");
                    emitted(
                        &mut result,
                        "example.call.CallRang",
                        "call_id",
                        &id("call_id"),
                    );
                    result
                }
                _ => took(&command, "not-dialing"),
            },
            "example.call.AnswerCall" => match calls.get(&id("call_id")).cloned() {
                None if self.mode == Mode::FoldsUnknownIntoWrongState => {
                    took(&command, "not-ringing")
                }
                None => {
                    let mut result = took(&command, "no-such-call");
                    result.error = Some(DeclaredErrorValue::new(
                        "example.call.CallNotFound".parse().unwrap(),
                    ));
                    result
                }
                Some(state) if state == "Ringing" => {
                    calls.insert(id("call_id"), "Connected".into());
                    let mut result = took(&command, "answered");
                    emitted(
                        &mut result,
                        "example.call.CallAnswered",
                        "call_id",
                        &id("call_id"),
                    );
                    result
                }
                Some(_) => took(&command, "not-ringing"),
            },
            "example.call.EndCall" => {
                if calls.contains_key(&id("call_id")) {
                    if self.mode != Mode::KeepsEndedRows {
                        calls.remove(&id("call_id"));
                    }
                    let mut result = took(&command, "ended");
                    emitted(
                        &mut result,
                        "example.call.CallEnded",
                        "call_id",
                        &id("call_id"),
                    );
                    result
                } else {
                    let mut result = took(&command, "no-such-call");
                    result.error = Some(DeclaredErrorValue::new(
                        "example.call.CallNotFound".parse().unwrap(),
                    ));
                    result
                }
            }
            "example.call.Touch" => {
                if self.mode == Mode::TouchWrites {
                    calls.insert(
                        "00000000-0000-4000-8000-000000000144".into(),
                        "Dialing".into(),
                    );
                }
                took(&command, "accepted")
            }
            _ => panic!("unexpected command {name}"),
        };
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let (rows, identity) = match request.view.to_string().as_str() {
            "example.call.Calls" => (self.rows.borrow().clone(), "call_id"),
            "example.call.Users" => (self.users.borrow().clone(), "user_id"),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult {
            rows: rows
                .into_iter()
                .map(|(id, state)| {
                    BTreeMap::from([
                        (identity.to_owned(), Node::Text(id)),
                        ("state".to_owned(), Node::Text(state)),
                    ])
                })
                .collect(),
            total: None,
        })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "the model declares none",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported("invocations", "unused"))
    }
}

/// Every scenario's status against `mode`, by id.
fn run(suite: &ConformanceSuite, mode: Mode) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Calls::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn failed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

// ---- the suite ------------------------------------------------------------------------------

#[test]
fn every_scenario_passes_against_the_behaviour_the_issues_describe() {
    let synthesis = synthesis_of(MODEL);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    let statuses = run(&synthesis.suite, Mode::Correct);
    assert!(
        failed(&statuses).is_empty(),
        "every scenario passes against the correct target: {:#?}",
        failed(&statuses)
    );
}

#[test]
fn issue_145_an_unknown_identity_takes_its_own_branch() {
    let synthesis = synthesis_of(MODEL);
    let id = "example.call.AnswerCall/outcome/no-such-call";
    let unknown = scenario(&synthesis.suite, id);
    let steps = after_preconditions(unknown);
    let ScenarioStep::ExecuteCommand { command, .. } = &steps[0] else {
        panic!(
            "the command is sent at once, arranging nothing:\n{}",
            text(unknown)
        );
    };
    assert_eq!(command.to_string(), "example.call.AnswerCall");
    let json = text(unknown);
    assert!(json.contains("\"no-such-call\""), "{json}");
    assert!(json.contains("example.call.CallNotFound"), "{json}");
    // The accepted wrong-state branch keeps its scenarios in the states it answers, and no
    // scenario claims it for an identity no record carries.
    assert!(!synthesis
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == "example.call.AnswerCall/outcome/not-ringing"));

    let wrong = run(&synthesis.suite, Mode::FoldsUnknownIntoWrongState);
    assert_eq!(failed(&wrong), vec![id]);
}

#[test]
fn issue_151_a_deleted_row_is_absent_and_a_later_command_meets_an_unknown_identity() {
    let synthesis = synthesis_of(MODEL);
    let id = "example.call.EndCall/outcome/ended";
    let ended = scenario(&synthesis.suite, id);
    let steps = after_preconditions(ended);
    assert!(
        steps.iter().any(|step| matches!(step,
            ScenarioStep::ExpectSubjectAbsent { view, .. } if view.to_string() == "example.call.Calls")),
        "absence from the entity's immediate view is asserted:\n{}",
        text(ended)
    );
    let sent = steps
        .iter()
        .filter(|step| matches!(step,
            ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "example.call.EndCall"))
        .count();
    assert_eq!(
        sent,
        2,
        "the deleted identity is sent again:\n{}",
        text(ended)
    );
    assert!(
        text(ended).contains("\"no-such-call\""),
        "the second send meets the unknown-instance answer:\n{}",
        text(ended)
    );

    let wrong = run(&synthesis.suite, Mode::KeepsEndedRows);
    assert!(failed(&wrong).contains(&id), "{:#?}", failed(&wrong));
    assert!(
        failed(&wrong)
            .iter()
            .all(|failing| failing.contains("EndCall")),
        "only the deletion's scenarios fail: {:#?}",
        failed(&wrong)
    );
}

#[test]
fn issue_150_a_call_offered_as_ringing_is_ringing_and_arranges_what_follows() {
    let synthesis = synthesis_of(MODEL);
    let id = "example.call.OfferCall/outcome/offered";
    let offered = scenario(&synthesis.suite, id);
    let json = text(offered);
    assert!(
        json.contains("\"Ringing\""),
        "the created row is asserted in `Ringing`:\n{json}"
    );
    // A move from `Ringing` is arranged through the creation into it: one command, not two.
    let answered = scenario(
        &synthesis.suite,
        "example.call.Call/transition/answer/by/example.call.AnswerCall/answered",
    );
    let arranged: Vec<String> = after_preconditions(answered)
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(
        arranged,
        vec!["example.call.OfferCall", "example.call.AnswerCall"],
        "{}",
        text(answered)
    );

    let wrong = run(&synthesis.suite, Mode::OffersIntoInitial);
    assert!(failed(&wrong).contains(&id), "{:#?}", failed(&wrong));
}

#[test]
fn issue_144_an_accepted_no_op_leaves_every_immediate_view_as_it_was() {
    let synthesis = synthesis_of(MODEL);
    let id = "example.call.Touch/outcome/accepted";
    let touched = scenario(&synthesis.suite, id);
    let steps = after_preconditions(touched);
    for view in ["example.call.Calls", "example.call.Users"] {
        assert!(
            steps.iter().any(|step| matches!(step,
                ScenarioStep::SnapshotView { view: named } if named.to_string() == view)),
            "{view} is snapshotted before:\n{}",
            text(touched)
        );
        assert!(
            steps.iter().any(|step| matches!(step,
                ScenarioStep::ExpectViewUnchanged { view: named } if named.to_string() == view)),
            "{view} is compared after:\n{}",
            text(touched)
        );
    }
    assert!(steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::ExpectNoError)));
    assert!(steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::ExpectNoEvents)));

    let wrong = run(&synthesis.suite, Mode::TouchWrites);
    assert_eq!(failed(&wrong), vec![id]);
}

#[test]
fn issue_152_every_scenario_runs_inside_the_preconditions() {
    let synthesis = synthesis_of(MODEL);
    for scenario in synthesis.suite.scenarios.values() {
        after_preconditions(scenario);
    }
    // Without the declaration, a target that cannot run outside a session fails at step 1.
    let without = synthesis_of(&MODEL.replace(PRECONDITION, ""));
    assert_eq!(
        without.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    let statuses = run(&without.suite, Mode::Correct);
    assert!(
        failed(&statuses).len() > 1,
        "the implementation's ambient session is what the preconditions state: {statuses:#?}"
    );
}

#[test]
fn the_coverage_suite_takes_the_coverage_major() {
    use ess_conformance::coverage::{Origins, Scope};
    let input = ess_conformance::coverage_build::build(
        &ir_of(MODEL),
        &[],
        Scope::System,
        Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/35"
    );
}

#[test]
fn an_older_suite_label_is_refused_before_any_target_activity() {
    let mut suite = synthesis_of(MODEL).suite;
    suite.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/19").unwrap();
    suite.provenance.scenario_initial_state = None;
    let error = AdmittedSuite::from_suite(&suite).expect_err("absence needs suite/22");
    assert!(error.to_string().contains("suite/22"), "{error}");
}

#[test]
fn a_model_using_none_of_the_constructs_keeps_its_suite_format() {
    let ir = ir_of(
        "format: ess/15
system: plain
version: v1
domain: plain.core
events:
  - {name: plain.core.Pinged}
commands:
  - name: plain.core.Ping
    outcomes:
      - {name: pinged, emits: [plain.core.Pinged]}
",
    );
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert_eq!(synthesis.suite.provenance.suite_version.major(), 34);
}
