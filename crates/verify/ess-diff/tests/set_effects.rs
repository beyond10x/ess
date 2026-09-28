//! Changing a set effect is a semantic change (ess/16, beyond10x/ess#167, #175,
//! `docs/design/set-effects-over-filtered-instances.md`).
//!
//! `outcome-set-effect-changed` carries the outcome's `instances:` and `affects:` on each side, and needs
//! `ess-diff/9`, so an older reader refuses the delta rather than reading a changed filter as no
//! change. A delta that moves nothing about set effects keeps its earlier format.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, DeltaFormat};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const DESK: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("desk.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn edited(from: &str, to: &str) -> String {
    assert!(DESK.contains(from), "{from}");
    DESK.replacen(from, to, 1)
}

#[test]
fn a_changed_set_filter_is_set_effect_changed_at_diff_9() {
    let after = edited(
        "instances: {where: team == input.team}\n        emits: [demo.desk.TeamEnded]",
        "instances: {where: team != input.team}\n        emits: [demo.desk.TeamEnded]",
    );
    let delta = diff(&ir(DESK), &ir(&after)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/9", "{json}");
    assert!(
        json.contains(r#""kind": "outcome-set-effect-changed""#),
        "{json}"
    );
    assert!(json.contains("team != input.team"), "{json}");
    assert!(
        delta
            .changes()
            .iter()
            .any(|change| change.describe().contains("team == input.team")),
        "{json}"
    );
    assert!(delta
        .to_canonical_json_for(DeltaFormat::parse("ess-diff/8").unwrap())
        .is_err());
}

#[test]
fn a_changed_affects_entry_is_set_effect_changed() {
    let after = edited(
        "            sets:\n              on_hold: true",
        "            sets:\n              on_hold: false",
    );
    let delta = diff(&ir(DESK), &ir(&after)).unwrap();
    let json = delta.to_canonical_json();
    assert!(
        json.contains(r#""kind": "outcome-set-effect-changed""#),
        "{json}"
    );
}

#[test]
fn a_delta_that_moves_nothing_about_set_effects_keeps_its_format() {
    let delta = diff(
        &ir(DESK),
        &ir(&edited(
            "demo.desk.Invited: {session_id: input.session_id}",
            "demo.desk.Invited: {session_id: {generated: true}}",
        )),
    )
    .unwrap();
    let json = delta.to_canonical_json();
    assert!(!json.contains("outcome-set-effect-changed"), "{json}");
    assert!(delta.format.major() < 9, "{json}");
}
