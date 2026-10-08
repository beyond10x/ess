//! Adversary pass 1 on `story:entity-runtime-lowers-alphabet-and-text-count`: a lowered definition
//! entity-core accepts must decide every value the way ESS does. Each case compiles a focused
//! specification, lowers it, and asks entity-core; where ESS's own answer is computable here, the
//! case asks `ess_primitives`' evaluator for it rather than writing it down.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use entity_core::{
    identity, CoreError, EntityInstance, Evaluation, FieldKind, LoadedDecision, PreloadDecision,
    Registry, Runtime,
};
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{
    lower, BoundTarget, CommandBinding, LoweredService, LoweringCode, LoweringDiagnostics,
    LoweringOptions,
};
use ess_primitives::facts::{FactStore, FactValue};
use ess_primitives::predicate::{Predicate, Truth};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

const SHARED: &str = "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n";
const SHARED_ALPHABET: &str = "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n    alphabet: \"kep xactly\"\n";
const KEPT_INPUT: &str = "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n";

fn contract_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract")
}

fn billing_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing")
}

/// The fixture under `base` with each `(file, before, after)` applied once, in order, compiled; or
/// what ESS refused it with.
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

/// The focused contract fixture at `ess/15` with `changes` applied.
fn contract(changes: &[(&str, &str, &str)]) -> Result<EssIr, String> {
    let mut all = vec![("system.yaml", "format: ess/4\n", "format: ess/15\n")];
    all.extend(changes.iter().copied());
    try_compile(&contract_fixture(), &all)
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("qualified name")
}

fn options(entries: &[&str]) -> LoweringOptions {
    LoweringOptions {
        definition_versions: entries
            .iter()
            .map(|entry| (name(entry), NonZeroU32::new(1).expect("nonzero")))
            .collect(),
        scales: BTreeMap::new(),
    }
}

fn lower_component(
    ir: &EssIr,
    component: &str,
    entities: &[&str],
) -> Result<LoweredService, LoweringDiagnostics> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(
        ir,
        &plan,
        &ComponentName::new(component).expect("component name"),
    )
    .expect("service extracts");
    lower(&service, &options(entities))
}

fn lower_contract(ir: &EssIr) -> Result<LoweredService, LoweringDiagnostics> {
    lower_component(
        ir,
        "local-service",
        &["contract.foreign.Owner", "contract.local.Child"],
    )
}

fn lower_billing(ir: &EssIr) -> Result<LoweredService, LoweringDiagnostics> {
    lower_component(
        ir,
        "invoice-service",
        &["billing.invoice.Account", "billing.invoice.Invoice"],
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

/// The arguments of the focused `Run`, every bound slot filled the way `tests/lowering.rs` fills it.
fn run_arguments(binding: &CommandBinding) -> Value {
    let mut bound = serde_json::Map::new();
    for (slot, value) in &binding.slots {
        let supplied = match &value.target {
            BoundTarget::ExternalEvidence { .. } => Some(json!(false)),
            BoundTarget::LogicalIdentity { .. } => {
                Some(json!("278f4f3a-c8b8-4e86-9a16-2c385910fc68"))
            }
            BoundTarget::ResponseField { field, .. } if field == "receipt" => {
                Some(json!("receipt-7"))
            }
            BoundTarget::ResponseField { field, .. } if field == "optional_receipt" => {
                Some(json!("optional-receipt-7"))
            }
            BoundTarget::EntityField { field, .. } if field == "memo" => Some(json!("memo-7")),
            BoundTarget::EventField { field, .. } if field == "optional_a" => {
                Some(json!("generated-a-7"))
            }
            BoundTarget::EventField { field, .. } if field == "optional_b" => None,
            target => panic!("unexpected focused bound target: {target:?}"),
        };
        if let Some(supplied) = supplied {
            bound.insert(format!("b{:08}", slot.index()), supplied);
        }
    }
    json!({
        "input": {
            "owner_id": "78e993a6-ac0d-42c0-a05c-8f2ee1898ee9",
            "note": "keep exactly"
        },
        "bound": bound
    })
}

/// What entity-core decides for `Run` creating a `Child`, with `input` merged over the defaults.
fn create_child(lowered: &LoweredService, input: &Value) -> Result<Evaluation, CoreError> {
    let registry = registry(lowered);
    let runtime = Runtime::new(&registry);
    let binding = &lowered.bindings().commands()[&name("contract.local.Run")];
    let logical_id = json!("278f4f3a-c8b8-4e86-9a16-2c385910fc68");
    let storage_id = identity::address(FieldKind::String, &logical_id).expect("identity address");
    let mut arguments = run_arguments(binding);
    for (key, value) in input.as_object().expect("input is an object") {
        arguments["input"][key] = value.clone();
    }
    runtime.decide_create("contract.local.Child", 1, storage_id, arguments)
}

fn accepted(outcome: Result<Evaluation, CoreError>) -> Result<(), String> {
    match outcome {
        Ok(evaluation) => evaluation
            .into_decision()
            .map(|_| ())
            .map_err(|refusal| format!("refused: {refusal:?}")),
        Err(error) => Err(format!("error: {error:?}")),
    }
}

fn outside_the_alphabet(outcome: &Result<Evaluation, CoreError>) -> bool {
    matches!(
        outcome,
        Err(CoreError::Validation(errors))
            if format!("{errors:?}").contains("is not in the alphabet")
    )
}

/// ESS's own answer for `predicate` with the stored `note` and the sent `input.note`.
fn ess_truth(predicate: &str, facts: &[(&str, &str)]) -> Truth {
    let mut store = FactStore::new();
    for (path, text) in facts {
        store.set_path(path, FactValue::text(*text));
    }
    Predicate::parse_expression(predicate)
        .unwrap_or_else(|error| panic!("{predicate} parses: {error:?}"))
        .evaluate(&store)
}

/// The billing example at `ess/15`, `CancelInvoice` taking a `note: String` input and refusing as
/// `posted` when `predicate` holds over the subject.
fn cancel_ir(predicate: &str) -> EssIr {
    let refusal = format!("      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: note\n        type: String\n\n    outcomes:\n      - name: posted\n        when_subject: {{predicate: '{predicate}'}}\n        error: billing.invoice.InvoiceStateConflict\n      - name: cancelled\n");
    try_compile(
        &billing_fixture(),
        &[
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
            ("domains/invoice.yaml", KEPT_INPUT, refusal.as_str()),
        ],
    )
    .unwrap_or_else(|errors| panic!("{predicate} compiles: {errors}"))
}

fn invoice_row(fields: &[(&str, Value)]) -> EntityInstance {
    let logical_id = json!("b404a1e8-9360-4af5-a0ac-8a483adfa225");
    let mut row: serde_json::Map<String, Value> = serde_json::from_value(json!({
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
    for (field, value) in fields {
        row.insert((*field).to_owned(), value.clone());
    }
    EntityInstance {
        entity: "billing.invoice.Invoice".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &logical_id).expect("identity address"),
        lifecycle_state: "Draft".to_owned(),
        revision: 1,
        fields: row,
    }
}

/// What the lowered `CancelInvoice` takes for `row`, called with `sent` as the input `note`:
/// `refused…` when `posted` is taken, `cancelled` when it is not.
fn cancel_taken(lowered: &LoweredService, sent: &str, row: &EntityInstance) -> String {
    let registry = registry(lowered);
    let runtime = Runtime::new(&registry);
    // `CancelInvoice` has no slot either branch fills before load, as `tests/lowering.rs` sends it.
    let arguments = json!({
        "input": {"invoice_id": "b404a1e8-9360-4af5-a0ac-8a483adfa225", "note": sent},
        "bound": {}
    });
    match runtime.decide_before_load(
        &row.entity,
        row.version,
        row.id.clone(),
        "billing.invoice.CancelInvoice",
        arguments,
    ) {
        Ok(PreloadDecision::Load(prepared)) => match prepared.select_with(row) {
            Ok(LoadedDecision::NeedsFulfillment(prepared)) => prepared.outcome().to_owned(),
            Ok(LoadedDecision::Complete(evaluation)) => match evaluation.into_decision() {
                Ok(_) => "accepted without fulfillment".to_owned(),
                Err(refusal) => format!("refused: {refusal:?}"),
            },
            Err(error) => format!("error: {error:?}"),
        },
        // A guard over the arguments alone selects its refusing branch before the row is loaded.
        Ok(PreloadDecision::Refused(refusal)) => format!("refused before load: {refusal:?}"),
        Err(error) => format!("error before load: {error:?}"),
    }
}

fn expected_for(truth: Truth) -> &'static str {
    match truth {
        Truth::True => "refused",
        Truth::False => "cancelled",
        Truth::Unknown => "unknown",
    }
}

/// ESS checks a `sets:` literal against a declared prefix and not against an alphabet, so it admits
/// `note: "kept!"` into a `Shared` whose alphabet has no `!`, and its evaluator stores the literal.
/// The lowering puts the alphabet on the stored field, and entity-core validates the row it is about
/// to write: the creation's one accepting branch then fails for every request. Either the lowering
/// refuses the literal by name, or the runtime takes the branch ESS takes.
#[test]
fn adv_a_sets_literal_outside_the_alphabet_does_not_lower_to_a_creation_entity_core_always_refuses()
{
    let ir = match contract(&[
        ("domains/local.yaml", SHARED, SHARED_ALPHABET),
        (
            "domains/local.yaml",
            "        sets:\n          owner_id: input.owner_id\n          note: input.note\n",
            "        sets:\n          owner_id: input.owner_id\n          note: \"kept!\"\n",
        ),
    ]) {
        Ok(ir) => ir,
        // ESS refusing the literal closes the gap where it starts.
        Err(errors) => {
            assert!(errors.contains("alphabet"), "{errors}");
            return;
        }
    };
    match lower_contract(&ir) {
        Err(diagnostics) => {
            let refused = diagnostics.into_vec();
            assert!(
                refused
                    .iter()
                    .any(|diagnostic| diagnostic.code == LoweringCode::AlphabetUnsupported),
                "{refused:#?}"
            );
        }
        Ok(lowered) => {
            let outcome = create_child(&lowered, &json!({"note": "keep exactly"}));
            assert!(
                !outside_the_alphabet(&outcome),
                "ESS admits `sets: note: \"kept!\"` and takes `completed`; the lowered creation \
                 fails every request: {outcome:?}"
            );
            accepted(outcome).expect("the lowered creation takes `completed` as ESS does");
        }
    }
}

/// ESS checks nested alphabets pairwise: each layer shares a character with each layer it wraps.
/// `ac` over `bc` over `ab` passes that check and has no character all three hold, so the only
/// value of the type is the empty text (ESS's own witness answers `unsupported` for it,
/// `ess-conformance/src/interpret/protected.rs`). The lowering intersects them to `""`, an
/// alphabet entity-core refuses at registration. A target refusal is the lowering emitting a
/// definition that is not one; a named refusal, or a lowering entity-core decides as ESS does,
/// is what the contract asks for.
#[test]
fn adv_alphabets_that_overlap_pairwise_and_share_no_character_are_not_a_target_definition_refusal()
{
    let ir = match contract(&[(
        "domains/local.yaml",
        SHARED,
        "  - name: contract.local.Base\n    kind: newtype\n    of: String\n    alphabet: \"ab\"\n\n  - name: contract.local.Middle\n    kind: newtype\n    of: contract.local.Base\n    alphabet: \"bc\"\n\n  - name: contract.local.Shared\n    kind: newtype\n    of: contract.local.Middle\n    alphabet: \"ac\"\n",
    )]) {
        Ok(ir) => ir,
        Err(errors) => {
            assert!(errors.contains("alphabet"), "{errors}");
            return;
        }
    };
    match lower_contract(&ir) {
        Err(diagnostics) => {
            let refused = diagnostics.into_vec();
            assert!(
                refused
                    .iter()
                    .all(|diagnostic| diagnostic.code != LoweringCode::TargetDefinitionRefused)
                    && refused
                        .iter()
                        .any(|diagnostic| diagnostic.code == LoweringCode::AlphabetUnsupported),
                "a valid specification lowered to a definition entity-core refuses: {refused:#?}"
            );
        }
        Ok(lowered) => {
            accepted(create_child(&lowered, &json!({"note": ""})))
                .expect("the empty text is the one value every layer admits");
            let outcome = create_child(&lowered, &json!({"note": "a"}));
            assert!(
                matches!(outcome, Err(CoreError::Validation(_))),
                "`a` is outside `bc`: {outcome:?}"
            );
        }
    }
}

/// Every ordering and equality over a text length, at, below and above each operand's length, over
/// the stored row and over the arguments, decided by entity-core and by ESS's evaluator. The stored
/// text is two graphemes, four scalar values and six bytes; the sent one is three of each scalar,
/// six bytes.
#[test]
fn adv_text_length_comparisons_decide_like_the_ess_evaluator_at_every_boundary() {
    let stored = "e\u{301}e\u{301}";
    let sent = "\u{e9}\u{e9}\u{e9}";
    let row = invoice_row(&[("note", json!(stored))]);
    let mut mismatches = Vec::new();
    for root in ["note", "input.note"] {
        for op in ["<", "<=", "==", "!=", ">=", ">"] {
            for bound in [2, 3, 4, 5] {
                let predicate = format!("{root}.count {op} {bound}");
                let lowered = lower_billing(&cancel_ir(&predicate))
                    .unwrap_or_else(|refused| panic!("{predicate} lowers: {refused:?}"));
                let ess = ess_truth(&predicate, &[("note", stored), ("input.note", sent)]);
                let taken = cancel_taken(&lowered, sent, &row);
                if !taken.starts_with(expected_for(ess)) {
                    mismatches.push(format!("{predicate}: ESS {ess:?}, entity-core {taken}"));
                }
            }
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

/// ESS admits an ordinal list read (`lines.0`), so a guard reads the length of a list element's text
/// without a quantifier. The subset table says a text length "in a list or map element" is
/// refused; whichever way the lowering goes, a lowered guard decides as ESS's evaluator does.
#[test]
fn adv_an_ordinal_list_element_text_length_decides_like_the_ess_evaluator() {
    let predicate = "lines.0.description.count > 3";
    let lowered = match lower_billing(&cancel_ir(predicate)) {
        Err(diagnostics) => {
            let refused = diagnostics.into_vec();
            assert!(
                refused
                    .iter()
                    .any(|diagnostic| diagnostic.code == LoweringCode::TextLengthUnsupported),
                "{refused:#?}"
            );
            return;
        }
        Ok(lowered) => lowered,
    };
    let mut mismatches = Vec::new();
    for description in ["e\u{301}e\u{301}", "\u{e9}\u{e9}\u{e9}", "abcd", "abc"] {
        let row = invoice_row(&[(
            "lines",
            json!([{
                "description": description,
                "quantity": 1,
                "unit_price": {"amount": 1, "currency": "EUR"}
            }]),
        )]);
        let ess = ess_truth(predicate, &[("lines.0.description", description)]);
        let taken = cancel_taken(&lowered, "sent", &row);
        if !taken.starts_with(expected_for(ess)) {
            mismatches.push(format!("{description:?}: ESS {ess:?}, entity-core {taken}"));
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

/// A String alphabet reached through a list element, a map value, a union payload and an optional
/// struct member: entity-core refuses a character outside it in each position and nowhere else.
#[test]
fn adv_an_alphabet_holds_through_list_map_union_and_optional_member() {
    let ir = contract(&[
        (
            "domains/local.yaml",
            SHARED,
            "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n    alphabet: \"kep xactly\"\n\n  - name: contract.local.Status\n    kind: union\n    tag: kind\n    variants:\n      open: String\n      closed: contract.local.Shared\n\n  - name: contract.local.Wrap\n    kind: struct\n    fields:\n      - name: inner\n        type: Optional<contract.local.Shared>\n",
        ),
        (
            "domains/local.yaml",
            "      - name: memo\n        type: Optional<String>\n",
            "      - name: memo\n        type: Optional<String>\n      - name: tags\n        type: Optional<List<contract.local.Shared>>\n      - name: bag\n        type: Optional<Map<String, contract.local.Shared>>\n      - name: status\n        type: Optional<contract.local.Status>\n      - name: wrap\n        type: Optional<contract.local.Wrap>\n",
        ),
        (
            "domains/local.yaml",
            "      - name: note\n        type: contract.local.Shared\n    response:",
            "      - name: note\n        type: contract.local.Shared\n      - name: tags\n        type: Optional<List<contract.local.Shared>>\n      - name: bag\n        type: Optional<Map<String, contract.local.Shared>>\n      - name: status\n        type: Optional<contract.local.Status>\n      - name: wrap\n        type: Optional<contract.local.Wrap>\n    response:",
        ),
        (
            "domains/local.yaml",
            "          note: input.note\n        summary:",
            "          note: input.note\n          tags: input.tags\n          bag: input.bag\n          status: input.status\n          wrap: input.wrap\n        summary:",
        ),
    ])
    .unwrap_or_else(|errors| panic!("the positions compile: {errors}"));
    let lowered = lower_contract(&ir).unwrap_or_else(|refused| panic!("{refused:?}"));
    let inside = json!({
        "tags": ["kept"],
        "bag": {"k": "kept"},
        "status": {"kind": "closed", "value": "kept"},
        "wrap": {"inner": "kept"}
    });
    accepted(create_child(&lowered, &inside)).expect("every position inside the alphabet");
    let mut open = inside.clone();
    open["status"] = json!({"kind": "open", "value": "kept!"});
    open["wrap"] = json!({});
    accepted(create_child(&lowered, &open)).expect("an `open` String and an absent member");
    let mut mismatches = Vec::new();
    for (field, value) in [
        ("tags", json!(["kept", "kept!"])),
        ("bag", json!({"k": "kept!"})),
        ("status", json!({"kind": "closed", "value": "kept!"})),
        ("wrap", json!({"inner": "kept!"})),
    ] {
        let mut input = inside.clone();
        input[field] = value;
        let outcome = create_child(&lowered, &input);
        if !outside_the_alphabet(&outcome) {
            mismatches.push(format!("{field}: {outcome:?}"));
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

/// The response refusal still names an alphabet reached through an optional struct member of a
/// response field, and only that one.
#[test]
fn adv_a_response_alphabet_behind_an_optional_struct_member_is_refused_by_name() {
    let ir = contract(&[
        (
            "domains/local.yaml",
            SHARED,
            "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n    alphabet: \"kep xactly\"\n\n  - name: contract.local.Wrap\n    kind: struct\n    fields:\n      - name: inner\n        type: Optional<contract.local.Shared>\n",
        ),
        (
            "domains/local.yaml",
            "      - name: optional_receipt\n        type: Optional<String>\n    outcomes:",
            "      - name: optional_receipt\n        type: Optional<String>\n      - name: wrapped\n        type: Optional<contract.local.Wrap>\n    outcomes:",
        ),
    ])
    .unwrap_or_else(|errors| panic!("the response compiles: {errors}"));
    let refused = lower_contract(&ir)
        .expect_err("a response alphabet is not enforced by entity-core")
        .into_vec();
    assert_eq!(
        refused
            .iter()
            .filter(|diagnostic| diagnostic.code == LoweringCode::AlphabetUnsupported)
            .map(|diagnostic| diagnostic.path.as_str())
            .collect::<Vec<_>>(),
        ["contract.local.Run.response.wrapped.inner"],
        "{refused:#?}"
    );
}

/// A String type's `value.count` invariant on an input the command does not store. ESS admits an
/// input only when its type's invariants hold, so `"abcd"` is not a `Label`; the lowering puts a
/// type's invariants on stored fields only (`lower_nominal_invariants` runs for the entity's
/// identity and fields), so entity-core takes the creation with it.
///
/// That gap predates the alphabet and text-count lowering and covers every invariant of an
/// unstored input's type, so it is `story:entity-runtime-lowering-enforces-input-type-invariants`.
/// This case pins today's behaviour: the invariant is not enforced. When that story lands, flip it
/// to assert that entity-core refuses `"abcd"` (or that the lowering refuses the rule by name).
#[test]
fn adv_an_input_only_text_length_invariant_is_dropped_by_the_lowering_today() {
    let ir = contract(&[
        (
            "domains/local.yaml",
            SHARED,
            "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n\n  - name: contract.local.Label\n    kind: newtype\n    of: String\n    invariants: [value.count <= 3]\n",
        ),
        (
            "domains/local.yaml",
            "      - name: note\n        type: contract.local.Shared\n    response:",
            "      - name: note\n        type: contract.local.Shared\n      - name: label\n        type: Optional<contract.local.Label>\n    response:",
        ),
    ])
    .unwrap_or_else(|errors| panic!("the input compiles: {errors}"));
    assert_eq!(
        ess_truth("value.count <= 3", &[("value", "abcd")]),
        Truth::False,
        "ESS: `abcd` is not a `Label`"
    );
    let lowered = lower_contract(&ir).unwrap_or_else(|refused| {
        panic!(
            "today the lowering drops an unstored input's type invariants rather than refusing \
             them; if it now refuses them, \
             story:entity-runtime-lowering-enforces-input-type-invariants has landed and this \
             case flips: {refused:?}"
        )
    });
    accepted(create_child(&lowered, &json!({"label": "abc"}))).expect("`abc` is a `Label`");
    let outcome = create_child(&lowered, &json!({"label": "abcd"}));
    assert!(
        accepted(outcome).is_ok(),
        "entity-core now refuses `label: \"abcd\"`, which ESS does not admit as a `Label`: \
         story:entity-runtime-lowering-enforces-input-type-invariants has landed, so flip this case \
         to assert the refusal"
    );
}
