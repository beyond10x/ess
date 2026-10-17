//! An actor becoming the system, and a system-issued command's `await_within:` moving (`ess/25`,
//! beyond10x/ess#542, `docs/design/system-issued-commands.md`, *Every lane*, the diff row), are
//! typed changes, never `system/<name>/unclassified-changed`.
//!
//! - `actor/<a>/system-changed {before, after}`: an actor that becomes the system stops being a
//!   caller, as a grant removed does, so it breaks callers; one that stops being the system breaks
//!   nobody.
//! - `command/<c>/await-within-changed {before, after}`: how soon the system claims to take its own
//!   step, narrowed or widened, breaks nobody.
//! - A command becoming system-issued is reported through those and the grant changes, never twice.
//! - Both kinds are `ess-diff/20` vocabulary.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, diff, Compatibility, EssDelta, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `{system}` is written on the scheduler, `{await}` on the command it issues.
const MODEL: &str = "\
format: ess/25
system: queueing
version: v1
domain: queueing.entry
types:
  - {name: queueing.entry.EntryId, kind: newtype, of: Uuid}
entities:
  - name: queueing.entry.Entry
    identity: {name: entry_id, type: queueing.entry.EntryId}
    lifecycle:
      initial: Waiting
      states: [Waiting, Assigned]
      terminal: [Assigned]
      transitions:
        - {name: assign, from: [Waiting], to: Assigned}
errors:
  - {name: queueing.entry.EntryStateConflict, summary: The entry is not waiting., fields: []}
events:
  - name: queueing.entry.Enqueued
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
  - name: queueing.entry.Assigned
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
actors:
  - name: queueing.entry.Caller
    may: [queueing.entry.Enqueue]
  - name: queueing.entry.Scheduler
{system}    may: [queueing.entry.Assign]
commands:
  - name: queueing.entry.Enqueue
    input: []
    outcomes:
      - name: enqueued
        creates: queueing.entry.Entry
        instance: entry_id
        emits: [queueing.entry.Enqueued]
        payload:
          queueing.entry.Enqueued: {entry_id: {generated: true}}
  - name: queueing.entry.Assign
{await}    input:
      - {name: entry_id, type: queueing.entry.EntryId}
    outcomes:
      - name: assigned
        moves: queueing.entry.Entry.assign
        instance: entry_id
        emits: [queueing.entry.Assigned]
        payload:
          queueing.entry.Assigned: {entry_id: input.entry_id}
      - {name: wrong-state, wrong_state: true, error: queueing.entry.EntryStateConflict}
";

const SYSTEM: &str = "actor/queueing.entry.Scheduler/system-changed";
const AWAIT: &str = "command/queueing.entry.Assign/await-within-changed";
const UNCLASSIFIED: &str = "system/queueing/unclassified-changed";

fn ir(system: Option<bool>, await_within: Option<&str>) -> EssIr {
    let system = system.map_or_else(String::new, |value| format!("    system: {value}\n"));
    let await_within =
        await_within.map_or_else(String::new, |value| format!("    await_within: {value}\n"));
    let text = MODEL
        .replace("{system}", &system)
        .replace("{await}", &await_within);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queueing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn driven() -> EssIr {
    ir(None, None)
}

fn issued() -> EssIr {
    ir(Some(true), None)
}

fn ids(delta: &EssDelta) -> Vec<String> {
    delta
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

fn answers(delta: &EssDelta, id: &str) -> [Compatibility; 3] {
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` in {}", delta.to_canonical_json()));
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    [
        compatibility.callers(),
        compatibility.readers(),
        compatibility.history(),
    ]
}

/// The `changed` member of the change `id` in a written delta.
fn changed(json: &str, id: &str) -> serde_json::Value {
    let document: serde_json::Value = serde_json::from_str(json).unwrap();
    document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` in {json}"))["change"]["changed"]
        .clone()
}

use Compatibility::{Breaking as B, Compatible as C};

#[test]
fn an_actor_becoming_the_system_breaks_its_callers() {
    let delta = classified(&driven(), &issued()).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(
        ids(&delta),
        [SYSTEM],
        "the command becoming system-issued is not reported twice: {json}"
    );
    assert_eq!(answers(&delta, SYSTEM), [B, C, C], "{json}");
    let changed = changed(&json, SYSTEM);
    assert_eq!(
        (&changed["before"], &changed["after"]),
        (&serde_json::json!(false), &serde_json::json!(true)),
        "{json}"
    );
    let text = ess_diff::render::text(&delta);
    assert!(text.contains(SYSTEM), "{text}");
}

#[test]
fn an_actor_no_longer_the_system_breaks_nobody() {
    let delta = classified(&issued(), &driven()).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(ids(&delta), [SYSTEM], "{json}");
    assert_eq!(answers(&delta, SYSTEM), [C, C, C], "{json}");
}

#[test]
fn adding_removing_or_moving_the_await_bound_breaks_nobody() {
    let bounded = ir(Some(true), Some("PT5S"));
    let longer = ir(Some(true), Some("PT9S"));
    for (before, after) in [
        (&issued(), &bounded),
        (&bounded, &issued()),
        (&bounded, &longer),
        (&longer, &bounded),
    ] {
        let delta = classified(before, after).unwrap();
        let json = delta.to_canonical_json();
        assert_eq!(ids(&delta), [AWAIT], "{json}");
        assert_eq!(answers(&delta, AWAIT), [C, C, C], "{json}");
    }
    let delta = classified(&bounded, &longer).unwrap();
    let json = delta.to_canonical_json();
    let changed = changed(&json, AWAIT);
    assert_eq!(
        (&changed["before"], &changed["after"]),
        (&serde_json::json!("PT5S"), &serde_json::json!("PT9S")),
        "{json}"
    );
    let text = ess_diff::render::text(&delta);
    assert!(text.contains("awaited within PT5S → PT9S"), "{text}");
}

#[test]
fn becoming_the_system_with_a_bound_is_two_changes_and_no_residual() {
    let delta = classified(&driven(), &ir(Some(true), Some("PT5S"))).unwrap();
    let json = delta.to_canonical_json();
    let mut found = ids(&delta);
    found.sort();
    let mut expected = vec![AWAIT.to_owned(), SYSTEM.to_owned()];
    expected.sort();
    assert_eq!(found, expected, "{json}");
    assert!(!json.contains(UNCLASSIFIED), "{json}");
}

#[test]
fn the_default_delta_carries_them_as_ess_diff_19_and_earlier_formats_refuse_them() {
    let delta = diff(&driven(), &ir(Some(true), Some("PT5S"))).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/20", "{json}");
    assert!(
        delta.compatibility().is_some(),
        "an ess-diff/20 delta is classified: {json}"
    );
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");

    let refused = delta
        .to_canonical_json_for("ess-diff/19".parse().unwrap())
        .expect_err("ess-diff/19 cannot write it");
    assert!(
        refused.to_string().contains("not representable"),
        "{refused}"
    );
    let mut older: serde_json::Value = serde_json::from_str(&json).unwrap();
    older["format"] = serde_json::Value::from("ess-diff/19");
    let raw: RawEssDelta = serde_json::from_value(older).unwrap();
    let refused = EssDelta::try_from(raw).expect_err("an `ess-diff/19` reader refuses it");
    assert!(
        refused.to_string().contains("outside the vocabulary"),
        "{refused}"
    );
}

#[test]
fn writing_system_false_is_no_change() {
    let delta = diff(&driven(), &ir(Some(false), None)).unwrap();
    assert!(delta.is_empty(), "{}", delta.to_canonical_json());
    let delta = diff(&issued(), &issued()).unwrap();
    assert!(delta.is_empty(), "{}", delta.to_canonical_json());
}
