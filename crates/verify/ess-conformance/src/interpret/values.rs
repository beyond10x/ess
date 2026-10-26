//! The values the interpreted target mints and the order it answers a view's rows in, for a
//! consumer that must agree with it.
//!
//! A tool that shows what the interpreter would answer (a renderer, a model preview) needs the
//! identifiers it mints and the row order it answers with. These are the functions the interpreter
//! itself calls, so a consumer calling them agrees with the interpreter of the same release instead
//! of with a copy that drifts.
//!
//! Both answer **the interpreted target's own choice**, not a rule of the specification. The runner
//! never compares a minted value against an expected one, and no implementation is checked against
//! either function. A later release may change the minting scheme or how a type ranks, and these
//! functions change with it.
//!
//! Not covered: values minted while replaying a recorded history, which come from the record, and
//! building a [`Store`](super::execute::Store) from outside the crate.

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedTypeRef, ResolvedView};
use ess_domain::types::Primitive;
use ess_primitives::node::Node;

use super::execute::Undetermined;
use crate::target::ViewRow;

/// The last tick [`mint`] mints a `Uuid` on: the tick is the UUID's last group, 12 decimal digits.
pub const MAX_UUID_TICK: u64 = 999_999_999_999;

/// The value of `target` the interpreter mints on the `tick`-th draw of a scenario's counter.
///
/// Every created instance's identity and every value the model leaves to the implementation is
/// drawn from one counter per scenario, which starts at zero when the scenario begins and is
/// advanced before each draw, so the first draw is `tick` 1. A scenario that mints nothing but
/// three identities mints `mint(ir, identity, 1)`, then `2`, then `3`.
///
/// A `Uuid` (directly or under newtypes that constrain nothing) is
/// `00000000-0000-4000-8000-<tick, zero-padded to 12 digits>`, for `tick` up to
/// [`MAX_UUID_TICK`]; past it the last group would not fit 12 digits, and the draw is refused.
///
/// Any other type is the witness synthesis builds for the `tick`-th further instance of that type,
/// so it satisfies the type's own constraints (a `String` newtype's `prefix:`, for example). Such
/// a witness is not always new on a new tick:
///
/// - a `Boolean` alternates, so ticks 1 and 3 give one value;
/// - a `Timestamp` is a day of one month and repeats every 28 ticks, so ticks 1 and 29 give one
///   value;
/// - an `Integer` or `Decimal` is `1 + tick` up to `tick` = `u32::MAX`, and every later tick gives
///   that same last value.
///
/// Where a new identity meets a held one, the interpreter draws again, a bounded number of times.
///
/// `None` for an `Optional` type: absent is a value the implementation may choose, and the one
/// that claims least. The interpreter does not advance its counter for such a draw.
///
/// The interpreted target's choice, not a specification rule; see the
/// [module documentation](self).
///
/// # Errors
///
/// [`Undetermined::NoValue`] where the type has no witness to mint, where `tick` does not fit a
/// `usize`, or for a `Uuid` past [`MAX_UUID_TICK`].
pub fn mint(ir: &EssIr, target: &ResolvedTypeRef, tick: u64) -> Result<Option<Node>, Undetermined> {
    if target.is_optional() {
        return Ok(None);
    }
    let no_value = || Undetermined::NoValue {
        what: format!("a value of `{target}`"),
    };
    if let Representation::Primitive(Primitive::Uuid) = representation(ir, target) {
        if tick > MAX_UUID_TICK {
            return Err(Undetermined::NoValue {
                what: format!(
                    "a value of `{target}` on tick {tick}: a minted UUID's last group holds at \
                     most 12 digits, so the last tick it mints is {MAX_UUID_TICK}"
                ),
            });
        }
        return Ok(Some(Node::Text(format!(
            "00000000-0000-4000-8000-{tick:012}"
        ))));
    }
    let field = ess_compiler::ir::ResolvedField {
        name: "generated".into(),
        type_ref: target.clone(),
        naming: ess_domain::name::Naming::default(),
    };
    let distinction = usize::try_from(tick).map_err(|_| no_value())?;
    crate::witness::fields(
        ir,
        &[field],
        crate::witness::Distinction::further(distinction),
    )
    .map(|mut values| values.remove("generated"))
    .map_err(|_| no_value())
}

/// Sorts `rows` into the order the interpreter answers `view` in: by each `order_by` key in turn,
/// descending where the key says so, and rows tied on every key keep the order the caller gave them
/// (a stable sort). A view without `order_by` leaves `rows` as they are.
///
/// A key ranks by its field's declared type: a `Timestamp` by its instant, not its spelling; a
/// newtype by the type underneath it; a struct by its members in declaration order; anything else
/// by its [`Node`] order. A field a row does not carry ranks below every value.
///
/// `view` is one of `ir`'s own views, and each row a map from that view's field names. Paging is
/// not applied: the interpreter orders every row first and then takes the page.
///
/// The interpreted target's choice, not a specification rule; see the
/// [module documentation](self).
///
/// # Panics
///
/// If an `order_by` key names no field of `view`, which a view compiled into `ir` never does.
pub fn order_view_rows(ir: &EssIr, view: &ResolvedView, rows: &mut [ViewRow]) {
    rows.sort_by(|left, right| super::views::rank(ir, view, left, right));
}

/// What a field's type is underneath every newtype, where no newtype on the way constrains it.
enum Representation {
    Primitive(Primitive),
    Enum,
    Other,
}

fn representation(ir: &EssIr, target: &ResolvedTypeRef) -> Representation {
    let mut current = target.required();
    // Bounded by the IR's own type depth: a newtype chain is a chain of declared handles.
    for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Primitive { name } => return Representation::Primitive(*name),
            ResolvedTypeRef::Declared { name } => {
                let declared = ir.named_type(name);
                match &declared.body {
                    ResolvedBody::Newtype { of, .. } if !declared.body.is_constrained() => {
                        current = of.required();
                    }
                    ResolvedBody::Enum { .. } => return Representation::Enum,
                    _ => return Representation::Other,
                }
            }
            _ => return Representation::Other,
        }
    }
    Representation::Other
}
