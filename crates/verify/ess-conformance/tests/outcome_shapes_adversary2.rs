//! Adversarial cases, pass 2, for the ess/15 outcome shapes (`docs/design/outcome-shapes.md`).
//!
//! Each case drives a synthesized suite against an in-memory target that behaves the way the design
//! page says a correct implementation behaves, and asserts what the page promises.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use ess_primitives::Number;

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, (Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let detail = format!("{:?}", result.diagnostics().collect::<Vec<_>>());
            (result.scenario.to_string(), (result.status, detail))
        })
        .collect()
}

fn took(command: &CommandRef, outcome: &str) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
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
            Ok(ImplementationIdentity::new(
                "outcome-shapes-adversary-2",
                "1",
            ))
        }
        fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
            Ok(())
        }
    };
}

// ---- #150 × aggregate views: the only creator creates into a non-initial state --------------

const INTO_ONLY: &str = "format: ess/15
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
      states: [Dialing, Ringing, Connected]
      terminal: [Connected]
      transitions:
        - {name: ring, from: [Dialing], to: Ringing}
        - {name: answer, from: [Ringing], to: Connected}
events:
  - name: example.call.CallOffered
    fields: [{name: call_id, type: example.call.CallId}]
  - name: example.call.CallRang
    fields: [{name: call_id, type: example.call.CallId}]
  - name: example.call.CallAnswered
    fields: [{name: call_id, type: example.call.CallId}]
actors:
  - name: example.call.Agent
    may: [example.call.OfferCall, example.call.RingCall, example.call.AnswerCall]
commands:
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
  - name: example.call.AnswerCall
    input:
      - {name: call_id, type: example.call.CallId}
    outcomes:
      - name: answered
        moves: example.call.Call.answer
        instance: call_id
        emits: [example.call.CallAnswered]
        payload:
          example.call.CallAnswered: {call_id: input.call_id}
      - {name: not-ringing, wrong_state: true, refuses: false}
views:
  - name: example.call.ConnectedByQueue
    source: example.call.Call
    consistency: read_your_writes
    filter: state == Connected
    group_by: [queue]
    fields:
      - {name: queue, type: String}
      - {name: calls, type: Integer, aggregate: {count: {}}}
";

/// The model as declared: an offered call is `Ringing`; the view counts `Connected` calls by queue.
#[derive(Default)]
struct Offered {
    rows: RefCell<BTreeMap<String, (String, String)>>,
}

impl ConformanceTarget for Offered {
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
        let mut step = |from: &str, to: &str, event: &str, outcome: &str, wrong: &str| match rows
            .get_mut(&id)
        {
            Some((state, _)) if state == from => {
                *state = to.into();
                call(event, outcome)
            }
            _ => took(&command, wrong),
        };
        Ok(match request.command.to_string().as_str() {
            "example.call.OfferCall" => {
                rows.insert(id.clone(), ("Ringing".into(), queue));
                call("example.call.CallOffered", "offered")
            }
            "example.call.RingCall" => step(
                "Dialing",
                "Ringing",
                "example.call.CallRang",
                "rang",
                "not-dialing",
            ),
            "example.call.AnswerCall" => step(
                "Ringing",
                "Connected",
                "example.call.CallAnswered",
                "answered",
                "not-ringing",
            ),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        assert_eq!(request.view.to_string(), "example.call.ConnectedByQueue");
        let mut counts: BTreeMap<String, i64> = BTreeMap::new();
        for (state, queue) in self.rows.borrow().values() {
            if state == "Connected" {
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
fn adversary2_an_aggregate_whose_only_creator_creates_into_a_later_state_passes() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(INTO_ONLY));
    let aggregate: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.starts_with("example.call.ConnectedByQueue"))
        .collect();
    assert!(
        !aggregate.is_empty(),
        "an aggregate scenario is synthesized: `Connected` is reachable from the only creation \
         state; refusals: {:#?}",
        synthesis.refusals
    );
    let result = statuses(&synthesis.suite, &Offered::default());
    let failed: Vec<(&String, &(Status, String))> = result
        .iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .collect();
    assert!(
        failed.is_empty(),
        "against the implementation the model declares, these fail: {failed:#?}"
    );
}

// ---- #152: preconditions that depend on each other -----------------------------------------

const DEPENDENT: &str = "format: ess/15
system: example
version: v1
preconditions:
  - command: example.d.CreateUser
    as: example.d.Agent
    input: {user_id: 00000000-0000-4000-8000-000000000001}
  - command: example.d.OpenSession
    as: example.d.Agent
    input: {session_id: {fixture: the-session}}
domain: example.d
types:
  - {name: example.d.UserId, kind: newtype, of: Uuid}
  - {name: example.d.SessionId, kind: newtype, of: Uuid}
entities:
  - name: example.d.User
    identity: {name: user_id, type: example.d.UserId}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: example.d.Session
    identity: {name: session_id, type: example.d.SessionId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: example.d.UserCreated
    fields: [{name: user_id, type: example.d.UserId}]
  - name: example.d.SessionOpened
    fields: [{name: session_id, type: example.d.SessionId}]
actors:
  - {name: example.d.Agent, may: [example.d.CreateUser, example.d.OpenSession]}
commands:
  - name: example.d.CreateUser
    input:
      - {name: user_id, type: example.d.UserId}
    outcomes:
      - name: created
        creates: example.d.User
        instance: user_id
        emits: [example.d.UserCreated]
        payload:
          example.d.UserCreated: {user_id: input.user_id}
  - name: example.d.OpenSession
    input:
      - {name: session_id, type: example.d.SessionId}
    fixture_inputs: {session_id: the-session}
    outcomes:
      - name: opened
        creates: example.d.Session
        instance: session_id
        emits: [example.d.SessionOpened]
        payload:
          example.d.SessionOpened: {session_id: input.session_id}
views:
  - name: example.d.Users
    source: example.d.User
    consistency: read_your_writes
    fields:
      - {name: user_id, type: example.d.UserId}
      - {name: state, type: example.d.User.State}
  - name: example.d.Sessions
    source: example.d.Session
    consistency: read_your_writes
    fields:
      - {name: session_id, type: example.d.SessionId}
      - {name: state, type: example.d.Session.State}
";

/// The ambient behaviour #152 describes: a session opens only once a user row exists — the
/// dependency the model cannot state, and the reason the first precondition precedes the second.
#[derive(Default)]
struct Ambient {
    users: RefCell<BTreeSet<String>>,
    sessions: RefCell<BTreeSet<String>>,
}

impl ConformanceTarget for Ambient {
    unused_target_methods!();
    fn fixture_values(
        &self,
        _: &ScenarioContext,
        contract: &ess_conformance::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        Ok(contract
            .fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str().to_owned(),
                    Node::Text("00000000-0000-4000-8000-0000000000aa".to_owned()),
                )
            })
            .collect())
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.users.borrow_mut().clear();
        self.sessions.borrow_mut().clear();
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        Ok(match request.command.to_string().as_str() {
            "example.d.CreateUser" => {
                let id = text_of(&request.input, "user_id");
                self.users.borrow_mut().insert(id.clone());
                let mut result = took(&command, "created");
                emit(
                    &mut result,
                    "example.d.UserCreated",
                    "user_id",
                    Node::Text(id),
                );
                result
            }
            "example.d.OpenSession" if self.users.borrow().is_empty() => {
                // No user row: the implementation has nothing to open a session for.
                SemanticCommandResult::undeclared()
            }
            "example.d.OpenSession" => {
                let id = text_of(&request.input, "session_id");
                self.sessions.borrow_mut().insert(id.clone());
                let mut result = took(&command, "opened");
                emit(
                    &mut result,
                    "example.d.SessionOpened",
                    "session_id",
                    Node::Text(id),
                );
                result
            }
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let (rows, field, state) = match request.view.to_string().as_str() {
            "example.d.Users" => (self.users.borrow().clone(), "user_id", "Active"),
            "example.d.Sessions" => (self.sessions.borrow().clone(), "session_id", "Open"),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult {
            rows: rows
                .into_iter()
                .map(|id| {
                    BTreeMap::from([
                        (field.to_owned(), Node::Text(id)),
                        ("state".to_owned(), Node::Text(state.to_owned())),
                    ])
                })
                .collect(),
            total: None,
        })
    }
}

#[test]
fn adversary2_a_scenario_skipping_one_recreating_precondition_still_runs_the_ones_before_it() {
    // The design: preconditions are "an ordered list of command invocations that synthesis runs
    // before every scenario's arrangement". The unit skips the prelude where a scenario would
    // recreate a precondition's identity — defensible for that one precondition, but it drops the
    // whole list, so the user row the session depends on is never made.
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(DEPENDENT));
    let id = "example.d.OpenSession/outcome/opened";
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "{id} is synthesized; refusals: {:#?}",
        synthesis.refusals
    );
    let result = statuses(&synthesis.suite, &Ambient::default());
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
        .unwrap();
    assert_eq!(
        result[id].0,
        Status::Passed,
        "the session command's own scenario runs without the user precondition that precedes \
         it ({}); steps:\n{}",
        result[id].1,
        serde_json::to_string_pretty(&scenario.steps).unwrap()
    );
}

// ---- IR bytes: a model writing none of the ess/15 constructs --------------------------------

#[test]
fn adversary2_a_model_without_the_new_constructs_carries_none_of_their_keys() {
    // Written under ess/14: none of the five constructs, so none of their IR keys.
    let model = INTO_ONLY
        .replace("format: ess/15", "format: ess/14")
        .replace("        into: Ringing\n", "");
    let json = serde_json::to_string(&ir_of(&model)).unwrap();
    for key in [
        "\"preconditions\"",
        "\"into\"",
        "\"accepts_nothing\"",
        "unknown_instance",
        "UnknownInstance",
        "deletes",
    ] {
        assert!(
            !json.contains(key),
            "the IR of a model without ess/15 constructs carries {key}"
        );
    }
}
