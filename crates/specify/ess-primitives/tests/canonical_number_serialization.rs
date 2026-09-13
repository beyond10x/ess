//! The `Integer` half of `story:primitive-canonical-serialization`'s acceptance.
//!
//! > Two distinct declared `Integer`s (and two distinct `Decimal`s) never compare equal after a
//! > write and a read through any persisted ESS format [...]
//!
//! **Only the `Integer` half is here, and the omission is deliberate.** The `Decimal` half cannot
//! hold without moving both number doors behind `ess-conformance/6` and
//! `ess-conformance-report/2` (`docs/design/review-primitive-semantics.md`, *What stage two
//! changes, and why it is not here*), which moves persisted bytes and is
//! `obligation:review-contract-rollout-coordination`'s decision, not this file's. The measured
//! consequence: closing it rewrites the committed vector
//! `a-decimal-with-more-places-than-binary64-carries` in `tests/vectors/primitive-semantics.json`,
//! which pins `display` and `json` to `0.1` for the authored `0.1000000000000000000001`.
//!
//! "Declared" is the load-bearing word in what is here, and it is why these cases start at the
//! **admission door** rather than at a `Number` that has already been built. `Repr::exact` refuses
//! to build any value that does not survive its own write, so every `Number` that exists is a
//! fixed point of write∘read by construction — asking a built `Number` whether it round-trips asks
//! the constructor's invariant, not the wire's. The question the acceptance asks is whether two
//! *spellings* an author may declare, denoting two different numbers, are still two numbers on the
//! far side of a persisted document.

use ess_primitives::facts::{FactValue, Number};
use ess_primitives::node::Node;

/// Two `Integer` spellings that binary64 cannot tell apart.
///
/// Both are inside `[i64::MIN, i64::MAX]`, so both are declared admissible by `Primitive::Integer`
/// (`docs/design/review-primitive-semantics.md`, *One `Integer` range, and `is_integral` is
/// `as_i64`*). `f64`'s step at this magnitude is `2^11 = 2048`, so both round to `2^63` and were
/// one value before the exact `Number`. Review finding F4 of
/// `review-result:primitive-semantics-adversary-wave21-pass2` measured the consequence — an
/// `expect_event` asserting one is satisfied by an implementation publishing the other.
///
/// `tests/vectors/primitive-semantics.json` pins `9223372036854775807` alone; the *pair* is what
/// F4 named, and nothing pinned it until this file.
const DISTINCT_INTEGERS: [&str; 2] = ["9223372036854775296", "9223372036854775807"];

/// Reads a declared spelling the way a document's author writes it.
fn declared(spelling: &str) -> Number {
    FactValue::parse_literal(spelling)
        .as_number()
        .unwrap_or_else(|| panic!("`{spelling}` is a declared number"))
}

/// One write and one read through JSON, the format every persisted ESS artifact is spelt in.
fn through_json(number: Number) -> Number {
    let written = serde_json::to_string(&number).expect("a number serialises");
    serde_json::from_str::<Number>(&written)
        .unwrap_or_else(|error| panic!("what was written (`{written}`) is read back: {error}"))
}

/// One write and one read through YAML, the format an authored specification is spelt in.
fn through_yaml(number: Number) -> Number {
    let written = serde_yaml::to_string(&number).expect("a number serialises");
    serde_yaml::from_str::<Number>(&written)
        .unwrap_or_else(|error| panic!("what was written (`{written}`) is read back: {error}"))
}

/// Asserts the acceptance for one pair through one format.
fn stay_distinct(format: &str, spellings: [&str; 2], round_trip: fn(Number) -> Number) {
    let [left, right] = spellings.map(declared);
    assert_ne!(
        left, right,
        "{format}: `{}` and `{}` are two declared numbers before any write",
        spellings[0], spellings[1]
    );

    let (left_back, right_back) = (round_trip(left), round_trip(right));
    assert_ne!(
        left_back,
        right_back,
        "{format}: `{}` and `{}` became one value across a write and a read \
         (`{}` and `{}`); an assertion naming one is satisfied by a publisher of the other",
        spellings[0],
        spellings[1],
        left_back.exact_text(),
        right_back.exact_text()
    );
}

#[test]
fn two_distinct_declared_integers_stay_distinct_across_a_json_write_and_read() {
    stay_distinct("JSON", DISTINCT_INTEGERS, through_json);
}

#[test]
fn two_distinct_declared_integers_stay_distinct_across_a_yaml_write_and_read() {
    stay_distinct("YAML", DISTINCT_INTEGERS, through_yaml);
}

/// The read door F4 named: two declared `Integer`s must not be one `Node::Number`.
///
/// `Node` is what a counterexample's input, an artifact's metadata and a witness carry, so this is
/// the door an `expect_event` comparison goes through — and `node.rs`'s own unit test covers only
/// that a number stays a number, not that two numbers stay two.
#[test]
fn two_distinct_declared_integers_are_two_nodes_across_a_json_write_and_read() {
    let [left, right] = DISTINCT_INTEGERS.map(|spelling| Node::Number(declared(spelling)));
    let round_trip = |node: &Node| {
        let written = serde_json::to_string(node).expect("a node serialises");
        serde_json::from_str::<Node>(&written)
            .unwrap_or_else(|error| panic!("what was written (`{written}`) is read back: {error}"))
    };
    assert_ne!(
        round_trip(&left),
        round_trip(&right),
        "two declared `Integer`s became one `Node::Number` across a write and a read"
    );
}
