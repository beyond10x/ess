//! The typed decision receipt, from a command's decision edge to the `ess-history/2` operation it
//! is recorded in (beyond10x/ess#244 part a, unit U4; `docs/design/expression-family-source22.md`,
//! "Typed decision receipt from execution to recording").
//!
//! Every recording runs through [`Atomic`] and through a non-atomic [`Interleaved`] fixture that
//! decides at its invoke instant and carries the receipt in its `Pending` until completion. The
//! controls: a returned result, a failure before the decision (no time), a failure after it (time
//! kept on the error), out-of-order completion of two distinct readings, missing evidence and a
//! retained retry. Each recorder corruption — wall-clock substitution, wrong-`Pending`
//! association, dropped error metadata, a copied retry time — must change the checker's answer on
//! a history the healthy recording leaves `Linearizable`: to `Violation`, or, where the corruption
//! removes an instant a necessary guard reads, to no verdict at all (`check.model-undetermined`,
//! the checker's existing answer for an unresolved alternative). A reread at completion is a
//! clock-edge fault, which the scripted provider's read count and reading decide (rule 18 of the
//! design's final review decisions).

mod support_occurrence_clock;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::sync::Arc;

use ess_compiler::ir::EssIr;
use ess_conformance::history::{
    self, Completion, History, HistoryFormat, Operation, QualifiedName, Verdict,
};
use ess_conformance::linearize;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::record::{self, Atomic, Call, Interleaved, Subject, Workload};
use ess_conformance::scenario::{OutcomeRef, SuiteProvenance};
use ess_conformance::sessions::{self, Act};
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RecordedCommandCompletion, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use support_occurrence_clock::*;

/// The setup readings, then an early decision every candidate expiry outlives and a late one none
/// does: `AcceptOffer` decided early answers `accepted`, decided late answers `lapsed`.
const SETUP: [&str; 2] = [T0, "2000-06-01T00:00:01Z"];
const EARLY: &str = "2000-07-01T00:00:00.5Z";
const LATE: &str = "2026-10-04T12:00:00.123456789Z";
/// What a recorder substituting the wall clock writes for every operation.
const WALL: &str = "2026-10-04T12:30:00Z";

fn offers_clock() -> Arc<Scripted> {
    Scripted::new(&[SETUP[0], SETUP[1], EARLY, LATE])
}

fn call(command: &str, input: &[(&str, &str)], subject: Subject) -> Call {
    Call::new(
        command,
        input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), text(value)))
            .collect::<BTreeMap<_, _>>(),
        subject,
    )
}

fn open_call() -> Call {
    call(
        "demo.offers.OpenOffer",
        &[("expires_at", DEADLINE)],
        Subject::Creates,
    )
}

fn accept_call(prefix: usize) -> Call {
    call(
        "demo.offers.AcceptOffer",
        &[("expires_at", DEADLINE)],
        Subject::Created(prefix),
    )
}

/// Two offers opened in the prefix, then one acceptance of each on its own client.
fn two_acceptances() -> Workload {
    Workload {
        prefix: vec![open_call(), open_call()],
        clients: vec![vec![accept_call(0)], vec![accept_call(1)]],
    }
}

fn verdict(ir: &EssIr, history: &History) -> Verdict {
    linearize::check(ir, history, linearize::DEFAULT_BUDGET)
        .unwrap_or_else(|refusal| panic!("checked: {refusal}"))
        .verdict
}

/// The checker's answer where an alternative needs an instant the history does not record: no
/// verdict at all — neither a violation nor a pass — naming the guard that reads `now`.
fn undetermined(ir: &EssIr, history: &History) -> String {
    match linearize::check(ir, history, linearize::DEFAULT_BUDGET) {
        Ok(checked) => panic!(
            "a verdict without the instant it needs: {:?}",
            checked.verdict
        ),
        Err(refusal) => {
            assert_eq!(refusal.code(), "check.model-undetermined", "{refusal}");
            assert!(refusal.to_string().contains("\"now\""), "{refusal}");
            refusal.to_string()
        }
    }
}

/// The history, written and read back: what a checker reading the document sees.
fn written(ir: &EssIr, history: &History) -> History {
    let bytes = serde_json::to_vec(history).expect("a recorded history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the recorded history is admitted: {refusal}"))
}

fn commands<'h>(history: &'h History, name: &str) -> Vec<&'h Operation> {
    history
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == name)
        .collect()
}

// ---- fixture adapters ----------------------------------------------------------------------------

/// Decides at the invoke instant and carries the whole receipt in its `Pending` until completion.
struct DecideAtInvoke<'t, T>(&'t T);

impl<T: ConformanceTarget> Interleaved for DecideAtInvoke<'_, T> {
    type Pending = RecordedCommandCompletion;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        self.0.execute_command_recorded(request)
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        pending.answer
    }

    fn complete_recorded(&self, pending: Self::Pending) -> RecordedCommandCompletion {
        pending
    }
}

/// Fault: decides at invoke like [`DecideAtInvoke`], but completes every call with the reading of
/// whichever call was invoked last, not the one its own `Pending` carries.
struct WrongPending<'t, T> {
    target: &'t T,
    last: Cell<Option<DecisionInstant>>,
}

impl<T: ConformanceTarget> Interleaved for WrongPending<'_, T> {
    type Pending = RecordedCommandCompletion;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        let receipt = self.target.execute_command_recorded(request);
        self.last.set(receipt.decision_time);
        receipt
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        pending.answer
    }

    fn complete_recorded(&self, pending: Self::Pending) -> RecordedCommandCompletion {
        RecordedCommandCompletion {
            answer: pending.answer,
            decision_time: self.last.get(),
        }
    }
}

/// Fault: writes the wall clock's reading in place of the receipt's.
struct WallSubstitution<'t, T>(&'t T);

impl<T: ConformanceTarget> Interleaved for WallSubstitution<'_, T> {
    type Pending = SemanticCommandRequest;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        request
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(pending)
    }

    fn complete_recorded(&self, pending: Self::Pending) -> RecordedCommandCompletion {
        let receipt = self.0.execute_command_recorded(pending);
        RecordedCommandCompletion {
            answer: receipt.answer,
            decision_time: Some(instant(WALL)),
        }
    }
}

/// Fault: decides once, then reads the provider again at completion and records that reading.
struct RereadAtCompletion<'t, T> {
    target: &'t T,
    clock: Arc<Scripted>,
}

impl<T: ConformanceTarget> Interleaved for RereadAtCompletion<'_, T> {
    type Pending = SemanticCommandRequest;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        request
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        self.target.execute_command(pending)
    }

    fn complete_recorded(&self, pending: Self::Pending) -> RecordedCommandCompletion {
        let receipt = self.target.execute_command_recorded(pending);
        RecordedCommandCompletion {
            answer: receipt.answer,
            decision_time: self.clock.read(),
        }
    }
}

/// Uses only the clock-free `complete`: every receipt is the compatibility default, with no time.
struct Discarding<'t, T>(&'t T);

impl<T: ConformanceTarget> Interleaved for Discarding<'_, T> {
    type Pending = SemanticCommandRequest;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        request
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(pending)
    }
}

/// How [`Failing`] answers the call it fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Failure {
    /// Refused before the command reaches its decision edge: nothing decided, no time.
    BeforeDecision,
    /// Decided and took effect; the answer is lost on the way back, with the receipt kept.
    AfterDecision,
    /// Fault: as `AfterDecision`, with the receipt's time dropped from the error.
    DroppedMetadata,
}

/// Atomic, except the first `AcceptOffer` call fails as `failure` says.
struct Failing<'t, T> {
    target: &'t T,
    failure: Failure,
    failed: Cell<bool>,
}

impl<T: ConformanceTarget> Interleaved for Failing<'_, T> {
    type Pending = SemanticCommandRequest;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        request
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        self.complete_recorded(pending).answer
    }

    fn complete_recorded(&self, pending: Self::Pending) -> RecordedCommandCompletion {
        if self.failed.get() || pending.command.to_string() != "demo.offers.AcceptOffer" {
            return self.target.execute_command_recorded(pending);
        }
        self.failed.set(true);
        let lost = TargetError::unavailable("answering", "the connection dropped");
        match self.failure {
            Failure::BeforeDecision => RecordedCommandCompletion {
                answer: Err(lost),
                decision_time: None,
            },
            Failure::AfterDecision | Failure::DroppedMetadata => {
                let receipt = self.target.execute_command_recorded(pending);
                assert!(receipt.answer.is_ok(), "the lost call was decided");
                RecordedCommandCompletion {
                    answer: Err(lost),
                    decision_time: (self.failure == Failure::AfterDecision)
                        .then_some(receipt.decision_time)
                        .flatten(),
                }
            }
        }
    }
}

// ---- returned results ----------------------------------------------------------------------------

/// The operations' readings are exactly the provider's, in decision order, one read per decision.
fn assert_receipts(clock: &Scripted, history: &History, decided_in_order: &[&Operation]) {
    assert_eq!(
        clock.reads(),
        history.operations.len(),
        "one read per decision"
    );
    for (n, operation) in decided_in_order.iter().enumerate() {
        assert_eq!(
            operation.decision_time,
            Some(clock.reading(n)),
            "operation {}",
            operation.operation_id.as_str()
        );
    }
}

#[test]
fn returned_results_carry_their_own_reading_through_atomic_and_interleaved() {
    let ir = model(OFFERS);
    for interleaved in [false, true] {
        for seed in 0..8 {
            let clock = offers_clock();
            let target = offers(Some(clock.clone()));
            let history = if interleaved {
                record::record(&ir, &DecideAtInvoke(&target), &two_acceptances(), seed)
            } else {
                record::record(&ir, &Atomic(&target), &two_acceptances(), seed)
            }
            .expect("recorded");
            assert_eq!(history.format, HistoryFormat::EssHistory2);
            // Atomic decides at the return instant, the fixture at the invoke instant.
            let mut order: Vec<&Operation> = history.operations.iter().collect();
            if interleaved {
                order.sort_by_key(|operation| operation.invoked_at);
            } else {
                order.sort_by_key(|operation| operation.returned_at);
            }
            assert_receipts(&clock, &history, &order);
            let accepted: Vec<_> = commands(&history, "demo.offers.AcceptOffer")
                .into_iter()
                .map(|operation| {
                    (
                        operation.outcome.as_ref().map(|it| it.as_str().to_owned()),
                        operation.decision_time,
                    )
                })
                .collect();
            assert!(
                accepted.contains(&(Some("accepted".to_owned()), Some(instant(EARLY))))
                    && accepted.contains(&(Some("lapsed".to_owned()), Some(instant(LATE)))),
                "seed {seed}: {accepted:?}"
            );
            assert_eq!(written(&ir, &history), history, "seed {seed}");
            assert_eq!(
                verdict(&ir, &history),
                Verdict::Linearizable,
                "seed {seed}, interleaved {interleaved}"
            );
        }
    }
}

/// The first seed whose acceptances overlap and complete in the other order than they were
/// invoked, through the fixture deciding at invoke.
fn out_of_order_seed(ir: &EssIr) -> u64 {
    (0..256)
        .find(|&seed| {
            let target = offers(Some(offers_clock()));
            let history =
                record::record(ir, &DecideAtInvoke(&target), &two_acceptances(), seed).unwrap();
            let [first, second] = commands(&history, "demo.offers.AcceptOffer")[..] else {
                return false;
            };
            first.invoked_at < second.invoked_at
                && second.invoked_at < second.returned_at.unwrap()
                && second.returned_at < first.returned_at
        })
        .expect("a seed completing the two acceptances out of order")
}

#[test]
fn out_of_order_completion_keeps_each_reading() {
    let ir = model(OFFERS);
    let seed = out_of_order_seed(&ir);
    let clock = offers_clock();
    let target = offers(Some(clock.clone()));
    let history = record::record(&ir, &DecideAtInvoke(&target), &two_acceptances(), seed).unwrap();
    let mut order: Vec<&Operation> = history.operations.iter().collect();
    order.sort_by_key(|operation| operation.invoked_at);
    assert_receipts(&clock, &history, &order);
    assert_ne!(
        commands(&history, "demo.offers.AcceptOffer")[0].decision_time,
        commands(&history, "demo.offers.AcceptOffer")[1].decision_time
    );
    assert_eq!(verdict(&ir, &history), Verdict::Linearizable);

    // Fault: each completion takes the reading of the call invoked last.
    let target = offers(Some(offers_clock()));
    let faulty = WrongPending {
        target: &target,
        last: Cell::new(None),
    };
    let corrupted = record::record(&ir, &faulty, &two_acceptances(), seed).unwrap();
    assert_eq!(
        verdict(&ir, &corrupted),
        Verdict::Violation,
        "a reading recorded on the wrong operation"
    );
}

#[test]
fn wall_clock_substitution_changes_the_verdict() {
    let ir = model(OFFERS);
    for seed in 0..8 {
        let target = offers(Some(offers_clock()));
        let corrupted =
            record::record(&ir, &WallSubstitution(&target), &two_acceptances(), seed).unwrap();
        assert!(corrupted
            .operations
            .iter()
            .all(|operation| operation.decision_time == Some(instant(WALL))));
        assert_eq!(verdict(&ir, &corrupted), Verdict::Violation, "seed {seed}");
    }
}

#[test]
fn reread_at_completion_fails_the_provider_control_and_the_history_verdict() {
    let ir = model(OFFERS);
    // Every decision reads what the healthy recording reads; every completion reads a later
    // instant the decision never used.
    let clock = Scripted::new(&[
        SETUP[0],
        "2026-10-04T13:00:01Z",
        SETUP[1],
        "2026-10-04T13:00:02Z",
        EARLY,
        "2026-10-04T13:00:03Z",
        LATE,
        "2026-10-04T13:00:04Z",
    ]);
    let target = offers(Some(clock.clone()));
    let faulty = RereadAtCompletion {
        target: &target,
        clock: clock.clone(),
    };
    let history = record::record(&ir, &faulty, &two_acceptances(), 0).unwrap();
    // The scripted provider is the authority for a clock-edge fault: two reads per decision, and
    // a recorded instant that is not the one the decision used.
    assert_eq!(clock.reads(), 2 * history.operations.len());
    assert!(history.operations.iter().all(|operation| {
        operation.decision_time.is_some_and(|recorded| {
            (0..clock.reads())
                .step_by(2)
                .all(|decided| clock.reading(decided) != recorded)
        })
    }));

    // Rule 18: a reread recorded in place of the decision's instant is a recorder corruption, so the
    // history verdict decides it too. The healthy recording of the same decisions is complete,
    // decisive and `Linearizable`; the corrupted one, with the same outcomes, is a `Violation`.
    let healthy_clock = offers_clock();
    let healthy_target = offers(Some(healthy_clock));
    let healthy = record::record(&ir, &Atomic(&healthy_target), &two_acceptances(), 0).unwrap();
    assert!(healthy
        .operations
        .iter()
        .all(|operation| operation.completion == Completion::Returned));
    let outcomes = |history: &History| -> Vec<Option<String>> {
        history
            .operations
            .iter()
            .map(|operation| operation.outcome.as_ref().map(|it| it.as_str().to_owned()))
            .collect()
    };
    assert_eq!(outcomes(&history), outcomes(&healthy), "the same decisions");
    assert_eq!(verdict(&ir, &healthy), Verdict::Linearizable);
    assert_eq!(verdict(&ir, &history), Verdict::Violation);
}

// ---- failures --------------------------------------------------------------------------------------

/// One client accepting the first offer and then archiving it: an archive that answers `archived`
/// requires the acceptance to have taken effect.
fn accept_then_archive() -> Workload {
    Workload {
        prefix: vec![open_call()],
        clients: vec![vec![
            accept_call(0),
            call("demo.offers.ArchiveOffer", &[], Subject::Created(0)),
        ]],
    }
}

/// One client accepting the first offer, nothing observing it afterwards.
fn accept_only() -> Workload {
    Workload {
        prefix: vec![open_call()],
        clients: vec![vec![accept_call(0)]],
    }
}

fn failing_recording(failure: Failure, workload: &Workload) -> (Arc<Scripted>, History) {
    let ir = model(OFFERS);
    let clock = Scripted::new(&[SETUP[0], EARLY]);
    let target = offers(Some(clock.clone()));
    let adapter = Failing {
        target: &target,
        failure,
        failed: Cell::new(false),
    };
    let history = record::record(&ir, &adapter, workload, 0).expect("recorded");
    (clock, history)
}

#[test]
fn failure_before_decision_carries_no_time() {
    let ir = model(OFFERS);
    let (clock, history) = failing_recording(Failure::BeforeDecision, &accept_only());
    let accept = commands(&history, "demo.offers.AcceptOffer")[0];
    assert_eq!(accept.completion, Completion::Indeterminate);
    assert_eq!(accept.decision_time, None, "no decision, no invented time");
    assert_eq!(clock.reads(), 1, "only the setup decided");
    assert_eq!(verdict(&ir, &history), Verdict::Linearizable);
}

#[test]
fn failure_after_decision_keeps_its_time_on_the_error() {
    let ir = model(OFFERS);
    let (clock, history) = failing_recording(Failure::AfterDecision, &accept_then_archive());
    let accept = commands(&history, "demo.offers.AcceptOffer")[0];
    assert_eq!(accept.completion, Completion::Indeterminate);
    assert_eq!(accept.returned_at, None);
    assert_eq!(accept.outcome, None);
    assert_eq!(accept.decision_time, Some(instant(EARLY)));
    assert_eq!(
        clock.reads(),
        3,
        "setup, the lost acceptance and the archive"
    );
    let archive = commands(&history, "demo.offers.ArchiveOffer")[0];
    assert_eq!(
        archive.outcome.as_ref().map(QualifiedName::as_str),
        Some("archived"),
        "a later observation requiring the lost acceptance to have taken effect"
    );
    assert_eq!(history.format, HistoryFormat::EssHistory2);
    assert_eq!(written(&ir, &history), history);
    assert_eq!(verdict(&ir, &history), Verdict::Linearizable);

    // Fault: the error loses the receipt's time, and the acceptance the archive needs can no
    // longer be decided.
    let (_, dropped) = failing_recording(Failure::DroppedMetadata, &accept_then_archive());
    assert_eq!(
        commands(&dropped, "demo.offers.AcceptOffer")[0].decision_time,
        None
    );
    undetermined(&ir, &dropped);
}

#[test]
fn missing_evidence_stays_unknown() {
    let ir = model(OFFERS);
    let clock = offers_clock();
    let target = offers(Some(clock.clone()));
    let history = record::record(&ir, &Discarding(&target), &two_acceptances(), 0).unwrap();
    assert_eq!(clock.reads(), 4, "the target still decided with its clock");
    assert!(history
        .operations
        .iter()
        .all(|operation| operation.decision_time.is_none()));
    assert_eq!(
        history.format,
        HistoryFormat::EssHistory1,
        "no operation records a time"
    );
    let bytes = serde_json::to_string(&history).unwrap();
    assert!(
        bytes.starts_with(r#"{"format":"ess-history/1","#),
        "{bytes}"
    );
    assert!(!bytes.contains("decision_time"), "{bytes}");
    // A returned `now` decision without its reading proves nothing either way.
    undetermined(&ir, &history);
}

// ---- a retained retry ------------------------------------------------------------------------------

/// `demo.claims`, by hand: `Settle` decides with one reading and retains its answer; the same
/// request sent again is answered from what was retained, with no new decision and no reading.
struct Claims {
    clock: Arc<Scripted>,
    /// Fault: a retained answer carries the reading of the decision it replays.
    copy_retry_time: bool,
    state: RefCell<ClaimState>,
}

#[derive(Default)]
struct ClaimState {
    held: BTreeMap<String, &'static str>,
    retained: Vec<(
        SemanticCommandRequest,
        SemanticCommandResult,
        Option<DecisionInstant>,
    )>,
    minted: u64,
}

fn taken(command: &str, outcome: &str) -> OutcomeRef {
    OutcomeRef::new(command.parse().unwrap(), outcome.parse().unwrap())
}

impl Claims {
    fn new(clock: Arc<Scripted>, copy_retry_time: bool) -> Self {
        Self {
            clock,
            copy_retry_time,
            state: RefCell::default(),
        }
    }
}

impl ConformanceTarget for Claims {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("claims-fixture", "1"))
    }

    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.state.borrow_mut() = ClaimState::default();
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.execute_command_recorded(request).answer
    }

    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        let mut state = self.state.borrow_mut();
        let command = request.command.to_string();
        if command == "demo.claims.Settle" {
            if let Some((_, answered, decided)) =
                state.retained.iter().find(|(sent, _, _)| *sent == request)
            {
                let mut replayed = answered.clone();
                replayed.outcome = Some(taken(&command, "replayed"));
                replayed.direct_events = Vec::new();
                return RecordedCommandCompletion {
                    answer: Ok(replayed),
                    decision_time: if self.copy_retry_time { *decided } else { None },
                };
            }
        }
        let decision_time = self.clock.read();
        let now = decision_time.expect("the scripted clock answers").instant();
        let answer = match command.as_str() {
            "demo.claims.Open" => {
                state.minted += 1;
                let claim = format!("00000000-0000-4000-8000-{:012}", state.minted);
                state.held.insert(claim.clone(), "Open");
                SemanticCommandResult::took(taken(&command, "opened")).emitting(
                    ObservedEvent::new("demo.claims.Opened".parse().unwrap())
                        .with("claim_id", Node::Text(claim)),
                )
            }
            "demo.claims.Settle" => {
                let claim = request.input["claim_id"].as_text().unwrap().to_owned();
                let due = ess_primitives::time::Rfc3339Instant::parse_rfc3339(
                    request.input["due_at"].as_text().unwrap(),
                )
                .unwrap();
                let mut result = if state.held.get(&claim) == Some(&"Open") && due >= now {
                    state.held.insert(claim, "Settled");
                    SemanticCommandResult::took(taken(&command, "settled"))
                        .emitting(ObservedEvent::new("demo.claims.Changed".parse().unwrap()))
                } else {
                    SemanticCommandResult::took(taken(&command, "lapsed"))
                        .emitting(ObservedEvent::new("demo.claims.Changed".parse().unwrap()))
                };
                result.response = Some(BTreeMap::from([(
                    "revision".to_owned(),
                    Node::Number(Number::from(1_i64)),
                )]));
                state
                    .retained
                    .push((request.clone(), result.clone(), decision_time));
                result
            }
            other => {
                return RecordedCommandCompletion {
                    answer: Err(TargetError::unsupported("command", other)),
                    decision_time: None,
                }
            }
        };
        RecordedCommandCompletion {
            answer: Ok(answer),
            decision_time,
        }
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", request.view.to_string()))
    }

    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }

    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "forcing",
            "a replay is never forced",
        ))
    }

    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "no binding"))
    }

    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
}

fn settle_with_retry() -> sessions::Workload {
    sessions::Workload {
        prefix: vec![call("demo.claims.Open", &[], Subject::Creates)],
        clients: vec![vec![Act::Call(call(
            "demo.claims.Settle",
            &[("due_at", DEADLINE)],
            Subject::Created(0),
        ))]],
    }
}

fn record_claims(copy_retry_time: bool, seed: u64, interleaved: bool) -> (Arc<Scripted>, History) {
    let ir = model(CLAIMS);
    let clock = Scripted::new(&[SETUP[0], EARLY]);
    let target = Claims::new(clock.clone(), copy_retry_time);
    begin(&target);
    let recorded = if interleaved {
        sessions::record_injected(
            &ir,
            &DecideAtInvoke(&target),
            &target,
            &settle_with_retry(),
            seed,
        )
    } else {
        sessions::record_injected(&ir, &Atomic(&target), &target, &settle_with_retry(), seed)
    }
    .expect("recorded");
    (clock, recorded.history)
}

#[test]
fn retained_retry_carries_no_fresh_time() {
    let ir = model(CLAIMS);
    for interleaved in [false, true] {
        for seed in 0..8 {
            let (clock, history) = record_claims(false, seed, interleaved);
            let settles = commands(&history, "demo.claims.Settle");
            assert_eq!(settles.len(), 2, "the request and its retry");
            assert!(settles.iter().any(|operation| operation.retry_of.is_some()));
            let replayed: Vec<_> = settles
                .iter()
                .filter(|operation| {
                    operation.outcome.as_ref().map(QualifiedName::as_str) == Some("replayed")
                })
                .collect();
            assert_eq!(replayed.len(), 1, "seed {seed}");
            assert_eq!(replayed[0].decision_time, None, "a replay decided nothing");
            assert_eq!(
                clock.reads(),
                2,
                "the open and the one fresh settlement read the provider; the replay did not"
            );
            assert_eq!(written(&ir, &history), history);
            assert_eq!(
                verdict(&ir, &history),
                Verdict::Linearizable,
                "seed {seed}, interleaved {interleaved}"
            );

            // Fault: the replay carries the reading of the decision it replays.
            let (_, copied) = record_claims(true, seed, interleaved);
            assert_eq!(
                verdict(&ir, &copied),
                Verdict::Violation,
                "seed {seed}, interleaved {interleaved}: a replay claiming a decision"
            );
        }
    }
}
