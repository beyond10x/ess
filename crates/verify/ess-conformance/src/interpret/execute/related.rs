//! Read the exact related row named by admitted input lookup syntax, never a nearby row.
use std::collections::BTreeMap;

use super::{
    subject::Held, EssIr, ResolvedCondition, ResolvedRelatedVia, ResolvedTypeRef, State,
    Undetermined,
};
use ess_primitives::node::Node;

/// Whether an Optional input reference is absent (ess/22, beyond10x/ess#304): omitted or null.
/// Such a reference reads no row and selects no related branch. A required reference is never
/// absent here; its omission stays the request error it always was.
pub(super) fn absent_optional(type_ref: &ResolvedTypeRef, value: Option<&Node>) -> bool {
    type_ref.is_optional() && value.is_none_or(|value| *value == Node::Null)
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
