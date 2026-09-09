//! Adversary pass 2: the round-trip law, driven from the documents this unit wrote about itself.
//!
//! `docs/design/review-primitive-semantics.md`, *The round-trip law*, and the doc comment on
//! `tests/primitive_corpus.rs::every_in_process_number_keeps_its_admission_across_the_write_that_loses_its_exactness`
//! state the admission half as universal: "**admission is stable across one write for every
//! `Number`, however it was built**". `facts.rs::is_integral` restates it as the thing that forbids
//! refusing, on re-read, a value that was just admitted and written.
//!
//! The corpus checks that claim only on the fourteen `numbers` vectors it happens to list. This
//! file asks it of the constructor the design page's own table names as a first-class source of a
//! `Number` — `FactValue::parse_literal` of an authored decimal — for values whose binary64 is
//! integral.

use ess_primitives::facts::{FactValue, Number};

/// Every literal a predicate can carry keeps its admission across one write.
///
/// The law's admission half, stated for `FactValue::parse_literal` rather than for the fourteen
/// spellings the corpus lists. `parse_literal` is the read door for a predicate expression
/// (`predicate.rs:267`) and for a quoted `any_of` element (`predicate.rs:1221`), so a `Number` it
/// builds is one that came off a document and is written back to one.
#[test]
fn a_predicate_literal_keeps_its_admission_across_one_write() {
    let write = |number: Number| serde_json::to_string(&number).expect("a number serialises");
    let read = |text: &str| serde_json::from_str::<Number>(text).expect("what was written is read");

    for literal in [
        // An authored decimal with more places than binary64 carries, whose binary64 is integral.
        "1.0000000000000000001",
        "2.0000000000000000001",
        // The brief's own candidate: a fractional value that rounds to an integral binary64.
        "9007199254740992.5",
    ] {
        let built = FactValue::parse_literal(literal)
            .as_number()
            .unwrap_or_else(|| panic!("{literal} is a numeric literal"));
        let written = write(built);
        let back = read(&written);
        assert_eq!(
            back.is_integral(),
            built.is_integral(),
            "{literal}: built integral={}, written {written}, read back integral={}",
            built.is_integral(),
            back.is_integral()
        );
        assert_eq!(
            back.as_i64(),
            built.as_i64(),
            "{literal}: built as_i64={:?}, written {written}, read back as_i64={:?}",
            built.as_i64(),
            back.as_i64()
        );
    }
}

/// `exact_text` is the grammar `ess-gen`'s `DECIMAL_PATTERN` publishes.
///
/// `facts.rs::exact_text`: "The exact decimal spelling: no exponent, no trailing zeroes, one
/// spelling per value. This is the grammar `ess-gen`'s `DECIMAL_PATTERN` publishes for
/// `Primitive::Decimal`." The design page's *What `Number` gains* table repeats it: "the exact
/// decimal spelling, no exponent".
#[test]
fn exact_text_is_always_the_decimal_grammar_it_says_it_is() {
    // ess-gen/src/types.rs:DECIMAL_PATTERN is `^-?(0|[1-9][0-9]*)(\.[0-9]+)?$`, spelt here as a
    // matcher rather than a regular expression because this crate has no regex dependency and an
    // adversary may not add one.
    let matches_decimal_pattern = |text: &str| {
        let body = text.strip_prefix('-').unwrap_or(text);
        let (whole, fraction) = match body.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (body, None),
        };
        let whole_ok = whole == "0"
            || (!whole.is_empty()
                && !whole.starts_with('0')
                && whole.bytes().all(|b| b.is_ascii_digit()));
        let fraction_ok = match fraction {
            None => true,
            Some(fraction) => !fraction.is_empty() && fraction.bytes().all(|b| b.is_ascii_digit()),
        };
        whole_ok && fraction_ok
    };
    // The matcher agrees with the pattern on the spellings the corpus already pins.
    assert!(matches_decimal_pattern("0"));
    assert!(matches_decimal_pattern("-0.1"));
    assert!(matches_decimal_pattern("9223372036854775807"));
    assert!(!matches_decimal_pattern("01"));
    assert!(!matches_decimal_pattern("1."));

    for value in [1e300_f64, -1e300, 1e-300, f64::MIN_POSITIVE] {
        let number = Number::new(value).expect("a finite binary64");
        let text = number.exact_text();
        assert!(
            matches_decimal_pattern(&text),
            "Number::new({value:e}).exact_text() = {text}, which DECIMAL_PATTERN refuses"
        );
    }
}
