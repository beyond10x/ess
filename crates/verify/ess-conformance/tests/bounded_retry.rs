//! A bounded retry is witnessed by its attempt count (ess/16 `on_failure: {retry: {attempts,
//! final}}`; beyond10x/ess#165, `docs/design/binding-delivery-guarantees.md`).
//!
//! With a retried refusal forced on every attempt, the suite requires exactly `attempts`
//! invocations; with a `final` refusal forced, exactly one. Neither requires the invoked command's
//! success event, so the `on-failure` check is a scenario rather than ESS-SYNTH-010. The suite
//! takes `ess-conformance/26`. One in-memory target implements the sender the issue describes;
//! each wrong mode breaks exactly one of the two scenarios.

mod support_versions;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{BindingGap, RefusalCause};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");
const BLOCK: &str = "retry: {attempts: 3, final: [demo.ledger.Unknown]}";

const ON_FAILURE: &str = "notify-ledger/binding/on-failure";
const FINAL: &str = "notify-ledger/binding/final-failure";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("bounded-retry.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesis_of(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

fn with_policy(policy: &str) -> String {
    let model = MODEL.replace(BLOCK, policy);
    assert_ne!(model, MODEL);
    model
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

fn ids(suite: &ConformanceSuite) -> Vec<String> {
    suite.scenarios.keys().map(ToString::to_string).collect()
}

/// The forced outcome and its repetition, and the invocation count, of a scenario's steps.
fn forced_and_counted(scenario: &ConformanceScenario) -> (Vec<String>, Vec<Option<u32>>) {
    let mut forced = Vec::new();
    let mut counted = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ConfigureExternalOutcome { force, times } => {
                forced.push(format!(
                    "{force} x{}",
                    times.map_or_else(|| "1".to_owned(), |t| t.to_string())
                ));
            }
            ScenarioStep::ExpectInvocation { count, .. } => {
                counted.push(count.map(NonZeroU32::get));
            }
            _ => {}
        }
    }
    (forced, counted)
}

// ---- the sender the issue describes, and ways to get it wrong ---------------------------------

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

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

// ---- the suite ----------------------------------------------------------------------------------

#[test]
fn the_bound_is_witnessed_by_exactly_n_invocations_with_a_retried_refusal_forced_on_each() {
    let synthesis = synthesis_of(MODEL);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused, ESS-SYNTH-010 included: {:#?}",
        synthesis.refusals
    );
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    let exhausted = scenario(&synthesis.suite, ON_FAILURE);
    assert_eq!(
        forced_and_counted(exhausted),
        (
            vec!["demo.ledger.Record/unavailable x3".to_owned()],
            vec![Some(3)]
        ),
        "{:#?}",
        exhausted.steps
    );
    assert!(
        !exhausted
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::EventuallyEvent { .. })),
        "after the last attempt the event is lost; no success is required: {:#?}",
        exhausted.steps
    );
}

#[test]
fn a_final_refusal_is_witnessed_by_exactly_one_invocation() {
    let synthesis = synthesis_of(MODEL);
    let single = scenario(&synthesis.suite, FINAL);
    assert_eq!(
        forced_and_counted(single),
        (
            vec!["demo.ledger.Record/rejected x1".to_owned()],
            vec![Some(1)]
        ),
        "{:#?}",
        single.steps
    );
}

#[test]
fn the_steps_carry_their_repetition_and_count_on_the_wire() {
    let synthesis = synthesis_of(MODEL);
    let exhausted = scenario(&synthesis.suite, ON_FAILURE);
    let json: Vec<serde_json::Value> = exhausted
        .steps
        .iter()
        .map(|step| serde_json::to_value(step).unwrap())
        .collect();
    let forced = json
        .iter()
        .find(|step| step["step"] == "configure_external_outcome")
        .unwrap();
    assert_eq!(forced["times"], 3, "{forced}");
    let counted = json
        .iter()
        .find(|step| step["step"] == "expect_invocation")
        .unwrap();
    assert_eq!(counted["count"], 3, "{counted}");
    for (step, value) in exhausted.steps.iter().zip(json) {
        let back: ScenarioStep = serde_json::from_value(value).unwrap();
        assert_eq!(&back, step);
    }
}

#[test]
fn every_scenario_passes_against_the_sender_the_issue_describes() {
    let synthesis = synthesis_of(MODEL);
    let statuses = run(&synthesis.suite, &Ledger::new(Mode::Correct));
    assert!(statuses.contains_key(ON_FAILURE), "{statuses:#?}");
    assert!(statuses.contains_key(FINAL), "{statuses:#?}");
    assert!(
        not_passed(&statuses).is_empty(),
        "every scenario passes: {:#?}",
        not_passed(&statuses)
    );
}

#[test]
fn exactly_one_scenario_catches_each_wrong_sender() {
    let synthesis = synthesis_of(MODEL);
    for (mode, caught) in [
        (Mode::Unbounded, ON_FAILURE),
        (Mode::TooFew, ON_FAILURE),
        (Mode::RetriesFinal, FINAL),
    ] {
        let statuses = run(&synthesis.suite, &Ledger::new(mode));
        assert_eq!(not_passed(&statuses), vec![caught], "{mode:?}");
        assert_eq!(statuses[caught], Status::Failed, "{mode:?}");
    }
}

#[test]
fn a_target_that_cannot_repeat_a_forced_outcome_is_unsupported_never_passed() {
    let synthesis = synthesis_of(MODEL);
    let statuses = run(&synthesis.suite, &Ledger::new(Mode::CannotRepeat));
    assert_eq!(not_passed(&statuses), vec![ON_FAILURE]);
    assert_eq!(statuses[ON_FAILURE], Status::Unsupported);
}

#[test]
fn a_suite_pinned_below_26_is_refused() {
    let mut suite = synthesis_of(MODEL).suite;
    suite.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/25").unwrap();
    let Err(error) = AdmittedSuite::from_suite(&suite) else {
        panic!("a suite/25 carrying the fields is refused");
    };
    assert!(
        error.to_string().contains("suite/26"),
        "the refusal names the format: {error}"
    );
}

#[test]
fn a_suite_document_with_a_zero_count_is_refused() {
    let suite = synthesis_of(MODEL).suite;
    let json = serde_json::to_string(&suite).unwrap();
    assert!(AdmittedSuite::from_json(&json).is_ok(), "the suite admits");
    for (from, to) in [
        (r#""count":3"#, r#""count":0"#),
        (r#""times":3"#, r#""times":0"#),
    ] {
        assert!(json.contains(from), "{from}");
        let broken = json.replace(from, to);
        assert!(
            AdmittedSuite::from_json(&broken).is_err(),
            "{to} is refused before anything runs"
        );
    }
}

#[test]
fn a_bound_whose_every_forcible_refusal_is_final_refuses_on_failure_by_name() {
    let synthesis = synthesis_of(&with_policy(
        "retry: {attempts: 3, final: [demo.ledger.Unknown, unavailable]}",
    ));
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == ON_FAILURE)
        })
        .unwrap_or_else(|| panic!("on-failure is refused: {:#?}", synthesis.refusals));
    assert!(
        matches!(
            &refusal.cause,
            RefusalCause::BindingUnobservable {
                gap: BindingGap::RetriedUnforcible { .. },
                ..
            }
        ),
        "{refusal}"
    );
    assert!(ids(&synthesis.suite).contains(&FINAL.to_owned()));
}

#[test]
fn a_bound_without_final_refusals_makes_no_final_failure_claim() {
    let synthesis = synthesis_of(&with_policy("retry: {attempts: 3}"));
    assert!(!ids(&synthesis.suite).contains(&FINAL.to_owned()));
    assert!(synthesis.refusals.iter().all(|refusal| refusal
        .scenario
        .as_ref()
        .is_none_or(|id| id.to_string() != FINAL)));
    // Every refusal is retried, `rejected` first in declaration order after `unavailable`.
    let exhausted = scenario(&synthesis.suite, ON_FAILURE);
    assert_eq!(
        forced_and_counted(exhausted).1,
        vec![Some(3)],
        "{:#?}",
        exhausted.steps
    );
}

#[test]
fn an_unbounded_retry_keeps_its_scenario_and_its_suite_format() {
    let synthesis = synthesis_of(&with_policy("retry"));
    assert!(!ess_conformance::bounded_retry::used_by(&synthesis.suite));
    assert!(!synthesis
        .suite
        .provenance
        .suite_version
        .to_string()
        .ends_with("/26"));
    assert!(!ids(&synthesis.suite).contains(&FINAL.to_owned()));
    let unbounded = scenario(&synthesis.suite, ON_FAILURE);
    assert_eq!(
        forced_and_counted(unbounded),
        (vec!["demo.ledger.Record/unavailable x1".to_owned()], vec![]),
        "{:#?}",
        unbounded.steps
    );
    assert!(unbounded.steps.iter().any(|step| matches!(step,
        ScenarioStep::EventuallyEvent { event, .. } if event.to_string() == "demo.ledger.Recorded")));
}

fn coverage_json(text: &str) -> String {
    let input = ess_conformance::coverage_build::build(
        &ir_of(text),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    input.selected().original_json().to_owned()
}

#[test]
fn coverage_for_the_fixture_is_27_and_carries_the_count() {
    let original = coverage_json(MODEL);
    assert!(original.contains("\"ess-conformance/35\""), "{original}");
    assert!(original.contains("\"count\": 3"), "{original}");
    AdmittedSuite::from_json(&original).unwrap_or_else(|error| panic!("{error}"));
    assert!(AdmittedSuite::from_json(&support_versions::legacy_json(&original, 25)).is_err());
}

#[test]
fn coverage_that_only_refuses_a_final_failure_still_takes_27() {
    // No refusal of `Record` can be forced, so both bounded aspects are refused and no scenario
    // carries a count. The refusal is still filed under `final-failure`, which a reader older than
    // the round-3 pair cannot parse, so the coverage document takes /27 all the same.
    let model = MODEL
        .replace(
            "        external: the ledger answers a server error\n",
            "        when: order_id == down\n",
        )
        .replace(
            "        external: the ledger answers a client error\n",
            "        when: order_id == unknown\n",
        );
    assert_ne!(model, MODEL);
    let original = coverage_json(&model);
    assert!(original.contains("final-failure"), "{original}");
    assert!(!original.contains("\"count\""), "{original}");
    assert!(original.contains("\"ess-conformance/35\""), "{original}");
}
