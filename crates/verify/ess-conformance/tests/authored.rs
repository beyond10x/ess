//! What an authored scenario compiles to, and every way it is refused.
//!
//! The point of the feature is the refusals. A scenario a person writes against a real model names
//! a few dozen constructs, and the model moves underneath it — so what an authoring surface is
//! *for* is that a name the specification no longer declares is refused when the file is compiled,
//! by name, rather than failing at run time in each consumer that happens to execute it.
//!
//! So there is a case per cause below, and `every_cause_is_reachable_from_a_document` asserts that
//! the set of codes these cases produce is the whole set the module can emit: a cause added without
//! a document that reaches it fails here rather than shipping unexercised.
//!
//! The documents are compiled against `examples/billing/` — the normative example — except where a
//! construct it does not carry is needed, which is once: `gatepass.visit.RegisterVisit` is the
//! command in this repository whose input holds an enum, and an enum is the only place a *variant*
//! can be misspelt.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::diagnostic::Code;
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Cause, Refusal, Source};
use ess_conformance::{
    ConformanceSuite, ScenarioId, ScenarioStep, SuiteProvenance, ViewExpectation,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

// ---- the models under test ---------------------------------------------------------------------

/// An example directory, compiled from the files it lives in rather than from a copy inlined here.
fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));

    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();

    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((SpecSource::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("`{name}` validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("`{name}` resolves:\n{diagnostics}"))
}

// ---- writing and compiling one document ----------------------------------------------------------

/// A document with the four keys every scenario carries, and whatever body a case needs.
fn document(body: &str) -> String {
    format!(
        "type: ess-scenario/1\n\
         domain: billing.invoice\n\
         scenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n\
         {body}"
    )
}

/// The valid timeline every positive case starts from: one invoice, created and accepted.
const CREATED: &str = "\
timeline:
  - at: 2026-01-05T09:00:00Z
    command: billing.invoice.CreateInvoice
    actor: billing.invoice.Customer
    input:
      account_id: 00000000-0000-4000-8000-000000000001
      customer_email: buyer@example.test
      amount: {amount: 10, currency: EUR}
    outcome: accepted
";

/// The instances the cases that bind one declare.
///
/// Raw strings throughout, because a `\` continuation in a Rust literal swallows the leading
/// whitespace of the next line — and in YAML the leading whitespace is the structure.
const ARRANGED: &str = r"arrange:
  - instance: made
    entity: billing.invoice.Invoice
";

/// The occurrence the creating act publishes, required by name.
const OBSERVED: &str = r"    events:
      - event: billing.invoice.InvoiceCreated
        payload: {customer_email: buyer@example.test}
";

/// Binding the identity that act published.
const CAPTURED: &str = r"    capture: {instance: made, event: billing.invoice.InvoiceCreated, field: invoice_id}
";

/// Reading the invoice back out of the projection that holds it.
const READ: &str = r"assert:
  - view: billing.invoice.InvoiceById
    contains: {invoice_id: {$instance: made}}
";

/// A claim about which row a ranked view puts first.
const RANKED: &str = r"assert:
  - view: billing.invoice.OutstandingInvoices
    at:
      row: first
      fields: {total: {amount: 10, currency: EUR}}
";

/// Compiles one document against a model.
fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

/// The one refusal a document produced, or a failure naming what it produced instead.
fn refusal(ir: &EssIr, text: &str) -> Refusal {
    let authoring = authoring(ir, text);
    assert!(
        authoring.scenarios.is_empty(),
        "a refused document produced a scenario, which would put an unresolvable check in a suite"
    );
    let mut refusals = authoring.refusals;
    assert_eq!(
        refusals.len(),
        1,
        "one document, one mistake, one refusal: {}",
        refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    refusals.remove(0)
}

/// The cause a document is refused for, when exactly one refusal is expected.
fn cause(ir: &EssIr, text: &str) -> Cause {
    refusal(ir, text).cause
}

/// Every cause a document produced, for a case that legitimately produces more than one.
fn causes(ir: &EssIr, text: &str) -> Vec<Cause> {
    authoring(ir, text)
        .refusals
        .into_iter()
        .map(|refusal| refusal.cause)
        .collect()
}

// ---- the shape of what compiles ------------------------------------------------------------------

#[test]
fn a_scenario_compiles_to_the_id_the_domain_and_the_name_make() {
    let ir = example("billing");
    let authoring = authoring(&ir, &document(CREATED));

    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let ids: Vec<String> = authoring
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(ids, vec!["billing.invoice/authored/a-scenario"]);

    // The id is the distinction, and it survives the document: parsed back it is the same id, so a
    // report, a fault matrix and a `go test -run` filter each read it without being told.
    let parsed: ScenarioId = "billing.invoice/authored/a-scenario"
        .parse()
        .expect("an authored id round-trips");
    assert!(authoring.scenarios.contains_key(&parsed));
    assert!(matches!(parsed, ScenarioId::Authored { .. }));
}

#[test]
fn the_steps_are_the_vocabulary_a_generated_scenario_already_uses() {
    // The whole claim of the feature: an authored scenario is not a second kind of check. It comes
    // out as the same closed step vocabulary, so the runners that exist run it with no change to
    // `ConformanceTarget`.
    let ir = example("billing");
    let body = format!("{ARRANGED}{CREATED}{OBSERVED}{CAPTURED}{READ}");
    let authoring = authoring(&ir, &document(&body));
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);

    let scenario = authoring.scenarios.values().next().expect("one scenario");
    let shape: Vec<&str> = scenario
        .steps
        .iter()
        .map(|step| match step {
            ScenarioStep::ExecuteCommand { .. } => "execute",
            ScenarioStep::ExpectOutcome { .. } => "outcome",
            ScenarioStep::ExpectEvent { .. } => "event",
            ScenarioStep::CaptureInstance { .. } => "capture",
            ScenarioStep::EventuallyView { .. } => "eventually-view",
            ScenarioStep::QueryView { .. } => "query",
            ScenarioStep::ExpectView { .. } => "expect-view",
            other => panic!("an authored scenario emitted {other:?}"),
        })
        .collect();
    // `InvoiceById` is `eventual`, so the assertion retries. The author did not say so and cannot:
    // the model decided it, exactly as it decides it for a generated scenario.
    assert_eq!(
        shape,
        vec!["execute", "outcome", "event", "capture", "eventually-view"]
    );
}

#[test]
fn a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author() {
    // The one thing synthesis will not write and an author has to: which row is first. The keys it
    // is relative to are not written here — the view declares `order_by: issued_at desc`, and a
    // second copy in the scenario is the copy that goes stale.
    let ir = example("billing");
    let body = format!("{CREATED}{RANKED}");
    let authoring = authoring(&ir, &document(&body));
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);

    let scenario = authoring.scenarios.values().next().expect("one scenario");
    let ordered = scenario.steps.iter().any(|step| {
        matches!(
            step,
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::At { order_by, .. },
                ..
            } if order_by.iter().map(ToString::to_string).collect::<Vec<_>>()
                == vec!["issued_at desc"]
        )
    });
    assert!(ordered, "the view's own ranking reaches the assertion");
}

#[test]
fn the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones() {
    // The acceptance, read off the artifact rather than off a run: one document, two populations,
    // and a reader can tell which is which from the key alone.
    let suite = ConformanceSuite::from_json(include_str!(
        "../../../../suites/generated/billing/suite.json"
    ))
    .expect("the committed billing suite parses");
    let authored: Vec<String> = suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ScenarioId::Authored { .. }))
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        authored,
        vec!["billing.invoice/authored/outstanding-invoices-rank-latest-first"]
    );
    assert_eq!(suite.len(), 30, "twenty-nine obligations and one assertion");
}

#[test]
fn two_compilations_of_one_file_produce_identical_bytes() {
    // §37's determinism, for the authoring half. Compiled twice and written twice, and read back
    // and written again: a suite that moved between two compilations of one unchanged file could
    // not be drift-checked, which is what a committed suite is for.
    let ir = example("billing");
    let text =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml",
        ))
        .expect("the committed authored scenario is readable");

    let render = || {
        let authoring = compile_authored(&ir, &[Source::new("scenario.yaml", text.clone())]);
        assert!(authoring.is_complete(), "{:?}", authoring.refusals);
        let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
        for (id, scenario) in authoring.scenarios {
            suite.insert(id, scenario).expect("one scenario, one id");
        }
        suite
    };
    let first = render().to_canonical_json();
    assert_eq!(first, render().to_canonical_json());
    assert_eq!(
        first,
        ConformanceSuite::from_json(&first)
            .expect("the suite reads back")
            .to_canonical_json(),
        "compile, write, read, write: the same bytes"
    );
}

#[test]
fn the_order_the_files_are_handed_over_in_does_not_reach_the_result() {
    // `read_dir` is not ordered, so the compiler orders its input. Without it, which of two files
    // declaring one scenario is the duplicate would depend on the file system.
    let ir = example("billing");
    let first = Source::new("a.yaml", document(CREATED));
    let second = Source::new(
        "b.yaml",
        document(CREATED).replace("scenario: a-scenario", "scenario: b-scenario"),
    );
    let forwards = compile_authored(&ir, &[first.clone(), second.clone()]);
    let backwards = compile_authored(&ir, &[second, first]);
    assert_eq!(forwards, backwards);
    assert_eq!(forwards.scenarios.len(), 2);
}

// ---- one case per refusal ------------------------------------------------------------------------

#[test]
fn a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario() {
    let ir = example("billing");
    let text = document("timelime:\n  - at: 2026-01-05T09:00:00Z\n");
    assert!(matches!(cause(&ir, &text), Cause::Unreadable { .. }));
}

#[test]
fn a_format_this_build_does_not_implement_is_refused_before_anything_is_read() {
    let ir = example("billing");
    let text = document(CREATED).replace("type: ess-scenario/1", "type: ess-scenario/9");
    assert!(matches!(
        cause(&ir, &text),
        Cause::UnsupportedFormat { found } if found == "ess-scenario/9"
    ));
}

#[test]
fn two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other() {
    let ir = example("billing");
    let authoring = compile_authored(
        &ir,
        &[
            Source::new("a.yaml", document(CREATED)),
            Source::new("b.yaml", document(CREATED)),
        ],
    );
    assert_eq!(authoring.scenarios.len(), 1, "the first one stands");
    assert!(matches!(
        authoring.refusals.as_slice(),
        [Refusal { origin, cause: Cause::Duplicate { first }, .. }]
            if origin == "b.yaml" && first == "a.yaml"
    ));
}

#[test]
fn a_domain_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace("domain: billing.invoice", "domain: billing.invoicing");
    assert!(matches!(
        cause(&ir, &text),
        Cause::UndeclaredDomain { domain, .. } if domain == "billing.invoicing"
    ));
}

#[test]
fn an_entity_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body =
        format!("arrange:\n  - instance: made\n    entity: billing.invoice.Facture\n{CREATED}");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredEntity { entity } if entity == "billing.invoice.Facture"
    ));
}

#[test]
fn a_command_the_model_does_not_declare_is_refused_by_name() {
    // The case the whole feature is measured by: a scenario naming a command the specification does
    // not declare is refused when it is compiled, naming the command and the file it is in.
    let ir = example("billing");
    let text = document(CREATED).replace(
        "command: billing.invoice.CreateInvoice",
        "command: billing.invoice.CreateInvoce",
    );
    let refused = refusal(&ir, &text);
    assert!(matches!(
        &refused.cause,
        Cause::UndeclaredCommand { command } if command == "billing.invoice.CreateInvoce"
    ));
    let printed = refused.to_string();
    assert!(
        printed.contains("billing.invoice.CreateInvoce"),
        "{printed}"
    );
    assert!(printed.contains("scenario.yaml"), "{printed}");
    assert!(printed.contains("ESS-AUTHOR-006"), "{printed}");
}

#[test]
fn an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does() {
    let ir = example("billing");
    let text = document(CREATED).replace("outcome: accepted", "outcome: approved");
    assert!(matches!(
        cause(&ir, &text),
        Cause::UndeclaredOutcome { outcome, declared, .. }
            if outcome == "approved" && declared == vec!["accepted", "rejected"]
    ));
}

#[test]
fn an_actor_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace(
        "actor: billing.invoice.Customer",
        "actor: billing.invoice.Cusomer",
    );
    assert!(matches!(
        cause(&ir, &text),
        Cause::UndeclaredActor { actor } if actor == "billing.invoice.Cusomer"
    ));
}

#[test]
fn an_actor_the_specification_does_not_grant_the_command_is_refused() {
    // Declared, spelt right, and not permitted: `may:` is a claim about who can do what, and a
    // scenario acting as somebody the model does not permit is checking a system it does not
    // describe.
    let ir = example("billing");
    let text = document(CREATED).replace(
        "actor: billing.invoice.Customer",
        "actor: billing.invoice.Auditor",
    );
    assert!(matches!(
        cause(&ir, &text),
        Cause::ActorMayNot { actor, command }
            if actor.to_string() == "billing.invoice.Auditor"
                && command.to_string() == "billing.invoice.CreateInvoice"
    ));
}

#[test]
fn an_event_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!("{CREATED}    events:\n      - event: billing.invoice.InvoiceMade\n");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredEvent { event } if event == "billing.invoice.InvoiceMade"
    ));
}

#[test]
fn an_error_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!("{CREATED}    error:\n      name: billing.invoice.BadAmount\n");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredError { error } if error == "billing.invoice.BadAmount"
    ));
}

#[test]
fn a_view_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}assert:\n  - view: billing.invoice.AllInvoices\n    counts: {{at_least: 1}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredView { view } if view == "billing.invoice.AllInvoices"
    ));
}

#[test]
fn a_field_the_surface_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace("      account_id:", "      acount_id:");
    let found = causes(&ir, &text);
    assert!(
        found.iter().any(|cause| matches!(
            cause,
            Cause::UndeclaredField { field, .. } if field == "acount_id"
        )),
        "{found:?}"
    );
}

#[test]
fn a_declared_field_nothing_supplies_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace("      amount: {amount: 10, currency: EUR}\n", "");
    assert!(matches!(
        cause(&ir, &text),
        Cause::MissingField { field, .. } if field == "amount"
    ));
}

#[test]
fn a_value_the_declared_type_does_not_admit_is_refused_where_it_sits() {
    let ir = example("billing");
    let text = document(CREATED).replace(
        "{amount: 10, currency: EUR}",
        "{amount: ten, currency: EUR}",
    );
    assert!(matches!(
        cause(&ir, &text),
        Cause::ValueRejected { detail, .. } if detail.contains("amount.amount")
    ));
}

#[test]
fn a_name_a_closed_set_does_not_have_is_refused_with_the_set() {
    // The one case `examples/billing` cannot make: it declares an enum and no command takes one.
    // `gatepass.visit.RegisterVisit` does, which is why this document is written against it.
    let ir = example("gatepass");
    let text = "\
type: ess-scenario/1
domain: gatepass.visit
scenario: a-scenario
summary: What this scenario proves, in one line.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: gatepass.visit.RegisterVisit
    input:
      building: Basement
";
    let found = causes(&ir, text);
    assert!(
        found.iter().any(|cause| matches!(
            cause,
            Cause::UndeclaredVariant { value, variants, .. }
                if value == "Basement" && variants == &["North", "South", "Annex"]
        )),
        "{found:?}"
    );
}

#[test]
fn a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant() {
    // A lifecycle's states reach the model as an enum like any other, and a reader who wrote
    // `Payed` needs to be told which states the *invoice* has.
    let ir = example("billing");
    let body = format!(
        "{CREATED}    error:\n      name: billing.invoice.InvoiceStateConflict\n      \
         fields: {{state: Payed}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredState { entity, state, declared }
            if entity.to_string() == "billing.invoice.Invoice"
                && state == "Payed"
                && declared == vec!["Cancelled", "Draft", "Issued", "Paid"]
    ));
}

#[test]
fn an_instance_the_arrangement_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}\
         assert:\n  - view: billing.invoice.InvoiceById\n    \
         contains: {{invoice_id: {{$instance: nobody}}}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnarrangedInstance { instance, .. } if instance.as_str() == "nobody"
    ));
}

#[test]
fn an_instance_named_before_anything_binds_it_is_refused() {
    // A suite carries no identity of its own — the run mints it — so a reference before the capture
    // that binds it is a step the runner could not have executed.
    let ir = example("billing");
    let body = format!(
        "arrange:\n  - instance: made\n    entity: billing.invoice.Invoice\n\
         {CREATED}\
         assert:\n  - view: billing.invoice.InvoiceById\n    \
         contains: {{invoice_id: {{$instance: made}}}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnboundInstance { instance } if instance.as_str() == "made"
    ));
}

#[test]
fn a_value_read_off_an_event_nothing_required_is_refused() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}  - at: 2026-01-05T09:00:01Z\n    \
         command: billing.invoice.PayInvoice\n    input:\n      \
         invoice_id: {{$observed: {{event: billing.invoice.InvoicePaid, field: invoice_id}}}}\n      \
         amount: {{amount: 10, currency: EUR}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::Unobserved { event } if event.to_string() == "billing.invoice.InvoicePaid"
    ));
}

#[test]
fn a_reference_where_the_suite_compares_a_value_it_carries_is_refused() {
    // `ExpectEvent`'s payload is compared against values the suite holds, so a reference there is a
    // claim the format cannot make — refused rather than silently dropped.
    let ir = example("billing");
    let body = format!(
        "arrange:\n  - instance: made\n    entity: billing.invoice.Invoice\n\
         {CREATED}    events:\n      - event: billing.invoice.InvoiceCreated\n        \
         payload: {{invoice_id: {{$instance: made}}}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::NotComparable { field, .. } if field == "invoice_id"
    ));
}

#[test]
fn an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused() {
    // `PayInvoice` takes an invoice and an amount, and the model says which is which. Binding the
    // invoice to the amount is a scenario nothing would ever have executed.
    let ir = example("billing");
    let body = format!(
        "arrange:\n  - instance: made\n    entity: billing.invoice.Invoice\n\
         {CREATED}    capture: {{instance: made, event: billing.invoice.InvoiceCreated, \
         field: invoice_id}}\n  - at: 2026-01-05T09:00:01Z\n    \
         command: billing.invoice.PayInvoice\n    input:\n      \
         invoice_id: {{$instance: made}}\n      amount: {{$instance: made}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::InstanceMistyped { field, declared, identity, .. }
            if field == "amount"
                && declared == "billing.invoice.Money"
                && identity == "billing.invoice.InvoiceId"
    ));
}

#[test]
fn a_timeline_whose_instants_do_not_ascend_is_refused() {
    // The file's order is the scenario's order, and `at:` is what states it. Without this a moved
    // block changes what the scenario means and nothing says so.
    let ir = example("billing");
    let body = format!(
        "{CREATED}  - at: 2026-01-05T08:00:00Z\n    \
         command: billing.invoice.CancelInvoice\n    input:\n      \
         invoice_id: 00000000-0000-4000-8000-000000000001\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnorderedTimeline { .. }
    ));
}

#[test]
fn a_position_in_a_view_that_declares_no_order_is_refused() {
    // `InvoiceById` declares no `order_by:`, so "the first row" is a different row on every read
    // and an assertion about it is a coin toss reported as a check.
    let ir = example("billing");
    let body = format!(
        "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    at:\n      row: first\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::Unordered { view } if view.to_string() == "billing.invoice.InvoiceById"
    ));
}

#[test]
fn an_assertion_that_states_other_than_one_claim_is_refused() {
    let ir = example("billing");
    let two = format!(
        "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    \
         counts: {{at_least: 1}}\n    contains: {{}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&two)),
        Cause::AmbiguousClaim { stated, .. } if stated == vec!["contains", "counts"]
    ));

    let none = format!("{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n");
    assert!(matches!(
        cause(&ir, &document(&none)),
        Cause::AmbiguousClaim { stated, .. } if stated.is_empty()
    ));
}

#[test]
fn a_predicate_reading_something_the_view_does_not_publish_is_refused() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    \
         satisfies: customer_email == buyer@example.test\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnreadablePredicate { path, .. } if path == "customer_email"
    ));
}

#[test]
fn a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check() {
    let ir = example("billing");
    assert!(matches!(cause(&ir, &document("")), Cause::NothingHappens));
}

// ---- the set is closed ----------------------------------------------------------------------------

#[test]
fn every_cause_is_reachable_from_a_document() {
    // The guard on the list above. A cause added to the module without a case that reaches it would
    // ship as a refusal nobody has seen the wording of — and the wording is the whole product here.
    let billing = example("billing");
    let gatepass = example("gatepass");
    let reached: BTreeSet<String> = refusable()
        .into_iter()
        .flat_map(|(model, documents)| {
            let ir = if model == "gatepass" {
                &gatepass
            } else {
                &billing
            };
            let sources: Vec<Source> = documents
                .into_iter()
                .enumerate()
                .map(|(index, text)| Source::new(format!("scenario-{index}.yaml"), text))
                .collect();
            compile_authored(ir, &sources).refusals
        })
        .map(|refusal| refusal.code().to_string())
        .collect();
    let declared: BTreeSet<String> = (1..=CAUSES)
        .map(|number| Code::new("AUTHOR", number).to_string())
        .collect();
    assert_eq!(
        reached, declared,
        "every numbered cause has a document that reaches it, and no number is unused"
    );
}

/// How many causes `ess_conformance::authored::Cause` numbers.
///
/// Written down rather than counted, because the point of the case above is that the numbering and
/// the documents agree: a count taken from the enum would agree with itself whatever happened.
const CAUSES: u16 = 27;

/// One entry per refusal, each the documents that reach it compiled together.
///
/// Restated here rather than shared with the cases above, because a case reads better with its
/// document beside it and this reads better as a list. All but one entry is a single document; the
/// duplicate is the one refusal that is about two.
fn refusable() -> Vec<(&'static str, Vec<String>)> {
    let billing = |body: String| ("billing", vec![document(&body)]);
    let edited = |from: &str, to: &str| ("billing", vec![document(CREATED).replace(from, to)]);
    vec![
        // 1 unreadable, 2 unsupported format, 3 duplicate, 4 undeclared domain
        billing("timelime: []\n".to_owned()),
        edited("ess-scenario/1", "ess-scenario/9"),
        (
            "billing",
            vec![document(CREATED), document(CREATED)],
        ),
        edited("domain: billing.invoice", "domain: billing.ledger"),
        // 5 entity, 6 command, 7 outcome, 8 actor, 9 grant
        billing(format!(
            "arrange:\n  - instance: made\n    entity: billing.invoice.Facture\n{CREATED}"
        )),
        edited("billing.invoice.CreateInvoice", "billing.invoice.CreateInvoce"),
        edited("outcome: accepted", "outcome: approved"),
        edited("actor: billing.invoice.Customer", "actor: billing.invoice.Cusomer"),
        edited("actor: billing.invoice.Customer", "actor: billing.invoice.Auditor"),
        // 10 event, 11 error, 12 view
        billing(format!(
            "{CREATED}    events:\n      - event: billing.invoice.InvoiceMade\n"
        )),
        billing(format!(
            "{CREATED}    error:\n      name: billing.invoice.BadAmount\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.AllInvoices\n    counts: {{at_least: 1}}\n"
        )),
        // 13 undeclared field, 14 missing field, 15 value rejected
        edited("      account_id:", "      acount_id:"),
        edited("      amount: {amount: 10, currency: EUR}\n", ""),
        edited("{amount: 10, currency: EUR}", "{amount: ten, currency: EUR}"),
        // 16 variant — the one case `examples/billing` cannot make
        (
            "gatepass",
            vec!["type: ess-scenario/1\ndomain: gatepass.visit\nscenario: a-scenario\n\
                  summary: What this scenario proves.\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    \
                  command: gatepass.visit.RegisterVisit\n    input:\n      building: Basement\n"
                .to_owned()],
        ),
        // 17 state
        billing(format!(
            "{CREATED}    error:\n      name: billing.invoice.InvoiceStateConflict\n      \
             fields: {{state: Payed}}\n"
        )),
        // 18 unarranged, 19 unbound, 20 unobserved, 21 not comparable, 22 mistyped
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    \
             contains: {{invoice_id: {{$instance: nobody}}}}\n"
        )),
        billing(format!("{ARRANGED}{CREATED}{READ}")),
        billing(format!(
            "{CREATED}  - at: 2026-01-05T09:00:01Z\n    command: billing.invoice.PayInvoice\n    \
             input:\n      invoice_id: {{$observed: {{event: billing.invoice.InvoicePaid, \
             field: invoice_id}}}}\n      amount: {{amount: 10, currency: EUR}}\n"
        )),
        billing(format!(
            "{ARRANGED}{CREATED}    events:\n      - event: billing.invoice.InvoiceCreated\n        \
             payload: {{invoice_id: {{$instance: made}}}}\n"
        )),
        billing(format!(
            "{ARRANGED}{CREATED}{CAPTURED}  - at: 2026-01-05T09:00:01Z\n    \
             command: billing.invoice.PayInvoice\n    input:\n      \
             invoice_id: {{$instance: made}}\n      amount: {{$instance: made}}\n"
        )),
        // 23 unordered timeline, 24 unordered view, 25 ambiguous, 26 unreadable predicate
        billing(format!(
            "{CREATED}  - at: 2026-01-05T08:00:00Z\n    command: billing.invoice.CancelInvoice\n    \
             input:\n      invoice_id: 00000000-0000-4000-8000-000000000001\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    at:\n      row: first\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    counts: {{at_least: 1}}\n    \
             contains: {{}}\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    \
             satisfies: customer_email == buyer@example.test\n"
        )),
        // 27 nothing happens
        billing(String::new()),
    ]
}
