//! Outcomes selected by whether the addressed record exists (ess/16, beyond10x/ess#164) in
//! generated seams: every code target refuses both forms by name, as it refuses `Json`, rather than
//! emitting a seam or an explorer that selects the branch some other way.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

const CREATED: &str = "demo.items.PutItem.outcomes.created.unknown_instance";
const TAKEN: &str = "demo.items.BookSlot.outcomes.already-booked.existing_instance";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("upsert-by-existence.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_code_target_refuses_both_forms_by_name() {
    let ir = ir(MODEL);
    for target in [Target::Rust, Target::Go] {
        let failure = synthesize_for(&ir, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses selection by existence"));
        let text = format!("{failure:?}");
        for named in [CREATED, TAKEN] {
            assert!(text.contains(named), "{target:?} names {named}: {text}");
        }
        assert!(
            text.contains(&format!("{:?}", TargetFailureCode::MissingRepresentation)),
            "{text}"
        );
    }
}

/// Each target's own `workspace` refuses the branches too, so a caller that skips
/// `synthesize_for` gets the same failure.
#[test]
fn every_direct_workspace_entry_refuses_both_forms_by_name() {
    let ir = ir(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    let failures = [
        ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
        ("clap", ess_synth::clap::workspace(&ir, &plan).err()),
    ];
    for (target, failure) in failures {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses selection by existence"));
        for named in [CREATED, TAKEN] {
            assert!(
                failure.to_canonical_json().contains(named),
                "{target} names {named}: {}",
                failure.to_canonical_json()
            );
        }
    }
}

#[test]
fn an_unknown_instance_refusal_is_not_refused_as_selection_by_existence() {
    // Without either form the model still synthesizes: the refusal is about these branches only.
    let model = MODEL
        .replace("        unknown_instance: true\n", "")
        .replace(
            "      - name: updated\n        updates: demo.items.Item\n        instance: item_id\n        emits: [demo.items.ItemStored]\n        payload:\n          demo.items.ItemStored: {item_id: input.item_id, label: input.label}\n        sets:\n          label: input.label\n        summary: An item with this id exists; its label is replaced.\n",
            "",
        )
        .replace(
            "      - {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}\n",
            "",
        );
    assert_ne!(model, MODEL);
    let ir = ir(&model);
    for target in [Target::Rust, Target::Go] {
        if let Err(failure) = synthesize_for(&ir, target) {
            let text = format!("{failure:?}");
            assert!(
                !text.contains("unknown_instance") && !text.contains("existing_instance"),
                "{target:?}: {text}"
            );
        }
    }
}
