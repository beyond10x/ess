//! Bounded closed enum/Boolean truth tables shared by validation and concrete witnesses.
use crate::expression::{check_predicate, resolve_path, ScalarKind, TypeEnvironment};
use ess_primitives::{
    facts::{FactPath, FactStore, FactValue},
    predicate::{CompareOp, Operand, Predicate, Truth},
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Maximum joint assignments admitted as a complete domain.
pub const MAX_ASSIGNMENTS: usize = 64;
/// Maximum AST nodes in all guards, independently of their nesting or domain size.
pub const MAX_PREDICATE_NODES: usize = 128;

/// Why the finite proof declined to enumerate the guards' domain (beyond10x/ess#426).
///
/// Rendered into the refusal of a command that has no default to answer what the proof cannot,
/// so its author is told which of the three it is, rather than all three at once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decline {
    /// A guard reads a fact whose values are not a closed set: an `Integer`, a text, an ordering
    /// comparison such as `weight_kg > 20`.
    Open(FactPath),
    /// A guard is outside what the proof reads: an optional or collection path, a construct
    /// other than equality and membership, more than [`MAX_PREDICATE_NODES`] nodes, or a guard the
    /// checker refuses.
    Unsupported,
    /// The declared values cross into this many joint assignments, more than
    /// [`MAX_ASSIGNMENTS`].
    Exceeds(usize),
}

impl fmt::Display for Decline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(path) => write!(f, "`{path}` has no closed set of values"),
            Self::Unsupported => f.write_str(
                "a guard reads an optional or collection path, or a construct other than \
                 equality and membership",
            ),
            Self::Exceeds(count) => {
                write!(f, "{count} joint assignments exceed {MAX_ASSIGNMENTS}")
            }
        }
    }
}

/// The joint assignment count of two sides, or why either side declined: a side that is not
/// finite is the reason over a side that is merely large.
fn crossed(left: Result<usize, Decline>, right: Result<usize, Decline>) -> Result<usize, Decline> {
    let count = |side: &Result<usize, Decline>| match side {
        Ok(count) | Err(Decline::Exceeds(count)) => Some(*count),
        Err(_) => None,
    };
    match (count(&left), count(&right)) {
        (Some(left), Some(right)) => {
            let product = left.saturating_mul(right);
            if product > MAX_ASSIGNMENTS {
                Err(Decline::Exceeds(product))
            } else {
                Ok(product)
            }
        }
        (None, _) => left,
        (_, None) => right,
    }
}

/// One actual declared assignment and the branches satisfied by the existing evaluator.
#[derive(Debug, Clone)]
pub struct Case {
    /// Values for every fact read by the guards by value, in declared path order.
    pub values: BTreeMap<FactPath, FactValue>,
    /// Whether a value is present, for every Optional fact the guards read only through
    /// `defined(…)`: the closed proofs without a default enumerate {absent, present} for such a
    /// path. Empty wherever the proof does not admit presence, as [`analyze`] does not.
    pub presence: BTreeMap<FactPath, bool>,
    /// Guard positions that evaluate true. Unknown never enters a complete table.
    pub selected: Vec<usize>,
}

/// One assignment as the refusals print it, in path order: `path = value` for a fact read by
/// value, `path = absent` or `path = present` for one read only through `defined(…)`.
pub(super) fn assignment_terms(
    values: &BTreeMap<FactPath, FactValue>,
    presence: &BTreeMap<FactPath, bool>,
) -> Vec<String> {
    let mut terms: BTreeMap<&FactPath, String> = values
        .iter()
        .map(|(path, value)| (path, format!("{path} = {value}")))
        .collect();
    for (path, present) in presence {
        let state = if *present { "present" } else { "absent" };
        terms.insert(path, format!("{path} = {state}"));
    }
    terms.into_values().collect()
}

/// The paths a guard set reads: by value, and only through `defined(…)`.
struct Reads {
    valued: BTreeSet<FactPath>,
    presence: BTreeSet<FactPath>,
}

/// Syntactic admission only; callers still need typed analysis to prove coverage.
///
/// A path read only through `defined(…)` is admitted beside the valued ones; [`presence_paths`]
/// names them, since only an Optional path has a presence to enumerate.
pub fn paths(guards: &[&Predicate]) -> Option<BTreeSet<FactPath>> {
    input_paths(guards, false, true, true)
        .ok()
        .map(|reads| reads.valued.into_iter().chain(reads.presence).collect())
}

/// The paths of [`paths`] that every guard reads only through `defined(…)`.
pub fn presence_paths(guards: &[&Predicate]) -> Option<BTreeSet<FactPath>> {
    input_paths(guards, false, true, true)
        .ok()
        .map(|reads| reads.presence)
}

fn input_paths(
    guards: &[&Predicate],
    allow_empty: bool,
    booleans: bool,
    presence: bool,
) -> Result<Reads, Decline> {
    if guards.len() > MAX_PREDICATE_NODES {
        return Err(Decline::Unsupported);
    }
    let mut paths = BTreeSet::new();
    let mut defined = BTreeSet::new();
    let mut pending: Vec<_> = guards.to_vec();
    let mut visited = 0;
    while let Some(predicate) = pending.pop() {
        visited += 1;
        if visited > MAX_PREDICATE_NODES {
            return Err(Decline::Unsupported);
        }
        match predicate {
            Predicate::Always | Predicate::Never => {}
            Predicate::All(children) | Predicate::Any(children) => {
                if children.len() > MAX_PREDICATE_NODES {
                    return Err(Decline::Unsupported);
                }
                pending.extend(children);
            }
            Predicate::Not(child) => pending.push(child),
            Predicate::Truthy(path) if booleans => {
                paths.insert(path.clone());
            }
            Predicate::Defined(path) if presence => {
                defined.insert(path.clone());
            }
            Predicate::Compare {
                left,
                op: CompareOp::Eq | CompareOp::Ne,
                right,
                ..
            } => match (left, right) {
                (Operand::Fact(path), Operand::Literal(FactValue::Text(_)))
                | (Operand::Literal(FactValue::Text(_)), Operand::Fact(path)) => {
                    paths.insert(path.clone());
                }
                (Operand::Fact(path), Operand::Literal(FactValue::Bool(_)))
                | (Operand::Literal(FactValue::Bool(_)), Operand::Fact(path))
                    if booleans =>
                {
                    paths.insert(path.clone());
                }
                (Operand::Fact(path), _) | (_, Operand::Fact(path)) => {
                    return Err(Decline::Open(path.clone()))
                }
                _ => return Err(Decline::Unsupported),
            },
            Predicate::Compare {
                left: Operand::Fact(path),
                ..
            }
            | Predicate::Compare {
                right: Operand::Fact(path),
                ..
            } => return Err(Decline::Open(path.clone())),
            Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
                if values.len() > MAX_PREDICATE_NODES {
                    return Err(Decline::Unsupported);
                }
                if values.iter().any(|v| {
                    !(matches!(v, FactValue::Text(_))
                        || booleans && matches!(v, FactValue::Bool(_)))
                }) {
                    return Err(Decline::Open(path.clone()));
                }
                paths.insert(path.clone());
            }
            _ => return Err(Decline::Unsupported),
        }
    }
    // A path read by value as well has values, not just a presence: the proof keeps declining it.
    if !paths.is_disjoint(&defined) {
        return Err(Decline::Unsupported);
    }
    if paths.is_empty() && defined.is_empty() && !allow_empty {
        Err(Decline::Unsupported)
    } else {
        Ok(Reads {
            valued: paths,
            presence: defined,
        })
    }
}

/// Enumerate the entire supported domain, or decline to make a finite proof.
///
/// Optional paths and open domains are deliberately unsupported — a `defined(…)` read included,
/// which only the closed partition proofs without a default enumerate. Every guard is checked
/// against the same existing type authority that admits ordinary predicates.
pub fn analyze<E: TypeEnvironment>(environment: &E, guards: &[&Predicate]) -> Option<Vec<Case>> {
    analyze_inputs(environment, guards, false, true, false).ok()
}

/// [`analyze`] with presence admitted, or why it declines (beyond10x/ess#426): the input partition
/// of a command without a default.
pub(super) fn input_coverage<E: TypeEnvironment>(
    environment: &E,
    guards: &[&Predicate],
) -> Result<Vec<Case>, Decline> {
    analyze_inputs(environment, guards, false, true, true)
}

/// What one path takes in one assignment.
#[derive(Debug, Clone)]
enum Slot {
    /// A declared value of a path read by value.
    Value(FactValue),
    /// Whether a path read only through `defined(…)` holds a value.
    Presence(bool),
}

/// The value a present path holds in the prover's fact store. Only `defined(…)` reads such a path,
/// so which value it is does not matter, only that one is set.
const PRESENT: FactValue = FactValue::Bool(true);

/// The domain of a path read only through `defined(…)`: {absent, present} where the path is
/// Optional, and a decline where it is not, since a required path is always present and a
/// collection has no single value to be absent.
fn presence_domain<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
) -> Result<Vec<Slot>, Decline> {
    let resolved = resolve_path(environment, path, "finite outcome coverage")
        .map_err(|_| Decline::Unsupported)?;
    if !resolved.optional || resolved.access.collection {
        return Err(Decline::Unsupported);
    }
    Ok(vec![Slot::Presence(false), Slot::Presence(true)])
}

/// The declared values of a path read by value: a required enum's variants in declaration order,
/// or `[false, true]` for a required Boolean where `booleans`; a decline for an Optional or
/// collection path, and for an open one.
fn value_domain<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
    booleans: bool,
) -> Result<Vec<Slot>, Decline> {
    let resolved = resolve_path(environment, path, "finite outcome coverage")
        .map_err(|_| Decline::Unsupported)?;
    if resolved.optional || resolved.access.collection {
        return Err(Decline::Unsupported);
    }
    let variants: Vec<FactValue> = if booleans && resolved.scalar == Some(ScalarKind::Bool) {
        vec![FactValue::Bool(false), FactValue::Bool(true)]
    } else {
        resolved
            .variants
            .ok_or_else(|| Decline::Open(path.clone()))?
            .into_iter()
            .map(FactValue::Text)
            .collect()
    };
    if variants.is_empty() {
        return Err(Decline::Unsupported);
    }
    Ok(variants.into_iter().map(Slot::Value).collect())
}

fn analyze_inputs<E: TypeEnvironment>(
    environment: &E,
    guards: &[&Predicate],
    allow_empty: bool,
    booleans: bool,
    presence: bool,
) -> Result<Vec<Case>, Decline> {
    let reads = input_paths(guards, allow_empty, booleans, presence)?;
    // Truthiness is newly admitted only for Boolean facts. The ordinary evaluator also has
    // text/number truthiness; teaching the finite proof those fragments is separate work.
    let mut pending = guards.to_vec();
    while let Some(guard) = pending.pop() {
        match guard {
            Predicate::All(children) | Predicate::Any(children) => pending.extend(children),
            Predicate::Not(child) => pending.push(child),
            Predicate::Truthy(path)
                if resolve_path(environment, path, "finite outcome coverage")
                    .map_err(|_| Decline::Unsupported)?
                    .scalar
                    != Some(ScalarKind::Bool) =>
            {
                return Err(Decline::Open(path.clone()));
            }
            _ => {}
        }
    }
    for guard in guards {
        if !check_predicate(environment, guard, "finite outcome coverage")
            .errors
            .is_empty()
        {
            return Err(Decline::Unsupported);
        }
    }
    // Every path's values first, so a domain past the cap is declined with its whole count.
    let mut domains = Vec::new();
    for path in reads.valued {
        let slots = value_domain(environment, &path, booleans)?;
        domains.push((path, slots));
    }
    for path in reads.presence {
        let slots = presence_domain(environment, &path)?;
        domains.push((path, slots));
    }
    let count = domains
        .iter()
        .fold(1_usize, |count, (_, slots): &(FactPath, Vec<Slot>)| {
            count.saturating_mul(slots.len())
        });
    if count > MAX_ASSIGNMENTS {
        return Err(Decline::Exceeds(count));
    }
    let mut assignments = vec![BTreeMap::new()];
    for (path, slots) in domains {
        let mut next = Vec::new();
        for assignment in assignments {
            for slot in &slots {
                let mut row = assignment.clone();
                row.insert(path.clone(), slot.clone());
                next.push(row);
            }
        }
        assignments = next;
    }
    let mut result = Vec::new();
    for row in assignments {
        let mut facts = FactStore::new();
        let mut values = BTreeMap::new();
        let mut presence = BTreeMap::new();
        for (path, slot) in row {
            match slot {
                Slot::Value(value) => {
                    facts.set(path.clone(), value.clone());
                    values.insert(path, value);
                }
                Slot::Presence(present) => {
                    if present {
                        facts.set(path.clone(), PRESENT);
                    }
                    presence.insert(path, present);
                }
            }
        }
        let mut selected = Vec::new();
        for (index, guard) in guards.iter().enumerate() {
            match guard.evaluate(&facts) {
                Truth::True => selected.push(index),
                Truth::False => {}
                Truth::Unknown => return Err(Decline::Unsupported),
            }
        }
        result.push(Case {
            values,
            presence,
            selected,
        });
    }
    Ok(result)
}

/// One guarded branch, keeping held-state authority separate from the input namespace.
///
/// The held side is the **set** of states the branch admits rather than one state, because two
/// conditions now select on it and only one of them names a state: a literal `when_subject_state:`
/// contributes a set of one, and `when_state_changes:` contributes the side of its move's own
/// `from` set that the answer picks. Generalising here rather than teaching this prover about
/// either condition keeps the proof one question — *does this branch admit this state* — asked the
/// same way for both.
#[derive(Debug, Clone)]
pub struct StateGuard<'a> {
    /// The held states this branch admits, or any existing declared state.
    pub states: Option<BTreeSet<crate::entity::StateName>>,
    /// The ordinary input guard, or any admitted input.
    pub predicate: Option<&'a Predicate>,
}

/// One complete declared held-state/input assignment.
#[derive(Debug, Clone)]
pub struct StateCase {
    /// The observed lifecycle state before command selection.
    pub state: crate::entity::StateName,
    /// The input assignment and the state-aware selected guard positions.
    pub input: Case,
}

/// The existing finite input proof crossed with declared held states, under one joint bound.
/// No synthetic input root is introduced; a real input named `subject` keeps its meaning.
pub fn analyze_with_states<E: TypeEnvironment>(
    environment: &E,
    guards: &[StateGuard<'_>],
    states: &BTreeSet<crate::entity::StateName>,
) -> Option<Vec<StateCase>> {
    state_inputs(environment, guards, states, true).ok()
}

/// Preserve the existing default-bearing state validator's enum-only proof policy.
pub(super) fn analyze_enum_states<E: TypeEnvironment>(
    environment: &E,
    guards: &[StateGuard<'_>],
    states: &BTreeSet<crate::entity::StateName>,
) -> Option<Vec<StateCase>> {
    state_inputs(environment, guards, states, false).ok()
}

fn state_inputs<E: TypeEnvironment>(
    environment: &E,
    guards: &[StateGuard<'_>],
    states: &BTreeSet<crate::entity::StateName>,
    booleans: bool,
) -> Result<Vec<StateCase>, Decline> {
    if states.is_empty() {
        return Err(Decline::Unsupported);
    }
    let always = Predicate::Always;
    let inputs: Vec<_> = guards
        .iter()
        .map(|guard| guard.predicate.unwrap_or(&always))
        .collect();
    let cases = analyze_inputs(environment, &inputs, true, booleans, false);
    crossed(
        cases.as_ref().map(Vec::len).map_err(Clone::clone),
        Ok(states.len()),
    )?;
    let cases = cases?;
    let mut result = Vec::new();
    for state in states {
        for case in &cases {
            let mut input = case.clone();
            input.selected.retain(|index| {
                guards[*index]
                    .states
                    .as_ref()
                    .is_none_or(|admitted| admitted.contains(state))
            });
            result.push(StateCase {
                state: state.clone(),
                input,
            });
        }
    }
    Ok(result)
}

/// One guarded branch of a command that reads the subject's stored fields (ess#75).
///
/// The held side of [`StateGuard`] generalised from one lifecycle state to an assignment over the
/// stored fields the guards name. The two namespaces stay apart — each side is analysed in its own
/// environment — so an input field and a stored field of one name are two facts, as they are when
/// the command runs.
#[derive(Debug, Clone)]
pub struct FieldGuard<'a> {
    /// What the branch requires of the stored fields, or any stored row.
    pub fields: Option<&'a Predicate>,
    /// The ordinary input guard, or any admitted input.
    pub input: Option<&'a Predicate>,
}

/// One complete joint assignment: stored fields crossed with input.
#[derive(Debug, Clone)]
pub struct FieldCase {
    /// Values for every stored field the guards read.
    pub fields: BTreeMap<FactPath, FactValue>,
    /// Values for every input field the guards read.
    pub input: BTreeMap<FactPath, FactValue>,
    /// Whether a value is present, for every Optional stored field the guards read only through
    /// `defined(…)` ([`Case::presence`]).
    pub fields_presence: BTreeMap<FactPath, bool>,
    /// Whether a value is present, for every Optional input field the guards read only through
    /// `defined(…)` ([`Case::presence`]).
    pub input_presence: BTreeMap<FactPath, bool>,
    /// Guard positions whose stored and input sides both evaluate true.
    pub selected: Vec<usize>,
}

/// The finite input proof run over the stored fields and over the input, crossed under one bound.
///
/// Declines exactly where [`analyze`] declines on either side — an open domain, an optional or
/// collection path read by value, a guard the checker refuses — and where the product exceeds
/// [`MAX_ASSIGNMENTS`]. An Optional path read only through `defined(…)` is enumerated as absent and
/// present. A side no guard reads contributes one empty assignment.
pub fn analyze_with_fields<F: TypeEnvironment, I: TypeEnvironment>(
    fields: &F,
    input: &I,
    guards: &[FieldGuard<'_>],
) -> Option<Vec<FieldCase>> {
    field_inputs(fields, input, guards, true).ok()
}

/// Preserve the existing default-bearing stored/related validator's enum-only proof policy.
pub(super) fn analyze_enum_fields<F: TypeEnvironment, I: TypeEnvironment>(
    fields: &F,
    input: &I,
    guards: &[FieldGuard<'_>],
) -> Option<Vec<FieldCase>> {
    field_inputs(fields, input, guards, false).ok()
}

/// [`analyze_with_fields`] where `booleans`, the enum-only proof otherwise, or why either declines
/// (beyond10x/ess#426).
pub(super) fn field_coverage<F: TypeEnvironment, I: TypeEnvironment>(
    fields: &F,
    input: &I,
    guards: &[FieldGuard<'_>],
    booleans: bool,
) -> Result<Vec<FieldCase>, Decline> {
    field_inputs(fields, input, guards, booleans)
}

fn field_inputs<F: TypeEnvironment, I: TypeEnvironment>(
    fields: &F,
    input: &I,
    guards: &[FieldGuard<'_>],
    booleans: bool,
) -> Result<Vec<FieldCase>, Decline> {
    let always = Predicate::Always;
    let stored: Vec<_> = guards
        .iter()
        .map(|guard| guard.fields.unwrap_or(&always))
        .collect();
    let supplied: Vec<_> = guards
        .iter()
        .map(|guard| guard.input.unwrap_or(&always))
        .collect();
    let stored = analyze_inputs(fields, &stored, true, booleans, booleans);
    let supplied = analyze_inputs(input, &supplied, true, booleans, booleans);
    crossed(
        stored.as_ref().map(Vec::len).map_err(Clone::clone),
        supplied.as_ref().map(Vec::len).map_err(Clone::clone),
    )?;
    let (stored, supplied) = (stored?, supplied?);
    let mut result = Vec::new();
    for row in &stored {
        for sent in &supplied {
            result.push(FieldCase {
                fields: row.values.clone(),
                input: sent.values.clone(),
                fields_presence: row.presence.clone(),
                input_presence: sent.presence.clone(),
                selected: row
                    .selected
                    .iter()
                    .copied()
                    .filter(|index| sent.selected.contains(index))
                    .collect(),
            });
        }
    }
    Ok(result)
}
