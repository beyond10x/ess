//! Adversary pass 1 for `trimmed:` text guards (beyond10x/ess#532, ess/25,
//! `docs/design/value-expressions.md` § E10): the set it trims, the texts at its boundaries, and the
//! sites that must refuse it by name.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

fn predicate(yaml: &str) -> Predicate {
    let node: Node = ess_yaml::from_str(yaml).unwrap_or_else(|error| panic!("{yaml}: {error}"));
    Predicate::from_node(&node).unwrap_or_else(|error| panic!("{yaml}: {error}"))
}

fn at(predicate: &Predicate, value: &str) -> Truth {
    let mut facts = FactStore::new();
    facts.set(FactPath::new("code").unwrap(), FactValue::text(value));
    predicate.evaluate(&facts)
}

/// Unicode whitespace outside the six ASCII characters is compared as itself, at either end, for
/// every wrapped operator: U+00A0, U+0085 NEL, U+2028, U+2029, U+3000, U+FEFF, U+1680, U+202F.
#[test]
fn adv_m3_unicode_whitespace_is_never_trimmed() {
    let unicode = [
        '\u{a0}', '\u{85}', '\u{2028}', '\u{2029}', '\u{3000}', '\u{feff}', '\u{1680}', '\u{202f}',
    ];
    let operators = [
        "code: {trimmed: {equals: DONE}}",
        "code: {trimmed: {in: [DONE]}}",
        "code: {trimmed: {equals_ignore_case: done}}",
        "code: {trimmed: {in_ignore_case: [done]}}",
        "code: {trimmed: {starts_with: DONE}}",
        "code: {trimmed: {ends_with: DONE}}",
    ];
    for operator in operators {
        let predicate = predicate(operator);
        for space in unicode {
            for value in [
                format!("{space}DONE"),
                format!("DONE{space}"),
                format!(" {space}DONE{space} "),
            ] {
                let starts = operator.contains("starts_with")
                    && value.trim_start_matches(' ').starts_with("DONE");
                let ends =
                    operator.contains("ends_with") && value.trim_end_matches(' ').ends_with("DONE");
                let expected = if starts || ends {
                    Truth::True
                } else {
                    Truth::False
                };
                assert_eq!(
                    at(&predicate, &value),
                    expected,
                    "{operator} over {value:?} (U+{:04X})",
                    space as u32
                );
            }
        }
    }
}

/// Whitespace inside the text is kept, and whitespace inside a literal is compared as written.
#[test]
fn adv_m3_inner_whitespace_is_kept() {
    for (operator, value, truth) in [
        (
            "code: {trimmed: {equals: 'DO NE'}}",
            "  DO NE\t",
            Truth::True,
        ),
        ("code: {trimmed: {equals: 'DO NE'}}", "DO  NE", Truth::False),
        ("code: {trimmed: {equals: DONE}}", "DO\tNE", Truth::False),
        ("code: {trimmed: {contains: 'O N'}}", " DO NE ", Truth::True),
        (
            "code: {trimmed: {starts_with: 'D O'}}",
            "\tD O",
            Truth::True,
        ),
        (
            "code: {trimmed: {equals_ignore_case: 'do ne'}}",
            "\nDO NE\n",
            Truth::True,
        ),
        (
            "code: {trimmed: {in_ignore_case: ['do ne']}}",
            "\nDONE\n",
            Truth::False,
        ),
    ] {
        assert_eq!(
            at(&predicate(operator), value),
            truth,
            "{operator} over {value:?}"
        );
    }
}

/// A text of ASCII whitespace alone trims to the empty text: every operator answers as it answers
/// the empty text, and the empty text is not a literal space.
#[test]
fn adv_m3_whitespace_only_text_trims_to_empty() {
    for value in ["", " ", "\t", " \t\n\u{b}\u{c}\r ", "\r\n\r\n"] {
        for (operator, truth) in [
            ("code: {trimmed: {equals: ''}}", Truth::True),
            ("code: {trimmed: {in: ['', x]}}", Truth::True),
            ("code: {trimmed: {equals_ignore_case: ''}}", Truth::True),
            ("code: {trimmed: {in_ignore_case: ['']}}", Truth::True),
            ("code: {trimmed: {starts_with: ''}}", Truth::True),
            ("code: {trimmed: {ends_with: ''}}", Truth::True),
            ("code: {trimmed: {contains: ''}}", Truth::True),
            ("code: {trimmed: {equals: ' '}}", Truth::False),
            ("code: {trimmed: {starts_with: ' '}}", Truth::False),
            ("code: {trimmed: {contains: ' '}}", Truth::False),
            ("code: {trimmed: {equals_ignore_case: ' '}}", Truth::False),
        ] {
            assert_eq!(
                at(&predicate(operator), value),
                truth,
                "{operator} over {value:?}"
            );
        }
    }
}

/// Only the fact is trimmed: an input on the other side is compared as sent, and a trimmed operator
/// under `not` and inside a quantifier reads the bound element trimmed.
#[test]
fn adv_m3_only_the_fact_is_trimmed_and_quantifiers_trim_the_element() {
    let mut facts = FactStore::new();
    facts.set(FactPath::new("code").unwrap(), FactValue::text(" SP1 "));
    facts.set(
        FactPath::new("input.prefix").unwrap(),
        FactValue::text("SP"),
    );
    assert_eq!(
        predicate("code: {trimmed: {starts_with: {input: prefix}}}").evaluate(&facts),
        Truth::True
    );
    let mut padded = FactStore::new();
    padded.set(FactPath::new("code").unwrap(), FactValue::text("SP1"));
    padded.set(
        FactPath::new("input.prefix").unwrap(),
        FactValue::text(" SP"),
    );
    assert_eq!(
        predicate("code: {trimmed: {starts_with: {input: prefix}}}").evaluate(&padded),
        Truth::False
    );

    // Round trip of a trimmed operator inside a quantifier and under `not`.
    for yaml in [
        "{forall: {in: codes, as: c, that: {c: {trimmed: {equals: DONE}}}}}",
        "{exists: {in: codes, as: c, that: {not: {c: {trimmed: {in_ignore_case: [a, b]}}}}}}",
        "{forall: {in: codes, as: c, that: {c: {trimmed: {starts_with: {input: prefix}}}}}}",
    ] {
        let read = predicate(yaml);
        assert!(read.uses_trim(), "{yaml}");
        let written = read.to_node();
        assert_eq!(
            Predicate::from_node(&written).expect("reads back"),
            read,
            "{yaml} written as {written:?}"
        );
    }
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("probe.yaml"), raw)])
}

fn refused(text: &str) -> Vec<ValidationError> {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
        .as_slice()
        .to_vec()
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

const EVERY: &str =
    include_str!("../../../../edge/ess-cli/tests/fixtures/parse-refusals/every-position.yaml");

fn every_at_25() -> String {
    let text = replaced(EVERY, "format: ess/22\n", "format: ess/25\n");
    // The created event carries the label as text, for a binding's `where:` to read.
    let text = replaced(
        &text,
        "      - {name: kind, type: probe.item.Kind}\n",
        "      - {name: kind, type: probe.item.Kind}\n      - {name: note, type: String}\n",
    );
    replaced(
        &text,
        "kind: input.kind}",
        "kind: input.kind, note: input.label}",
    )
}

/// A binding's `where:` is not a command guard (E10, "Where"): `trimmed` there is refused as
/// `unsupported_construct`, by name, and nothing else is reported.
#[test]
fn adv_m3_trimmed_in_a_binding_condition_is_refused_by_name() {
    let base = every_at_25();
    assemble(&base).unwrap_or_else(|errors| panic!("the control validates: {errors}"));
    let text = replaced(
        &base,
        "where: event.kind == fancy",
        "where: {event.note: {trimmed: {equals: fancy}}}",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedConstruct
                && error.message.contains("trimmed")),
        "{errors:#?}"
    );
}

/// A view filter over the stored row: refused by name, once.
#[test]
fn adv_m3_trimmed_in_a_view_filter_is_refused_once() {
    let text = replaced(
        &every_at_25(),
        "filter: label != \"\"",
        "filter: {label: {trimmed: {equals: x}}}",
    );
    let errors = refused(&text);
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.message.contains("trimmed"))
            .count(),
        1,
        "{errors:#?}"
    );
    assert_eq!(
        errors[0].code,
        ValidationCode::UnsupportedConstruct,
        "{errors:#?}"
    );
}

/// An unknown fact under `trimmed` is reported once, not once by the wrapper and again by the
/// operator it wraps.
#[test]
fn adv_m3_an_unknown_fact_under_trimmed_is_reported_once() {
    let text = replaced(
        &every_at_25(),
        "when: input.label == \"\"",
        "when: {input.nosuch: {trimmed: {equals: x}}}",
    );
    let errors = refused(&text);
    assert_eq!(errors.len(), 1, "{errors:#?}");
}

/// Over a fact that is not text, once, as `type_mismatch`: an enum, an identity and a list.
#[test]
fn adv_m3_trimmed_over_non_text_is_one_type_mismatch() {
    for fact in ["input.kind", "input.owner"] {
        let text = replaced(
            &every_at_25(),
            "when: input.label == \"\"",
            &format!("when: {{{fact}: {{trimmed: {{equals: x}}}}}}"),
        );
        let errors = refused(&text);
        assert_eq!(errors.len(), 1, "{fact}: {errors:#?}");
        assert_eq!(
            errors[0].code,
            ValidationCode::TypeMismatch,
            "{fact}: {errors:#?}"
        );
    }
}

/// Below ess/25 a document using `trimmed` once is refused once, naming ess/25, whatever else the
/// operator would be refused for at ess/25: the format answers first, with no cascade.
#[test]
fn adv_m3_ess24_refuses_trimmed_once_naming_ess25() {
    for guard in [
        "when: {input.label: {trimmed: {equals: x}}}",
        "when: {input.kind: {trimmed: {equals: x}}}",
    ] {
        let text = replaced(&every_at_25(), "when: input.label == \"\"", guard);
        let text = replaced(&text, "format: ess/25\n", "format: ess/24\n");
        let errors = refused(&text);
        assert_eq!(errors.len(), 1, "{guard}: {errors:#?}");
        assert_eq!(
            errors[0].code,
            ValidationCode::UnsupportedFormatVersion,
            "{guard}: {errors:#?}"
        );
        assert!(errors[0].message.contains("ess/25"), "{guard}: {errors:#?}");
    }
}

/// The same question of the sibling operator below its format (`equals_ignore_case`, ess/15), to
/// tell whether a format refusal followed by a type refusal is this unit's or the gate's shape.
#[test]
fn adv_m3_control_case_fold_below_ess15_over_an_enum() {
    let text = replaced(
        &every_at_25(),
        "when: input.label == \"\"",
        "when: {input.kind: {equals_ignore_case: x}}",
    );
    let text = replaced(&text, "format: ess/25\n", "format: ess/14\n");
    let errors = refused(&text);
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == ValidationCode::TypeMismatch)
            .count(),
        0,
        "{errors:#?}"
    );
}
