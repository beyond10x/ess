//! An `Optional<T>` input read as `T` in a branch that is only ever taken with it present (source
//! format `ess/16`, beyond10x/ess#169, `docs/design/optional-input-narrowing.md`).
//!
//! Two facts about outcome selection make an input present in a branch, and neither depends on the
//! order outcomes are declared in:
//!
//! 1. A branch whose own guard requires `defined(x)` — the guard is `defined(x)`, or an `all:`
//!    with it as a member — is taken only when its guard holds.
//! 2. The default branch is taken only when no guarded sibling matched. A sibling that refuses with
//!    `error:` under exactly `not defined(x)` (`missing(x)` is the same predicate) matches every
//!    request without `x`, so the default never sees one. Only the default written without a
//!    condition counts: `when: true` is a guard that holds, and Entity Runtime tries it in declared
//!    order, so a `when: true` branch before the refusal would take a request without `x`.
//!
//! Anything weaker narrows nothing: a refusal of `x`'s absence *and* something else leaves some
//! absent requests to the default, a sibling that succeeds on absence says nothing about the
//! others, and a guarded branch other than the default can match a request its refusing sibling
//! matches too — overlap between input guards over unbounded types is not refused.
//!
//! Only a top-level input (`x`, not `x.y`) narrows, and only one `Optional` layer comes off.

use ess_primitives::predicate::Predicate;

use super::{CommandSpec, Outcome, OutcomeCondition};
use crate::system::FormatVersion;
use crate::types::{ConversionRegistry, Field, TypeRef};

impl CommandSpec {
    /// `true` when `input.<read>` may fill a field of type `target` in `outcome`, under `format`.
    ///
    /// The declared type first, with every conversion declared from it: narrowing adds a way in
    /// and never revokes a crossing the author declared from `Optional<T>`. Then, from `ess/16`
    /// on, the narrowed type where [`Self::narrowed_input`] finds one. An unknown format narrows
    /// nothing. A refusal names the declared type, which is what the author wrote.
    pub(crate) fn admits_input_read(
        &self,
        outcome: &Outcome,
        read: &Field,
        target: &TypeRef,
        conversions: &ConversionRegistry,
        format: Option<FormatVersion>,
    ) -> bool {
        if conversions.permits(&read.type_ref, target) {
            return true;
        }
        let admitted = format.is_some_and(|format| format.major() >= FormatVersion::V16.major());
        admitted
            && self
                .narrowed_input(outcome, &read.name)
                .is_some_and(|present| conversions.permits(present, target))
    }

    /// The type `input.<field>` is read at in `outcome` when it is narrowed there: `T` for an
    /// `Optional<T>` input the branch is only ever taken with present. `None` when the field is not
    /// an `Optional` input, or is not known present in this branch — read it at its declared type.
    ///
    /// Format-blind: callers apply it from `ess/16` on.
    pub fn narrowed_input(&self, outcome: &Outcome, field: &str) -> Option<&TypeRef> {
        let TypeRef::Optional(present) = &self.input_field(field)?.type_ref else {
            return None;
        };
        let guarded = outcome
            .condition
            .predicate()
            .is_some_and(|predicate| requires_defined(predicate, field));
        // The written default only: `when: true` is an ordinary guard that holds, and Entity Runtime
        // tries it in declared order, possibly before the refusal.
        let defaulted =
            outcome.condition == OutcomeCondition::Otherwise && self.refuses_absence_of(field);
        (guarded || defaulted).then_some(present.as_ref())
    }

    /// `true` when a guarded sibling refuses exactly the absence of `field`.
    fn refuses_absence_of(&self, field: &str) -> bool {
        self.outcomes.iter().any(|outcome| {
            outcome.is_refusal()
                && matches!(
                    &outcome.condition,
                    OutcomeCondition::When(Predicate::Not(inner))
                        if is_defined(inner, field)
                )
        })
    }
}

/// `true` when `predicate` cannot hold unless `field` is present.
fn requires_defined(predicate: &Predicate, field: &str) -> bool {
    match predicate {
        Predicate::All(members) => members.iter().any(|member| requires_defined(member, field)),
        other => is_defined(other, field),
    }
}

fn is_defined(predicate: &Predicate, field: &str) -> bool {
    matches!(predicate, Predicate::Defined(path) if path.segments() == [field])
}
