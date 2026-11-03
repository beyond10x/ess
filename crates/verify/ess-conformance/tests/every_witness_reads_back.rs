//! Every witness reads the addressed row back and requires exactly its outcome's effect
//! (`story:every-witness-reads-back-its-outcome-effect`; beyond10x/ess#581, #585, #587).
//!
//! The reproduction: `Redeem` refuses a wrong proof by a compound `when_subject:` guard, answers
//! the `external:` `store-unavailable` before it reads the row, answers `no-ticket` for an identity
//! no row carries and otherwise deletes the ticket. From `ess/34` a second `external:` refusal,
//! `consumed-on-failure`, compensates by deleting the ticket before it answers its error.
//!
//! * #581: every witness of the deleting branch, the further ones synthesis adds for the guard's
//!   complement included, expects the redeemed ticket absent after the delete, never present.
//! * #587 (shipped in `ess/26`) and the input-guarded and related-row siblings (`ess/34`): a
//!   refusal of a command addressing an existing row snapshots the complete row before the send and
//!   requires it unchanged after.
//! * #585: a compensating refusal may delete its addressed row from `ess/34`, read back as absent;
//!   below it the model is refused naming the format.
//!
//! Every model is inline, so it is no repository model the suite-bytes tables walk.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError, ViewRow,
};
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, Runner, ScenarioId, ScenarioStep, ScenarioValue,
    ViewExpectation,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

/// The reproduction, `{format}` to fill, `{compensating}` the `consumed-on-failure` branch or
/// nothing.
const TICKETS: &str = r#"format: {format}
system: demo
version: v1
domain: demo.tickets
summary: Tickets issued with a secret and redeemed once with a matching proof.
types:
  - {name: demo.tickets.TicketId, kind: newtype, of: Uuid}
entities:
  - name: demo.tickets.Ticket
    identity: {name: ticket_id, type: demo.tickets.TicketId}
    fields:
      - {name: secret, type: String}
    lifecycle: {initial: Issued, states: [Issued], terminal: [Issued]}
errors:
  - {name: demo.tickets.StoreUnavailable, summary: The store could not be read.}
  - {name: demo.tickets.ConsumedOnFailure, summary: The ticket was consumed and the grant failed.}
  - {name: demo.tickets.NoTicket, summary: No ticket carries the identity.}
  - {name: demo.tickets.WrongProof, summary: The proof is not the ticket's secret.}
commands:
  - name: demo.tickets.Issue
    input:
      - {name: secret, type: String}
    outcomes:
      - name: issued
        creates: demo.tickets.Ticket
        instance: ticket_id
        sets: {secret: input.secret}
        emits: [demo.tickets.Issued]
        payload:
          demo.tickets.Issued: {ticket_id: {generated: true}}
  - name: demo.tickets.Redeem
    input:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: proof, type: String}
    outcomes:
      - name: store-unavailable
        external: the store read of the ticket fails
        before_row: true
        error: demo.tickets.StoreUnavailable
      - name: no-ticket
        unknown_instance: true
        error: demo.tickets.NoTicket
      - name: wrong-proof
        when_subject: {predicate: {all: ['secret != ""', 'secret != input.proof']}}
        error: demo.tickets.WrongProof
{compensating}      - name: redeemed
        deletes: demo.tickets.Ticket
        instance: ticket_id
        emits: [demo.tickets.Redeemed]
        payload:
          demo.tickets.Redeemed: {ticket_id: input.ticket_id}
events:
  - name: demo.tickets.Issued
    fields: [{name: ticket_id, type: demo.tickets.TicketId}]
  - name: demo.tickets.Redeemed
    fields: [{name: ticket_id, type: demo.tickets.TicketId}]
views:
  - name: demo.tickets.Tickets
    source: demo.tickets.Ticket
    consistency: read_your_writes
    fields:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: secret, type: String}
      - {name: state, type: demo.tickets.Ticket.State}
"#;

/// The compensating delete of #585.
const COMPENSATING: &str = "      - name: consumed-on-failure
        external: the grant fails after the ticket is consumed
        compensates: true
        deletes: demo.tickets.Ticket
        instance: ticket_id
        error: demo.tickets.ConsumedOnFailure
";

const REDEEM: &str = "demo.tickets.Redeem";
const REDEEMED: &str = "demo.tickets.Redeem/outcome/redeemed";
const STORE_UNAVAILABLE: &str = "demo.tickets.Redeem/outcome/store-unavailable";
const CONSUMED: &str = "demo.tickets.Redeem/outcome/consumed-on-failure";

fn tickets(format: &str, compensating: bool) -> String {
    TICKETS.replace("{format}", format).replace(
        "{compensating}",
        if compensating { COMPENSATING } else { "" },
    )
}

fn parsed(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("tickets.yaml"), raw)])
        .map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = parsed(text).unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the model resolves:\n{d}"))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenario(&ScenarioId::parse(id).expect("an id"))
        .unwrap_or_else(|| {
            panic!(
                "`{id}` is synthesized; refusals: {:#?}",
                result
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
}

/// What a planted defect does to a ticket a `Redeem` send took `outcome` for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    /// Nothing: the model interpreter as it is.
    None,
    /// The ticket stays listed, as last read, after the branch deleted it.
    Keeps(&'static str),
    /// The ticket stops being listed after the branch answered, as if it were deleted.
    Deletes(&'static str),
}

/// The model interpreter with [`Fault`] applied to the views' rows.
struct Target {
    inner: Interpreted,
    fault: Fault,
    seen: RefCell<BTreeMap<Node, ViewRow>>,
    touched: RefCell<BTreeSet<Node>>,
}

impl Target {
    fn new(model: &str, fault: Fault) -> Self {
        Self {
            inner: Interpreted::for_model(ir(model)),
            fault,
            seen: RefCell::new(BTreeMap::new()),
            touched: RefCell::new(BTreeSet::new()),
        }
    }
}

impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.seen.borrow_mut().clear();
        self.touched.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let ticket = request.input.get("ticket_id").cloned();
        let result = self.inner.execute_command(request)?;
        let taken = result
            .outcome
            .as_ref()
            .map(|taken| taken.outcome.as_str().to_owned());
        let faulted = match self.fault {
            Fault::None => None,
            Fault::Keeps(outcome) | Fault::Deletes(outcome) => Some(outcome),
        };
        if let (Some(ticket), Some(taken), Some(faulted)) = (ticket, taken, faulted) {
            if taken == faulted {
                self.touched.borrow_mut().insert(ticket);
            }
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        let touched = self.touched.borrow();
        match self.fault {
            Fault::None => {}
            Fault::Deletes(_) => result.rows.retain(|row| {
                !row.get("ticket_id")
                    .is_some_and(|ticket| touched.contains(ticket))
            }),
            Fault::Keeps(_) => {
                let listed: BTreeSet<Node> = result
                    .rows
                    .iter()
                    .filter_map(|row| row.get("ticket_id").cloned())
                    .collect();
                for ticket in touched.iter().filter(|ticket| !listed.contains(*ticket)) {
                    if let Some(row) = self.seen.borrow().get(ticket) {
                        result.rows.push(row.clone());
                    }
                }
            }
        }
        let mut seen = self.seen.borrow_mut();
        for row in &result.rows {
            if let Some(ticket) = row.get("ticket_id") {
                seen.insert(ticket.clone(), row.clone());
            }
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

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// Each instance a `Redeem` send of `scenario` took `outcome` for, with the index of the send.
fn taken_for(scenario: &ConformanceScenario, outcome: &str) -> Vec<(usize, String)> {
    scenario
        .steps
        .windows(2)
        .enumerate()
        .filter_map(|(at, pair)| match pair {
            [ScenarioStep::ExecuteCommand { command, input, .. }, ScenarioStep::ExpectOutcome { outcome: taken }]
                if command.to_string() == REDEEM && taken.outcome.as_str() == outcome =>
            {
                match input.get("ticket_id") {
                    Some(ScenarioValue::Instance { instance }) => {
                        Some((at, instance.as_str().to_owned()))
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

/// Whether `step` requires a view row naming `instance`.
fn requires_present(step: &ScenarioStep, instance: &str) -> bool {
    let (ScenarioStep::ExpectView {
        expectation: ViewExpectation::Contains { fields },
        ..
    }
    | ScenarioStep::EventuallyView {
        expectation: ViewExpectation::Contains { fields },
        ..
    }) = step
    else {
        return false;
    };
    fields.values().any(|value| {
        matches!(value, ScenarioValue::Instance { instance: named } if named.as_str() == instance)
    })
}

/// #581.
#[test]
fn extra_witness_of_a_deleting_outcome_expects_the_row_absent() {
    for format in ["ess/25", "ess/33"] {
        let model = tickets(format, false);
        let result = synthesize(&ir(&model));
        let redeemed = scenario(&result, REDEEMED);
        let deleted = taken_for(redeemed, "redeemed");
        assert!(
            !deleted.is_empty(),
            "{format}: `{REDEEMED}` redeems an issued ticket: {:#?}",
            redeemed.steps
        );
        for (at, instance) in &deleted {
            assert!(
                !redeemed.steps[at + 2..]
                    .iter()
                    .any(|step| requires_present(step, instance)),
                "{format}: `{REDEEMED}` deletes `{instance}` at step {at} and later requires it \
                 present: {:#?}",
                redeemed.steps
            );
        }
        let honest = run(&result.suite, &Target::new(&model, Fault::None));
        assert_eq!(not_passed(&honest), Vec::<String>::new(), "{format}");
        let keeps = run(
            &result.suite,
            &Target::new(&model, Fault::Keeps("redeemed")),
        );
        assert_ne!(
            keeps.get(REDEEMED),
            Some(&Status::Passed),
            "{format}: a target that keeps the redeemed ticket fails `{REDEEMED}`"
        );
    }
}

/// #587 on the reproduction: the forced `before_row` refusal reads the ticket back unchanged.
#[test]
fn external_refusal_requires_the_row_unchanged() {
    let model = tickets("ess/34", true);
    let result = synthesize(&ir(&model));
    let forced = scenario(&result, STORE_UNAVAILABLE);
    assert!(
        forced
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::SnapshotCompleteSubject { .. }))
            && forced
                .steps
                .iter()
                .any(|step| matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. })),
        "`{STORE_UNAVAILABLE}` snapshots the ticket and requires it unchanged: {:#?}",
        forced.steps
    );
    let deletes = run(
        &result.suite,
        &Target::new(&model, Fault::Deletes("store-unavailable")),
    );
    assert_ne!(deletes.get(STORE_UNAVAILABLE), Some(&Status::Passed));
}

/// #585: the compensating delete validates at `ess/34`, its scenario forces it, requires the
/// error and no event, then the ticket absent; a target that keeps the ticket fails it.
#[test]
fn compensating_refusal_deletes_its_row() {
    let model = tickets("ess/34", true);
    let result = synthesize(&ir(&model));
    let consumed = scenario(&result, CONSUMED);
    let steps = &consumed.steps;
    let forced = steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { .. }))
        .unwrap_or_else(|| panic!("`{CONSUMED}` forces its branch: {steps:#?}"));
    let rest = &steps[forced..];
    assert!(
        rest.iter()
            .any(|step| matches!(step, ScenarioStep::ExpectError { .. })),
        "`{CONSUMED}` requires its error: {steps:#?}"
    );
    assert!(
        rest.iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectNoEvent { .. } | ScenarioStep::ExpectNoEvents
        )),
        "`{CONSUMED}` requires no event: {steps:#?}"
    );
    assert!(
        rest.iter()
            .any(|step| matches!(step, ScenarioStep::ExpectSubjectAbsent { .. })),
        "`{CONSUMED}` requires the ticket absent from an immediate view: {steps:#?}"
    );
    let honest = run(&result.suite, &Target::new(&model, Fault::None));
    assert_eq!(not_passed(&honest), Vec::<String>::new());
    let keeps = run(
        &result.suite,
        &Target::new(&model, Fault::Keeps("consumed-on-failure")),
    );
    assert_ne!(
        keeps.get(CONSUMED),
        Some(&Status::Passed),
        "a target that keeps the consumed ticket fails `{CONSUMED}`: {keeps:#?}"
    );
    let deletes = run(
        &result.suite,
        &Target::new(&model, Fault::Deletes("store-unavailable")),
    );
    assert_ne!(
        deletes.get(STORE_UNAVAILABLE),
        Some(&Status::Passed),
        "a target that deletes the ticket on `store-unavailable` fails it: {deletes:#?}"
    );
}

/// A card read through `when_related:` beside an input-guarded refusal, both on a command that
/// addresses an existing ticket.
const GUARDED: &str = r#"format: {format}
system: demo
version: v1
domain: demo.passes
types:
  - {name: demo.passes.PassId, kind: newtype, of: String}
  - {name: demo.passes.CardId, kind: newtype, of: String}
entities:
  - name: demo.passes.Card
    identity: {name: card_id, type: demo.passes.CardId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.passes.Pass
    identity: {name: pass_id, type: demo.passes.PassId}
    fields:
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.passes.EmptyNote, summary: The note is empty.}
  - {name: demo.passes.NoCard, summary: No card carries the identity.}
commands:
  - name: demo.passes.AddCard
    input:
      - {name: card_id, type: demo.passes.CardId}
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: demo.passes.Card
        instance: card_id
        sets: {label: input.label}
        emits: [demo.passes.CardAdded]
        payload:
          demo.passes.CardAdded: {card_id: input.card_id}
  - name: demo.passes.OpenPass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassOpened]
        payload:
          demo.passes.PassOpened: {pass_id: input.pass_id}
  - name: demo.passes.Annotate
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: card_id, type: demo.passes.CardId}
      - {name: note, type: String}
    outcomes:
      - name: empty-note
        when: 'note == ""'
        error: demo.passes.EmptyNote
      - name: no-card
        when_related: {via: input.card_id, exists: false}
        error: demo.passes.NoCard
      - name: annotated
        updates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassAnnotated]
        payload:
          demo.passes.PassAnnotated: {pass_id: input.pass_id}
events:
  - name: demo.passes.CardAdded
    fields: [{name: card_id, type: demo.passes.CardId}]
  - name: demo.passes.PassOpened
    fields: [{name: pass_id, type: demo.passes.PassId}]
  - name: demo.passes.PassAnnotated
    fields: [{name: pass_id, type: demo.passes.PassId}]
views:
  - name: demo.passes.Passes
    source: demo.passes.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
      - {name: state, type: demo.passes.Pass.State}
"#;

/// Whether `scenario` snapshots the complete subject before some send of `command` taking
/// `outcome` and requires it unchanged after.
fn reads_complete(scenario: &ConformanceScenario, command: &str, outcome: &str) -> bool {
    let steps = &scenario.steps;
    steps.windows(2).enumerate().any(|(at, pair)| match pair {
        [ScenarioStep::ExecuteCommand { command: sent, .. }, ScenarioStep::ExpectOutcome { outcome: taken }]
            if sent.to_string() == command && taken.outcome.as_str() == outcome =>
        {
            steps[..at]
                .iter()
                .any(|step| matches!(step, ScenarioStep::SnapshotCompleteSubject { .. }))
                && steps[at..]
                    .iter()
                    .any(|step| matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. }))
        }
        _ => false,
    })
}

/// The input-guarded and related-row siblings of #587, from `ess/34`.
#[test]
fn input_and_related_refusals_require_the_row_unchanged() {
    let model = GUARDED.replace("{format}", "ess/34");
    let result = synthesize(&ir(&model));
    for outcome in ["empty-note", "no-card"] {
        let id = format!("demo.passes.Annotate/outcome/{outcome}");
        let witness = scenario(&result, &id);
        assert!(
            reads_complete(witness, "demo.passes.Annotate", outcome),
            "`{id}` snapshots the addressed pass completely and requires it unchanged: {:#?}",
            witness.steps
        );
    }
    let honest = run(&result.suite, &Interpreted::for_model(ir(&model)));
    assert_eq!(not_passed(&honest), Vec::<String>::new());
}

/// #585 below the format: refused `refusal_mutated_state`, naming `ess/34`.
#[test]
fn compensating_delete_refused_below_the_format() {
    let errors = parsed(&tickets("ess/33", true))
        .expect_err("a compensating delete is refused below ess/34");
    assert!(
        errors.contains("refusal_mutated_state") || errors.contains("ESS-COMMAND"),
        "{errors}"
    );
    assert!(
        errors.contains("ess/34"),
        "the refusal names the format: {errors}"
    );
    parsed(&tickets("ess/34", true)).unwrap_or_else(|errors| panic!("{errors}"));
}
