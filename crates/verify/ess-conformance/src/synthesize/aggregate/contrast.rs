//! The contrast a conditional aggregate measure needs (beyond10x/ess#363,
//! `docs/design/conditional-aggregate-measures.md`, "Synthesis and target obligations").
//!
//! A measure that reads only the rows its `where:` admits is witnessed only by a group holding
//! rows the condition admits *and* rows it refuses, arranged so that the measure's exact value
//! tells its condition from no condition at all and from the inverted one. Every field such a
//! condition reads that the creating command sets from its input — and the lifecycle state, where
//! the view does not group by it — is a dimension the arrangement varies across group A's rows.
//! The values each dimension is varied over are its type's own (`false`/`true`, an enum's
//! variants, the states, the initial state first) or, for an open type, two ladder values and the
//! literals the condition compares it with, one either side of a number. A field a measure
//! aggregates is varied too where a condition reads it (`sum: cents, where: cents > 5`): each row
//! keeps the value the pattern gave it or takes one of those literals.
//!
//! Group B is the decoy: its rows take the first values that make every condition refuse them, so
//! the scenario always asserts a group whose conditioned measures select nothing — which a target
//! that drops such a group, or applies a condition to the whole view, cannot report. Where no
//! values do, the view is refused by name. Every other row holds each dimension's first value.
//! A conditioned `avg` keeps the rounding control: its selected values must have a mean that
//! rounding and truncation tell apart.
//!
//! The patterns are tried in a fixed order, at most [`PATTERNS`] of them; the first that makes
//! every conditioned measure decisive — and, where one exists, that also keeps a swap of two
//! measures' conditions and the loss of any branch of a composite condition observable — is
//! arranged. The arrangement is then read back: what the rows actually hold decides the expected
//! values and is checked again ([`decisive`]), so a move or a guard that changes what was chosen
//! is a named refusal, never an assertion that a dropped condition also passes.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{ResolvedAggregate, ResolvedTypeRef};
use ess_domain::entity::{EntitySpec, StateName};
use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate};

use super::super::{row_truth, Determined};
use super::{held, kind, leaf, Arranged, Key, Ladder, Plan, Row};
use crate::aggregate::{evaluate, evaluate_skipping_absent, ValueKind};
use crate::scenario::ScenarioValue;

/// The most row patterns one search tries.
const PATTERNS: usize = 1 << 15;

/// One value the measures' conditions read that the arrangement varies across group A's rows.
#[derive(Debug, Clone)]
pub(super) struct Dimension {
    /// The source field, or `state`.
    field: String,
    /// The values it is varied over; the first is the one every other row holds. `None` keeps
    /// the value the row's pattern already gave the field (a field the view aggregates).
    candidates: Vec<Option<Node>>,
}

/// Which rows of a group one measure reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Selection {
    /// The rows its condition admits: the declared measure.
    Declared,
    /// Every row, as a target that drops the condition reads.
    Dropped,
    /// The rows its condition refuses, as a target that inverts it reads.
    Inverted,
}

/// The conditioned measures of the plan's view, by field.
pub(super) fn conditioned<'p>(plan: &Plan<'p>) -> Vec<(&'p String, &'p ResolvedAggregate)> {
    plan.aggregation
        .functions
        .iter()
        .filter(|(_, aggregate)| aggregate.r#where.is_some())
        .collect()
}

/// A fact path's first segment.
fn root(path: &FactPath) -> String {
    path.segments().first().cloned().unwrap_or_default()
}

/// A literal as the node a row holds.
fn node(value: &FactValue) -> Node {
    match value {
        FactValue::Bool(held) => Node::Bool(*held),
        FactValue::Number(number) => Node::Number(*number),
        FactValue::Text(text) => Node::Text(text.clone()),
    }
}

/// Every source field `predicate` reads, with the literals it compares each with. A path under a
/// quantifier's binder is read of the collection the binder walks: its literals are elements.
fn reads(
    predicate: &Predicate,
    binders: &BTreeMap<String, String>,
    out: &mut BTreeMap<String, Vec<FactValue>>,
) {
    let target = |path: &FactPath| {
        let first = root(path);
        binders.get(&first).cloned().unwrap_or(first)
    };
    let mut note = |path: &FactPath, literals: &[FactValue]| {
        let field = target(path);
        if field == ess_domain::view::ViewSpec::PARAM {
            return;
        }
        out.entry(field)
            .or_default()
            .extend(literals.iter().cloned());
    };
    match predicate {
        Predicate::Always | Predicate::Never => {}
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                reads(child, binders, out);
            }
        }
        Predicate::Not(child) => reads(child, binders, out),
        Predicate::Compare { left, right, .. } => {
            for (side, other) in [(left, right), (right, left)] {
                if let Operand::Fact(path) = side {
                    let literal: Vec<FactValue> = match other {
                        Operand::Literal(value) => vec![value.clone()],
                        Operand::Fact(_) | Operand::Offset(_) => Vec::new(),
                    };
                    note(path, &literal);
                }
                // An offset (ess/22, A2) reads its base fact too.
                if let Operand::Offset(offset) = side {
                    note(&offset.base, &[]);
                }
            }
        }
        Predicate::Truthy(path) | Predicate::Defined(path) => note(path, &[]),
        Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
            note(path, values);
        }
        Predicate::TextMatch { path, value, .. } => note(path, std::slice::from_ref(value)),
        Predicate::FoldMatch { path, values, .. } => note(path, values),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            note(&quantified.over, &[]);
            let mut inner = binders.clone();
            inner.insert(quantified.bind.clone(), target(&quantified.over));
            reads(&quantified.body, &inner, out);
        }
    }
}

/// The values a literal a condition compares with suggests: the literal, and for a whole number
/// the one either side of it, so an ordering comparison has a row on each side.
fn near(literal: &FactValue) -> Vec<Node> {
    match literal {
        FactValue::Number(number) => number.as_i64().map_or_else(
            || vec![Node::Number(*number)],
            |whole| {
                [whole.checked_sub(1), Some(whole), whole.checked_add(1)]
                    .into_iter()
                    .flatten()
                    .map(|value| Node::Number(Number::from(value)))
                    .collect()
            },
        ),
        other => vec![node(other)],
    }
}

/// The values one field is varied over, every one its declared type admits.
fn candidates(
    plan: &Plan<'_>,
    field: &str,
    type_ref: &ResolvedTypeRef,
    literals: &[FactValue],
) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    if let ResolvedTypeRef::List { .. } = type_ref.required() {
        out.push(Node::Seq(Vec::new()));
        out.extend(
            literals
                .iter()
                .map(|literal| Node::Seq(vec![node(literal)])),
        );
    } else {
        let (_, found) = leaf(plan.ir, type_ref);
        let Some(ladder) = Ladder::of(&plan.view.name, field, &found) else {
            return Vec::new();
        };
        if let Some(finite) = ladder.len() {
            out.extend((0..finite).map(|ordinal| ladder.at(ordinal)));
        } else {
            out.extend([ladder.at(0), ladder.at(1)]);
            for literal in literals {
                out.extend(near(literal));
            }
        }
    }
    let mut kept: Vec<Node> = Vec::new();
    for candidate in out {
        if !kept.contains(&candidate)
            && crate::input::validate_typed_value(plan.ir, type_ref, &candidate).is_ok()
        {
            kept.push(candidate);
        }
    }
    kept
}

/// The dimensions the conditions of the plan's view read: every field the creating command sets
/// from its input (`mapped`) that is no key or scope (`keyed`), and the lifecycle state where no
/// key holds it. A field the view aggregates (`inputs`) keeps its pattern value or takes a literal
/// a condition compares it with.
pub(super) fn dimensions(
    plan: &Plan<'_>,
    mapped: &BTreeMap<&str, &str>,
    keyed: &BTreeSet<&str>,
    inputs: &BTreeSet<&str>,
) -> Vec<Dimension> {
    let mut read = BTreeMap::new();
    for (_, aggregate) in conditioned(plan) {
        if let Some(condition) = &aggregate.r#where {
            reads(condition, &BTreeMap::new(), &mut read);
        }
    }
    let mut out = Vec::new();
    for (field, literals) in read {
        if field == EntitySpec::STATE {
            if plan
                .keys
                .iter()
                .any(|(_, key)| matches!(key, Key::State(_)))
            {
                continue;
            }
            let initial = &plan.entity.lifecycle.initial;
            let states: Vec<Option<Node>> = std::iter::once(initial)
                .chain(
                    plan.entity
                        .lifecycle
                        .states
                        .iter()
                        .filter(|state| *state != initial),
                )
                .map(|state| Some(Node::Text(state.to_string())))
                .collect();
            if states.len() > 1 {
                out.push(Dimension {
                    field,
                    candidates: states,
                });
            }
            continue;
        }
        if field == plan.entity.identity.name
            || keyed.contains(field.as_str())
            || !mapped.contains_key(field.as_str())
        {
            continue;
        }
        let Some(declared) = plan.entity.observable_field(&field) else {
            continue;
        };
        let candidates: Vec<Option<Node>> = if inputs.contains(field.as_str()) {
            let mut kept: Vec<Option<Node>> = vec![None];
            for value in literals.iter().flat_map(near) {
                let value = Some(value);
                if !kept.contains(&value)
                    && value.as_ref().is_some_and(|value| {
                        crate::input::validate_typed_value(plan.ir, &declared.type_ref, value)
                            .is_ok()
                    })
                {
                    kept.push(value);
                }
            }
            kept
        } else {
            candidates(plan, &field, &declared.type_ref, &literals)
                .into_iter()
                .map(Some)
                .collect()
        };
        if candidates.len() > 1 {
            out.push(Dimension { field, candidates });
        }
    }
    out
}

/// One row as the search projects it: the values it is to hold and the state it is to rest in.
struct Projected {
    values: BTreeMap<String, Node>,
    state: StateName,
}

impl Projected {
    fn settled(&self, plan: &Plan<'_>) -> BTreeMap<String, Determined> {
        self.values
            .iter()
            .filter_map(|(field, value)| {
                plan.entity.observable_field(field).map(|declared| {
                    (
                        field.clone(),
                        Determined {
                            value: ScenarioValue::literal(value.clone()),
                            type_ref: declared.type_ref,
                        },
                    )
                })
            })
            .collect()
    }

    fn value(&self, field: &str) -> Node {
        if field == EntitySpec::STATE {
            return Node::Text(self.state.to_string());
        }
        self.values.get(field).cloned().unwrap_or(Node::Null)
    }
}

/// One measure's value over the rows `holds` selects of `rows`, where it has one.
fn value_over(
    plan: &Plan<'_>,
    aggregate: &ResolvedAggregate,
    inputs: &[Node],
    holds: &[bool],
    selection: Selection,
) -> Option<Node> {
    let picked: Vec<Node> = inputs
        .iter()
        .zip(holds)
        .filter(|(_, holds)| match selection {
            Selection::Declared => **holds,
            Selection::Dropped => true,
            Selection::Inverted => !**holds,
        })
        .map(|(input, _)| input.clone())
        .collect();
    let kind = aggregate.input.as_ref().map_or(ValueKind::Other, |input| {
        kind(plan.ir, plan.entity, &input.name)
    });
    if aggregate.skip_absent {
        evaluate_skipping_absent(aggregate.function, &picked, kind)
    } else {
        evaluate(aggregate.function, &picked, kind)
    }
}

/// Whether the measure's value over `holds` tells its condition from no condition and from the
/// inverted one, over rows some of which it admits and some of which it refuses.
fn separates(
    plan: &Plan<'_>,
    aggregate: &ResolvedAggregate,
    inputs: &[Node],
    holds: &[bool],
) -> bool {
    if !holds.iter().any(|held| *held) || holds.iter().all(|held| *held) {
        return false;
    }
    let declared = value_over(plan, aggregate, inputs, holds, Selection::Declared);
    let dropped = value_over(plan, aggregate, inputs, holds, Selection::Dropped);
    let inverted = value_over(plan, aggregate, inputs, holds, Selection::Inverted);
    declared.is_some() && declared != dropped && declared != inverted
}

/// Whether a conditioned `avg` over the rows `holds` selects has a mean that rounding and
/// truncation tell apart, as an unconditioned one must (`rounding_is_observable`); every other
/// function passes.
pub(super) fn rounds(aggregate: &ResolvedAggregate, inputs: &[Node], holds: &[bool]) -> bool {
    if aggregate.function != ess_domain::view::AggregateFunction::Avg {
        return true;
    }
    let picked: Vec<Node> = inputs
        .iter()
        .zip(holds)
        .filter(|(_, holds)| **holds)
        .map(|(input, _)| input.clone())
        .collect();
    let picked = if aggregate.skip_absent {
        crate::aggregate::present(&picked)
    } else {
        picked
    };
    crate::aggregate::avg_separates_rounding(&picked) == Some(true)
}

/// The truth of `predicate` for every projected row, or `None` where one is unknown.
fn truths(
    plan: &Plan<'_>,
    predicate: &Predicate,
    rows: &[Projected],
    params: &BTreeMap<String, ScenarioValue>,
) -> Option<Vec<bool>> {
    rows.iter()
        .map(|row| {
            row_truth(
                plan.ir,
                plan.view,
                predicate,
                &row.state,
                &row.settled(plan),
                None,
                params,
            )
            .ok()
        })
        .collect()
}

/// How a pattern fares: every conditioned measure decisive (`hard`), and every swap of two
/// conditions and every lost branch of a composite one observable too (`soft`).
fn judge(
    plan: &Plan<'_>,
    rows: &[Projected],
    params: &BTreeMap<String, ScenarioValue>,
) -> (bool, bool) {
    if let Some(filter) = &plan.view.filter {
        for row in rows {
            let settled = row.settled(plan);
            if row_truth(
                plan.ir, plan.view, filter, &row.state, &settled, None, params,
            ) == Ok(false)
            {
                return (false, false);
            }
        }
    }
    let measures = conditioned(plan);
    let mut soft = true;
    for (_, aggregate) in &measures {
        let Some(condition) = &aggregate.r#where else {
            continue;
        };
        let inputs: Vec<Node> = rows
            .iter()
            .map(|row| {
                aggregate
                    .input
                    .as_ref()
                    .map_or(Node::Null, |input| row.value(&input.name))
            })
            .collect();
        let Some(holds) = truths(plan, condition, rows, params) else {
            return (false, false);
        };
        if !separates(plan, aggregate, &inputs, &holds) || !rounds(aggregate, &inputs, &holds) {
            return (false, false);
        }
        let declared = value_over(plan, aggregate, &inputs, &holds, Selection::Declared);
        // Another measure's condition in its place, and each branch of a composite one alone.
        let mut others: Vec<&Predicate> = measures
            .iter()
            .filter_map(|(_, other)| other.r#where.as_ref())
            .filter(|other| *other != condition)
            .collect();
        if let Predicate::All(children) | Predicate::Any(children) = condition {
            if children.len() > 1 {
                others.extend(children);
            }
        }
        for other in others {
            let Some(swapped) = truths(plan, other, rows, params) else {
                soft = false;
                continue;
            };
            if value_over(plan, aggregate, &inputs, &swapped, Selection::Declared) == declared {
                soft = false;
            }
        }
    }
    (true, soft)
}

/// Chooses the values every dimension takes in each row: in group A the first pattern that makes
/// every conditioned measure decisive, preferring one that also keeps swaps and lost branches
/// observable; everywhere else each dimension's first value.
pub(super) fn arrange(
    plan: &Plan<'_>,
    rows: &mut [Row],
    dimensions: &[Dimension],
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<(), super::super::RefusalCause> {
    let measures = conditioned(plan);
    if measures.is_empty() {
        return Ok(());
    }
    let group: Vec<usize> = (0..rows.len())
        .filter(|index| rows[*index].tuple == 0 && rows[*index].admitted)
        .collect();
    let radix: Vec<usize> = dimensions
        .iter()
        .map(|dimension| dimension.candidates.len())
        .collect();
    let per_row: usize = radix.iter().product();
    let total = u32::try_from(group.len())
        .ok()
        .and_then(|rows| per_row.checked_pow(rows))
        .map_or(PATTERNS, |total| total.min(PATTERNS));
    // Which candidate of each dimension option `option` of one row takes.
    let decode = |mut option: usize| -> Vec<usize> {
        radix
            .iter()
            .map(|size| {
                let chosen = option % size;
                option /= size;
                chosen
            })
            .collect()
    };
    let project = |pattern: usize| -> Vec<(usize, Vec<usize>)> {
        let mut rest = pattern;
        group
            .iter()
            .map(|index| {
                let option = rest % per_row;
                rest /= per_row;
                (*index, decode(option))
            })
            .collect()
    };
    let mut fallback = None;
    let mut found = None;
    for pattern in 0..total {
        let choice = project(pattern);
        let (hard, soft) = judge(plan, &projected(plan, rows, dimensions, &choice), params);
        if hard && soft {
            found = Some(choice);
            break;
        }
        if hard && fallback.is_none() {
            fallback = Some(choice);
        }
    }
    let Some(choice) = found.or(fallback) else {
        let named: Vec<String> = measures
            .iter()
            .map(|(field, _)| format!("`{field}`"))
            .collect();
        return Err(plan.unwitnessed(format!(
            "{}: no arrangement of one group's rows gives the condition of {} rows it admits and \
             rows it refuses that its measure tells apart (and, for an `avg`, a mean rounding and \
             truncation tell apart), so a target dropping or inverting the condition would pass",
            super::super::CONDITION_REASON,
            named.join(", ")
        )));
    };
    let decoy = decoy(plan, rows, dimensions, &decode, per_row, params)?;
    apply(rows, dimensions, choice.into_iter().chain(decoy).collect());
    Ok(())
}

/// Group B's rows, each with the first option of the dimensions that makes every condition refuse
/// it: the group whose conditioned measures select nothing. Refused by name where no option does.
/// An ungrouped view has no second group, and none is arranged.
fn decoy(
    plan: &Plan<'_>,
    rows: &[Row],
    dimensions: &[Dimension],
    decode: &dyn Fn(usize) -> Vec<usize>,
    per_row: usize,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<Vec<(usize, Vec<usize>)>, super::super::RefusalCause> {
    if plan.aggregation.is_ungrouped() {
        return Ok(Vec::new());
    }
    let measures = conditioned(plan);
    let mut out = Vec::new();
    for index in (0..rows.len()).filter(|index| rows[*index].tuple == 1 && rows[*index].admitted) {
        let refused = (0..per_row).find(|option| {
            let choice = [(index, decode(*option))];
            let projected = projected(plan, rows, dimensions, &choice);
            let row = &projected[0];
            let settled = row.settled(plan);
            let truth = |predicate: &Predicate| {
                row_truth(
                    plan.ir, plan.view, predicate, &row.state, &settled, None, params,
                )
            };
            plan.view
                .filter
                .as_ref()
                .is_none_or(|filter| truth(filter) != Ok(false))
                && measures.iter().all(|(_, aggregate)| {
                    aggregate
                        .r#where
                        .as_ref()
                        .is_some_and(|condition| truth(condition) == Ok(false))
                })
        });
        let Some(option) = refused else {
            let named: Vec<String> = measures
                .iter()
                .map(|(field, _)| format!("`{field}`"))
                .collect();
            return Err(plan.unwitnessed(format!(
                "{}: no row of a second group can be arranged that the condition of every one of \
                 {} refuses, so no group whose conditioned measures select nothing is asserted, and \
                 a target dropping such a group or applying a condition to the whole view would pass",
                super::super::CONDITION_REASON,
                named.join(", ")
            )));
        };
        out.push((index, decode(option)));
    }
    Ok(out)
}

/// The rows of group A a pattern `choice` projects: each row's values and state, with every
/// dimension at the candidate the pattern gives it.
fn projected(
    plan: &Plan<'_>,
    rows: &[Row],
    dimensions: &[Dimension],
    choice: &[(usize, Vec<usize>)],
) -> Vec<Projected> {
    choice
        .iter()
        .map(|(index, chosen)| {
            let row = &rows[*index];
            let mut values = row.values.clone();
            let mut state = plan
                .keys
                .iter()
                .zip(&plan.tuples[row.tuple].1)
                .find_map(|((_, key), value)| match (key, value) {
                    (Key::State(_), Node::Text(state)) => StateName::new(state).ok(),
                    _ => None,
                })
                .unwrap_or_else(|| plan.entity.lifecycle.initial.clone());
            for (dimension, at) in dimensions.iter().zip(chosen) {
                let Some(value) = dimension.candidates[*at].clone() else {
                    continue;
                };
                if dimension.field == EntitySpec::STATE {
                    if let Node::Text(text) = &value {
                        if let Ok(named) = StateName::new(text) {
                            state = named;
                        }
                    }
                } else {
                    values.insert(dimension.field.clone(), value);
                }
            }
            Projected { values, state }
        })
        .collect()
}

/// Writes the chosen pattern into the rows: every dimension at its first candidate in every row,
/// then group A's and the decoy's rows at the pattern's.
fn apply(rows: &mut [Row], dimensions: &[Dimension], choice: Vec<(usize, Vec<usize>)>) {
    for row in rows.iter_mut() {
        for dimension in dimensions {
            if let (false, Some(first)) = (
                dimension.field == EntitySpec::STATE,
                dimension.candidates[0].clone(),
            ) {
                row.values.insert(dimension.field.clone(), first);
            }
        }
    }
    for (index, chosen) in choice {
        for (dimension, at) in dimensions.iter().zip(chosen) {
            let Some(value) = dimension.candidates[at].clone() else {
                continue;
            };
            if dimension.field == EntitySpec::STATE {
                if let Node::Text(text) = &value {
                    rows[index].state = StateName::new(text).ok();
                }
            } else {
                rows[index].values.insert(dimension.field.clone(), value);
            }
        }
    }
}

/// Whether `condition` holds of the arranged row `index` for a read sending `params`; a condition
/// nothing decides is refused by name rather than read as false.
pub(super) fn holds(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    index: usize,
    field: &str,
    condition: &Predicate,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<bool, super::super::RefusalCause> {
    let row = &arranged[index].arrangement;
    row_truth(
        plan.ir,
        plan.view,
        condition,
        &row.state,
        &row.settled,
        Some(&row.identity()),
        params,
    )
    .map_err(|unknown| {
        let unknown: Vec<String> = unknown.iter().map(|path| format!("`{path}`")).collect();
        plan.unwitnessed(format!(
            "{}: the condition of `{field}` for row `{}` is unknown: nothing answers {}",
            super::super::CONDITION_REASON,
            arranged[index].row.label,
            unknown.join(", ")
        ))
    })
}

/// The conditioned measures whose exact value over the admitted rows `members` tells their
/// condition from none and from the inverted one, added to `seen`. Answers whether the group is
/// one every condition refuses: rows, and no conditioned measure selecting any of them.
pub(super) fn decisive(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    members: &[usize],
    params: &BTreeMap<String, ScenarioValue>,
    seen: &mut BTreeSet<String>,
) -> Result<bool, super::super::RefusalCause> {
    let mut nothing = !members.is_empty();
    for (field, aggregate) in conditioned(plan) {
        let Some(condition) = &aggregate.r#where else {
            continue;
        };
        let mut inputs = Vec::new();
        let mut truths = Vec::new();
        for index in members {
            inputs.push(match &aggregate.input {
                None => Node::Null,
                Some(input) => {
                    held(plan, &arranged[*index], *index, &input.name).unwrap_or(Node::Null)
                }
            });
            truths.push(holds(plan, arranged, *index, field, condition, params)?);
        }
        if truths.iter().any(|held| *held) {
            nothing = false;
        }
        if separates(plan, aggregate, &inputs, &truths) {
            seen.insert(field.clone());
        }
    }
    Ok(nothing)
}

/// Refuses the view where some conditioned measure was decisive in no group the scenario asserts,
/// or, for a grouped view, where no asserted group is one every condition refuses (`nothing`).
pub(super) fn all_decisive(
    plan: &Plan<'_>,
    seen: &BTreeSet<String>,
    nothing: bool,
) -> Result<(), super::super::RefusalCause> {
    let measures = conditioned(plan);
    let missing: Vec<String> = measures
        .iter()
        .filter(|(field, _)| !seen.contains(*field))
        .map(|(field, _)| format!("`{field}`"))
        .collect();
    if !missing.is_empty() {
        return Err(plan.unwitnessed(format!(
            "{}: the arranged rows give the condition of {} no group in which its measure tells it \
             from no condition and from the inverted one",
            super::super::CONDITION_REASON,
            missing.join(", ")
        )));
    }
    if !measures.is_empty() && !plan.aggregation.is_ungrouped() && !nothing {
        return Err(plan.unwitnessed(format!(
            "{}: the arranged rows leave no asserted group whose conditioned measures select \
             nothing",
            super::super::CONDITION_REASON
        )));
    }
    Ok(())
}
