//! Adversary pass 1 against the bounded retry (ess/16 `on_failure: {retry: {attempts, final}}`,
//! beyond10x/ess#165): what the count step and the forced branch do at their edges.
//!
//! * A sender whose attempts become visible over time (any target whose invocations are not all
//!   recorded before the first observation) is judged the moment the count is reached, so a sender
//!   that retries a `final` refusal, or retries past the bound, passes.
//! * The retried branch is chosen as "the first `external:` branch not in `final`", without asking
//!   whether that branch is a failure at all.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");
const BLOCK: &str = "retry: {attempts: 3, final: [demo.ledger.Unknown]}";

const ON_FAILURE: &str = "notify-ledger/binding/on-failure";
const FINAL: &str = "notify-ledger/binding/final-failure";

const PLACE: &str = "demo.ledger.Place";
const RECORD: &str = "demo.ledger.Record";
const BINDING: &str = "notify-ledger";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("bounded-retry.yaml"), raw)])
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
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

fn forced(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ConfigureExternalOutcome { force, .. } => Some(force.to_string()),
            _ => None,
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sender {
    /// Up to three attempts; `rejected` ends the retry at once.
    Correct,
    /// Retries `rejected` as if it were a server error.
    RetriesFinal,
    /// Retries until the ledger records it.
    Unbounded,
}

#[derive(Default)]
struct State {
    forced: Option<(String, u32)>,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
    /// How many invocations the next observation shows, where the sender is seen late.
    visible: usize,
}

/// The ledger sender, whose invocations are recorded at once but, when `late`, become visible to
/// an observer one per observation: a sender that retries on a timer, seen through a real queue.
struct Ledger {
    sender: Sender,
    late: bool,
    state: RefCell<State>,
}

impl Ledger {
    fn new(sender: Sender, late: bool) -> Self {
        Self {
            sender,
            late,
            state: RefCell::new(State::default()),
        }
    }

    fn outcome(command: &str, outcome: &str) -> OutcomeRef {
        OutcomeRef::new(
            CommandRef::new(command.parse().unwrap()),
            outcome.parse().unwrap(),
        )
    }

    fn event(name: &str, order_id: &Node) -> ObservedEvent {
        ObservedEvent::new(name.parse().unwrap()).with("order_id", order_id.clone())
    }

    fn record(state: &mut State, order_id: &Node) -> (String, SemanticCommandResult) {
        let forced = match state.forced.as_mut() {
            Some((outcome, remaining)) if *remaining > 0 => {
                *remaining -= 1;
                Some(outcome.clone())
            }
            _ => None,
        };
        let outcome = forced.unwrap_or_else(|| "recorded".to_owned());
        let mut result = SemanticCommandResult::took(Self::outcome(RECORD, &outcome));
        match outcome.as_str() {
            "recorded" => {
                let event = Self::event("demo.ledger.Recorded", order_id);
                state.log.push(event.clone());
                result.direct_events.push(event);
            }
            "unavailable" => {
                result.error = Some(DeclaredErrorValue::new(
                    "demo.ledger.Unavailable".parse().unwrap(),
                ));
            }
            _ => {
                result.error = Some(DeclaredErrorValue::new(
                    "demo.ledger.Unknown".parse().unwrap(),
                ));
            }
        }
        (outcome, result)
    }

    fn notify(&self, state: &mut State, placed: &ObservedEvent) {
        let order_id = placed.payload["order_id"].clone();
        let bound = match self.sender {
            Sender::Unbounded => 10,
            Sender::Correct | Sender::RetriesFinal => 3,
        };
        for _ in 0..bound {
            state.invocations.push(
                ObservedInvocation::new(
                    BindingRef::new(ess_domain::binding::BindingName::new(BINDING).unwrap()),
                    CommandRef::new(RECORD.parse().unwrap()),
                )
                .with("order_id", order_id.clone()),
            );
            let (outcome, _) = Self::record(state, &order_id);
            match outcome.as_str() {
                "recorded" => return,
                "rejected" if self.sender != Sender::RetriesFinal => return,
                _ => {}
            }
        }
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-retry", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.state.borrow_mut() = State::default();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut state = self.state.borrow_mut();
        let order_id = request.input.get("order_id").cloned().unwrap_or(Node::Null);
        match request.command.to_string().as_str() {
            PLACE => {
                let placed = Self::event("demo.ledger.OrderPlaced", &order_id);
                state.log.push(placed.clone());
                let mut result = SemanticCommandResult::took(Self::outcome(PLACE, "placed"));
                result.direct_events.push(placed.clone());
                self.notify(&mut state, &placed);
                Ok(result)
            }
            RECORD => Ok(Self::record(&mut state, &order_id).1),
            other => panic!("unexpected command {other}"),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .state
            .borrow()
            .log
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = Some((request.force.outcome.to_string(), 1));
        Ok(())
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: NonZeroU32,
    ) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = Some((request.force.outcome.to_string(), times.get()));
        Ok(())
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        let mut state = self.state.borrow_mut();
        let placed = state
            .log
            .iter()
            .rev()
            .find(|event| event.event == request.event)
            .cloned()
            .expect("the event was published");
        self.notify(&mut state, &placed);
        Ok(())
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        let mut state = self.state.borrow_mut();
        let all: Vec<ObservedInvocation> = state
            .invocations
            .iter()
            .filter(|invocation| {
                invocation.binding == request.binding && invocation.command == request.command
            })
            .cloned()
            .collect();
        if !self.late {
            return Ok(all);
        }
        state.visible += 1;
        Ok(all.into_iter().take(state.visible).collect())
    }
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Guard: the late ledger is a conformant sender; seen late, the correct one still passes both.
#[test]
fn adversary_retry_a_correct_sender_seen_late_passes() {
    let suite = synthesis_of(MODEL).suite;
    let statuses = run(&suite, &Ledger::new(Sender::Correct, true));
    assert_eq!(statuses[ON_FAILURE], Status::Passed, "{statuses:#?}");
    assert_eq!(statuses[FINAL], Status::Passed, "{statuses:#?}");
}

/// Acceptance: "with a final error forced, exactly one invocation". A sender that retries the
/// final refusal makes two; seen late, the second is not yet visible when the count first reads 1.
#[test]
fn adversary_retry_a_sender_retrying_a_final_refusal_seen_late_is_caught() {
    let suite = synthesis_of(MODEL).suite;
    let statuses = run(&suite, &Ledger::new(Sender::RetriesFinal, true));
    assert_eq!(
        statuses[FINAL],
        Status::Failed,
        "a sender that retries `rejected` makes two invocations, and `final-failure` requires \
         exactly one: {statuses:#?}"
    );
}

/// Acceptance: "exactly N invocations". A sender that goes on past the bound makes four; seen
/// late, the fourth is not yet visible when the count first reads 3.
#[test]
fn adversary_retry_a_sender_retrying_past_the_bound_seen_late_is_caught() {
    let suite = synthesis_of(MODEL).suite;
    let statuses = run(&suite, &Ledger::new(Sender::Unbounded, true));
    assert_eq!(
        statuses[ON_FAILURE],
        Status::Failed,
        "a sender that makes a fourth attempt breaks `attempts: 3`: {statuses:#?}"
    );
}

/// "`final` names the invoked command's declared refusals; any other failure is retried": the
/// branch `on-failure` forces on every attempt has to be a failure. An `external:` branch that
/// carries no `error:` is a success the adapter decides, and a correct sender stops on it after one
/// invocation, so a scenario forcing it three times and counting three fails the correct sender.
#[test]
fn adversary_retry_on_failure_forces_a_refusal_not_an_external_success() {
    let model = MODEL.replace(
        "      - name: unavailable\n",
        "      - name: deferred\n        external: the ledger records it later\n        emits: \
         [demo.ledger.Recorded]\n        payload:\n          demo.ledger.Recorded: {order_id: \
         input.order_id}\n      - name: unavailable\n",
    );
    assert_ne!(model, MODEL);
    let synthesis = synthesis_of(&model);
    let exhausted = scenario(&synthesis.suite, ON_FAILURE);
    assert_eq!(
        forced(exhausted),
        vec!["demo.ledger.Record/unavailable".to_owned()],
        "the retried branch is a refusal: {:#?}",
        exhausted.steps
    );
}

/// Boundary: the smallest bound, forced and counted twice, and the correct two-attempt sender
/// passing it.
#[test]
fn adversary_retry_two_attempts_is_forced_and_counted_twice() {
    let synthesis = synthesis_of(&MODEL.replace(BLOCK, "retry: {attempts: 2, final: [rejected]}"));
    let exhausted = scenario(&synthesis.suite, ON_FAILURE);
    let mut times = Vec::new();
    let mut counts = Vec::new();
    for step in &exhausted.steps {
        match step {
            ScenarioStep::ConfigureExternalOutcome { times: t, .. } => {
                times.push(t.map(NonZeroU32::get));
            }
            ScenarioStep::ExpectInvocation { count, .. } => counts.push(count.map(NonZeroU32::get)),
            _ => {}
        }
    }
    assert_eq!(times, vec![Some(2)], "{:#?}", exhausted.steps);
    assert_eq!(counts, vec![Some(2)], "{:#?}", exhausted.steps);
}

/// `final` by the outcome and `final` by the error it reports are one claim, and one suite.
#[test]
fn adversary_retry_final_by_outcome_and_by_error_synthesize_the_same_suite() {
    let by_error = synthesis_of(MODEL).suite;
    let by_outcome =
        synthesis_of(&MODEL.replace(BLOCK, "retry: {attempts: 3, final: [rejected]}")).suite;
    assert_eq!(
        serde_json::to_string(&by_error.scenarios).unwrap(),
        serde_json::to_string(&by_outcome.scenarios).unwrap()
    );
}

/// The count reads every invocation by the binding in the scenario, so an arrangement that itself
/// publishes the binding's event adds invocations the bound did not make: a correct sender then
/// shows `attempts + 1`. Here `Amend` (the publisher synthesis picks, first by name) needs an
/// order, and `Place`, which makes one, publishes the same `OrderPlaced`.
#[test]
fn adversary_retry_the_arrangement_does_not_set_off_the_counted_binding() {
    let model = MODEL
        .replace(
            "events:\n",
            r"entities:
  - name: demo.ledger.Order
    identity: {name: order_id, type: demo.ledger.OrderId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []
events:
",
        )
        .replace(
            r"commands:
  - name: demo.ledger.Place
    input:
      - {name: order_id, type: demo.ledger.OrderId}
    outcomes:
      - name: placed
",
            r"commands:
  - name: demo.ledger.Amend
    input:
      - {name: order_id, type: demo.ledger.OrderId}
      - {name: note, type: String}
    outcomes:
      - name: amended
        updates: demo.ledger.Order
        instance: order_id
        sets: {note: input.note}
        emits: [demo.ledger.OrderPlaced]
        payload:
          demo.ledger.OrderPlaced: {order_id: input.order_id}
  - name: demo.ledger.Place
    input:
      - {name: order_id, type: demo.ledger.OrderId}
      - {name: note, type: String}
    outcomes:
      - name: placed
        creates: demo.ledger.Order
        instance: order_id
        sets: {note: input.note}
",
        )
        .replace(
            "may: [demo.ledger.Place, demo.ledger.Record]",
            "may: [demo.ledger.Amend, demo.ledger.Place, demo.ledger.Record]",
        );
    assert!(model.contains("demo.ledger.Amend\n"), "{model}");
    let synthesis = synthesis_of(&model);
    let exhausted = scenario(&synthesis.suite, ON_FAILURE);
    let arming = exhausted
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { .. }))
        .expect("the failure is forced");
    let publishers = ["demo.ledger.Place", "demo.ledger.Amend"];
    let before: Vec<String> = exhausted.steps[..arming]
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .filter(|command| publishers.contains(&command.as_str()))
        .collect();
    assert!(
        before.is_empty(),
        "the arrangement runs {before:?}, which publish `OrderPlaced` and so invoke \
         `notify-ledger` before the count's attempts begin: {:#?}",
        exhausted.steps
    );
}
