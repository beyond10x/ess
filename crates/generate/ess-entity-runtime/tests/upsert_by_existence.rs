//! Outcomes selected by whether the addressed record exists (ess/16, beyond10x/ess#164) have no
//! Entity Runtime definition: entity-core answers a missing row itself and selects no branch by
//! it, so lowering refuses both forms by name — `ExistenceSelectionUnsupported` — rather than
//! dropping the branch or lowering the creation as unconditional.

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

/// The focused contract fixture at ess/16, with each `(file, before, after)` edit applied once.
fn contract(changes: &[(&str, &str, &str)]) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let mut changes = changes.to_vec();
    changes.push(("system.yaml", "format: ess/4", "format: ess/16"));
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

/// An event publishing the identity a creation takes from its input.
const STORED: (&str, &str, &str) = (
    "domains/local.yaml",
    "  - name: contract.local.First\n",
    "  - name: contract.local.Stored\n    fields:\n      - name: child_id\n        type: contract.local.ChildId\n\n  - name: contract.local.First\n",
);

const CREATE: &str = "      - name: made\n        creates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.Stored]\n        payload:\n          contract.local.Stored: {child_id: input.child_id}\n        sets:\n          owner_id: input.owner_id\n          note: input.note\n";

const INPUT: &str = "    input:\n      - name: child_id\n        type: contract.local.ChildId\n      - name: owner_id\n        type: contract.foreign.OwnerId\n      - name: note\n        type: contract.local.Shared\n";

fn with_command(name: &str, outcomes: &str) -> EssIr {
    contract(&[
        STORED,
        (
            "domains/local.yaml",
            "  - name: contract.local.Admin\n",
            &format!(
                "  - name: contract.local.{name}\n{INPUT}    outcomes:\n{outcomes}\n  - name: contract.local.Admin\n"
            ),
        ),
        (
            "wiring.yaml",
            "        - contract.local.Run\n",
            &format!("        - contract.local.Run\n        - contract.local.{name}\n"),
        ),
    ])
}

#[test]
fn create_or_update_is_refused_by_name() {
    let ir = with_command(
        "Put",
        &format!(
            "      - name: stored\n        updates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.First]\n        sets:\n          note: input.note\n{}",
            CREATE.replace(
                "        creates:",
                "        unknown_instance: true\n        creates:"
            )
        ),
    );
    let codes = refused_codes(&ir);
    assert!(
        codes.contains(&LoweringCode::ExistenceSelectionUnsupported),
        "{codes:?}"
    );
}

#[test]
fn create_or_refuse_is_refused_by_name() {
    let ir = with_command(
        "Book",
        &format!(
            "{CREATE}      - {{name: taken, existing_instance: true, error: contract.local.Rejected}}\n"
        ),
    );
    let codes = refused_codes(&ir);
    assert!(
        codes.contains(&LoweringCode::ExistenceSelectionUnsupported),
        "{codes:?}"
    );
}

#[test]
fn a_plain_creation_from_input_is_not_refused_for_it() {
    let ir = with_command("Make", CREATE);
    assert!(!refused_codes(&ir).contains(&LoweringCode::ExistenceSelectionUnsupported));
}
