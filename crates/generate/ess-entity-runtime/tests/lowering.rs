//! Complete lowerer acceptance over real and focused ESS fixtures.
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use entity_core::{
    identity, EntityInstance, FieldKind, LoadedDecision, OperationFieldAction, PreloadDecision,
    Registry, Runtime, Semantics,
};
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{
    lower, BindingRequirement, BoundPresence, BoundTarget, InstanceBinding, LoweringCode,
    LoweringOptions, OperationFieldActions, OperationFieldCoordinate, RuntimeEntrypoint,
    ENTITY_RUNTIME_REVISION,
};
use ess_service_contract::{extract, ServiceIr};
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
}

fn compile_directory(base: &Path) -> EssIr {
    compile_changes(base, &[])
}

fn compile_replacing(base: &Path, target: &str, replacements: &[(&str, &str)]) -> EssIr {
    let changes = replacements
        .iter()
        .map(|(before, after)| (target, *before, *after))
        .collect::<Vec<_>>();
    compile_changes(base, &changes)
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
    let specification = Specification::assemble(parsed).expect("fixture validates");
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

fn options(entries: &[&str]) -> LoweringOptions {
    LoweringOptions {
        definition_versions: entries
            .iter()
            .map(|entry| (name(entry), NonZeroU32::new(1).expect("nonzero")))
            .collect(),
        scales: BTreeMap::new(),
    }
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

#[test]
fn billing_and_gatepass_lower_completely_deterministically_without_touching_source() {
    for (fixture, component, entities) in [
        (
            "billing",
            "invoice-service",
            vec!["billing.invoice.Account", "billing.invoice.Invoice"],
        ),
        ("gatepass", "pass-service", vec!["gatepass.visit.Visit"]),
    ] {
        let ir = compile_directory(&example(fixture));
        let source_bytes = ir.to_canonical_json();
        let source_digest = ir.source_digest();
        let plan = SynthesisPlan::of(&ir);
        let service = selected(&ir, &plan, component);
        let options = options(&entities);
        let first = lower(&service, &options).expect("complete real service lowers");
        let second = lower(&service, &options).expect("repeated lowering succeeds");

        assert_eq!(first, second);
        assert_eq!(first.target_revision(), ENTITY_RUNTIME_REVISION);
        assert_eq!(first.source_digest(), source_digest);
        assert_eq!(ir.to_canonical_json(), source_bytes);
        assert_eq!(ir.source_digest(), source_digest);
        assert_eq!(
            first.definitions().keys().cloned().collect::<Vec<_>>(),
            entities.iter().map(|entry| name(entry)).collect::<Vec<_>>()
        );
        for definition in first.definitions().values() {
            serde_json::to_vec(definition.as_definition())
                .expect("target has deterministic canonical-ready bytes");
        }
        let _ = registry(&first);
    }
}

#[test]
fn complete_real_fixture_inventory_keeps_versions_slots_events_effects_and_fulfillment() {
    let billing = compile_directory(&example("billing"));
    let billing_plan = SynthesisPlan::of(&billing);
    let lowered = lower(
        &selected(&billing, &billing_plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
    .expect("billing lowers");
    assert_eq!(
        lowered.definitions()[&name("billing.invoice.Account")].semantics,
        Semantics::Service1
    );
    let invoice = &lowered.definitions()[&name("billing.invoice.Invoice")];
    assert_eq!(invoice.semantics, Semantics::Service3);
    assert_eq!(
        invoice.create.outcomes[0]
            .emits
            .iter()
            .map(|event| event.event_type.as_str())
            .collect::<Vec<_>>(),
        ["billing.invoice.InvoiceCreated"]
    );
    assert!(invoice.create.outcomes[0]
        .set_if_present
        .contains_key("note"));
    assert!(invoice.create.outcomes[0]
        .set_if_present
        .contains_key("issued_at"));
    let issue = &invoice.operations["billing.invoice.IssueInvoice"].outcomes[0];
    assert!(
        matches!(issue.effect, entity_core::OutcomeEffect::Moves { ref to, .. } if to == "Issued")
    );
    assert_eq!(issue.emits.len(), 1);
    assert!(issue.fulfills.contains_key("issued_at"));
    let pay = &invoice.operations["billing.invoice.PayInvoice"].outcomes[0];
    assert!(matches!(pay.effect, entity_core::OutcomeEffect::Moves { ref to, .. } if to == "Paid"));
    assert_eq!(pay.emits.len(), 1);
    assert_eq!(pay.fulfills.len(), 12);

    let issue_binding = &lowered.bindings().commands()[&name("billing.invoice.IssueInvoice")];
    assert_eq!(issue_binding.target.version.get(), 1);
    assert!(matches!(
        issue_binding.entrypoint,
        RuntimeEntrypoint::Operation { .. }
    ));
    assert!(
        matches!(issue_binding.instance, InstanceBinding::Supplied { ref input_field } if input_field == "invoice_id")
    );
    let issued_at = &issue_binding.operation_fields[&OperationFieldCoordinate {
        outcome: ess_domain::command::OutcomeName::new("issued").expect("outcome"),
        field: "issued_at".to_owned(),
    }];
    assert_eq!(issued_at.actions, OperationFieldActions::Optional);
    assert!(lowered
        .bindings()
        .requirements()
        .iter()
        .any(|requirement| matches!(
            requirement,
            BindingRequirement::OperationFieldPolicySupplied { command, target, .. }
                if command == &name("billing.invoice.IssueInvoice") && target.field == "issued_at"
        )));

    let gatepass = compile_directory(&example("gatepass"));
    let gatepass_plan = SynthesisPlan::of(&gatepass);
    let lowered = lower(
        &selected(&gatepass, &gatepass_plan, "pass-service"),
        &options(&["gatepass.visit.Visit"]),
    )
    .expect("gatepass lowers");
    let visit = &lowered.definitions()[&name("gatepass.visit.Visit")];
    assert_eq!(visit.semantics, Semantics::Service3);
    assert!(visit.create.outcomes[0]
        .set_if_present
        .contains_key("badge"));
    let admit = &visit.operations["gatepass.visit.AdmitVisitor"].outcomes[0];
    assert!(admit.fulfills.contains_key("badge"));
    assert_eq!(admit.emits[0].payload["badge"], json!("$args.input.badge"));
    let binding = &lowered.bindings().commands()[&name("gatepass.visit.AdmitVisitor")];
    assert_eq!(
        binding.operation_fields[&OperationFieldCoordinate {
            outcome: ess_domain::command::OutcomeName::new("admitted").expect("outcome"),
            field: "badge".to_owned(),
        }]
            .actions,
        OperationFieldActions::Optional
    );
    assert!(binding.slots.values().all(|value| matches!(
        value.presence,
        BoundPresence::Required | BoundPresence::Optional
    )));
}

fn invoice_fields() -> serde_json::Map<String, Value> {
    serde_json::from_value(json!({
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
    .expect("invoice fields")
}

#[test]
fn pay_invoice_refuses_before_load_then_selects_exact_subject_before_fulfillment() {
    let ir = compile_directory(&example("billing"));
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(
        &selected(&ir, &plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
    .expect("billing lowers");
    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let logical_id = json!("b404a1e8-9360-4af5-a0ac-8a483adfa225");
    let storage_id = identity::address(FieldKind::String, &logical_id).expect("identity address");

    let rejected = runtime
        .decide_before_load(
            "billing.invoice.Invoice",
            1,
            storage_id.clone(),
            "billing.invoice.PayInvoice",
            json!({"input": {"invoice_id": logical_id, "amount": {"amount": 0, "currency": "EUR"}}, "bound": {}}),
        )
        .expect("input-only selection succeeds");
    assert!(matches!(rejected, PreloadDecision::Refused(_)));

    let prepared = runtime
        .decide_before_load(
            "billing.invoice.Invoice",
            1,
            storage_id.clone(),
            "billing.invoice.PayInvoice",
            json!({"input": {"invoice_id": logical_id, "amount": {"amount": 25, "currency": "EUR"}}, "bound": {}}),
        )
        .expect("positive input is admitted");
    let PreloadDecision::Load(prepared) = prepared else {
        panic!("positive payment requires the exact subject")
    };
    assert_eq!(prepared.subject().id(), storage_id);
    let instance = EntityInstance {
        entity: "billing.invoice.Invoice".to_owned(),
        version: 1,
        id: storage_id,
        lifecycle_state: "Issued".to_owned(),
        revision: 3,
        fields: invoice_fields(),
    };
    let LoadedDecision::NeedsFulfillment(prepared) = prepared
        .select_with(&instance)
        .expect("loaded subject selects")
    else {
        panic!("selected payment requires explicit field actions")
    };
    assert_eq!(prepared.outcome(), "settled");
    let actions = prepared
        .requirements()
        .keys()
        .map(|field| (field.clone(), OperationFieldAction::Preserve))
        .collect();
    let decision = prepared
        .complete(actions)
        .expect("all exact actions complete the selected branch")
        .into_decision()
        .expect("accepted decision");
    assert_eq!(decision.instance.lifecycle_state, "Paid");
    assert_eq!(decision.events.len(), 1);
    assert_eq!(decision.events[0].event_type, "billing.invoice.InvoicePaid");
}

#[test]
fn input_diagnostics_accumulate_and_never_return_partial_output() {
    let ir = compile_directory(&example("billing"));
    let plan = SynthesisPlan::of(&ir);
    let service = selected(&ir, &plan, "invoice-service");
    let diagnostics = lower(
        &service,
        &LoweringOptions {
            definition_versions: BTreeMap::from([(
                name("billing.invoice.Other"),
                NonZeroU32::new(1).expect("nonzero"),
            )]),
            scales: BTreeMap::from([(name("billing.invoice.Outside"), BTreeMap::new())]),
        },
    )
    .expect_err("missing and extra coordinates refuse without partial output");
    assert_eq!(
        diagnostics
            .as_slice()
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        [
            LoweringCode::MissingDefinitionVersion,
            LoweringCode::MissingDefinitionVersion,
            LoweringCode::UnknownDefinitionVersion,
            LoweringCode::UnknownScaleEntity,
        ]
    );

    let email = selected(&ir, &plan, "email-service");
    let diagnostics = lower(&email, &options(&[])).expect_err("stateless commands remain explicit");
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|diagnostic| diagnostic.code == LoweringCode::StatelessCommandUnsupported));
}

fn codes(diagnostics: ess_entity_runtime::LoweringDiagnostics) -> Vec<LoweringCode> {
    diagnostics
        .into_vec()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn structural_boundary_diagnostics_name_nullable_elements_and_recursion() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let nullable = compile_replacing(
        &fixture,
        "domains/local.yaml",
        &[(
            "      - name: memo\n        type: Optional<String>\n    lifecycle:",
            "      - name: memo\n        type: Optional<String>\n      - name: nullable_items\n        type: List<Optional<String>>\n    lifecycle:",
        )],
    );
    let plan = SynthesisPlan::of(&nullable);
    let diagnostics = lower(
        &selected(&nullable, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect_err("nullable collection members cannot be represented by ER field presence");
    assert!(codes(diagnostics).contains(&LoweringCode::NullableElementUnsupported));

    let recursive = compile_replacing(
        &fixture,
        "domains/local.yaml",
        &[(
            "      - name: memo\n        type: Optional<String>\n    lifecycle:",
            "      - name: memo\n        type: Optional<String>\n      - name: recursive\n        type: contract.local.Node\n    lifecycle:",
        )],
    );
    let plan = SynthesisPlan::of(&recursive);
    let diagnostics = lower(
        &selected(&recursive, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect_err("recursive definitions cannot be embedded forever");
    assert!(codes(diagnostics).contains(&LoweringCode::RecursiveTypeUnsupported));
}

#[test]
fn operation_identity_and_optional_output_boundaries_are_explicit() {
    let billing_path = example("billing");
    let identity = compile_replacing(
        &billing_path,
        "domains/invoice.yaml",
        &[(
            "        payload:\n          billing.invoice.InvoiceIssued:\n            invoice_id: input.invoice_id\n        summary:",
            "        payload:\n          billing.invoice.InvoiceIssued:\n            invoice_id: input.invoice_id\n        sets:\n          invoice_id: input.invoice_id\n        summary:",
        )],
    );
    let plan = SynthesisPlan::of(&identity);
    let diagnostics = lower(
        &selected(&identity, &plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
    .expect_err("an operation may not rewrite its identity mirror");
    assert!(codes(diagnostics).contains(&LoweringCode::OperationIdentityMutationUnsupported));

    let optional = compile_replacing(
        &billing_path,
        "domains/invoice.yaml",
        &[
            (
                "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: issued",
                "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: supplied_issued_at\n        type: Optional<Timestamp>\n\n    outcomes:\n      - name: issued",
            ),
            (
                "        payload:\n          billing.invoice.InvoiceIssued:\n            invoice_id: input.invoice_id\n        summary:",
                "        payload:\n          billing.invoice.InvoiceIssued:\n            invoice_id: input.invoice_id\n        sets:\n          issued_at: input.supplied_issued_at\n        summary:",
            ),
        ],
    );
    let plan = SynthesisPlan::of(&optional);
    let diagnostics = lower(
        &selected(&optional, &plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
    .expect_err("conditional operation output needs an explicit post-selection action");
    assert!(codes(diagnostics).contains(&LoweringCode::OptionalBoundOutputUnsupported));
}

#[test]
fn target_validation_failures_are_returned_without_approximation() {
    let billing_path = example("billing");
    let billing = compile_directory(&billing_path);
    let plan = SynthesisPlan::of(&billing);
    let mut invalid = options(&["billing.invoice.Account", "billing.invoice.Invoice"]);
    invalid.scales.insert(
        name("billing.invoice.Invoice"),
        BTreeMap::from([(String::new(), Vec::new())]),
    );
    let diagnostics = lower(&selected(&billing, &plan, "invoice-service"), &invalid)
        .expect_err("invalid declared scales are returned as target validation diagnostics");
    assert!(codes(diagnostics).contains(&LoweringCode::TargetDefinitionRefused));
}

#[test]
fn command_shape_diagnostics_cover_mixed_multiple_and_subjectless_entrypoints() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let mixed = compile_changes(
        &fixture,
        &[
            (
                "domains/local.yaml",
                "  - name: contract.local.Run\n    input:\n      - name: owner_id\n        type: contract.foreign.OwnerId\n      - name: note",
                "  - name: contract.local.Run\n    input:\n      - name: owner_id\n        type: contract.foreign.OwnerId\n      - name: child_id\n        type: contract.local.ChildId\n      - name: note",
            ),
            (
                "domains/local.yaml",
                "      - name: rejected\n        external: an upstream authority rejects the request\n        error: contract.local.Rejected\n        summary: Nothing is stored or emitted.",
                "      - name: revised\n        when: note == special\n        updates: contract.local.Child\n        instance: child_id\n        emits:\n          - contract.local.AdminDone\n        summary: An existing child is revised.",
            ),
            (
                "wiring.yaml",
                "\n  - id: invoke-selected\n    when:\n      event: contract.foreign.Trigger\n    invoke:\n      command: contract.local.Run\n    mapping:\n      owner_id: event.owner_id\n      note: event.note\n    delivery: at_most_once\n    on_failure: drop\n",
                "\n",
            ),
        ],
    );
    let plan = SynthesisPlan::of(&mixed);
    let diagnostics = lower(
        &selected(&mixed, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect_err("one command cannot mix creation and existing-instance effects");
    assert!(codes(diagnostics).contains(&LoweringCode::MixedEntrypointUnsupported));

    let subjectless = compile_replacing(
        &fixture,
        "domains/local.yaml",
        &[(
            "        external: an upstream authority rejects the request\n        error: contract.local.Rejected\n        summary: Nothing is stored or emitted.",
            "        external: an upstream authority rejects the request\n        emits:\n          - contract.local.AdminDone\n        summary: An external fact emits without a subject.",
        )],
    );
    let plan = SynthesisPlan::of(&subjectless);
    let diagnostics = lower(
        &selected(&subjectless, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect_err("ordinary accepting branches require an entity subject");
    assert!(codes(diagnostics).contains(&LoweringCode::AcceptingOutcomeWithoutSubject));

    let multiple = compile_changes(
        &fixture,
        &[
            (
                "wiring.yaml",
                "        - contract.local.Run\n",
                "        - contract.local.Run\n        - contract.local.Admin\n",
            ),
            (
                "domains/local.yaml",
                "  - name: contract.local.AdminDone\n",
                "  - name: contract.local.AdminDone\n    fields:\n      - name: child_id\n        type: contract.local.ChildId\n",
            ),
            (
                "domains/local.yaml",
                "  - name: contract.local.Admin\n    outcomes:\n      - name: completed\n        emits:\n          - contract.local.AdminDone",
                "  - name: contract.local.Admin\n    outcomes:\n      - name: completed\n        creates: contract.local.Child\n        instance: child_id\n        emits:\n          - contract.local.AdminDone\n        payload:\n          contract.local.AdminDone:\n            child_id: {generated: true}",
            ),
        ],
    );
    let plan = SynthesisPlan::of(&multiple);
    let diagnostics = lower(
        &selected(&multiple, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect_err("an ER entity has one unnamed creation entrypoint");
    assert!(codes(diagnostics).contains(&LoweringCode::MultipleCreationCommands));
}

#[test]
fn command_shape_diagnostics_cover_multiple_targets_and_identity_bindings() {
    let billing_path = example("billing");
    let second_input = (
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: amount\n        type: billing.invoice.Money\n\n    outcomes:\n      - name: settled",
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: other_id\n        type: billing.invoice.InvoiceId\n      - name: amount\n        type: billing.invoice.Money\n\n    outcomes:\n      - name: settled",
    );
    let refusal = "      - name: rejected\n        error: billing.invoice.InvalidAmount\n        summary: The payment was not positive, so the invoice did not move.";

    let ambiguous = compile_replacing(
        &billing_path,
        "domains/invoice.yaml",
        &[
            second_input,
            (
                refusal,
                "      - name: rejected\n        updates: billing.invoice.Invoice\n        instance: other_id\n        emits:\n          - billing.invoice.InvoicePaid\n        payload:\n          billing.invoice.InvoicePaid:\n            invoice_id: input.other_id\n            amount: input.amount\n        summary: Another supplied invoice is revised.",
            ),
        ],
    );
    let plan = SynthesisPlan::of(&ambiguous);
    let diagnostics = lower(
        &selected(&ambiguous, &plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
    .expect_err("all existing-instance outcomes must select the same supplied identity");
    assert!(codes(diagnostics).contains(&LoweringCode::AmbiguousInstanceBinding));

    let spanning = compile_replacing(
        &billing_path,
        "domains/invoice.yaml",
        &[
            (
                second_input.0,
                "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: account_id\n        type: billing.invoice.AccountId\n      - name: amount\n        type: billing.invoice.Money\n\n    outcomes:\n      - name: settled",
            ),
            (
                refusal,
                "      - name: rejected\n        updates: billing.invoice.Account\n        instance: account_id\n        emits:\n          - billing.invoice.InvoicePaid\n        payload:\n          billing.invoice.InvoicePaid:\n            invoice_id: input.invoice_id\n            amount: input.amount\n        summary: The account is revised instead.",
            ),
        ],
    );
    let plan = SynthesisPlan::of(&spanning);
    let diagnostics = lower(
        &selected(&spanning, &plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
    .expect_err("one ER operation cannot span entity definitions");
    assert!(codes(diagnostics).contains(&LoweringCode::CommandSpansEntities));
}

#[test]
fn ordered_occurrences_exact_literals_and_shared_response_values_survive_lowering() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let ir = compile_directory(&fixture);
    let plan = SynthesisPlan::of(&ir);
    let service = selected(&ir, &plan, "local-service");
    let lowered = lower(
        &service,
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect("focused service lowers");
    let child = &lowered.definitions()[&name("contract.local.Child")];
    assert_eq!(child.semantics, Semantics::Service2);
    let completed = child
        .create
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "completed")
        .expect("completed branch remains present after default-branch normalization");
    assert_eq!(
        completed
            .emits
            .iter()
            .map(|event| event.event_type.as_str())
            .collect::<Vec<_>>(),
        [
            "contract.local.PrivateEmission",
            "contract.local.First",
            "contract.local.Second",
        ]
    );
    assert_eq!(
        completed.emits[0].payload["marker"],
        json!("exact-marker-007")
    );
    assert_eq!(
        completed.emits[0].payload["note"],
        json!("$args.input.note")
    );
    assert_eq!(
        completed.emits[2].payload["note"],
        json!("$args.input.note")
    );
    assert_eq!(completed.emits[1].payload, json!({}));
    assert_eq!(
        completed.emits[0].payload["receipt"],
        completed.responds["receipt"]
    );
    assert_eq!(
        completed.emits[0].payload_if_present["optional_receipt"],
        completed.responds_if_present["optional_receipt"]
    );
    assert_ne!(
        completed.emits[2].payload_if_present["optional_a"],
        completed.emits[2].payload_if_present["optional_b"]
    );
    let binding = &lowered.bindings().commands()[&name("contract.local.Run")];
    assert!(matches!(binding.instance, InstanceBinding::Created { .. }));
    assert_eq!(
        binding
            .slots
            .values()
            .filter(|value| matches!(value.target, ess_entity_runtime::BoundTarget::ResponseField { ref field, .. } if field == "receipt"))
            .count(),
        1
    );
    assert_eq!(
        binding
            .slots
            .values()
            .filter(|value| value.presence == BoundPresence::Optional)
            .count(),
        4
    );
    assert_eq!(
        lowered.bindings().source_capabilities(),
        service.capabilities().cloned().collect::<Vec<_>>()
    );
}

fn focused_create_arguments(binding: &ess_entity_runtime::CommandBinding, optional: bool) -> Value {
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
                optional.then(|| json!("optional-receipt-7"))
            }
            BoundTarget::EntityField { field, .. } if field == "memo" => {
                optional.then(|| json!("memo-7"))
            }
            BoundTarget::EventField { field, .. } if field == "optional_a" => {
                optional.then(|| json!("generated-a-7"))
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

#[test]
fn authored_dollar_literals_survive_an_actual_runtime_decision() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let authored = "$args.bound.b00000002";
    let ir = compile_replacing(
        &fixture,
        "domains/local.yaml",
        &[(
            "            marker: exact-marker-007",
            "            marker: '$args.bound.b00000002'",
        )],
    );
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(
        &selected(&ir, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect("focused service lowers");
    let completed = lowered.definitions()[&name("contract.local.Child")]
        .create
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "completed")
        .expect("completed outcome");
    assert_eq!(
        completed.emits[0].payload["marker"],
        json!("$$args.bound.b00000002"),
        "the ER document escapes authored data that otherwise has template syntax"
    );
    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let binding = &lowered.bindings().commands()[&name("contract.local.Run")];
    let logical_id = json!("278f4f3a-c8b8-4e86-9a16-2c385910fc68");
    let storage_id = identity::address(FieldKind::String, &logical_id).expect("identity address");

    let decision = runtime
        .decide_create(
            "contract.local.Child",
            1,
            storage_id,
            focused_create_arguments(binding, true),
        )
        .expect("runtime admits the lowered definition")
        .into_decision()
        .expect("default creation branch accepts");

    assert_eq!(decision.events[0].payload["marker"], json!(authored));
}

#[test]
fn optional_creation_event_and_response_values_share_presence_without_defaults() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let ir = compile_directory(&fixture);
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(
        &selected(&ir, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect("focused service lowers");
    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let binding = &lowered.bindings().commands()[&name("contract.local.Run")];
    let logical_id = json!("278f4f3a-c8b8-4e86-9a16-2c385910fc68");
    let storage_id = identity::address(FieldKind::String, &logical_id).expect("identity address");

    let absent = runtime
        .decide_create(
            "contract.local.Child",
            1,
            storage_id.clone(),
            focused_create_arguments(binding, false),
        )
        .expect("optional absence validates")
        .into_decision()
        .expect("default creation branch accepts");
    assert!(!absent.instance.fields.contains_key("memo"));
    assert!(!absent.events[0]
        .payload
        .as_object()
        .expect("payload")
        .contains_key("optional_receipt"));
    assert!(!absent.events[2]
        .payload
        .as_object()
        .expect("payload")
        .contains_key("optional_a"));
    assert!(!absent
        .record
        .response
        .as_ref()
        .expect("response")
        .contains_key("optional_receipt"));

    let present = runtime
        .decide_create(
            "contract.local.Child",
            1,
            storage_id,
            focused_create_arguments(binding, true),
        )
        .expect("optional presence validates")
        .into_decision()
        .expect("default creation branch accepts");
    assert_eq!(present.instance.fields["memo"], json!("memo-7"));
    assert_eq!(
        present.events[0].payload["optional_receipt"],
        json!("optional-receipt-7")
    );
    assert_eq!(
        present.events[0].payload["optional_receipt"],
        present.record.response.as_ref().expect("response")["optional_receipt"]
    );
    assert_eq!(
        present.events[2].payload["optional_a"],
        json!("generated-a-7")
    );
    assert!(!present.events[2]
        .payload
        .as_object()
        .expect("payload")
        .contains_key("optional_b"));
}

fn selected_fulfillment<'a>(
    runtime: &'a Runtime<'a>,
    instance: &EntityInstance,
    operation: &str,
    input: &Value,
) -> entity_core::PreparedOutcome<'a> {
    let prepared = runtime
        .decide_before_load(
            &instance.entity,
            instance.version,
            instance.id.clone(),
            operation,
            json!({"input": input, "bound": {}}),
        )
        .expect("operation input is admitted");
    let PreloadDecision::Load(prepared) = prepared else {
        panic!("operation loads its exact subject")
    };
    let LoadedDecision::NeedsFulfillment(prepared) = prepared
        .select_with(instance)
        .expect("loaded branch selects")
    else {
        panic!("selected accepting branch requires fulfillment")
    };
    prepared
}

fn preserves(
    prepared: &entity_core::PreparedOutcome<'_>,
) -> BTreeMap<String, OperationFieldAction> {
    prepared
        .requirements()
        .keys()
        .map(|field| (field.clone(), OperationFieldAction::Preserve))
        .collect()
}

#[test]
fn accepted_operation_action_algebra_is_enforced_after_branch_selection() {
    let ir = compile_directory(&example("billing"));
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(
        &selected(&ir, &plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
    .expect("billing lowers");
    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let logical_id = json!("b404a1e8-9360-4af5-a0ac-8a483adfa225");
    let instance = EntityInstance {
        entity: "billing.invoice.Invoice".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &logical_id).expect("identity address"),
        lifecycle_state: "Draft".to_owned(),
        revision: 1,
        fields: invoice_fields(),
    };
    let operation = "billing.invoice.IssueInvoice";
    let input = json!({"invoice_id": logical_id});

    assert!(selected_fulfillment(&runtime, &instance, operation, &input)
        .complete(BTreeMap::new())
        .is_err());

    let prepared = selected_fulfillment(&runtime, &instance, operation, &input);
    let mut extra = preserves(&prepared);
    extra.insert("not_declared".to_owned(), OperationFieldAction::Preserve);
    assert!(prepared.complete(extra).is_err());

    let prepared = selected_fulfillment(&runtime, &instance, operation, &input);
    let mut required_remove = preserves(&prepared);
    required_remove.insert("account_id".to_owned(), OperationFieldAction::Remove);
    assert!(prepared.complete(required_remove).is_err());

    let prepared = selected_fulfillment(&runtime, &instance, operation, &input);
    let mut wrong_type = preserves(&prepared);
    wrong_type.insert(
        "reminder_count".to_owned(),
        OperationFieldAction::Set {
            value: json!("not-an-integer"),
        },
    );
    assert!(prepared.complete(wrong_type).is_err());

    let prepared = selected_fulfillment(&runtime, &instance, operation, &input);
    let mut set_and_remove = preserves(&prepared);
    set_and_remove.insert(
        "issued_at".to_owned(),
        OperationFieldAction::Set {
            value: json!("2026-09-16T10:30:00Z"),
        },
    );
    set_and_remove.insert("note".to_owned(), OperationFieldAction::Remove);
    let issued = prepared
        .complete(set_and_remove)
        .expect("optional set and absent remove are admitted")
        .into_decision()
        .expect("issue accepts");
    assert_eq!(
        issued.instance.fields["issued_at"],
        json!("2026-09-16T10:30:00Z")
    );
    assert!(!issued.instance.fields.contains_key("note"));
    assert_eq!(issued.instance.lifecycle_state, "Issued");

    let mut present = instance.clone();
    present.fields.insert("note".to_owned(), json!("remove me"));
    let prepared = selected_fulfillment(&runtime, &present, operation, &input);
    let mut remove_present = preserves(&prepared);
    remove_present.insert("note".to_owned(), OperationFieldAction::Remove);
    let issued = prepared
        .complete(remove_present)
        .expect("present optional field removes")
        .into_decision()
        .expect("issue accepts");
    assert!(!issued.instance.fields.contains_key("note"));
}

fn visit_fields() -> serde_json::Map<String, Value> {
    serde_json::from_value(json!({
        "visit_id": "2fa7e9ae-9aa1-4f72-b8ce-7eb044967639",
        "visitor": "Ada",
        "building": "North",
        "host": {"kind": "employee", "value": "e-1"},
        "expected_minutes": 30,
        "expected_stay": "PT30M",
        "deposit": {"amount": 0, "currency": "EUR"},
        "escorts": [],
        "notes": {},
        "on_watchlist": false
    }))
    .expect("visit fields")
}

#[test]
fn admit_visitor_reuses_the_normalized_badge_for_action_and_event() {
    let ir = compile_directory(&example("gatepass"));
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(
        &selected(&ir, &plan, "pass-service"),
        &options(&["gatepass.visit.Visit"]),
    )
    .expect("gatepass lowers");
    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let logical_id = json!("2fa7e9ae-9aa1-4f72-b8ce-7eb044967639");
    let instance = EntityInstance {
        entity: "gatepass.visit.Visit".to_owned(),
        version: 1,
        id: identity::address(FieldKind::String, &logical_id).expect("identity address"),
        lifecycle_state: "Expected".to_owned(),
        revision: 1,
        fields: visit_fields(),
    };
    let badge = json!({"serial": "badge-7", "signature": "AA=="});
    let input = json!({"visit_id": logical_id, "badge": badge});
    let prepared = selected_fulfillment(&runtime, &instance, "gatepass.visit.AdmitVisitor", &input);
    let mut actions = preserves(&prepared);
    actions.insert(
        "badge".to_owned(),
        OperationFieldAction::Set {
            value: badge.clone(),
        },
    );
    let admitted = prepared
        .complete(actions)
        .expect("badge action completes after selection")
        .into_decision()
        .expect("admission accepts");
    assert_eq!(admitted.instance.fields["badge"], badge);
    assert_eq!(
        admitted.events[0].payload["badge"],
        admitted.instance.fields["badge"]
    );
    assert_eq!(admitted.instance.lifecycle_state, "OnSite");
}

#[test]
#[allow(clippy::too_many_lines)]
fn creation_identity_tracks_each_accepting_branch_observation_position() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let completed = r"      - name: completed
        creates: contract.local.Child
        instance: child_id
        emits:
          - contract.local.PrivateEmission
          - contract.local.First
          - contract.local.Second
        payload:
          contract.local.PrivateEmission:
            child_id: {generated: true}
            receipt: {response: receipt}
            note: input.note
            marker: exact-marker-007
            optional_receipt: {response: optional_receipt}
          contract.local.Second:
            note: input.note
            optional_a: {generated: true}
            optional_b: {generated: true}
        sets:
          owner_id: input.owner_id
          note: input.note
        summary: The child is stored and all ordered facts are emitted.
";
    let conditional = completed
        .replacen(
            "      - name: completed\n",
            "      - name: conditional\n        when: note == first\n",
            1,
        )
        .replacen(
            "          - contract.local.PrivateEmission\n          - contract.local.First\n",
            "          - contract.local.First\n          - contract.local.PrivateEmission\n",
            1,
        );
    let replacement = format!("{conditional}{completed}");
    let ir = compile_replacing(
        &base,
        "domains/local.yaml",
        &[(completed, replacement.as_str())],
    );
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(
        &selected(&ir, &plan, "local-service"),
        &options(&["contract.foreign.Owner", "contract.local.Child"]),
    )
    .expect("both creation branches lower");
    let binding = &lowered.bindings().commands()[&name("contract.local.Run")];
    let InstanceBinding::Created { observed_at, .. } = &binding.instance else {
        panic!("Run remains a creation binding");
    };
    assert_eq!(observed_at.outcome.as_str(), "conditional");
    assert_eq!(observed_at.occurrence, 1);
    assert!(binding.slots.values().all(|value| {
        !matches!(
            &value.target,
            BoundTarget::EventField { field, .. } if field == "child_id"
        )
    }));

    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let logical_id = json!("278f4f3a-c8b8-4e86-9a16-2c385910fc68");
    let storage_id = identity::address(FieldKind::String, &logical_id).expect("identity address");
    for (note, identity_event) in [("first", 1), ("later", 0)] {
        let mut bound = serde_json::Map::new();
        for (slot, value) in &binding.slots {
            let supplied = match &value.target {
                BoundTarget::ExternalEvidence { .. } => Some(json!(false)),
                BoundTarget::LogicalIdentity { .. } => Some(logical_id.clone()),
                BoundTarget::ResponseField { field, .. } if field == "receipt" => {
                    Some(json!("receipt-7"))
                }
                BoundTarget::ResponseField { field, .. } if field == "optional_receipt" => None,
                BoundTarget::EntityField { field, .. } if field == "memo" => None,
                BoundTarget::EventField { field, .. }
                    if field == "optional_a" || field == "optional_b" =>
                {
                    None
                }
                target => panic!("unexpected bound target: {target:?}"),
            };
            if let Some(value) = supplied {
                bound.insert(format!("b{:08}", slot.index()), value);
            }
        }
        let decision = runtime
            .decide_create(
                "contract.local.Child",
                1,
                storage_id.clone(),
                json!({
                    "input": {
                        "owner_id": "78e993a6-ac0d-42c0-a05c-8f2ee1898ee9",
                        "note": note
                    },
                    "bound": Value::Object(bound)
                }),
            )
            .expect("branch decision is admitted")
            .into_decision()
            .expect("branch accepts");
        assert_eq!(decision.instance.fields["child_id"], logical_id);
        assert_eq!(
            decision.events[identity_event].payload["child_id"],
            logical_id
        );
    }
}

fn lower_billing_changes(
    ir: &EssIr,
) -> Result<ess_entity_runtime::LoweredService, ess_entity_runtime::LoweringDiagnostics> {
    let plan = SynthesisPlan::of(ir);
    lower(
        &selected(ir, &plan, "invoice-service"),
        &options(&["billing.invoice.Account", "billing.invoice.Invoice"]),
    )
}

fn compile_billing_ess7(invoice: &[(&str, &str)]) -> EssIr {
    let mut changes = vec![
        ("system.yaml", "format: ess/1\n", "format: ess/7\n"),
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

fn invoice_instance(state: &str, channel: &str) -> EntityInstance {
    let logical_id = json!("b404a1e8-9360-4af5-a0ac-8a483adfa225");
    let mut fields = invoice_fields();
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

#[test]
fn enum_fields_carry_each_variant_name_exactly_as_declared() {
    let lowered =
        lower_billing_changes(&compile_directory(&example("billing"))).expect("billing lowers");
    let invoice = &lowered.definitions()[&name("billing.invoice.Invoice")];
    let channel = &invoice.schema.fields["channel"];
    assert_eq!(channel.kind, FieldKind::Enum);
    assert_eq!(channel.values, ["Email", "Post", "Portal"]);
}

#[test]
fn a_creation_that_clears_an_optional_field_leaves_it_absent_without_a_host_slot() {
    let ir = compile_replacing(
        &example("billing"),
        "domains/invoice.yaml",
        &[(
            "        sets:\n          account_id: input.account_id\n          total: input.amount\n",
            "        sets:\n          account_id: input.account_id\n          total: input.amount\n          note: {cleared: true}\n",
        )],
    );
    let lowered = lower_billing_changes(&ir).expect("a cleared optional creation field lowers");
    let invoice = &lowered.definitions()[&name("billing.invoice.Invoice")];
    let accepted = invoice
        .create
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "accepted")
        .expect("accepted creation branch");
    assert!(!accepted.set.contains_key("note"));
    assert!(!accepted.set_if_present.contains_key("note"));
    assert!(accepted.set_if_present.contains_key("issued_at"));
    let binding = &lowered.bindings().commands()[&name("billing.invoice.CreateInvoice")];
    assert!(binding.slots.values().all(|value| !matches!(
        &value.target,
        BoundTarget::EntityField { field, .. } if field == "note"
    )));
    let _ = registry(&lowered);
}

#[test]
fn an_operation_that_clears_a_field_is_refused_rather_than_left_to_the_host() {
    let ir = compile_replacing(
        &example("billing"),
        "domains/invoice.yaml",
        &[(
            "            invoice_id: input.invoice_id\n        summary: The invoice leaves Draft and is now Issued.\n",
            "            invoice_id: input.invoice_id\n        sets:\n          note: {cleared: true}\n        summary: The invoice leaves Draft and is now Issued.\n",
        )],
    );
    let diagnostics = lower_billing_changes(&ir)
        .expect_err("ER has no source-determined removal for an operation field");
    let diagnostics = diagnostics.into_vec();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LoweringCode::ClearedValueUnsupported
            && diagnostic.path == "billing.invoice.IssueInvoice.issued.sets.note"
    }));
}

const KEPT_INPUT: &str = "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n";
const KEPT_OUTCOME: &str = "      - name: kept\n        when_subject: {field: channel, equals: Post}\n        preserves: billing.invoice.Invoice\n        instance: invoice_id\n        summary: A posted invoice is kept exactly as it is.\n";

#[test]
fn a_silent_preserving_outcome_is_refused_because_er_cannot_observe_it() {
    let replacement = format!(
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n{KEPT_OUTCOME}      - name: cancelled\n"
    );
    let ir = compile_billing_ess7(&[(KEPT_INPUT, replacement.as_str())]);
    let diagnostics = lower_billing_changes(&ir).expect_err("a silent preserve is unobservable");
    let diagnostics = diagnostics.into_vec();
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.as_str()))
            .collect::<Vec<_>>(),
        [(
            LoweringCode::SilentPreserveUnsupported,
            "billing.invoice.CancelInvoice.kept"
        )]
    );
}

#[test]
fn subject_field_selection_and_a_responding_preserve_lower_and_decide_faithfully() {
    let replacement = format!(
        "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n    response:\n      - name: seen\n        type: Boolean\n\n    outcomes:\n{KEPT_OUTCOME}      - name: cancelled\n"
    );
    let ir = compile_billing_ess7(&[(KEPT_INPUT, replacement.as_str())]);
    let lowered = lower_billing_changes(&ir).expect("a responding preserve lowers");
    let invoice = &lowered.definitions()[&name("billing.invoice.Invoice")];
    let cancel = &invoice.operations["billing.invoice.CancelInvoice"];
    let kept = &cancel.outcomes[0];
    assert_eq!(kept.name, "kept");
    assert_eq!(
        serde_json::to_value(&kept.when).expect("condition serializes"),
        json!({"compare": {"left": "$fields.channel", "op": "eq", "right": "Post"}})
    );
    assert_eq!(kept.in_state, None);
    assert!(kept.effect.is_none());
    assert!(kept.set.is_empty());
    assert!(kept.set_if_present.is_empty());
    assert!(kept.fulfills.is_empty());
    assert!(kept.emits.is_empty());
    assert!(kept.responds.contains_key("seen"));
    assert!(!lowered.bindings().requirements().iter().any(|requirement| matches!(
        requirement,
        BindingRequirement::OperationFieldPolicySupplied { target, .. } if target.outcome.as_str() == "kept"
    )));

    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let binding = &lowered.bindings().commands()[&name("billing.invoice.CancelInvoice")];
    let bound = bound_for(binding, |target| match target {
        BoundTarget::ResponseField { field, .. } if field == "seen" => Some(json!(true)),
        target => panic!("unexpected bound target: {target:?}"),
    });
    let arguments = json!({
        "input": {"invoice_id": "b404a1e8-9360-4af5-a0ac-8a483adfa225"},
        "bound": bound
    });

    let posted = invoice_instance("Draft", "Post");
    let PreloadDecision::Load(prepared) = runtime
        .decide_before_load(
            &posted.entity,
            posted.version,
            posted.id.clone(),
            "billing.invoice.CancelInvoice",
            arguments.clone(),
        )
        .expect("input admitted")
    else {
        panic!("a subject-field selector needs the loaded subject")
    };
    let LoadedDecision::Complete(evaluation) = prepared
        .select_with(&posted)
        .expect("posted invoice selects")
    else {
        panic!("a preserving branch needs no host field action")
    };
    let decision = evaluation.into_decision().expect("preserve accepts");
    assert_eq!(decision.instance.lifecycle_state, "Draft");
    assert_eq!(decision.instance.fields, posted.fields);
    assert!(decision.events.is_empty());

    let emailed = invoice_instance("Draft", "Email");
    let PreloadDecision::Load(prepared) = runtime
        .decide_before_load(
            &emailed.entity,
            emailed.version,
            emailed.id.clone(),
            "billing.invoice.CancelInvoice",
            arguments,
        )
        .expect("input admitted")
    else {
        panic!("a subject-field selector needs the loaded subject")
    };
    let LoadedDecision::NeedsFulfillment(prepared) = prepared
        .select_with(&emailed)
        .expect("emailed invoice selects")
    else {
        panic!("cancellation needs explicit field actions")
    };
    assert_eq!(prepared.outcome(), "cancelled");
}

#[test]
fn state_change_selection_partitions_the_held_state_exactly() {
    let ir = compile_billing_ess7(&[
        (
            "        - name: issue\n          from: [Draft]\n",
            "        - name: issue\n          from: [Draft, Issued]\n",
        ),
        (
            "      - name: issued\n        moves: billing.invoice.Invoice.issue\n",
            "      - name: issued\n        when_state_changes: true\n        moves: billing.invoice.Invoice.issue\n",
        ),
        (
            "        summary: The invoice leaves Draft and is now Issued.\n",
            "        summary: The invoice leaves Draft and is now Issued.\n      - name: reissued\n        when_state_changes: false\n        moves: billing.invoice.Invoice.issue\n        instance: invoice_id\n        emits:\n          - billing.invoice.InvoiceIssued\n        payload:\n          billing.invoice.InvoiceIssued:\n            invoice_id: input.invoice_id\n        summary: An issued invoice is issued again.\n",
        ),
        (
            "      - name: wrong-state\n        wrong_state: true\n        error: billing.invoice.InvoiceStateConflict\n        summary: The invoice is not in Draft, so it was not issued.\n",
            "      - name: settled-or-cancelled\n        error: billing.invoice.InvoiceStateConflict\n        summary: The invoice is Paid or Cancelled, so it was not issued.\n",
        ),
    ]);
    let lowered = lower_billing_changes(&ir).expect("state-change branches lower");
    let invoice = &lowered.definitions()[&name("billing.invoice.Invoice")];
    let issue = &invoice.operations["billing.invoice.IssueInvoice"];
    let when = |outcome: &str| {
        let outcome = issue
            .outcomes
            .iter()
            .find(|candidate| candidate.name == outcome)
            .expect("outcome is lowered");
        assert_eq!(outcome.in_state, None);
        serde_json::to_value(&outcome.when).expect("condition serializes")
    };
    assert_eq!(when("issued"), json!({"in": ["$from_state", ["Draft"]]}));
    assert_eq!(when("reissued"), json!({"in": ["$from_state", ["Issued"]]}));

    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let input = json!({"invoice_id": "b404a1e8-9360-4af5-a0ac-8a483adfa225"});
    for (state, expected) in [("Draft", "issued"), ("Issued", "reissued")] {
        let instance = invoice_instance(state, "Email");
        let prepared =
            selected_fulfillment(&runtime, &instance, "billing.invoice.IssueInvoice", &input);
        assert_eq!(prepared.outcome(), expected);
    }
}

#[test]
fn external_when_requires_both_input_eligibility_and_supplied_evidence() {
    let ir = compile_billing_ess7(&[(
        "      - name: settled\n        when: amount.amount > 0\n",
        "      - name: declined\n        when: amount.amount > 0\n        external: the bank declines the payment\n        error: billing.invoice.InvalidAmount\n        summary: The bank declined, so the invoice did not move.\n      - name: settled\n        when: amount.amount > 0\n",
    )]);
    let lowered = lower_billing_changes(&ir).expect("an input-guarded external branch lowers");
    let invoice = &lowered.definitions()[&name("billing.invoice.Invoice")];
    let pay = &invoice.operations["billing.invoice.PayInvoice"];
    let declined = &pay.outcomes[0];
    let settled = &pay.outcomes[1];
    assert_eq!(declined.name, "declined");
    assert_eq!(settled.name, "settled");
    let binding = &lowered.bindings().commands()[&name("billing.invoice.PayInvoice")];
    let evidence = binding
        .slots
        .iter()
        .find(|(_, value)| {
            matches!(&value.target, BoundTarget::ExternalEvidence { outcome } if outcome.as_str() == "declined")
        })
        .map(|(slot, _)| *slot)
        .expect("the external verdict is one supplied slot");
    assert!(lowered
        .bindings()
        .requirements()
        .iter()
        .any(|requirement| matches!(
            requirement,
            BindingRequirement::ExternalEvidenceSupplied { outcome, cause, .. }
                if outcome.as_str() == "declined" && cause == "the bank declines the payment"
        )));
    assert_eq!(
        serde_json::to_value(&declined.when).expect("condition serializes"),
        json!({"all": [
            serde_json::to_value(&settled.when).expect("condition serializes"),
            {"truthy": format!("$args.bound.b{:08}", evidence.index())}
        ]})
    );

    let registry = registry(&lowered);
    let runtime = Runtime::new(&registry);
    let instance = invoice_instance("Issued", "Email");
    let decide = |amount: i64, verdict: bool| {
        runtime
            .decide_before_load(
                &instance.entity,
                instance.version,
                instance.id.clone(),
                "billing.invoice.PayInvoice",
                json!({
                    "input": {
                        "invoice_id": "b404a1e8-9360-4af5-a0ac-8a483adfa225",
                        "amount": {"amount": amount, "currency": "EUR"}
                    },
                    "bound": {format!("b{:08}", evidence.index()): verdict}
                }),
            )
            .expect("input admitted")
    };
    let PreloadDecision::Refused(refusal) = decide(25, true) else {
        panic!("eligible input with a declining verdict refuses")
    };
    assert_eq!(refusal.outcome, "declined");
    let PreloadDecision::Refused(refusal) = decide(0, true) else {
        panic!("ineligible input falls through to the amount refusal")
    };
    assert_eq!(refusal.outcome, "rejected");
    let PreloadDecision::Load(prepared) = decide(25, false) else {
        panic!("eligible input without the verdict settles")
    };
    let LoadedDecision::NeedsFulfillment(prepared) =
        prepared.select_with(&instance).expect("settlement selects")
    else {
        panic!("settlement needs explicit field actions")
    };
    assert_eq!(prepared.outcome(), "settled");
}
