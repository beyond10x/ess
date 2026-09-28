//! Adversarial cases, second pass, for `story:linearizability-checker-over-the-interpreter`.
//!
//! Aimed at the pass-1 corrections — `Generated::Recorded` in the interpreter and `relabel` after
//! `shrink` — and at what pass 1 did not reach: identities a history reuses or never created, the
//! exact budget edge, and shrinking races recorded beside other subjects and other calls.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, Completion, History, ReturnBound, Verdict};
use ess_conformance::interpret::Interpreted;
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::record::{self, Atomic, Call, Subject, Workload};
use ess_conformance::reference::Billing;
use ess_conformance::scenario::SuiteProvenance;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const REGISTER: &str = include_str!("fixtures/register/register.yaml");

fn register_model() -> EssIr {
    let raw = RawSpecFile::parse(REGISTER).expect("the model parses");
    let specification = Specification::assemble([(Source::new("register.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn billing_model() -> EssIr {
    let base = root()
        .join("examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
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
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside")
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

fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the history is admitted: {refusal}"))
}

fn verdict(ir: &EssIr, history: &History, budget: u64) -> Verdict {
    linearize::check(ir, history, budget)
        .unwrap_or_else(|refusal| panic!("the history is checked: {refusal:?}"))
        .verdict
}

/// Two registers created one after the other by the model's own interpreter, then written.
fn two_registers() -> (EssIr, History) {
    let model = register_model();
    let target = Interpreted::for_model(model.clone());
    let workload = Workload {
        prefix: vec![
            Call::new(
                "register.cell.CreateRegister",
                BTreeMap::new(),
                Subject::Creates,
            ),
            Call::new(
                "register.cell.CreateRegister",
                BTreeMap::new(),
                Subject::Creates,
            ),
            Call::new(
                "register.cell.WriteOne",
                BTreeMap::new(),
                Subject::Created(1),
            ),
        ],
        clients: Vec::new(),
    };
    let recorded = record::record(&model, &Atomic(&target), &workload, 0).expect("recorded");
    let history = through_the_reader(&model, &recorded);
    assert_ne!(
        history.operations[0].subject_key, history.operations[1].subject_key,
        "the interpreter mints two identities: {history:#?}"
    );
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Linearizable,
        "the history as recorded is the model's own"
    );
    (model, history)
}

// ---- `Generated::Recorded`: identities a history reuses or never created ------------------------

#[test]
fn two_creations_that_both_published_one_identity_are_a_violation() {
    // `Recorded` uses a given slot as `Given` does, so an identity the store holds is not created
    // again. An implementation that hands out one identity twice is told apart from one that does
    // not.
    let (model, mut history) = two_registers();
    let first = history.operations[0].subject_key.clone();
    history.operations[1].subject_key.clone_from(&first);
    history.operations[2].subject_key = first;
    let history = through_the_reader(&model, &history);
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Violation
    );
}

#[test]
fn an_unanswered_creation_of_a_held_identity_may_never_have_happened() {
    let (model, mut history) = two_registers();
    let first = history.operations[0].subject_key.clone();
    history.operations[1].subject_key.clone_from(&first);
    history.operations[1].completion = Completion::Indeterminate;
    history.operations[1].returned_at = None;
    history.operations[1].outcome = None;
    history.operations[2].subject_key = first;
    let history = through_the_reader(&model, &history);
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Linearizable
    );
}

#[test]
fn a_write_to_an_identity_no_creation_published_is_a_violation() {
    // An absent slot is minted under `Recorded`; a write naming an identity nothing created must not
    // be explained by an instance the checker minted for it.
    let (model, mut history) = two_registers();
    history.operations[2].subject_key = "00000000-0000-4000-8000-0000000fffff".to_owned();
    let history = through_the_reader(&model, &history);
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Violation
    );
}

// ---- the budget edge -----------------------------------------------------------------------------

#[test]
fn the_budget_a_search_spent_is_exactly_enough_and_one_less_is_unknown() {
    let model = billing_model();
    for seed in 0..6 {
        let workload = faulty::lost_update_workload();
        let faulted = record::record(&model, &faulty::billing(Fault::LostUpdate), &workload, seed)
            .expect("recorded");
        let reference = Billing::new();
        let unfaulted =
            record::record(&model, &Atomic(&reference), &workload, seed).expect("recorded");
        for history in [faulted, unfaulted] {
            let history = through_the_reader(&model, &history);
            let full = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
            assert_ne!(full.verdict, Verdict::Unknown, "seed {seed}");
            let exact = linearize::check(&model, &history, full.steps).expect("checked");
            assert_eq!(
                (exact.verdict, exact.steps),
                (full.verdict, full.steps),
                "seed {seed}: the steps a search reports spending are enough for it"
            );
            let short = linearize::check(&model, &history, full.steps - 1).expect("checked");
            assert_eq!(short.verdict, Verdict::Unknown, "seed {seed}");
        }
    }
}

// ---- shrinking races recorded beside other subjects and other calls ------------------------------

/// Client 0's prefix creates and issues `invoices` invoices; each list is one client's payments,
/// naming invoices by their index.
fn payments(invoices: usize, clients: &[&[usize]]) -> Workload {
    let template = faulty::lost_update_workload();
    let pay = template.clients[0][0].clone();
    let mut prefix = Vec::new();
    for invoice in 0..invoices {
        prefix.push(template.prefix[0].clone());
        let mut issue = template.prefix[1].clone();
        issue.subject = Subject::Created(invoice * 2);
        prefix.push(issue);
    }
    Workload {
        prefix,
        clients: clients
            .iter()
            .map(|calls| {
                calls
                    .iter()
                    .map(|&invoice| {
                        let mut call = pay.clone();
                        call.subject = Subject::Created(invoice * 2);
                        call
                    })
                    .collect()
            })
            .collect(),
    }
}

/// `true` when no client of `history` has two calls in flight at once.
fn every_client_is_sequential(history: &History) -> bool {
    let mut by_client: BTreeMap<u64, Vec<(u64, ReturnBound)>> = BTreeMap::new();
    for operation in &history.operations {
        by_client
            .entry(operation.client)
            .or_default()
            .push((operation.invoked_at, operation.return_bound()));
    }
    by_client.values_mut().all(|calls| {
        calls.sort();
        calls
            .windows(2)
            .all(|pair| pair[0].1 < ReturnBound::At(pair[1].0))
    })
}

#[test]
fn every_lost_update_race_shrinks_to_two_clients_and_four_operations_whatever_else_was_recorded() {
    let model = billing_model();
    let workloads: [(&str, Workload); 4] = [
        ("two clients paying twice", payments(1, &[&[0, 0], &[0, 0]])),
        ("two invoices, both raced", payments(2, &[&[0, 1], &[0, 1]])),
        (
            "two invoices, three clients",
            payments(2, &[&[0], &[1], &[0, 1]]),
        ),
        (
            "three clients on one invoice",
            payments(1, &[&[0], &[0], &[0]]),
        ),
    ];
    let mut failures: Vec<String> = Vec::new();
    for (name, workload) in &workloads {
        let mut violations = 0;
        for seed in 0..24 {
            let history = through_the_reader(
                &model,
                &record::record(&model, &faulty::billing(Fault::LostUpdate), workload, seed)
                    .expect("recorded"),
            );
            if verdict(&model, &history, DEFAULT_BUDGET) != Verdict::Violation {
                continue;
            }
            violations += 1;
            let shrunk = linearize::shrink(&model, &history, DEFAULT_BUDGET).expect("shrunk");
            let clients: BTreeSet<u64> = shrunk.operations.iter().map(|it| it.client).collect();
            let rechecked = verdict(&model, &through_the_reader(&model, &shrunk), DEFAULT_BUDGET);
            if clients.len() > 2
                || shrunk.clients > 2
                || shrunk.operations.len() > 4
                || !every_client_is_sequential(&shrunk)
                || rechecked != Verdict::Violation
            {
                failures.push(format!(
                    "{name}, seed {seed}: {} client(s) used, `clients` {}, {} operation(s), \
                     sequential clients {}, re-checked {rechecked:?}\n{}",
                    clients.len(),
                    shrunk.clients,
                    shrunk.operations.len(),
                    every_client_is_sequential(&shrunk),
                    shrunk
                        .operations
                        .iter()
                        .map(|it| format!(
                            "  client {} [{}, {:?}] {} -> {:?}",
                            it.client,
                            it.invoked_at,
                            it.returned_at,
                            it.command.as_str(),
                            it.outcome.as_ref().map(history::QualifiedName::as_str)
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }
        assert!(violations > 0, "{name}: some seed overlaps two payments");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
