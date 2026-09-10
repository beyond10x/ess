//! Independent selected-surface expectations and persisted disposition refusal controls.

use ess_compiler::{source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_service_contract::{command_fields, compile, view_fields};

fn model() -> EssIr {
    let documents = [
        ("system.yaml", include_str!("fixtures/system.yaml")),
        ("work.yaml", include_str!("fixtures/work.yaml")),
        ("other.yaml", include_str!("fixtures/other.yaml")),
    ];
    let mut sources = SourceMap::new();
    let parsed = documents
        .iter()
        .map(|(label, text)| {
            sources.insert(*label, *text);
            (
                Source::new(*label),
                RawSpecFile::parse(text).expect("fixture parses"),
            )
        })
        .collect::<Vec<_>>();
    let specification = Specification::assemble(parsed).expect("fixture validates");
    ess_compiler::compile(&specification, &sources).expect("fixture resolves")
}

#[test]
fn selection_retains_complete_outcomes_and_only_declared_state_ownership() {
    let source = model();
    let service = compile(&source, &"work-service".parse().unwrap()).unwrap();
    assert_eq!(
        service
            .commands()
            .map(|command| command.name.to_string())
            .collect::<Vec<_>>(),
        ["sample.work.Ping", "sample.work.Put"]
    );
    let put = service
        .commands()
        .find(|command| command.name.to_string() == "sample.work.Put")
        .unwrap();
    assert_eq!(put.outcomes.len(), 2);
    assert_eq!(
        put.outcomes[0]
            .emits
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["sample.work.Stored", "sample.work.Audited"]
    );
    assert!(put.outcomes[0].subject.is_some());
    assert_eq!(put.outcomes[0].payload.len(), 1);
    assert!(
        put.outcomes[1].emits.is_empty(),
        "zero-event refusal stays present"
    );
    assert_eq!(command_fields(put)["id"], "Uuid");
    assert_eq!(
        service
            .entities()
            .map(|entity| entity.name.to_string())
            .collect::<Vec<_>>(),
        ["sample.work.Row"]
    );
    assert_eq!(
        service
            .events()
            .map(|event| event.name.to_string())
            .collect::<Vec<_>>(),
        ["sample.work.Audited", "sample.work.Stored"]
    );
    assert_eq!(
        service
            .errors()
            .map(|error| error.name.to_string())
            .collect::<Vec<_>>(),
        ["sample.work.Disabled", "sample.work.Unavailable"]
    );
    let external = service.external_outcomes().collect::<Vec<_>>();
    assert_eq!(external.len(), 1);
    assert_eq!(external[0].0.name.to_string(), "sample.work.Ping");
    assert_eq!(external[0].1.name.to_string(), "unavailable");
    assert!(
        matches!(&external[0].1.condition, ess_compiler::ir::ResolvedCondition::External { cause } if cause == "the peer is unavailable")
    );
    let view = service.views().next().unwrap();
    assert_eq!(service.views().count(), 1);
    assert_eq!(view.name.to_string(), "sample.work.ById");
    assert_eq!(view.params.len(), 1);
    assert!(view.filter.is_some());
    assert_eq!(view.order_by.len(), 1);
    assert_eq!(view_fields(view)["id"], "Uuid");
    assert_eq!(
        view.consistency,
        ess_domain::view::Consistency::ReadYourWrites
    );
    assert!(
        std::ptr::eq(service.source(), std::ptr::from_ref(&source)),
        "lookup authority is the original model"
    );
    assert!(compile(&source, &"absent".parse().unwrap()).is_err());
}

#[cfg(feature = "synthesis")]
#[test]
fn required_capabilities_preserve_wire_values_and_refuse_missing_duplicate_or_refused_entries() {
    use ess_service_contract::synthesis::{
        required_disposition, CapabilityError, RequiredDisposition,
    };
    use ess_synth::{
        CapabilityKind, RefusalReason, RefusalStage, SynthesisDisposition, SynthesisPlan,
        SynthesisRefusal,
    };
    let source = model();
    let mut plan = SynthesisPlan::of(&source);
    let kind = CapabilityKind::CommandContract;
    let name = "sample.work.Ping";
    assert_eq!(
        serde_json::to_string(&required_disposition(&plan, kind, name).unwrap()).unwrap(),
        r#"{"disposition":"generated"}"#
    );
    let behavior = required_disposition(&plan, CapabilityKind::CommandBehavior, name).unwrap();
    assert!(
        matches!(&behavior, RequiredDisposition::Obligation { reason, contract } if reason.contains("the peer is unavailable") && !contract.is_empty())
    );
    assert_eq!(
        serde_json::from_str::<RequiredDisposition>(&serde_json::to_string(&behavior).unwrap())
            .unwrap(),
        behavior
    );
    assert!(serde_json::from_str::<RequiredDisposition>(
        r#"{"disposition":"generated","extra":true}"#
    )
    .is_err());
    let index = plan
        .capabilities
        .iter()
        .position(|entry| entry.capability.kind == kind && entry.capability.source == name)
        .unwrap();
    let original = plan.capabilities.remove(index);
    assert!(matches!(
        required_disposition(&plan, kind, name),
        Err(CapabilityError::Cardinality { found: 0, .. })
    ));
    plan.capabilities.extend([original.clone(), original]);
    assert!(matches!(
        required_disposition(&plan, kind, name),
        Err(CapabilityError::Cardinality { found: 2, .. })
    ));
    plan.capabilities.pop();
    plan.capabilities.last_mut().unwrap().disposition =
        SynthesisDisposition::Refused(SynthesisRefusal {
            reason: RefusalReason::NeedsCallerIdentity,
            stage: RefusalStage::Target,
            detail: "identity is not supplied".into(),
        });
    assert!(
        matches!(required_disposition(&plan, kind, name), Err(CapabilityError::Refused { detail, .. }) if detail == "identity is not supplied")
    );
}
