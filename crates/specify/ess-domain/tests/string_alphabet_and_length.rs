//! `alphabet:` on a `String` newtype, `example:` on a command input, and `.count` on text
//! (beyond10x/ess#103, beyond10x/ess#104), at validation.
//!
//! `docs/design/string-alphabet-and-length.md` is the binding design. Each case is one row of its
//! validation tables, one predicate position of section 3, or one format gate of section 5.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = include_str!("fixtures/text-length-positions.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("keypad.yaml"), raw)])
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("must be admitted: {errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    match assemble(text) {
        Ok(_) => panic!("must refuse:\n{text}"),
        Err(errors) => errors,
    }
}

fn reader_error(text: &str) -> String {
    RawSpecFile::parse(text)
        .err()
        .unwrap_or_else(|| panic!("the reader must refuse:\n{text}"))
        .to_string()
}

/// `true` when one refusal carries `code` at a location holding `site`, whose message holds
/// `says`.
fn has(errors: &ValidationErrors, code: ValidationCode, site: &str, says: &str) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == code && error.location.contains(site) && error.message.contains(says)
    })
}

fn assert_has(errors: &ValidationErrors, code: ValidationCode, site: &str, says: &str) {
    assert!(
        has(errors, code, site, says),
        "expected {code:?} at …{site} saying {says:?}, got:\n{errors}"
    );
}

/// The model with one line replaced, asserting the line was there to replace.
fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the fixture carries {from:?}");
    model.replacen(from, to, 1)
}

fn at(format: &str, model: &str) -> String {
    with(model, "format: ess/11", &format!("format: {format}"))
}

fn alphabet(value: &str) -> String {
    with(MODEL, "    # ALPHABET", &format!("    alphabet: {value}"))
}

fn example(field: &str, value: &str) -> String {
    let (_, commands) = MODEL
        .split_once("  - name: keypad.dial.SendKeys")
        .expect("the fixture declares SendKeys");
    let line = commands
        .lines()
        .find(|line| line.contains(&format!("{{name: {field}, type: ")))
        .unwrap_or_else(|| panic!("the fixture has an input `{field}`"));
    let written = line.replace('}', &format!(", example: {value}}}"));
    with(MODEL, line, &written)
}

// ---- the base, at both formats --------------------------------------------------------------

#[test]
fn the_fixture_is_admitted_at_ess_11_and_at_ess_10() {
    admitted(MODEL);
    admitted(&at("ess/10", MODEL));
}

// ---- alphabet: -------------------------------------------------------------------------------

#[test]
fn an_alphabet_on_a_string_newtype_is_admitted_and_kept_in_order() {
    let spec = admitted(&alphabet(r#""0123456789*#ABCD""#));
    let declared = spec
        .system()
        .types
        .get(&"keypad.dial.KeySequence".parse().unwrap())
        .unwrap();
    let ess_domain::TypeBody::Newtype { alphabet, .. } = &declared.body else {
        panic!("a newtype")
    };
    assert_eq!(alphabet.as_deref(), Some("0123456789*#ABCD"));
}

#[test]
fn an_alphabet_over_a_type_that_is_not_string_is_a_type_mismatch() {
    for (of, name) in [
        ("Uuid", "Uuid"),
        ("Integer", "Integer"),
        ("keypad.dial.Channel", "keypad.dial.Channel"),
    ] {
        let model = with(
            &alphabet(r#""abc""#),
            "    of: String\n    alphabet",
            &format!("    of: {of}\n    alphabet"),
        )
        .replace(
            "    invariants: [value != \"\"]\n  - name: keypad.dial.Short",
            "  - name: keypad.dial.Short",
        );
        let errors = refused(&model);
        assert_has(
            &errors,
            ValidationCode::TypeMismatch,
            "types.keypad.dial.KeySequence.alphabet",
            name,
        );
    }
}

#[test]
fn an_alphabet_through_a_newtype_of_string_is_admitted() {
    admitted(&with(
        MODEL,
        "    # INNER-ALPHABET",
        r#"    alphabet: "123""#,
    ));
}

#[test]
fn an_empty_alphabet_is_an_empty_declaration() {
    let errors = refused(&alphabet(r#""""#));
    assert_has(
        &errors,
        ValidationCode::EmptyDeclaration,
        "types.keypad.dial.KeySequence.alphabet",
        "",
    );
}

#[test]
fn a_character_written_twice_is_a_duplicate_naming_it_and_both_positions() {
    let errors = refused(&alphabet(r#""01#2#""#));
    assert_has(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "types.keypad.dial.KeySequence.alphabet",
        "'#'",
    );
    assert_has(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "types.keypad.dial.KeySequence.alphabet",
        "positions 3 and 5",
    );
}

#[test]
fn nested_alphabets_that_share_no_character_conflict_at_the_outer_type() {
    let model = with(
        &alphabet(r#""0123""#),
        "    # INNER-ALPHABET",
        r#"    alphabet: "ABCD""#,
    );
    let errors = refused(&model);
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "types.keypad.dial.Short.alphabet",
        "keypad.dial.KeySequence",
    );
    admitted(&with(
        &alphabet(r#""0123""#),
        "    # INNER-ALPHABET",
        r#"    alphabet: "3AB""#,
    ));
}

#[test]
fn an_alphabet_below_ess_11_is_refused_with_the_format_it_needs() {
    let errors = refused(&at("ess/10", &alphabet(r#""0123""#)));
    assert_has(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "types.keypad.dial.KeySequence.alphabet",
        "declared alphabets require specification format ess/11",
    );
}

#[test]
fn an_alphabet_that_is_not_a_yaml_string_is_a_reader_error() {
    let error = reader_error(&alphabet("123"));
    assert!(error.contains("string"), "{error}");
}

#[test]
fn an_alphabet_on_a_struct_is_an_unknown_field() {
    let error = reader_error(&with(
        MODEL,
        "    invariants: [text != \"\"]",
        "    invariants: [text != \"\"]\n    alphabet: \"abc\"",
    ));
    assert!(error.contains("unknown field `alphabet`"), "{error}");
}

// ---- example: --------------------------------------------------------------------------------

#[test]
fn an_example_on_a_scalar_input_is_admitted_and_round_trips() {
    for (field, value) in [
        ("keys", r#""12#""#),
        ("short", r#""7""#),
        ("note", r#""hello""#),
        ("channel", "Pulse"),
        ("ref", r#""00000000-0000-4000-8000-000000000001""#),
        ("at", r#""2024-02-03T04:05:06Z""#),
        ("n", "12"),
        ("amount", "3"),
        ("level", "2.5"),
        ("flag", "false"),
    ] {
        let spec = admitted(&example(field, value));
        let command = &spec.commands()[&"keypad.dial.SendKeys".parse().unwrap()];
        assert!(command.examples.contains_key(field), "{field}: {command:?}");
        let written = serde_yaml::to_string(command).unwrap();
        assert!(written.contains("example:"), "{written}");
        let reread: ess_domain::command::RawCommandSpec = serde_yaml::from_str(&written).unwrap();
        let reread = ess_domain::command::CommandSpec::try_from(reread).unwrap();
        assert_eq!(&reread, command, "{field} round-trips");
    }
}

#[test]
fn an_example_on_an_aggregate_input_or_binary64_is_a_type_mismatch() {
    for (field, value) in [
        ("tags", "[a]"),
        ("codes", "{a: b}"),
        ("label", "{text: a}"),
        ("ratio", "1.5"),
    ] {
        let errors = refused(&example(field, value));
        assert_has(
            &errors,
            ValidationCode::TypeMismatch,
            &format!("command.keypad.dial.SendKeys.input.{field}.example"),
            "",
        );
    }
}

#[test]
fn a_null_example_is_refused_as_no_value_even_over_an_optional_input() {
    for field in ["keys", "note"] {
        let errors = refused(&example(field, "null"));
        assert_has(
            &errors,
            ValidationCode::TypeMismatch,
            &format!("command.keypad.dial.SendKeys.input.{field}.example"),
            "so it cannot be an example of one",
        );
    }
}

#[test]
fn an_example_that_is_not_a_value_of_its_type_is_a_type_mismatch() {
    for (field, value) in [
        ("keys", "123"),
        ("keys", "null"),
        ("note", "null"),
        ("channel", "Rotary"),
        ("ref", r#""not-a-uuid""#),
        ("blob", r#""!!""#),
        ("at", r#""yesterday""#),
        ("n", "1.5"),
        ("flag", r#""true""#),
    ] {
        let errors = refused(&example(field, value));
        assert_has(
            &errors,
            ValidationCode::TypeMismatch,
            &format!("command.keypad.dial.SendKeys.input.{field}.example"),
            "",
        );
    }
}

/// Measured, and not what the design page's validation table says: this reader takes an unquoted
/// `0123` as the text "0123" (a leading zero is not a YAML 1.2 number to it), so only a number
/// written without one — `123` — is the wrong kind for a text example.
#[test]
fn an_unquoted_number_with_a_leading_zero_is_read_as_text() {
    let spec = admitted(&example("keys", "0123"));
    let command = &spec.commands()[&"keypad.dial.SendKeys".parse().unwrap()];
    assert_eq!(
        command.examples["keys"],
        ess_primitives::node::Node::Text("0123".to_owned())
    );
}

#[test]
fn an_example_that_breaks_an_alphabet_or_an_invariant_conflicts_naming_the_layer() {
    let model = with(
        &alphabet(r#""0123456789*#ABCD""#),
        "{name: keys, type: keypad.dial.KeySequence}",
        r#"{name: keys, type: keypad.dial.KeySequence, example: "12x"}"#,
    );
    let errors = refused(&model);
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "command.keypad.dial.SendKeys.input.keys.example",
        "keypad.dial.KeySequence",
    );
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "command.keypad.dial.SendKeys.input.keys.example",
        "'x'",
    );

    // The inner layer's invariant, reached through the outer newtype.
    let errors = refused(&example("short", r#""""#));
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "command.keypad.dial.SendKeys.input.short.example",
        "value != \"\"",
    );
    let errors = refused(&example("amount", "-1"));
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "command.keypad.dial.SendKeys.input.amount.example",
        "keypad.dial.Amount",
    );
}

#[test]
fn an_example_whose_type_reads_its_length_is_held_to_the_length() {
    let model = with(
        MODEL,
        "    invariants: [value != \"\"]",
        "    invariants: [value.count <= 3]",
    );
    admitted(&with(
        &model,
        "{name: keys, type: keypad.dial.KeySequence}",
        r#"{name: keys, type: keypad.dial.KeySequence, example: "123"}"#,
    ));
    let errors = refused(&with(
        &model,
        "{name: keys, type: keypad.dial.KeySequence}",
        r#"{name: keys, type: keypad.dial.KeySequence, example: "1234"}"#,
    ));
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "command.keypad.dial.SendKeys.input.keys.example",
        "value.count <= 3",
    );
}

#[test]
fn an_example_below_ess_11_is_refused_with_the_format_it_needs() {
    let errors = refused(&at("ess/10", &example("keys", r#""12#""#)));
    assert_has(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "command.keypad.dial.SendKeys.input.keys.example",
        "input examples require specification format ess/11",
    );
}

#[test]
fn an_example_anywhere_but_a_command_input_is_an_unknown_field() {
    for (from, to) in [
        (
            "      - {name: session_id, type: Uuid}\n  - name: keypad.dial.KeysSent",
            "      - {name: session_id, type: Uuid, example: \"x\"}\n  - name: keypad.dial.KeysSent",
        ),
        (
            "      - {name: name, type: String}\n    invariants",
            "      - {name: name, type: String, example: x}\n    invariants",
        ),
        (
            "      - {name: text, type: String}",
            "      - {name: text, type: String, example: x}",
        ),
        (
            "      - {name: name, type: String}\n",
            "      - {name: name, type: String, example: x}\n",
        ),
    ] {
        let error = reader_error(&with(MODEL, from, to));
        assert!(error.contains("unknown field `example`"), "{from}: {error}");
    }
}

#[test]
fn the_published_input_field_is_a_field_plus_exactly_example() {
    let field = serde_json::to_value(schemars::schema_for!(ess_domain::Field)).unwrap();
    let input =
        serde_json::to_value(schemars::schema_for!(ess_domain::command::InputField)).unwrap();
    let properties = |schema: &serde_json::Value| -> std::collections::BTreeSet<String> {
        schema["properties"]
            .as_object()
            .expect("an object schema")
            .keys()
            .cloned()
            .collect()
    };
    let mut expected = properties(&field);
    expected.insert("example".to_owned());
    assert_eq!(properties(&input), expected);
    let strip = |mut schema: serde_json::Value| {
        let object = schema.as_object_mut().unwrap();
        object.remove("title");
        object.remove("description");
        object["properties"]
            .as_object_mut()
            .unwrap()
            .remove("example");
        // The one definition `example` itself brings in.
        if let Some(definitions) = object
            .get_mut("definitions")
            .and_then(serde_json::Value::as_object_mut)
        {
            definitions.remove("Node");
        }
        schema
    };
    assert_eq!(strip(input), strip(field), "every other key is the same");
}

// ---- .count on text --------------------------------------------------------------------------

/// Each predicate position the checker serves, as `(from, to, owner)`: the base line and the one
/// that reads a text's length there.
const POSITIONS: &[(&str, &str, &str)] = &[
    (
        "        when: n > 64",
        "        when: keys.count > 64",
        "SendKeys.outcomes.too-long",
    ),
    (
        "        when: n > 64",
        "        when: short.count > 64",
        "SendKeys.outcomes.too-long",
    ),
    (
        "        when: n > 64",
        "        when: note.count > 64",
        "SendKeys.outcomes.too-long",
    ),
    (
        "        when: n > 64",
        "        when: label.text.count > 64",
        "SendKeys.outcomes.too-long",
    ),
    (
        "        when: n > 64",
        "        when: {forall: {in: tags, as: t, that: t.count <= 8}}",
        "SendKeys.outcomes.too-long",
    ),
    (
        "          predicate: name == \"anonymous\"",
        "          predicate: name.count > 64",
        "Close.outcomes.unnamed",
    ),
    (
        "    invariants: [value != \"\"]",
        "    invariants: [value.count <= 64]",
        "KeySequence.invariants",
    ),
    (
        "    invariants: [text != \"\"]",
        "    invariants: [text.count >= 1]",
        "Label.invariants",
    ),
    (
        "    invariants: [name != \"\"]",
        "    invariants: [name.count >= 1]",
        "Session.invariants",
    ),
    (
        "    filter: name != \"\"",
        "    filter: name.count > 0",
        "Sessions.filter",
    ),
];

#[test]
fn a_text_length_is_admitted_in_every_predicate_position_at_ess_11() {
    for (from, to, _) in POSITIONS {
        admitted(&with(MODEL, from, to));
    }
}

#[test]
fn a_text_length_below_ess_11_is_refused_in_every_position_with_the_format_it_needs() {
    for (from, to, owner) in POSITIONS {
        let errors = refused(&at("ess/10", &with(MODEL, from, to)));
        assert!(
            errors.as_slice().iter().any(|error| error.code
                == ValidationCode::UnsupportedFormatVersion
                && error.location.contains(owner)
                && error
                    .message
                    .contains("the length of a String requires specification format ess/11")),
            "{to} at ess/10: {errors}"
        );
    }
}

#[test]
fn a_count_on_a_text_that_is_not_string_is_still_unobservable() {
    for field in ["ref", "at", "wait", "blob", "channel", "n", "level", "flag"] {
        let errors = refused(&with(
            MODEL,
            "        when: n > 64",
            &format!("        when: {field}.count > 1"),
        ));
        assert_has(
            &errors,
            ValidationCode::UnobservableFact,
            "SendKeys.outcomes.too-long",
            "cannot select `count`",
        );
    }
}

#[test]
fn a_selector_past_a_text_length_is_unobservable_and_says_it_is_a_length() {
    let errors = refused(&with(
        MODEL,
        "        when: n > 64",
        "        when: keys.count.more > 64",
    ));
    assert_has(
        &errors,
        ValidationCode::UnobservableFact,
        "SendKeys.outcomes.too-long",
        "`keys.count` is a text length of type Integer (Number); `more` selects nothing from it",
    );
}

#[test]
fn a_text_length_is_a_number_to_the_type_checker() {
    let errors = refused(&with(
        MODEL,
        "        when: n > 64",
        "        when: keys.count == \"64\"",
    ));
    assert_has(
        &errors,
        ValidationCode::TypeMismatch,
        "SendKeys.outcomes.too-long",
        "",
    );
}
