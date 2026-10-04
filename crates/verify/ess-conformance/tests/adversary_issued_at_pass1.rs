//! Adversary cases for beyond10x/ess#425: the billing `IssueInvoice` takes `issued_at: Timestamp`,
//! its `issued` outcome records it (`sets: {issued_at: input.issued_at}`), and
//! `OutstandingInvoices` ranks by `issued_at desc`.
//!
//! The suite is the committed one, `suites/generated/billing/suite.json`, so the authored
//! `outstanding-invoices-rank-latest-first` scenario is in it.

use std::cell::RefCell;
use std::path::Path;

use ess_conformance::reference::Billing;
use ess_conformance::report::ConformanceStatus;
use ess_conformance::runner::Runner;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    InvocationObservationRequest, ObservedEvent, ObservedInvocation, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_primitives::node::Node;
use ess_primitives::Rfc3339Instant;

const AUTHORED: &str = "billing.invoice/authored/outstanding-invoices-rank-latest-first";

fn committed_suite_text() -> String {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../suites/generated/billing/suite.json");
    std::fs::read_to_string(path).expect("the committed billing suite exists")
}

fn only_authored(text: &str) -> ConformanceSuite {
    let mut suite = ConformanceSuite::from_json(text).expect("the committed suite parses");
    suite.scenarios.retain(|id, _| id.to_string() == AUTHORED);
    assert_eq!(
        suite.scenarios.len(),
        1,
        "the authored ranking scenario is in the suite"
    );
    suite
}

fn failures<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> Vec<String> {
    let report = Runner::for_suite(suite)
        .run(suite, target)
        .expect("admitted");
    let mut failed: Vec<String> = report
        .failures()
        .map(|result| format!("{} — {}", result.scenario, result.status))
        .collect();
    failed.extend(report.diagnostics().take(1).map(ToString::to_string));
    failed
}

/// The reference target is the suite's known-good implementation. A `Timestamp` names an instant,
/// and `Rfc3339Instant` says ordering two of them orders "the instants and not the spellings".
/// The caller now writes `issued_at`, so an offset spelling reaches the reference's ranking.
#[test]
fn reference_billing_ranks_outstanding_invoices_by_instant_not_by_spelling() {
    let text = committed_suite_text();
    // Control: the scenario as committed passes, so a failure below is the spelling alone.
    assert!(
        failures(&only_authored(&text), &Billing::new()).is_empty(),
        "control: the committed authored scenario passes against the reference"
    );

    // `earlier` is issued at 08:00:01Z, written with an offset: still before `later` (09:00:03Z),
    // and lexicographically after it.
    let offset = "2026-01-05T10:00:01+02:00";
    assert!(
        Rfc3339Instant::parse_rfc3339(offset).unwrap()
            < Rfc3339Instant::parse_rfc3339("2026-01-05T09:00:03Z").unwrap()
    );
    assert_eq!(text.matches("2026-01-05T09:00:01Z").count(), 1);
    let patched = text.replace("2026-01-05T09:00:01Z", offset);
    let wrong = failures(&only_authored(&patched), &Billing::new());
    assert!(
        wrong.is_empty(),
        "the reference ranks `issued_at` by its text, so the later instant is not first:\n{}",
        wrong.join("\n")
    );
}

/// A target that ignores `issued_at` when ranking, and puts the most recently *issued* invoice
/// first. It disagrees with `order_by: issued_at desc` for any caller that issues out of instant
/// order, which is exactly what taking `issued_at` from the caller allows.
struct RanksByIssueSequence {
    inner: Billing,
    issued: RefCell<Vec<Node>>,
}

impl ConformanceTarget for RanksByIssueSequence {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("ranks-by-issue-sequence", "0"))
    }

    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.issued.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }

    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let id = request.input.get("invoice_id").cloned();
        let result = self.inner.execute_command(request)?;
        if result.outcome.as_ref().map(ToString::to_string).as_deref()
            == Some("billing.invoice.IssueInvoice/issued")
        {
            self.issued
                .borrow_mut()
                .push(id.expect("issue names an invoice"));
        }
        Ok(result)
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let outstanding = request.view.to_string() == "billing.invoice.OutstandingInvoices";
        let mut result = self.inner.query_view(request)?;
        if outstanding {
            let issued = self.issued.borrow();
            let position = |row: &ess_conformance::target::ViewRow| {
                row.get("invoice_id")
                    .and_then(|id| issued.iter().position(|it| it == id))
                    .unwrap_or(0)
            };
            result
                .rows
                .sort_by_key(|row| std::cmp::Reverse(position(row)));
        }
        Ok(result)
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }

    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

/// The CHANGELOG: the authored scenario "could not be decided" until the caller stated the
/// instant. Every `issued_at` the committed suite sends rises with the order the invoices are
/// issued in (synthesized: 2020-01-01, -02, -03, -04; authored: 09:00:01 then 09:00:03), so ranking
/// by issue sequence and ranking by `issued_at` give the same rows, and the suite cannot tell them
/// apart.
#[test]
fn committed_suite_fails_a_target_that_ranks_outstanding_by_issue_sequence() {
    let suite = ConformanceSuite::from_json(&committed_suite_text()).expect("parses");
    let target = RanksByIssueSequence {
        inner: Billing::new(),
        issued: RefCell::default(),
    };
    let report = Runner::for_suite(&suite)
        .run(&suite, &target)
        .expect("admitted");
    assert_ne!(
        report.status,
        ConformanceStatus::Passed,
        "a target ranking OutstandingInvoices by issue sequence, ignoring `issued_at`, passes all \
         {} scenarios of the committed suite",
        report.scenarios.len()
    );
}
