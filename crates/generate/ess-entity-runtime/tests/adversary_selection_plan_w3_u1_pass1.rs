//! Adversary pass 1 against `story:entity-runtime-lowering-reads-selection-plan` (wave 3, U1).
//!
//! The wave's invariant: no behaviour change for any model that validates, with one byte exception
//! (a held-state branch declared after an accepting or external branch whose input guard the
//! finite prover shows disjoint). The story's acceptance: no branch-selection-order rule in
//! `lower_command` other than the two entity-core target rules.
//!
//! A refusal written `when: true` + `error:` validates (it is the command's one unconditional
//! branch, `Outcome::is_unconditional`). The base lowering put it in category 0 with every other
//! `when:` + `error:` branch, in declaration order, so declared before an input-guarded refusal it
//! was lowered first and Entity Runtime answered it for every request. The precedence plan places
//! it as the unguarded refusal at the head of `HeldState`, after `InputRefusal`, so the lowering
//! now puts the input-guarded refusal first and Entity Runtime answers that refusal wherever its
//! guard holds. That is neither the epic's named exception nor an unchanged answer.
//!
//! Which side is right is not settled here. The model interpreter's `refused_by_input` declines a
//! trivially true guard (`ess-conformance/src/interpret/execute.rs`, `refused_by_input`), so it
//! answers as the new lowering does. The Rust emitter answers every `when:` + `error:` branch with
//! no subject first, in declaration order (`ess-synth/src/rust/behaviour.rs`, the loop writing
//! "an input-guarded refusal, before the addressed subject is loaded"), and the generated explorer
//! decides "in Entity Runtime's order" with the same rule (`ess-conformance/src/ts/explore.ts`,
//! `isInputRefusal`), so they answer as the base lowering did.
//!
//! Decided in <https://github.com/beyond10x/ess/issues/489>: validation refuses this shape
//! (<https://github.com/beyond10x/ess/pull/490>). Until that is in this tree's base, the case below
//! asserts the current state, the plan's lowered order and the interpreter's answer; once it is,
//! the case becomes a check that validation refuses the model.
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
use ess_entity_runtime::{lower, LoweredService, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

const CANCEL: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n        moves: billing.invoice.Invoice.cancel\n";

/// `CancelInvoice` with `paused: when: true` + `error:` declared first, then the input-guarded
/// refusal `rush-refused: when: rush == true`, then `cancelled` guarded by `rush == false`, so
/// `paused` is the command's one unconditional branch; the example's `wrong-state` follows.
const PAUSED_FIRST: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: rush\n        type: Boolean\n\n    outcomes:\n      - name: paused\n        when: true\n        error: billing.invoice.InvoiceStateConflict\n        summary: Cancelling is paused.\n      - name: rush-refused\n        when: rush == true\n        error: billing.invoice.InvalidAmount\n        summary: A rushed cancellation is refused.\n      - name: cancelled\n        when: rush == false\n        moves: billing.invoice.Invoice.cancel\n";

/// The billing example at `ess/20`, with `CancelInvoice`'s input and first branch replaced by
/// `cancel`.
fn assembled(cancel: &str) -> (Result<Specification, String>, SourceMap, Vec<String>) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing");
    let changes: [(&str, &str, &str); 4] = [
        ("system.yaml", "format: ess/1\n", "format: ess/20\n"),
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
        ("domains/invoice.yaml", CANCEL, cancel),
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
    (
        Specification::assemble(parsed).map_err(|errors| errors.to_string()),
        sources,
        labels,
    )
}

fn compiled(cancel: &str) -> EssIr {
    let (specification, sources, labels) = assembled(cancel);
    let specification =
        specification.unwrap_or_else(|errors| panic!("the model validates: {errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn lowered(ir: &EssIr) -> LoweredService {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("invoice-service").unwrap())
        .expect("service extracts");
    lower(
        &service,
        &LoweringOptions {
            definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
                .iter()
                .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .unwrap_or_else(|diagnostics| panic!("the model lowers: {diagnostics:?}"))
}

/// `CancelInvoice`'s branches in their lowered order.
fn lowered_order(lowered: &LoweredService) -> Vec<String> {
    lowered.definitions()[&QualifiedName::new("billing.invoice.Invoice").unwrap()]
        .as_definition()
        .operations["billing.invoice.CancelInvoice"]
        .outcomes
        .iter()
        .map(|outcome| outcome.name.clone())
        .collect()
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

/// Which branch `CancelInvoice` takes with `rush` on a `Draft` invoice.
fn taken(registry: &Registry, rush: bool) -> String {
    let runtime = Runtime::new(registry);
    let row = invoice("Draft");
    let arguments = json!({"input": {"invoice_id": INVOICE, "rush": rush}, "bound": {}});
    match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        "billing.invoice.CancelInvoice",
        arguments,
    ) {
        Ok(PreloadDecision::Refused(refusal)) => format!("before load: {}", refusal.outcome),
        Ok(PreloadDecision::Load(prepared)) => match prepared.select_with(&row) {
            Ok(LoadedDecision::NeedsFulfillment(prepared)) => prepared.outcome().to_string(),
            Ok(LoadedDecision::Complete(evaluation)) => match evaluation.into_decision() {
                Ok(_) => "accepted".to_owned(),
                Err(refusal) => format!("refused {refusal:?}"),
            },
            Err(error) => format!("error {error:?}"),
        },
        Err(error) => format!("error before load: {error:?}"),
    }
}

/// A `when: true` refusal declared before an input-guarded refusal is lowered after it, as the
/// precedence plan places it, and Entity Runtime answers `rush: true` with `rush-refused`, as the
/// model interpreter does; the base lowering answered `paused`. Validation is to refuse the shape
/// (<https://github.com/beyond10x/ess/issues/489>).
#[test]
fn adv_w3u1_p1_a_when_true_refusal_declared_before_an_input_refusal_lowers_after_it() {
    let ir = compiled(PAUSED_FIRST);
    let lowered = lowered(&ir);
    let registry = registry(&lowered);

    // Both inputs, before any assertion, so the red output names every answer that moved.
    let answers = (taken(&registry, true), taken(&registry, false));
    assert_eq!(
        (lowered_order(&lowered), answers),
        (
            vec![
                "rush-refused".to_owned(),
                "paused".to_owned(),
                "cancelled".to_owned(),
                "wrong-state".to_owned(),
            ],
            (
                "before load: rush-refused".to_owned(),
                "before load: paused".to_owned()
            ),
        ),
        "(lowered order, (answer with rush: true, answer with rush: false)) with the lowering \
         reading the precedence plan; https://github.com/beyond10x/ess/issues/489 decides that \
         validation refuses this model, and once https://github.com/beyond10x/ess/pull/490 is in \
         this tree's base this case becomes a check that validation refuses it"
    );
}
