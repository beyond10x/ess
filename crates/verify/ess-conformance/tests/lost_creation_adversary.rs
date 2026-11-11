//! Adversary cases for `story:rust-recorder-does-not-lose-a-creation`: which unanswered creations
//! leave a row out of a recorded read, and which rows stay, as `exploreWrittenRows` (Go) and
//! `writtenRows` (TypeScript) write them.
//!
//! The views here answer one extra invoice no operation created (`PHANTOM`), so every case can
//! tell a row left out from a row kept.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::history::{self, History, Operation};
use ess_conformance::linearize::{self, Anomaly, Checked, DEFAULT_BUDGET};
use ess_conformance::record::{Call, Interleaved, Subject};
use ess_conformance::reference::Billing;
use ess_conformance::scenario::SuiteProvenance;
use ess_conformance::sessions::{self, Act};
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const INVOICE_BY_ID: &str = "billing.invoice.InvoiceById";
const CREATE_INVOICE: &str = "billing.invoice.CreateInvoice";
const OPEN_ACCOUNT: &str = "billing.invoice.OpenAccount";
const LOST_EMAIL: &str = "lost@example.com";
const PHANTOM: &str = "00000000-0000-4000-8000-0000000fffff";
const SEEDS: u64 = 64;

/// `examples/billing`, with `OpenAccount` (creating a `billing.invoice.Account`) added when
/// `with_account` holds.
fn billing_model(with_account: bool) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
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
        let mut text = std::fs::read_to_string(&path).expect("readable");
        if with_account {
            text = with_open_account(&label, &text);
        }
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

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn with_open_account(label: &str, text: &str) -> String {
    match label {
        "domains/invoice.yaml" => replaced(
            &replaced(
                text,
                "\ncommands:\n",
                "\ncommands:\n  - name: billing.invoice.OpenAccount\n    input:\n      - {name: \
                 display_name, type: String}\n    outcomes:\n      - name: opened\n        \
                 creates: billing.invoice.Account\n        instance: account_id\n        sets: \
                 {display_name: input.display_name}\n        emits: \
                 [billing.invoice.AccountOpened]\n\n",
            ),
            "\nevents:\n",
            "\nevents:\n  - name: billing.invoice.AccountOpened\n    fields:\n      - {name: \
             account_id, type: billing.invoice.AccountId}\n\n",
        ),
        "components.yaml" => replaced(
            &replaced(
                text,
                "        - billing.invoice.CancelInvoice\n",
                "        - billing.invoice.CancelInvoice\n        - billing.invoice.OpenAccount\n",
            ),
            "        - billing.invoice.InvoiceCancelled\n",
            "        - billing.invoice.InvoiceCancelled\n        - billing.invoice.AccountOpened\n",
        ),
        _ => text.to_owned(),
    }
}

fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the recorded history is admitted: {refusal}"))
}

fn money(amount: f64) -> Node {
    Node::Map(BTreeMap::from([
        (
            "amount".to_owned(),
            Node::Number(Number::new(amount).expect("finite")),
        ),
        (
            "currency".to_owned(),
            Node::Text("amount.currency".to_owned()),
        ),
    ]))
}

fn create_invoice(email: &str, amount: f64) -> Call {
    Call::new(
        CREATE_INVOICE,
        BTreeMap::from([
            ("amount".to_owned(), money(amount)),
            (
                "account_id".to_owned(),
                Node::Text("00000000-0000-4000-8000-0000000000aa".to_owned()),
            ),
            ("customer_email".to_owned(), Node::Text(email.to_owned())),
        ]),
        Subject::Creates,
    )
}

fn open_account() -> Call {
    Call::new(
        OPEN_ACCOUNT,
        BTreeMap::from([("display_name".to_owned(), Node::Text("someone".to_owned()))]),
        Subject::Creates,
    )
}

fn read() -> Act {
    Act::Read(INVOICE_BY_ID.to_owned())
}

/// Executes every command; a `CreateInvoice` for [`LOST_EMAIL`] runs `executions` times and an
/// `OpenAccount` none, and both answer an error: the answer is lost.
struct Loses<'t> {
    inner: &'t Billing,
    executions: usize,
}

impl Interleaved for Loses<'_> {
    type Pending = SemanticCommandRequest;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        request
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        let email = pending.input.get("customer_email").and_then(Node::as_text);
        if pending.input.contains_key("display_name") {
            return Err(TargetError::unavailable(OPEN_ACCOUNT, "lost"));
        }
        if email != Some(LOST_EMAIL) {
            return self.inner.execute_command(pending);
        }
        for _ in 0..self.executions {
            let _ = self.inner.execute_command(pending.clone());
        }
        Err(TargetError::unavailable(CREATE_INVOICE, "lost"))
    }
}

/// Executes a command when it is sent and answers it when the client receives the answer.
struct ExecutesAtInvoke<'t>(&'t Billing);

impl Interleaved for ExecutesAtInvoke<'_> {
    type Pending = Result<SemanticCommandResult, TargetError>;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        self.0.execute_command(request)
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        pending
    }
}

/// The reference's views, with `PHANTOM` appended to every answer when `phantom` holds.
struct Views<'t> {
    inner: &'t Billing,
    phantom: bool,
}

impl ConformanceTarget for Views<'_> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("phantom-views", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        if self.phantom {
            result.rows.push(BTreeMap::from([(
                "invoice_id".to_owned(),
                Node::Text(PHANTOM.to_owned()),
            )]));
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
}

/// Records `clients` against [`Loses`] at `seed`, through the reader, and checks it.
fn run(
    model: &EssIr,
    executions: usize,
    phantom: bool,
    clients: Vec<Vec<Act>>,
    seed: u64,
) -> (History, Checked) {
    let reference = Billing::with_lag(0);
    let target = Loses {
        inner: &reference,
        executions,
    };
    let views = Views {
        inner: &reference,
        phantom,
    };
    let workload = sessions::Workload {
        prefix: Vec::new(),
        clients,
    };
    let recorded = sessions::record(model, &target, &views, &workload, seed).expect("recorded");
    let history = through_the_reader(model, &recorded);
    let checked = linearize::check(model, &history, DEFAULT_BUDGET).expect("checked");
    (history, checked)
}

fn the_read(history: &History) -> &Operation {
    history
        .operations
        .iter()
        .find(|operation| operation.command.as_str() == INVOICE_BY_ID)
        .expect("the read")
}

fn by_client(history: &History, client: u64) -> &Operation {
    history
        .operations
        .iter()
        .find(|operation| operation.client == client)
        .expect("the client's operation")
}

fn rows(read: &Operation) -> Vec<String> {
    read.rows.clone().expect("rows")
}

fn future_read_of(checked: &Checked) -> Option<String> {
    checked
        .read
        .as_ref()
        .filter(|violation| violation.anomaly == Anomaly::FutureRead)
        .map(|violation| violation.subject_key.clone())
}

/// A lost creation of another entity (`Account`) accounts for no invoice row: Go and TypeScript
/// count only creations whose entity is the view's source.
#[test]
fn a_lost_creation_of_another_entity_leaves_no_row_of_the_view_out() {
    let model = billing_model(true);
    let mut overlapping = 0;
    for seed in 0..SEEDS {
        let (history, checked) = run(
            &model,
            0,
            true,
            vec![vec![Act::Call(open_account())], vec![read()]],
            seed,
        );
        let open = by_client(&history, 0);
        assert_eq!(open.command.as_str(), OPEN_ACCOUNT);
        assert_eq!(open.returned_at, None);
        let read = the_read(&history);
        overlapping += usize::from(open.invoked_at < read.returned_at.expect("answered"));
        assert_eq!(rows(read), vec![PHANTOM.to_owned()], "seed {seed}");
        assert_eq!(
            future_read_of(&checked).as_deref(),
            Some(PHANTOM),
            "seed {seed}: {checked:?}"
        );
    }
    assert!(
        overlapping > 0,
        "no seed invoked the account before the read returned"
    );
}

/// A lost creation invoked after the read returned accounts for no row of that read.
#[test]
fn a_lost_creation_invoked_after_the_read_returned_leaves_no_row_out() {
    let model = billing_model(false);
    let (mut after, mut before) = (0, 0);
    for seed in 0..SEEDS {
        let (history, checked) = run(
            &model,
            0,
            true,
            vec![
                vec![read()],
                vec![Act::Call(create_invoice(LOST_EMAIL, 5.0))],
            ],
            seed,
        );
        let lost = by_client(&history, 1);
        let read = the_read(&history);
        if lost.invoked_at > read.returned_at.expect("answered") {
            after += 1;
            assert_eq!(rows(read), vec![PHANTOM.to_owned()], "seed {seed}");
            assert_eq!(
                future_read_of(&checked).as_deref(),
                Some(PHANTOM),
                "seed {seed}"
            );
        } else {
            before += 1;
            assert_eq!(
                rows(read),
                Vec::<String>::new(),
                "seed {seed}: one row left out"
            );
            assert!(checked.read.is_none(), "seed {seed}: {checked:?}");
        }
    }
    assert!(after > 0 && before > 0, "after {after}, before {before}");
}

/// A creation that was answered, with no subject (refused), accounts for no row.
#[test]
fn a_refused_creation_leaves_no_row_out() {
    let model = billing_model(false);
    let mut overlapping = 0;
    for seed in 0..SEEDS {
        let (history, checked) = run(
            &model,
            0,
            true,
            vec![
                vec![Act::Call(create_invoice("payer@example.com", -5.0))],
                vec![read()],
            ],
            seed,
        );
        let refused = by_client(&history, 0);
        assert!(refused.returned_at.is_some() && refused.subject_key.is_empty());
        let read = the_read(&history);
        overlapping += usize::from(refused.invoked_at < read.returned_at.expect("answered"));
        assert_eq!(rows(read), vec![PHANTOM.to_owned()], "seed {seed}");
        assert_eq!(
            future_read_of(&checked).as_deref(),
            Some(PHANTOM),
            "seed {seed}"
        );
    }
    assert!(
        overlapping > 0,
        "no seed sent the refused creation before the read returned"
    );
}

/// A row an operation names is written even while a lost creation has a row to account for.
#[test]
fn a_named_row_is_written_beside_a_lost_creation() {
    let model = billing_model(false);
    let mut both = 0;
    for seed in 0..SEEDS {
        let (history, checked) = run(
            &model,
            0,
            false,
            vec![
                vec![Act::Call(create_invoice("payer@example.com", 5.0))],
                vec![Act::Call(create_invoice(LOST_EMAIL, 5.0))],
                vec![read()],
            ],
            seed,
        );
        let created = by_client(&history, 0);
        let lost = by_client(&history, 1);
        let read = the_read(&history);
        assert_ne!(created.subject_key, "");
        assert!(checked.read.is_none(), "seed {seed}: {checked:?}");
        if created.returned_at.expect("answered") < read.invoked_at {
            assert_eq!(rows(read), vec![created.subject_key.clone()], "seed {seed}");
            both += usize::from(lost.invoked_at < read.returned_at.expect("answered"));
        }
    }
    assert!(
        both > 0,
        "no seed had the named row and the lost creation before the read"
    );
}

/// The first unnamed row in answer order is the one left out; the phantom, answered last, stays
/// and is the future read.
#[test]
fn the_first_unnamed_row_is_left_out_and_the_phantom_is_reported() {
    let model = billing_model(false);
    let mut shown = 0;
    for seed in 0..SEEDS {
        let (history, checked) = run(
            &model,
            1,
            true,
            vec![
                vec![Act::Call(create_invoice(LOST_EMAIL, 5.0))],
                vec![read()],
            ],
            seed,
        );
        let lost = by_client(&history, 0);
        let read = the_read(&history);
        let written = rows(read);
        if written.is_empty() {
            assert!(
                lost.invoked_at < read.returned_at.expect("answered"),
                "seed {seed}"
            );
            assert!(checked.read.is_none(), "seed {seed}: {checked:?}");
            continue;
        }
        assert_eq!(written, vec![PHANTOM.to_owned()], "seed {seed}");
        assert_eq!(
            future_read_of(&checked).as_deref(),
            Some(PHANTOM),
            "seed {seed}"
        );
        shown += usize::from(lost.invoked_at < read.returned_at.expect("answered"));
    }
    assert!(shown > 0, "no seed showed both rows and left one out");
}

/// Two lost creations account for two rows: the one row they made is left out.
#[test]
fn two_lost_creations_and_one_row_write_no_row() {
    let model = billing_model(false);
    for seed in 0..SEEDS {
        let reference = Billing::with_lag(0);
        let once = Loses {
            inner: &reference,
            executions: 1,
        };
        let workload = sessions::Workload {
            prefix: Vec::new(),
            clients: vec![
                vec![Act::Call(create_invoice(LOST_EMAIL, 5.0))],
                vec![Act::Call(open_free_lost())],
                vec![read()],
            ],
        };
        let recorded =
            sessions::record(&model, &once, &reference, &workload, seed).expect("recorded");
        let history = through_the_reader(&model, &recorded);
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        assert!(rows(the_read(&history)).len() <= 1, "seed {seed}");
        assert_eq!(future_read_of(&checked), None, "seed {seed}: {checked:?}");
    }
}

/// A second lost `CreateInvoice`, which [`Loses`] answers with an error and never executes.
fn open_free_lost() -> Call {
    let mut call = create_invoice(LOST_EMAIL, 5.0);
    call.input
        .insert("display_name".to_owned(), Node::Text("never".to_owned()));
    call
}

/// A creation answered after the read returned names its row: the read showing it keeps it, and
/// it is no future read.
#[test]
fn a_creation_answered_after_the_read_returned_names_the_row_it_showed() {
    let model = billing_model(false);
    let mut late = 0;
    for seed in 0..SEEDS {
        let reference = Billing::with_lag(0);
        let workload = sessions::Workload {
            prefix: Vec::new(),
            clients: vec![
                vec![Act::Call(create_invoice("payer@example.com", 5.0))],
                vec![read()],
            ],
        };
        let recorded = sessions::record(
            &model,
            &ExecutesAtInvoke(&reference),
            &reference,
            &workload,
            seed,
        )
        .expect("recorded");
        let history = through_the_reader(&model, &recorded);
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        let created = by_client(&history, 0);
        let read = the_read(&history);
        assert!(checked.read.is_none(), "seed {seed}: {checked:?}");
        if created.invoked_at < read.invoked_at
            && created.returned_at.expect("answered") > read.returned_at.expect("answered")
        {
            assert_eq!(rows(read), vec![created.subject_key.clone()], "seed {seed}");
            late += 1;
        }
    }
    assert!(
        late > 0,
        "no seed answered the creation after the read returned"
    );
}
