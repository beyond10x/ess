//! A setup gap is reported once per cause, not once per derived scenario it also refuses
//! (downstream report, 2026-10-18).
//!
//! A one-time response multiplies the scenarios synthesis owes: one disclosure cell per actor and
//! follow-up. A gap the ordinary outcome scenario already has, such as an entity no command
//! creates, used to come back once per cell. `ess verify conform synthesize` now prints one block
//! per cause and one line counting the cells it also refuses; `ess specify validate --format json`
//! still lists every refused id, each derived one linked to its cause by `caused_by`.

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// `IssueToken` returns a one-time `secret`; `EditNote` updates a `Note` nothing creates. `Owner`
/// may send both, `Guest` and `Auditor` neither.
const MODEL: &str = r"format: ess/24
system: vault
version: v1
domain: vault.keys
types:
  - {name: vault.keys.NoteId, kind: newtype, of: Uuid}
  - {name: vault.keys.TokenId, kind: newtype, of: Uuid}
entities:
  - name: vault.keys.Note
    identity: {name: note_id, type: vault.keys.NoteId}
    fields:
      - {name: body, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: vault.keys.Token
    identity: {name: token_id, type: vault.keys.TokenId}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
actors:
  - {name: vault.keys.Owner, may: [vault.keys.IssueToken, vault.keys.EditNote]}
  - {name: vault.keys.Guest, may: []}
  - {name: vault.keys.Auditor, may: []}
events:
  - name: vault.keys.TokenIssued
    fields:
      - {name: token_id, type: vault.keys.TokenId}
  - name: vault.keys.NoteEdited
    fields:
      - {name: note_id, type: vault.keys.NoteId}
commands:
  - name: vault.keys.IssueToken
    input: []
    response:
      - {name: secret, type: String}
    outcomes:
      - name: issued
        creates: vault.keys.Token
        instance: token_id
        returns: true
        one_time_response: [secret]
        emits: [vault.keys.TokenIssued]
        payload:
          vault.keys.TokenIssued: {token_id: {generated: true}}
  - name: vault.keys.EditNote
    input:
      - {name: note_id, type: vault.keys.NoteId}
      - {name: body, type: String}
    outcomes:
      - name: edited
        updates: vault.keys.Note
        instance: note_id
        sets: {body: input.body}
        emits: [vault.keys.NoteEdited]
        payload:
          vault.keys.NoteEdited: {note_id: input.note_id}
views:
  - name: vault.keys.Notes
    source: vault.keys.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: vault.keys.NoteId}
      - {name: body, type: String}
";

const EDIT: &str = "vault.keys.EditNote/outcome/edited";
const CELL: &str = "vault.keys.IssueToken/disclosure/issued/secret";

fn model(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("refusal-causes")
        .join(format!("{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("system.yaml");
    fs::write(&path, MODEL).unwrap();
    path
}

fn ess(args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    output
}

#[test]
fn synthesize_prints_one_block_per_cause() {
    let path = model("text");
    let output = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        path.to_str().unwrap(),
    ]);
    let printed = String::from_utf8(output.stdout).unwrap();
    let refused: Vec<&str> = printed
        .lines()
        .filter(|line| line.starts_with("refused: ") || line.starts_with("  "))
        .collect();
    assert_eq!(
        refused,
        [
            "refused: refusal[ESS-SYNTH-004]: outcome vault.keys.EditNote/edited has no scenario \
             `vault.keys.EditNote/outcome/edited`",
            "  it needs an instance of `vault.keys.Note` to change without moving, and no outcome \
             creates one",
            "  help: give some command outcome `creates:` for this entity; nothing can act on an \
             instance nothing brings into existence",
            "  and 3 disclosure scenario(s) that send `vault.keys.EditNote`, refused for the same \
             cause",
            "refused: refusal[ESS-SYNTH-001]: outcome vault.keys.IssueToken/issued has no scenario \
             `vault.keys.IssueToken/disclosure/issued/secret/read/vault.keys.Notes/as/actor/\
             vault.keys.Auditor`",
            "  the disclosure cell cannot be arranged or observed: the read's originating caller \
             has no grant to issue this value",
            "  help: the line above names what the cell lacks; a `read/<view>` cell for an actor \
             not granted the originating command is refused by design and needs no change",
            "  and 1 more disclosure scenario(s) refused for the same cause",
        ],
        "{printed}"
    );
    assert!(
        printed
            .lines()
            .last()
            .unwrap()
            .ends_with(", 6 refusal(s), 2 distinct cause(s), nothing written"),
        "{printed}"
    );
}

#[test]
fn validate_json_keeps_every_id_and_links_it_to_its_cause() {
    let path = model("json");
    let output = ess(&[
        "specify",
        "validate",
        "--path",
        path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let completeness = &report["completeness"];
    assert_eq!(completeness["counts"]["unscenarioed"], 6);
    let linked: Vec<(&str, &str, Option<&str>)> = completeness["unscenarioed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["code"].as_str().unwrap(),
                entry["scenario"].as_str().unwrap(),
                entry.get("caused_by").map(|cause| cause.as_str().unwrap()),
            )
        })
        .collect();
    let auditor_read = format!("{CELL}/read/vault.keys.Notes/as/actor/vault.keys.Auditor");
    assert_eq!(
        linked,
        [
            ("ESS-SYNTH-004", EDIT, None),
            (
                "ESS-SYNTH-004",
                &*format!("{CELL}/denied/vault.keys.EditNote/as/actor/vault.keys.Auditor"),
                Some(EDIT)
            ),
            ("ESS-SYNTH-001", &*auditor_read, None),
            (
                "ESS-SYNTH-004",
                &*format!("{CELL}/denied/vault.keys.EditNote/as/actor/vault.keys.Guest"),
                Some(EDIT)
            ),
            (
                "ESS-SYNTH-001",
                &*format!("{CELL}/read/vault.keys.Notes/as/actor/vault.keys.Guest"),
                Some(&*auditor_read)
            ),
            (
                "ESS-SYNTH-004",
                &*format!("{CELL}/command/vault.keys.EditNote/edited/as/actor/vault.keys.Owner"),
                Some(EDIT)
            ),
        ]
    );
}

/// Two ordinary refusals of one command (review 2026-10-19, T5): each cell sending a follow-up
/// outcome is linked to that outcome's own refusal, not to the first refusal of its command.
#[test]
fn a_cell_is_linked_to_its_own_outcome_among_several_refused() {
    let text = MODEL.replace(
        "      - {name: body, type: String}\n    outcomes:\n      - name: edited\n",
        "      - {name: body, type: String}
      - {name: priority, type: Integer}
    outcomes:
      - name: escalated
        when: priority > 5
        updates: vault.keys.Note
        instance: note_id
        sets: {body: input.body}
        emits: [vault.keys.NoteEdited]
        payload:
          vault.keys.NoteEdited: {note_id: input.note_id}
      - name: edited\n",
    );
    assert_ne!(text, MODEL, "the second outcome is declared");
    let path = model("two-outcomes");
    fs::write(&path, &text).unwrap();
    let output = ess(&[
        "specify",
        "validate",
        "--path",
        path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let entries = report["completeness"]["unscenarioed"].as_array().unwrap();
    let cause = |scenario: &str| {
        entries
            .iter()
            .find(|entry| entry["scenario"] == scenario)
            .unwrap_or_else(|| panic!("{scenario} is refused: {entries:#?}"))
            .get("caused_by")
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    for outcome in ["edited", "escalated"] {
        let own = format!("vault.keys.EditNote/outcome/{outcome}");
        assert_eq!(cause(&own), None, "{own} is a cause of its own");
        let cell =
            format!("{CELL}/command/vault.keys.EditNote/{outcome}/as/actor/vault.keys.Owner");
        assert_eq!(cause(&cell), Some(own), "{cell}: {entries:#?}");
    }
}

/// The one-time responses guide shows this model's output, its notes left out.
#[test]
fn the_guide_shows_what_synthesize_prints() {
    let guide = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../website/docs/guides/verify/one-time-responses.md"),
    )
    .unwrap();
    let shown = guide
        .split("```shell-session\n$ ess verify conform synthesize --path system.yaml\n")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .expect("the guide shows the grouped refusals");
    let path = model("guide");
    let output = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        path.to_str().unwrap(),
    ]);
    let printed = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("note: "))
        .fold(String::new(), |mut printed, line| {
            printed.push_str(line);
            printed.push('\n');
            printed
        });
    assert_eq!(shown, printed);
}
