//! Acceptance for unequal authored creation identity sources after ER selects the branch.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use entity_core::{replay, Runtime};
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{
    lower, BoundTarget, CommandBinding, IdentityValue, InstanceBinding, LoweredService,
    LoweringOptions,
};
use ess_service_contract::{extract, ServiceIr};
use ess_synth::SynthesisPlan;
use serde_json::{json, Value};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract")
}

fn compile_with_two_creation_outcomes() -> EssIr {
    let base = fixture();
    let mut pending = vec![base.clone()];
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
            .strip_prefix(&base)
            .expect("fixture child")
            .display()
            .to_string();
        let mut text = std::fs::read_to_string(&path).expect("fixture source is readable");
        if label == "domains/local.yaml" {
            let start = text
                .find("      - name: completed\n")
                .expect("creation outcome starts");
            let end = text[start..]
                .find("      - name: rejected\n")
                .map(|offset| start + offset)
                .expect("following refusal starts");
            let default = &text[start..end];
            let conditional = default.replacen(
                "      - name: completed\n",
                "      - name: conditional\n        when: note == first\n",
                1,
            )
            .replacen(
                "          - contract.local.PrivateEmission\n          - contract.local.First\n",
                "          - contract.local.First\n          - contract.local.PrivateEmission\n",
                1,
            )
            .replacen(
                "child_id: {generated: true}",
                "child_id: input.first_child_id",
                1,
            );
            let default = default.replacen(
                "child_id: {generated: true}",
                "child_id: input.second_child_id",
                1,
            );
            text = format!(
                "{}{}{}{}",
                &text[..start],
                conditional,
                default,
                &text[end..]
            );
            text = text.replacen(
                "      - name: note\n        type: contract.local.Shared\n    response:\n",
                "      - name: note\n        type: contract.local.Shared\n      - name: first_child_id\n        type: contract.local.ChildId\n      - name: second_child_id\n        type: contract.local.ChildId\n    response:\n",
                1,
            );
        } else if label == "domains/foreign.yaml" {
            text = text.replacen(
                "      - name: note\n        type: contract.local.Shared\n\n  - name: contract.foreign.DeliveryFailed\n",
                "      - name: note\n        type: contract.local.Shared\n      - name: first_child_id\n        type: contract.local.ChildId\n      - name: second_child_id\n        type: contract.local.ChildId\n\n  - name: contract.foreign.DeliveryFailed\n",
                1,
            );
        } else if label == "wiring.yaml" {
            text = text.replacen(
                "      note: event.note\n    delivery: at_most_once\n",
                "      note: event.note\n      first_child_id: event.first_child_id\n      second_child_id: event.second_child_id\n    delivery: at_most_once\n",
                1,
            );
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

fn selected<'a>(ir: &'a EssIr, plan: &'a SynthesisPlan) -> ServiceIr<'a> {
    extract(
        ir,
        plan,
        &ComponentName::new("local-service").expect("component name"),
    )
    .expect("service extracts")
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("qualified name")
}

fn supplied_bound(binding: &CommandBinding) -> serde_json::Map<String, Value> {
    binding
        .slots
        .iter()
        .filter_map(|(slot, value)| {
            let supplied = match &value.target {
                BoundTarget::ExternalEvidence { .. } => Some(json!(false)),
                BoundTarget::LogicalIdentity { .. } => {
                    panic!("direct input identities must not need a slot")
                }
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
            supplied.map(|value| (format!("b{:08}", slot.index()), value))
        })
        .collect()
}

fn assert_selected_runtime_branches(lowered: &LoweredService, binding: &CommandBinding) {
    let mut registry = entity_core::Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry.validate_all().expect("closure validates");
    let runtime = Runtime::new(&registry);
    let first_branch_id = json!("278f4f3a-c8b8-4e86-9a16-2c385910fc68");
    let selected_default_id = json!("84f3e37f-4e8a-48c4-b019-895572d6c3ec");
    let bound = supplied_bound(binding);
    for (note, expected, identity_event) in [
        ("first", &first_branch_id, 1),
        ("keep exactly", &selected_default_id, 0),
    ] {
        let arguments = json!({
            "input": {
                "owner_id": "78e993a6-ac0d-42c0-a05c-8f2ee1898ee9",
                "note": note,
                "first_child_id": first_branch_id.clone(),
                "second_child_id": selected_default_id.clone()
            },
            "bound": Value::Object(bound.clone())
        });
        let decision = runtime
            .decide_create_derived("contract.local.Child", 1, arguments.clone())
            .expect("runtime derives the selected identity")
            .into_decision()
            .expect("selected creation outcome accepts");
        assert_eq!(&decision.instance.fields["child_id"], expected);
        assert_eq!(
            &decision.events[identity_event].payload["child_id"],
            expected
        );

        let supplied = runtime
            .decide_create(
                "contract.local.Child",
                1,
                decision.instance.id.clone(),
                arguments,
            )
            .expect("the compatible supplied-address path decides")
            .into_decision()
            .expect("the same branch accepts");
        assert_eq!(decision, supplied);
        assert_eq!(
            replay(std::slice::from_ref(&decision.record)).expect("record replays"),
            decision.instance
        );
    }
}

#[test]
fn selected_creation_outcomes_keep_their_authored_identity_and_event_coordinate() {
    let ir = compile_with_two_creation_outcomes();
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(
        &selected(&ir, &plan),
        &LoweringOptions {
            definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
                .into_iter()
                .map(|entry| (name(entry), NonZeroU32::new(1).expect("nonzero")))
                .collect(),
            scales: BTreeMap::new(),
        },
    )
    .expect("admitted service lowers");
    let binding = &lowered.bindings().commands()[&name("contract.local.Run")];
    assert_selected_runtime_branches(&lowered, binding);

    let InstanceBinding::SelectedOutcome { identities } = &binding.instance else {
        panic!("unequal authored identity sources require selected-outcome binding");
    };
    assert_eq!(identities.len(), 2);
    assert!(matches!(
        &identities[&ess_domain::command::OutcomeName::new("conditional").expect("outcome")]
            .logical_identity,
        IdentityValue::InputField { field } if field == "first_child_id"
    ));
    assert_eq!(
        identities[&ess_domain::command::OutcomeName::new("conditional").expect("outcome")]
            .observed_at
            .occurrence,
        1
    );
    assert!(matches!(
        &identities[&ess_domain::command::OutcomeName::new("completed").expect("outcome")]
            .logical_identity,
        IdentityValue::InputField { field } if field == "second_child_id"
    ));
    assert_eq!(
        identities[&ess_domain::command::OutcomeName::new("completed").expect("outcome")]
            .observed_at
            .occurrence,
        0
    );
}
