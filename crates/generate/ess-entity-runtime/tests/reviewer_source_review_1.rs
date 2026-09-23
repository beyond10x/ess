//! Independent source-review regressions for the ESS to Entity Runtime lowerer.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, BoundPresence, BoundTarget, LoweringOptions};
use ess_service_contract::{extract, ServiceIr};
use ess_synth::SynthesisPlan;
use serde_json::json;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract")
}

fn compile_replacing(target: &str, before: &str, after: &str) -> EssIr {
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

    let mut replaced = false;
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
        if label == target {
            assert!(text.contains(before), "review mutation source is present");
            text = text.replacen(before, after, 1);
            replaced = true;
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).expect("fixture parses"),
        ));
        labels.push(label);
    }
    assert!(replaced, "review mutation target is present");
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

fn options() -> LoweringOptions {
    LoweringOptions {
        definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
            .into_iter()
            .map(|entry| (name(entry), NonZeroU32::new(1).expect("nonzero")))
            .collect(),
        scales: BTreeMap::new(),
    }
}

#[test]
fn slot_canonicalization_does_not_rewrite_an_authored_literal_that_looks_like_a_slot() {
    let ir = compile_replacing(
        "domains/local.yaml",
        "            marker: exact-marker-007",
        "            marker: '$args.bound.b00000002'",
    );
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(&selected(&ir, &plan), &options()).expect("admitted service lowers");
    let completed = lowered.definitions()[&name("contract.local.Child")]
        .create
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "completed")
        .expect("completed branch remains present after normalization");

    assert_eq!(
        completed.emits[0].payload["marker"],
        json!("$$args.bound.b00000002"),
        "ER escapes authored data that equals template syntax; the runtime case proves the decoded value"
    );
}

#[test]
fn required_response_reused_by_an_optional_event_field_keeps_required_presence() {
    let ir = compile_replacing(
        "domains/local.yaml",
        "    response:\n      - name: receipt\n        type: contract.local.Receipt\n      - name: optional_receipt\n        type: Optional<String>",
        "    response:\n      - name: receipt\n        type: contract.local.Receipt\n      - name: optional_receipt\n        type: String",
    );
    let plan = SynthesisPlan::of(&ir);
    let lowered = lower(&selected(&ir, &plan), &options()).expect("admitted service lowers");
    let binding = &lowered.bindings().commands()[&name("contract.local.Run")];
    let response = binding
        .slots
        .values()
        .find(|value| {
            matches!(
                value.target,
                BoundTarget::ResponseField { ref field, .. } if field == "optional_receipt"
            )
        })
        .expect("the reused response has one semantic slot");

    assert_eq!(response.presence, BoundPresence::Required);
    let completed = lowered.definitions()[&name("contract.local.Child")]
        .create
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "completed")
        .expect("completed branch remains present after normalization");
    assert!(completed.emits[0]
        .payload
        .as_object()
        .expect("event payload is an object")
        .contains_key("optional_receipt"));
    assert!(!completed.emits[0]
        .payload_if_present
        .contains_key("optional_receipt"));
    assert!(completed.responds.contains_key("optional_receipt"));
    assert!(!completed
        .responds_if_present
        .contains_key("optional_receipt"));
}
