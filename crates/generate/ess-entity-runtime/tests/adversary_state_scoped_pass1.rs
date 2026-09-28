//! Adversary pass 1 on the state-scoped unit (beyond10x/ess#204): the lowered selector must mean
//! what ESS means when a `when_subject` predicate reading `state` names a state no move of the
//! command starts from, beside a `wrong_state:` branch. ESS selects the guarded branch before
//! `wrong_state` applies (the beyond10x/ess#192 ruling), so the runtime must too.
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

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

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
}

fn compile_changes(base: &Path, changes: &[(&str, &str, &str)]) -> EssIr {
    let mut pending = vec![base.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut parsed = Vec::new();
    let mut labels = Vec::new();
    let mut sources = SourceMap::new();
    for path in paths {
        let label = path
            .strip_prefix(base)
            .expect("fixture child")
            .display()
            .to_string();
        let mut text = std::fs::read_to_string(&path).expect("fixture source is readable");
        for (target, before, after) in changes {
            if label == *target {
                assert!(
                    text.contains(before),
                    "mutation source is present in {target}: {before}"
                );
                text = text.replacen(before, after, 1);
            }
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).expect("fixture parses"),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("qualified name")
}

fn compile_billing_ess18(invoice: &[(&str, &str)]) -> EssIr {
    let mut changes = vec![
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
    ];
    changes.extend(
        invoice
            .iter()
            .map(|(before, after)| ("domains/invoice.yaml", *before, *after)),
    );
    compile_changes(&example("billing"), &changes)
}

fn lowered(ir: &EssIr) -> ess_entity_runtime::LoweredService {
    let plan = SynthesisPlan::of(ir);
    let service = extract(
        ir,
        &plan,
        &ComponentName::new("invoice-service").expect("component name"),
    )
    .expect("service extracts");
    lower(
        &service,
        &LoweringOptions {
            definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
                .iter()
                .map(|entry| (name(entry), NonZeroU32::new(1).expect("nonzero")))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"))
}

fn invoice(state: &str, channel: &str) -> EntityInstance {
    let logical_id = json!("b404a1e8-9360-4af5-a0ac-8a483adfa225");
    let mut fields: serde_json::Map<String, Value> = serde_json::from_value(json!({
        "invoice_id": "b404a1e8-9360-4af5-a0ac-8a483adfa225",
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
    fields.insert("channel".to_owned(), json!(channel));
    EntityInstance {
        entity: "billing.invoice.Invoice".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &logical_id).expect("identity address"),
        lifecycle_state: state.to_owned(),
        revision: 1,
        fields,
    }
}

/// Which branch the lowered runtime takes for `operation` on an invoice resting in `state`.
fn taken_in(
    lowered: &ess_entity_runtime::LoweredService,
    operation: &str,
    state: &str,
    channel: &str,
) -> String {
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry.validate_all().expect("complete closure validates");
    let runtime = Runtime::new(&registry);
    // `CancelInvoice` binds nothing beyond its input.
    assert!(lowered.bindings().commands().contains_key(&name(operation)));
    let arguments = json!({
        "input": {"invoice_id": "b404a1e8-9360-4af5-a0ac-8a483adfa225"},
        "bound": {}
    });
    let row = invoice(state, channel);
    match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        operation,
        arguments,
    ) {
        Ok(PreloadDecision::Load(prepared)) => match prepared.select_with(&row) {
            Ok(LoadedDecision::NeedsFulfillment(prepared)) => prepared.outcome().to_owned(),
            Ok(LoadedDecision::Complete(evaluation)) => match evaluation.into_decision() {
                Ok(_) => "accepted without fulfillment".to_owned(),
                Err(refusal) => format!("refused: {refusal:?}"),
            },
            Err(error) => format!("error: {error:?}"),
        },
        Ok(_) => "decided before load".to_owned(),
        Err(error) => format!("error before load: {error:?}"),
    }
}

const CANCEL_OUTCOMES: &str = "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n";

/// `CancelInvoice` keeps its `wrong_state:` branch for `Paid` and `Cancelled`; a posted, paid
/// invoice is refused by a guarded branch reading `state` instead. ESS answers a paid, posted row
/// with `posted-paid` (guarded before `wrong_state`), and any other paid row with `wrong-state`.
#[test]
fn a_state_reading_refusal_in_a_wrong_state_is_selected_before_wrong_state_at_runtime() {
    let ir = compile_billing_ess18(&[(
        CANCEL_OUTCOMES,
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: posted-paid\n        when_subject:\n          predicate:\n            all:\n              - state == Paid\n              - channel == Post\n        error: billing.invoice.InvoiceStateConflict\n      - name: cancelled\n",
    )]);
    let lowered = lowered(&ir);
    for (state, channel, expected) in [
        ("Paid", "Post", "\"posted-paid\""),
        ("Paid", "Email", "\"wrong-state\""),
        ("Cancelled", "Post", "\"wrong-state\""),
    ] {
        let taken = taken_in(&lowered, "billing.invoice.CancelInvoice", state, channel);
        assert!(
            taken.starts_with("refused") && taken.contains(expected),
            "{state}/{channel}: expected {expected}, runtime took {taken}"
        );
    }
    assert_eq!(
        taken_in(&lowered, "billing.invoice.CancelInvoice", "Issued", "Post"),
        "cancelled"
    );
}
