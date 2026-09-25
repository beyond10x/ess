//! Final source-review regression for creation identity reuse across accepting outcomes.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use entity_core::{identity, FieldKind, Runtime};
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
            );
            text = format!("{}{}{}", &text[..start], conditional, &text[start..]);
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

#[test]
fn every_accepting_creation_outcome_reuses_the_one_observed_logical_identity() {
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
    let logical_id = json!("278f4f3a-c8b8-4e86-9a16-2c385910fc68");
    let other_id = json!("84f3e37f-4e8a-48c4-b019-895572d6c3ec");
    let mut bound = serde_json::Map::new();
    for (slot, value) in &binding.slots {
        let supplied = match &value.target {
            BoundTarget::ExternalEvidence { .. } => Some(json!(false)),
            BoundTarget::LogicalIdentity { .. } => Some(logical_id.clone()),
            BoundTarget::EventField { field, outcome, .. }
                if field == "child_id" && outcome.as_str() == "completed" =>
            {
                Some(other_id.clone())
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
        if let Some(value) = supplied {
            bound.insert(format!("b{:08}", slot.index()), value);
        }
    }

    let registry = {
        let mut registry = entity_core::Registry::new();
        for definition in lowered.definitions().values() {
            registry
                .register(definition.as_definition().clone())
                .expect("definition registers");
        }
        registry.validate_all().expect("closure validates");
        registry
    };
    let runtime = Runtime::new(&registry);
    let storage_id = identity::address(FieldKind::String, &logical_id).expect("identity address");
    let decision = runtime
        .decide_create(
            "contract.local.Child",
            1,
            storage_id,
            json!({
                "input": {
                    "owner_id": "78e993a6-ac0d-42c0-a05c-8f2ee1898ee9",
                    "note": "keep exactly"
                },
                "bound": Value::Object(bound)
            }),
        )
        .expect("runtime admits the lowered definition")
        .into_decision()
        .expect("default creation outcome accepts");

    assert_eq!(decision.instance.fields["child_id"], logical_id);
    assert_eq!(decision.events[0].payload["child_id"], logical_id);
}
