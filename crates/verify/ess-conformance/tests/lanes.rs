//! A checked history is drawn as client lanes (`story:concurrent-history-lanes`).
//!
//! The acceptance, case by case:
//!
//! * the page for the shrunk `LostUpdate` history names the two conflicting operations and the
//!   state each linearization would have required;
//! * rendering the same history twice gives identical bytes.
//!
//! And the outcome around it: one lane per client, each operation's invoke–return interval, the
//! linearization points found, the operation where the search failed, and nothing fetched from
//! outside the page. The command line is `crates/edge/ess-cli/tests/conform_web_history.rs`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, History, Verdict};
use ess_conformance::lanes;
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::record::{self, Atomic};
use ess_conformance::reference::Billing;
use ess_conformance::scenario::SuiteProvenance;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

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
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("billing validates");
    compile(&specification, &sources).expect("billing resolves")
}

/// The seed whose interleaving overlaps the two payments.
const OVERLAPPING: u64 = 0;

const PAY: &str = "billing.invoice.PayInvoice";

fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the recorded history is admitted: {refusal}"))
}

fn lost_update(ir: &EssIr) -> History {
    let recorded = record::record(
        ir,
        &faulty::billing(Fault::LostUpdate),
        &faulty::lost_update_workload(),
        OVERLAPPING,
    )
    .expect("the workload names what billing declares");
    through_the_reader(ir, &recorded)
}

fn shrunk_lost_update(ir: &EssIr) -> History {
    let shrunk =
        linearize::shrink(ir, &lost_update(ir), DEFAULT_BUDGET).expect("the history shrinks");
    through_the_reader(ir, &shrunk)
}

fn unfaulted(ir: &EssIr) -> History {
    let reference = Billing::new();
    let recorded = record::record(
        ir,
        &Atomic(&reference),
        &faulty::lost_update_workload(),
        OVERLAPPING,
    )
    .expect("the workload names what billing declares");
    through_the_reader(ir, &recorded)
}

fn page(ir: &EssIr, history: &History) -> String {
    lanes::render(ir, history, DEFAULT_BUDGET)
        .unwrap_or_else(|refusal| panic!("the history renders: {refusal}"))
}

fn pay_ids(history: &History) -> BTreeSet<String> {
    history
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == PAY)
        .map(|operation| operation.operation_id.as_str().to_owned())
        .collect()
}

/// Every `<li class="conflict" …>` of the page, as (operation, required, supplied).
fn conflicts(page: &str) -> Vec<(String, String, String)> {
    let attribute = |tag: &str, name: &str| -> String {
        let key = format!("{name}=\"");
        let start = tag
            .find(&key)
            .unwrap_or_else(|| panic!("`{tag}` carries `{name}`"))
            + key.len();
        tag[start..]
            .split('"')
            .next()
            .expect("a closed attribute")
            .to_owned()
    };
    page.split("<li class=\"conflict\"")
        .skip(1)
        .map(|rest| {
            let tag = rest.split('>').next().expect("a closed tag");
            (
                attribute(tag, "data-operation"),
                attribute(tag, "data-required"),
                attribute(tag, "data-supplied"),
            )
        })
        .collect()
}

// ---- the acceptance ------------------------------------------------------------------------------

#[test]
fn the_page_for_the_shrunk_lost_update_names_both_payments_and_the_state_each_order_needed() {
    let model = billing_model();
    let shrunk = shrunk_lost_update(&model);
    assert_eq!(
        linearize::check(&model, &shrunk, DEFAULT_BUDGET)
            .expect("checked")
            .verdict,
        Verdict::Violation
    );
    let page = page(&model, &shrunk);

    let found = conflicts(&page);
    let named: BTreeSet<String> = found.iter().map(|(id, _, _)| id.clone()).collect();
    assert_eq!(
        named,
        pay_ids(&shrunk),
        "the conflict names exactly the two payments: {found:?}\n{page}"
    );
    // Each payment answered `settled`, which the model allows only from `Issued`; whichever of the
    // two an order puts second finds the invoice already `Paid`.
    for (id, required, supplied) in &found {
        assert_eq!(required, "Issued", "payment {id} needed an Issued invoice");
        assert_eq!(supplied, "Paid", "the order gave payment {id} a Paid one");
    }
    assert!(page.contains("Violation"), "the page states the verdict");
}

#[test]
fn rendering_the_same_history_twice_gives_identical_bytes() {
    let model = billing_model();
    for history in [
        shrunk_lost_update(&model),
        lost_update(&model),
        unfaulted(&model),
    ] {
        assert_eq!(page(&model, &history), page(&model, &history));
    }
}

// ---- the outcome around it -------------------------------------------------------------------

#[test]
fn the_conflict_accessor_names_the_two_payments_and_their_states() {
    let model = billing_model();
    let shrunk = shrunk_lost_update(&model);
    let checked = linearize::check(&model, &shrunk, DEFAULT_BUDGET).expect("checked");
    let conflict = linearize::conflict(&model, &shrunk, &checked)
        .expect("the conflict is computed")
        .expect("a command violation has a conflict");
    let sides: BTreeSet<String> = [&conflict.failing, &conflict.against]
        .iter()
        .map(|side| side.operation_id.clone())
        .collect();
    assert_eq!(sides, pay_ids(&shrunk));
    assert!(
        !checked
            .linearization
            .contains(&conflict.failing.operation_id),
        "the failing payment is the one the longest order could not place"
    );
    for side in [&conflict.failing, &conflict.against] {
        assert_eq!(side.required, vec!["Issued".to_owned()]);
        assert_eq!(side.supplied, vec!["Paid".to_owned()]);
    }
}

#[test]
fn the_page_draws_one_lane_per_client_and_fetches_nothing_from_outside() {
    let model = billing_model();
    for history in [
        shrunk_lost_update(&model),
        lost_update(&model),
        unfaulted(&model),
    ] {
        let page = page(&model, &history);
        let clients: BTreeSet<u64> = history.operations.iter().map(|it| it.client).collect();
        // The shrunk section, where there is one, draws its own lanes; count only the first.
        let first = page
            .split("<section class=\"history\"")
            .nth(1)
            .expect("the page draws the history");
        assert_eq!(
            first.matches("<g class=\"lane\"").count(),
            clients.len(),
            "one lane per client:\n{page}"
        );
        assert_eq!(
            first.matches("<rect class=\"op").count(),
            history.operations.len(),
            "one interval per operation"
        );
        for fetch in [
            "src=", "href=", "http:", "https:", "@import", "url(", "<script",
        ] {
            assert!(!page.contains(fetch), "the page fetches with `{fetch}`");
        }
    }
}

#[test]
fn a_linearizable_history_shows_a_linearization_point_for_every_operation() {
    let model = billing_model();
    let history = unfaulted(&model);
    let page = page(&model, &history);
    assert!(page.contains("Linearizable"));
    assert_eq!(
        page.matches("<circle class=\"point\"").count(),
        history.operations.len(),
        "{page}"
    );
    assert!(!page.contains("<li class=\"conflict\""));
}

#[test]
fn a_failure_no_two_operations_explain_names_no_pair_and_still_marks_where_the_search_failed() {
    // The register history: a read of one returns before a read of zero is invoked, so the read of
    // zero cannot be moved before the write that the read of one saw. Swapping it with the write
    // alone would break the recorded instants; the page claims no pair it cannot stand behind.
    let raw = RawSpecFile::parse(include_str!("fixtures/register/register.yaml"))
        .expect("the register fixture parses");
    let specification = Specification::assemble([(Source::new("register.yaml"), raw)])
        .expect("the register fixture validates");
    let model = compile(&specification, &SourceMap::new()).expect("the register fixture resolves");
    let history = history::read(
        include_bytes!("fixtures/register/not-linearizable.json"),
        &SuiteProvenance::of(&model).spec_digest,
    )
    .expect("the register history is admitted");
    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(checked.verdict, Verdict::Violation);
    assert_eq!(
        linearize::conflict(&model, &history, &checked).expect("computed"),
        None
    );
    let page = page(&model, &history);
    assert!(conflicts(&page).is_empty(), "{page}");
    assert_eq!(
        page.matches("<rect class=\"op failing\"").count(),
        1,
        "{page}"
    );
}

#[test]
fn a_violation_marks_the_operation_where_the_search_failed() {
    let model = billing_model();
    let shrunk = shrunk_lost_update(&model);
    let checked = linearize::check(&model, &shrunk, DEFAULT_BUDGET).expect("checked");
    let page = page(&model, &shrunk);
    let failing: Vec<&str> = page
        .split("<rect class=\"op failing\"")
        .skip(1)
        .map(|rest| rest.split('>').next().expect("a closed tag"))
        .collect();
    assert_eq!(failing.len(), 1, "{page}");
    let unplaced: Vec<String> = shrunk
        .operations
        .iter()
        .map(|operation| operation.operation_id.as_str().to_owned())
        .filter(|id| !checked.linearization.contains(id))
        .collect();
    assert_eq!(unplaced.len(), 1);
    assert!(
        failing[0].contains(&format!("data-operation=\"{}\"", unplaced[0])),
        "{}",
        failing[0]
    );
    // Only the operations the longest order placed carry a point.
    assert_eq!(
        page.matches("<circle class=\"point\"").count(),
        checked.linearization.len()
    );
}
