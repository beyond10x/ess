//! An outcome's `accepts: nothing` (ess/15), `returns:` (ess/17) and caller-decided refusal
//! (ess/16) are compared, not left to the residual (beyond10x/ess#253).
//!
//! Each flag moving is its own kind on the outcome — `outcome-accepts-nothing-changed`,
//! `outcome-returns-changed`, `outcome-decided-by-caller-changed` — shaped and related as
//! `outcome-refuses-changed` is: `{outcome, before, after}` booleans, `changed`. Each is
//! `ess-diff/12` vocabulary. `retains_result` is derived from the command's `replays:`, so a
//! change to it is reported by the replay or outcome change that caused it and never twice.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("flags.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn delta(before: &str, after: &str) -> EssDelta {
    diff(&ir(before), &ir(after)).unwrap()
}

/// The delta carries `kind` on `outcome`, from `before` to `after`, at `ess-diff/12`, beside
/// nothing unclassified; it reads back, and every writer below `/12` refuses it.
fn assert_flag(delta: &EssDelta, id: &str, before: bool, after: bool) {
    let json = delta.to_canonical_json();
    assert!(!json.contains("unclassified"), "{json}");
    assert_eq!(delta.format.to_string(), "ess-diff/12", "{json}");
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` in {json}"));
    assert_eq!(change.relation(), SemanticRelation::Changed);
    assert_eq!(change.minimum_format(), 12);
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let changed = document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == id)
        .map(|entry| &entry["change"]["changed"])
        .unwrap();
    assert_eq!(changed["before"], before, "{json}");
    assert_eq!(changed["after"], after, "{json}");
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), *delta, "it reads back");
    for old in 3..=11 {
        assert!(
            delta
                .to_canonical_json_for(format!("ess-diff/{old}").parse().unwrap())
                .is_err(),
            "ess-diff/{old} refuses `{id}`"
        );
    }
    // Each direction is the same kind.
    let text = ess_diff::render::text(delta);
    assert!(text.contains(id), "{text}");
}

const CHECKED: &str = "format: ess/19\nsystem: demo\nversion: v1\ndomain: demo.order\n\
events:\n  - name: demo.order.Checked\n    fields:\n      - {name: quantity, type: Integer}\n\
commands:\n  - name: demo.order.Check\n    input:\n      - {name: quantity, type: Integer}\n    \
outcomes:\n      - name: checked\n        emits: [demo.order.Checked]\n        payload:\n          \
demo.order.Checked: {quantity: input.quantity}\n";

#[test]
fn accepting_nothing_is_classified_both_ways() {
    let nothing = CHECKED.replace(
        "        emits: [demo.order.Checked]\n        payload:\n          \
         demo.order.Checked: {quantity: input.quantity}\n",
        "        accepts: nothing\n",
    );
    assert_ne!(nothing, CHECKED);
    let id = "command/demo.order.Check/outcome-accepts-nothing-changed/checked";
    assert_flag(&delta(CHECKED, &nothing), id, false, true);
    assert_flag(&delta(&nothing, CHECKED), id, true, false);
}

const LIBRARY: &str = "format: ess/17\nsystem: library\nversion: v1\ndomain: library.api\n\
commands:\n  - name: library.api.Read\n    response:\n      - {name: value, type: Integer}\n    \
outcomes:\n      - {name: returned, returns: true}\n";

#[test]
fn returning_the_response_is_classified_both_ways() {
    let nothing = LIBRARY.replace("returns: true", "accepts: nothing");
    assert_ne!(nothing, LIBRARY);
    let id = "command/library.api.Read/outcome-returns-changed/returned";
    assert_flag(&delta(LIBRARY, &nothing), id, true, false);
    assert_flag(&delta(&nothing, LIBRARY), id, false, true);
}

fn guarded(guard: &str) -> String {
    format!(
        "format: ess/16\nsystem: demo\nversion: v1\ndomain: demo.order\n\
         actors:\n  - name: demo.order.Clerk\n    attributes:\n      - {{name: limit, type: Integer}}\n    \
         may: [demo.order.Check]\n\
         errors:\n  - name: demo.order.TooMany\n\
         events:\n  - name: demo.order.Checked\n    fields:\n      - {{name: quantity, type: Integer}}\n\
         commands:\n  - name: demo.order.Check\n    input:\n      - {{name: quantity, type: Integer}}\n    \
         outcomes:\n      - name: too-many\n        when: {guard}\n        error: demo.order.TooMany\n      \
         - name: checked\n        emits: [demo.order.Checked]\n        payload:\n          \
         demo.order.Checked: {{quantity: input.quantity}}\n"
    )
}

#[test]
fn a_refusal_the_caller_decides_is_classified_both_ways() {
    let (input, caller) = (
        guarded("quantity > 10"),
        guarded("quantity != caller.limit"),
    );
    let id = "command/demo.order.Check/outcome-decided-by-caller-changed/too-many";
    assert_flag(&delta(&input, &caller), id, false, true);
    assert_flag(&delta(&caller, &input), id, true, false);
}

#[test]
fn a_retained_result_moves_only_with_the_replay_that_derives_it() {
    let before = include_str!("../../ess-conformance/tests/fixtures/retained-replay.yaml").replace(
        "      - name: replayed",
        "      - name: alternate\n        external: alternative original success\n        \
         creates: retained.core.Record\n        instance: record_id\n        \
         emits: [retained.core.Seeded]\n        payload:\n          retained.core.Seeded:\n            \
         record_id: {generated: true}\n      - name: replayed",
    );
    let after = before.replace("replays: seeded", "replays: alternate");
    let retains = |model: &EssIr| -> Vec<(String, bool)> {
        let command = model.commands().values().next().unwrap();
        command
            .outcomes
            .iter()
            .map(|o| (o.name.as_str().to_owned(), o.retains_result))
            .collect()
    };
    let (old, new) = (ir(&before), ir(&after));
    assert_ne!(retains(&old), retains(&new), "the derived flag moved");
    let delta = diff(&old, &new).unwrap();
    let ids: Vec<String> = delta.changes().iter().map(|c| c.id().to_string()).collect();
    assert_eq!(
        ids,
        ["command/retained.core.Seed/outcome-replay-changed/replayed"],
        "the replay reports it, once"
    );
}
