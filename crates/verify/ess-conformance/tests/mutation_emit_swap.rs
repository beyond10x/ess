//! `emit-swap`: single-event substitution, and the sites where none is admissible
//! (beyond10x/ess#295).
//!
//! `docs/design/mutation-scope-and-known-failures.md`, "Single-event mutation: emit-swap". An
//! outcome that emits exactly one event and names no error has no `emit-drop` mutant that compiles:
//! dropping its only event leaves a branch that neither emits nor refuses (`ESS-COMMAND-007`). An
//! `emit-swap` mutant replaces that event with another declared one of exactly the same resolved
//! fields, published by every component that accepts the command, and renames the outcome's
//! `payload:` key; the first such candidate in byte order of name that compiles is the mutant.
//! Where none exists the site is an unavailable site, `no_compatible_event_alternative`: neither a
//! kill nor stillborn, and it keeps the audit from succeeding.
//!
//! `tests/fixtures/emit-swap.yaml` holds the rejected kinds of candidate. For `Place/placed`, which
//! emits `Placed { order_id: OrderId }` and is accepted by both `alpha` and `beta`: `Booked` has an
//! `Optional<OrderId>` field and compiles in its place; `Cancelled` and `Filed` (which is `placed`
//! on the wire, as `Placed` is) carry another field; `Logged` has exactly the fields and compiles,
//! but `beta` does not publish it; `Noted` is the swap. `Cancel/cancelled` has no alternative.
//! `tests/fixtures/emit-swap-served.yaml` has a creating, an updating and a moving swap, with an
//! `Optional` lookalike ahead of the creating and updating one in byte order, and one creating site
//! with no alternative. Every target here is real: the billing reference, and the model interpreter
//! for the two fixtures.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, MutantClass, MutateCode, Mutation, MutationReport, UnavailableReason,
    UnavailableSite, Verdict, BASELINE_DIR, MANIFEST_FILE, MANIFEST_FORMAT, MANIFEST_FORMAT_4,
    REPORT_FILE, REPORT_FORMAT, REPORT_FORMAT_4, SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_conformance::report::Status;
use ess_conformance::runner::Runner;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::{json, Value};

const SWAP: &[MutantClass] = &[MutantClass::EmitSwap];

const PLACE: &str = "emit-swap/swaps.order.Place/placed/swaps.order.Placed/swaps.order.Noted";
const CANCEL: &str = "emit-swap/swaps.order.Cancel/cancelled/swaps.order.Cancelled";
const ISSUE: &str = "emit-swap/billing.invoice.IssueInvoice/issued/billing.invoice.InvoiceIssued/\
                     billing.invoice.InvoiceCancelled";
const CANCEL_INVOICE: &str = "emit-swap/billing.invoice.CancelInvoice/cancelled/\
                              billing.invoice.InvoiceCancelled/billing.invoice.InvoiceIssued";
/// The billing sites with no alternative: the three events whose fields no other event repeats.
const BILLING_UNAVAILABLE: [&str; 3] = [
    "emit-swap/billing.email.SendEmail/sent/billing.email.EmailSent",
    "emit-swap/billing.invoice.CreateInvoice/accepted/billing.invoice.InvoiceCreated",
    "emit-swap/billing.invoice.PayInvoice/settled/billing.invoice.InvoicePaid",
];
const CREATED: &str =
    "emit-swap/demo.items.PutItem/created/demo.items.ItemStored/demo.items.ItemRestored";
const UPDATED: &str =
    "emit-swap/demo.items.PutItem/updated/demo.items.ItemStored/demo.items.ItemRestored";
const ARCHIVED: &str =
    "emit-swap/demo.items.ArchiveItem/archived/demo.items.ItemArchived/demo.items.ItemReviewed";
const BOOKED: &str = "emit-swap/demo.items.BookSlot/booked/demo.items.SlotBooked";

// ---- the specifications -------------------------------------------------------------------------

fn documents(base: &Path) -> (Vec<Document>, SourceMap) {
    let mut found: Vec<PathBuf> = Vec::new();
    if base.is_file() {
        found.push(base.to_path_buf());
    } else {
        let mut pending = vec![base.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).expect("readable") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().is_some_and(|it| it == "yaml") {
                    found.push(path);
                }
            }
        }
    }
    found.sort();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{label}: {error}"));
        texts.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    (parsed, texts)
}

fn fixture(name: &str) -> (Vec<Document>, SourceMap) {
    documents(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
}

fn swaps() -> (Vec<Document>, SourceMap) {
    fixture("emit-swap.yaml")
}

fn served() -> (Vec<Document>, SourceMap) {
    fixture("emit-swap-served.yaml")
}

fn billing() -> (Vec<Document>, SourceMap) {
    documents(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../examples/billing")
            .canonicalize()
            .expect("billing exists"),
    )
}

/// Billing with no component declared: nothing says who publishes what.
fn billing_without_components() -> (Vec<Document>, SourceMap) {
    let (mut files, texts) = billing();
    for (_, file) in &mut files {
        file.components.clear();
        file.topology = None;
    }
    (files, texts)
}

fn interpreted((files, texts): &(Vec<Document>, SourceMap)) -> impl Fn() -> Interpreted {
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    move || Interpreted::for_model(ir.clone())
}

fn ids(found: &[mutate::Mutant]) -> Vec<&str> {
    found.iter().map(|mutant| mutant.id.as_str()).collect()
}

fn unavailable_ids(sites: &[UnavailableSite]) -> Vec<&str> {
    sites.iter().map(|site| site.id.as_str()).collect()
}

fn site<'a>(report: &'a MutationReport, id: &str) -> &'a mutate::MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("`{id}` is in the report:\n{}", report.render_text()))
}

/// The suite `documents` with `mutation` applied obliges, as the audit synthesizes it.
fn mutant_suite(
    documents: &[Document],
    texts: &SourceMap,
    mutation: &Mutation,
) -> ConformanceSuite {
    let mutated = mutate::apply(documents, mutation).expect("the site exists");
    let ir = mutate::compile(mutated, texts).expect("the mutant compiles");
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite
        .scenarios
        .retain(|id, _| !matches!(id, ess_conformance::ScenarioId::Grant { .. }));
    suite.select_fresh_format_for(&ir);
    suite
}

fn statuses(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).expect("admitted");
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .scenarios
        .iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

// ---- candidates ---------------------------------------------------------------------------------

#[test]
fn the_swap_is_the_first_compiling_event_of_the_same_fields_every_accepting_component_publishes() {
    let (files, _) = swaps();
    let selection = mutate::selection(&files, SWAP);
    assert_eq!(ids(&selection.mutants), [PLACE]);
    let mutant = &selection.mutants[0];
    assert_eq!(mutant.class, MutantClass::EmitSwap);
    assert_eq!(
        mutant.mutation,
        Mutation::EmitSwap {
            command: "swaps.order.Place".to_owned(),
            outcome: "placed".to_owned(),
            event: "swaps.order.Placed".to_owned(),
            to: "swaps.order.Noted".to_owned(),
        }
    );
    assert_eq!(
        mutant.change,
        "`emits: [swaps.order.Placed]` becomes `emits: [swaps.order.Noted]`"
    );
    assert_eq!(unavailable_ids(&selection.unavailable), [CANCEL]);
    let cancel = &selection.unavailable[0];
    assert_eq!(
        cancel.reason,
        UnavailableReason::NoCompatibleEventAlternative
    );
    assert_eq!(cancel.class, MutantClass::EmitSwap);
    assert_eq!(cancel.command, "swaps.order.Cancel");
    assert_eq!(cancel.outcome, "cancelled");
    assert_eq!(cancel.event, "swaps.order.Cancelled");
    assert!(
        cancel
            .unaudited
            .starts_with("single-event substitution was not audited here"),
        "{}",
        cancel.unaudited
    );
    // `mutants` is the selection's mutants; the same tree selects the same bytes.
    assert_eq!(mutate::mutants(&files, SWAP), selection.mutants);
    assert_eq!(mutate::selection(&files, SWAP), selection);
}

/// `Booked` (an `Optional` field) and `Logged` (published by `alpha` alone) come before `Noted`
/// and compile in its place, so only the candidate rules keep them out; `Filed` shares the wire
/// label and not the fields.
#[test]
fn a_wrong_field_type_a_shared_wire_label_or_a_missing_publisher_is_not_an_alternative() {
    let (files, texts) = swaps();
    let swap = |to: &str| Mutation::EmitSwap {
        command: "swaps.order.Place".to_owned(),
        outcome: "placed".to_owned(),
        event: "swaps.order.Placed".to_owned(),
        to: to.to_owned(),
    };
    for compiles in ["swaps.order.Booked", "swaps.order.Logged"] {
        let mutated = mutate::apply(&files, &swap(compiles)).expect("an edit of the site");
        mutate::compile(mutated, &texts)
            .unwrap_or_else(|refused| panic!("{compiles} compiles in its place: {refused:?}"));
    }
    let mutated = mutate::apply(&files, &swap("swaps.order.Filed")).unwrap();
    assert!(mutate::compile(mutated, &texts).is_err());
    let chosen: Vec<String> = mutate::mutants(&files, SWAP)
        .into_iter()
        .filter_map(|mutant| match mutant.mutation {
            Mutation::EmitSwap { to, .. } => Some(to),
            _ => None,
        })
        .collect();
    assert_eq!(chosen, ["swaps.order.Noted"]);
}

#[test]
fn the_swap_renames_the_payload_key_and_leaves_the_baseline_documents_alone() {
    let (files, _) = swaps();
    let before = format!("{files:?}");
    let mutant = mutate::mutants(&files, SWAP).remove(0);
    assert_eq!(
        format!("{files:?}"),
        before,
        "enumeration edits no document"
    );
    let mutated = mutate::apply(&files, &mutant.mutation).unwrap();
    assert_eq!(format!("{files:?}"), before, "applying edits a copy");
    let outcome = mutated
        .iter()
        .flat_map(|(_, file)| &file.commands)
        .find(|command| command.name.to_string() == "swaps.order.Place")
        .and_then(|command| command.outcomes.first())
        .unwrap();
    let emitted: Vec<String> = outcome.emits.iter().map(ToString::to_string).collect();
    assert_eq!(emitted, ["swaps.order.Noted"]);
    let keys: Vec<String> = outcome
        .payload
        .0
        .iter()
        .map(|(event, _)| event.to_string())
        .collect();
    assert_eq!(keys, ["swaps.order.Noted"]);
    let original = files
        .iter()
        .flat_map(|(_, file)| &file.commands)
        .find(|command| command.name.to_string() == "swaps.order.Place")
        .and_then(|command| command.outcomes.first())
        .unwrap();
    assert_eq!(
        format!("{:?}", outcome.payload.0[0].1),
        format!("{:?}", original.payload.0[0].1),
        "the field expressions are kept"
    );
    // No event is declared, and none removed.
    let events = |documents: &[Document]| -> Vec<String> {
        documents
            .iter()
            .flat_map(|(_, file)| &file.events)
            .map(|event| event.name.to_string())
            .collect()
    };
    assert_eq!(events(&mutated), events(&files));
}

#[test]
fn creating_updating_and_moving_outcomes_are_swap_sites() {
    let (files, _) = served();
    let selection = mutate::selection(&files, SWAP);
    assert_eq!(ids(&selection.mutants), [ARCHIVED, CREATED, UPDATED]);
    assert_eq!(unavailable_ids(&selection.unavailable), [BOOKED]);
    let (files, _) = billing();
    let selection = mutate::selection(&files, SWAP);
    assert_eq!(ids(&selection.mutants), [CANCEL_INVOICE, ISSUE]);
    assert_eq!(unavailable_ids(&selection.unavailable), BILLING_UNAVAILABLE);
}

#[test]
fn a_model_without_components_has_no_publisher_and_so_no_alternative() {
    let (files, texts) = billing_without_components();
    mutate::compile(files.clone(), &texts).expect("billing compiles without components");
    let selection = mutate::selection(&files, SWAP);
    assert_eq!(ids(&selection.mutants), Vec::<&str>::new());
    let mut every = BILLING_UNAVAILABLE.to_vec();
    every.extend([
        "emit-swap/billing.invoice.CancelInvoice/cancelled/billing.invoice.InvoiceCancelled",
        "emit-swap/billing.invoice.IssueInvoice/issued/billing.invoice.InvoiceIssued",
    ]);
    every.sort_unstable();
    assert_eq!(unavailable_ids(&selection.unavailable), every);
    // Still an audit, never ESS-MUTATE-003: the sites were selected and are reported.
    let report = mutate::audit(&files, &texts, SWAP, Billing::new).expect("an audit");
    assert_eq!(report.counts.mutants, 0);
    assert_eq!(report.format, REPORT_FORMAT_4);
    assert_eq!(
        unavailable_ids(report.unavailable_sites.as_deref().unwrap()),
        every
    );
}

/// An outcome emitting two events keeps its two `emit-drop` mutants and is no swap site.
#[test]
fn a_multi_event_outcome_keeps_emit_drop_and_is_no_swap_site() {
    let (mut files, texts) = swaps();
    let place = files
        .iter_mut()
        .flat_map(|(_, file)| &mut file.commands)
        .find(|command| command.name.to_string() == "swaps.order.Place")
        .unwrap();
    let outcome = &mut place.outcomes[0];
    let noted = ess_domain::name::QualifiedName::new("swaps.order.Noted").unwrap();
    outcome.emits.push(noted.clone());
    let table = outcome.payload.0[0].1.clone();
    outcome.payload.0.push((noted, table));
    mutate::compile(files.clone(), &texts).expect("two events compile");
    let selection = mutate::selection(&files, &[MutantClass::EmitDrop, MutantClass::EmitSwap]);
    assert_eq!(
        ids(&selection.mutants),
        [
            "emit-drop/swaps.order.Cancel/cancelled/swaps.order.Cancelled",
            "emit-drop/swaps.order.Place/placed/swaps.order.Noted",
            "emit-drop/swaps.order.Place/placed/swaps.order.Placed",
        ]
    );
    assert_eq!(unavailable_ids(&selection.unavailable), [CANCEL]);
    let report = mutate::audit(
        &files,
        &texts,
        &[MutantClass::EmitDrop],
        interpreted(&(files.clone(), texts.clone())),
    )
    .expect("an audit");
    assert_eq!(
        report.format, REPORT_FORMAT,
        "emit-drop alone writes what it wrote"
    );
    assert_eq!(report.unavailable_sites, None);
    for id in [
        "emit-drop/swaps.order.Place/placed/swaps.order.Noted",
        "emit-drop/swaps.order.Place/placed/swaps.order.Placed",
    ] {
        assert_eq!(site(&report, id).verdict, Verdict::Killed, "{id}");
    }
}

// ---- the direct audit ---------------------------------------------------------------------------

#[test]
fn the_billing_reference_kills_both_moving_swaps_through_the_event_it_published() {
    let (files, texts) = billing();
    let report = mutate::audit(&files, &texts, SWAP, Billing::new).expect("an audit");
    assert_eq!(report.format, REPORT_FORMAT_4);
    assert_eq!(report.counts.mutants, 2);
    assert_eq!(report.counts.killed, 2, "{}", report.render_text());
    assert_eq!(report.counts.stillborn, 0);
    for (id, killer) in [
        (ISSUE, "billing.invoice.IssueInvoice/outcome/issued"),
        (
            CANCEL_INVOICE,
            "billing.invoice.CancelInvoice/outcome/cancelled",
        ),
    ] {
        let entry = site(&report, id);
        assert!(
            entry.killers.iter().flatten().any(|it| it == killer),
            "{id}: {:?}",
            entry.killers
        );
    }
    let unavailable = report.unavailable_sites.as_deref().expect("listed");
    assert_eq!(unavailable_ids(unavailable), BILLING_UNAVAILABLE);
    assert!(unavailable
        .iter()
        .all(|site| site.reason == UnavailableReason::NoCompatibleEventAlternative));
    let text = report.render_text();
    for id in BILLING_UNAVAILABLE {
        let line = text
            .lines()
            .find(|line| line.starts_with(&format!("unavailable {id}:")))
            .unwrap_or_else(|| panic!("`{id}` has a line:\n{text}"));
        assert!(
            line.contains("no_compatible_event_alternative")
                && line.contains("single-event substitution was not audited here"),
            "{line}"
        );
    }
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(json["unavailable_sites"].as_array().unwrap().len(), 3);
    assert_eq!(
        json["unavailable_sites"][0]["reason"],
        "no_compatible_event_alternative"
    );
    assert!(json["unavailable_sites"][0]["unaudited"]
        .as_str()
        .unwrap()
        .starts_with("single-event substitution was not audited here"));
}

/// The kill is the event observation's: the mutant's own `issued` scenario fails at the event the
/// reference did not publish, and every other scenario of its suite passes.
#[test]
fn a_swap_is_killed_by_the_event_expectation_and_nothing_else() {
    let (files, texts) = billing();
    let mutant = mutate::mutants(&files, SWAP)
        .into_iter()
        .find(|mutant| mutant.id == ISSUE)
        .unwrap();
    let suite = mutant_suite(&files, &texts, &mutant.mutation);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let run = Runner::for_suite(&suite).run_admitted(&admitted, &Billing::new());
    let failed: Vec<&ess_conformance::report::ScenarioResult> = run
        .scenarios
        .iter()
        .filter(|result| result.status == Status::Failed)
        .collect();
    assert_ne!(
        failed.len(),
        0,
        "the swap fails a scenario of its own suite"
    );
    for result in &failed {
        let rendered: Vec<String> = result
            .diagnostics()
            .map(|diagnostic| format!("{diagnostic:?}"))
            .collect();
        assert!(
            rendered
                .iter()
                .any(|it| it.contains("InvoiceCancelled") || it.contains("InvoiceIssued")),
            "{}: {rendered:#?}",
            result.scenario
        );
    }
    // The same suite with its event expectations taken out passes against the same target.
    let stripped = without_event_expectations(&suite);
    let passed = statuses(&stripped, &Billing::new());
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "{passed:?}"
    );
}

#[test]
fn creating_updating_and_moving_swaps_are_killed_by_a_healthy_target() {
    let spec = served();
    let report = mutate::audit(&spec.0, &spec.1, SWAP, interpreted(&spec)).expect("an audit");
    assert_eq!(report.counts.mutants, 3);
    assert_eq!(report.counts.killed, 3, "{}", report.render_text());
    for (id, killer) in [
        (CREATED, "demo.items.PutItem/outcome/created"),
        (UPDATED, "demo.items.PutItem/outcome/updated"),
        (ARCHIVED, "demo.items.ArchiveItem/outcome/archived"),
    ] {
        let entry = site(&report, id);
        assert!(
            entry.killers.iter().flatten().any(|it| it == killer),
            "{id}: {:?}",
            entry.killers
        );
    }
    assert_eq!(
        unavailable_ids(report.unavailable_sites.as_deref().unwrap()),
        [BOOKED]
    );
}

#[test]
fn an_unavailable_site_is_neither_killed_nor_stillborn_and_is_listed_after_the_baseline() {
    let spec = swaps();
    let report = mutate::audit(&spec.0, &spec.1, SWAP, interpreted(&spec)).expect("an audit");
    assert_eq!(report.counts.mutants, 1);
    assert_eq!(report.counts.killed, 1);
    assert_eq!(report.counts.stillborn, 0);
    assert_eq!(site(&report, PLACE).verdict, Verdict::Killed);
    assert!(report.mutants.iter().all(|entry| entry.id != CANCEL));
    let sites = report.unavailable_sites.as_deref().unwrap();
    assert_eq!(unavailable_ids(sites), [CANCEL]);
    let text = report.render_text();
    let first = text.lines().next().unwrap();
    assert!(first.ends_with("; 1 unavailable"), "{first}");
    assert!(text
        .lines()
        .nth(1)
        .unwrap()
        .starts_with(&format!("unavailable {CANCEL}:")));
    // `ESS-MUTATE-003` is for classes that find nothing at all.
    assert_eq!(MutateCode::NoSite.code().to_string(), "ESS-MUTATE-003");
}

// ---- a target that emits the substituted event, and a runner that ignores events ------------------

/// `inner`, except that `command` publishes `to` where `inner` publishes `from`: the target the
/// swap mutant describes.
struct Renaming<T> {
    inner: T,
    command: &'static str,
    from: &'static str,
    to: &'static str,
}

impl<T: ConformanceTarget> Renaming<T> {
    fn rename(&self, event: &mut ObservedEvent) {
        if event.event.to_string() == self.from {
            event.event = self.to.parse().unwrap();
        }
    }
}

impl<T: ConformanceTarget> ConformanceTarget for Renaming<T> {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let ours = request.command.to_string() == self.command;
        let mut result = self.inner.execute_command(request)?;
        if ours {
            for event in &mut result.direct_events {
                self.rename(event);
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
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let asked = request.event.to_string();
        if asked == self.from {
            return Ok(Vec::new());
        }
        if asked == self.to {
            let mut renamed = request.clone();
            renamed.event = self.from.parse().unwrap();
            let mut seen = self.inner.observe_events(renamed)?;
            for event in &mut seen {
                self.rename(event);
            }
            let mut own = self.inner.observe_events(request)?;
            own.extend(seen);
            return Ok(own);
        }
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(request, times)
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
    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.inner.mark_instant(request)
    }
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        self.inner.observe_elapsed(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

fn wrongly_issuing() -> Renaming<Billing> {
    Renaming {
        inner: Billing::new(),
        command: "billing.invoice.IssueInvoice",
        from: "billing.invoice.InvoiceIssued",
        to: "billing.invoice.InvoiceCancelled",
    }
}

/// `suite` with every event expectation taken out, as a runner that discards them would run it.
fn without_event_expectations(suite: &ConformanceSuite) -> ConformanceSuite {
    let mut stripped = suite.clone();
    for scenario in stripped.scenarios.values_mut() {
        scenario.steps.retain(|step| {
            !matches!(
                step,
                ScenarioStep::ExpectEvent { .. }
                    | ScenarioStep::ExpectEventValues { .. }
                    | ScenarioStep::EventuallyEvent { .. }
                    | ScenarioStep::ExpectNoEvent { .. }
                    | ScenarioStep::ExpectNoEvents
            )
        });
    }
    stripped
}

/// The report/2 the Rust runner writes for `target` over the suite text `suite`.
fn report_of(suite: &str, target: &impl ConformanceTarget) -> String {
    let admitted = AdmittedSuite::from_json(suite).expect("an emitted suite is admitted");
    let executed = Runner::for_suite(admitted.suite()).run_admitted(&admitted, target);
    CountReport::from_run(&executed, &admitted)
        .expect("a complete run")
        .to_canonical_json()
        .expect("serializes")
}

/// The report/2 of a runner that discards event expectations: it runs `suite` without them and
/// reports each scenario of the suite it was given.
fn report_discarding_events(suite: &str, target: &impl ConformanceTarget) -> String {
    let admitted = AdmittedSuite::from_json(suite).unwrap();
    let stripped = without_event_expectations(admitted.suite());
    let ran = statuses(&stripped, target);
    let results: Vec<Value> = ran
        .iter()
        .map(|(id, status)| json!({"scenario_id": id, "status": status.to_string()}))
        .collect();
    let results = json!({"format": "ess-conformance-results/1",
        "completed_at": 1_700_000_000_000_u64, "results": results})
    .to_string();
    ess_conformance::results::report(suite, &results, "billing-reference 3.0.0", None)
        .expect("a results document of this suite")
        .to_canonical_json()
        .expect("serializes")
}

/// Every suite directory of an emission, the baseline first.
fn suite_dirs(emission: &mutate::Emission) -> Vec<String> {
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|mutant| mutant.dir.clone()),
    );
    dirs
}

fn collect(written: &BTreeMap<String, String>) -> MutationReport {
    mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|refusal| panic!("{refusal}"))
}

#[test]
fn a_target_that_emits_the_substituted_event_passes_that_mutants_suite_and_survives_it() {
    let (files, texts) = billing();
    let mutant = mutate::mutants(&files, SWAP)
        .into_iter()
        .find(|mutant| mutant.id == ISSUE)
        .unwrap();
    let suite = mutant_suite(&files, &texts, &mutant.mutation);
    let ran = statuses(&suite, &wrongly_issuing());
    assert!(
        ran.values().all(|status| *status == Status::Passed),
        "{ran:?}"
    );
    // Through emit and collect: the baseline answered by the reference, that mutant's suite by the
    // target that publishes what the mutant says. It survives; the other swap is still killed.
    let emission = mutate::emit(&files, &texts, SWAP).expect("emits");
    let mut written = emission.files.clone();
    for dir in suite_dirs(&emission) {
        let suite = &emission.files[&format!("{dir}/{SUITE_FILE}")];
        let report = if dir == ISSUE {
            report_of(suite, &wrongly_issuing())
        } else {
            report_of(suite, &Billing::new())
        };
        written.insert(format!("{dir}/{REPORT_FILE}"), report);
    }
    let report = collect(&written);
    assert_eq!(site(&report, ISSUE).verdict, Verdict::Survived);
    assert_eq!(site(&report, CANCEL_INVOICE).verdict, Verdict::Killed);
    assert_eq!(report.counts.survived, 1);
}

#[test]
fn a_runner_that_discards_event_expectations_fails_the_audit() {
    let (files, texts) = billing();
    let emission = mutate::emit(&files, &texts, SWAP).expect("emits");
    let mut written = emission.files.clone();
    for dir in suite_dirs(&emission) {
        let suite = &emission.files[&format!("{dir}/{SUITE_FILE}")];
        written.insert(
            format!("{dir}/{REPORT_FILE}"),
            report_discarding_events(suite, &Billing::new()),
        );
    }
    let report = collect(&written);
    assert_eq!(report.counts.survived, 2, "{}", report.render_text());
    assert_eq!(report.counts.killed, 0);
}

// ---- emit and collect ---------------------------------------------------------------------------

fn reported(
    emission: &mutate::Emission,
    target: impl Fn() -> Box<dyn Fn(&str) -> String>,
) -> BTreeMap<String, String> {
    let mut written = emission.files.clone();
    for dir in suite_dirs(emission) {
        let suite = &emission.files[&format!("{dir}/{SUITE_FILE}")];
        written.insert(format!("{dir}/{REPORT_FILE}"), target()(suite));
    }
    written
}

#[test]
fn emit_and_collect_score_exactly_what_the_direct_audit_scores() {
    for (name, spec) in [
        ("billing", billing()),
        ("swaps", swaps()),
        ("served", served()),
    ] {
        let (files, texts) = &spec;
        let (direct, written) = if name == "billing" {
            let direct = mutate::audit(files, texts, SWAP, Billing::new).unwrap();
            let emission = mutate::emit(files, texts, SWAP).unwrap();
            let written = reported(&emission, || {
                Box::new(|suite: &str| report_of(suite, &Billing::new()))
            });
            (direct, written)
        } else {
            let make = interpreted(&spec);
            let direct = mutate::audit(files, texts, SWAP, &make).unwrap();
            let emission = mutate::emit(files, texts, SWAP).unwrap();
            assert_eq!(emission.manifest.format, MANIFEST_FORMAT_4, "{name}");
            let make = interpreted(&spec);
            let written = reported(&emission, move || {
                let target = make();
                Box::new(move |suite: &str| report_of(suite, &target))
            });
            (direct, written)
        };
        let collected = same_label(collect(&written), &direct);
        assert_eq!(
            collected.to_canonical_json(),
            direct.to_canonical_json(),
            "{name}: one audit, two routes"
        );
        assert_eq!(collected.render_text(), direct.render_text(), "{name}");
    }
}

/// `collected` with the direct audit's implementation label. The direct audit names the target by
/// its name, and a collected report/2 by `<name> <version>`, as both routes always have; every
/// other byte is compared.
fn same_label(mut collected: MutationReport, direct: &MutationReport) -> MutationReport {
    assert!(
        collected
            .implementation
            .starts_with(&format!("{} ", direct.implementation)),
        "{} against {}",
        collected.implementation,
        direct.implementation
    );
    collected.implementation.clone_from(&direct.implementation);
    collected
}

#[test]
fn the_manifest_records_unavailable_sites_and_older_formats_refuse_them() {
    let (files, texts) = swaps();
    let emission = mutate::emit(&files, &texts, SWAP).unwrap();
    assert_eq!(emission.manifest.format, MANIFEST_FORMAT_4);
    assert_eq!(
        unavailable_ids(emission.manifest.unavailable_sites.as_deref().unwrap()),
        [CANCEL]
    );
    let text = &emission.files[MANIFEST_FILE];
    let read = mutate::Manifest::from_json(text).expect("reads back");
    assert_eq!(read, emission.manifest);
    let value: Value = serde_json::from_str(text).unwrap();
    assert_eq!(value["unavailable_sites"][0]["id"], CANCEL);
    assert_eq!(
        value["unavailable_sites"][0]["reason"],
        "no_compatible_event_alternative"
    );
    // A /3 manifest carrying the field is refused by name, before it is ignored.
    let mut older = value.clone();
    older["format"] = MANIFEST_FORMAT.into();
    older["mutants"] = json!([]);
    older["baseline"]
        .as_object_mut()
        .unwrap()
        .remove("suite_digest");
    let refused =
        mutate::Manifest::from_json(&older.to_string()).expect_err("/3 has no such field");
    assert!(refused.contains("unavailable_sites"), "{refused}");
    // A closed reason, an exact id, and a class that has unavailable sites.
    for (field, replacement, needle) in [
        ("reason", json!("no_alternative"), "no_alternative"),
        ("id", json!("emit-swap/swaps.order.Cancel/cancelled"), "id"),
        ("class", json!("emit-drop"), "emit-drop"),
    ] {
        let mut changed = value_with(&value, field, replacement);
        changed["format"] = MANIFEST_FORMAT_4.into();
        let refused = mutate::Manifest::from_json(&changed.to_string())
            .expect_err("an incoherent unavailable site");
        assert!(refused.contains(needle), "{field}: {refused}");
    }
    // `outside_component` names a component the emission was scoped to.
    let changed = value_with(&value, "reason", json!("outside_component"));
    let refused = mutate::Manifest::from_json(&changed.to_string()).expect_err("no component");
    assert!(refused.contains("component"), "{refused}");
}

fn value_with(manifest: &Value, field: &str, replacement: Value) -> Value {
    let mut changed = manifest.clone();
    changed["unavailable_sites"][0][field] = replacement;
    changed
}

#[test]
fn a_component_scoped_emission_lists_another_components_unavailable_site_as_outside_it() {
    let spec = swaps();
    let (files, texts) = &spec;
    let emission = mutate::emit_for(files, texts, SWAP, Some("beta")).unwrap();
    let sites = emission.manifest.unavailable_sites.as_deref().unwrap();
    assert_eq!(unavailable_ids(sites), [CANCEL]);
    assert_eq!(sites[0].reason, UnavailableReason::OutsideComponent);
    let in_scope: Vec<&str> = emission
        .manifest
        .mutants
        .iter()
        .filter(|mutant| !mutant.out_of_scope)
        .map(|mutant| mutant.id.as_str())
        .collect();
    assert_eq!(in_scope, [PLACE]);
    let make = interpreted(&spec);
    let written = reported(&emission, move || {
        let target = make();
        Box::new(move |suite: &str| report_of(suite, &target))
    });
    let report = mutate::collect_for(|path| written.get(path).cloned(), Some("beta")).unwrap();
    assert_eq!(report.format, REPORT_FORMAT_4);
    assert_eq!(report.counts.killed, 1);
    let sites = report.unavailable_sites.as_deref().unwrap();
    assert_eq!(sites[0].reason, UnavailableReason::OutsideComponent);
    assert!(
        report
            .render_text()
            .contains(&format!("unavailable {CANCEL}: outside_component")),
        "{}",
        report.render_text()
    );
    // `alpha` accepts both commands: its own unavailable site is in scope.
    let emission = mutate::emit_for(files, texts, SWAP, Some("alpha")).unwrap();
    assert_eq!(
        emission.manifest.unavailable_sites.as_deref().unwrap()[0].reason,
        UnavailableReason::NoCompatibleEventAlternative
    );
}

#[test]
fn an_emission_whose_only_sites_are_unavailable_still_collects_them() {
    let (files, texts) = billing_without_components();
    let emission = mutate::emit(&files, &texts, SWAP).expect("emits, not ESS-MUTATE-003");
    assert_eq!(emission.manifest.mutants.len(), 0);
    assert_eq!(
        emission
            .manifest
            .unavailable_sites
            .as_deref()
            .unwrap()
            .len(),
        5
    );
    let written = reported(&emission, || {
        Box::new(|suite: &str| report_of(suite, &Billing::new()))
    });
    let direct = mutate::audit(&files, &texts, SWAP, Billing::new).unwrap();
    let collected = same_label(collect(&written), &direct);
    assert_eq!(collected.to_canonical_json(), direct.to_canonical_json());
}

#[test]
fn the_class_list_ends_with_emit_swap_and_older_classes_keep_their_reports() {
    assert_eq!(MutantClass::ALL.last(), Some(&MutantClass::EmitSwap));
    assert_eq!(MutantClass::EmitSwap.as_str(), "emit-swap");
    assert_eq!(MutantClass::EmitSwap.manifest_format(), MANIFEST_FORMAT_4);
    // emit-drop on billing writes the report it always wrote.
    let (files, texts) = billing();
    let report = mutate::audit(&files, &texts, &[MutantClass::EmitDrop], Billing::new).unwrap();
    assert_eq!(report.format, REPORT_FORMAT);
    assert_eq!(report.unavailable_sites, None);
    assert_eq!(report.counts.stillborn, 5);
    assert!(!report.to_canonical_json().contains("unavailable"));
}
