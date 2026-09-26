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
//! repair was not available either, because an unquoted `paused: false` was not a `PayloadSource`
//! at all. Since beyond10x/ess#113 it is: the reader keeps the YAML type and the same rule types
//! it, admitting exactly what the quoted form is admitted as and naming the quotes as the repair
//! where the target is text.

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

/// [`model`], with the literal written unquoted: a YAML boolean, integer or decimal.
fn unquoted(target: &str, value: &str) -> String {
    model(target, value).replace(
        &format!("{target}: \"{value}\""),
        &format!("{target}: {value}"),
    )
}

#[test]
fn an_unquoted_boolean_or_integer_sets_the_field_of_its_own_type() {
    // beyond10x/ess#113: `items: 0` was refused by the reader while `items: '0'` compiled. The
    // domain keeps the YAML type so the rule can check it; the compiler test in `ess-compiler`
    // (`typed_literals.rs`) holds the IR to the quoted form's bytes.
    for (target, value) in [
        ("paused", "false"),
        ("paused", "true"),
        ("tries", "0"),
        ("tries", "12"),
        ("tries", "-3"),
    ] {
        spec(&unquoted(target, value))
            .unwrap_or_else(|error| panic!("`{target}: {value}` must compile:\n{error}"));
    }
}

#[test]
fn an_unquoted_scalar_over_text_is_refused_with_the_quoted_spelling() {
    // A text target takes the quoted form, so the repair is exactly one pair of quotes, and the
    // refusal says so rather than naming a type the author did not write.
    for (target, value) in [("label", "0"), ("label", "true"), ("label", "1.5")] {
        let error = spec(&unquoted(target, value))
            .expect_err("an unquoted scalar over text is not the text it spells");
        assert!(
            error.contains("type_mismatch"),
            "`{target}: {value}`:\n{error}"
        );
        assert!(
            error.contains(&format!("quote it: `{target}: '{value}'`")),
            "`{target}: {value}`: the hint spells the quoted form:\n{error}"
        );
    }
}

#[test]
fn an_unquoted_scalar_of_the_wrong_primitive_gets_the_quoted_forms_refusal() {
    // The quoted form is refused here too, so quoting is no repair and the hint is the one the
    // quoted form already carries: what the field DOES accept.
    for (target, value, accepted) in [
        ("paused", "1", "`true` or `false`"),
        ("paused", "0.5", "`true` or `false`"),
        ("tries", "true", "without a sign or leading zeroes"),
        ("tries", "1.5", "without a sign or leading zeroes"),
        ("tries", "3.0", "without a sign or leading zeroes"),
        ("reason", "false", "is not a variant"),
        (
            "tries",
            "9999999999999999999",
            "without a sign or leading zeroes",
        ),
    ] {
        let error = spec(&unquoted(target, value))
            .expect_err("a scalar of another type is not a value of this one");
        assert!(
            error.contains("type_mismatch") && error.contains(accepted),
            "`{target}: {value}` expected {accepted:?}:\n{error}"
        );
        assert!(
            !error.contains("quote it"),
            "`{target}: {value}`: quoting would not help, so it is not offered:\n{error}"
        );
    }
    // A structured target has no literal at all, quoted or not.
    let error = spec(&unquoted("window", "0")).expect_err("a struct has no literal");
    assert!(error.contains("has structure"), "{error}");
}

#[test]
fn a_payload_literal_is_read_by_the_same_rule() {
    // One reader serves `sets:` and `payload:`, and one rule types both.
    let payload = |value: &str| {
        model("label", "manual")
            .replace(
                "  - name: demo.dial.Finished\n    fields: []",
                "  - name: demo.dial.Finished\n    fields:\n      - {name: attempts, type: Integer}\n      - {name: note, type: String}",
            )
            .replace(
                "        emits: [demo.dial.Finished]\n",
                &format!(
                    "        emits: [demo.dial.Finished]\n        payload:\n          demo.dial.Finished:\n            attempts: {value}\n            note: \"n\"\n"
                ),
            )
    };
    spec(&payload("3")).expect("`attempts: 3` over an Integer payload field compiles");
    let error = spec(&payload("true")).expect_err("a boolean is not an Integer");
    assert!(
        error.contains("type_mismatch") && error.contains("without a sign or leading zeroes"),
        "{error}"
    );
    let text = payload("3")
        .replace("attempts: 3", "attempts: '3'")
        .replace("note: \"n\"", "note: 0");
    let error = spec(&text).expect_err("an integer is not text");
    assert!(error.contains("quote it: `note: '0'`"), "{error}");
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
fn an_unquoted_boolean_is_typed_by_the_rule_and_not_admitted_by_the_reader() {
    // This case used to pin the reader's refusal of `paused: false` (`invalid type: boolean`), so
    // the construct could not be closed by loosening the deserializer. beyond10x/ess#113 reversed
    // the reading on purpose; what the pin protected still holds: the reader admits the scalar
    // and the ONE rule that types a literal decides it, so the loosened reader lets nothing
    // through that the quoted form would not.
    spec(&unquoted("paused", "false")).expect("an unquoted boolean over a Boolean compiles");
    let error = spec(&unquoted("tries", "false")).expect_err("a boolean is not an Integer");
    assert!(
        !error.contains("invalid type"),
        "the reader no longer decides it:\n{error}"
    );
    assert!(
        error.contains("type_mismatch") && error.contains("without a sign or leading zeroes"),
        "the rule decides it, as it decides `tries: \"false\"`:\n{error}"
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
