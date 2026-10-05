//! A string operator against a command input, `{input: <name>}` (`ess/22`, beyond10x/ess#200), at
//! Entity Runtime lowering.
//!
//! entity-core's `starts_with`, `ends_with` and `contains` take a reference operand, so the input is
//! lowered as the reference `$args.input.<name>` — never as the text `{input: <name>}` or the
//! spelling `input.<name>` — and the lowered runtime decides the guard the way ESS does: the stored
//! text tested against the input, byte for byte, and not the other way round.
//! `docs/design/expression-family-source22.md`, "Typed text operands (#200)", is the design.
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
use ess_entity_runtime::{lower, BoundTarget, LoweringOptions};
use ess_service_contract::{extract, ServiceIr};
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
                assert!(text.contains(before), "{target}: {before}");
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
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates: {errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn selected<'a>(ir: &'a EssIr, plan: &'a SynthesisPlan, component: &str) -> ServiceIr<'a> {
    extract(
        ir,
        plan,
        &ComponentName::new(component).expect("component name"),
    )
    .expect("service extracts")
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("qualified name")
}

fn lowered(ir: &EssIr) -> ess_entity_runtime::LoweredService {
    let plan = SynthesisPlan::of(ir);
    lower(
        &selected(ir, &plan, "invoice-service"),
        &LoweringOptions {
            definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
                .iter()
                .map(|entry| (name(entry), NonZeroU32::new(1).expect("nonzero")))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .unwrap_or_else(|diagnostics| panic!("the service lowers: {:?}", diagnostics.into_vec()))
}

fn registry(lowered: &ess_entity_runtime::LoweredService) -> Registry {
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers again");
    }
    registry.validate_all().expect("complete closure validates");
    registry
}

const KEPT_INPUT: &str =
    "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n";

/// `CancelInvoice` under `ess/22`, with String inputs `note` and `reference` beside the identity,
/// and the refusal `posted` guarded by `guard`, a `when:` or a `when_subject:`.
fn cancel(guard: &str) -> EssIr {
    let refusal = format!(
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: note\n        \
         type: String\n      - name: reference\n        type: String\n\n    outcomes:\n      - \
         name: posted\n{guard}        error: billing.invoice.InvoiceStateConflict\n      - name: \
         cancelled\n"
    );
    compile_changes(
        &example("billing"),
        &[
            ("system.yaml", "format: ess/1\n", "format: ess/22\n"),
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
            ("domains/invoice.yaml", KEPT_INPUT, refusal.as_str()),
        ],
    )
}

/// The lowered condition of `CancelInvoice`'s first outcome.
fn condition(lowered: &ess_entity_runtime::LoweredService) -> String {
    let posted = &lowered.definitions()[&name("billing.invoice.Invoice")].operations
        ["billing.invoice.CancelInvoice"]
        .outcomes[0];
    serde_json::to_value(&posted.when)
        .expect("condition serializes")
        .to_string()
}

#[test]
fn t200_an_input_operand_of_a_plain_guard_lowers_to_the_argument_reference() {
    for (keyword, lowered_keyword) in [
        ("starts_with", "starts_with"),
        ("ends_with", "ends_with"),
        ("contains", "contains"),
    ] {
        let ir = cancel(&format!(
            "        when: {{note: {{{keyword}: {{input: reference}}}}}}\n"
        ));
        let when = condition(&lowered(&ir));
        assert!(
            when.contains(&format!(
                r#"{{"{lowered_keyword}":["$args.input.note","$args.input.reference"]}}"#
            )),
            "{keyword}: {when}"
        );
        assert!(!when.contains("{input"), "{when}");
    }
}

#[test]
fn t200_an_input_operand_of_a_stored_row_guard_lowers_and_the_runtime_decides_it() {
    let lowered = lowered(&cancel(
        "        when_subject: {predicate: {note: {starts_with: {input: reference}}}}\n",
    ));
    let when = condition(&lowered);
    assert!(
        when.contains(r#"{"starts_with":["$fields.note","$args.input.reference"]}"#),
        "{when}"
    );
    assert!(!when.contains("$fields.input"), "{when}");

    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let binding = &lowered.bindings().commands()[&name("billing.invoice.CancelInvoice")];
    let taken = |sent: &str| {
        let arguments = json!({
            "input": {
                "invoice_id": "b404a1e8-9360-4af5-a0ac-8a483adfa225",
                "note": "unused",
                "reference": sent,
            },
            "bound": bound_for(binding, |_| None)
        });
        let row = invoice_instance("routine");
        match runtime.decide_before_load(
            &row.entity,
            row.version,
            row.id.clone(),
            "billing.invoice.CancelInvoice",
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
    };
    // The stored note begins with the input: the refusal holds.
    assert!(taken("rout").starts_with("refused"), "{when}");
    assert!(taken("").starts_with("refused"), "{when}");
    // Not a prefix, a prefix in another case, the stored note extended — the operator the wrong
    // way round would hold of that one — and the spelling of the operand: the command cancels.
    for sent in ["outine", "ROUT", "routine!", "{input: reference}"] {
        assert_eq!(taken(sent), "cancelled", "{sent}: {when}");
    }
}

fn invoice_instance(note: &str) -> EntityInstance {
    let logical_id = json!("b404a1e8-9360-4af5-a0ac-8a483adfa225");
    let fields = serde_json::from_value(json!({
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
        "reminder_count": 0,
        "note": note
    }))
    .expect("invoice fields");
    EntityInstance {
        entity: "billing.invoice.Invoice".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &logical_id).expect("identity address"),
        lifecycle_state: "Draft".to_owned(),
        revision: 1,
        fields,
    }
}

fn bound_for(
    binding: &ess_entity_runtime::CommandBinding,
    mut supply: impl FnMut(&BoundTarget) -> Option<Value>,
) -> Value {
    let mut bound = serde_json::Map::new();
    for (slot, value) in &binding.slots {
        if let Some(supplied) = supply(&value.target) {
            bound.insert(format!("b{:08}", slot.index()), supplied);
        }
    }
    Value::Object(bound)
}
