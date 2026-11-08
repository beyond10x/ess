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
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedAggregate, ResolvedBody, ResolvedEntity, ResolvedTypeRef};
use ess_domain::types::{Primitive, MAX_TYPE_DEPTH};
use ess_domain::view::AggregateFunction;
use ess_primitives::facts::{FactPath, FactValue, Number};
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

/// How the values one measure reads compare (ess/36, `docs/design/generic-language-features.md`
/// § G3): a [`ValueKind`], and where the values are a `Duration` or an enum declaring
/// `ordered: true`, what [`measured`] turns them into first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasureKind {
    /// Compared as the kind says.
    Plain(ValueKind),
    /// Elapsed seconds: [`measured`] reads each as a number, and a `sum`, `min` or `max` is
    /// spelled back as a `Duration`; a mean stays the `Decimal` of seconds.
    Duration,
    /// The variants of an ordered enum, in declaration order: [`measured`] reads each as its
    /// position, and a `min` or `max` is spelled back as the variant.
    Ordered(Vec<String>),
}

/// How the values of a measure over `type_ref`, a source field's declared type, compare.
pub fn field_kind(ir: &EssIr, type_ref: &ResolvedTypeRef) -> MeasureKind {
    let mut current = type_ref.required();
    for _ in 0..=MAX_TYPE_DEPTH {
        let ResolvedTypeRef::Declared { name } = current else {
            break;
        };
        let declared = ir.named_type(name);
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => current = of.required(),
            ResolvedBody::Enum { variants } if declared.ordered => {
                return MeasureKind::Ordered(
                    variants
                        .iter()
                        .map(|variant| variant.name().to_owned())
                        .collect(),
                )
            }
            _ => return MeasureKind::Plain(ValueKind::Other),
        }
    }
    match current {
        ResolvedTypeRef::Primitive {
            name: Primitive::Integer | Primitive::Decimal,
        } => MeasureKind::Plain(ValueKind::Numeric),
        ResolvedTypeRef::Primitive {
            name: Primitive::String,
        } => MeasureKind::Plain(ValueKind::Text),
        ResolvedTypeRef::Primitive {
            name: Primitive::Timestamp,
        } => MeasureKind::Plain(ValueKind::Timestamp),
        ResolvedTypeRef::Primitive {
            name: Primitive::Duration,
        } => MeasureKind::Duration,
        _ => MeasureKind::Plain(ValueKind::Other),
    }
}

/// How the values of `aggregate` over rows of `entity` compare: its input field's, or what its
/// expression computes; [`ValueKind::Other`] for `count`, which reads no value.
pub fn measure_kind(
    ir: &EssIr,
    entity: &ResolvedEntity,
    aggregate: &ResolvedAggregate,
) -> MeasureKind {
    use ess_primitives::expr::Kind;
    match (&aggregate.input, &aggregate.expr) {
        (Some(input), _) => entity
            .observable_field(&input.name)
            .map_or(MeasureKind::Plain(ValueKind::Other), |field| {
                field_kind(ir, &field.type_ref)
            }),
        (None, Some(measured)) => match measured.typed.kind {
            Kind::Integer | Kind::Decimal => MeasureKind::Plain(ValueKind::Numeric),
            Kind::String => MeasureKind::Plain(ValueKind::Text),
            Kind::Timestamp => MeasureKind::Plain(ValueKind::Timestamp),
            Kind::Duration => MeasureKind::Duration,
            Kind::Bytes | Kind::Boolean => MeasureKind::Plain(ValueKind::Other),
        },
        (None, None) => MeasureKind::Plain(ValueKind::Other),
    }
}

/// The value `aggregate` reads off one source row of `entity`, `row` holding its observable
/// fields by name, in the form [`evaluate_measure`] compares: `null` where it is absent, and `None`
/// where it is unknown — an expression over a value it cannot read.
///
/// A field input is the row's value; an expression (ess/36) is evaluated by the shared evaluator,
/// a read `<field>.<attribute>` reading that attribute of the variant the row holds. A `Duration`
/// is read as its elapsed seconds and an ordered enum's variant as its position, so the functions
/// compare them as numbers.
pub fn measured(
    ir: &EssIr,
    entity: &ResolvedEntity,
    aggregate: &ResolvedAggregate,
    row: &BTreeMap<String, Node>,
) -> Option<Node> {
    use ess_primitives::expr::{Evaluated, Reading, Value};
    let kind = measure_kind(ir, entity, aggregate);
    let held = match (&aggregate.input, &aggregate.expr) {
        (Some(input), _) => row.get(&input.name).cloned().unwrap_or(Node::Null),
        (None, Some(measured)) => {
            let reads = |path: &FactPath| match path.segments() {
                [field] => row.get(field).map_or(Reading::Absent, Reading::of_node),
                [field, attribute] => match row.get(field) {
                    None | Some(Node::Null) => Reading::Absent,
                    Some(Node::Text(variant)) => {
                        attribute_of(ir, entity, field, variant, attribute)
                    }
                    Some(_) => Reading::Unknown,
                },
                _ => Reading::Unknown,
            };
            match measured.expr.evaluate(&reads) {
                Evaluated::Absent => Node::Null,
                Evaluated::Unknown => return None,
                Evaluated::Value(Value::Duration(seconds)) => {
                    return Some(Node::Number(Number::from(i64::try_from(seconds).ok()?)))
                }
                value @ Evaluated::Value(_) => value.to_node()?,
            }
        }
        (None, None) => return Some(Node::Null),
    };
    Some(match (&kind, held) {
        (_, Node::Null) => Node::Null,
        (MeasureKind::Duration, Node::Text(text)) => {
            Node::Number(Number::from(ess_primitives::expr::parse_elapsed(&text)?))
        }
        (MeasureKind::Ordered(variants), Node::Text(text)) => Node::Number(Number::from(
            variants.iter().position(|variant| *variant == text)?,
        )),
        (_, held) => held,
    })
}

/// `attribute` of `variant`, the variant of the enum `field` of `entity` is declared as, as the
/// expression reads it: absent where the variant leaves an `Optional` attribute unfilled.
fn attribute_of(
    ir: &EssIr,
    entity: &ResolvedEntity,
    field: &str,
    variant: &str,
    attribute: &str,
) -> ess_primitives::expr::Reading {
    use ess_primitives::expr::Reading;
    let Some(declared) = entity.observable_field(field) else {
        return Reading::Unknown;
    };
    let mut current = declared.type_ref.required();
    for _ in 0..=MAX_TYPE_DEPTH {
        let ResolvedTypeRef::Declared { name } = current else {
            return Reading::Unknown;
        };
        match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => current = of.required(),
            ResolvedBody::Enum { variants } => {
                let Some(found) = variants
                    .iter()
                    .find(|declared| declared.name() == variant)
                    .and_then(|declared| {
                        declared
                            .attributes
                            .iter()
                            .find(|held| held.name == attribute)
                    })
                else {
                    return Reading::Unknown;
                };
                let Some(text) = &found.value else {
                    return Reading::Absent;
                };
                let textual = matches!(
                    found.type_ref.required(),
                    ess_domain::types::TypeRef::Primitive(Primitive::String)
                );
                if textual {
                    return Reading::Value(FactValue::Text(text.clone()));
                }
                return match FactValue::parse_literal(text) {
                    FactValue::Number(number) => Reading::Value(FactValue::Number(number)),
                    FactValue::Bool(value) => Reading::Value(FactValue::Bool(value)),
                    FactValue::Text(_) => Reading::Value(FactValue::Text(text.clone())),
                };
            }
            _ => return Reading::Unknown,
        }
    }
    Reading::Unknown
}

/// The value `function` takes over `values`, each read by [`measured`] under `kind`, skipping
/// absent values where `skip_absent` says so: [`evaluate`] or [`evaluate_skipping_absent`], with a
/// `Duration`'s total and extremes spelled back as a `Duration` and an ordered enum's extremes as
/// the variant.
pub fn evaluate_measure(
    function: AggregateFunction,
    values: &[Node],
    kind: &MeasureKind,
    skip_absent: bool,
) -> Option<Node> {
    let plain = match kind {
        MeasureKind::Plain(kind) => *kind,
        MeasureKind::Duration | MeasureKind::Ordered(_) => ValueKind::Numeric,
    };
    let value = if skip_absent {
        evaluate_skipping_absent(function, values, plain)
    } else {
        evaluate(function, values, plain)
    }?;
    let spelled = matches!(
        function,
        AggregateFunction::Sum | AggregateFunction::Min | AggregateFunction::Max
    );
    Some(match (kind, &value) {
        (MeasureKind::Duration, Node::Number(number)) if spelled => {
            let seconds = number.as_i64()?;
            if seconds < 0 {
                return None;
            }
            Node::Text(format!("PT{seconds}S"))
        }
        (MeasureKind::Ordered(variants), Node::Number(number))
            if function != AggregateFunction::Sum && spelled =>
        {
            Node::Text(
                variants
                    .get(usize::try_from(number.as_i64()?).ok()?)?
                    .clone(),
            )
        }
        _ => value,
    })
}

/// The value a skipping aggregate (`skip_absent: true`, `docs/design/aggregate-views.md`, "Absent
/// values") takes over `values`, where an absent value is `Node::Null`: SQL's treatment of `NULL`.
///
/// The absent values are skipped and [`evaluate`] reads the rest, except that a `sum` over no
/// present value is absent, as SQL's `SUM` is — not the `0` a required input's empty sum is.
/// `count_distinct` over no present value is `0`. `count` counts rows and is never skipping; it is
/// passed through unchanged.
pub fn evaluate_skipping_absent(
    function: AggregateFunction,
    values: &[Node],
    kind: ValueKind,
) -> Option<Node> {
    if function == AggregateFunction::Count {
        return evaluate(function, values, kind);
    }
    let present = present(values);
    if present.is_empty() && function == AggregateFunction::Sum {
        return Some(Node::Null);
    }
    evaluate(function, &present, kind)
}

/// The values that are not absent.
pub fn present(values: &[Node]) -> Vec<Node> {
    values
        .iter()
        .filter(|value| **value != Node::Null)
        .cloned()
        .collect()
}

/// How much a numeric aggregate moved from `before` to `after`, exactly: `after − before`, where an
/// absent value (`Node::Null`) is zero — a `sum` over no present value, as a change reads it.
///
/// `None` where either is neither absent nor a number with an exact decimal spelling.
pub fn change(before: &Node, after: &Node) -> Option<Node> {
    let zero = |value: &Node| match value {
        Node::Null => Some((0, 0)),
        other => exact(other),
    };
    let (before, after) = (zero(before)?, zero(after)?);
    let common = before.1.max(after.1);
    let widen =
        |(units, scale): (i128, u32)| units.checked_mul(10_i128.checked_pow(common - scale)?);
    decimal(widen(after)?.checked_sub(widen(before)?)?, common)
}

/// Whether two numbers are one value, whatever their spelling (`5`, `5.0`).
///
/// `None` where either is not a number with an exact decimal spelling.
pub fn same_number(left: &Node, right: &Node) -> Option<bool> {
    equal(left, right, ValueKind::Numeric)
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

    #[test]
    fn a_change_is_exact_and_reads_an_absent_value_as_zero() {
        let n = |text: &str| match FactValue::parse_literal(text) {
            FactValue::Number(number) => Node::Number(number),
            other => panic!("{other:?}"),
        };
        let moved = |before: &Node, after: &Node| match change(before, after) {
            Some(Node::Number(number)) => number.exact_text(),
            other => panic!("{other:?}"),
        };
        assert_eq!(moved(&n("40"), &n("45")), "5");
        assert_eq!(moved(&Node::Null, &n("5")), "5");
        assert_eq!(moved(&n("1.25"), &n("3")), "1.75");
        assert_eq!(moved(&n("7"), &n("4")), "-3");
        assert_eq!(moved(&Node::Null, &Node::Null), "0");
        assert_eq!(change(&Node::Text("5".into()), &n("5")), None);
        assert_eq!(same_number(&n("5"), &n("5.0")), Some(true));
        assert_eq!(same_number(&n("5"), &n("4")), Some(false));
    }
}
