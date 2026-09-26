//! Bounded closed-enum truth tables shared by validation and concrete witnesses.
use crate::expression::{check_predicate, resolve_path, TypeEnvironment};
use ess_primitives::{
    facts::{FactPath, FactStore, FactValue},
    predicate::{CompareOp, Operand, Predicate, Truth},
};
use std::collections::{BTreeMap, BTreeSet};

/// Maximum joint enum assignments admitted as a complete domain.
pub const MAX_ASSIGNMENTS: usize = 64;
/// Maximum AST nodes in all guards, independently of their nesting or domain size.
pub const MAX_PREDICATE_NODES: usize = 128;

/// One actual declared assignment and the branches satisfied by the existing evaluator.
#[derive(Debug, Clone)]
pub struct Case {
    /// Values for every fact read by the guards, in declared path order.
    pub values: BTreeMap<FactPath, String>,
    /// Guard positions that evaluate true. Unknown never enters a complete table.
    pub selected: Vec<usize>,
}

/// Syntactic admission only; callers still need typed analysis to prove coverage.
pub fn paths(guards: &[&Predicate]) -> Option<BTreeSet<FactPath>> {
    input_paths(guards, false)
}

fn input_paths(guards: &[&Predicate], allow_empty: bool) -> Option<BTreeSet<FactPath>> {
    if guards.len() > MAX_PREDICATE_NODES {
        return None;
    }
    let mut paths = BTreeSet::new();
    let mut pending: Vec<_> = guards.to_vec();
    let mut visited = 0;
    while let Some(predicate) = pending.pop() {
        visited += 1;
        if visited > MAX_PREDICATE_NODES {
            return None;
        }
        match predicate {
            Predicate::Always | Predicate::Never => {}
            Predicate::All(children) | Predicate::Any(children) => {
                if children.len() > MAX_PREDICATE_NODES {
                    return None;
                }
                pending.extend(children);
            }
            Predicate::Not(child) => pending.push(child),
            Predicate::Compare {
                left,
                op: CompareOp::Eq | CompareOp::Ne,
                right,
            } => match (left, right) {
                (Operand::Fact(path), Operand::Literal(FactValue::Text(_)))
                | (Operand::Literal(FactValue::Text(_)), Operand::Fact(path)) => {
                    paths.insert(path.clone());
                }
                _ => return None,
            },
            Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
                if values.len() > MAX_PREDICATE_NODES
                    || values.iter().any(|v| !matches!(v, FactValue::Text(_)))
                {
                    return None;
                }
                paths.insert(path.clone());
            }
            _ => return None,
        }
    }
    if paths.is_empty() && !allow_empty {
        None
    } else {
        Some(paths)
    }
}

/// Enumerate the entire supported domain, or decline to make a finite proof.
///
/// Optional paths and open domains are deliberately unsupported. Every guard is checked
/// against the same existing type authority that admits ordinary predicates.
pub fn analyze<E: TypeEnvironment>(environment: &E, guards: &[&Predicate]) -> Option<Vec<Case>> {
    analyze_inputs(environment, guards, false)
}

fn analyze_inputs<E: TypeEnvironment>(
    environment: &E,
    guards: &[&Predicate],
    allow_empty: bool,
) -> Option<Vec<Case>> {
    let paths = input_paths(guards, allow_empty)?;
    for guard in guards {
        if !check_predicate(environment, guard, "finite outcome coverage")
            .errors
            .is_empty()
        {
            return None;
        }
    }
    let mut assignments = vec![BTreeMap::new()];
    for path in paths {
        let resolved = resolve_path(environment, &path, "finite outcome coverage").ok()?;
        if resolved.optional || resolved.access.collection {
            return None;
        }
        let variants = resolved.variants?;
        if variants.is_empty() || assignments.len().checked_mul(variants.len())? > MAX_ASSIGNMENTS {
            return None;
        }
        let mut next = Vec::new();
        for assignment in assignments {
            for variant in &variants {
                let mut row = assignment.clone();
                row.insert(path.clone(), variant.clone());
                next.push(row);
            }
        }
        assignments = next;
    }
    let mut result = Vec::new();
    for values in assignments {
        let mut facts = FactStore::new();
        for (path, value) in &values {
            facts.set(path.clone(), value.as_str());
        }
        let mut selected = Vec::new();
        for (index, guard) in guards.iter().enumerate() {
            match guard.evaluate(&facts) {
                Truth::True => selected.push(index),
                Truth::False => {}
                Truth::Unknown => return None,
            }
        }
        result.push(Case { values, selected });
    }
    Some(result)
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
    if states.is_empty() || states.len() > MAX_ASSIGNMENTS {
        return None;
    }
    let always = Predicate::Always;
    let inputs: Vec<_> = guards
        .iter()
        .map(|guard| guard.predicate.unwrap_or(&always))
        .collect();
    let cases = analyze_inputs(environment, &inputs, true)?;
    if cases.len().checked_mul(states.len())? > MAX_ASSIGNMENTS {
        return None;
    }
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
    Some(result)
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
    pub fields: BTreeMap<FactPath, String>,
    /// Values for every input field the guards read.
    pub input: BTreeMap<FactPath, String>,
    /// Guard positions whose stored and input sides both evaluate true.
    pub selected: Vec<usize>,
}

/// The finite input proof run over the stored fields and over the input, crossed under one bound.
///
/// Declines exactly where [`analyze`] declines on either side — an open domain, an optional or
/// collection path, a guard the checker refuses — and where the product exceeds
/// [`MAX_ASSIGNMENTS`]. A side no guard reads contributes one empty assignment.
pub fn analyze_with_fields<F: TypeEnvironment, I: TypeEnvironment>(
    fields: &F,
    input: &I,
    guards: &[FieldGuard<'_>],
) -> Option<Vec<FieldCase>> {
    let always = Predicate::Always;
    let stored: Vec<_> = guards
        .iter()
        .map(|guard| guard.fields.unwrap_or(&always))
        .collect();
    let supplied: Vec<_> = guards
        .iter()
        .map(|guard| guard.input.unwrap_or(&always))
        .collect();
    let stored = analyze_inputs(fields, &stored, true)?;
    let supplied = analyze_inputs(input, &supplied, true)?;
    if stored.len().checked_mul(supplied.len())? > MAX_ASSIGNMENTS {
        return None;
    }
    let mut result = Vec::new();
    for row in &stored {
        for sent in &supplied {
            result.push(FieldCase {
                fields: row.values.clone(),
                input: sent.values.clone(),
                selected: row
                    .selected
                    .iter()
                    .copied()
                    .filter(|index| sent.selected.contains(index))
                    .collect(),
            });
        }
    }
    Some(result)
}
