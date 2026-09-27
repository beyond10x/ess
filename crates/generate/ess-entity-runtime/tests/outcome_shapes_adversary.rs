//! Adversarial cases: the two ess/15 outcome shapes `tests/outcome_shapes.rs` does not lower —
//! an `unknown_instance:` branch and `accepts: nothing` — are refused by name too, as the design
//! page's status line says of "all four outcome shapes".

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
use ess_entity_runtime::{lower, LoweringCode, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

fn contract(changes: &[(&str, &str, &str)]) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let mut changes = changes.to_vec();
    changes.push(("system.yaml", "format: ess/4", "format: ess/15"));
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

fn refused_codes(ir: &EssIr) -> Vec<LoweringCode> {
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
        Err(diagnostics) => diagnostics
            .into_vec()
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect(),
    }
}

#[test]
fn adversary_an_unknown_instance_branch_is_refused_by_name() {
    let ir = contract(&[
        (
            "domains/local.yaml",
            "  - name: contract.local.Admin\n",
            "  - name: contract.local.Edit\n    input:\n      - name: child_id\n        type: contract.local.ChildId\n      - name: note\n        type: contract.local.Shared\n    outcomes:\n      - name: edited\n        updates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.First]\n        sets:\n          note: input.note\n      - name: missing\n        unknown_instance: true\n        error: contract.local.Rejected\n\n  - name: contract.local.Admin\n",
        ),
        (
            "wiring.yaml",
            "        - contract.local.Run\n",
            "        - contract.local.Run\n        - contract.local.Edit\n",
        ),
    ]);
    let codes = refused_codes(&ir);
    assert!(
        codes.contains(&LoweringCode::OutcomeShapeUnsupported),
        "{codes:?}"
    );
}

#[test]
fn adversary_an_accepts_nothing_branch_is_refused_by_name() {
    let ir = contract(&[
        (
            "domains/local.yaml",
            "  - name: contract.local.Admin\n",
            "  - name: contract.local.Poke\n    input:\n      - name: child_id\n        type: contract.local.ChildId\n      - name: note\n        type: contract.local.Shared\n    outcomes:\n      - name: skipped\n        when: note == \"\"\n        accepts: nothing\n      - name: noted\n        updates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.First]\n        sets:\n          note: input.note\n\n  - name: contract.local.Admin\n",
        ),
        (
            "wiring.yaml",
            "        - contract.local.Run\n",
            "        - contract.local.Run\n        - contract.local.Poke\n",
        ),
    ]);
    let codes = refused_codes(&ir);
    assert!(
        codes.contains(&LoweringCode::OutcomeShapeUnsupported),
        "{codes:?}"
    );
}
