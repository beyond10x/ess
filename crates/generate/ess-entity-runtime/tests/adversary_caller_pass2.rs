//! Adversary, pass 2, `story:caller-value-source-and-guard` (beyond10x/ess#168): the Entity Runtime
//! refusal for a caller read the unit's own cases do not reach — one in an event payload only, and
//! one in a `when_subject:` guard.

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

/// The operator carries a label of the child's note type, read only in an event payload.
const LABEL: (&str, &str, &str) = (
    "domains/local.yaml",
    "  - name: contract.local.Operator\n    may:\n",
    "  - name: contract.local.Operator\n    attributes:\n      - {name: owner_id, type: contract.foreign.OwnerId}\n      - {name: label, type: contract.local.Shared}\n    may:\n",
);

#[test]
fn adversary_caller_pass2_a_caller_read_only_in_an_event_payload_is_refused_by_name() {
    let ir = contract(&[
        LABEL,
        (
            "domains/local.yaml",
            "          contract.local.Second:\n            note: input.note\n",
            "          contract.local.Second:\n            note: {caller: label}\n",
        ),
    ]);
    assert!(refused_codes(&ir).contains(&LoweringCode::CallerUnsupported));
}

#[test]
fn adversary_caller_pass2_a_when_subject_guard_reading_the_caller_is_refused_by_name() {
    let ir = contract(&[
        LABEL,
        (
            "wiring.yaml",
            "        - contract.local.Run\n",
            "        - contract.local.Run\n        - contract.local.Touch\n",
        ),
        (
            "domains/local.yaml",
            "      - contract.local.Admin\n",
            "      - contract.local.Admin\n      - contract.local.Touch\n",
        ),
        (
            "domains/local.yaml",
            "  - name: contract.local.Admin\n",
            "  - name: contract.local.Touch\n    input:\n      - name: child_id\n        type: contract.local.ChildId\n      - name: note\n        type: contract.local.Shared\n    outcomes:\n      - name: not-theirs\n        when_subject: {predicate: owner_id != caller.owner_id}\n        error: contract.local.Rejected\n      - name: touched\n        updates: contract.local.Child\n        instance: child_id\n        sets:\n          note: input.note\n        emits:\n          - contract.local.First\n\n  - name: contract.local.Admin\n",
        ),
    ]);
    let codes = refused_codes(&ir);
    assert!(
        codes.contains(&LoweringCode::CallerUnsupported),
        "a `when_subject:` guard reading the caller lowers with {codes:?}"
    );
}
