//! Adversary pass 2 for `story:optional-input-narrowed-after-refusal` (beyond10x/ess#169): the
//! narrowed default spelled `when: true`.
//!
//! `ess-domain` treats `when: true` as the command's default (`Outcome::is_unconditional`), so a
//! sibling refusing `not defined(note)` narrows it, and the IR records `input.note` at its present
//! type. Entity Runtime lowers that read to an unconditional `$args.input.note`. That copy is sound
//! only if entity-core tries the refusal before the `when: true` branch; entity-core selects the
//! first branch whose `when` holds, and moves only a branch with no `when` to the end.

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
    contract_or_refusal(changes).unwrap_or_else(|errors| panic!("{errors}"))
}

fn contract_or_refusal(
    changes: &[(&str, &str, &str)],
) -> Result<EssIr, ess_primitives::error::ValidationErrors> {
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
    let specification = Specification::assemble(parsed)?;
    Ok(compile_locating(&specification, &sources, &labels).expect("fixture compiles"))
}

const OPTIONAL_NOTE: (&str, &str, &str) = (
    "domains/local.yaml",
    "      - name: note\n        type: contract.local.Shared\n    response:",
    "      - name: note\n        type: Optional<contract.local.Shared>\n    response:",
);
const WHEN_TRUE: (&str, &str, &str) = (
    "domains/local.yaml",
    "      - name: completed\n        creates: contract.local.Child\n",
    "      - name: completed\n        when: \"true\"\n        creates: contract.local.Child\n",
);
const REFUSAL: (&str, &str, &str) = (
    "domains/local.yaml",
    "      - name: rejected\n        external: an upstream authority rejects the request\n",
    "      - name: note-missing\n        when: not defined(note)\n        error: contract.local.Rejected\n        summary: No note was sent.\n      - name: rejected\n        external: an upstream authority rejects the request\n",
);

#[test]
fn a_when_true_branch_declared_before_the_refusal_is_not_narrowed() {
    // Coordinator decision, correction round 2: `when: true` is not the default, so the refusal
    // does not narrow it and the unconditional copy entity-core would try first never compiles.
    let errors = contract_or_refusal(&[OPTIONAL_NOTE, WHEN_TRUE, REFUSAL])
        .expect_err("the `when: true` branch copies an input it is not known to hold");
    assert!(
        errors.as_slice().iter().any(|error| error.code
            == ess_primitives::error::ValidationCode::TypeMismatch
            && error.location.contains("completed")),
        "{errors}"
    );
}

#[test]
fn a_narrowed_default_is_tried_after_the_refusal_that_narrows_it() {
    let ir = contract(&[OPTIONAL_NOTE, REFUSAL]);
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
    let outcomes = &child.create.outcomes;
    let position = |name: &str| {
        outcomes
            .iter()
            .position(|outcome| outcome.name == name)
            .unwrap_or_else(|| panic!("`{name}` is lowered"))
    };
    let completed = &outcomes[position("completed")];
    // The narrowing premise holds in the IR: the copy is unconditional.
    assert_eq!(completed.set["note"], json!("$args.input.note"));
    // And entity-core must be able to reach the refusal with `note` absent.
    let order: Vec<_> = outcomes
        .iter()
        .map(|outcome| (outcome.name.clone(), outcome.when.is_some()))
        .collect();
    assert!(
        completed.when.is_none() || position("note-missing") < position("completed"),
        "entity-core selects the first branch whose guard holds; the `when: true` branch copies \
         `$args.input.note` unconditionally and is tried before the refusal of its absence: \
         {order:?}"
    );
}
