//! Adversary pass 1 for beyond10x/ess#230: a `references` carried by the source's own identity
//! lowers to an Entity Runtime definition set the runtime's registry accepts, and is never an
//! ownership there.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use entity_core::Registry;
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

const MIRROR: &str = "  - name: contract.local.Mirror
    identity:
      name: child_id
      type: contract.local.ChildId
    fields:
      - name: note
        type: contract.local.Shared
    relations:
      - name: child
        kind: references
        target: contract.local.Child
        cardinality: one
        via: child_id
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
      transitions: []

actors:
";

fn contract() -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let mut paths = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "yaml") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let (mut parsed, mut labels, mut sources) = (Vec::new(), Vec::new(), SourceMap::new());
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        if label == "domains/local.yaml" {
            let before = text.clone();
            text = text.replacen("\nactors:\n", &format!("\n{MIRROR}"), 1);
            assert_ne!(text, before, "the mirror entity was inserted");
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

#[test]
fn an_identity_carried_reference_lowers_to_definitions_the_registry_accepts() {
    let ir = contract();
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("local-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [
            "contract.foreign.Owner",
            "contract.local.Child",
            "contract.local.Mirror",
        ]
        .into_iter()
        .map(|name| {
            (
                QualifiedName::new(name).unwrap(),
                NonZeroU32::new(1).unwrap(),
            )
        })
        .collect(),
        scales: BTreeMap::new(),
    };
    let lowered = lower(&service, &options)
        .unwrap_or_else(|diagnostics| panic!("lowering refuses: {:?}", diagnostics.into_vec()));
    let mirror = lowered
        .definitions()
        .get(&QualifiedName::new("contract.local.Mirror").unwrap())
        .expect("the mirror entity is lowered");
    let definition = mirror.as_definition();
    let relation = &definition.relations["child"];
    assert_eq!(relation.via, "child_id");
    assert_eq!(relation.kind, entity_core::RelationKind::References);
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry
        .validate_all()
        .unwrap_or_else(|errors| panic!("the registry accepts the closure: {errors:?}"));
}
