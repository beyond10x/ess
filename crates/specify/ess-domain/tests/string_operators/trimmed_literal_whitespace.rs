//! A `trimmed:` literal with ASCII whitespace at an end it is compared at holds for no text
//! (beyond10x/ess#532, `docs/design/value-expressions.md` § E10).
//!
//! `trimmed:` removes space, tab, LF, VT, FF and CR at either end of the fact before the wrapped
//! operator compares it, so a trimmed text never starts or ends with one of them. An `equals` or
//! `in` literal (case-folded or not) with such a character at either end, a `starts_with` literal
//! with one at its start and an `ends_with` literal with one at its end can never hold: each is
//! refused as `empty_declaration`, as an `in` listing no literal is. `contains`, a `starts_with`
//! literal ending in whitespace, an `ends_with` literal starting with it, inner whitespace and
//! Unicode spaces can all hold, and are admitted.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = r"format: ess/25
system: promo
version: v1
domain: promo.codes
errors:
  - name: promo.codes.Unknown
    summary: The code is not known.
    fields: []
events:
  - name: promo.codes.Matched
    fields:
      - {name: n, type: Integer}
commands:
  - name: promo.codes.Close
    input:
      - {name: code, type: String}
      - {name: n, type: Integer}
    outcomes:
      - name: matched
        when: GUARD
        emits: [promo.codes.Matched]
        payload:
          promo.codes.Matched:
            n: input.n
      - name: other
        error: promo.codes.Unknown
";

fn with_operator(operator: &str) -> String {
    MODEL.replace("GUARD", &format!("{{code: {{trimmed: {operator}}}}}"))
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("promo.yaml"), raw)])
}

#[test]
fn a_trimmed_literal_with_whitespace_where_it_is_compared_is_refused_as_empty() {
    for operator in [
        r#"{equals: " DONE"}"#,
        r#"{equals: "DONE\t"}"#,
        r#"{eq: "\nDONE"}"#,
        r#"{equals: " "}"#,
        r#"{in: [NEW, "DONE\r"]}"#,
        r#"{any_of: ["\vNEW"]}"#,
        r#"{equals_ignore_case: "done\f"}"#,
        r#"{in_ignore_case: [new, " done "]}"#,
        r#"{starts_with: " SP"}"#,
        r#"{starts_with: "\tSP "}"#,
        r#"{ends_with: "24\n"}"#,
        r#"{ends_with: " 24 "}"#,
    ] {
        let errors = assemble(&with_operator(operator))
            .err()
            .unwrap_or_else(|| panic!("{operator}: must refuse"));
        let errors = errors.as_slice();
        assert_eq!(errors.len(), 1, "{operator}: {errors:#?}");
        assert_eq!(
            errors[0].code,
            ValidationCode::EmptyDeclaration,
            "{operator}: {errors:#?}"
        );
        assert!(
            errors[0].message.contains("not trimmed")
                && errors[0].message.contains("holds for no text"),
            "{operator}: {errors:#?}"
        );
    }
}

#[test]
fn a_trimmed_literal_that_can_hold_is_admitted() {
    for operator in [
        r"{equals: DONE}",
        r#"{equals: ""}"#,
        r#"{equals: "DO NE"}"#,
        "{equals: \"\u{a0}DONE\u{a0}\"}",
        r#"{in: [NEW, "DO\tNE"]}"#,
        r#"{equals_ignore_case: "do ne"}"#,
        r#"{starts_with: "SP "}"#,
        r#"{ends_with: " 24"}"#,
        r#"{contains: " ON "}"#,
        r#"{contains: " "}"#,
    ] {
        assemble(&with_operator(operator)).unwrap_or_else(|errors| panic!("{operator}: {errors}"));
    }
}
