//! An input-guarded refusal beside branches selected by the held state (beyond10x/ess#227).
//!
//! The refusal is answered before existence and before the held state
//! (`docs/design/outcome-shapes.md` "Precedence"). Entity Runtime takes the first branch whose
//! guard holds, and the lowering orders every input-guarded refusal first
//! (`docs/design/input-guard-overlap-precedence.md`), so a refused request is decided before any
//! row is loaded — for an identity nothing stores, and for a record in any state — whatever order
//! the source declared the branches in.
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use entity_core::{
    identity, EntityInstance, FieldKind, LoadedDecision, PreloadDecision, Registry, Runtime,
};
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

/// `CancelInvoice` rewritten as a held-state command: `cancelled` in `Draft` or `Issued`, a
/// default refusal for the rest, and — declared last — `no-reason: reason == ""`, which names
/// no subject.
const CANCEL: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n        moves: billing.invoice.Invoice.cancel\n";
const HELD: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: reason\n        type: String\n\n    outcomes:\n      - name: cancelled\n        when_subject_state: [Draft, Issued]\n        moves: billing.invoice.Invoice.cancel\n";
const WRONG_STATE: &str = "      - name: wrong-state\n        wrong_state: true\n        error: billing.invoice.InvoiceStateConflict\n        summary: The invoice is already Paid or already Cancelled, so nothing was cancelled.\n";
const REFUSALS: &str = "      - name: wrong-state\n        error: billing.invoice.InvoiceStateConflict\n        summary: The invoice is already Paid or already Cancelled, so nothing was cancelled.\n      - name: no-reason\n        when: reason == \"\"\n        error: billing.invoice.InvalidAmount\n        summary: A cancellation needs a reason.\n";

fn compiled() -> EssIr {
    compiled_with(REFUSALS)
}

fn compiled_with(refusals: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing");
    let changes: [(&str, &str, &str); 5] = [
        ("system.yaml", "format: ess/1\n", "format: ess/18\n"),
        (
            "domains/email.yaml",
            "            recipient: input.recipient\n",
            "            recipient: input.recipient\n            message_id: {generated: true}\n",
        ),
        (
            "domains/invoice.yaml",
            "          billing.invoice.InvoiceCreated:\n",
            "          billing.invoice.InvoiceCreated:\n            invoice_id: {generated: true}\n",
        ),
        ("domains/invoice.yaml", CANCEL, HELD),
        ("domains/invoice.yaml", WRONG_STATE, refusals),
    ];
    let mut pending = vec![base.clone()];
    let mut paths = Vec::new();
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
        for (target, before, after) in &changes {
            if label == *target {
                assert!(text.contains(before), "{label}: `{before}` is present");
                text = text.replacen(before, after, 1);
            }
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn runtime_registry() -> Registry {
    registry_of(&compiled())
}

fn registry_of(ir: &EssIr) -> Registry {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("invoice-service").unwrap())
        .expect("service extracts");
    let lowered = lower(
        &service,
        &LoweringOptions {
            definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
                .iter()
                .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"));
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry.validate_all().expect("closure validates");
    registry
}

const INVOICE: &str = "b404a1e8-9360-4af5-a0ac-8a483adfa225";

fn invoice(state: &str) -> EntityInstance {
    let fields: serde_json::Map<String, Value> = serde_json::from_value(json!({
        "invoice_id": INVOICE,
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
        id: identity::address(FieldKind::String, &json!(INVOICE)).expect("identity address"),
        lifecycle_state: state.to_owned(),
        revision: 1,
        fields,
    }
}

/// Which branch `CancelInvoice` takes with `reason`: decided before any row is read, or on a
/// record resting in `state`.
fn taken(registry: &Registry, reason: &str, state: &str) -> String {
    let runtime = Runtime::new(registry);
    let row = invoice(state);
    let arguments = json!({"input": {"invoice_id": INVOICE, "reason": reason}, "bound": {}});
    match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        "billing.invoice.CancelInvoice",
        arguments,
    ) {
        Ok(PreloadDecision::Refused(refusal)) => format!("before load: {}", refusal.outcome),
        Ok(PreloadDecision::Load(prepared)) => match prepared.select_with(&row) {
            Ok(LoadedDecision::NeedsFulfillment(prepared)) => {
                format!("{state}: {}", prepared.outcome())
            }
            Ok(LoadedDecision::Complete(evaluation)) => match evaluation.into_decision() {
                Ok(_) => format!("{state}: accepted"),
                Err(refusal) => format!("{state}: refused {refusal:?}"),
            },
            Err(error) => format!("{state}: error {error:?}"),
        },
        Err(error) => format!("error before load: {error:?}"),
    }
}

#[test]
fn a_refused_input_is_decided_before_any_row_is_read_in_every_state() {
    let registry = runtime_registry();
    for state in ["Draft", "Issued", "Paid", "Cancelled"] {
        assert_eq!(
            taken(&registry, "", state),
            "before load: no-reason",
            "{state}"
        );
    }
}

#[test]
fn an_input_the_refusal_does_not_claim_is_still_selected_by_the_held_state() {
    let registry = runtime_registry();
    for state in ["Draft", "Issued"] {
        let answer = taken(&registry, "duplicate", state);
        assert!(answer.ends_with(": cancelled"), "{state}: {answer}");
    }
    for state in ["Paid", "Cancelled"] {
        let answer = taken(&registry, "duplicate", state);
        assert!(
            answer.contains("refused") && answer.contains("wrong-state"),
            "{state}: {answer}"
        );
    }
}

/// The kernel half of the precedence order (`docs/design/cross-record-and-stored-field-guards.md`,
/// "The precedence order"): input-guarded refusals are decided before any row is loaded, and of
/// two that claim one input the first declared answers — whichever of them that is, and although
/// both are declared after the held-state branch.
#[test]
fn of_two_overlapping_refusals_the_first_declared_answers_before_any_row_is_read() {
    const NO_REASON: &str = "      - name: no-reason\n        when: reason == \"\"\n        error: billing.invoice.InvalidAmount\n        summary: A cancellation needs a reason.\n";
    const NO_REASON_AGAIN: &str = "      - name: no-reason-again\n        when: reason == \"\"\n        error: billing.invoice.InvoiceStateConflict\n        summary: A cancellation still needs a reason.\n";
    let wrong = "      - name: wrong-state\n        error: billing.invoice.InvoiceStateConflict\n        summary: The invoice is already Paid or already Cancelled, so nothing was cancelled.\n";
    for (first, second, expected) in [
        (NO_REASON, NO_REASON_AGAIN, "no-reason"),
        (NO_REASON_AGAIN, NO_REASON, "no-reason-again"),
    ] {
        let registry = registry_of(&compiled_with(&format!("{wrong}{first}{second}")));
        for state in ["Draft", "Issued", "Paid", "Cancelled"] {
            assert_eq!(
                taken(&registry, "", state),
                format!("before load: {expected}"),
                "{state}"
            );
        }
    }
}
