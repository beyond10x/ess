//! Adversarial cases for the ess/15 outcome shapes (`docs/design/outcome-shapes.md`).
//!
//! Each case drives a synthesized suite against an in-memory target that behaves the way the design
//! page says a correct (or a specifically wrong) implementation behaves, and asserts what the page
//! promises.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use ess_primitives::Number;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/outcome-shapes.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn took(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    result
}

fn refused(command: &CommandRef, outcome: &str, error: &str) -> SemanticCommandResult {
    let mut result = took(command, outcome);
    result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
    result
}

fn emit(result: &mut SemanticCommandResult, event: &str, field: &str, value: Node) {
    let mut observed = ObservedEvent::new(event.parse().unwrap());
    observed.payload.insert(field.to_owned(), value);
    result.direct_events.push(observed);
}

fn text_of(input: &BTreeMap<String, Node>, field: &str) -> String {
    input
        .get(field)
        .and_then(Node::as_text)
        .map(str::to_owned)
        .unwrap_or_default()
}

macro_rules! unused_target_methods {
    () => {
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
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            Ok(ImplementationIdentity::new("outcome-shapes-adversary", "1"))
        }
        fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
            Ok(())
        }
    };
}

// ---- #151: a second command for a deleted identity -----------------------------------------

/// The fixture model's behaviour, except that `EndCall` keeps a tombstone for every call it ended
/// and, sent again for one, answers the unknown-instance branch **and publishes `CallEnded` a
/// second time**. For an identity no record carries the design requires no event (the
/// `unknown_instance` scenario asserts `expect_no_event` for every declared event).
struct Tombstones {
    users: RefCell<BTreeSet<String>>,
    rows: RefCell<BTreeMap<String, String>>,
    ended: RefCell<BTreeSet<String>>,
}

impl ConformanceTarget for Tombstones {
    unused_target_methods!();
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.users.borrow_mut().clear();
        self.rows.borrow_mut().clear();
        self.ended.borrow_mut().clear();
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let name = request.command.to_string();
        let command = request.command.clone();
        let id = |field: &str| text_of(&request.input, field);
        let mut calls = self.rows.borrow_mut();
        let call = |event: &str, outcome: &str, id: String| {
            let mut result = took(&command, outcome);
            emit(&mut result, event, "call_id", Node::Text(id));
            result
        };
        Ok(match name.as_str() {
            "example.call.OpenSession" => {
                self.users.borrow_mut().insert(id("user_id"));
                let mut result = took(&command, "opened");
                emit(
                    &mut result,
                    "example.call.SessionOpened",
                    "user_id",
                    Node::Text(id("user_id")),
                );
                result
            }
            "example.call.PlaceCall" => {
                calls.insert(id("call_id"), "Dialing".into());
                call("example.call.CallPlaced", "placed", id("call_id"))
            }
            "example.call.OfferCall" => {
                calls.insert(id("call_id"), "Ringing".into());
                call("example.call.CallOffered", "offered", id("call_id"))
            }
            "example.call.RingCall" => match calls.get(&id("call_id")).map(String::as_str) {
                Some("Dialing") => {
                    calls.insert(id("call_id"), "Ringing".into());
                    call("example.call.CallRang", "rang", id("call_id"))
                }
                _ => took(&command, "not-dialing"),
            },
            "example.call.AnswerCall" => match calls.get(&id("call_id")).cloned() {
                None => refused(&command, "no-such-call", "example.call.CallNotFound"),
                Some(state) if state == "Ringing" => {
                    calls.insert(id("call_id"), "Connected".into());
                    call("example.call.CallAnswered", "answered", id("call_id"))
                }
                Some(_) => took(&command, "not-ringing"),
            },
            "example.call.EndCall" => {
                if calls.remove(&id("call_id")).is_some() {
                    self.ended.borrow_mut().insert(id("call_id"));
                    call("example.call.CallEnded", "ended", id("call_id"))
                } else {
                    let mut result = refused(&command, "no-such-call", "example.call.CallNotFound");
                    if self.ended.borrow().contains(&id("call_id")) {
                        // The defect: a deleted identity answers "not found" and ends it again.
                        emit(
                            &mut result,
                            "example.call.CallEnded",
                            "call_id",
                            Node::Text(id("call_id")),
                        );
                    }
                    result
                }
            }
            "example.call.Touch" => took(&command, "accepted"),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "example.call.Calls" => self
                .rows
                .borrow()
                .iter()
                .map(|(id, state)| {
                    BTreeMap::from([
                        ("call_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text(state.clone())),
                    ])
                })
                .collect(),
            "example.call.Users" => self
                .users
                .borrow()
                .iter()
                .map(|id| {
                    BTreeMap::from([
                        ("user_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Active".to_owned())),
                    ])
                })
                .collect(),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult { rows, total: None })
    }
}

#[test]
fn adversary_a_deleted_identity_answered_with_the_event_again_is_caught() {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(MODEL)).suite;
    let target = Tombstones {
        users: RefCell::default(),
        rows: RefCell::default(),
        ended: RefCell::default(),
    };
    let result = statuses(&suite, &target);
    let ended = "example.call.EndCall/outcome/ended";
    assert!(result.contains_key(ended), "{result:#?}");
    assert_ne!(
        result[ended],
        Status::Passed,
        "a deleted identity is answered exactly as an unknown one (design, #151), and an unknown \
         identity publishes no event; this target publishes `CallEnded` again on the second send, \
         and every scenario passes: {result:#?}"
    );
}

// ---- #150 × aggregate views: a creation `into:` a state is not where route search starts ------

const AGGREGATE: &str = "format: ess/15
system: example
version: v1
domain: example.call
types:
  - {name: example.call.CallId, kind: newtype, of: Uuid}
entities:
  - name: example.call.Call
    identity: {name: call_id, type: example.call.CallId}
    fields:
      - {name: queue, type: String}
    lifecycle:
      initial: Dialing
      states: [Dialing, Ringing]
      terminal: [Ringing]
      transitions:
        - {name: ring, from: [Dialing], to: Ringing}
events:
  - name: example.call.CallPlaced
    fields: [{name: call_id, type: example.call.CallId}]
  - name: example.call.CallOffered
    fields: [{name: call_id, type: example.call.CallId}]
  - name: example.call.CallRang
    fields: [{name: call_id, type: example.call.CallId}]
actors:
  - name: example.call.Agent
    may: [example.call.PlaceCall, example.call.OfferCall, example.call.RingCall]
commands:
  - name: example.call.PlaceCall
    input:
      - {name: call_id, type: example.call.CallId}
      - {name: queue, type: String}
    outcomes:
      - name: placed
        creates: example.call.Call
        instance: call_id
        emits: [example.call.CallPlaced]
        payload:
          example.call.CallPlaced: {call_id: input.call_id}
        sets: {queue: input.queue}
  - name: example.call.OfferCall
    input:
      - {name: call_id, type: example.call.CallId}
      - {name: queue, type: String}
    outcomes:
      - name: offered
        creates: example.call.Call
        instance: call_id
        into: Ringing
        emits: [example.call.CallOffered]
        payload:
          example.call.CallOffered: {call_id: input.call_id}
        sets: {queue: input.queue}
  - name: example.call.RingCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - name: rang
        moves: example.call.Call.ring
        instance: call_id
        emits: [example.call.CallRang]
        payload:
          example.call.CallRang: {call_id: input.call_id}
      - {name: not-dialing, wrong_state: true, refuses: false}
views:
  - name: example.call.DialingByQueue
    source: example.call.Call
    consistency: read_your_writes
    filter: state == Dialing
    group_by: [queue]
    fields:
      - {name: queue, type: String}
      - {name: calls, type: Integer, aggregate: {count: {}}}
";

/// The aggregate model, implemented as it is declared: a placed call is `Dialing`, an offered one
/// `Ringing`, and the view counts the `Dialing` calls of each queue.
#[derive(Default)]
struct Queues {
    rows: RefCell<BTreeMap<String, (String, String)>>,
}

impl ConformanceTarget for Queues {
    unused_target_methods!();
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let id = text_of(&request.input, "call_id");
        let queue = text_of(&request.input, "queue");
        let mut rows = self.rows.borrow_mut();
        let call = |event: &str, outcome: &str| {
            let mut result = took(&command, outcome);
            emit(&mut result, event, "call_id", Node::Text(id.clone()));
            result
        };
        Ok(match request.command.to_string().as_str() {
            "example.call.PlaceCall" => {
                rows.insert(id.clone(), ("Dialing".into(), queue));
                call("example.call.CallPlaced", "placed")
            }
            "example.call.OfferCall" => {
                rows.insert(id.clone(), ("Ringing".into(), queue));
                call("example.call.CallOffered", "offered")
            }
            "example.call.RingCall" => match rows.get_mut(&id) {
                Some((state, _)) if state == "Dialing" => {
                    *state = "Ringing".into();
                    call("example.call.CallRang", "rang")
                }
                _ => took(&command, "not-dialing"),
            },
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        assert_eq!(request.view.to_string(), "example.call.DialingByQueue");
        let mut counts: BTreeMap<String, i64> = BTreeMap::new();
        for (state, queue) in self.rows.borrow().values() {
            if state == "Dialing" {
                *counts.entry(queue.clone()).or_default() += 1;
            }
        }
        Ok(SemanticViewResult {
            rows: counts
                .into_iter()
                .map(|(queue, calls)| {
                    BTreeMap::from([
                        ("queue".to_owned(), Node::Text(queue)),
                        ("calls".to_owned(), Node::from(Number::from(calls))),
                    ])
                })
                .collect(),
            total: None,
        })
    }
}

#[test]
fn adversary_an_aggregate_arranged_through_a_creation_into_a_state_passes_against_a_correct_target()
{
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(AGGREGATE));
    let id = "example.call.DialingByQueue/aggregate";
    let Some(scenario) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
    else {
        panic!(
            "the aggregate scenario is synthesized; refusals: {:#?}",
            synthesis.refusals
        );
    };
    let result = statuses(&synthesis.suite, &Queues::default());
    assert_eq!(
        result[id],
        Status::Passed,
        "the aggregate scenario fails against the implementation the model declares; it arranged \
         its rows as:\n{}",
        serde_json::to_string_pretty(&scenario.steps).unwrap()
    );
}

// ---- #152: a precondition the target refuses ------------------------------------------------

const REFUSED_PRECONDITION: &str = "format: ess/15
system: example
version: v1
preconditions:
  - command: example.s.OpenSession
    as: example.s.Agent
    input: {user_id: 00000000-0000-4000-8000-000000000001, locked: true, resume: false}
domain: example.s
types:
  - {name: example.s.UserId, kind: newtype, of: Uuid}
entities:
  - name: example.s.User
    identity: {name: user_id, type: example.s.UserId}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: example.s.SessionOpened
    fields: [{name: user_id, type: example.s.UserId}]
  - {name: example.s.SessionResumed}
  - {name: example.s.Pinged}
errors:
  - {name: example.s.Locked}
actors:
  - {name: example.s.Agent, may: [example.s.OpenSession, example.s.Ping]}
commands:
  - name: example.s.OpenSession
    input:
      - {name: user_id, type: example.s.UserId}
      - {name: locked, type: Boolean}
      - {name: resume, type: Boolean}
    outcomes:
      - name: locked-out
        when: locked == true
        error: example.s.Locked
      - name: resumed
        when: resume == true
        emits: [example.s.SessionResumed]
      - name: opened
        creates: example.s.User
        instance: user_id
        emits: [example.s.SessionOpened]
        payload:
          example.s.SessionOpened: {user_id: input.user_id}
  - name: example.s.Ping
    outcomes:
      - {name: pinged, emits: [example.s.Pinged]}
views:
  - name: example.s.Users
    source: example.s.User
    consistency: read_your_writes
    fields:
      - {name: user_id, type: example.s.UserId}
      - {name: state, type: example.s.User.State}
";

/// Implements the model exactly: a locked request is refused with `Locked`.
#[derive(Default)]
struct Sessions {
    users: RefCell<BTreeSet<String>>,
}

impl ConformanceTarget for Sessions {
    unused_target_methods!();
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.users.borrow_mut().clear();
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let flag = |field: &str| request.input.get(field) == Some(&Node::Bool(true));
        Ok(match request.command.to_string().as_str() {
            "example.s.OpenSession" if flag("locked") => {
                refused(&command, "locked-out", "example.s.Locked")
            }
            "example.s.OpenSession" if flag("resume") => {
                let mut result = took(&command, "resumed");
                result.direct_events.push(ObservedEvent::new(
                    "example.s.SessionResumed".parse().unwrap(),
                ));
                result
            }
            "example.s.OpenSession" => {
                let id = text_of(&request.input, "user_id");
                self.users.borrow_mut().insert(id.clone());
                let mut result = took(&command, "opened");
                emit(
                    &mut result,
                    "example.s.SessionOpened",
                    "user_id",
                    Node::Text(id),
                );
                result
            }
            "example.s.Ping" => {
                let mut result = took(&command, "pinged");
                result
                    .direct_events
                    .push(ObservedEvent::new("example.s.Pinged".parse().unwrap()));
                result
            }
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self
                .users
                .borrow()
                .iter()
                .map(|id| {
                    BTreeMap::from([
                        ("user_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Active".to_owned())),
                    ])
                })
                .collect(),
            total: None,
        })
    }
}

#[test]
fn adversary_a_refused_precondition_never_lets_a_scenario_pass() {
    // Either the specification is refused (a precondition whose input selects a refusal), or every
    // scenario fails as setup when the target refuses the precondition. Passing is the one answer
    // the design rules out: "Their outcomes must be the declared success branch; a precondition
    // that is refused fails the scenario as setup".
    let raw = RawSpecFile::parse(REFUSED_PRECONDITION).expect("the model parses");
    let spec = match Specification::assemble([(Source::new("adversary.yaml"), raw)]) {
        Ok(spec) => spec,
        Err(errors) => {
            // Refusing the precondition at validation is an acceptable answer; any other refusal
            // is a defect of this model, not a finding.
            assert!(
                errors.to_string().contains("preconditions"),
                "the model is refused for another reason: {errors}"
            );
            return;
        }
    };
    let ir = compile(&spec, &SourceMap::new()).expect("the model compiles");
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let result = statuses(&suite, &Sessions::default());
    let passed: Vec<&String> = result
        .iter()
        .filter(|(_, status)| **status == Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(
        passed.is_empty(),
        "every scenario ran after a refused precondition, and these passed: {passed:#?}"
    );
}

// ---- #152 × fixture inputs: the precondition's own command -----------------------------------

const FIXTURE_PRECONDITION: &str = "format: ess/15
system: example
version: v1
preconditions:
  - command: example.f.OpenSession
    as: example.f.Agent
    input: {user_id: {fixture: session-user}}
domain: example.f
types:
  - {name: example.f.UserId, kind: newtype, of: Uuid}
entities:
  - name: example.f.User
    identity: {name: user_id, type: example.f.UserId}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: example.f.SessionOpened
    fields: [{name: user_id, type: example.f.UserId}]
actors:
  - {name: example.f.Agent, may: [example.f.OpenSession]}
commands:
  - name: example.f.OpenSession
    input:
      - {name: user_id, type: example.f.UserId}
    fixture_inputs: {user_id: session-user}
    outcomes:
      - name: opened
        creates: example.f.User
        instance: user_id
        emits: [example.f.SessionOpened]
        payload:
          example.f.SessionOpened: {user_id: input.user_id}
views:
  - name: example.f.Users
    source: example.f.User
    consistency: read_your_writes
    fields:
      - {name: user_id, type: example.f.UserId}
      - {name: state, type: example.f.User.State}
";

#[test]
fn adversary_the_precondition_commands_own_scenario_does_not_create_the_identity_it_already_created(
) {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(FIXTURE_PRECONDITION));
    let id = "example.f.OpenSession/outcome/opened";
    let Some(scenario) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string().starts_with("example.f.OpenSession/"))
        .map(|(_, scenario)| scenario)
    else {
        panic!(
            "a scenario for {id} is synthesized; refusals: {:#?}",
            synthesis.refusals
        );
    };
    let sends: Vec<&BTreeMap<String, ScenarioValue>> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "example.f.OpenSession" =>
            {
                Some(input)
            }
            _ => None,
        })
        .collect();
    let same_identity = sends
        .iter()
        .filter(|input| {
            matches!(input.get("user_id"), Some(ScenarioValue::Fixture { fixture })
                if fixture.as_str() == "session-user")
        })
        .count();
    assert!(
        same_identity < 2,
        "the precondition creates `User` for fixture `session-user`, and the scenario then asserts \
         `opened` creating a `User` for the same fixture again — a creation over an existing \
         record:\n{}",
        serde_json::to_string_pretty(&scenario.steps).unwrap()
    );
}
