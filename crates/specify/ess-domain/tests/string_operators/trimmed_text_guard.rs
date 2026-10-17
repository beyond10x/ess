//! A text guard compares a `String` fact with its surrounding ASCII whitespace removed (ess/25,
//! beyond10x/ess#532, `docs/design/value-expressions.md` § E10).
//!
//! The issue's refusal: `not: {reason: {trimmed: {in_ignore_case: [NEW, HOLD, DONE]}}}` refuses
//! every reason the implementation does not accept once it trims and upper-cases it. `trimmed:` is
//! admitted from `ess/25`, in a command outcome's guard only, over a `String` fact or a newtype of
//! one, and wraps exactly one text operator.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

const MODEL: &str = r"format: ess/25
system: promo
version: v1
domain: promo.codes
types:
  - name: promo.codes.Reason
    kind: newtype
    of: String
entities:
  - name: promo.codes.Code
    identity: {name: code_id, type: Uuid}
    fields:
      - {name: reason, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
errors:
  - name: promo.codes.UnknownReason
    summary: The reason is not a known reason.
    fields: []
events:
  - name: promo.codes.Closed
    fields:
      - {name: reason, type: promo.codes.Reason}
commands:
  - name: promo.codes.Close
    input:
      - {name: reason, type: promo.codes.Reason}
      - {name: code, type: Optional<String>}
      - {name: count, type: Integer}
    outcomes:
      - name: unknown-reason
        when: {not: {reason: {trimmed: {in_ignore_case: [NEW, HOLD, DONE]}}}}
        error: promo.codes.UnknownReason
      - name: spring
        when: {code: {trimmed: {starts_with: SPRING}}}
        emits: [promo.codes.Closed]
        payload:
          promo.codes.Closed:
            reason: input.reason
      - name: closed
        emits: [promo.codes.Closed]
        payload:
          promo.codes.Closed:
            reason: input.reason
";

const GUARD: &str = "when: {not: {reason: {trimmed: {in_ignore_case: [NEW, HOLD, DONE]}}}}";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("promo.yaml"), raw)])
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

#[test]
fn trimmed_guard_validates_at_ess_25_and_is_refused_below_it() {
    let spec = assemble(MODEL).unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"promo.codes.Close".parse().unwrap()];
    assert_eq!(command.outcomes.len(), 3);

    // Every wrapped operator validates over a String fact.
    for operator in [
        "{equals: DONE}",
        "{eq: DONE}",
        "{in: [NEW, DONE]}",
        "{any_of: [NEW]}",
        "{equals_ignore_case: done}",
        "{in_ignore_case: [new, done]}",
        "{starts_with: D}",
        "{ends_with: E}",
        "{contains: ON}",
    ] {
        let text = replaced(
            MODEL,
            GUARD,
            &format!("when: {{not: {{reason: {{trimmed: {operator}}}}}}}"),
        );
        assemble(&text).unwrap_or_else(|errors| panic!("{operator}: {errors}"));
    }

    // Below ess/25 each `trimmed:` is refused by format, at its own site, naming ess/25.
    for format in ["ess/24", "ess/15"] {
        let errors = refused(&replaced(
            MODEL,
            "format: ess/25\n",
            &format!("format: {format}\n"),
        ));
        let gated: Vec<&ValidationError> = errors
            .iter()
            .filter(|error| error.code == ValidationCode::UnsupportedFormatVersion)
            .collect();
        assert_eq!(gated.len(), 2, "{format}: {errors:#?}");
        for error in gated {
            assert!(
                error.message.contains("`trimmed`") && error.message.contains("ess/25"),
                "{format}: {error:#?}"
            );
        }
        assert!(
            errors
                .iter()
                .all(|error| error.code == ValidationCode::UnsupportedFormatVersion),
            "{format}: nothing but the format refuses it: {errors:#?}"
        );
    }

    // Over a fact that is not text: type_mismatch.
    let errors = refused(&replaced(
        MODEL,
        "{code: {trimmed: {starts_with: SPRING}}}",
        "{count: {trimmed: {equals: '3'}}}",
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch
                && error.message.contains("trimmed")),
        "{errors:#?}"
    );

    // A literal that is not text: type_mismatch, with the quoting hint.
    let errors = refused(&replaced(
        MODEL,
        "{code: {trimmed: {starts_with: SPRING}}}",
        "{code: {trimmed: {equals: 44}}}",
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch
                && error.message.contains("quote the literal")),
        "{errors:#?}"
    );

    // Wrapping an operator that is not a text operator is refused by the reader.
    for operator in ["{gt: 3}", "{ne: DONE}", "{none_of: [DONE]}", "DONE"] {
        let errors = refused(&replaced(
            MODEL,
            GUARD,
            &format!("when: {{reason: {{trimmed: {operator}}}}}"),
        ));
        assert!(
            errors.iter().any(|error| error
                .message
                .contains("`trimmed` takes a mapping of exactly one")),
            "{operator}: {errors:#?}"
        );
    }
}

#[test]
fn trimmed_is_refused_by_name_outside_a_command_guard() {
    let filtered = replaced(
        MODEL,
        "commands:\n",
        "views:
  - name: promo.codes.Closes
    source: promo.codes.Code
    consistency: eventual
    filter: {reason: {trimmed: {equals: DONE}}}
    fields:
      - {name: code_id, type: Uuid}
commands:\n",
    );
    let errors = refused(&filtered);
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("`trimmed` is admitted only in a command outcome's guard")),
        "{errors:#?}"
    );
    let invariant = replaced(
        MODEL,
        "      - {name: reason, type: String}\n    lifecycle:",
        "      - {name: reason, type: String}\n    invariants:\n      - not: {reason: {trimmed: {equals: \"\"}}}\n    lifecycle:",
    );
    let errors = refused(&invariant);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedConstruct
                && error
                    .message
                    .contains("`trimmed` is admitted only in a command outcome's guard")),
        "{errors:#?}"
    );
}

const EVERY: &str =
    include_str!("../../../../edge/ess-cli/tests/fixtures/parse-refusals/every-position.yaml");

/// The guards over a stored row admit it too: `when_subject:` over the subject's fields and
/// `when_related:` over the related row's (E10, "Where").
#[test]
fn trimmed_is_admitted_over_a_stored_row() {
    let at_25 = replaced(EVERY, "format: ess/22\n", "format: ess/25\n");
    for (from, to) in [
        (
            "when_subject: {predicate: label == \"\"}",
            "when_subject: {predicate: {label: {trimmed: {equals: \"\"}}}}",
        ),
        (
            "when_related: {via: input.owner, predicate: label == \"\"}",
            "when_related: {via: input.owner, predicate: {label: {trimmed: {in_ignore_case: [x, y]}}}}",
        ),
    ] {
        let text = replaced(&at_25, from, to);
        assemble(&text).unwrap_or_else(|errors| panic!("{to}: {errors}"));
        let errors = refused(&replaced(&text, "format: ess/25\n", "format: ess/24\n"));
        assert!(
            errors.iter().any(|error| error.code == ValidationCode::UnsupportedFormatVersion
                && error.message.contains("`trimmed`")),
            "{to}: {errors:#?}"
        );
    }
}
