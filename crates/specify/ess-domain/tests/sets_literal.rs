//! A `sets:` literal is type-checked against the field it sets.
//!
//! It was not, for two releases. The `sets:` loop skipped every source that was not an
//! `input.<field>` read, with a comment saying a literal "is checked where a payload literal is" —
//! and the function that checks one was reachable from the payload path alone. So `paused: "false"`
//! over a `Boolean` field compiled with no diagnostic, and synthesis then abstained on it without
//! saying so: a branch that claimed to determine a field and proved nothing about it.
//!
//! The rule is the payload's, and it is about the TEXT: a literal may be text, a variant of an
//! enum, `true`/`false` for a `Boolean`, or a whole number for an `Integer`. What it may not be is
//! text that spells none of those — `paused: "perhaps"` — or a target with no literal spelling at
//! all.
//!
//! The first cut of this check refused every non-text primitive, which took 52 lines of an
//! adopter's model from compiling-and-silently-dropped to not compiling: `paused: "false"`,
//! `recording: "true"` and `unread: "0"` each say something true and checkable, and the obvious
//! repair is not available either, because an unquoted `paused: false` is not a `PayloadSource` at
//! all. That last refusal is pinned below so nobody closes this by loosening the deserializer.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("sets.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// One entity holding a field of every representation a literal can meet, and one branch setting
/// `{target}` to the literal `{value}`.
fn model(target: &str, value: &str) -> String {
    format!(
        "format: ess/4
system: demo
version: v1
domain: demo.dial
types:
  - name: demo.dial.Reason
    kind: enum
    variants: [Busy, NoAnswer]
  - name: demo.dial.Label
    kind: newtype
    of: String
  - name: demo.dial.Window
    kind: struct
    fields:
      - name: opens
        type: String
      - name: closes
        type: String
entities:
  - name: demo.dial.Attempt
    identity:
      name: attempt_id
      type: Uuid
    fields:
      - name: paused
        type: Boolean
      - name: tries
        type: Integer
      - name: label
        type: demo.dial.Label
      - name: reason
        type: demo.dial.Reason
      - name: window
        type: demo.dial.Window
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
fn a_literal_the_field_can_hold_still_compiles() {
    // Text, and a newtype over text, are what a literal may be — that reading is unchanged.
    spec(&model("label", "manual")).expect("a String-backed newtype takes a text literal");
    // And a variant of the enum the field carries.
    spec(&model("reason", "Busy")).expect("a declared variant is a literal the field can hold");
}

#[test]
fn the_text_of_a_literal_may_spell_a_boolean_or_a_whole_number() {
    // The adopter's own three spellings, which the first cut of this check refused.
    for (target, value) in [
        ("paused", "false"),
        ("paused", "true"),
        ("tries", "0"),
        ("tries", "12"),
        ("tries", "-3"),
    ] {
        spec(&model(target, value))
            .unwrap_or_else(|error| panic!("`{target}: \"{value}\"` must compile:\n{error}"));
    }
}

#[test]
fn text_that_spells_no_value_of_the_primitive_is_still_refused() {
    // The hole stays closed: what changed is that `"false"` is a well-typed Boolean literal and
    // `"perhaps"` is not, not that a Boolean stopped being checked.
    let error = spec(&model("paused", "perhaps")).expect_err("`perhaps` is not a Boolean");
    for required in [
        "type_mismatch",
        "demo.dial.Attempt.paused",
        "Boolean",
        "perhaps",
    ] {
        assert!(
            error.contains(required),
            "{required:?} missing from:\n{error}"
        );
    }
    assert!(
        error.contains("`true` or `false`"),
        "the hint says what IS accepted:\n{error}"
    );
}

#[test]
fn an_integer_literal_is_canonical_decimal_or_it_is_refused() {
    // Each of these names a number and none is the text a reader sends, so normalising them here
    // would put a second spelling into the model that nothing downstream writes.
    for value in ["007", "+7", " 7", "3.0", "three", "9999999999999999999999"] {
        let error = spec(&model("tries", value))
            .expect_err("only canonical decimal spells an Integer literal");
        assert!(
            error.contains("Integer") && error.contains("type_mismatch"),
            "`{value}`: {error}"
        );
        assert!(
            error.contains("without a sign or leading zeroes"),
            "`{value}`: the hint says what IS accepted:\n{error}"
        );
    }
}

#[test]
fn an_unquoted_boolean_is_not_a_source_at_all() {
    // The repair an author reaches for after the refusal above, and it is not available: a
    // `PayloadSource` is text or a mapping. Pinned so this construct is never closed by loosening
    // the deserializer instead — the admitted spelling is the quoted one, decided in one place.
    let error = spec(&model("paused", "false").replace("\"false\"", "false"))
        .expect_err("an unquoted boolean is not a payload source");
    assert!(
        error.contains("invalid type: boolean"),
        "the reader refuses it before any rule sees it:\n{error}"
    );
}

#[test]
fn every_representation_a_literal_cannot_be_is_refused() {
    for (target, value, expected) in [
        ("paused", "perhaps", "Boolean"),
        ("tries", "three", "Integer"),
        ("window", "anything", "has structure"),
        ("reason", "Engaged", "is not a variant"),
    ] {
        let error = spec(&model(target, value))
            .err()
            .unwrap_or_else(|| panic!("`{target}: {value}` must not compile"));
        assert!(
            error.contains(expected) && error.contains("type_mismatch"),
            "`{target}: {value}` expected {expected:?}:\n{error}"
        );
    }
}

#[test]
fn a_value_outside_the_enum_names_the_variants_it_could_have_been() {
    let error = spec(&model("reason", "Engaged")).expect_err("`Engaged` is not declared");
    assert!(
        error.contains("Busy") && error.contains("NoAnswer"),
        "the hint lists what the field can hold:\n{error}"
    );
}

#[test]
fn the_payload_path_keeps_its_own_wording() {
    // The rule is shared and the sentence says which construct carried the literal, so a reader is
    // sent to the line they wrote rather than to the other one.
    let error = spec(
        "format: ess/4
system: demo
version: v1
domain: demo.dial
entities:
  - name: demo.dial.Attempt
    identity: {name: attempt_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Open], to: Done}
events:
  - name: demo.dial.Opened
    fields:
      - {name: attempt_id, type: Uuid}
  - name: demo.dial.Finished
    fields:
      - {name: at, type: Uuid}
commands:
  - name: demo.dial.Open
    outcomes:
      - name: opened
        creates: demo.dial.Attempt
        instance: attempt_id
        emits: [demo.dial.Opened]
        payload:
          demo.dial.Opened:
            attempt_id: {generated: true}
  - name: demo.dial.Finish
    input:
      - {name: attempt_id, type: Uuid}
    outcomes:
      - name: finished
        moves: demo.dial.Attempt.finish
        instance: attempt_id
        emits: [demo.dial.Finished]
        payload:
          demo.dial.Finished:
            at: \"1\"
",
    )
    .expect_err("a Uuid payload field has no literal spelling");
    assert!(
        error.contains("a literal in a payload is text, `true` or `false`"),
        "the payload sentence is unchanged:\n{error}"
    );
}
