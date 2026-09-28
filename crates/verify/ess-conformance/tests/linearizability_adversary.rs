//! Adversarial cases for `story:linearizability-checker-over-the-interpreter`.
//!
//! Each case states a property the acceptance or the module documentation of `linearize` rests
//! on, and drives the checker from a history the model's own interpreter (or the recorder the unit
//! ships) produced.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, History, Verdict};
use ess_conformance::interpret::Interpreted;
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::record::{self, Atomic, Call, Subject, Workload};
use ess_conformance::scenario::SuiteProvenance;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const REGISTER: &str = include_str!("fixtures/register/register.yaml");

fn model_from(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let specification = Specification::assemble([(Source::new("register.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

/// The committed register fixture, with the `Written` event carrying one more value the
/// implementation assigns: a `{generated: true}` stamp, the same construct the fixture already
/// uses for the created identity.
fn stamped_register_model() -> EssIr {
    let text = REGISTER
        .replace(
            "types:\n",
            "types:\n  - name: register.cell.Stamp\n    kind: newtype\n    of: Uuid\n",
        )
        .replace(
            "  - name: register.cell.Written\n    fields:\n      - {name: register_id, type: register.cell.RegisterId}\n",
            "  - name: register.cell.Written\n    fields:\n      - {name: register_id, type: register.cell.RegisterId}\n      - {name: stamp, type: register.cell.Stamp}\n",
        )
        .replace(
            "          register.cell.Written:\n            register_id: input.register_id\n",
            "          register.cell.Written:\n            register_id: input.register_id\n            stamp: {generated: true}\n",
        );
    assert!(
        text.contains("stamp: {generated: true}") && text.contains("register.cell.Stamp"),
        "the variant was built"
    );
    model_from(&text)
}

fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the recorded history is admitted: {refusal}"))
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

// ---- a false Violation: an event field the implementation assigns ------------------------------

#[test]
fn a_sequential_history_the_models_own_interpreter_answered_is_linearizable_when_an_event_carries_a_generated_field(
) {
    // `linearize`'s module documentation: "every other value the implementation assigns is left to
    // the model's own counter". `execute_generating` under `Generated::Given` does the opposite: a
    // required slot the caller did not give means the branch yields no step. The checker gives only
    // the created identity, so any emitted event with another `{generated: true}` field is a step
    // no input explains, and a one-client, one-call-at-a-time history recorded against the model's
    // own interpreter is reported as a violation.
    let model = stamped_register_model();
    let target = Interpreted::for_model(model.clone());
    let workload = Workload {
        prefix: vec![
            Call::new(
                "register.cell.CreateRegister",
                BTreeMap::new(),
                Subject::Creates,
            ),
            Call::new(
                "register.cell.WriteOne",
                BTreeMap::new(),
                Subject::Created(0),
            ),
        ],
        clients: Vec::new(),
    };
    let recorded = record::record(&model, &Atomic(&target), &workload, 0).expect("recorded");
    let history = through_the_reader(&model, &recorded);
    let outcomes: Vec<Option<&str>> = history
        .operations
        .iter()
        .map(|operation| {
            operation
                .outcome
                .as_ref()
                .map(history::QualifiedName::as_str)
        })
        .collect();
    assert_eq!(
        outcomes,
        vec![Some("created"), Some("written")],
        "the interpreter answered both calls as the model declares: {history:#?}"
    );
    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(
        checked.verdict,
        Verdict::Linearizable,
        "a sequential history the model itself produced has an order the model accepts: \
         {checked:#?}"
    );
}

// ---- error branches are answered, not only success branches ------------------------------------

#[test]
fn a_declared_refusal_recorded_from_the_interpreter_is_explained() {
    let model = model_from(REGISTER);
    let target = Interpreted::for_model(model.clone());
    let workload = Workload {
        prefix: vec![
            Call::new(
                "register.cell.CreateRegister",
                BTreeMap::new(),
                Subject::Creates,
            ),
            Call::new(
                "register.cell.ReadOne",
                BTreeMap::new(),
                Subject::Created(0),
            ),
        ],
        clients: Vec::new(),
    };
    let history = through_the_reader(
        &model,
        &record::record(&model, &Atomic(&target), &workload, 0).expect("recorded"),
    );
    assert_eq!(
        history.operations[1]
            .outcome
            .as_ref()
            .map(history::QualifiedName::as_str),
        Some("other")
    );
    assert_eq!(
        linearize::check(&model, &history, DEFAULT_BUDGET)
            .expect("checked")
            .verdict,
        Verdict::Linearizable
    );
}

// ---- shrinking beyond the pinned two-client workload -------------------------------------------

#[test]
fn a_three_client_lost_update_shrinks_to_at_most_two_clients_and_four_operations() {
    // The acceptance bound on the shrunk history is about the violation, not about the workload that
    // happened to be recorded: a third paying client adds nothing the race needs.
    let model = billing_model();
    let mut workload: Workload = faulty::lost_update_workload();
    let third = workload.clients[0].clone();
    workload.clients.push(third);
    let target = faulty::billing(Fault::LostUpdate);
    let mut violations = 0;
    for seed in 0..16 {
        let history = through_the_reader(
            &model,
            &record::record(&model, &target, &workload, seed).expect("recorded"),
        );
        if linearize::check(&model, &history, DEFAULT_BUDGET)
            .expect("checked")
            .verdict
            != Verdict::Violation
        {
            continue;
        }
        violations += 1;
        let shrunk = linearize::shrink(&model, &history, DEFAULT_BUDGET).expect("shrunk");
        let clients: BTreeSet<u64> = shrunk.operations.iter().map(|it| it.client).collect();
        assert!(
            clients.len() <= 2 && shrunk.operations.len() <= 4,
            "seed {seed}: {} clients, {} operations: {shrunk:#?}",
            clients.len(),
            shrunk.operations.len()
        );
        assert_eq!(
            linearize::check(&model, &through_the_reader(&model, &shrunk), DEFAULT_BUDGET)
                .expect("checked")
                .verdict,
            Verdict::Violation,
            "seed {seed}"
        );
    }
    assert!(violations > 0, "some seed overlaps two payments");
}
