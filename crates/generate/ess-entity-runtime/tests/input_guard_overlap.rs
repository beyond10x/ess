//! An input-guarded refusal is taken before any accepting branch whose guard it overlaps
//! (beyond10x/ess#178), and among the accepting guarded branches the first declared whose guard
//! holds answers (beyond10x/ess#217) — `docs/design/input-guard-overlap-precedence.md`.
//!
//! Entity Runtime selects the first branch whose guard holds, so the lowering orders every
//! input-guarded refusal ahead of the accepting guarded branches — whatever order the source wrote
//! them in — and keeps the accepting guarded branches in the order they are declared.
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

const REJECTED: &str = "      - name: rejected\n        error: billing.invoice.InvalidAmount\n        summary: The payment was not positive";

const SETTLED: &str = "      - name: settled\n        when: amount.amount > 0\n";

/// The billing example with `branches` written into `PayInvoice` just before `anchor`.
fn billing_with(anchor: &str, branches: &str) -> ess_compiler::ir::EssIr {
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
            assert!(text.contains(anchor), "the mutation site is present");
            text = text.replacen(anchor, &format!("{branches}{anchor}"), 1);
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

/// The billing example with `too-large: amount.amount > 1000` declared after
/// `settled: amount.amount > 0`, which it overlaps above 1000.
fn billing_with_a_late_refusal() -> ess_compiler::ir::EssIr {
    billing_with(
        REJECTED,
        "      - name: too-large\n        when: amount.amount > 1000\n        error: billing.invoice.InvalidAmount\n        summary: The payment is above the limit.\n",
    )
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

const INVOICE_ID: &str = "b404a1e8-9360-4af5-a0ac-8a483adfa225";

#[test]
fn an_input_guarded_refusal_is_selected_before_the_accepting_branch_it_overlaps() {
    let lowered = lowered(&billing_with_a_late_refusal());
    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let logical_id = json!(INVOICE_ID);
    let storage_id = identity::address(FieldKind::String, &logical_id).unwrap();
    let decide = |amount: i64| {
        runtime
            .decide_before_load(
                "billing.invoice.Invoice",
                1,
                storage_id.clone(),
                "billing.invoice.PayInvoice",
                json!({"input": {"invoice_id": logical_id, "amount": {"amount": amount, "currency": "EUR"}}, "bound": {}}),
            )
            .expect("input-only selection succeeds")
    };
    match decide(2000) {
        PreloadDecision::Refused(refusal) => assert_eq!(refusal.outcome, "too-large"),
        PreloadDecision::Load(prepared) => {
            panic!(
                "the overlap point reached `settled`: {:?}",
                prepared.subject()
            )
        }
    }
    assert!(matches!(decide(25), PreloadDecision::Load(_)));
    match decide(0) {
        PreloadDecision::Refused(refusal) => assert_eq!(refusal.outcome, "rejected"),
        PreloadDecision::Load(_) => panic!("a non-positive payment is refused"),
    }
}

/// `written-off: amount.amount < 10` moves the invoice to `Cancelled`; `settled: amount.amount > 0`
/// moves it to `Paid`. They overlap over `0 < amount < 10`.
const WRITTEN_OFF: &str = "      - name: written-off\n        when: amount.amount < 10\n        moves: billing.invoice.Invoice.cancel\n        instance: invoice_id\n        emits:\n          - billing.invoice.InvoiceCancelled\n        payload:\n          billing.invoice.InvoiceCancelled:\n            invoice_id: input.invoice_id\n        summary: A payment below ten writes the invoice off.\n";

/// An issued invoice.
fn issued() -> EntityInstance {
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
        lifecycle_state: "Issued".to_owned(),
        revision: 1,
        fields,
    }
}

/// The branch `PayInvoice` with `amount` takes for an issued invoice.
fn taken(lowered: &LoweredService, amount: i64) -> String {
    let registry = registry(lowered);
    let runtime = Runtime::new(&registry);
    let row = issued();
    let arguments = json!({"input": {"invoice_id": INVOICE_ID, "amount": {"amount": amount, "currency": "EUR"}}, "bound": {}});
    let prepared = match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        "billing.invoice.PayInvoice",
        arguments,
    ) {
        Ok(PreloadDecision::Load(prepared)) => prepared,
        Ok(PreloadDecision::Refused(refusal)) => panic!("refused before load: {}", refusal.outcome),
        Err(error) => panic!("not decided before load: {error}"),
    };
    match prepared.select_with(&row) {
        Ok(LoadedDecision::NeedsFulfillment(prepared)) => prepared.outcome().to_owned(),
        Ok(LoadedDecision::Complete(Evaluation::Accepted(_))) => {
            panic!("an accepting `PayInvoice` branch asks for its fields to be fulfilled")
        }
        Ok(LoadedDecision::Complete(Evaluation::Refused(refusal))) => {
            panic!("the payment is refused: {}", refusal.outcome)
        }
        Err(error) => panic!("the payment is not decided: {error}"),
    }
}

/// Among accepting guarded branches Entity Runtime answers with the first declared whose guard
/// holds (beyond10x/ess#217): `settled` before `written-off` pays an invoice at `5`, and the same
/// two written the other way round write it off.
#[test]
fn the_first_declared_accepting_branch_answers_the_overlap() {
    let settled_first = lowered(&billing_with(REJECTED, WRITTEN_OFF));
    assert_eq!(taken(&settled_first, 5), "settled");
    assert_eq!(taken(&settled_first, 25), "settled");
    let written_off_first = lowered(&billing_with(SETTLED, WRITTEN_OFF));
    assert_eq!(taken(&written_off_first, 5), "written-off");
    assert_eq!(taken(&written_off_first, 25), "settled");
}

/// `declined: external` declared after `settled: amount.amount > 0` and before the default.
const DECLINED: &str = "      - name: declined\n        external: the bank declines the payment\n        error: billing.invoice.InvalidAmount\n        summary: The bank declined, so the invoice did not move.\n";

/// An external branch follows the same declaration order as the accepting guarded branches
/// (beyond10x/ess#217): `settled`, declared first, answers an input its guard claims even when the
/// provider's verdict for `declined` is supplied, and the verdict decides only an input `settled`
/// does not claim.
#[test]
fn an_accepting_branch_declared_before_an_external_one_answers_first() {
    let lowered = lowered(&billing_with(REJECTED, DECLINED));
    let binding =
        &lowered.bindings().commands()[&QualifiedName::new("billing.invoice.PayInvoice").unwrap()];
    let evidence = binding
        .slots
        .iter()
        .find(|(_, value)| {
            matches!(&value.target, ess_entity_runtime::BoundTarget::ExternalEvidence { outcome } if outcome.as_str() == "declined")
        })
        .map(|(slot, _)| *slot)
        .expect("the external verdict is one supplied slot");
    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let logical_id = json!(INVOICE_ID);
    let storage_id = identity::address(FieldKind::String, &logical_id).unwrap();
    let decide = |amount: i64, verdict: bool| {
        runtime
            .decide_before_load(
                "billing.invoice.Invoice",
                1,
                storage_id.clone(),
                "billing.invoice.PayInvoice",
                json!({
                    "input": {"invoice_id": logical_id, "amount": {"amount": amount, "currency": "EUR"}},
                    "bound": {format!("b{:08}", evidence.index()): verdict}
                }),
            )
            .expect("input admitted")
    };
    assert!(
        matches!(decide(25, true), PreloadDecision::Load(_)),
        "`declined` answered an input `settled`, declared first, claims"
    );
    match decide(0, true) {
        PreloadDecision::Refused(refusal) => assert_eq!(refusal.outcome, "declined"),
        PreloadDecision::Load(_) => panic!("a declining verdict below `settled` is not taken"),
    }
    match decide(0, false) {
        PreloadDecision::Refused(refusal) => assert_eq!(refusal.outcome, "rejected"),
        PreloadDecision::Load(_) => panic!("a non-positive payment is refused"),
    }
}
