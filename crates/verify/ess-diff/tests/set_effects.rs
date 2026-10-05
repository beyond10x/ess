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

/// The desk model at ess/22 with `Invite`'s `affects:` entry parking the rows it selects
/// (beyond10x/ess#229).
fn parking() -> (String, String) {
    let before = DESK.replacen("format: ess/16", "format: ess/22", 1);
    let from = "            where: team == subject.team\n";
    assert!(before.contains(from), "{from}");
    let after = before.replacen(
        from,
        "            where: team == subject.team\n            moves: demo.desk.Session.park\n",
        1,
    );
    (before, after)
}

#[test]
fn issue_229_a_move_added_to_an_affects_entry_is_set_effect_changed() {
    let (before, after) = parking();
    let delta = diff(&ir(&before), &ir(&after)).unwrap();
    let json = delta.to_canonical_json();
    assert!(
        json.contains(r#""kind": "outcome-set-effect-changed""#),
        "{json}"
    );
    assert!(
        delta
            .changes()
            .iter()
            .any(|change| change.describe().contains("moves along `park`")),
        "the entry's move is on the line that changed: {json}"
    );
}

/// `RevokeTokens` deletes every token a filter selects; `DeleteUser` deletes its subject and, in
/// an `affects:` entry, every token it owns (ess/23, beyond10x/ess#452).
const DELETES: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-deletes.yaml");

#[test]
fn set_delete_diff_line_names_the_deletion() {
    let narrowed = DELETES.replacen(
        "{all: [user_id == input.user_id, scope == input.scope]}",
        "user_id == input.user_id",
        1,
    );
    assert_ne!(narrowed, DELETES);
    let delta = diff(&ir(DELETES), &ir(&narrowed)).unwrap();
    let json = delta.to_canonical_json();
    assert!(
        json.contains(r#""kind": "outcome-set-effect-changed""#),
        "{json}"
    );
    assert!(
        delta.changes().iter().any(|change| {
            let line = change.describe();
            line.contains("deletes every `demo.auth.Token`")
                && line.contains("scope == input.scope")
        }),
        "the bulk deletion is on the line that changed: {json}"
    );
    let entry = "        affects:\n          - entity: demo.auth.Token\n            where: user_id == subject.user_id\n            deletes: demo.auth.Token\n";
    assert!(DELETES.contains(entry), "{entry}");
    let without = DELETES.replacen(entry, "", 1);
    let delta = diff(&ir(&without), &ir(DELETES)).unwrap();
    let json = delta.to_canonical_json();
    assert!(
        delta.changes().iter().any(|change| {
            let line = change.describe();
            line.contains("`demo.auth.Token` where `user_id == subject.user_id`")
                && line.contains("deletes")
        }),
        "the deleting entry is on the line that changed: {json}"
    );
}
