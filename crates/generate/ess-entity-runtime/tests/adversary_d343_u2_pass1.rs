//! Adversary pass 1 on `story:entity-runtime-lowering-enforces-input-type-invariants`: every rule
//! of an unstored command input's type either lowers to an operation precondition entity-core
//! enforces the way ESS decides it, or is refused by name naming the input; none is dropped.
//!
//! Each case compiles the billing example with `CancelInvoice` (an update) taking one extra input
//! whose type carries rules, lowers it, and asks entity-core for the branch it takes. A named
//! refusal of the input is always an acceptable answer; a lowered definition must accept what ESS
//! admits and refuse what ESS does not.

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
use ess_entity_runtime::{
    lower, LoweredService, LoweringCode, LoweringDiagnostics, LoweringOptions,
};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

const STORY: &str = "story:entity-runtime-lowering-enforces-input-type-invariants";
const KEPT_INPUT: &str = "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n";
const ID: &str = "b404a1e8-9360-4af5-a0ac-8a483adfa225";

fn billing_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing")
}

fn try_compile(base: &Path, changes: &[(&str, &str, &str)]) -> Result<EssIr, String> {
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
            RawSpecFile::parse(&text).map_err(|error| format!("{error:?}"))?,
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).map_err(|errors| format!("{errors:?}"))?;
    compile_locating(&specification, &sources, &labels).map_err(|errors| format!("{errors:?}"))
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("qualified name")
}

fn lower_billing(ir: &EssIr) -> Result<LoweredService, LoweringDiagnostics> {
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
}

fn registry(lowered: &LoweredService) -> Registry {
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers again");
    }
    registry.validate_all().expect("complete closure validates");
    registry
}

fn invoice_row() -> EntityInstance {
    let logical_id = json!(ID);
    let row: serde_json::Map<String, Value> = serde_json::from_value(json!({
        "invoice_id": ID,
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
        id: identity::address(FieldKind::String, &logical_id).expect("identity address"),
        lifecycle_state: "Draft".to_owned(),
        revision: 1,
        fields: row,
    }
}

/// The branch the lowered `CancelInvoice` takes on a Draft invoice for `input`.
fn cancel_with(lowered: &LoweredService, input: &Value) -> String {
    let registry = registry(lowered);
    let runtime = Runtime::new(&registry);
    let row = invoice_row();
    let arguments = json!({"input": input, "bound": {}});
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
        Ok(PreloadDecision::Refused(refusal)) => format!("refused before load: {refusal:?}"),
        Err(error) => format!("error before load: {error:?}"),
    }
}

/// The billing example at `ess/15` with `types` added to the invoice domain and `CancelInvoice`
/// taking `inputs` after its `invoice_id`, plus any `extra` changes.
fn billing_with(types: &str, inputs: &str, extra: &[(&str, &str, &str)]) -> Result<EssIr, String> {
    let types_after = format!("types:\n{types}\n");
    let input_after = format!(
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n{inputs}\n    outcomes:\n      - name: cancelled\n"
    );
    let mut changes = vec![
        ("system.yaml", "format: ess/1\n", "format: ess/15\n"),
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
        ("domains/invoice.yaml", "types:\n", types_after.as_str()),
        ("domains/invoice.yaml", KEPT_INPUT, input_after.as_str()),
    ];
    changes.extend(extra.iter().copied());
    try_compile(&billing_fixture(), &changes)
}

/// Either the lowering refuses, every diagnostic naming `CancelInvoice`'s input `field`, or the
/// lowered update takes each `(input, expected)` branch: `expected` is `cancelled` for an input ESS
/// admits, or a prefix of entity-core's refusal (`error: PreconditionFailed`) for one it does not.
fn holds_or_refuses_by_name(ir: &EssIr, field: &str, cases: &[(Value, &str)]) {
    let lowered = match lower_billing(ir) {
        Ok(lowered) => lowered,
        Err(refused) => {
            let refused = refused.into_vec();
            let at = format!("billing.invoice.CancelInvoice.input.{field}");
            assert!(
                !refused.is_empty()
                    && refused
                        .iter()
                        .all(|diagnostic| diagnostic.path.starts_with(&at)),
                "{STORY}: a refusal names the input `{at}`: {refused:#?}"
            );
            return;
        }
    };
    for (extra, expected) in cases {
        let mut input = json!({"invoice_id": ID});
        for (key, value) in extra.as_object().expect("an object") {
            input[key] = value.clone();
        }
        let taken = cancel_with(&lowered, &input);
        assert!(
            taken.starts_with(expected),
            "{STORY}: `CancelInvoice` with {input} takes `{taken}`, expected `{expected}…`"
        );
    }
}

const ADMITTED: &str = "cancelled";
const NOT_ADMITTED: &str = "error: PreconditionFailed";

/// An unstored input whose type is a `Timestamp` newtype bounded below, `value >=
/// "2000-01-01T00:00:00Z"`. ESS orders it as an instant and admits `2026-01-01T00:00:00Z`; the
/// lowered precondition must too. Invariants are lowered untyped (`lower_predicate`), so the
/// ordering is a plain `compare` over two strings, which entity-core answers `Unknown` without a
/// declared scale: every request, valid or not, fails as `PreconditionUnobservable`. A typed guard
/// lowers the same ordering to `after`.
#[test]
fn adv_d343_u2_a_timestamp_ordering_invariant_of_an_update_input_admits_a_valid_instant() {
    let ir = billing_with(
        "  - name: billing.invoice.NotBefore\n    kind: newtype\n    of: Timestamp\n    invariants:\n      - value >= \"2000-01-01T00:00:00Z\"\n",
        "      - name: since\n        type: billing.invoice.NotBefore\n",
        &[],
    )
    .unwrap_or_else(|errors| panic!("the instant input compiles: {errors}"));
    holds_or_refuses_by_name(
        &ir,
        "since",
        &[
            (json!({"since": "2026-01-01T00:00:00Z"}), ADMITTED),
            (json!({"since": "1999-01-01T00:00:00Z"}), NOT_ADMITTED),
        ],
    );
}

/// The same gap over text: a newtype `Code` of `String` with `value >= "m"`. ESS orders text by its
/// bytes (ess#94), and a guard with that ordering is refused by name because entity-core has no
/// byte-order operator; the type invariant lowered untyped becomes a string `compare` entity-core
/// answers `Unknown`, so `"q"`, which ESS admits, fails the precondition.
#[test]
fn adv_d343_u2_a_text_ordering_invariant_of_an_update_input_admits_a_valid_value() {
    let ir = billing_with(
        "  - name: billing.invoice.Code\n    kind: newtype\n    of: String\n    invariants:\n      - value >= \"m\"\n",
        "      - name: code\n        type: billing.invoice.Code\n",
        &[],
    )
    .unwrap_or_else(|errors| panic!("the code input compiles: {errors}"));
    holds_or_refuses_by_name(
        &ir,
        "code",
        &[
            (json!({"code": "q"}), ADMITTED),
            (json!({"code": "a"}), NOT_ADMITTED),
        ],
    );
}

/// Nesting: a struct input whose members are a newtype with a text-length invariant, one of them
/// `Optional`. Each member's rule holds over `$args.input.wrap.<member>`; an absent optional member
/// holds every rule.
#[test]
fn adv_d343_u2_a_newtype_member_of_a_struct_input_is_held_including_an_optional_one() {
    let ir = billing_with(
        "  - name: billing.invoice.Label\n    kind: newtype\n    of: String\n    invariants: [value.count <= 3]\n\n  - name: billing.invoice.Wrap\n    kind: struct\n    fields:\n      - name: label\n        type: billing.invoice.Label\n      - name: maybe\n        type: Optional<billing.invoice.Label>\n",
        "      - name: wrap\n        type: billing.invoice.Wrap\n",
        &[],
    )
    .unwrap_or_else(|errors| panic!("the wrap input compiles: {errors}"));
    holds_or_refuses_by_name(
        &ir,
        "wrap",
        &[
            (json!({"wrap": {"label": "abc"}}), ADMITTED),
            (json!({"wrap": {"label": "abc", "maybe": "ab"}}), ADMITTED),
            (json!({"wrap": {"label": "abcd"}}), NOT_ADMITTED),
            (
                json!({"wrap": {"label": "abc", "maybe": "abcd"}}),
                NOT_ADMITTED,
            ),
        ],
    );
}

/// Lists: an `Optional<List<Tag>>` input, `Tag` a newtype with a `prefix:`. Every element holds
/// the prefix; the empty list and an absent list hold it vacuously.
#[test]
fn adv_d343_u2_a_prefix_on_list_elements_of_an_optional_update_input_is_held() {
    let ir = billing_with(
        "  - name: billing.invoice.Tag\n    kind: newtype\n    of: String\n    prefix: \"t-\"\n",
        "      - name: tags\n        type: Optional<List<billing.invoice.Tag>>\n",
        &[],
    )
    .unwrap_or_else(|errors| panic!("the tags input compiles: {errors}"));
    holds_or_refuses_by_name(
        &ir,
        "tags",
        &[
            (json!({}), ADMITTED),
            (json!({"tags": []}), ADMITTED),
            (json!({"tags": ["t-a", "t-b"]}), ADMITTED),
            (json!({"tags": ["t-a", "x-b"]}), NOT_ADMITTED),
        ],
    );
}

/// Several rules on one type, at two layers of a newtype chain: an outer `Short` over an inner
/// `Prefixed`, with a `prefix:`, a lower and an upper length bound. Each one refuses on its own.
#[test]
fn adv_d343_u2_several_rules_across_a_newtype_chain_each_refuse_on_their_own() {
    let ir = billing_with(
        "  - name: billing.invoice.Prefixed\n    kind: newtype\n    of: String\n    prefix: \"x\"\n    invariants: [value.count >= 2]\n\n  - name: billing.invoice.Short\n    kind: newtype\n    of: billing.invoice.Prefixed\n    invariants: [value.count <= 3]\n",
        "      - name: code\n        type: billing.invoice.Short\n",
        &[],
    )
    .unwrap_or_else(|errors| panic!("the chained input compiles: {errors}"));
    holds_or_refuses_by_name(
        &ir,
        "code",
        &[
            (json!({"code": "xab"}), ADMITTED),
            (json!({"code": "x"}), NOT_ADMITTED),
            (json!({"code": "xabc"}), NOT_ADMITTED),
            (json!({"code": "yab"}), NOT_ADMITTED),
        ],
    );
}

/// A creation with two accepting branches, each writing a different input into `Child.note:
/// Shared`: `tagged` stores `tag`, `completed` stores `note`. Each input is stored on one branch
/// only, so on the other branch ESS still rejects `"abcd"` and entity-core holds no rule over it:
/// both inputs are refused by name. No case of the unit separates "every accepting branch" from
/// "some accepting branch" in `stored_on_every_accepting_branch`; this one does.
#[test]
fn adv_d343_u2_a_creation_input_stored_on_one_of_two_accepting_branches_is_refused_by_name() {
    let contract = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let ir = try_compile(
        &contract,
        &[
            ("system.yaml", "format: ess/4\n", "format: ess/15\n"),
            (
                "domains/local.yaml",
                "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n",
                "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n    invariants: [value.count <= 3]\n",
            ),
            (
                "domains/local.yaml",
                "      - name: note\n        type: contract.local.Shared\n    response:",
                "      - name: note\n        type: contract.local.Shared\n      - name: tag\n        type: contract.local.Shared\n    response:",
            ),
            (
                "domains/local.yaml",
                "    outcomes:\n      - name: completed\n        creates: contract.local.Child\n",
                "    outcomes:\n      - name: tagged\n        when: tag == \"t\"\n        creates: contract.local.Child\n        instance: child_id\n        emits:\n          - contract.local.PrivateEmission\n        payload:\n          contract.local.PrivateEmission:\n            child_id: {generated: true}\n            receipt: {response: receipt}\n            note: input.tag\n            marker: exact-marker-007\n            optional_receipt: {response: optional_receipt}\n        sets:\n          owner_id: input.owner_id\n          note: input.tag\n        summary: A tagged child.\n      - name: completed\n        creates: contract.local.Child\n",
            ),
            (
                "wiring.yaml",
                "      note: event.note\n    delivery: at_most_once",
                "      note: event.note\n      tag: abc\n    delivery: at_most_once",
            ),
        ],
    )
    .unwrap_or_else(|errors| panic!("the two-branch creation compiles: {errors}"));
    let plan = SynthesisPlan::of(&ir);
    let service = extract(
        &ir,
        &plan,
        &ComponentName::new("local-service").expect("component name"),
    )
    .expect("service extracts");
    let refused = lower(
        &service,
        &LoweringOptions {
            definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
                .iter()
                .map(|entry| (name(entry), NonZeroU32::new(1).expect("nonzero")))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .err()
    .unwrap_or_else(|| {
        panic!("{STORY}: an input stored on one accepting branch of two is refused by name")
    })
    .into_vec();
    let named = refused
        .iter()
        .filter(|diagnostic| diagnostic.code == LoweringCode::InputInvariantUnsupported)
        .map(|diagnostic| diagnostic.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        named,
        [
            "contract.local.Run.input.note",
            "contract.local.Run.input.tag"
        ],
        "{STORY}: {refused:#?}"
    );
}

/// One type as the unstored input of a creation and of an update. The creation is refused by name,
/// naming only its own input; the update's input is not named, because it lowers to a precondition.
#[test]
fn adv_d343_u2_a_type_shared_by_a_creation_and_an_update_refuses_only_the_creation_input() {
    let ir = billing_with(
        "  - name: billing.invoice.Label\n    kind: newtype\n    of: String\n    invariants: [value.count <= 3]\n",
        "      - name: label\n        type: billing.invoice.Label\n",
        &[(
            "domains/invoice.yaml",
            "        type: billing.invoice.Money\n\n    # Two outcomes",
            "        type: billing.invoice.Money\n      - name: tag\n        type: billing.invoice.Label\n\n    # Two outcomes",
        )],
    )
    .unwrap_or_else(|errors| panic!("the shared label compiles: {errors}"));
    let refused = lower_billing(&ir)
        .err()
        .unwrap_or_else(|| panic!("{STORY}: the creation's unstored `tag` is refused by name"))
        .into_vec();
    assert_eq!(
        refused
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.as_str()))
            .collect::<Vec<_>>(),
        [(
            LoweringCode::InputInvariantUnsupported,
            "billing.invoice.CreateInvoice.input.tag"
        )],
        "{STORY}: {refused:#?}"
    );
}
