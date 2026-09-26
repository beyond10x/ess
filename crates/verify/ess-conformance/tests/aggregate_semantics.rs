//! The expected value of every aggregate function, as `docs/design/aggregate-views.md` fixes it.
//!
//! "Deciding checks" 3: half-even at 6 places, the zero-row values, and `Decimal` equality in
//! `count_distinct`. These are the numbers a synthesized suite asserts, so they are checked here
//! against the page and not against an implementation.
use ess_conformance::aggregate::{round_half_even, ValueKind, AVG_SCALE};
use ess_domain::view::AggregateFunction as F;
use ess_primitives::facts::FactValue;
use ess_primitives::node::Node;

fn n(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{text} is not a number: {other:?}"),
    }
}

fn t(text: &str) -> Node {
    Node::Text(text.to_owned())
}

/// The value the page fixes; every input here is an exact decimal, so there is always one.
fn eval(function: F, values: &[Node], kind: ValueKind) -> Node {
    ess_conformance::aggregate::evaluate(function, values, kind).expect("an exact value")
}

#[test]
fn avg_rounds_to_six_places_ties_to_even() {
    assert_eq!(AVG_SCALE, 6);
    assert_eq!(eval(F::Avg, &[n("0.0000005")], ValueKind::Numeric), n("0"));
    assert_eq!(
        eval(F::Avg, &[n("0.0000015")], ValueKind::Numeric),
        n("0.000002")
    );
    assert_eq!(
        eval(F::Avg, &[n("0.0000025")], ValueKind::Numeric),
        n("0.000002")
    );
    assert_eq!(
        eval(F::Avg, &[n("-0.0000015")], ValueKind::Numeric),
        n("-0.000002")
    );
    // The page's example: 46 / 6.
    let talk = ["1", "1", "3", "7", "13", "21"].map(n);
    assert_eq!(eval(F::Avg, &talk, ValueKind::Numeric), n("7.666667"));
    // Not truncated: 7.6666666… rounds up.
    assert_ne!(eval(F::Avg, &talk, ValueKind::Numeric), n("7.666666"));
    assert_eq!(round_half_even(5, 7, 6), (0, 6));
    assert_eq!(round_half_even(15, 7, 6), (2, 6));
    assert_eq!(round_half_even(-25, 7, 6), (-2, 6));
}

#[test]
fn over_zero_rows_count_and_sum_are_zero_and_extremes_and_avg_are_absent() {
    assert_eq!(eval(F::Count, &[], ValueKind::Other), n("0"));
    assert_eq!(eval(F::CountDistinct, &[], ValueKind::Text), n("0"));
    assert_eq!(eval(F::Sum, &[], ValueKind::Numeric), n("0"));
    assert_eq!(eval(F::Min, &[], ValueKind::Numeric), Node::Null);
    assert_eq!(eval(F::Max, &[], ValueKind::Text), Node::Null);
    assert_eq!(eval(F::Avg, &[], ValueKind::Numeric), Node::Null);
}

#[test]
fn count_distinct_uses_the_equality_of_the_type() {
    // `1.0` and `1` are one value.
    assert_eq!(
        eval(
            F::CountDistinct,
            &[n("1.0"), n("1"), n("1.5")],
            ValueKind::Numeric
        ),
        n("2")
    );
    // The same instant, spelled twice.
    assert_eq!(
        eval(
            F::CountDistinct,
            &[t("2026-01-01T01:00:00Z"), t("2026-01-01T02:00:00+01:00")],
            ValueKind::Timestamp
        ),
        n("1")
    );
    assert_eq!(
        eval(F::CountDistinct, &[t("a"), t("b"), t("a")], ValueKind::Text),
        n("2")
    );
    assert_eq!(
        eval(F::Count, &[t("a"), t("b"), t("a")], ValueKind::Text),
        n("3")
    );
}

#[test]
fn sum_is_exact_and_extremes_follow_the_type_order() {
    assert_eq!(
        eval(F::Sum, &[n("0.1"), n("0.2")], ValueKind::Numeric),
        n("0.3")
    );
    assert_eq!(
        eval(F::Sum, &["101", "101", "103"].map(n), ValueKind::Numeric),
        n("305")
    );
    assert_eq!(
        eval(F::Max, &["101", "113", "103"].map(n), ValueKind::Numeric),
        n("113")
    );
    assert_eq!(
        eval(F::Min, &["101", "113", "103"].map(n), ValueKind::Numeric),
        n("101")
    );
    // Byte-wise for text: `B` (0x42) sorts before `a` (0x61).
    assert_eq!(eval(F::Min, &[t("a"), t("B")], ValueKind::Text), t("B"));
    // The instant for a timestamp, and the value as the row holds it.
    assert_eq!(
        eval(
            F::Max,
            &[t("2026-01-01T02:30:00+01:00"), t("2026-01-01T02:00:00Z")],
            ValueKind::Timestamp
        ),
        t("2026-01-01T02:00:00Z")
    );
}
