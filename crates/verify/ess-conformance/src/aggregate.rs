//! Aggregate views (beyond10x/ess#96): what an aggregate is expected to be, and the suite formats
//! that carry one.
//!
//! `docs/design/aggregate-views.md` is the binding design. Two things live here and nowhere else:
//!
//! * **The expected value of each function** over the values a scenario arranged, under the page's
//!   "Semantics": `count` and `count_distinct` over the type's equality, the exact `sum`, `min` and
//!   `max` in the type's order, and `avg` — the exact quotient rounded to [`AVG_SCALE`] fractional
//!   digits, ties to even. Over zero rows `count`, `count_distinct` and `sum` are `0` and the rest are
//!   absent. Synthesis reads every number it asserts from [`evaluate`], so the numbers a suite holds
//!   are the page's and no second implementation of them exists in this crate.
//! * **The format authority**: an aggregate scenario is carried by ordinary suite/[`ORDINARY`] and
//!   coverage suite/[`COVERAGE`], and an `AggregateUnscoped` or `AggregateUnwitnessed` refusal
//!   (`ESS-SYNTH-016`, `ESS-SYNTH-017`) needs coverage suite/[`COVERAGE`]. A reader older than the
//!   construct would refuse an `…/aggregate` id as a malformed name, which blames the document for the
//!   age of the tool; the number turns that into "upgrade the tool". Each major implies every one below
//!   it.
use std::cmp::Ordering;

use ess_domain::view::AggregateFunction;
use ess_primitives::facts::{FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::time::Rfc3339Instant;

use crate::admission::AdmissionError;
use crate::scenario::ScenarioId;
use crate::ConformanceSuite;

/// The first ordinary suite major that carries an aggregate scenario.
pub const ORDINARY: u32 = 16;

/// The coverage counterpart of [`ORDINARY`], and the first coverage major that admits the two
/// aggregate refusal codes.
pub const COVERAGE: u32 = 17;

/// The fractional digits `avg` is rounded to, ties to even.
pub const AVG_SCALE: u32 = 6;

/// `RefusalCause::AggregateUnscoped`'s number in the `SYNTH` family.
pub const UNSCOPED: u16 = 16;

/// `RefusalCause::AggregateUnwitnessed`'s number in the `SYNTH` family.
pub const UNWITNESSED: u16 = 17;

const REQUIRES: &str = "aggregate views require suite/16 or /17";

/// How one input's values compare, read off its unwrapped type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// `Integer` or `Decimal`: numeric value, exact.
    Numeric,
    /// `String`: byte-wise, as text is ordered across the Rust, Go and TypeScript lanes.
    Text,
    /// `Timestamp`: the instant, whatever the offset it is spelled with.
    Timestamp,
    /// `Boolean`, `Uuid` or an enum: equality of the value as carried, and no order.
    Other,
}

/// Whether a typed suite carries an aggregate scenario.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .keys()
        .any(|id| matches!(id, ScenarioId::Aggregate { .. }))
}

/// Refuse an explicitly pinned older format before serialization or target effects.
pub fn admit_suite(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if suite.provenance.suite_version.major() < ORDINARY && used_by(suite) {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}

/// Whether a coverage refusal code is one only coverage suite/[`COVERAGE`] may carry.
pub fn is_aggregate_refusal(code: &str) -> bool {
    [UNSCOPED, UNWITNESSED]
        .iter()
        .any(|number| code == format!("ESS-SYNTH-{number:03}"))
}

/// The value `function` takes over `values`, which a group's admitted rows hold.
///
/// `None` only where a numeric value has no exact decimal spelling — a magnitude the scenario's
/// arrangement never produces, and one no claim is made about — or where a value is not of the
/// kind named.
pub fn evaluate(function: AggregateFunction, values: &[Node], kind: ValueKind) -> Option<Node> {
    match function {
        AggregateFunction::Count => Some(integer(values.len())),
        AggregateFunction::CountDistinct => {
            let mut distinct: Vec<&Node> = Vec::new();
            for value in values {
                let mut seen = false;
                for held in &distinct {
                    if equal(held, value, kind)? {
                        seen = true;
                        break;
                    }
                }
                if !seen {
                    distinct.push(value);
                }
            }
            Some(integer(distinct.len()))
        }
        AggregateFunction::Sum => {
            let (units, scale) = sum(values)?;
            decimal(units, scale)
        }
        AggregateFunction::Min | AggregateFunction::Max => {
            let mut best: Option<&Node> = None;
            for value in values {
                best = Some(match best {
                    None => value,
                    Some(held) => {
                        let order = compare(value, held, kind)?;
                        let wins = if function == AggregateFunction::Min {
                            order == Ordering::Less
                        } else {
                            order == Ordering::Greater
                        };
                        if wins {
                            value
                        } else {
                            held
                        }
                    }
                });
            }
            Some(best.cloned().unwrap_or(Node::Null))
        }
        AggregateFunction::Avg => {
            if values.is_empty() {
                return Some(Node::Null);
            }
            let (numerator, denominator) = avg_fraction(values)?;
            decimal(divide_half_even(numerator, denominator)?, AVG_SCALE)
        }
    }
}

/// Whether `avg` over `values` rounded (half-even) and truncated at [`AVG_SCALE`] places are two
/// different numbers — the only case in which an asserted mean catches a truncating `avg`.
///
/// `None` where a value has no exact decimal spelling, or there are no values.
pub fn avg_separates_rounding(values: &[Node]) -> Option<bool> {
    if values.is_empty() {
        return None;
    }
    let (numerator, denominator) = avg_fraction(values)?;
    Some(divide_half_even(numerator, denominator)? != numerator / denominator)
}

/// `avg` over `values` as the fraction whose quotient, rounded to an integer, is the mean in
/// units of 10⁻⁶.
fn avg_fraction(values: &[Node]) -> Option<(i128, i128)> {
    let (units, scale) = sum(values)?;
    let count = i128::try_from(values.len()).ok()?;
    if scale <= AVG_SCALE {
        Some((
            units.checked_mul(10_i128.checked_pow(AVG_SCALE - scale)?)?,
            count,
        ))
    } else {
        Some((
            units,
            count.checked_mul(10_i128.checked_pow(scale - AVG_SCALE)?)?,
        ))
    }
}

/// `units × 10⁻ˢᶜᵃˡᵉ` rounded to `to` fractional digits, ties to even, as `(units, to)`.
///
/// A value already at or under `to` digits is returned at `to` digits unchanged.
pub fn round_half_even(units: i128, scale: u32, to: u32) -> (i128, u32) {
    if scale <= to {
        let widened = 10_i128
            .checked_pow(to - scale)
            .and_then(|factor| units.checked_mul(factor))
            .unwrap_or(units);
        return (widened, to);
    }
    let divisor = 10_i128.checked_pow(scale - to).unwrap_or(i128::MAX);
    (divide_half_even(units, divisor).unwrap_or(0), to)
}

/// `numerator / denominator` rounded to an integer, ties to even.
fn divide_half_even(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator == 0 {
        return None;
    }
    let negative = (numerator < 0) != (denominator < 0);
    let (a, b) = (numerator.checked_abs()?, denominator.checked_abs()?);
    let (mut quotient, remainder) = (a / b, a % b);
    let twice = remainder.checked_mul(2)?;
    if twice > b || (twice == b && quotient % 2 == 1) {
        quotient += 1;
    }
    Some(if negative { -quotient } else { quotient })
}

/// The exact sum of numeric values, as `(units, scale)`.
fn sum(values: &[Node]) -> Option<(i128, u32)> {
    let mut total: (i128, u32) = (0, 0);
    for value in values {
        let (units, scale) = exact(value)?;
        let common = total.1.max(scale);
        let left = total
            .0
            .checked_mul(10_i128.checked_pow(common - total.1)?)?;
        let right = units.checked_mul(10_i128.checked_pow(common - scale)?)?;
        total = (left.checked_add(right)?, common);
    }
    Some(total)
}

/// A number node as an exact decimal `(units, scale)`.
fn exact(value: &Node) -> Option<(i128, u32)> {
    let Node::Number(number) = value else {
        return None;
    };
    let text = number.exact_text();
    if text.contains(['e', 'E']) {
        return None;
    }
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.as_str()),
    };
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    let units: i128 = format!("{whole}{fraction}").parse().ok()?;
    let scale = u32::try_from(fraction.len()).ok()?;
    Some((if negative { -units } else { units }, scale))
}

/// The number `units × 10⁻ˢᶜᵃˡᵉ`, spelled without trailing zeroes.
fn decimal(units: i128, scale: u32) -> Option<Node> {
    let negative = units < 0;
    let digits = units.unsigned_abs().to_string();
    let width = usize::try_from(scale).ok()?;
    let padded = format!("{digits:0>width$}", width = width + 1);
    let (whole, fraction) = padded.split_at(padded.len() - width);
    let fraction = fraction.trim_end_matches('0');
    let mut text = String::new();
    if negative && (!whole.trim_start_matches('0').is_empty() || !fraction.is_empty()) {
        text.push('-');
    }
    text.push_str(whole);
    if !fraction.is_empty() {
        text.push('.');
        text.push_str(fraction);
    }
    match FactValue::parse_literal(&text) {
        FactValue::Number(number) => Some(Node::Number(number)),
        _ => None,
    }
}

fn integer(count: usize) -> Node {
    Node::Number(Number::from(count))
}

fn instant(value: &Node) -> Option<Rfc3339Instant> {
    match value {
        Node::Text(text) => Rfc3339Instant::parse_rfc3339(text),
        _ => None,
    }
}

/// Whether two values are one under the type's equality.
fn equal(left: &Node, right: &Node, kind: ValueKind) -> Option<bool> {
    match kind {
        ValueKind::Numeric => Some(exact_normalised(left)? == exact_normalised(right)?),
        ValueKind::Timestamp => Some(instant(left)? == instant(right)?),
        ValueKind::Text | ValueKind::Other => Some(left == right),
    }
}

/// An exact decimal with its trailing zeroes removed, so `1.0` and `1` are one key.
fn exact_normalised(value: &Node) -> Option<(i128, u32)> {
    let (mut units, mut scale) = exact(value)?;
    while scale > 0 && units % 10 == 0 {
        units /= 10;
        scale -= 1;
    }
    Some((units, scale))
}

/// The order `min` and `max` read, or `None` for a kind that has none.
fn compare(left: &Node, right: &Node, kind: ValueKind) -> Option<Ordering> {
    match kind {
        ValueKind::Numeric => {
            let (l, r) = (exact(left)?, exact(right)?);
            let common = l.1.max(r.1);
            let widen = |(units, scale): (i128, u32)| {
                units.checked_mul(10_i128.checked_pow(common - scale)?)
            };
            Some(widen(l)?.cmp(&widen(r)?))
        }
        ValueKind::Timestamp => Some(instant(left)?.cmp(&instant(right)?)),
        ValueKind::Text => match (left, right) {
            (Node::Text(l), Node::Text(r)) => Some(l.as_bytes().cmp(r.as_bytes())),
            _ => None,
        },
        ValueKind::Other => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_decimal_spells_its_value_without_trailing_zeroes() {
        let text = |units, scale| match decimal(units, scale) {
            Some(Node::Number(number)) => number.exact_text(),
            other => panic!("{other:?}"),
        };
        assert_eq!(text(7_666_667, 6), "7.666667");
        assert_eq!(text(1_500_000, 6), "1.5");
        assert_eq!(text(0, 6), "0");
        assert_eq!(text(-2, 6), "-0.000002");
        assert_eq!(text(85_000_000, 6), "85");
    }
}
