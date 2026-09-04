//! What an early stop comes to when something actually runs it.
//!
//! The reason this file exists rather than a line in `execution.rs`: an early stop is the second
//! claim in this vocabulary — after a window — where *being ignored* and *being satisfied* look
//! identical from outside. Every other assertion fails loudly when a target does not honour it,
//! because the rows are the evidence and the rows are wrong. Here the rows are the same either way.
//! A target that reads the whole listing and hands back the first two rows returns exactly what a
//! target that pulled two and stopped returns.
//!
//! So the cases below are not "does the happy path work". They are the four ways a halt claim can be
//! answered, each asserted to reach a *different* verdict:
//!
//! | the target | the verdict | why it is that one |
//! |---|---|---|
//! | stops pulling when the reader says stop | `passed` | the claim was checked and held |
//! | materialises the listing and hands back a prefix | `failed` | the reader stopped, the producer did not, and this is the defect |
//! | holds fewer rows than the reader would take | `failed` | the read ended because the source ran out, which is not a halt |
//! | cannot read a view a row at a time | `unsupported` | §28 — a target that could not answer has not shown the answer, and the run fails |
//!
//! The last two rows are the ones a weaker design gets wrong. A runner that compared only the count
//! would pass the third against a listing of exactly the right length; a runner that took the
//! target's word for "I stopped" would pass the second.

use std::cell::RefCell;

use ess_conformance::report::{CheckCode, ConformanceStatus, Status};
use ess_conformance::runner::Runner;
use ess_conformance::scenario::{
    ConformanceScenario, ConformanceSuite, ScenarioId, ScenarioStep, SuiteFormat, SuiteProvenance,
    ViewRef,
};
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, OrderedScan, OrderedScanRequest, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_primitives::evidence::SpecDigest;

// ---- a target that reads a listing one row at a time ----------------------------------------------

/// An implementation whose ordered listing is a number of rows and a way of iterating them.
///
/// Nothing here is a view in any real sense, and that is deliberate: the only thing under test is
/// what a target reports about *its own iteration*, so the fixture is the iteration and nothing
/// else. The three ways of being wrong differ in one line each.
struct Listing {
    /// How many rows the ordered source has to give.
    rows: usize,
    /// How the target is wrong, where it is.
    how: Wrong,
    /// How many times the runner has asked. Read by the case about retrying.
    asked: RefCell<usize>,
}

/// The one thing a [`Listing`] does differently.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Wrong {
    /// Nothing. The reader's stop stops the producer, and the target says how many rows it produced.
    Nothing,
    /// The listing is materialised before anybody looks at it.
    ///
    /// The defect an early stop exists to catch, and the one nothing else in the suite can see. Its
    /// rows are the right rows, in the right order, and the reader's stop stopped the *reader*. The
    /// honest report of that is `produced = rows`, and it is what turns the scenario red.
    Materialises,
    /// The listing is short: the source runs out before the reader gets to say stop.
    ///
    /// Reported honestly — `halted: false` — and it must still fail, because a read that ended
    /// because there was nothing left is not a read a consumer stopped.
    RunsOut,
    /// The listing fills up on the second ask, which is what an eventual projection does.
    Lagging,
}

impl Listing {
    fn new(how: Wrong, rows: usize) -> Self {
        Self {
            rows,
            how,
            asked: RefCell::new(0),
        }
    }
}

impl ConformanceTarget for Listing {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("listing", "1"))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.asked.borrow_mut() = 0;
        Ok(())
    }

    fn scan_view(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        let mut asked = self.asked.borrow_mut();
        *asked += 1;
        // The lagging target has nothing to give on the first ask, which is the ordinary state of a
        // projection that has not caught up — and is why the eventual form of the claim retries the
        // whole bounded read rather than deciding on the first answer.
        let available = match self.how {
            Wrong::RunsOut => request.stop_after.saturating_sub(1),
            Wrong::Lagging if *asked == 1 => 0,
            _ => self.rows,
        };
        // The target's own loop, written out because what it reports is a fact about this loop and
        // not about the rows it returns.
        let mut produced = 0;
        let mut halted = false;
        while produced < available {
            produced += 1;
            if produced >= request.stop_after {
                halted = true;
                break;
            }
        }
        if self.how == Wrong::Materialises {
            // The whole listing was pulled before the reader ever saw a row. The reader still said
            // stop and the target still honours it — what it cannot honestly claim is that the stop
            // reached the producer.
            produced = available;
        }
        Ok(OrderedScan { produced, halted })
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(
            request.command.to_string(),
            "this target reads a listing and nothing else",
        ))
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            request.view.to_string(),
            "this target reads a listing one row at a time and no other way",
        ))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported(
            request.event.to_string(),
            "this target reads a listing and nothing else",
        ))
    }

    fn configure_external_outcome(
        &self,
        _request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "an external outcome",
            "this target reads a listing and nothing else",
        ))
    }

    fn redeliver_event(&self, _request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "a redelivery",
            "this target reads a listing and nothing else",
        ))
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
}

/// A target that reads a view only in one piece, which is most of them.
///
/// It implements the methods that existed before an early stop did and not one line more — the
/// compatibility claim made executable: this type would have compiled against the earlier interface
/// unchanged, and it still compiles now.
struct Wholesale;

impl ConformanceTarget for Wholesale {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("wholesale", "1"))
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

    fn query_view(&self, _request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        // It reads views perfectly well. What it cannot do is read one a row at a time, which is the
        // whole point: this is not a broken target, it is a complete one that has to say
        // `unsupported` to exactly one claim.
        Ok(SemanticViewResult::default())
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

fn view() -> ViewRef {
    "billing.invoice.OutstandingInvoices"
        .parse()
        .expect("a view")
}

fn scenario_id() -> ScenarioId {
    ScenarioId::parse("billing.invoice/authored/a-reader-stops-the-listing").expect("a scenario id")
}

/// A suite of one scenario: one bounded ordered read, claimed to have been stopped.
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
                "a reader of the ranked listing takes two rows and stops the producer"
                    .parse()
                    .expect("a purpose"),
                steps,
                [],
            ),
        )
        .expect("the id is free");
    suite
}

/// The claim, made of a listing the model calls `read_your_writes`.
fn halt() -> Vec<ScenarioStep> {
    vec![ScenarioStep::ExpectHalt {
        view: view(),
        params: std::collections::BTreeMap::new(),
        after: 2,
    }]
}

/// The same claim, made of a listing the model calls `eventual`.
fn eventual_halt() -> Vec<ScenarioStep> {
    vec![ScenarioStep::EventuallyHalt {
        view: view(),
        params: std::collections::BTreeMap::new(),
        after: 2,
    }]
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

/// The diagnostic of the first check, which is what a reader is told.
fn diagnostic(report: &ess_conformance::report::ConformanceReport) -> String {
    report.scenarios[0].checks[0]
        .diagnostic
        .as_ref()
        .expect("a check that did not pass carries one")
        .to_string()
}

// ---- the four verdicts ---------------------------------------------------------------------------

#[test]
fn a_target_whose_producer_stops_when_the_reader_does_passes() {
    let suite = suite(halt());
    let report = Runner::for_suite(&suite).run(&suite, &Listing::new(Wrong::Nothing, 5));

    assert_eq!(report.status, ConformanceStatus::Passed);
    let recorded = checks(&report);
    assert_eq!(
        recorded
            .iter()
            .map(|(code, status, _)| (*code, *status))
            .collect::<Vec<_>>(),
        vec![(CheckCode::Halt, Status::Passed)]
    );
    assert_eq!(
        recorded[0].2,
        "reading `billing.invoice.OutstandingInvoices` stops after 2 row(s) because the reader did",
        "the claim is stated in the words it was written in"
    );
}

#[test]
fn a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped() {
    // The whole feature, as one assertion. This target returns the right rows, in the right order,
    // and honours the reader's stop; under `at:`, `ranked:` and `counts:` it is indistinguishable
    // from a correct one. What separates them is that it pulled five rows to hand over two, and
    // nothing said so.
    let suite = suite(halt());
    let report = Runner::for_suite(&suite).run(&suite, &Listing::new(Wrong::Materialises, 5));

    assert_eq!(report.status, ConformanceStatus::Failed);
    let recorded = checks(&report);
    assert_eq!(recorded[0].0, CheckCode::Halt);
    assert_eq!(recorded[0].1, Status::Failed);
    let rendered = diagnostic(&report);
    assert!(
        rendered.contains("the target produced 5 row(s) to hand it 2"),
        "the diagnostic says what the target reported rather than that the check failed: {rendered}"
    );
}

#[test]
fn a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt() {
    // The reason the observation carries a flag beside the count. This target holds one row, so it
    // produces one and ends because there is nothing left. A runner comparing only "did it produce
    // no more than the reader took" would pass it, and the scenario would be green against a system
    // that never demonstrated the thing it claims.
    let suite = suite(halt());
    let report = Runner::for_suite(&suite).run(&suite, &Listing::new(Wrong::RunsOut, 5));

    assert_eq!(report.status, ConformanceStatus::Failed);
    let rendered = diagnostic(&report);
    assert!(
        rendered.contains("the source ran out after 1 row(s), not because the reader stopped it"),
        "a read that ended for its own reasons is told apart from one a consumer stopped: {rendered}"
    );
}

#[test]
fn a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails() {
    // §28, and the one row that must never be `passed`. A target implementing only the methods that
    // existed before an early stop did still compiles, still runs every other scenario, and is told
    // apart from a target that read a listing and stopped it.
    let suite = suite(halt());
    let report = Runner::for_suite(&suite).run(&suite, &Wholesale);

    assert_ne!(
        report.status,
        ConformanceStatus::Passed,
        "a scan nobody stopped is never a scan that halted"
    );
    let recorded = checks(&report);
    assert_eq!(
        recorded.len(),
        1,
        "the scenario stops at the read it could not make"
    );
    assert_eq!(recorded[0].0, CheckCode::Target);
    assert_eq!(recorded[0].1, Status::Unsupported);
    assert_eq!(report.scenarios[0].status, Status::Unsupported);
    let rendered = diagnostic(&report);
    assert!(
        rendered.contains("stopped the producer or merely stopped looking"),
        "the target says why it cannot answer, in its own words: {rendered}"
    );
}

// ---- the eventual half ---------------------------------------------------------------------------

#[test]
fn a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up() {
    // The reason there are two steps rather than one. A listing a projection maintains may hold
    // nothing yet, and a scan of an empty one runs out before the reader stops it — which is lag and
    // not a wrong implementation. The whole bounded read is retried, because each attempt is its own
    // experiment: half an answer from the first attempt would say nothing about the second.
    let suite = suite(eventual_halt());
    let report = Runner::for_suite(&suite).run(&suite, &Listing::new(Wrong::Lagging, 5));

    assert_eq!(report.status, ConformanceStatus::Passed);
    assert_eq!(checks(&report)[0].1, Status::Passed);
}

#[test]
fn retrying_does_not_rescue_a_producer_that_never_stops() {
    // And the retry is not a way past the claim: a materialising listing answers the same thing
    // every time it is asked, and the deadline arrives.
    let suite = suite(eventual_halt());
    let report = Runner::for_suite(&suite).run(&suite, &Listing::new(Wrong::Materialises, 5));

    assert_eq!(report.status, ConformanceStatus::Failed);
    let rendered = diagnostic(&report);
    assert!(
        rendered.contains("still so after"),
        "the diagnostic says it was asked more than once: {rendered}"
    );
}

// ---- determinism ---------------------------------------------------------------------------------

#[test]
fn two_runs_over_one_halt_claim_produce_byte_identical_reports() {
    // §37, for the half of the vocabulary that is about iteration. Nothing here reads a clock, so
    // the only thing that could differ between two runs is what the target counted — and a count is
    // a fact about the read rather than about the machine it ran on.
    let suite = suite(halt());
    let first = Runner::for_suite(&suite)
        .run(&suite, &Listing::new(Wrong::Materialises, 5))
        .to_canonical_json();
    let second = Runner::for_suite(&suite)
        .run(&suite, &Listing::new(Wrong::Materialises, 5))
        .to_canonical_json();
    assert_eq!(first, second);
}
