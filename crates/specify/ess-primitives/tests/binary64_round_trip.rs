//! The round-trip law (`docs/design/review-primitive-semantics.md`, *The round-trip law*) for the
//! integers binary64 still carries exactly, just below `2^53` (beyond10x/ess#251).
//!
//! Such an integer is written as its binary64 — `9007199254740991.0` — and its spelling has
//! seventeen significant digits. `serde_json`'s default float reader is not correctly rounded for
//! every such token: it read `9007199254740991.0` back as `9007199254740990`, so a suite that
//! arranged a row at an invariant's bound `lte: 9007199254740991` sent `...990` to the target once
//! the suite was admitted. Every `Number` must read back as itself, through every JSON door.

use ess_primitives::facts::Number;
use ess_primitives::json;
use ess_primitives::node::Node;

/// `2^53 - 1` and other integers near `2^53` whose written spelling ends in `.0`.
fn near_two_to_the_53() -> Vec<i64> {
    let top = (1_i64 << 53) - 1;
    let mut values: Vec<i64> = (0..64).map(|offset| top - offset).collect();
    values.extend((1..=64).map(|step| top - step * 1_000_003));
    values.extend([1 << 52, (1 << 52) + 1, (1 << 53) - 2, 4_503_599_627_370_497]);
    values.extend(values.clone().iter().map(|value| -value));
    values
}

#[test]
fn an_integer_below_two_to_the_53_reads_back_as_itself_as_a_number() {
    let mut wrong = Vec::new();
    for value in near_two_to_the_53() {
        let number = Number::from(value);
        let written = serde_json::to_string(&number).unwrap();
        let read: Number = serde_json::from_str(&written).unwrap();
        if read != number {
            wrong.push(format!(
                "{value}: wrote `{written}`, read `{}`",
                read.exact_text()
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn an_integer_below_two_to_the_53_reads_back_as_itself_as_a_node() {
    let mut wrong = Vec::new();
    for value in near_two_to_the_53() {
        let node = Node::Number(Number::from(value));
        let written = serde_json::to_string(&node).unwrap();
        let read: Node = serde_json::from_str(&written).unwrap();
        if read != node {
            wrong.push(format!("{value}: wrote `{written}`, read `{read:?}`"));
        }
        let through_value: Node =
            serde_json::from_value(json::from_str(&written).unwrap()).unwrap();
        if through_value != node {
            wrong.push(format!(
                "{value}: wrote `{written}`, read through a value `{through_value:?}`"
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn the_bound_of_issue_251_reads_back_as_itself() {
    let read: Number = serde_json::from_str("9007199254740991.0").unwrap();
    assert_eq!(read.as_i64(), Some(9_007_199_254_740_991));
    assert_eq!(
        json::number("9007199254740991.0").unwrap().as_f64(),
        Some(9_007_199_254_740_991.0)
    );
}
