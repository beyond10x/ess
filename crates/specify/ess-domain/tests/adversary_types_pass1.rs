//! Adversary pass 1 on the wave-2 `types` unit: literals against `prefix:` and `Json` in the
//! positions the design page names but the unit's own tests do not reach, and the two field-naming
//! spellings beside `presence:`.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::Presence;

fn spec(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("dial.yaml"), raw)]).map_err(|errors| errors.to_string())
}

/// One entity with a prefixed text field and a `Json` field, and one branch that `sets:` `target`
/// to `value`.
fn sets(target: &str, value: &str) -> String {
    format!(
        "format: ess/15
system: demo
version: v1
domain: demo.dial
types:
  - name: demo.dial.Label
    kind: newtype
    of: String
    prefix: \"L-\"
entities:
  - name: demo.dial.Attempt
    identity:
      name: attempt_id
      type: Uuid
    fields:
      - name: label
        type: demo.dial.Label
      - name: maybe_label
        type: Optional<demo.dial.Label>
      - name: blob
        type: Json
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - name: finish
          from: [Open]
          to: Done
events:
  - name: demo.dial.Opened
    fields:
      - name: attempt_id
        type: Uuid
  - name: demo.dial.Finished
    fields: []
commands:
  - name: demo.dial.Open
    outcomes:
      - name: opened
        creates: demo.dial.Attempt
        instance: attempt_id
        emits: [demo.dial.Opened]
        payload:
          demo.dial.Opened:
            attempt_id: {{generated: true}}
  - name: demo.dial.Finish
    input:
      - name: attempt_id
        type: Uuid
    outcomes:
      - name: finished
        moves: demo.dial.Attempt.finish
        instance: attempt_id
        sets:
          {target}: \"{value}\"
        emits: [demo.dial.Finished]
"
    )
}

#[test]
fn adversary_146_a_sets_literal_is_held_to_the_prefix() {
    spec(&sets("label", "L-ok")).expect("`L-ok` starts with `L-`");
    let error = spec(&sets("label", "ok")).expect_err("no value of `Label` is `ok`");
    assert!(error.contains("prefix"), "{error}");
}

#[test]
fn adversary_146_a_sets_literal_through_an_optional_is_held_to_the_prefix() {
    spec(&sets("maybe_label", "L-ok")).expect("`L-ok` starts with `L-`");
    let error =
        spec(&sets("maybe_label", "ok")).expect_err("no value of `Optional<Label>` is `ok`");
    assert!(error.contains("prefix"), "{error}");
}

#[test]
fn adversary_138_a_sets_literal_for_a_json_field_is_refused() {
    spec(&sets("blob", "x")).expect_err("no literal spells a JSON value");
}

#[test]
fn adversary_138_a_newtype_of_json_is_not_a_map_key() {
    let text = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
  - name: demo.msgs.Envelope
    kind: struct
    fields:
      - {name: keyed, type: \"Map<demo.msgs.Body, String>\"}
";
    spec(text).expect_err("a newtype of Json has no key spelling either");
}

/// `primitive_admission::command_fields` gates a response field's `presence:`; no other test
/// reaches that loop, so deleting it left the unit's suite green.
#[test]
fn adversary_139_a_response_field_presence_is_gated_like_every_other_field() {
    let model = |format: &str, ty: &str| {
        format!(
            "format: {format}
system: demo
version: v1
domain: demo.orders
actors:
  - {{name: demo.orders.Clerk, may: [demo.orders.Place]}}
events:
  - name: demo.orders.Placed
    fields: []
commands:
  - name: demo.orders.Place
    response:
      - {{name: partner_ref, type: {ty}, presence: null_when_absent}}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
"
        )
    };
    spec(&model("ess/15", "Optional<String>")).expect("an Optional response field takes one");
    let error = spec(&model("ess/15", "String")).expect_err("a required response field does not");
    assert!(error.contains("presence"), "{error}");
    let error = spec(&model("ess/14", "Optional<String>")).expect_err("below ess/15 it is refused");
    assert!(error.contains("ess/15"), "{error}");
}

#[test]
fn adversary_142_139_the_nested_wire_name_and_a_presence_policy_are_both_kept() {
    let text = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Receipt
    kind: struct
    fields:
      - name: partner_ref
        type: Optional<String>
        naming: {wire: partnerRef}
        presence: null_when_absent
";
    let spec = spec(text).expect("validates");
    let declared = spec
        .system()
        .types
        .get(&"demo.orders.Receipt".parse().unwrap())
        .expect("declared");
    let ess_domain::types::TypeBody::Struct { fields, .. } = &declared.body else {
        panic!("a struct");
    };
    assert_eq!(fields[0].naming.wire.as_deref(), Some("partnerRef"));
    assert_eq!(fields[0].presence(), Some(Presence::NullWhenAbsent));
}
