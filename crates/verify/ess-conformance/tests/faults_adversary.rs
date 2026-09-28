//! Adversarial cases against declared fault injection (story `declared-fault-injection`).
//!
//! The first two are red on the tree they were written against: under `Generated::Recorded` the
//! interpreter steps a `replays:` branch as an ordinary external branch, so a replay answered for a
//! request nothing retained is accepted as linearizable. The rest pin guards the unit's own suite
//! leaves unobserved.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, Completion, Verdict};
use ess_conformance::linearize;
use ess_conformance::record::{Atomic, Call, Subject};
use ess_conformance::reference::{Billing, Retained};
use ess_conformance::scenario::SuiteProvenance;
use ess_conformance::sessions::{self, Act};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const RETRY: &str = "crates/verify/ess-conformance/tests/fixtures/explore-retry";
const SEEDS: u64 = 24;

fn model_rewritten(path: &str, rewrite: impl Fn(&str) -> String) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(path)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{path}` exists: {error}"));
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let entry = entry.expect("an entry").path();
            if entry.is_dir() {
                pending.push(entry);
            } else if entry.extension().is_some_and(|it| it == "yaml") {
                found.push(entry);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for file in found {
        let label = file
            .strip_prefix(&base)
            .expect("inside")
            .display()
            .to_string();
        let text = rewrite(&std::fs::read_to_string(&file).expect("readable"));
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("`{path}` validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("`{path}` resolves:\n{diagnostics}"))
}

fn model(path: &str) -> EssIr {
    model_rewritten(path, ToOwned::to_owned)
}

/// One `retry.core.Seed` operation, as the history writes it.
fn seed_op(id: u8, invoked: u64, returned: u64, outcome: &str, retry_of: Option<u8>) -> String {
    let key = if outcome == "seeded" {
        format!("00000000-0000-4000-8000-0000000000a{id}")
    } else {
        String::new()
    };
    let retry = retry_of.map_or_else(String::new, |of| {
        format!(r#","retry_of":"00000000-0000-4000-8000-00000000000{of}""#)
    });
    format!(
        r#"{{"operation_id":"00000000-0000-4000-8000-00000000000{id}","client":0,"command":"retry.core.Seed","subject_key":"{key}","invoked_at":{invoked},"returned_at":{returned},"completion":"Returned","outcome":"{outcome}"{retry}}}"#
    )
}

fn retry_history(model: &EssIr, operations: &[String]) -> history::History {
    let digest = SuiteProvenance::of(model).spec_digest;
    let bytes = format!(
        r#"{{"format":"ess-history/1","history_id":"00000000-0000-4000-8000-000000000001","spec_digest":"{digest}","seed":1,"clients":1,"operations":[{}]}}"#,
        operations.join(",")
    );
    history::read(bytes.as_bytes(), &digest).expect("admitted")
}

#[test]
fn a_replay_answered_for_a_request_nothing_retained_is_not_linearizable() {
    // One `Seed`, never answered `seeded` by anything, answered `replayed`: the retained result of
    // a request nobody sent before. `replays: seeded` declares the replay only of a request the
    // origin branch already answered, so no linearization explains it.
    let model = model(RETRY);
    let history = retry_history(&model, &[seed_op(1, 1, 2, "replayed", None)]);
    let checked = linearize::check(&model, &history, linearize::DEFAULT_BUDGET).expect("checked");
    assert_ne!(
        checked.verdict,
        Verdict::Linearizable,
        "a replay with nothing retained was accepted: {checked:?}"
    );
}

#[test]
fn a_replay_answered_before_its_request_was_ever_applied_is_not_linearizable() {
    // The original answers `replayed` and returns before its retry is even sent; the retry then
    // answers `seeded`. The replay came from nothing: the origin answer is strictly later.
    let model = model(RETRY);
    let history = retry_history(
        &model,
        &[
            seed_op(1, 1, 2, "replayed", None),
            seed_op(2, 3, 4, "seeded", Some(1)),
        ],
    );
    let checked = linearize::check(&model, &history, linearize::DEFAULT_BUDGET).expect("checked");
    assert_ne!(
        checked.verdict,
        Verdict::Linearizable,
        "a replay preceding its origin answer was accepted: {checked:?}"
    );
}

#[test]
fn a_command_declaring_only_a_replays_external_branch_is_never_delayed_or_left_unanswered() {
    // `retry.core.Seed`'s only `external:` branch is `replayed`, which a retry exercises; a delay
    // or loss of its answer is not declared.
    let model = model(RETRY);
    for seed in 0..SEEDS {
        let reference = Retained::new();
        let recorded = sessions::record_injected(
            &model,
            &Atomic(&reference),
            &reference,
            &faulty::retry_workload(),
            seed,
        )
        .expect("recorded");
        assert!(
            recorded.injected.delayed.is_empty() && recorded.injected.unanswered.is_empty(),
            "seed {seed}: {:?}",
            recorded.injected
        );
        assert!(
            recorded
                .history
                .operations
                .iter()
                .all(|operation| operation.completion == Completion::Returned),
            "seed {seed}"
        );
    }
}

#[test]
fn nothing_is_injected_into_a_prefix_of_external_calls() {
    // `SendEmail` declares `failed` as `external:`; called in the prefix it is neither delayed nor
    // left unanswered, however many seeds are drawn.
    let model = model("examples/billing");
    let email = || {
        Call::new(
            "billing.email.SendEmail",
            BTreeMap::from([
                (
                    "recipient".to_owned(),
                    Node::Text("payer@example.com".to_owned()),
                ),
                (
                    "template".to_owned(),
                    Node::Text("invoice-created".to_owned()),
                ),
            ]),
            Subject::Creates,
        )
    };
    let workload = sessions::Workload {
        prefix: vec![email(), email(), email()],
        clients: vec![vec![Act::Read("billing.invoice.InvoiceById".to_owned())]],
    };
    for seed in 0..SEEDS {
        let reference = Billing::new();
        let recorded =
            sessions::record_injected(&model, &Atomic(&reference), &reference, &workload, seed)
                .expect("recorded");
        assert_eq!(
            recorded.injected.total(),
            0,
            "seed {seed}: {:?}",
            recorded.injected
        );
        assert!(
            recorded.history.operations[..3]
                .iter()
                .all(|operation| operation.completion == Completion::Returned),
            "seed {seed}"
        );
    }
}

#[test]
fn an_event_one_at_most_once_binding_also_reacts_to_is_never_delivered_twice() {
    // Two bindings react to `InvoiceCreated`: the declared `at_least_once` one and a second
    // declaring `at_most_once`. A second delivery reaches both, so it is not declared.
    let mixed = model_rewritten("examples/billing", |text| {
        text.replace(
            "  - id: notify-on-invoice-created\n",
            "  - id: audit-on-invoice-created\n    when:\n      event: billing.invoice.InvoiceCreated\n    invoke:\n      command: billing.email.SendEmail\n    mapping:\n      recipient: event.customer_email\n      template: invoice-created\n    delivery: at_most_once\n    on_failure: drop\n  - id: notify-on-invoice-created\n",
        )
    });
    assert_eq!(mixed.bindings().len(), 2, "the second binding was added");
    for seed in 0..SEEDS {
        let target = faulty::billing(Fault::DoubleApplyOnRedelivery);
        let recorded = sessions::record_injected(
            &mixed,
            &target,
            &target,
            &faulty::double_apply_workload(),
            seed,
        )
        .expect("recorded");
        assert!(
            recorded.injected.redeliveries.is_empty(),
            "seed {seed}: {:?}",
            recorded.injected
        );
    }
}
