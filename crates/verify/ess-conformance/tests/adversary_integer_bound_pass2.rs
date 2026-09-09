//! Adversary pass 2: the acceptance statement at the boundary this unit newly admits.
//!
//! `story:review-primitive-semantics` asks for *equivalent admission and comparison results across
//! facts, conformance, generated codecs and schemas*. The corpus this unit wrote
//! (`crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`) says
//! `integer-two-to-63` — the JSON token `9223372036854775808` — is an admitted `Integer`, and all
//! three admission lanes agree. The generated codec is the fourth lane, and it is not asked.
//!
//! At the base commit `Number::is_integral` was `fract() == 0 && |v| < 2^53`, so this token was
//! *refused* by the Rust conformance admitter. It is admitted now.

use ess_conformance::Holds;
use ess_domain::Primitive;
use ess_primitives::node::Node;
use serde_json::Value;

const CORPUS: &str =
    include_str!("../../../specify/ess-primitives/tests/vectors/primitive-semantics.json");

#[derive(serde::Deserialize)]
struct Vector {
    name: String,
    kind: String,
    value: Value,
    admitted: bool,
}

fn vectors() -> Vec<Vector> {
    #[derive(serde::Deserialize)]
    struct Corpus {
        admission: Vec<Vector>,
    }
    serde_json::from_str::<Corpus>(CORPUS)
        .expect("the corpus is readable")
        .admission
}

/// Every `Integer` the corpus admits is one the generated codec can decode.
///
/// `ess-synth`'s Go layout maps `Primitive::Integer` to `int64` (`go/layout.rs:867`) and decodes it
/// through `json.Number.Int64` (`go/http.rs:1304`), which is `strconv.ParseInt(_, 10, 64)`; the
/// Rust equivalent is `i64`. Both refuse `9223372036854775808`. A conformance suite carrying that
/// token for an `Integer` field is admitted by every gate in this wave and refused by the
/// implementation the same specification generates, which is the divergence the acceptance
/// statement names.
#[test]
fn every_integer_the_corpus_admits_is_one_the_generated_codec_decodes() {
    let mut checked = 0;
    for vector in vectors() {
        if vector.kind != "integer" || !vector.admitted {
            continue;
        }
        // The conformance lane, on the corpus's own value.
        let node: Node = serde_json::from_value(vector.value.clone()).expect("a corpus node");
        assert!(
            Holds::Primitive {
                kind: Primitive::Integer
            }
            .admits(&node),
            "{}: the corpus says admitted and the Rust admitter refuses",
            vector.name
        );
        // The generated-codec lane, on the same JSON token.
        let token = serde_json::to_string(&vector.value).expect("a token");
        assert!(
            serde_json::from_str::<i64>(&token).is_ok(),
            "{}: conformance admits the token {token} as an Integer and the generated codec's \
             int64/i64 decoder refuses it",
            vector.name
        );
        checked += 1;
    }
    assert!(checked >= 3, "the corpus reached {checked} integer vectors");
}

/// The admitted `Integer` range and `as_i64` are the same range.
///
/// `docs/design/review-primitive-semantics.md` claims a reader that needs the integer "gets `None`
/// and must refuse" — so `is_integral` is true on a value no consumer of `as_i64` can use. This
/// asserts the acceptance statement's promise instead: a value admitted as an `Integer` carries the
/// integer it is admitted as.
#[test]
fn a_value_admitted_as_an_integer_yields_the_integer_it_was_admitted_as() {
    for token in [
        "9223372036854775807",
        "-9223372036854775808",
        "9223372036854775808",
    ] {
        let node: Node = serde_json::from_str(token).expect("a number node");
        let admitted = Holds::Primitive {
            kind: Primitive::Integer,
        }
        .admits(&node);
        let Node::Number(number) = node else {
            panic!("{token} is a number node")
        };
        assert_eq!(
            admitted,
            number.as_i64().is_some(),
            "{token}: admitted as an Integer = {admitted}, as_i64 = {:?}",
            number.as_i64()
        );
    }
}

/// Two `Integer`s the model declares distinct stay distinct once they are in a payload.
///
/// The acceptance statement: *equivalent admission and comparison results across facts,
/// conformance, generated codecs and schemas **without losing promised integer or decimal
/// precision***. `9223372036854775296` and `9223372036854775807` are both inside
/// `[i64::MIN, i64::MAX]`, which the design page's `Integer` row declares the abstract value, and
/// both are admitted. An `expect_event` asserting one is therefore satisfied by an implementation
/// that published the other.
#[test]
fn two_declared_integers_are_not_one_value_in_an_expect_event_payload() {
    let expected: Node = serde_json::from_str("9223372036854775807").expect("a number node");
    let observed: Node = serde_json::from_str("9223372036854775296").expect("a number node");
    let holds = Holds::Primitive {
        kind: Primitive::Integer,
    };
    assert!(holds.admits(&expected) && holds.admits(&observed));
    assert_ne!(
        expected, observed,
        "an expect_event asserting i64::MAX is satisfied by an event carrying i64::MAX - 511"
    );
}
