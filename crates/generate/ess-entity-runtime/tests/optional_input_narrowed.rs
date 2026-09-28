//! A narrowed `Optional` input lowers to an unconditional copy (beyond10x/ess#169, `ess/16`,
//! `docs/design/optional-input-narrowing.md`).
//!
//! The IR records a narrowed read at its present type, so the branch copies `$args.input.<x>` into
//! a required field and an event the way it copies any required input, and does not fall back to
//! the if-present form an `Optional` read gets — which would leave a required field unset.

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
use ess_entity_runtime::{lower, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::json;

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

#[test]
fn a_narrowed_input_is_copied_unconditionally_into_the_entity_and_the_events() {
    let ir = contract(&[
        (
            "domains/local.yaml",
            "      - name: note\n        type: contract.local.Shared\n    response:",
            "      - name: note\n        type: Optional<contract.local.Shared>\n    response:",
        ),
        (
            "domains/local.yaml",
            "      - name: rejected\n        external: an upstream authority rejects the request\n",
            "      - name: note-missing\n        when: not defined(note)\n        error: contract.local.Rejected\n        summary: No note was sent.\n      - name: rejected\n        external: an upstream authority rejects the request\n",
        ),
    ]);
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("local-service").unwrap()).unwrap();
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
    let lowered = lower(&service, &options)
        .unwrap_or_else(|diagnostics| panic!("the service lowers: {diagnostics:?}"));
    let child = &lowered.definitions()[&QualifiedName::new("contract.local.Child").unwrap()];
    let completed = child
        .create
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "completed")
        .expect("the default branch");
    assert_eq!(completed.set["note"], json!("$args.input.note"));
    assert!(!completed.set_if_present.contains_key("note"));
    assert_eq!(
        completed.emits[0].payload["note"],
        json!("$args.input.note")
    );
    assert!(!completed.emits[0].payload_if_present.contains_key("note"));
}
