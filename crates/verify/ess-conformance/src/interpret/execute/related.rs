//! Read the exact related row named by admitted input lookup syntax — or, from ess/22, by a stored
//! field of the addressed subject (beyond10x/ess#304) — never a nearby row.
use std::collections::BTreeMap;

use super::{
    subject::Held, EssIr, Phase, PrecedencePlan, ResolvedCommand, ResolvedCondition,
    ResolvedOutcome, ResolvedRelatedVia, ResolvedTypeRef, Row, State, Undetermined, Value,
};
use ess_compiler::ir::EntityHandle;
use ess_primitives::node::Node;

/// Whether an Optional input reference is absent (ess/22, beyond10x/ess#304): omitted or null.
/// Such a reference reads no row and selects no related branch. A required reference is never
/// absent here; its omission stays the request error it always was.
pub(super) fn absent_optional(type_ref: &ResolvedTypeRef, value: Option<&Node>) -> bool {
    type_ref.is_optional() && value.is_none_or(|value| *value == Node::Null)
}

/// The stored field of the addressed subject a command's related guards read (ess/22,
/// beyond10x/ess#304) and the entity whose row it names; `None` for a command reading its related
/// row through its input, or reading none.
pub(super) fn stored(spec: &ResolvedCommand) -> Option<(&str, &EntityHandle)> {
    spec.outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                via: ResolvedRelatedVia::Subject { field, .. },
                entity,
                ..
            } => Some((field.as_str(), entity)),
            _ => None,
        })
}

/// Whether the plan's `phase` holds an `exists: false` branch over a row the input names: the phase
/// a missing such row is answered in — [`Phase::RelatedRow`], or under the one rule
/// [`Phase::PresentRelated`] (`docs/design/generic-language-features.md` § G4).
pub(super) fn reads_missing(plan: &PrecedencePlan<'_>, phase: Phase) -> bool {
    plan.branches(phase).iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::Related {
                via: ResolvedRelatedVia::Input { .. },
                test: ess_compiler::ir::ResolvedRelatedTest::Absent,
                ..
            }
        )
    })
}

/// Whether the row `condition` names through the input is missing: the request names an identity
/// and no row of the entity carries it. `false` for any other condition, and for an absent Optional
/// reference or an identity the request does not carry, which name no row.
pub(super) fn missing(
    ir: &EssIr,
    store: &State,
    input: &BTreeMap<String, Node>,
    condition: &ResolvedCondition,
) -> bool {
    let ResolvedCondition::Related {
        via, entity, also, ..
    } = condition
    else {
        return false;
    };
    // ess/28: an `all:` branch's row is missing where any row it lists is.
    std::iter::once((via, entity))
        .chain(also.iter().map(|row| (&row.via, &row.entity)))
        .any(|(via, entity)| {
            let ResolvedRelatedVia::Input { field, type_ref } = via else {
                return false;
            };
            !absent_optional(type_ref, input.get(field))
                && input
                    .get(field)
                    .is_some_and(|value| names_missing(ir, store, (via, entity), value))
        })
}

/// Whether `value`, read through `via`, names an identity no row of `entity` carries: one
/// identity, or for a list of references (ess/28, beyond10x/ess#558) any of its elements. An empty
/// list names none.
pub(super) fn names_missing(
    ir: &EssIr,
    store: &State,
    (via, entity): (&ResolvedRelatedVia, &EntityHandle),
    value: &Node,
) -> bool {
    first_missing(ir, store, (via, entity), value).is_some()
}

/// The first identity `value` names, in list order for a list of references (ess/28,
/// beyond10x/ess#558), that no row of `entity` carries; `None` where every one names a row.
pub(super) fn first_missing(
    ir: &EssIr,
    store: &State,
    (via, entity): (&ResolvedRelatedVia, &EntityHandle),
    value: &Node,
) -> Option<Node> {
    let name = &ir.entity(entity).name;
    let carried = |identity: &Node| store.instance_typed(name, identity).is_some();
    match (via.type_ref(), value) {
        (ResolvedTypeRef::List { .. }, Node::Seq(items)) => {
            items.iter().find(|item| !carried(item)).cloned()
        }
        _ => (!carried(value)).then(|| value.clone()),
    }
}

/// Whether `via` reads a list of references (ess/28, beyond10x/ess#558).
pub(super) fn lists(via: &ResolvedRelatedVia) -> bool {
    matches!(via.type_ref(), ResolvedTypeRef::List { .. })
}

/// One branch per related row the command reads, in the order its rows are read when looking for a
/// missing one: each row's `exists: false` branch in the order the plan's `phase` reads them, then
/// the first declared branch of any row declaring none (beyond10x/ess#283). One branch on a command
/// reading one row.
pub(super) fn reads_in_order<'c>(
    plan: &PrecedencePlan<'c>,
    phase: Phase,
    spec: &'c ResolvedCommand,
) -> Vec<&'c ResolvedOutcome> {
    fn row(outcome: &ResolvedOutcome) -> Option<&str> {
        match &outcome.condition {
            ResolvedCondition::Related { via, .. } => Some(via.field()),
            _ => None,
        }
    }
    let mut ordered: Vec<&ResolvedOutcome> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for outcome in plan.branches(phase).iter().copied().chain(&spec.outcomes) {
        if let Some(field) = row(outcome) {
            if !seen.contains(&field) {
                seen.push(field);
                ordered.push(outcome);
            }
        }
    }
    ordered
}

/// What a stored reference names, read from the addressed subject as it was before the branch.
pub(super) enum Reference<'a> {
    /// Nothing is stored: an Optional reference left absent. No row is read and no related branch
    /// is selected.
    Absent,
    /// An identity no row of the entity carries: the `exists: false` branch's answer.
    Missing,
    /// The row it names.
    Row(&'a Row),
}

/// The row the addressed subject's stored `field` names, among `entity`'s rows in `store`.
pub(super) fn reference<'a>(
    ir: &EssIr,
    store: &'a State,
    subject: &Row,
    field: &str,
    entity: &EntityHandle,
) -> Result<Reference<'a>, Undetermined> {
    let identity = match subject.fields.get(field).map(Value::concrete) {
        None | Some(Ok(None | Some(Node::Null))) => return Ok(Reference::Absent),
        Some(Ok(Some(identity))) => identity,
        Some(Err(())) => {
            return Err(Undetermined::Undecidable {
                outcome: "related-row selection".into(),
                guard: format!("an unrecorded value stored in `{field}` names the related row"),
            })
        }
    };
    Ok(store
        .instance_typed(&ir.entity(entity).name, &identity)
        .map_or(Reference::Missing, Reference::Row))
}

pub(super) fn held<'a>(
    ir: &'a EssIr,
    store: &'a State,
    input: &BTreeMap<String, Node>,
    condition: &ResolvedCondition,
) -> Result<Option<Held<'a>>, Undetermined> {
    let ResolvedCondition::Related {
        via, entity, also, ..
    } = condition
    else {
        return Ok(None);
    };
    let Some(mut held) = held_row(ir, store, input, (via, entity))? else {
        return Ok(None);
    };
    // ess/28: every further row an `all:` branch lists, read the same way; one absent Optional
    // reference selects no branch over them.
    for row in also {
        let Some(further) = held_row(ir, store, input, (&row.via, &row.entity))? else {
            return Ok(None);
        };
        held.also.push(further);
    }
    Ok(Some(held))
}

/// The row `via` names in `store`, held for its predicates; `None` for an absent Optional reference.
fn held_row<'a>(
    ir: &'a EssIr,
    store: &'a State,
    input: &BTreeMap<String, Node>,
    (via, entity): (&ResolvedRelatedVia, &EntityHandle),
) -> Result<Option<Held<'a>>, Undetermined> {
    let ResolvedRelatedVia::Input { field, type_ref } = via else {
        return Err(Undetermined::NotInterpreted {
            construct: "a related guard not named by command input".into(),
        });
    };
    // An absent Optional reference holds no row, so its related branches are never selected; a
    // list of references (ess/28) is tested for a missing row only, and holds no row either.
    if absent_optional(type_ref, input.get(field)) || lists(via) {
        return Ok(None);
    }
    let identity = input
        .get(field)
        .ok_or_else(|| Undetermined::NotInterpreted {
            construct: format!("a related guard with no identity in `{field}`"),
        })?;
    let entity = ir.entity(entity);
    // Missing-row routing runs before input refusals and this present-row selection. Never
    // turn an absent row into an empty fact source whose default could authorize a command.
    let instance = store
        .instance_typed(&entity.name, identity)
        .ok_or_else(|| Undetermined::Undecidable {
            outcome: "related-row selection".into(),
            guard: format!("no `{}` row holds the addressed identity", entity.name),
        })?;
    Held::new(ir, entity, instance).map(Some)
}
