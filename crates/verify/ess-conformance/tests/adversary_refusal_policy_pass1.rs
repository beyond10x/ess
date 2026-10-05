//! Adversary pass 1 against the refusal-selected failure policy unit (ess/22, beyond10x/ess#269):
//! the synthesized `<binding>/binding/refusal/<outcome>` scenarios held against senders the unit's
//! own tests do not build, and the native interpreter driven through refusal sequences its named
//! controls do not script.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::{Interpreted, PortAnswer};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};

const MODEL: &str = include_str!("fixtures/refusal-policy.yaml");
const POLICY: &str = "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
";
const BINDING: &str = "notify-ledger";
const PLACE: &str = "demo.ledger.Place";
const RECORD: &str = "demo.ledger.Record";
const RECORDED: &str = "demo.ledger.Recorded";
const ESCALATED: &str = "demo.ledger.RecordEscalated";

fn id(outcome: &str) -> String {
    format!("{BINDING}/binding/refusal/{outcome}")
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("refusal-policy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
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

// ---- a sender the unit's faulty modes do not include ---------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// The binding as the table says.
    Correct,
    /// On an escalated refusal, publishes the escalation event twice for the one attempt, and stops:
    /// one invocation, two escalations. "Escalate emits its declared event once" (design,
    /// "Attempts and actual refusal authority"); the acceptance kills "duplicate-escalation
    /// behavior".
    EscalatesTwiceOnOneAttempt,
    /// Escalates the refusal it should drop, published late: visible to the twentieth observation
    /// of the escalation event onward, inside the fifty-ask window.
    EscalatesDroppedLate,
}

#[derive(Default)]
struct State {
    forced: Option<(String, u32)>,
    log: Vec<ObservedEvent>,
    late: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
}

struct Ledger {
    mode: Mode,
    state: RefCell<State>,
    escalation_asks: Cell<u32>,
}

impl Ledger {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            state: RefCell::new(State::default()),
            escalation_asks: Cell::new(0),
        }
    }

    fn event(name: &str, order_id: &Node) -> ObservedEvent {
        ObservedEvent::new(name.parse().unwrap()).with("order_id", order_id.clone())
    }

    fn answer(state: &mut State) -> String {
        match state.forced.as_mut() {
            Some((outcome, remaining)) if *remaining > 0 => {
                *remaining -= 1;
                outcome.clone()
            }
            _ => "recorded".to_owned(),
        }
    }

    fn notify(&self, state: &mut State, placed: &ObservedEvent) {
        let order_id = placed.payload["order_id"].clone();
        let mut attempts = 0_u32;
        while attempts < 64 {
            attempts += 1;
            state.invocations.push(
                ObservedInvocation::new(
                    BindingRef::new(ess_domain::binding::BindingName::new(BINDING).unwrap()),
                    CommandRef::new(RECORD.parse().unwrap()),
                )
                .with("order_id", order_id.clone()),
            );
            let answer = Self::answer(state);
            match answer.as_str() {
                "recorded" => {
                    state.log.push(Self::event(RECORDED, &order_id));
                    return;
                }
                "wrong-state" => {
                    if self.mode == Mode::EscalatesDroppedLate {
                        state.late.push(Self::event(ESCALATED, &order_id));
                    }
                    return;
                }
                "unavailable" | "busy" => {
                    if attempts >= 3 {
                        return;
                    }
                }
                "rejected" => return,
                _ => {
                    state.log.push(Self::event(ESCALATED, &order_id));
                    if self.mode == Mode::EscalatesTwiceOnOneAttempt {
                        state.log.push(Self::event(ESCALATED, &order_id));
                    }
                    return;
                }
            }
        }
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("refusal-policy-adversary", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.state.borrow_mut() = State::default();
        self.escalation_asks.set(0);
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
                let mut result = SemanticCommandResult::took(OutcomeRef::new(
                    CommandRef::new(PLACE.parse().unwrap()),
                    "placed".parse().unwrap(),
                ));
                result.direct_events.push(placed.clone());
                self.notify(&mut state, &placed);
                Ok(result)
            }
            RECORD => {
                let answer = Self::answer(&mut state);
                let mut result = SemanticCommandResult::took(OutcomeRef::new(
                    CommandRef::new(RECORD.parse().unwrap()),
                    answer.parse().unwrap(),
                ));
                let error = match answer.as_str() {
                    "recorded" => {
                        let event = Self::event(RECORDED, &order_id);
                        state.log.push(event.clone());
                        result.direct_events.push(event);
                        None
                    }
                    "unavailable" | "busy" => Some("demo.ledger.Unavailable"),
                    "rejected" => Some("demo.ledger.Unknown"),
                    "at-limit" => Some("demo.ledger.AtLimit"),
                    _ => Some("demo.ledger.WrongState"),
                };
                result.error = error.map(|error| DeclaredErrorValue::new(error.parse().unwrap()));
                Ok(result)
            }
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
        let state = self.state.borrow();
        let mut seen: Vec<ObservedEvent> = state
            .log
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect();
        if request.event.to_string() == ESCALATED {
            self.escalation_asks.set(self.escalation_asks.get() + 1);
            if self.escalation_asks.get() >= 20 {
                seen.extend(state.late.iter().cloned());
            }
        }
        Ok(seen)
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

/// The healthy sender passes every refusal scenario: the baseline the two faulty senders below are
/// measured against.
#[test]
fn adversary_the_baseline_sender_passes_every_refusal_scenario() {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(MODEL)).suite;
    let statuses = run(&suite, &Ledger::new(Mode::Correct));
    let refusal: Vec<(&String, &Status)> = statuses
        .iter()
        .filter(|(scenario, _)| scenario.contains("/binding/refusal/"))
        .collect();
    assert_eq!(refusal.len(), 5, "{statuses:#?}");
    for (scenario, status) in refusal {
        assert_eq!(*status, Status::Passed, "{scenario}: {statuses:#?}");
    }
}

/// An escalation published twice for one attempt is duplicate escalation, which the acceptance
/// says the suite kills. The escalated refusal's scenario must not pass it.
#[test]
fn adversary_an_escalation_published_twice_on_one_attempt_fails_the_escalated_refusal() {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(MODEL)).suite;
    let statuses = run(&suite, &Ledger::new(Mode::EscalatesTwiceOnOneAttempt));
    assert_ne!(
        statuses[&id("at-limit")],
        Status::Passed,
        "a sender that publishes `{ESCALATED}` twice for one refused attempt passes {}: \
         {statuses:#?}",
        id("at-limit")
    );
}

/// `expect_no_publication` reads the whole window: an escalation of a dropped refusal that becomes
/// visible late, inside the window, still fails the dropped refusal's scenario.
#[test]
fn adversary_a_late_escalation_of_a_dropped_refusal_fails_inside_the_window() {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(MODEL)).suite;
    let statuses = run(&suite, &Ledger::new(Mode::EscalatesDroppedLate));
    assert_eq!(
        statuses[&id("wrong-state")],
        Status::Failed,
        "{statuses:#?}"
    );
}

// ---- the native interpreter through refusal sequences the named controls do not script -----------

#[derive(Debug, PartialEq, Eq)]
struct Placed {
    ok: bool,
    invocations: usize,
    recorded: usize,
    escalated: usize,
}

fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.ledger/authored/adversary".parse().unwrap(),
        CorrelationId::new("adversary").unwrap(),
    )
}

fn place(target: &Interpreted, answers: &[&str]) -> Placed {
    target.begin_scenario(&context()).unwrap();
    target.script_binding_port(
        &CommandRef::new(RECORD.parse().unwrap()),
        answers.iter().map(|answer| {
            if *answer == "untyped" {
                PortAnswer::Untyped
            } else {
                PortAnswer::Outcome(answer.parse().unwrap())
            }
        }),
    );
    let ok = target
        .execute_command(SemanticCommandRequest {
            command: CommandRef::new(PLACE.parse().unwrap()),
            actor: None,
            caller: None,
            input: BTreeMap::from([("order_id".to_owned(), Node::Text("o-1".into()))]),
            correlation: context().correlation,
        })
        .is_ok();
    let invocations = target
        .observe_invocations(InvocationObservationRequest {
            binding: BindingRef::new(ess_domain::binding::BindingName::new(BINDING).unwrap()),
            command: CommandRef::new(RECORD.parse().unwrap()),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .len();
    let events = |name: &str| {
        target
            .observe_events(EventObservationRequest {
                event: name.parse().unwrap(),
                correlation: context().correlation,
                deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
            })
            .unwrap()
            .len()
    };
    Placed {
        ok,
        invocations,
        recorded: events(RECORDED),
        escalated: events(ESCALATED),
    }
}

fn placed(invocations: usize, recorded: usize, escalated: usize) -> Placed {
    Placed {
        ok: true,
        invocations,
        recorded,
        escalated,
    }
}

/// Retried refusals followed by a refusal of another policy: the later refusal's policy answers,
/// at the attempt it arrives on, whatever came before.
#[test]
fn adversary_native_reselects_on_every_attempt_across_policies() {
    let target = Interpreted::for_model(ir_of(MODEL));
    for (answers, expected) in [
        (vec!["busy", "at-limit"], placed(2, 0, 1)),
        (vec!["unavailable", "wrong-state"], placed(2, 0, 0)),
        (vec!["unavailable", "untyped"], placed(2, 0, 1)),
        (vec!["busy", "unavailable", "at-limit"], placed(3, 0, 1)),
        (vec!["busy", "rejected", "at-limit"], placed(2, 0, 0)),
        // A, B, A: the budget is the total of three, not a run of one refusal.
        (vec!["busy", "unavailable", "busy", "busy"], placed(3, 0, 0)),
    ] {
        assert_eq!(place(&target, &answers), expected, "{answers:?}");
    }
}

/// A bounded fallback retry with `final`: the total budget spans declared refusals the fallback
/// selects and untyped failures alike, in any interleaving, and `final` reached through the
/// fallback ends it at once.
#[test]
fn adversary_native_fallback_retry_counts_an_interleaving_against_one_budget() {
    let model = MODEL.replace(
        POLICY,
        "    on_failure:\n      drop: [wrong-state]\n      escalate: {emits: demo.ledger.RecordEscalated, outcomes: [at-limit]}\n      retry: {except: [wrong-state, at-limit], attempts: 4, final: [rejected]}\n",
    );
    assert_ne!(model, MODEL);
    let target = Interpreted::for_model(ir_of(&model));
    for (answers, expected) in [
        (
            vec!["busy", "untyped", "busy", "untyped", "busy"],
            placed(4, 0, 0),
        ),
        (vec!["untyped", "busy", "untyped"], placed(4, 1, 0)),
        (vec!["untyped", "busy", "rejected"], placed(3, 0, 0)),
        (vec!["untyped", "busy", "at-limit"], placed(3, 0, 1)),
    ] {
        assert_eq!(place(&target, &answers), expected, "{answers:?}");
    }
}

/// `final` answered on the very attempt that exhausts the bound, and on the first attempt.
#[test]
fn adversary_native_final_at_the_edges_of_the_bound() {
    let target = Interpreted::for_model(ir_of(MODEL));
    assert_eq!(place(&target, &["rejected"]), placed(1, 0, 0));
    assert_eq!(
        place(&target, &["busy", "busy", "rejected"]),
        placed(3, 0, 0)
    );
}

// ---- a selected retry on a binding whose destination row is arranged before its trigger -----------

const ARRANGEMENT: &str = include_str!("fixtures/binding-arrangement.yaml");
const KICKED_DROP: &str = "  - id: kicked-starts
    when: {event: jobs.job.Kicked}
    invoke: {command: jobs.job.Start}
    mapping: {job_id: event.job_id}
    delivery: at_least_once
    on_failure: drop";

fn kicked(policy: &str) -> String {
    let model = ARRANGEMENT
        .replace("format: ess/18", "format: ess/22")
        .replace(
            KICKED_DROP,
            &KICKED_DROP.replace("    on_failure: drop", policy),
        );
    assert_ne!(
        model,
        ARRANGEMENT.replace("format: ess/18", "format: ess/22")
    );
    model
}

/// `kicked-starts` starts a job `Kick` does not own, so the row it addresses must be arranged
/// before the trigger (beyond10x/ess#267), and `Start` has branches its input decides
/// (`wrong-state`, `not-found`). Universal `on_failure: retry` is witnessed through that
/// arrangement, which picks the branch, and the binding-running interpreter passes it. The same
/// retry selected for the one `external:` refusal, `unavailable`, is forceable and makes the same
/// claim for that refusal: the acceptance's "synthesis witnesses each forceable selected refusal"
/// asks for its scenario, and the interpreter must pass it.
#[test]
fn adversary_a_selected_retry_on_an_arranged_row_is_witnessed_as_the_universal_retry_is() {
    let universal = kicked("    on_failure: retry");
    let ir = ir_of(&universal);
    let statuses = run(
        &ess_conformance::synthesize::synthesize(&ir).suite,
        &Interpreted::for_model(ir),
    );
    assert_eq!(
        statuses.get("kicked-starts/binding/on-failure"),
        Some(&Status::Passed),
        "the universal baseline: {statuses:#?}"
    );
    let selected =
        kicked("    on_failure:\n      retry: [unavailable]\n      drop: {except: [unavailable]}");
    let ir = ir_of(&selected);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let statuses = run(&synthesis.suite, &Interpreted::for_model(ir));
    let scenario = "kicked-starts/binding/refusal/unavailable";
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(scenario))
        .collect();
    assert_eq!(
        statuses.get(scenario),
        Some(&Status::Passed),
        "the forceable refusal is not witnessed; refused as: {refused:#?}"
    );
}
