//! Acceptance coverage for the reusable selected-service contract.

use std::path::{Path, PathBuf};

use ess_compiler::ir::{
    EssIr, ResolvedCondition, ResolvedEffect, ResolvedMappingValue, ResolvedPayloadValue,
};
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::entity::{Cardinality, RelationKind};
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_service_contract::{extract, ServiceDiagnostic};
use ess_synth::{
    Capability, CapabilityKind, ImplementationObligation, ObligationReason, RefusalReason,
    RefusalStage, SynthesisDisposition, SynthesisPlan, SynthesisRefusal,
};

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
}

fn compile_directory(base: &Path) -> EssIr {
    let mut pending = vec![base.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example directory is readable") {
            let path = entry.expect("an example entry").path();
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

    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    let mut sources = SourceMap::new();
    for path in paths {
        let label = path
            .strip_prefix(base)
            .expect("the example file is below its base")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("the example file is readable");
        let raw = RawSpecFile::parse(&text).expect("the example file is well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label.clone()), raw));
        labels.push(label);
    }

    let specification =
        Specification::assemble(parsed).expect("the example specification validates");
    compile_locating(&specification, &sources, &labels).expect("the example specification resolves")
}

fn compile_example(name: &str) -> EssIr {
    compile_directory(&example(name))
}

fn focused_fixture() -> EssIr {
    compile_directory(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"))
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("valid qualified name")
}

fn names<'a, T>(entries: impl Iterator<Item = (&'a QualifiedName, T)>) -> Vec<String> {
    entries.map(|(name, _)| name.to_string()).collect()
}

fn capability_rows(
    selected: &ess_service_contract::ServiceIr<'_>,
) -> Vec<(CapabilityKind, String)> {
    selected
        .capabilities()
        .map(|planned| (planned.capability.kind, planned.capability.source.clone()))
        .collect()
}

#[test]
fn billing_invoice_service_exposes_its_declared_surface() {
    let ir = compile_example("billing");
    let plan = SynthesisPlan::of(&ir);
    let selected = extract(
        &ir,
        &plan,
        &ComponentName::new("invoice-service").expect("valid component name"),
    )
    .expect("the complete plan and known component are admitted");

    assert_eq!(selected.component().name.as_str(), "invoice-service");
    assert_eq!(names(selected.owned_domains().iter()), ["billing.invoice"]);
    assert_eq!(
        names(selected.operations().iter()),
        [
            "billing.invoice.CancelInvoice",
            "billing.invoice.CreateInvoice",
            "billing.invoice.IssueInvoice",
            "billing.invoice.PayInvoice",
        ]
    );
    assert_eq!(
        names(selected.published_events().iter()),
        [
            "billing.invoice.InvoiceCancelled",
            "billing.invoice.InvoiceCreated",
            "billing.invoice.InvoiceIssued",
            "billing.invoice.InvoicePaid",
        ]
    );
    assert_eq!(
        names(selected.views().iter()),
        [
            "billing.invoice.InvoiceById",
            "billing.invoice.OutstandingInvoices",
        ]
    );
}

#[test]
fn billing_and_gatepass_keep_complete_compiler_values_and_source_bytes() {
    let billing = compile_example("billing");
    let billing_bytes = billing.to_canonical_json();
    let billing_digest = billing.source_digest();
    let billing_plan = SynthesisPlan::of(&billing);
    let email = extract(
        &billing,
        &billing_plan,
        &ComponentName::new("email-service").expect("valid component name"),
    )
    .expect("billing email service is selected");

    assert!(std::ptr::eq(email.source(), std::ptr::from_ref(&billing)));
    assert!(std::ptr::eq(
        email.source_plan(),
        std::ptr::from_ref(&billing_plan)
    ));
    let send = email
        .operations()
        .get(&name("billing.email.SendEmail"))
        .expect("the accepted command remains selected");
    assert!(std::ptr::eq(
        *send,
        billing
            .commands()
            .get(&name("billing.email.SendEmail"))
            .expect("the compiler owns SendEmail")
    ));
    assert_eq!(
        send.outcomes
            .iter()
            .map(|outcome| outcome.name.as_str())
            .collect::<Vec<_>>(),
        ["sent", "failed"]
    );
    assert_eq!(
        send.outcomes[0]
            .emits
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["billing.email.EmailSent"]
    );
    assert!(matches!(
        &send.outcomes[1].condition,
        ResolvedCondition::External { cause }
            if cause == "the provider rejects the recipient address"
    ));
    assert_eq!(
        send.outcomes[1].error.as_ref().map(ToString::to_string),
        Some("billing.email.Undeliverable".to_owned())
    );
    assert_eq!(billing.to_canonical_json(), billing_bytes);
    assert_eq!(billing.source_digest(), billing_digest);
    assert_eq!(
        capability_rows(&email),
        capability_rows(
            &extract(
                &billing,
                &billing_plan,
                &ComponentName::new("email-service").expect("valid component name"),
            )
            .expect("repeated extraction succeeds")
        )
    );

    let gatepass = compile_example("gatepass");
    let gatepass_plan = SynthesisPlan::of(&gatepass);
    let pass = extract(
        &gatepass,
        &gatepass_plan,
        &ComponentName::new("pass-service").expect("valid component name"),
    )
    .expect("gatepass service is selected");
    assert_eq!(
        names(pass.operations().iter()),
        [
            "gatepass.visit.AdmitVisitor",
            "gatepass.visit.RegisterVisit",
            "gatepass.visit.SignOutVisitor",
        ]
    );
    assert_eq!(
        names(pass.views().iter()),
        ["gatepass.visit.ExpectedVisits", "gatepass.visit.VisitById",]
    );
    assert_eq!(
        names(pass.published_events().iter()),
        [
            "gatepass.visit.VisitRegistered",
            "gatepass.visit.VisitorAdmitted",
            "gatepass.visit.VisitorDeparted",
        ]
    );
}

#[test]
fn input_diagnostics_accumulate_in_stable_order_and_admit_only_the_exact_plan() {
    let ir = compile_example("billing");
    let plan = SynthesisPlan::of(&ir);
    let unknown = ComponentName::new("missing-service").expect("valid component name");
    assert_eq!(
        extract(&ir, &plan, &unknown).err(),
        Some(vec![ServiceDiagnostic::UnknownComponent {
            component: unknown.clone(),
        }])
    );

    let foreign_ir = compile_example("gatepass");
    let foreign_plan = SynthesisPlan::of(&foreign_ir);
    let invoice = ComponentName::new("invoice-service").expect("valid component name");
    assert_eq!(
        extract(&ir, &foreign_plan, &invoice).err(),
        Some(vec![ServiceDiagnostic::PlanMismatch])
    );
    assert_eq!(
        extract(&ir, &foreign_plan, &unknown).err(),
        Some(vec![
            ServiceDiagnostic::UnknownComponent { component: unknown },
            ServiceDiagnostic::PlanMismatch,
        ])
    );

    let refusal = SynthesisDisposition::Refused(SynthesisRefusal {
        reason: RefusalReason::TopologyDeferred,
        stage: RefusalStage::Planning,
        detail: "test-only altered disposition".to_owned(),
    });
    let mut changed = plan.clone();
    changed.capabilities[0].disposition = refusal;
    let mut removed = plan.clone();
    removed.capabilities.remove(0);
    let mut duplicated = plan.clone();
    duplicated.capabilities.push(plan.capabilities[0].clone());
    let mut reordered = plan.clone();
    reordered.capabilities.swap(0, 1);
    for altered in [&changed, &removed, &duplicated, &reordered] {
        assert_eq!(
            extract(&ir, altered, &invoice).err(),
            Some(vec![ServiceDiagnostic::PlanMismatch])
        );
    }
}

fn assert_focused_surfaces(selected: &ess_service_contract::ServiceIr<'_>) {
    assert_eq!(names(selected.owned_domains().iter()), ["contract.local"]);
    assert_eq!(names(selected.operations().iter()), ["contract.local.Run"]);
    assert_eq!(
        names(selected.published_events().iter()),
        ["contract.local.PublicOnly"]
    );
    assert_eq!(names(selected.views().iter()), ["contract.local.ChildById"]);
    assert_eq!(
        names(selected.owned_entities().iter()),
        ["contract.local.Child"]
    );
}

fn assert_focused_operation(selected: &ess_service_contract::ServiceIr<'_>) {
    let run = selected.operations()[&name("contract.local.Run")];
    assert_eq!(run.response.len(), 1);
    assert_eq!(run.response[0].name, "receipt");
    assert_eq!(
        run.response[0].type_ref.to_string(),
        "contract.local.Receipt"
    );
    assert_eq!(
        run.outcomes
            .iter()
            .map(|outcome| outcome.name.as_str())
            .collect::<Vec<_>>(),
        ["completed", "rejected"]
    );
    assert_eq!(
        run.outcomes[0]
            .emits
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        [
            "contract.local.PrivateEmission",
            "contract.local.First",
            "contract.local.Second",
        ]
    );
    assert!(run.outcomes[1].emits.is_empty());
    assert!(matches!(
        &run.outcomes[1].condition,
        ResolvedCondition::External { cause }
            if cause == "an upstream authority rejects the request"
    ));
    assert!(matches!(
        &run.outcomes[0]
            .subject
            .as_ref()
            .expect("the completed branch creates the child")
            .effect,
        ResolvedEffect::Creates
    ));
    assert_eq!(
        run.outcomes[1].error.as_ref().map(ToString::to_string),
        Some("contract.local.Rejected".to_owned())
    );
    assert_focused_payload(run);
}

fn assert_focused_payload(run: &ess_compiler::ir::ResolvedCommand) {
    let private = &run.outcomes[0].payload[0];
    assert_eq!(private.event.to_string(), "contract.local.PrivateEmission");
    assert!(matches!(
        &private.fields[0].value,
        ResolvedPayloadValue::Generated
    ));
    assert!(matches!(
        &private.fields[1].value,
        ResolvedPayloadValue::ResponseField { field, type_ref }
            if field == "receipt" && type_ref.to_string() == "contract.local.Receipt"
    ));
    assert!(matches!(
        &private.fields[2].value,
        ResolvedPayloadValue::InputField { field, type_ref }
            if field == "note" && type_ref.to_string() == "contract.local.Shared"
    ));
    assert!(matches!(
        &private.fields[3].value,
        ResolvedPayloadValue::Literal { value } if value == "exact-marker-007"
    ));
    assert_eq!(
        run.outcomes[0]
            .sets
            .iter()
            .map(|field| field.target.as_str())
            .collect::<Vec<_>>(),
        ["owner_id", "note"]
    );
    assert!(matches!(
        &run.outcomes[0].sets[0].value,
        ResolvedPayloadValue::InputField { field, type_ref }
            if field == "owner_id" && type_ref.to_string() == "contract.foreign.OwnerId"
    ));
}

fn assert_focused_binding(ir: &EssIr) {
    let binding = ir
        .bindings()
        .values()
        .find(|binding| binding.name.to_string() == "react-public")
        .expect("the published event has its reacting binding");
    assert_eq!(binding.command.to_string(), "contract.foreign.HandlePublic");
    assert_eq!(binding.mapping.len(), 1);
    assert_eq!(binding.mapping[0].target, "payload");
    assert_eq!(
        binding.mapping[0].conversion.as_deref(),
        Some("The foreign handler wraps the selected service's text in its declared envelope.")
    );
    assert!(matches!(
        &binding.mapping[0].value,
        ResolvedMappingValue::EventField { field, type_ref }
            if field == "shared" && type_ref.to_string() == "contract.local.Shared"
    ));
}

fn assert_focused_definitions(ir: &EssIr) {
    let rejected = ir
        .errors()
        .get(&name("contract.local.Rejected"))
        .expect("the selected error retains its definition");
    assert_eq!(
        rejected.summary.as_deref(),
        Some("The upstream system rejected the request.")
    );
    assert_eq!(rejected.fields[0].name, "detail");
    assert_eq!(
        rejected.fields[0].type_ref.to_string(),
        "contract.local.Shared"
    );

    let carried = ir.relations_carried_by(&name("contract.local.Child"));
    let owner = carried
        .get("owner_id")
        .expect("the child carries its owner relation");
    assert_eq!(owner.source.to_string(), "contract.foreign.Owner");
    assert_eq!(owner.relation.name, "children");
    assert_eq!(owner.relation.kind, RelationKind::Owns);
    assert_eq!(owner.relation.cardinality, Cardinality::Many);
    assert_eq!(owner.relation.target.to_string(), "contract.local.Child");
}

fn focused_capabilities() -> Vec<(CapabilityKind, String)> {
    [
        (
            CapabilityKind::DomainType,
            "contract.foreign.ForeignPayload",
        ),
        (CapabilityKind::DomainType, "contract.foreign.Owner.State"),
        (CapabilityKind::DomainType, "contract.foreign.OwnerId"),
        (CapabilityKind::DomainType, "contract.local.Child.State"),
        (CapabilityKind::DomainType, "contract.local.ChildId"),
        (CapabilityKind::DomainType, "contract.local.Envelope"),
        (CapabilityKind::DomainType, "contract.local.Node"),
        (CapabilityKind::DomainType, "contract.local.Receipt"),
        (CapabilityKind::DomainType, "contract.local.Shared"),
        (CapabilityKind::EntityLifecycle, "contract.foreign.Owner"),
        (CapabilityKind::EntityLifecycle, "contract.local.Child"),
        (CapabilityKind::CommandContract, "contract.local.Run"),
        (CapabilityKind::CommandBehavior, "contract.local.Run"),
        (CapabilityKind::EventType, "contract.foreign.DeliveryFailed"),
        (CapabilityKind::EventType, "contract.foreign.Trigger"),
        (CapabilityKind::EventType, "contract.local.First"),
        (CapabilityKind::EventType, "contract.local.PrivateEmission"),
        (CapabilityKind::EventType, "contract.local.PublicOnly"),
        (CapabilityKind::EventType, "contract.local.Second"),
        (CapabilityKind::ErrorType, "contract.local.Rejected"),
        (CapabilityKind::ViewType, "contract.local.ChildById"),
        (CapabilityKind::ViewQuery, "contract.local.ChildById"),
        (
            CapabilityKind::Conversion,
            "contract.local.Shared -> contract.foreign.ForeignPayload",
        ),
        (CapabilityKind::ActorGrants, "contract.local.Operator"),
        (CapabilityKind::BindingTransformation, "invoke-selected"),
        (CapabilityKind::BindingDelivery, "invoke-selected"),
        (CapabilityKind::BindingTransformation, "react-public"),
        (CapabilityKind::BindingDelivery, "react-public"),
        (CapabilityKind::BindingEscalation, "react-public"),
        (CapabilityKind::ComponentPort, "local-service"),
        (CapabilityKind::ComponentTransport, "local-service"),
        (CapabilityKind::Workload, "local-service"),
    ]
    .into_iter()
    .map(|(kind, source)| (kind, source.to_owned()))
    .collect()
}

#[test]
fn contextual_closure_and_publication_select_exact_capabilities_in_plan_order() {
    let ir = focused_fixture();
    let before = ir.to_canonical_json();
    let digest = ir.source_digest();
    let plan = SynthesisPlan::of(&ir);
    let selected = extract(
        &ir,
        &plan,
        &ComponentName::new("local-service").expect("valid component name"),
    )
    .expect("the focused service is selected");

    assert_focused_surfaces(&selected);
    assert_focused_operation(&selected);
    assert_focused_binding(&ir);
    assert_focused_definitions(&ir);

    assert_eq!(capability_rows(&selected), focused_capabilities());
    assert!(selected.capabilities().all(|planned| {
        planned.capability.source != "react-private"
            && planned.capability.source != "contract.local.Admin"
    }));

    assert_eq!(ir.to_canonical_json(), before);
    assert_eq!(ir.source_digest(), digest);
}

fn expected_obligations() -> Vec<(Capability, ImplementationObligation)> {
    vec![
        (
            Capability {
                kind: CapabilityKind::CommandBehavior,
                source: "contract.local.Run".to_owned(),
            },
            ImplementationObligation {
                reason: ObligationReason::External {
                    cause: "an upstream authority rejects the request".to_owned(),
                },
                contract: "given `contract.local.Run` input, decide and enact exactly one outcome — `completed` otherwise, creates `contract.local.Child`, emits `contract.local.PrivateEmission`, emits `contract.local.First`, emits `contract.local.Second`; `rejected` externally decided (an upstream authority rejects the request), error `contract.local.Rejected`".to_owned(),
            },
        ),
        (
            Capability {
                kind: CapabilityKind::ViewQuery,
                source: "contract.local.ChildById".to_owned(),
            },
            ImplementationObligation {
                reason: ObligationReason::ProjectionMaintenance,
                contract: "a query answering `contract.local.ChildById` with rows projected from `contract.local.Child` at `read_your_writes` consistency".to_owned(),
            },
        ),
        (
            Capability {
                kind: CapabilityKind::Conversion,
                source: "contract.local.Shared -> contract.foreign.ForeignPayload".to_owned(),
            },
            ImplementationObligation {
                reason: ObligationReason::UnspecifiedAlgorithm,
                contract: "a function from `contract.local.Shared` to `contract.foreign.ForeignPayload` — the crossing is permitted (The foreign handler wraps the selected service's text in its declared envelope.), the computation is not declared".to_owned(),
            },
        ),
        (
            Capability {
                kind: CapabilityKind::BindingTransformation,
                source: "react-public".to_owned(),
            },
            ImplementationObligation {
                reason: ObligationReason::UnspecifiedAlgorithm,
                contract: "a transformation from `contract.local.PublicOnly` to `contract.foreign.HandlePublic` input — `payload` is filled from event field `shared` through the declared crossing to `contract.foreign.ForeignPayload`, whose computation is owed".to_owned(),
            },
        ),
        (
            Capability {
                kind: CapabilityKind::BindingEscalation,
                source: "react-public".to_owned(),
            },
            ImplementationObligation {
                reason: ObligationReason::UnspecifiedAlgorithm,
                contract: "the declared `contract.foreign.DeliveryFailed`, recording that delivering `contract.foreign.HandlePublic` for `react-public` was given up on — the event is declared; how its fields are filled from the failed invocation is not".to_owned(),
            },
        ),
    ]
}

fn expected_refusals() -> Vec<(Capability, SynthesisRefusal)> {
    vec![
        (
            Capability {
                kind: CapabilityKind::ActorGrants,
                source: "contract.local.Operator".to_owned(),
            },
            SynthesisRefusal {
                reason: RefusalReason::NeedsCallerIdentity,
                stage: RefusalStage::Planning,
                detail: "may invoke `contract.local.Admin`, `contract.local.Run`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling".to_owned(),
            },
        ),
        (
            Capability {
                kind: CapabilityKind::Workload,
                source: "local-service".to_owned(),
            },
            SynthesisRefusal {
                reason: RefusalReason::TopologyDeferred,
                stage: RefusalStage::Planning,
                detail: "requires at least 2 replica(s); topology synthesis is deferred with its design".to_owned(),
            },
        ),
    ]
}

fn is_explicitly_non_generated(planned: &ess_synth::PlannedCapability) -> bool {
    matches!(
        (planned.capability.kind, planned.capability.source.as_str()),
        (CapabilityKind::CommandBehavior, "contract.local.Run")
            | (CapabilityKind::ViewQuery, "contract.local.ChildById")
            | (
                CapabilityKind::Conversion,
                "contract.local.Shared -> contract.foreign.ForeignPayload"
            )
            | (CapabilityKind::ActorGrants, "contract.local.Operator")
            | (
                CapabilityKind::BindingTransformation | CapabilityKind::BindingEscalation,
                "react-public"
            )
            | (CapabilityKind::Workload, "local-service")
    )
}

#[test]
fn selected_obligations_and_refusals_retain_their_complete_plan_values() {
    let ir = focused_fixture();
    let plan = SynthesisPlan::of(&ir);
    let selected = extract(
        &ir,
        &plan,
        &ComponentName::new("local-service").expect("valid component name"),
    )
    .expect("the focused service is selected");

    let obligations = selected
        .obligations()
        .map(|(capability, obligation)| (capability.clone(), obligation.clone()))
        .collect::<Vec<_>>();
    assert_eq!(obligations, expected_obligations());

    let refusals = selected
        .refusals()
        .map(|(capability, refusal)| (capability.clone(), refusal.clone()))
        .collect::<Vec<_>>();
    assert_eq!(refusals, expected_refusals());
    for planned in selected.capabilities() {
        if !is_explicitly_non_generated(planned) {
            assert_eq!(
                planned.disposition,
                SynthesisDisposition::Generated,
                "the fixture explicitly expects generated disposition for {:?} `{}`",
                planned.capability.kind,
                planned.capability.source
            );
        }
    }
}
