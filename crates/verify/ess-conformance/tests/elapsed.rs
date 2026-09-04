//! What a duration claim comes to when something actually runs it.
//!
//! The reason this file exists rather than a line in `execution.rs`: the failure a duration claim is
//! written to prevent is **silence read as agreement**. Every other assertion in the suite fails
//! loudly when a target cannot answer it — a view that is not there, an event that did not arrive.
//! A window is different: a target that never waits, never advances a clock and never says so leaves
//! nothing behind that a passing run would look different from.
//!
//! So the cases below are not "does the happy path work". They are the four ways a window can be
//! wrong, each asserted to reach a *different* verdict:
//!
//! | the target | the verdict | why it is that one |
//! |---|---|---|
//! | holds the window and reports it | `passed` | the claim was checked and held |
//! | reports less than the claim asked for | `failed` | the window did not close, and the runner will not round towards the claim |
//! | publishes the watched event inside it | `failed` | the bounded negative is what the window is for |
//! | implements no clock at all | `unsupported` | §28 — a target that could not answer has not shown the answer, and the run fails |
//!
//! The last row is the whole point. There is no fifth row where nothing happens and the scenario
//! passes.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_conformance::report::{CheckCode, ConformanceStatus, Status};
use ess_conformance::runner::Runner;
use ess_conformance::scenario::{
    ConformanceScenario, ConformanceSuite, Elapsed, EventRef, InstantName, ScenarioId,
    ScenarioStep, SuiteFormat, SuiteProvenance,
};
use ess_conformance::target::{
    ConformanceTarget, ElapsedObservation, ElapsedObservationRequest, EventObservationRequest,
    ExternalOutcomeControl, ImplementationIdentity, InstantMark, ObservedEvent, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_primitives::evidence::SpecDigest;

// ---- a target that keeps a clock of its own ------------------------------------------------------

/// An implementation whose clock is a number it owns.
///
/// The second of the three clock models a duration claim could have been built on, and the one that
/// makes this file run in no time at all: nothing sleeps, `advance` is an addition, and two runs
/// produce the same report because there is no wall clock anywhere to disagree about. That is the
/// point of asking a target for a *reading* rather than telling it how to keep time — a target with
/// a clock it can move answers instantly and honestly, and the suite cannot tell the difference from
/// one that waited, because there is no difference in what was claimed.
struct Ticking {
    /// What the clock reads now, in milliseconds since the scenario began.
    now: RefCell<u64>,
    /// The instant each mark named.
    marked: RefCell<BTreeMap<InstantName, u64>>,
    /// How the target is wrong, where it is.
    how: Wrong,
}

/// The one thing a [`Ticking`] does differently.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Wrong {
    /// Nothing. The clock advances when asked and reports what it reads.
    Nothing,
    /// The clock does not move: every window is answered immediately, with a reading of zero.
    ///
    /// The defect a duration claim exists to catch, and the one nothing else in the suite can see.
    /// A system that fires every timer the instant it is armed passes every check ESS could write
    /// before this: the consequence arrives, it carries what it declares, and the view holds it.
    NeverHolds,
    /// The clock advances, and the watched event is published inside the window.
    NotQuiet,
    /// The clock runs ahead: a window claimed to be closed within five seconds took twenty.
    Slow,
}

impl Ticking {
    fn new(how: Wrong) -> Self {
        Self {
            now: RefCell::new(0),
            marked: RefCell::new(BTreeMap::new()),
            how,
        }
    }
}

impl ConformanceTarget for Ticking {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("ticking", "1"))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.now.borrow_mut() = 0;
        self.marked.borrow_mut().clear();
        Ok(())
    }

    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.marked
            .borrow_mut()
            .insert(request.instant, *self.now.borrow());
        Ok(())
    }

    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        let opened = *self.marked.borrow().get(&request.instant).ok_or_else(|| {
            TargetError::unavailable(
                "a marked instant",
                format!("`{}` is not marked", request.instant),
            )
        })?;
        if self.how != Wrong::NeverHolds {
            // The whole of the target's side of a hold: move the clock to the end of the window, or
            // leave it where it is if the window has already closed.
            let closes = opened + request.hold.millis();
            let mut now = self.now.borrow_mut();
            *now = (*now).max(closes);
            if self.how == Wrong::Slow {
                *now += 15_000;
            }
        }
        Ok(ElapsedObservation {
            elapsed_ms: (*self.now.borrow()).saturating_sub(opened),
            published: usize::from(self.how == Wrong::NotQuiet && request.watching.is_some()),
        })
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(
            request.command.to_string(),
            "this target runs windows and nothing else",
        ))
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            request.view.to_string(),
            "this target runs windows and nothing else",
        ))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported(
            request.event.to_string(),
            "this target runs windows and nothing else",
        ))
    }

    fn configure_external_outcome(
        &self,
        _request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "an external outcome",
            "this target runs windows and nothing else",
        ))
    }

    fn redeliver_event(&self, _request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "a redelivery",
            "this target runs windows and nothing else",
        ))
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
}

/// A target with no clock at all, which is most of them.
///
/// It implements the nine methods that existed before duration claims did and not one line more —
/// which is the compatibility claim made executable: this type would have compiled against the
/// earlier interface unchanged, and it still compiles now.
struct Clockless;

impl ConformanceTarget for Clockless {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("clockless", "1"))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(
            request.command.to_string(),
            "nothing here runs a command",
        ))
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            request.view.to_string(),
            "nothing here reads a view",
        ))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported(
            request.event.to_string(),
            "nothing here observes an event",
        ))
    }

    fn configure_external_outcome(
        &self,
        _request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "an external outcome",
            "nothing here injects one",
        ))
    }

    fn redeliver_event(&self, _request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "a redelivery",
            "nothing here delivers anything",
        ))
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
}

// ---- the suite these targets are run against -----------------------------------------------------

fn instant(name: &str) -> InstantName {
    name.parse().expect("a lower-kebab instant name")
}

fn event() -> EventRef {
    "billing.invoice.InvoicePaid".parse().expect("an event")
}

fn scenario_id() -> ScenarioId {
    ScenarioId::parse("billing.invoice.PayInvoice/outcome/settled").expect("a scenario id")
}

/// A suite of one scenario: a marked instant, a hold, a bounded negative and a deadline.
fn suite(steps: Vec<ScenarioStep>) -> ConformanceSuite {
    let digest = |value: &str| SpecDigest::new(value).expect("a digest");
    let mut suite = ConformanceSuite::new(SuiteProvenance {
        suite_version: SuiteFormat::CURRENT,
        system: "billing".to_owned(),
        specification_version: "v3".to_owned(),
        spec_digest: digest("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        contract_digest: digest("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
        component: None,
    });
    suite
        .insert(
            scenario_id(),
            ConformanceScenario::new(
                "a window of twenty seconds really passes and nothing happens in it"
                    .parse()
                    .expect("a purpose"),
                steps,
                [],
            ),
        )
        .expect("the id is free");
    suite
}

/// The three claims, over one marked instant.
fn windows() -> Vec<ScenarioStep> {
    vec![
        ScenarioStep::MarkInstant {
            instant: instant("bridged"),
        },
        ScenarioStep::ExpectNotBefore {
            instant: instant("bridged"),
            elapsed: Elapsed::seconds(20),
        },
        ScenarioStep::ExpectQuiet {
            event: event(),
            instant: instant("bridged"),
            elapsed: Elapsed::seconds(20),
        },
    ]
}

/// Which check codes a run recorded, and at which status.
fn checks(report: &ess_conformance::report::ConformanceReport) -> Vec<(CheckCode, Status, String)> {
    report
        .scenarios
        .iter()
        .flat_map(|result| result.checks.iter())
        .map(|check| (check.code, check.status, check.about.clone()))
        .collect()
}

// ---- the four verdicts ---------------------------------------------------------------------------

#[test]
fn a_target_that_holds_the_window_and_reports_it_passes() {
    let suite = suite(windows());
    let report = Runner::for_suite(&suite).run(&suite, &Ticking::new(Wrong::Nothing));

    assert_eq!(report.status, ConformanceStatus::Passed);
    let recorded = checks(&report);
    assert_eq!(
        recorded
            .iter()
            .map(|(code, status, _)| (*code, *status))
            .collect::<Vec<_>>(),
        vec![
            (CheckCode::Elapsed, Status::Passed),
            (CheckCode::Quiet, Status::Passed),
        ]
    );
    assert_eq!(
        recorded[0].2, "at least PT20S has passed since `bridged`",
        "the claim is stated in the words the author wrote it in"
    );
}

#[test]
fn a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited() {
    // The whole feature, as one assertion. A system that fires every timer the instant it is armed
    // is indistinguishable from a correct one under every check ESS could write before this: the
    // consequence arrives, it carries what it declares, the view holds it. What separates them is
    // that twenty seconds did not pass, and nothing said so.
    let suite = suite(windows());
    let report = Runner::for_suite(&suite).run(&suite, &Ticking::new(Wrong::NeverHolds));

    assert_eq!(report.status, ConformanceStatus::Failed);
    let recorded = checks(&report);
    assert_eq!(recorded[0].0, CheckCode::Elapsed);
    assert_eq!(recorded[0].1, Status::Failed);
    let diagnostic = report.scenarios[0].checks[0]
        .diagnostic
        .as_ref()
        .expect("a failed check carries one")
        .to_string();
    assert!(
        diagnostic.contains("the target measured 0ms since `bridged`"),
        "the diagnostic says what the target reported rather than that the check failed: \
         {diagnostic}"
    );

    // And the bounded negative fails for the *window*, not for the event: nothing was published,
    // and the claim still does not hold, because the window it is about never closed.
    assert_eq!(recorded[1].0, CheckCode::Quiet);
    assert_eq!(recorded[1].1, Status::Failed);
    let quiet = report.scenarios[0].checks[1]
        .diagnostic
        .as_ref()
        .expect("a failed check carries one")
        .to_string();
    assert!(
        quiet.contains("shorter than the one the scenario claims"),
        "a window that did not close is not a window that was quiet: {quiet}"
    );
}

#[test]
fn an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else() {
    let suite = suite(windows());
    let report = Runner::for_suite(&suite).run(&suite, &Ticking::new(Wrong::NotQuiet));

    assert_eq!(report.status, ConformanceStatus::Failed);
    assert_eq!(
        checks(&report)
            .iter()
            .map(|(code, status, _)| (*code, *status))
            .collect::<Vec<_>>(),
        vec![
            (CheckCode::Elapsed, Status::Passed),
            (CheckCode::Quiet, Status::Failed),
        ],
        "the hold held; what failed is what happened during it"
    );
}

#[test]
fn a_deadline_the_target_ran_past_fails_the_within_claim() {
    let suite = suite(vec![
        ScenarioStep::MarkInstant {
            instant: instant("bridged"),
        },
        ScenarioStep::ExpectNotBefore {
            instant: instant("bridged"),
            elapsed: Elapsed::seconds(20),
        },
        ScenarioStep::ExpectWithin {
            instant: instant("bridged"),
            elapsed: Elapsed::seconds(30),
        },
    ]);
    // `Slow` adds fifteen seconds on every reading, so the hold lands at 35s and the deadline of
    // thirty is missed by a system that did arrive — which is the claim `within` exists to make.
    let report = Runner::for_suite(&suite).run(&suite, &Ticking::new(Wrong::Slow));

    assert_eq!(report.status, ConformanceStatus::Failed);
    let recorded = checks(&report);
    assert_eq!(
        recorded[0].1,
        Status::Passed,
        "the hold was more than enough"
    );
    assert_eq!(recorded[1].0, CheckCode::Elapsed);
    assert_eq!(recorded[1].1, Status::Failed);
    assert_eq!(
        recorded[1].2,
        "no more than PT30S has passed since `bridged`"
    );
}

#[test]
fn a_target_with_no_clock_reports_unsupported_and_the_run_fails() {
    // §28, and the one row that must never be `passed`. A target implementing only the methods that
    // existed before duration claims did still compiles, still runs every other scenario, and is
    // told apart from a target that checked the window and found it held.
    let suite = suite(windows());
    let report = Runner::for_suite(&suite).run(&suite, &Clockless);

    assert_ne!(
        report.status,
        ConformanceStatus::Passed,
        "a window nobody held is never a window that passed"
    );
    let recorded = checks(&report);
    assert_eq!(
        recorded.len(),
        1,
        "the scenario stops at the mark it could not make"
    );
    assert_eq!(recorded[0].0, CheckCode::Target);
    assert_eq!(recorded[0].1, Status::Unsupported);
    assert_eq!(report.scenarios[0].status, Status::Unsupported);
}

#[test]
fn a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation()
{
    // The compiler refuses this in an authored file (`ESS-AUTHOR-028`), so it can only reach a
    // runner from a hand-built suite. It is still told apart from a wrong implementation: an
    // `error` naming the suite, because a window measured from nothing is a question, not an answer.
    let suite = suite(vec![ScenarioStep::ExpectNotBefore {
        instant: instant("bridged"),
        elapsed: Elapsed::seconds(20),
    }]);
    let report = Runner::for_suite(&suite).run(&suite, &Ticking::new(Wrong::Nothing));

    let recorded = checks(&report);
    assert_eq!(recorded[0].0, CheckCode::Suite);
    assert_eq!(recorded[0].1, Status::Error);
    let diagnostic = report.scenarios[0].checks[0]
        .diagnostic
        .as_ref()
        .expect("an errored check carries one")
        .to_string();
    assert!(
        diagnostic.contains("no instant has been marked in this scenario"),
        "{diagnostic}"
    );
}

#[test]
fn two_runs_over_one_window_produce_byte_identical_reports() {
    // §37, for the half of the vocabulary that is about time. A logical clock is what makes this
    // assertable at all: a target that waited on a wall clock would report a different number of
    // milliseconds every run, which is exactly why the *claim* is a length and the *reading* is the
    // target's, and why the report records the verdict rather than the measurement.
    let suite = suite(windows());
    let first = Runner::for_suite(&suite)
        .run(&suite, &Ticking::new(Wrong::Nothing))
        .to_canonical_json();
    let second = Runner::for_suite(&suite)
        .run(&suite, &Ticking::new(Wrong::Nothing))
        .to_canonical_json();
    assert_eq!(first, second);
}
