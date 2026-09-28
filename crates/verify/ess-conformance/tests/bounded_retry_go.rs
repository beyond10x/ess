//! The generated Go runtime witnesses a binding's bounded retry (`configure_external_outcome.times`,
//! `expect_invocation.count`, the `final-failure` scenario; suite/26, beyond10x/ess#165) as the
//! reference runner does (beyond10x/ess#188).
//!
//! The target is `tests/bounded_retry.rs`'s, with every wrong sender, recorded once and replayed to
//! the Go runtime. A Go target without `RepeatedOutcomeTarget` skips the one scenario that forces an
//! outcome more than once.

mod support_go;

use std::cell::RefCell;
use std::num::NonZeroU32;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");

const ON_FAILURE: &str = "notify-ledger/binding/on-failure";
const FINAL: &str = "notify-ledger/binding/final-failure";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("bounded-retry.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite() -> ConformanceSuite {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(MODEL)).suite;
    assert!(ess_conformance::bounded_retry::used_by(&suite));
    assert!(suite.scenarios.keys().any(|id| id.to_string() == FINAL));
    suite
}

#[test]
fn go_gives_the_reference_verdict_for_every_sender() {
    let suite = suite();
    for (mode, expected) in [
        (Mode::Correct, vec![]),
        (Mode::Unbounded, vec![(ON_FAILURE, "failed")]),
        (Mode::TooFew, vec![(ON_FAILURE, "failed")]),
        (Mode::RetriesFinal, vec![(FINAL, "failed")]),
        (Mode::CannotRepeat, vec![(ON_FAILURE, "skipped")]),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("retry-{mode:?}").to_lowercase(),
            &suite,
            Ledger::new(mode),
        );
        let wrong: Vec<(&str, &str)> = verdicts
            .iter()
            .filter(|(_, status)| *status != "passed")
            .map(|(id, status)| (id.as_str(), status.as_str()))
            .collect();
        assert_eq!(wrong, expected, "{mode:?}");
    }
}

#[test]
fn a_go_target_without_repeated_outcomes_skips_only_the_bounded_scenario() {
    let (rust, replayed) = support_go::compare_with(
        "retry-bare",
        &suite(),
        Ledger::new(Mode::Correct),
        &support_go::Options {
            bare: true,
            ..support_go::Options::default()
        },
    );
    assert!(support_go::not_passed(&rust).is_empty(), "{rust:?}");
    assert_eq!(
        support_go::not_passed(&replayed.go.outcomes),
        vec![ON_FAILURE],
        "{}",
        replayed.go.log
    );
    assert_eq!(replayed.go.outcomes[ON_FAILURE], "skipped");
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Up to three attempts; `rejected` ends the retry at once.
    Correct,
    /// Retries until the ledger records it, as `on_failure: retry` with no bound says.
    Unbounded,
    /// Gives up after two attempts.
    TooFew,
    /// Retries `rejected` as if it were a server error.
    RetriesFinal,
    /// Cannot force an outcome on more than one invocation.
    CannotRepeat,
}

#[derive(Default)]
struct State {
    forced: Option<(String, u32)>,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
}

struct Ledger {
    mode: Mode,
    state: RefCell<State>,
}

const PLACE: &str = "demo.ledger.Place";
const RECORD: &str = "demo.ledger.Record";
const BINDING: &str = "notify-ledger";

impl Ledger {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
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

    /// One answer of `demo.ledger.Record`, honouring a forced outcome.
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

    /// The binding: invoke `Record` until it is recorded, a final refusal answers, or the bound
    /// is spent.
    fn notify(&self, state: &mut State, placed: &ObservedEvent) {
        let order_id = placed.payload["order_id"].clone();
        let bound = match self.mode {
            Mode::Unbounded => 10,
            Mode::TooFew => 2,
            Mode::Correct | Mode::RetriesFinal | Mode::CannotRepeat => 3,
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
                "rejected" if self.mode != Mode::RetriesFinal => return,
                _ => {}
            }
        }
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("bounded-retry", "1"))
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
        if self.mode == Mode::CannotRepeat {
            return Err(TargetError::unsupported(
                "a repeated external outcome",
                "this adapter forces the next answer only",
            ));
        }
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
        Ok(self
            .state
            .borrow()
            .invocations
            .iter()
            .filter(|invocation| {
                invocation.binding == request.binding && invocation.command == request.command
            })
            .cloned()
            .collect())
    }
}
