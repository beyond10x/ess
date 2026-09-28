//! An `input_absent:` branch (ess/16, beyond10x/ess#170) has no Entity Runtime definition: an
//! absent request body is a transport fact entity-core never sees, so lowering refuses it by name —
//! `InputAbsentUnsupported` — rather than dropping the branch.

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

#[test]
fn an_input_absent_branch_is_refused_by_name() {
    let ir = contract(&[(
        "domains/local.yaml",
        "      - name: rejected\n",
        "      - name: body-missing\n        input_absent: true\n        error: contract.local.Rejected\n      - name: rejected\n",
    )]);
    assert!(
        refused_codes(&ir).contains(&LoweringCode::InputAbsentUnsupported),
        "{:?}",
        refused_codes(&ir)
    );
}

#[test]
fn the_fixture_at_ess_16_without_the_marker_is_not_refused_for_it() {
    let ir = contract(&[]);
    assert!(!refused_codes(&ir).contains(&LoweringCode::InputAbsentUnsupported));
}
