//! Adversarial cases for `starts_with`, `ends_with` and `contains` (beyond10x/ess#95) at
//! validation, pass 1. `docs/design/string-predicate-operators.md` is the binding design.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn admits(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("strings.yaml"), raw)])
}

fn selection_guarded(guard: &str) -> String {
    include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml")
        .replace("format: ess/3", "format: ess/8")
        .replace(
            "          where: 'item.id != \"\"'",
            &format!("          where: {guard}"),
        )
}

/// *Selection*: "a text literal, not empty, at most `MAX_TEXT_BYTES` = 4096 bytes". The existing
/// case refuses 4097 ASCII bytes and admits short literals, so it cannot tell `<=` from `<`, nor
/// bytes from characters. The bound itself is admitted, and it is counted in bytes.
#[test]
fn a_selection_literal_of_exactly_the_byte_bound_is_admitted_and_one_byte_more_is_not() {
    let at_bound = "x".repeat(4096);
    if let Err(errors) = admits(&selection_guarded(&format!(
        "{{item.id: {{starts_with: \"{at_bound}\"}}}}"
    ))) {
        panic!("4096 bytes is the bound, not past it: {errors}");
    }
    // 1365 three-byte characters are 4095 bytes; 1366 are 4098 bytes but still fewer than 4096
    // characters.
    let under = "\u{20ac}".repeat(1365);
    if let Err(errors) = admits(&selection_guarded(&format!(
        "{{item.id: {{ends_with: \"{under}\"}}}}"
    ))) {
        panic!("4095 bytes: {errors}");
    }
    let over = "\u{20ac}".repeat(1366);
    assert!(
        admits(&selection_guarded(&format!(
            "{{item.id: {{ends_with: \"{over}\"}}}}"
        )))
        .is_err(),
        "4098 bytes in 1366 characters is past the byte bound"
    );
}

/// The number-operand refusal tells the author how to repair `{starts_with: +44}`. YAML has already
/// read `+44` as the number 44, and the message builds its suggested spelling from that number,
/// so the repair it prints — `{starts_with: "44"}` — tests for a different prefix than the one the
/// author wrote (the design's motivating `+44` caller prefix).
#[test]
fn the_quote_it_hint_does_not_suggest_a_literal_that_drops_the_sign_the_author_wrote() {
    let model = r"format: ess/8
system: calls
version: v1
domain: calls.core
events:
  - name: calls.core.Routed
    fields: []
errors:
  - name: calls.core.Refused
    fields: []
commands:
  - name: calls.core.Route
    input:
      - {name: caller, type: String}
    outcomes:
      - name: uk
        when: {caller: {starts_with: +44}}
        emits: [calls.core.Routed]
      - name: other
        error: calls.core.Refused
";
    let errors = admits(model).expect_err("a number operand is refused");
    let error = errors
        .as_slice()
        .iter()
        .find(|error| error.code == ValidationCode::TypeMismatch)
        .unwrap_or_else(|| panic!("{errors}"));
    assert!(
        !error.message.contains(r#"{starts_with: "44"}"#),
        "the suggested repair drops the `+`: {}",
        error.message
    );
}
