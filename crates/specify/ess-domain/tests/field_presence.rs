//! `presence:` on an `Optional<T>` field (beyond10x/ess#139, ess/15).
//!
//! `null_when_absent` says an absent value is sent as an explicit `null`; `omitted_when_absent` says
//! the key is left out. Refused on a required field, below ess/15, and on every naming that is not a
//! field's.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::{Field, Presence};

fn single(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("orders.yaml"), raw)])
        .map_err(|errors| errors.to_string())
}

const ISSUE_139: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.OrderReceipt
    kind: struct
    fields:
      - name: partner_ref
        type: Optional<String>
        presence: null_when_absent
      - name: discount_code
        type: Optional<String>
        presence: omitted_when_absent
      - name: note
        type: Optional<String>
";

fn receipt(spec: &Specification) -> Vec<Option<Presence>> {
    let declared = spec
        .system()
        .types
        .get(&"demo.orders.OrderReceipt".parse().unwrap())
        .expect("declared");
    ["partner_ref", "discount_code", "note"]
        .iter()
        .map(|name| declared.field(name).expect("a field").presence())
        .collect()
}

#[test]
fn issue_139_an_optional_field_declares_its_presence_policy() {
    let spec = single(ISSUE_139).expect("#139's repro validates");
    assert_eq!(
        receipt(&spec),
        vec![
            Some(Presence::NullWhenAbsent),
            Some(Presence::OmittedWhenAbsent),
            None
        ]
    );
}

#[test]
fn presence_is_written_back_and_absent_presence_keeps_the_bytes() {
    let field: Field =
        ess_yaml::from_str("name: a\ntype: Optional<String>\npresence: null_when_absent\n")
            .expect("reads");
    let written = ess_yaml::to_string(&field).expect("writes");
    assert!(written.contains("presence: null_when_absent"), "{written}");
    let plain: Field = ess_yaml::from_str("name: a\ntype: Optional<String>\n").expect("reads");
    assert_eq!(
        ess_yaml::to_string(&plain).expect("writes"),
        "name: a\ntype: Optional<String>\n"
    );
}

#[test]
fn presence_on_a_required_field_is_refused() {
    let text = ISSUE_139.replacen(
        "type: Optional<String>\n        presence",
        "type: String\n        presence",
        1,
    );
    let error = single(&text).expect_err("a required field is always sent");
    assert!(error.contains("partner_ref"), "{error}");
    assert!(error.contains("presence"), "{error}");
}

#[test]
fn presence_below_ess_15_is_refused_with_the_format_it_needs() {
    let error = single(&ISSUE_139.replace("format: ess/15", "format: ess/14"))
        .expect_err("an older reader would drop the policy");
    assert!(error.contains("ess/15"), "{error}");
}

#[test]
fn an_unknown_presence_policy_is_refused() {
    let error =
        ess_yaml::from_str::<Field>("name: a\ntype: Optional<String>\npresence: sometimes\n")
            .expect_err("refused");
    assert!(error.to_string().contains("sometimes"), "{error}");
}

#[test]
fn presence_is_not_a_key_of_any_other_naming_position() {
    // A command's own `naming:` is the same Naming a field carries; presence is a field's wire
    // property and nothing else can declare one.
    let command = "format: ess/15
system: demo
version: v1
domain: demo.orders
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    naming: {presence: null_when_absent}
    outcomes:
      - name: placed
";
    let error = single(command).expect_err("refused on a command");
    assert!(error.contains("presence"), "{error}");
    let variant = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Kind
    kind: enum
    variants:
      - {name: A, presence: null_when_absent}
";
    let error = single(variant).expect_err("refused on an enum variant");
    assert!(error.contains("presence"), "{error}");
    let error = ess_yaml::from_str::<Field>(
        "name: a\ntype: Optional<String>\nnaming: {presence: null_when_absent}\n",
    )
    .expect_err("presence is a field key, not a naming key");
    assert!(error.to_string().contains("presence"), "{error}");
}

#[test]
fn presence_on_command_input_and_event_fields() {
    let text = "format: ess/15
system: demo
version: v1
domain: demo.orders
events:
  - name: demo.orders.Placed
    fields:
      - {name: note, type: Optional<String>, presence: null_when_absent}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: note, type: Optional<String>, presence: omitted_when_absent}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed: {note: input.note}
";
    let spec = single(text).expect("validates");
    let event = spec
        .events()
        .get(&"demo.orders.Placed".parse().unwrap())
        .unwrap();
    assert_eq!(event.fields[0].presence(), Some(Presence::NullWhenAbsent));
    let command = spec
        .commands()
        .get(&"demo.orders.Place".parse().unwrap())
        .unwrap();
    assert_eq!(
        command.input[0].presence(),
        Some(Presence::OmittedWhenAbsent)
    );
}

/// `null` is a JSON value, so an `Optional<Json>` whose absence is "never sent as null" would refuse
/// one of its own values: refused, through newtypes too. `null_when_absent` stays admitted.
#[test]
fn omitted_when_absent_on_an_optional_json_is_refused() {
    let text = |of: &str, policy: &str| {
        format!(
            "format: ess/15\nsystem: demo\nversion: v1\ndomain: demo.msgs\ntypes:\n  - {{name: demo.msgs.Body, kind: newtype, of: Json}}\n  - name: demo.msgs.Envelope\n    kind: struct\n    fields:\n      - {{name: body, type: \"Optional<{of}>\", presence: {policy}}}\n"
        )
    };
    for of in ["Json", "demo.msgs.Body"] {
        let error = single(&text(of, "omitted_when_absent")).expect_err(of);
        assert!(error.contains("body"), "{error}");
        assert!(error.contains("null"), "{error}");
        assert!(error.contains("omitted_when_absent"), "{error}");
        single(&text(of, "null_when_absent")).unwrap_or_else(|error| panic!("{of}: {error}"));
    }
}

/// `presence: omitted_or_null` (beyond10x/ess#678, `docs/design/omitted-or-null-presence.md`):
/// one model at every position that takes `presence:`.
const EVERY_POSITION: &str = "format: ess/31
system: demo
version: v1
domain: demo.notes
types:
  - name: demo.notes.Edit
    kind: struct
    fields:
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
entities:
  - name: demo.notes.Note
    identity: {name: id, type: Uuid}
    fields:
      - {name: summary, type: Optional<String>, presence: omitted_or_null}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.notes.Edited
    fields:
      - {name: summary, type: Optional<String>, presence: omitted_or_null}
errors:
  - name: demo.notes.Refused
    summary: The edit is refused.
    fields:
      - {name: detail, type: Optional<String>, presence: omitted_or_null}
actors:
  - {name: demo.notes.Clerk, may: [demo.notes.EditNote]}
commands:
  - name: demo.notes.EditNote
    input:
      - {name: summary, type: Optional<String>, presence: omitted_or_null}
      - {name: edit, type: demo.notes.Edit}
    response:
      - {name: summary, type: Optional<String>, presence: omitted_or_null}
    outcomes:
      - name: edited
        emits: [demo.notes.Edited]
        payload:
          demo.notes.Edited: {summary: input.summary}
";

#[test]
fn omitted_or_null_is_admitted_at_every_presence_position_from_ess_31() {
    let spec = single(EVERY_POSITION).unwrap_or_else(|error| panic!("{error}"));
    let either = Some(Presence::OmittedOrNull);
    let edit = spec
        .system()
        .types
        .get(&"demo.notes.Edit".parse().unwrap())
        .unwrap();
    assert_eq!(edit.field("reason").unwrap().presence(), either);
    let note = spec
        .entities()
        .get(&"demo.notes.Note".parse().unwrap())
        .unwrap();
    assert_eq!(
        note.fields
            .iter()
            .find(|field| field.name == "summary")
            .unwrap()
            .presence(),
        either
    );
    let event = spec
        .events()
        .get(&"demo.notes.Edited".parse().unwrap())
        .unwrap();
    assert_eq!(event.fields[0].presence(), either);
    let error = spec
        .errors()
        .get(&"demo.notes.Refused".parse().unwrap())
        .unwrap();
    assert_eq!(error.fields[0].presence(), either);
    let command = spec
        .commands()
        .get(&"demo.notes.EditNote".parse().unwrap())
        .unwrap();
    assert_eq!(command.input[0].presence(), either);
    assert_eq!(command.response[0].presence(), either);
}

#[test]
fn omitted_or_null_below_ess_31_is_refused_naming_ess_31() {
    let error = single(&EVERY_POSITION.replace("format: ess/31", "format: ess/30"))
        .expect_err("an older reader fails the value as an unknown variant");
    assert!(error.contains("unsupported_format_version"), "{error}");
    assert!(error.contains("ess/31"), "{error}");
    assert!(error.contains("omitted_or_null"), "{error}");
    for at in [
        "types.demo.notes.Edit.fields.reason.presence",
        "command.demo.notes.EditNote.input.summary.presence",
        "command.demo.notes.EditNote.response.summary.presence",
    ] {
        assert!(error.contains(at), "{at} in {error}");
    }
    // The two older values keep their own format.
    single(
        &EVERY_POSITION
            .replace("format: ess/31", "format: ess/30")
            .replace("omitted_or_null", "null_when_absent"),
    )
    .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn omitted_or_null_on_a_required_field_is_refused() {
    let text = EVERY_POSITION.replacen(
        "{name: reason, type: Optional<String>, presence: omitted_or_null}",
        "{name: reason, type: String, presence: omitted_or_null}",
        1,
    );
    let error = single(&text).expect_err("a required field is always sent");
    assert!(error.contains("type_mismatch"), "{error}");
    assert!(error.contains("reason"), "{error}");
}

/// `null` spells absence here, as under `null_when_absent`, so an `Optional<Json>` takes it.
#[test]
fn omitted_or_null_on_an_optional_json_is_admitted() {
    let text = "format: ess/31\nsystem: demo\nversion: v1\ndomain: demo.msgs\ntypes:\n  - {name: demo.msgs.Body, kind: newtype, of: Json}\n  - name: demo.msgs.Envelope\n    kind: struct\n    fields:\n      - {name: body, type: \"Optional<demo.msgs.Body>\", presence: omitted_or_null}\n";
    single(text).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn omitted_or_null_is_written_back() {
    let field: Field =
        ess_yaml::from_str("name: a\ntype: Optional<String>\npresence: omitted_or_null\n")
            .expect("reads");
    assert_eq!(field.presence(), Some(Presence::OmittedOrNull));
    let written = ess_yaml::to_string(&field).expect("writes");
    assert!(written.contains("presence: omitted_or_null"), "{written}");
}

/// A view field carries its source field's policy (beyond10x/ess#677) and keeps refusing the key.
#[test]
fn a_view_field_still_refuses_presence() {
    let text = "format: ess/31
system: demo
version: v1
domain: demo.notes
entities:
  - name: demo.notes.Note
    identity: {name: id, type: Uuid}
    fields:
      - {name: summary, type: Optional<String>, presence: null_when_absent}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: id, type: Uuid}
      - {name: summary, type: Optional<String>, presence: null_when_absent}
";
    let error = single(text).expect_err("a view field takes no `presence:`");
    assert!(error.contains("presence"), "{error}");
}
