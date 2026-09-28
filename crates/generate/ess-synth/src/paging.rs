//! Paged views (`paging:`, ess/16, beyond10x/ess#174, `docs/design/view-paging.md`), refused by
//! every code target as `Json` is.
//!
//! A generated view handler serves every row the projection the port owes holds: it reads no
//! parameter, slices nothing and counts nothing. A paged view promises a page of its declared order
//! and, with `total: true`, the filtered count beside it, so a handler emitted for one would answer
//! every read with every row and no total. Each paged view is named rather than emitted with a
//! paging nobody implemented.

use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
use crate::{SynthesisPlan, Target};

/// Why the view has no representation.
const DETAIL: &str =
    "this target serves a view as every row its projection holds, with no page and no total";

/// Every paged view, named at the key its author wrote.
pub(crate) fn refuse(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    let causes: Vec<TargetFailureCause> = ir
        .views()
        .values()
        .filter(|view| view.paging.is_some())
        .map(|view| {
            TargetFailureCause::new(
                TargetFailureCode::MissingRepresentation,
                vec![format!("views.{}.paging", view.name)],
                DETAIL.to_owned(),
            )
        })
        .collect();
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}
