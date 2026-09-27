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
        serde_yaml::from_str("name: a\ntype: Optional<String>\npresence: null_when_absent\n")
            .expect("reads");
    let written = serde_yaml::to_string(&field).expect("writes");
    assert!(written.contains("presence: null_when_absent"), "{written}");
    let plain: Field = serde_yaml::from_str("name: a\ntype: Optional<String>\n").expect("reads");
    assert_eq!(
        serde_yaml::to_string(&plain).expect("writes"),
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
        serde_yaml::from_str::<Field>("name: a\ntype: Optional<String>\npresence: sometimes\n")
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
    let error = serde_yaml::from_str::<Field>(
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
