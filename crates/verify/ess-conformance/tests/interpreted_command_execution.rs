//! A command's declared outcome is executed from the IR (`story:interpreted-command-execution`).
//!
//! The acceptance, verbatim: every scenario in `examples/billing`'s committed suite that exercises
//! only command execution, transitions, `sets:` writes, emitted events and declared refusals reports
//! the same result under `--target interpreted` as under `--target billing`.
//!
//! The scenarios are selected by the steps they hold, not by a list typed here, and the selection is
//! pinned so that a filter which quietly admits nothing cannot pass. Two further cases hold the
//! story's second sentence — nothing is chosen by the interpreter, and a refusal the model does not
//! declare is not available to it — against the library step the linearizability checker will call.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::reference::Billing;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::runner::Runner;
use ess_conformance::scenario::{ScenarioStep, SuiteProvenance};
use ess_conformance::AdmittedSuite;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// `examples/billing`, compiled from the files it lives in.
fn billing_model() -> EssIr {
    let base = root()
        .join("examples/billing")
        .canonicalize()
        .expect("the billing example exists");
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
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("billing validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("billing resolves:\n{diagnostics}"))
}

/// The committed suite, admitted from its original bytes exactly as `ess conform run --suite` does.
fn committed_suite() -> AdmittedSuite {
    let text = std::fs::read_to_string(root().join("suites/generated/billing/suite.json"))
        .expect("the committed billing suite is readable");
    AdmittedSuite::from_json(&text).expect("the committed billing suite is admitted")
}

/// `true` when every step is command execution or an assertion about what that execution did.
///
/// No view, no binding, no injected external outcome, no time: those are other stories of the
/// epic, and a scenario holding one is outside this acceptance whatever else it holds.
fn exercises_only_command_execution(steps: &[ScenarioStep]) -> bool {
    steps.iter().all(|step| {
        matches!(
            step,
            ScenarioStep::ExecuteCommand { .. }
                | ScenarioStep::ExpectOutcome { .. }
                | ScenarioStep::ExpectError { .. }
                | ScenarioStep::ExpectNoError
                | ScenarioStep::ExpectEvent { .. }
                | ScenarioStep::ExpectNoEvent { .. }
                | ScenarioStep::ExpectNoEvents
                | ScenarioStep::CaptureInstance { .. }
        )
    })
}

/// Every scenario result by id.
fn by_id(results: Vec<ScenarioResult>) -> BTreeMap<String, ScenarioResult> {
    results
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

#[test]
fn the_model_interpreted_is_the_model_the_committed_suite_was_synthesized_from() {
    let admitted = committed_suite();
    assert_eq!(
        SuiteProvenance::of(&billing_model()).spec_digest,
        admitted.suite().provenance.spec_digest,
        "the interpreter must run the document the suite came from, or agreement proves nothing"
    );
}

#[test]
fn every_command_execution_scenario_reports_the_same_result_as_billing() {
    let admitted = committed_suite();
    let suite = admitted.suite();

    let in_scope: BTreeSet<String> = suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| exercises_only_command_execution(&scenario.steps))
        .map(|(id, _)| id.to_string())
        .collect();
    // Pinned, so a selection that admits nothing — or drops a scenario — fails here rather than
    // passing vacuously.
    let expected: BTreeSet<String> = [
        "billing.email.SendEmail/outcome/sent",
        "billing.invoice.CancelInvoice/outcome/wrong-state",
        "billing.invoice.CreateInvoice/outcome/rejected",
        "billing.invoice.Invoice/state/Cancelled/refuses/billing.invoice.CancelInvoice",
        "billing.invoice.Invoice/state/Cancelled/refuses/billing.invoice.IssueInvoice",
        "billing.invoice.Invoice/state/Cancelled/refuses/billing.invoice.PayInvoice",
        "billing.invoice.Invoice/state/Draft/refuses/billing.invoice.PayInvoice",
        "billing.invoice.Invoice/state/Issued/refuses/billing.invoice.IssueInvoice",
        "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.CancelInvoice",
        "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.IssueInvoice",
        "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.PayInvoice",
        "billing.invoice.IssueInvoice/outcome/wrong-state",
        "billing.invoice.PayInvoice/outcome/rejected",
        "billing.invoice.PayInvoice/outcome/wrong-state",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(in_scope, expected, "the scenarios this acceptance covers");

    let billing = by_id(
        Runner::for_suite(suite)
            .run_admitted(&admitted, &Billing::new())
            .into_report()
            .scenarios,
    );
    let interpreted = by_id(
        Runner::for_suite(suite)
            .run_admitted(&admitted, &Interpreted::for_model(billing_model()))
            .into_report()
            .scenarios,
    );

    for id in &in_scope {
        let reference = &billing[id];
        let candidate = &interpreted[id];
        assert_eq!(
            reference.status,
            Status::Passed,
            "`{id}` passes against the hand-written reference"
        );
        assert_eq!(
            (candidate.status, &candidate.checks),
            (reference.status, &reference.checks),
            "`{id}` reports the same result under the interpreter as under billing"
        );
    }
}

#[test]
fn no_scenario_outside_the_acceptance_is_contradicted_by_what_the_interpreter_has_not_derived() {
    let admitted = committed_suite();
    let suite = admitted.suite();
    let billing = by_id(
        Runner::for_suite(suite)
            .run_admitted(&admitted, &Billing::new())
            .into_report()
            .scenarios,
    );
    let interpreted = by_id(
        Runner::for_suite(suite)
            .run_admitted(&admitted, &Interpreted::for_model(billing_model()))
            .into_report()
            .scenarios,
    );
    assert_eq!(interpreted.len(), 33, "every committed scenario ran");
    for (id, candidate) in &interpreted {
        let same = candidate.status == billing[id].status && candidate.checks == billing[id].checks;
        assert!(
            same || candidate.status == Status::Unsupported,
            "`{id}` either agrees with billing or is an unsatisfied obligation — never a failure \
             or an error the interpreter caused by guessing: {:#?}",
            candidate.checks
        );
    }
}

// ---- the library step, as the linearizability checker calls it ---------------------------------

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a well-formed name")
}

fn money(amount: i64) -> Node {
    Node::Map(BTreeMap::from([
        ("amount".to_owned(), Node::Number(Number::from(amount))),
        ("currency".to_owned(), Node::Text("EUR".to_owned())),
    ]))
}

fn create_input(amount: i64) -> BTreeMap<String, Node> {
    BTreeMap::from([
        (
            "account_id".to_owned(),
            Node::Text("00000000-0000-4000-8000-00000000abcd".to_owned()),
        ),
        (
            "customer_email".to_owned(),
            Node::Text("someone@example.invalid".to_owned()),
        ),
        ("amount".to_owned(), money(amount)),
    ])
}

fn invoice_input(id: &str) -> BTreeMap<String, Node> {
    BTreeMap::from([("invoice_id".to_owned(), Node::Text(id.to_owned()))])
}

/// Creates one invoice and returns the store after it and the identity it was given.
fn one_draft(ir: &EssIr) -> (Store, String) {
    let steps = execute(
        ir,
        &Store::default(),
        &name("billing.invoice.CreateInvoice"),
        &create_input(5),
        &Externals::Withheld,
    )
    .expect("the model determines CreateInvoice");
    assert_eq!(steps.len(), 1, "a positive amount has exactly one outcome");
    let step = &steps[0];
    let created = step
        .events
        .iter()
        .find(|event| event.event.to_string() == "billing.invoice.InvoiceCreated")
        .expect("accepted emits InvoiceCreated");
    let id = created.payload["invoice_id"]
        .as_text()
        .expect("the identity is published as text")
        .to_owned();
    assert_eq!(
        created.payload["amount"],
        money(5),
        "`amount: input.amount` is the payload mapping"
    );
    (step.next.clone(), id)
}

#[test]
fn a_step_takes_state_and_command_to_the_set_of_outcomes_the_model_allows() {
    let ir = billing_model();
    let (draft, id) = one_draft(&ir);
    let instance = draft
        .instance(&name("billing.invoice.Invoice"), &id)
        .expect("the created invoice is held");
    assert_eq!(
        instance.state.as_str(),
        "Draft",
        "`creates:` lands at `initial`"
    );
    assert_eq!(
        instance.fields.get("reminder_count"),
        Some(&Node::Number(Number::from(0_i64))),
        "`sets: reminder_count: \"0\"` writes the typed literal"
    );
    assert_eq!(
        instance.fields.get("total"),
        Some(&money(5)),
        "`sets: total: input.amount` writes the input"
    );

    // Issuing a Draft invoice has one allowed outcome, and it is not the refusal: a wrong-state
    // answer here would be a refusal the model does not declare for this state.
    let issued = execute(
        &ir,
        &draft,
        &name("billing.invoice.IssueInvoice"),
        &invoice_input(&id),
        &Externals::Withheld,
    )
    .expect("the model determines IssueInvoice");
    let outcomes: Vec<String> = issued
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), ToString::to_string)
        })
        .collect();
    assert_eq!(outcomes, ["billing.invoice.IssueInvoice/issued"]);
    assert_eq!(
        issued[0]
            .next
            .instance(&name("billing.invoice.Invoice"), &id)
            .map(|instance| instance.state.as_str().to_owned()),
        Some("Issued".to_owned()),
        "`moves: billing.invoice.Invoice.issue` takes the transition"
    );
    assert_eq!(
        draft
            .instance(&name("billing.invoice.Invoice"), &id)
            .map(|instance| instance.state.as_str().to_owned()),
        Some("Draft".to_owned()),
        "the state handed in is not mutated"
    );

    // Issuing it again is the declared refusal, and it changes nothing.
    let again = execute(
        &ir,
        &issued[0].next,
        &name("billing.invoice.IssueInvoice"),
        &invoice_input(&id),
        &Externals::Withheld,
    )
    .expect("the model determines IssueInvoice");
    assert_eq!(again.len(), 1);
    assert_eq!(
        again[0]
            .outcome
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("billing.invoice.IssueInvoice/wrong-state")
    );
    assert_eq!(
        again[0]
            .error
            .as_ref()
            .map(|error| error.error.to_string())
            .as_deref(),
        Some("billing.invoice.InvoiceStateConflict")
    );
    assert!(again[0].events.is_empty(), "a refusal emits nothing");
    assert_eq!(again[0].next, issued[0].next, "a refusal changes nothing");
}

#[test]
fn an_external_outcome_is_one_more_allowed_outcome_and_never_the_interpreters_pick() {
    let ir = billing_model();
    let input = BTreeMap::from([
        ("recipient".to_owned(), Node::Text("recipient".to_owned())),
        ("template".to_owned(), Node::Text("template".to_owned())),
    ]);
    let outcomes = |externals: &Externals| -> BTreeSet<String> {
        execute(
            &ir,
            &Store::default(),
            &name("billing.email.SendEmail"),
            &input,
            externals,
        )
        .expect("the model determines SendEmail")
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), ToString::to_string)
        })
        .collect()
    };
    assert_eq!(
        outcomes(&Externals::Withheld),
        BTreeSet::from(["billing.email.SendEmail/sent".to_owned()]),
        "nothing outside decided anything, so the external branch is not taken"
    );
    assert_eq!(
        outcomes(&Externals::Open),
        BTreeSet::from([
            "billing.email.SendEmail/failed".to_owned(),
            "billing.email.SendEmail/sent".to_owned(),
        ]),
        "a checker searching histories sees both"
    );
    assert_eq!(
        outcomes(&Externals::Forced(
            "failed".parse().expect("an outcome name")
        )),
        BTreeSet::from(["billing.email.SendEmail/failed".to_owned()]),
        "a scenario that forces the branch gets it"
    );
}

#[test]
fn a_command_naming_an_invoice_nobody_created_gets_the_declared_branch_and_nothing_invented() {
    let ir = billing_model();
    let steps = execute(
        &ir,
        &Store::default(),
        &name("billing.invoice.PayInvoice"),
        &BTreeMap::from([
            (
                "invoice_id".to_owned(),
                Node::Text("00000000-0000-4000-8000-999999999999".to_owned()),
            ),
            ("amount".to_owned(), money(3)),
        ]),
        &Externals::Withheld,
    )
    .expect("the model determines PayInvoice");
    assert_eq!(steps.len(), 1);
    let step = &steps[0];
    assert_eq!(
        step.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("billing.invoice.PayInvoice/wrong-state")
    );
    let error = step.error.as_ref().expect("the branch carries its error");
    assert!(
        error.fields.is_empty(),
        "no field of the error is determined by the model, so none is invented: {:?}",
        error.fields
    );
}
