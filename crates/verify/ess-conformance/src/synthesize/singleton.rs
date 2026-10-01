//! An identity whose type has one value: the one row of a singleton entity (beyond10x/ess#287).
//!
//! A specification says an entity has exactly one row by giving its identity a type with one
//! value. There is no `singleton:` key; the type says it, in any of the forms [`has_one_value`]
//! reads — an enum of one variant, or a newtype whose invariants admit one value (equal to a
//! literal, `in:` one distinct value, an `Integer` range one value wide, a one-character alphabet
//! of pinned `value.count`). Synthesis otherwise draws, for every scenario needing an identity no
//! row carries, a value no other scenario sends. A one-value type has no such value, and asking for
//! one refused every existence branch.
//!
//! So for such an identity each scenario relies on starting from an empty target: the one value is
//! unknown where the scenario arranged no row, new where the scenario creates it, and stored where
//! the scenario created it — and a second creation of it is the `existing_instance:` refusal. A
//! target shared between scenarios is not served by these scenarios, and nothing in the suite says
//! so yet (beyond10x/ess#312). No scenario arranges a second row: the decoys a `when_related:`
//! guard or a `{related: …}` source is witnessed between are left out (there is no other row to
//! answer instead), rows of other entities that reference the one row are arranged against the row
//! the scenario already made, and a scenario acting on the row is sent with the row arranged by one
//! caller and acted on by the other (`caller.rs`).
//!
//! [`withdraw_second_creations`] holds the finished suite to that rule, whatever family built a
//! scenario: a repeated creation of the one row is merged into the first where one row stands for
//! both, and the scenario is otherwise withdrawn and refused, as is one expecting the row missing
//! after it created it.

use std::collections::BTreeMap;

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedOutcome, ResolvedRelatedTest, ResolvedTypeRef,
};
use ess_domain::types::Primitive;
use ess_primitives::facts::FactValue;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use crate::scenario::{ConformanceScenario, InstanceName, ScenarioId, ScenarioStep};
use crate::witness::WitnessGap;

use super::{Note, Refusal, RefusalCause, Synthesis};

/// Why a scenario has no second row of a singleton entity.
const TWO_ROWS: &str =
    "has one value, so it names one row, and this scenario creates two in one run";

/// Why a scenario cannot find the one row of a singleton entity missing once it created it.
const MISSING_AFTER: &str =
    "has one value, so it names one row, and this scenario expects it missing after creating it";

/// `true` where `type_ref` has exactly one value, through any chain of newtypes: an enum of one
/// variant, or a newtype whose invariants, read together, admit one `value` — equal to a literal,
/// `in:` a list of one distinct value, a number range one value wide (an `Integer` between `>= 1`
/// and `<= 1`, or `> 0` and `< 2`), or a text over a one-character alphabet whose `value.count` is
/// pinned the same ways.
pub(super) fn has_one_value(ir: &EssIr, type_ref: &ResolvedTypeRef) -> bool {
    let mut held: Vec<&Predicate> = Vec::new();
    let mut alphabet: Option<&str> = None;
    let mut at = type_ref;
    while let ResolvedTypeRef::Declared { name } = at {
        match &ir.named_type(name).body {
            ResolvedBody::Enum { variants } => {
                return variants.len() == 1 || pinned(&held, &["value"], false);
            }
            ResolvedBody::Newtype {
                of,
                invariants,
                alphabet: declared,
                ..
            } => {
                for invariant in invariants {
                    conjuncts(&invariant.predicate, &mut held);
                }
                // The outermost layer's alphabet is the effective one.
                alphabet = alphabet.or(declared.as_deref());
                at = of;
            }
            ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => return false,
        }
    }
    let ResolvedTypeRef::Primitive { name } = at else {
        return false;
    };
    let integral = *name == Primitive::Integer;
    let one_letter = alphabet.is_some_and(|alphabet| {
        let mut letters: Vec<char> = alphabet.chars().collect();
        letters.sort_unstable();
        letters.dedup();
        letters.len() == 1
    });
    pinned(&held, &["value"], integral)
        || (*name == Primitive::String && one_letter && pinned(&held, &["value", "count"], true))
}

/// Every conjunct of `predicate`, with nested `all` flattened, pushed onto `found`.
fn conjuncts<'p>(predicate: &'p Predicate, found: &mut Vec<&'p Predicate>) {
    match predicate {
        Predicate::All(children) => {
            for child in children {
                conjuncts(child, found);
            }
        }
        other => found.push(other),
    }
}

/// `true` where the conjuncts `held` admit one value at `path`: equal to a literal, `in:` a list of
/// one distinct value, or between a lower and an upper bound one value apart — for an `integral`
/// path, the integers the strict or inclusive bounds leave; otherwise two inclusive bounds at the
/// same number.
fn pinned(held: &[&Predicate], path: &[&str], integral: bool) -> bool {
    let at = |operand: &Operand| matches!(operand, Operand::Fact(fact) if fact.segments() == path);
    let mut lower: Option<(f64, bool)> = None;
    let mut upper: Option<(f64, bool)> = None;
    for predicate in held {
        match predicate {
            Predicate::AnyOf { path: fact, values } if fact.segments() == path => {
                let mut distinct: Vec<&FactValue> = Vec::new();
                for value in values {
                    if !distinct.contains(&value) {
                        distinct.push(value);
                    }
                }
                if distinct.len() == 1 {
                    return true;
                }
            }
            Predicate::Compare { left, op, right } => {
                // Read as `path <op> literal`, flipping a literal written first.
                let (op, literal) = match (left, right) {
                    (fact, Operand::Literal(literal)) if at(fact) => (*op, literal),
                    (Operand::Literal(literal), fact) if at(fact) => (flipped(*op), literal),
                    _ => continue,
                };
                if op == CompareOp::Eq {
                    return true;
                }
                let Some(number) = literal.as_number().map(ess_primitives::facts::Number::get)
                else {
                    continue;
                };
                // The tighter of two bounds on one side: the nearer, and at one number the strict.
                let tighter =
                    |bound: Option<(f64, bool)>, new: (f64, bool), above: bool| match bound {
                        Some((old, inclusive))
                            if (above && old > new.0)
                                || (!above && old < new.0)
                                || (old.total_cmp(&new.0).is_eq() && !inclusive) =>
                        {
                            bound
                        }
                        _ => Some(new),
                    };
                match op {
                    CompareOp::Ge => lower = tighter(lower, (number, true), true),
                    CompareOp::Gt => lower = tighter(lower, (number, false), true),
                    CompareOp::Le => upper = tighter(upper, (number, true), false),
                    CompareOp::Lt => upper = tighter(upper, (number, false), false),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    let (Some((low, low_inclusive)), Some((high, high_inclusive))) = (lower, upper) else {
        return false;
    };
    if integral {
        let least = if low_inclusive {
            low.ceil()
        } else {
            low.floor() + 1.0
        };
        let most = if high_inclusive {
            high.floor()
        } else {
            high.ceil() - 1.0
        };
        least.total_cmp(&most).is_eq()
    } else {
        low_inclusive && high_inclusive && low.total_cmp(&high).is_eq()
    }
}

/// The operator that says the same with its operands swapped.
fn flipped(op: CompareOp) -> CompareOp {
    match op {
        CompareOp::Lt => CompareOp::Gt,
        CompareOp::Le => CompareOp::Ge,
        CompareOp::Gt => CompareOp::Lt,
        CompareOp::Ge => CompareOp::Le,
        other => other,
    }
}

/// `true` where `field` of `command`'s input names the one row of a singleton entity.
pub(super) fn names_the_one_row(ir: &EssIr, command: &ResolvedCommand, field: &str) -> bool {
    command
        .input
        .iter()
        .find(|input| input.name == field)
        .is_some_and(|input| has_one_value(ir, &input.type_ref))
}

/// `true` where `entity`'s identity has one value, so it has no row beside the one.
pub(super) fn is_singleton(ir: &EssIr, entity: &EntityHandle) -> bool {
    has_one_value(ir, &ir.entity(entity).identity.type_ref)
}

/// Why a scenario that needs two rows of a singleton entity in one run has none: the input and its
/// type.
pub(super) fn second_row(command: &ResolvedCommand, field: &str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path: field.to_owned(),
        type_ref: command
            .input
            .iter()
            .find(|input| input.name == field)
            .map(|input| input.type_ref.to_string())
            .unwrap_or_default(),
        reason: TWO_ROWS,
    })
}

/// Every scenario of `synthesis` held to the one row of each singleton entity: a second creation of
/// it merged into the first where that leaves the scenario meaning what it did, and otherwise the
/// scenario withdrawn and refused by name, with any note about it.
///
/// A target holding one row answers a second creation with its existing-instance refusal, so a
/// scenario expecting two fails the very model it was synthesized from. Rows of another entity
/// that each reference the one row — the decoy and companion jobs a guard on a job is witnessed
/// between, each started against "its" switch — are arranged against the row the scenario already
/// made: the repeated creation, sent exactly as the first was, is dropped, and every step naming
/// the instance it captured names the first one. Where the repeat is sent otherwise, the row was
/// moved or changed after it was made, or something asserts the repeat itself, no single row
/// stands for both, and the scenario is refused.
pub(super) fn withdraw_second_creations(ir: &EssIr, synthesis: &mut Synthesis) {
    let mut withdrawn: Vec<(ScenarioId, RefusalCause)> = Vec::new();
    for (id, scenario) in &mut synthesis.suite.scenarios {
        if let Err(cause) = merge_second_creations(ir, scenario) {
            withdrawn.push((id.clone(), cause));
        }
    }
    for (id, cause) in withdrawn {
        synthesis.suite.scenarios.remove(&id);
        synthesis.notes.retain(|note| match note {
            Note::PartialObservation { scenario, .. }
            | Note::UnseparatedSources { scenario, .. }
            | Note::UnwitnessedOverlap { scenario, .. }
            | Note::UnswappedCallers { scenario, .. }
            | Note::UnaccompaniedRelatedCopy { scenario, .. } => *scenario != id,
            _ => true,
        });
        synthesis.refusals.push(Refusal::about(&id, cause));
    }
}

/// `scenario` with every repeated creation of a singleton row merged into the first; the cause
/// where one cannot be.
fn merge_second_creations(
    ir: &EssIr,
    scenario: &mut ConformanceScenario,
) -> Result<(), RefusalCause> {
    while let Some(repeat) = second_creation(ir, &scenario.steps)? {
        scenario.steps.drain(repeat.start..repeat.end);
        if let Some((again, first)) = repeat.renamed {
            let tail = scenario.steps.split_off(repeat.start);
            let mut value = serde_json::to_value(&tail).map_err(|_| repeat.cause.clone())?;
            rename(&mut value, again.as_str(), first.as_str());
            let tail: Vec<ScenarioStep> =
                serde_json::from_value(value).map_err(|_| repeat.cause.clone())?;
            scenario.steps.extend(tail);
        }
    }
    Ok(())
}

/// A repeated creation of a singleton row that [`merge_second_creations`] drops: the steps
/// `start..end` sending and capturing it, and the instance name it captured with the first
/// creation's, where it captured one.
struct Repeat {
    start: usize,
    end: usize,
    renamed: Option<(InstanceName, InstanceName)>,
    cause: RefusalCause,
}

/// The one row of a singleton entity as a scenario made it: the step that sent its creation, the
/// instance it was captured as, and whether anything moved or changed it since.
struct Made<'s> {
    sent: &'s ScenarioStep,
    captured: Option<InstanceName>,
    touched: bool,
}

/// The first repeated creation of a singleton row in `steps` that can be merged into the first;
/// `None` where there is none; the cause where one is found that cannot be, or where a
/// `when_related: {exists: false}` branch is expected after the row was made.
fn second_creation(ir: &EssIr, steps: &[ScenarioStep]) -> Result<Option<Repeat>, RefusalCause> {
    let cause = |entity: &EntityHandle, reason: &'static str| {
        let identity = &ir.entity(entity).identity;
        RefusalCause::NoWitness(WitnessGap {
            path: identity.name.clone(),
            type_ref: identity.type_ref.to_string(),
            reason,
        })
    };
    let captured_at = |at: usize, entity: &EntityHandle| match steps.get(at) {
        Some(ScenarioStep::CaptureInstance {
            instance,
            entity: captured,
            ..
        }) if captured.to_string() == ir.entity(entity).name.to_string() => Some(instance.clone()),
        _ => None,
    };
    let mut made: BTreeMap<&EntityHandle, Made<'_>> = BTreeMap::new();
    for (at, step) in steps.iter().enumerate() {
        let Some(declared) = expected(ir, step) else {
            continue;
        };
        if let ResolvedCondition::Related {
            test: ResolvedRelatedTest::Absent,
            entity,
            ..
        } = &declared.condition
        {
            if made.contains_key(entity) {
                return Err(cause(entity, MISSING_AFTER));
            }
        }
        let Some(subject) = declared
            .subject
            .as_ref()
            .filter(|subject| is_singleton(ir, &subject.entity))
        else {
            continue;
        };
        let entity = &subject.entity;
        match subject.effect {
            ResolvedEffect::Creates => {
                let sent = at.checked_sub(1).and_then(|before| steps.get(before));
                let Some(first) = made.get(entity) else {
                    if let Some(sent) = sent {
                        made.insert(
                            entity,
                            Made {
                                sent,
                                captured: captured_at(at + 1, entity),
                                touched: false,
                            },
                        );
                    }
                    continue;
                };
                let refused = cause(entity, TWO_ROWS);
                let again = captured_at(at + 1, entity);
                let end = at + 1 + usize::from(again.is_some());
                let changed_later = steps[end..].iter().any(|later| {
                    expected(ir, later)
                        .and_then(|declared| declared.subject.as_ref())
                        .is_some_and(|later| {
                            later.entity == *entity && later.effect != ResolvedEffect::Creates
                        })
                });
                let asserted = steps.get(end).is_some_and(|next| {
                    !matches!(
                        next,
                        ScenarioStep::ExecuteCommand { .. }
                            | ScenarioStep::ExecuteCommandWithoutInput { .. }
                    )
                });
                let renamed = match (again, &first.captured) {
                    (Some(again), Some(first)) => Some((again, first.clone())),
                    (None, _) => None,
                    (Some(_), None) => return Err(refused),
                };
                if first.touched || changed_later || asserted || sent != Some(first.sent) {
                    return Err(refused);
                }
                return Ok(Some(Repeat {
                    start: at - 1,
                    end,
                    renamed,
                    cause: refused,
                }));
            }
            ResolvedEffect::Deletes => {
                made.remove(entity);
            }
            _ => {
                if let Some(row) = made.get_mut(entity) {
                    row.touched = true;
                }
            }
        }
    }
    Ok(None)
}

/// The declared outcome an `ExpectOutcome` step requires.
fn expected<'i>(ir: &'i EssIr, step: &ScenarioStep) -> Option<&'i ResolvedOutcome> {
    let ScenarioStep::ExpectOutcome { outcome } = step else {
        return None;
    };
    ir.commands()
        .get(outcome.command.name())?
        .outcomes
        .iter()
        .find(|declared| declared.name == outcome.outcome)
}

/// Every instance name `from` in a serialized run replaced by `to`.
fn rename(value: &mut serde_json::Value, from: &str, to: &str) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, inner) in map.iter_mut() {
                match inner {
                    serde_json::Value::String(name) if key == "instance" && name == from => {
                        to.clone_into(name);
                    }
                    _ => rename(inner, from, to),
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                rename(item, from, to);
            }
        }
        _ => {}
    }
}
