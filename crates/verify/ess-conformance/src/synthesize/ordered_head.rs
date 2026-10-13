//! The head of an ordered selection, arranged (`docs/design/ordered-related-head.md`, "Synthesis";
//! source format `ess/25`, beyond10x/ess#543).
//!
//! A branch reading `{related: {entity, where, order_by, first}}` is witnessed on rows
//! [`super::row_set`] arranges: three selected rows S1, S2 and S3, created in that order, the first
//! key steered so that S2 ranks first, S1 second and S3 last. The head is then neither the first
//! nor the last inserted, nor the head under the first key's reversed direction, nor the first or
//! last by identity alone. Where the identity is chosen by the scenario's input, a row T ties S2 on
//! every key before it and loses on the identity, created before S2; and for each further key, a
//! row ties S2 on the keys before it and loses on that one. The head is computed over the rows the
//! steps leave, with the comparator the interpreter ranks a view's order and a head with, and the
//! expected value is that row's.
//!
//! This module holds what the arrangement and the reading share: the ranking of two rows, which
//! keys a scenario can steer, and the named gaps.

use std::cmp::Ordering;

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedEffect, ResolvedRowSelection, ResolvedTypeRef,
};
use ess_domain::view::{Direction, Ranking};
use ess_primitives::node::Node;

/// The reason a branch reading a head is refused where no creating command steers a key.
pub(super) const UNSTEERED: &str =
    "the ranked head is not arranged: no creating command writes this key from its input";
/// The reason a branch reading a head over at most one selected row is refused.
pub(super) const UNOBSERVABLE: &str =
    "the ranked head is not observable: the branch reading it admits at most one selected row";
/// The reason a head decided by a minted identity is not arranged with a tie on it.
pub(super) const MINTED: &str = "the ranked head is decided by an identity the target mints, \
     which no scenario orders in advance";

/// The key a head's value is settled under in the arrangement's fields: not a field name — it
/// holds spaces — so nothing reading fields by name finds it.
pub(super) fn key(selection: &ResolvedRowSelection, order_by: &[Ranking], field: &str) -> String {
    format!(
        "ranked head of {} where {} by {} field {field}",
        selection.entity.name(),
        selection.filter,
        order_by
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Whether `name` is `entity`'s identity.
pub(super) fn is_identity(ir: &EssIr, entity: &EntityHandle, name: &str) -> bool {
    ir.entity(entity).identity.name == name
}

/// The type a ranking key of `entity` ranks at: a declared field's, the identity's, or the
/// entity's `State` for `state`, as the interpreter's head reads it.
fn key_type(ir: &EssIr, entity: &EntityHandle, name: &str) -> ResolvedTypeRef {
    let entity = ir.entity(entity);
    entity
        .fields
        .iter()
        .chain(std::iter::once(&entity.identity))
        .find(|field| field.name == name)
        .map_or_else(
            || entity.state_field().type_ref,
            |field| field.type_ref.clone(),
        )
}

/// How `left` and `right` rank on the one key `ranking`, its direction applied; `None` where either
/// value is unknown.
pub(super) fn on(
    ir: &EssIr,
    entity: &EntityHandle,
    ranking: &Ranking,
    left: Option<&Node>,
    right: Option<&Node>,
) -> Option<Ordering> {
    let (left, right) = (left?, right?);
    let ordering = crate::interpret::rank_compare(
        ir,
        &key_type(ir, entity, &ranking.field),
        Some(left),
        Some(right),
    );
    Some(if ranking.direction == Direction::Descending {
        ordering.reverse()
    } else {
        ordering
    })
}

/// How two rows rank by `order_by`, each key read by `value`; `None` where a key that would decide
/// is unknown for either.
pub(super) fn rank<R>(
    ir: &EssIr,
    entity: &EntityHandle,
    order_by: &[Ranking],
    value: &dyn Fn(&R, &str) -> Option<Node>,
    left: &R,
    right: &R,
) -> Option<Ordering> {
    for ranking in order_by {
        let ordering = on(
            ir,
            entity,
            ranking,
            value(left, &ranking.field).as_ref(),
            value(right, &ranking.field).as_ref(),
        )?;
        if ordering.is_ne() {
            return Some(ordering);
        }
    }
    Some(Ordering::Equal)
}

/// The index of the row of `rows` that ranks first by `order_by`, where every comparison with it is
/// decided; `None` for no rows, or where which is first is not decided.
pub(super) fn first<R>(
    ir: &EssIr,
    entity: &EntityHandle,
    order_by: &[Ranking],
    value: &dyn Fn(&R, &str) -> Option<Node>,
    rows: &[R],
) -> Option<usize> {
    (0..rows.len()).find(|at| {
        rows.iter().enumerate().all(|(other, row)| {
            other == *at
                || rank(ir, entity, order_by, value, &rows[*at], row) == Some(Ordering::Less)
        })
    })
}

/// Whether some declared creation of `entity` writes the key `name` from its input: a field set
/// straight from an input, or the identity published from one. `state` is never steered.
pub(super) fn steerable(ir: &EssIr, entity: &EntityHandle, name: &str) -> bool {
    let identity = is_identity(ir, entity, name);
    ir.drivers().get(entity).is_some_and(|drivers| {
        drivers
            .iter()
            .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
            .any(|driver| {
                if identity {
                    return super::published_identity_input(driver.outcome).is_some();
                }
                driver.outcome.sets.iter().any(|set| {
                    set.target == name
                        && set.conversion.is_none()
                        && matches!(
                            set.value,
                            ess_compiler::ir::ResolvedPayloadValue::InputField { .. }
                        )
                })
            })
    })
}
