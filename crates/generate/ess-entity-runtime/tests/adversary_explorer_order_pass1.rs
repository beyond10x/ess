//! Adversary pass 1 on `story:explorers-decide-input-guards-before-wrong-state`
//! (beyond10x/ess#235): what Entity Runtime answers where an external branch that moves nothing
//! meets a subject in a state no move of the command starts from.
//!
//! The billing example's `CancelInvoice` (`cancelled` moves from `Draft`/`Issued`, `wrong-state`
//! answers in `Paid`/`Cancelled`) with an external `blocked` added. With the provider's verdict for
//! `blocked` supplied, Entity Runtime answers `blocked` on a `Paid` invoice, before the row is
//! loaded; without it, `wrong-state`. The generated explorer answers `wrong-state` in both cases
//! (`ess-conformance/tests/adversary_explore_order_pass1.rs`).
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use entity_core::{
    identity, EntityInstance, Evaluation, FieldKind, LoadedDecision, PreloadDecision, Registry,
    Runtime,
};
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweredService, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

const ANCHOR: &str = "      - name: wrong-state\n        wrong_state: true\n        error: billing.invoice.InvoiceStateConflict\n        summary: The invoice is already Paid or already Cancelled";

const BLOCKED: &str = "      - name: blocked\n        external: the ledger blocks the cancellation\n        error: billing.invoice.InvoiceStateConflict\n        summary: The ledger blocked it, so nothing was cancelled.\n\n";

const INVOICE_ID: &str = "b404a1e8-9360-4af5-a0ac-8a483adfa225";

fn billing_with_blocked() -> ess_compiler::ir::EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing");
    let mut paths = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut parsed = Vec::new();
    let mut labels = Vec::new();
    let mut sources = SourceMap::new();
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        if label == "domains/invoice.yaml" {
            assert_eq!(
                text.matches(ANCHOR).count(),
                1,
                "the mutation site is present"
            );
            text = text.replacen(ANCHOR, &format!("{BLOCKED}{ANCHOR}"), 1);
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).expect("fixture validates");
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn lowered(ir: &ess_compiler::ir::EssIr) -> LoweredService {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("invoice-service").unwrap())
        .expect("service extracts");
    let options = LoweringOptions {
        definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
            .iter()
            .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
            .collect(),
        scales: BTreeMap::new(),
    };
    lower(&service, &options).expect("billing lowers")
}

fn registry(lowered: &LoweredService) -> Registry {
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry.validate_all().expect("closure validates");
    registry
}

fn invoice(state: &str) -> EntityInstance {
    let fields: serde_json::Map<String, Value> = serde_json::from_value(json!({
        "invoice_id": INVOICE_ID,
        "account_id": "ae861ea5-96d8-48cf-a843-c9a0baf11389",
        "total": {"amount": 25, "currency": "EUR"},
        "payee": {"kind": "person", "value": "buyer@example.test"},
        "channel": "Email",
        "lines": [],
        "metadata": {},
        "settlement_window": "PT1H",
        "is_recurring": false,
        "signature": "AA==",
        "reminder_count": 0
    }))
    .expect("invoice fields");
    EntityInstance {
        entity: "billing.invoice.Invoice".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &json!(INVOICE_ID)).expect("identity address"),
        lifecycle_state: state.to_owned(),
        revision: 1,
        fields,
    }
}

/// The branch Entity Runtime answers `CancelInvoice` with, on an invoice in `state`, with the
/// provider's verdict for `blocked` set to `verdict`.
fn answered(lowered: &LoweredService, state: &str, verdict: bool) -> String {
    let binding = &lowered.bindings().commands()
        [&QualifiedName::new("billing.invoice.CancelInvoice").unwrap()];
    let evidence = binding
        .slots
        .iter()
        .find(|(_, value)| {
            matches!(&value.target, ess_entity_runtime::BoundTarget::ExternalEvidence { outcome } if outcome.as_str() == "blocked")
        })
        .map(|(slot, _)| *slot)
        .expect("the external verdict is one supplied slot");
    let registry = registry(lowered);
    let runtime = Runtime::new(&registry);
    let row = invoice(state);
    let arguments = json!({
        "input": {"invoice_id": INVOICE_ID},
        "bound": {format!("b{:08}", evidence.index()): verdict}
    });
    let prepared = match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        "billing.invoice.CancelInvoice",
        arguments,
    ) {
        Ok(PreloadDecision::Load(prepared)) => prepared,
        Ok(PreloadDecision::Refused(refusal)) => return refusal.outcome,
        Err(error) => panic!("not decided before load: {error}"),
    };
    match prepared.select_with(&row) {
        Ok(LoadedDecision::NeedsFulfillment(prepared)) => prepared.outcome().to_owned(),
        Ok(LoadedDecision::Complete(Evaluation::Refused(refusal))) => refusal.outcome,
        Ok(LoadedDecision::Complete(Evaluation::Accepted(_))) => "accepted".to_owned(),
        Err(error) => format!("error: {error}"),
    }
}

/// Entity Runtime's cells for the explorer's `Close`-shaped command: the arranged external that
/// moves nothing answers in the state no move starts from; only the default's move there is the
/// wrong-state answer.
#[test]
fn entity_runtime_answers_an_arranged_external_that_moves_nothing_in_a_wrong_state() {
    let lowered = lowered(&billing_with_blocked());
    assert_eq!(answered(&lowered, "Issued", false), "cancelled");
    assert_eq!(answered(&lowered, "Issued", true), "blocked");
    assert_eq!(answered(&lowered, "Paid", false), "wrong-state");
    assert_eq!(answered(&lowered, "Paid", true), "blocked");
}
