//! A lifecycle state's wire name (`ess/32`, beyond10x/ess#639) has no Entity Runtime form: a
//! `LifecycleDefinition` names its states and nothing spells them apart. Lowering refuses it by
//! name — `StateWireNameUnsupported` — rather than lowering a lifecycle whose stored and answered
//! state is the declared name where the specification promises the wire name.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringCode, LoweringDiagnostic, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

/// The contract fixture under `ess/32`, `Child`'s one state written as `states`.
fn contract(states: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let changes = [
        ("system.yaml", "format: ess/4", "format: ess/32".to_owned()),
        (
            "domains/local.yaml",
            "      states: [Active]\n",
            format!("      states: {states}\n"),
        ),
    ];
    let mut paths = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
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
    let (mut parsed, mut labels, mut sources) = (Vec::new(), Vec::new(), SourceMap::new());
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (target, before, after) in &changes {
            if label == *target {
                assert!(text.contains(before), "{target} holds {before}");
                text = text.replacen(before, after, 1);
            }
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

fn diagnostics(ir: &EssIr) -> Vec<LoweringDiagnostic> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("local-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
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
    match lower(&service, &options) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics.into_vec(),
    }
}

#[test]
fn a_state_wire_name_is_refused_by_name() {
    let diagnostics = diagnostics(&contract("[{name: Active, wire: active}]"));
    let refused: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LoweringCode::StateWireNameUnsupported)
        .collect();
    assert_eq!(refused.len(), 1, "{diagnostics:#?}");
    assert_eq!(
        refused[0].path, "contract.local.Child.lifecycle.states.Active",
        "{diagnostics:#?}"
    );
    assert!(
        refused[0].message.contains("`active`"),
        "{}",
        refused[0].message
    );
}

#[test]
fn a_bare_lifecycle_is_not_refused_for_it() {
    let diagnostics = diagnostics(&contract("[Active]"));
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != LoweringCode::StateWireNameUnsupported),
        "{diagnostics:#?}"
    );
}
