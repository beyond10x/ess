//! Set effects over filtered instances (ess/16, beyond10x/ess#167, #175,
//! `docs/design/set-effects-over-filtered-instances.md`), refused by every code target as `Json`
//! is; and a re-key (ess/23, beyond10x/ess#429, `docs/design/identity-changing-updates.md`),
//! refused by every code target but Rust.
//!
//! `instances:` changes every stored row a filter selects, and `affects:` changes rows beside the
//! subject. The generated seams and explorers act on the one instance a request names and hold no
//! query over the stored rows, so each such branch is named rather than emitted changing one row
//! or none.
//!
//! An `updates:` whose `sets:` writes the identity moves its row to another identity. The generated
//! Rust behaviour inserts the row under the new identity and then removes the old one, through its
//! storage port; the Go, Web and Clap seams write the row their request names in place, so each such
//! branch is named rather than emitted writing a field the row is stored under.

use ess_compiler::ir::ResolvedOutcome;

use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
use crate::{SynthesisPlan, Target};

/// Why the branch has no representation.
const DETAIL: &str = "this target cannot change the rows a filter selects; a generated seam acts \
                      on the one instance its request names";

/// Why a re-key has no representation outside Rust.
const REKEY: &str = "this target cannot move a record to another identity; a generated seam \
                     writes the row its request names in place, and the identity it is stored \
                     under is not a field it can write";

/// The keys an author wrote a set effect with on this branch.
fn keys(outcome: &ResolvedOutcome) -> impl Iterator<Item = &'static str> {
    [
        outcome.instances.is_some().then_some("instances"),
        (!outcome.affects.is_empty()).then_some("affects"),
    ]
    .into_iter()
    .flatten()
}

/// Every set effect, named at the key its author wrote.
pub(crate) fn refuse(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    let mut causes = Vec::new();
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            for key in keys(outcome) {
                causes.push(TargetFailureCause::new(
                    TargetFailureCode::MissingRepresentation,
                    vec![format!(
                        "commands.{}.outcomes.{}.{key}",
                        command.name, outcome.name
                    )],
                    DETAIL.to_owned(),
                ));
            }
            if target == Target::Rust {
                continue;
            }
            if let Some(write) = outcome.identity_write(ir) {
                causes.push(TargetFailureCause::new(
                    TargetFailureCode::MissingRepresentation,
                    vec![format!(
                        "commands.{}.outcomes.{}.sets.{}",
                        command.name, outcome.name, write.target
                    )],
                    REKEY.to_owned(),
                ));
            }
        }
    }
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}
