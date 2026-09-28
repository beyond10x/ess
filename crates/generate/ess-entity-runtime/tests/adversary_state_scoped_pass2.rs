//! Adversary pass 2 on the state-scoped unit (beyond10x/ess#201, #204): Entity Runtime parity for
//! the shapes the correction added to synthesis — a `state`-reading refusal answering every state
//! no move starts from with no `wrong_state:` branch, two subjectless listed refusals of one state
//! told apart by the input, and a guard mixing `state` with a stored field under `or`.
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

/// Which branch the lowered runtime takes for `CancelInvoice` on an invoice resting in `state`
/// with `channel` stored, sent `extra` beside the identity.
fn taken(
    lowered: &ess_entity_runtime::LoweredService,
    state: &str,
    channel: &str,
    extra: &[(&str, &str)],
) -> String {
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry.validate_all().expect("complete closure validates");
    let runtime = Runtime::new(&registry);
    let mut input = serde_json::Map::new();
    input.insert(
        "invoice_id".to_owned(),
        json!("b404a1e8-9360-4af5-a0ac-8a483adfa225"),
    );
    for (key, value) in extra {
        input.insert((*key).to_owned(), json!(value));
    }
    let arguments = json!({"input": Value::Object(input), "bound": {}});
    let row = invoice(state, channel);
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
}

const WRONG_STATE: &str = "      - name: wrong-state\n        wrong_state: true\n        error: billing.invoice.InvoiceStateConflict\n        summary: The invoice is already Paid or already Cancelled, so nothing was cancelled.\n";

const CANCEL_INPUT: &str = "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n";

fn refused_by(taken: &str, branch: &str) -> bool {
    taken.starts_with("refused") && taken.contains(&format!("\"{branch}\""))
}

/// The #204 spelling of #201: one refusal reading `state` answers every state `cancel` does not
/// start from, and there is no `wrong_state:` branch.
#[test]
fn a_state_inequality_refusal_decides_every_state_at_runtime() {
    let ir = compile_billing_ess18(&[(
        WRONG_STATE,
        "      - name: settled\n        when_subject:\n          predicate:\n            all:\n              - state != Draft\n              - state != Issued\n        error: billing.invoice.InvoiceStateConflict\n",
    )]);
    let lowered = lowered(&ir);
    let mut wrong = Vec::new();
    for state in ["Paid", "Cancelled"] {
        for channel in ["Email", "Post"] {
            let got = taken(&lowered, state, channel, &[]);
            if !refused_by(&got, "settled") {
                wrong.push(format!("{state}/{channel}: {got}"));
            }
        }
    }
    for state in ["Draft", "Issued"] {
        let got = taken(&lowered, state, "Post", &[]);
        if got != "cancelled" {
            wrong.push(format!("{state}: {got}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Two subjectless listed refusals of the same states, told apart by the input.
#[test]
fn listed_refusals_split_by_the_input_decide_per_input_at_runtime() {
    let ir = compile_billing_ess18(&[
        (
            CANCEL_INPUT,
            "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: via\n        type: billing.invoice.Channel\n\n    outcomes:\n      - name: cancelled\n",
        ),
        (
            WRONG_STATE,
            "      - name: settled-by-post\n        when_subject_state: [Paid, Cancelled]\n        when: via == Post\n        error: billing.invoice.InvoiceStateConflict\n      - name: settled-by-email\n        when_subject_state: [Paid, Cancelled]\n        when: via != Post\n        error: billing.invoice.InvoiceStateConflict\n",
        ),
    ]);
    let lowered = lowered(&ir);
    let mut wrong = Vec::new();
    for state in ["Paid", "Cancelled"] {
        for (via, branch) in [("Post", "settled-by-post"), ("Email", "settled-by-email")] {
            let got = taken(&lowered, state, "Email", &[("via", via)]);
            if !refused_by(&got, branch) {
                wrong.push(format!("{state}/{via}: expected {branch}, took {got}"));
            }
        }
    }
    for state in ["Draft", "Issued"] {
        let got = taken(&lowered, state, "Email", &[("via", "Post")]);
        if got != "cancelled" {
            wrong.push(format!("{state}: {got}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// `or` mixing the held state with a stored field on a refusal beside `wrong_state:`. The guarded
/// branch selects first (#192): a cancelled row, or any posted row, is `held`; a paid row sent by
/// email is `wrong-state`; draft and issued rows sent by email are cancelled.
#[test]
fn a_state_or_field_refusal_beside_wrong_state_decides_at_runtime() {
    let ir = compile_billing_ess18(&[(
        CANCEL_INPUT,
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: held\n        when_subject:\n          predicate:\n            any:\n              - state == Cancelled\n              - channel == Post\n        error: billing.invoice.InvoiceStateConflict\n      - name: cancelled\n",
    )]);
    let lowered = lowered(&ir);
    let mut wrong = Vec::new();
    for (state, channel, expected) in [
        ("Cancelled", "Email", "held"),
        ("Cancelled", "Post", "held"),
        ("Paid", "Post", "held"),
        ("Draft", "Post", "held"),
        ("Issued", "Post", "held"),
        ("Paid", "Email", "wrong-state"),
    ] {
        let got = taken(&lowered, state, channel, &[]);
        if !refused_by(&got, expected) {
            wrong.push(format!(
                "{state}/{channel}: expected {expected}, took {got}"
            ));
        }
    }
    for state in ["Draft", "Issued"] {
        let got = taken(&lowered, state, "Email", &[]);
        if got != "cancelled" {
            wrong.push(format!("{state}/Email: {got}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}
