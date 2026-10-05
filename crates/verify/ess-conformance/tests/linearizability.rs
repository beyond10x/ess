//! A recorded history is checked for linearizability against the interpreter, and shrunk
//! (`story:linearizability-checker-over-the-interpreter`).
//!
//! The acceptance, case by case:
//!
//! * a recorded two-client history against `Fault::LostUpdate` is `Violation`;
//! * the same workload, recorded against the unfaulted `Billing`, is `Linearizable`;
//! * the shrunk violation holds at most 2 clients and 4 operations and is a `Violation` again;
//! * the committed register histories under `tests/fixtures/register/` are judged as their names say;
//! * the verdict and the shrunk history are a function of the history and the budget.
//!
//! Exit codes and the command line are `crates/edge/ess-cli/tests/check_history.rs`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, History, Verdict};
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

const REGISTER: &str = include_str!("fixtures/register/register.yaml");

/// The register fixture: one cell holding zero or one, a write and two reads.
fn register_model() -> EssIr {
    let raw = RawSpecFile::parse(REGISTER).expect("the register fixture parses");
    let specification = Specification::assemble([(Source::new("register.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the register fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the register fixture resolves:\n{diagnostics}"))
}

/// A committed register history, admitted by the reader exactly as `check-history` admits it.
fn register_history(name: &str) -> History {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/register")
            .join(name),
    )
    .unwrap_or_else(|error| panic!("{name} is committed: {error}"));
    history::read(&bytes, &SuiteProvenance::of(&register_model()).spec_digest)
        .unwrap_or_else(|refusal| panic!("{name} is admitted: {refusal}"))
}

/// A recorded history, written out and read back, so what is checked is what a runner would hand
/// the checker rather than a value the reader never saw.
fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the recorded history is admitted: {refusal}"))
}

/// The seed whose interleaving overlaps the two payments.
const OVERLAPPING: u64 = 0;

fn lost_update(ir: &EssIr, seed: u64) -> History {
    let target = faulty::billing(Fault::LostUpdate);
    let recorded = record::record(ir, &target, &faulty::lost_update_workload(), seed)
        .expect("the workload names what billing declares");
    through_the_reader(ir, &recorded)
}

fn unfaulted(ir: &EssIr, seed: u64) -> History {
    let target = Billing::new();
    let recorded = record::record(ir, &Atomic(&target), &faulty::lost_update_workload(), seed)
        .expect("the workload names what billing declares");
    through_the_reader(ir, &recorded)
}

fn verdict(ir: &EssIr, history: &History) -> Verdict {
    linearize::check(ir, history, DEFAULT_BUDGET)
        .unwrap_or_else(|refusal| panic!("the history is checked: {refusal:?}"))
        .verdict
}

/// `true` when the two payments were both in flight at once.
fn payments_overlap(history: &History) -> bool {
    let pays: Vec<_> = history
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == "billing.invoice.PayInvoice")
        .collect();
    assert_eq!(pays.len(), 2, "the workload pays twice");
    let ends = |index: usize| pays[index].returned_at.expect("both payments answered");
    pays[0].invoked_at < ends(1) && pays[1].invoked_at < ends(0)
}

// ---- the committed register histories ----------------------------------------------------------

#[test]
fn a_register_history_no_order_explains_is_a_violation() {
    // Herlihy & Wing 1990, Fig. 1's shape: once a read has seen the write, a later read by the
    // same client cannot see the value from before it. No order of the four calls answers both.
    let model = register_model();
    assert_eq!(
        verdict(&model, &register_history("not-linearizable.json")),
        Verdict::Violation
    );
}

#[test]
fn a_register_history_an_order_explains_is_linearizable() {
    // The same calls with the reads the other way round: the first read comes before the write
    // takes effect and the second after it. It also carries a write that never answered, which is
    // read as finishing after every other call (design decision 4) and so is free to have taken
    // effect or not.
    let model = register_model();
    assert_eq!(
        verdict(&model, &register_history("linearizable.json")),
        Verdict::Linearizable
    );
}

// ---- LostUpdate against the unfaulted reference ------------------------------------------------

#[test]
fn a_recorded_two_client_history_against_lost_update_is_a_violation() {
    let model = billing_model();
    let history = lost_update(&model, OVERLAPPING);
    assert_eq!(history.clients, 2);
    assert!(
        payments_overlap(&history),
        "seed {OVERLAPPING} is the one pinned for its overlapping payments: {history:#?}"
    );
    assert_eq!(verdict(&model, &history), Verdict::Violation);
}

#[test]
fn the_same_recording_against_the_unfaulted_reference_is_linearizable() {
    let model = billing_model();
    let history = unfaulted(&model, OVERLAPPING);
    assert!(payments_overlap(&history));
    assert_eq!(verdict(&model, &history), Verdict::Linearizable);
}

#[test]
fn lost_update_is_a_violation_exactly_when_the_payments_overlap() {
    // The fault is a race, so what decides it is the interleaving and nothing else: over a run of
    // seeds, every history where the two payments overlap is a violation, every one where they do
    // not is linearizable, and the unfaulted reference is linearizable under all of them.
    let model = billing_model();
    let mut overlapping = 0;
    let mut apart = 0;
    for seed in 0..24 {
        let faulted = lost_update(&model, seed);
        let expected = if payments_overlap(&faulted) {
            overlapping += 1;
            Verdict::Violation
        } else {
            apart += 1;
            Verdict::Linearizable
        };
        assert_eq!(verdict(&model, &faulted), expected, "seed {seed}");
        assert_eq!(
            verdict(&model, &unfaulted(&model, seed)),
            Verdict::Linearizable,
            "seed {seed} against the reference"
        );
    }
    assert!(
        overlapping > 0 && apart > 0,
        "the seeds exercise both sides: {overlapping} overlapping, {apart} apart"
    );
}

// ---- shrinking --------------------------------------------------------------------------------

#[test]
fn the_shrunk_lost_update_violation_is_small_and_still_a_violation() {
    let model = billing_model();
    let history = lost_update(&model, OVERLAPPING);
    let shrunk = linearize::shrink(&model, &history, DEFAULT_BUDGET).expect("the history shrinks");

    let clients: BTreeSet<u64> = shrunk
        .operations
        .iter()
        .map(|operation| operation.client)
        .collect();
    assert!(
        clients.len() <= 2 && shrunk.operations.len() <= 4,
        "{} clients and {} operations: {shrunk:#?}",
        clients.len(),
        shrunk.operations.len()
    );
    // The race survives the shrink: both payments are still there. A shrinker that only asked for
    // "a violation" would drop the creation and report one payment of an invoice nobody created,
    // which is a violation and not this one.
    let payments = shrunk
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == "billing.invoice.PayInvoice")
        .count();
    assert_eq!(payments, 2, "{shrunk:#?}");
    assert_eq!(
        verdict(&model, &through_the_reader(&model, &shrunk)),
        Verdict::Violation,
        "a shrunk history is itself a violation when it is checked again, as the reader reads it"
    );
}

#[test]
fn the_verdict_and_the_shrunk_history_are_a_function_of_the_history_and_the_budget() {
    let model = billing_model();
    let history = lost_update(&model, OVERLAPPING);
    let first = linearize::report(&model, &history, DEFAULT_BUDGET).expect("checked");
    let second = linearize::report(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(first.to_json(), second.to_json());
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.verdict, Verdict::Violation);
    assert!(
        first.shrunk.is_some(),
        "a violation reports its shrunk history"
    );
    assert!(
        !first.linearization.is_empty(),
        "a violation names the longest partial linearization it found"
    );
}

#[test]
fn shrinking_drops_what_the_violation_does_not_need_and_keeps_what_it_does() {
    // The LostUpdate recording with a second invoice beside it, created and issued in the prefix,
    // neither of which the race on the first invoice needs. Its partition is dropped whole; the
    // Fig. 1 register history needs every one of its four calls and keeps them.
    let model = billing_model();
    let mut workload = faulty::lost_update_workload();
    let mut noise = workload.prefix.clone();
    noise[1].subject = record::Subject::Created(2);
    workload.prefix.extend(noise);
    let target = faulty::billing(Fault::LostUpdate);
    let history = through_the_reader(
        &model,
        &record::record(&model, &target, &workload, OVERLAPPING).expect("recorded"),
    );
    assert_eq!(verdict(&model, &history), Verdict::Violation);
    let shrunk = linearize::shrink(&model, &history, DEFAULT_BUDGET).expect("shrunk");
    assert_eq!(history.operations.len(), 6);
    assert_eq!(shrunk.operations.len(), 4, "{shrunk:#?}");
    let subjects: BTreeSet<&str> = shrunk
        .operations
        .iter()
        .map(|operation| operation.subject_key.as_str())
        .collect();
    assert_eq!(
        subjects.len(),
        1,
        "one invoice's partition is left: {shrunk:#?}"
    );

    let register = register_model();
    let fig1 = register_history("not-linearizable.json");
    let shrunk = linearize::shrink(&register, &fig1, DEFAULT_BUDGET).expect("shrunk");
    assert_eq!(shrunk.operations, fig1.operations);
}

// ---- the budget -------------------------------------------------------------------------------

#[test]
fn a_search_that_runs_out_of_budget_is_unknown_and_never_a_pass() {
    let model = billing_model();
    let history = unfaulted(&model, OVERLAPPING);
    let checked = linearize::check(&model, &history, 1).expect("checked");
    assert_eq!(checked.verdict, Verdict::Unknown);
    let report = linearize::report(&model, &history, 1).expect("checked");
    assert_eq!(report.exit_code(), 3);
    assert!(report.shrunk.is_none(), "only a violation is shrunk");
}

#[test]
fn the_exit_code_is_the_verdict() {
    let model = register_model();
    let yes = linearize::report(
        &model,
        &register_history("linearizable.json"),
        DEFAULT_BUDGET,
    )
    .expect("checked");
    let no = linearize::report(
        &model,
        &register_history("not-linearizable.json"),
        DEFAULT_BUDGET,
    )
    .expect("checked");
    assert_eq!((yes.exit_code(), no.exit_code()), (0, 1));
}

// ---- views ------------------------------------------------------------------------------------

#[test]
fn a_read_of_a_view_not_declared_current_is_listed_and_not_judged() {
    // `examples/billing` declares `InvoiceById` eventual. A read of it is a question for
    // `story:session-and-eventual-view-checks`; here it is named as not judged, and it cannot turn
    // a linearizable history into anything else.
    let model = billing_model();
    let mut history = unfaulted(&model, OVERLAPPING);
    let mut read = history.operations[0].clone();
    read.operation_id =
        history::Uuid::new("00000000-0000-4000-8000-0000000000ff").expect("a canonical UUID");
    read.command = history::QualifiedName::new("billing.invoice.InvoiceById").expect("a name");
    read.outcome = Some(history::QualifiedName::new("read").expect("a name"));
    history.operations.push(read);
    let history = through_the_reader(&model, &history);

    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(checked.verdict, Verdict::Linearizable);
    assert_eq!(checked.not_judged.len(), 1);
    assert_eq!(
        checked.not_judged[0].operation_id,
        "00000000-0000-4000-8000-0000000000ff"
    );
    assert_eq!(checked.not_judged[0].consistency, "eventual");
}

/// What a specification promises when two commands race, stated on the concepts page beside the
/// tests above that hold it, and linked from the two verify guides that rely on it.
#[test]
fn concepts_page_states_the_race_promise() {
    let docs = root().join("website/docs");
    let page = std::fs::read_to_string(docs.join("concepts/ess.md")).expect("concepts/ess.md");
    let headings: Vec<&str> = page.lines().filter(|l| l.starts_with("## ")).collect();
    let derived = headings
        .iter()
        .position(|h| *h == "## What gets derived")
        .expect("concepts/ess.md keeps `## What gets derived`");
    let race = headings
        .iter()
        .position(|h| *h == "## When commands race")
        .expect("concepts/ess.md has the heading `## When commands race`");
    assert!(
        race > derived,
        "`## When commands race` comes after `## What gets derived`"
    );
    let start = page.find("\n## When commands race\n").expect("heading") + 1;
    let rest = &page[start..];
    let section = &rest[..rest[3..].find("\n## ").map_or(rest.len(), |end| end + 3)];
    for phrase in [
        "one sequential order",
        "per subject",
        "lock local to one process",
        "`check-history`",
        "a single-client suite cannot see a race",
        "subject by subject",
        "set effect",
        "shared partition",
        "unanswered call",
    ] {
        assert!(
            section.contains(phrase),
            "`## When commands race` is missing {phrase}"
        );
    }
    for guide in [
        "guides/verify/one-time-responses.md",
        "guides/verify/explore.md",
    ] {
        let text = std::fs::read_to_string(docs.join(guide)).expect("guide");
        assert!(
            text.contains("../../concepts/ess.md#when-commands-race"),
            "{guide} links ../../concepts/ess.md#when-commands-race"
        );
    }
    let explore = std::fs::read_to_string(docs.join("guides/verify/explore.md")).expect("explore");
    let start = explore
        .find("\n## Check a concurrent history\n")
        .expect("explore.md keeps `## Check a concurrent history`")
        + 1;
    let rest = &explore[start..];
    let section = &rest[..rest[3..].find("\n## ").map_or(rest.len(), |end| end + 3)];
    assert!(
        section.contains("../../concepts/ess.md#when-commands-race"),
        "the `Check a concurrent history` section links the race promise"
    );
}
