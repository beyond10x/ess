//! What a synthesized scenario expects of an expression value, `{expr: …}` (ess/27,
//! beyond10x/ess#560, `docs/design/generic-language-features.md` § G1, "Synthesis cost").
//!
//! A value is evaluated forwards: the scenario arranges the facts, and the expected value is what
//! the shared evaluator ([`ess_primitives::expr`]) computes from the input the scenario sent and the
//! row it arranged, so no search is added. What is added is a choice of inputs worth checking the
//! figure on, each a further run of the branch ([`points`]): the instants and numbers an expression
//! reads moved apart, a zero divisor, and a rounding tie on either neighbour.

use std::collections::BTreeMap;

use ess_compiler::ir::{ResolvedOutcome, ResolvedPayloadValue};
use ess_primitives::expr::{ArithOp, Evaluated, Expr, Kind, Reading, Rounding, DIVISION_DIGITS};
use ess_primitives::facts::{FactPath, Number};
use ess_primitives::node::Node;
use ess_primitives::time::Rfc3339Instant;

use super::Determined;
use crate::ScenarioValue;

/// The value `expr` computes over what the scenario sent (`supplied`) and the row before the act
/// (`before`): `null` where it is absent, and `None` where it is unknown or reads a value the
/// scenario does not determine.
pub(super) fn expected(
    expr: &Expr,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> Option<ScenarioValue> {
    let reads = |path: &FactPath| -> Reading {
        let text = path.to_string();
        if let Some(input) = text.strip_prefix("input.") {
            return match super::supplied_at(supplied, input) {
                Some(ScenarioValue::Literal { value }) => Reading::of_node(&value),
                Some(_) => Reading::Unknown,
                None => Reading::Absent,
            };
        }
        if let Some(field) = text.strip_prefix("subject.") {
            return match before.get(field).map(|held| &held.value) {
                Some(ScenarioValue::Literal { value }) => Reading::of_node(value),
                _ => Reading::Unknown,
            };
        }
        Reading::Unknown
    };
    match expr.evaluate(&reads) {
        Evaluated::Unknown => None,
        evaluated => evaluated
            .to_node()
            .map(|value| ScenarioValue::Literal { value }),
    }
}

/// A view's computed field over a row (`expr:`, ess/27, beyond10x/ess#560): each field it reads
/// from what the row is claimed to hold (`shown`, the identity among them) or what the scenario
/// settled. `None` where a field it reads is not determined, or the value is unknown.
pub(super) fn computed_field(
    expr: &Expr,
    shown: &BTreeMap<String, ScenarioValue>,
    settled: &BTreeMap<String, Determined>,
) -> Option<ScenarioValue> {
    let reads = |path: &FactPath| -> Reading {
        let name = path.to_string();
        let value = shown
            .get(&name)
            .or_else(|| settled.get(&name).map(|held| &held.value));
        match value {
            Some(ScenarioValue::Literal { value }) => Reading::of_node(value),
            _ => Reading::Unknown,
        }
    };
    match expr.evaluate(&reads) {
        Evaluated::Unknown => None,
        evaluated => evaluated
            .to_node()
            .map(|value| ScenarioValue::Literal { value }),
    }
}

/// The input fields `expr` reads, by their first segment, each once.
pub(super) fn input_reads(expr: &Expr) -> Vec<&str> {
    let mut found = Vec::new();
    for path in expr.reads() {
        if let [root, field, ..] = path.segments() {
            if root == "input" && !found.contains(&field.as_str()) {
                found.push(field.as_str());
            }
        }
    }
    found
}

/// Every expression an outcome's values compute, payloads, error payload and `sets:` alike.
fn expressions(outcome: &ResolvedOutcome) -> Vec<&Expr> {
    let mut found = Vec::new();
    for field in outcome
        .payload
        .iter()
        .flat_map(|payload| &payload.fields)
        .chain(&outcome.error_payload)
        .chain(&outcome.sets)
    {
        if let ResolvedPayloadValue::Expression { expr, .. } = &field.value {
            if !found.contains(&expr) {
                found.push(expr);
            }
        }
    }
    found
}

/// One input read directly by an expression: its field and kind.
fn direct_input(expr: &Expr) -> Option<(&str, Kind)> {
    let Expr::Read {
        path,
        kind: Some(kind),
    } = expr
    else {
        return None;
    };
    match path.segments() {
        [root, field] if root == "input" => Some((field.as_str(), *kind)),
        _ => None,
    }
}

/// The input overrides that move every input the branch's expressions read apart from the others,
/// starting from `plain`: the `Timestamp`s from the first one's value, the first read latest, each
/// `61` minutes after the next; the `Duration`s likewise by `61` seconds; and each number up by
/// `7 × n`, the first read most. So `ended_at - started_at` is a positive duration whatever the
/// witness sent, two instants are never equal, and a target computing `started_at - ended_at`, or a
/// constant, publishes another figure.
pub(super) fn apart(
    outcome: &ResolvedOutcome,
    plain: &BTreeMap<String, Node>,
) -> BTreeMap<String, Node> {
    let mut read: Vec<(&String, Kind)> = Vec::new();
    for expr in expressions(outcome) {
        for path in expr.reads() {
            let [root, field] = path.segments() else {
                continue;
            };
            if root != "input" || read.iter().any(|(seen, _)| *seen == field) {
                continue;
            }
            if let Some(kind) = read_kind(expr, path) {
                read.push((field, kind));
            }
        }
    }
    let mut apart = BTreeMap::new();
    for kind in [
        Kind::Timestamp,
        Kind::Duration,
        Kind::Integer,
        Kind::Decimal,
    ] {
        let of_kind: Vec<&String> = read
            .iter()
            .filter(|(_, read_kind)| *read_kind == kind)
            .map(|(field, _)| *field)
            .collect();
        let count = i64::try_from(of_kind.len()).unwrap_or(i64::MAX);
        let base = of_kind.first().and_then(|field| plain.get(*field));
        for (position, field) in (0i64..).zip(of_kind.iter()) {
            let from = match kind {
                Kind::Timestamp | Kind::Duration => base,
                _ => plain.get(*field),
            };
            if let Some(moved) = from.and_then(|value| moved(value, kind, count - position)) {
                apart.insert((*field).clone(), moved);
            }
        }
    }
    apart
}

/// The input overrides of every further run the branch's expressions are checked on, in order,
/// each over [`apart`]'s:
///
/// 1. **Apart**, alone.
/// 2. **A zero divisor.** For each `a / b` whose divisor is an input read directly, the divisor
///    `0`, so the quotient is absent and what stands in for it is asserted.
/// 3. **A tie.** For each such quotient whose dividend is an input read directly too, the dividend
///    and divisor that put the quotient exactly half-way between two neighbours at its digits — the
///    digits of the `round(…)` around it, or six — once on an even and once on an odd neighbour, so a
///    target rounding by another mode fails one of them.
///
/// `plain` is the witness the run would otherwise send: a point moves values from it.
pub(super) fn points(
    outcome: &ResolvedOutcome,
    plain: &BTreeMap<String, Node>,
) -> Vec<BTreeMap<String, Node>> {
    let mut points = Vec::new();
    let expressions = expressions(outcome);
    if expressions.is_empty() {
        return points;
    }
    let apart = apart(outcome, plain);
    points.push(apart.clone());
    let with = |overrides: &[(&str, Node)]| {
        let mut point = apart.clone();
        for (field, value) in overrides {
            point.insert((*field).to_owned(), value.clone());
        }
        point
    };
    for expr in expressions {
        quotients(expr, None, &mut |dividend, divisor, digits| {
            let Some((divisor, divisor_kind)) = direct_input(divisor) else {
                return;
            };
            points.push(with(&[(divisor, zero(divisor_kind))]));
            let Some((dividend, dividend_kind)) = direct_input(dividend) else {
                return;
            };
            // `odd / (2 · 10^digits)` is `odd` half-units at the digits: a tie on the neighbour
            // below it when `(odd - 1) / 2` is even, and above it when it is odd.
            let Some(scale) = 10i64.checked_pow(u32::from(digits)) else {
                return;
            };
            for odd in [5i64, 3] {
                let (Some(top), Some(bottom)) = (
                    number(odd, dividend_kind),
                    scale
                        .checked_mul(2)
                        .and_then(|bottom| number(bottom, divisor_kind)),
                ) else {
                    continue;
                };
                points.push(with(&[(dividend, top), (divisor, bottom)]));
            }
        });
    }
    points.dedup();
    points
}

/// Whether every expression of `outcome` that reads only the input has a value its field can hold
/// over `input`: a point where one has none — a negative `Duration`, an instant past the years a
/// `date-time` spells — checks nothing a target could publish, so it is not run.
pub(super) fn spelled_at(outcome: &ResolvedOutcome, input: &BTreeMap<String, Node>) -> bool {
    let reads = |path: &FactPath| -> Reading {
        match path.segments() {
            [root, rest @ ..] if root == "input" && !rest.is_empty() => {
                ess_compiler::ir::read_input(input, &rest.join("."))
                    .map_or(Reading::Absent, Reading::of_node)
            }
            _ => Reading::Unknown,
        }
    };
    expressions(outcome).into_iter().all(|expr| {
        let only_input = expr
            .reads()
            .iter()
            .all(|path| path.segments().first().is_some_and(|root| root == "input"));
        !only_input || expr.evaluate(&reads).to_node().is_some()
    })
}

/// How many further runs [`points`] can ask for at most: one apart, and three per quotient.
pub(super) fn point_bound(outcome: &ResolvedOutcome) -> usize {
    let expressions = expressions(outcome);
    if expressions.is_empty() {
        return 0;
    }
    let mut bound = 1;
    for expr in expressions {
        quotients(expr, None, &mut |_, _, _| bound += 3);
    }
    bound
}

/// Calls `found` with the dividend, divisor and digits of every quotient in `expr`.
fn quotients(expr: &Expr, digits: Option<u8>, found: &mut dyn FnMut(&Expr, &Expr, u8)) {
    match expr {
        Expr::Arith {
            op: ArithOp::Divide,
            left,
            right,
        } => {
            found(left, right, digits.unwrap_or(DIVISION_DIGITS));
            quotients(left, None, found);
            quotients(right, None, found);
        }
        Expr::Arith { left, right, .. } => {
            quotients(left, None, found);
            quotients(right, None, found);
        }
        Expr::Round {
            value,
            digits,
            mode: Rounding::HalfEven | Rounding::HalfUp | Rounding::Down,
        } => quotients(value, Some(*digits), found),
        Expr::Negate(inner) => quotients(inner, None, found),
        Expr::Call { arguments, .. } => {
            for argument in arguments {
                quotients(argument, None, found);
            }
        }
        Expr::Integer(_)
        | Expr::Decimal(_)
        | Expr::Duration { .. }
        | Expr::Text(_)
        | Expr::Boolean(_)
        | Expr::Now
        | Expr::Read { .. } => {}
    }
}

/// The kind `expr` reads `path` at.
fn read_kind(expr: &Expr, path: &FactPath) -> Option<Kind> {
    let mut kind = None;
    let mut visit = |inner: &Expr| {
        if let Expr::Read {
            path: read,
            kind: Some(read_kind),
        } = inner
        {
            if read == path {
                kind = Some(*read_kind);
            }
        }
    };
    walk(expr, &mut visit);
    kind
}

fn walk(expr: &Expr, visit: &mut dyn FnMut(&Expr)) {
    visit(expr);
    match expr {
        Expr::Negate(inner) | Expr::Round { value: inner, .. } => walk(inner, visit),
        Expr::Arith { left, right, .. } => {
            walk(left, visit);
            walk(right, visit);
        }
        Expr::Call { arguments, .. } => {
            for argument in arguments {
                walk(argument, visit);
            }
        }
        _ => {}
    }
}

/// `value` of `kind` moved by the `ordinal`th step, or `None` where the kind is not moved.
fn moved(value: &Node, kind: Kind, ordinal: i64) -> Option<Node> {
    match (kind, value) {
        (Kind::Timestamp, Node::Text(text)) => {
            let at = Rfc3339Instant::parse_rfc3339(text)?;
            Some(Node::Text(at.plus_elapsed(ordinal * 61 * 60)?.to_rfc3339()))
        }
        (Kind::Integer | Kind::Decimal, Node::Number(number)) => {
            let units = number.as_i64()?.checked_add(ordinal.checked_mul(7)?)?;
            Some(Node::Number(Number::from(units)))
        }
        (Kind::Duration, Node::Text(text)) => {
            let seconds = ess_primitives::expr::parse_elapsed(text)?;
            Some(Node::Text(format!(
                "PT{}S",
                seconds.checked_add(ordinal * 61)?
            )))
        }
        _ => None,
    }
}

fn zero(kind: Kind) -> Node {
    number(0, kind).unwrap_or(Node::Null)
}

/// `value` as a number of `kind`.
fn number(value: i64, kind: Kind) -> Option<Node> {
    matches!(kind, Kind::Integer | Kind::Decimal).then(|| Node::Number(Number::from(value)))
}
