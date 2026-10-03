//! Read the exact related row named by admitted input lookup syntax, never a nearby row.
use std::collections::BTreeMap;

use super::{subject::Held, EssIr, ResolvedCondition, ResolvedRelatedVia, Store, Undetermined};
use ess_primitives::node::Node;

pub(super) fn held<'a>(
    ir: &'a EssIr,
    store: &'a Store,
    input: &BTreeMap<String, Node>,
    condition: &ResolvedCondition,
) -> Result<Option<Held<'a>>, Undetermined> {
    let ResolvedCondition::Related { via, entity, .. } = condition else {
        return Ok(None);
    };
    let ResolvedRelatedVia::Input { field, .. } = via else {
        return Err(Undetermined::NotInterpreted {
            construct: "a related guard not named by command input".into(),
        });
    };
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
