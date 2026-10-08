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

/// One branch per related row the command reads, in the order its rows are read when looking for a
/// missing one: each row's `exists: false` branch in the order the plan's [`Phase::RelatedRow`]
/// reads them, then the first declared branch of any row declaring none (beyond10x/ess#283). One
/// branch on a command reading one row.
pub(super) fn reads_in_order<'c>(
    plan: &PrecedencePlan<'c>,
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
    for outcome in plan
        .branches(Phase::RelatedRow)
        .iter()
        .copied()
        .chain(&spec.outcomes)
    {
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
    let ResolvedCondition::Related { via, entity, .. } = condition else {
        return Ok(None);
    };
    let ResolvedRelatedVia::Input { field, type_ref } = via else {
        return Err(Undetermined::NotInterpreted {
            construct: "a related guard not named by command input".into(),
        });
    };
    // An absent Optional reference holds no row, so its related branches are never selected.
    if absent_optional(type_ref, input.get(field)) {
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
