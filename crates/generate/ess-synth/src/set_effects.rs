//! A re-key (ess/23, beyond10x/ess#429, `docs/design/identity-changing-updates.md`), refused by
//! every code target but Rust.
//!
//! An `updates:` whose `sets:` writes the identity moves its row to another identity. The generated
//! Rust behaviour inserts the row under the new identity and then removes the old one, through its
//! storage port; the Go, Web and Clap seams write the row their request names in place, so each such
//! branch is named rather than emitted writing a field the row is stored under.
//!
//! Set effects over filtered instances (`instances:` and `affects:`, ess/16) are not refused here:
//! from beyond10x/ess#547 (`docs/design/set-effect-identity-filters.md`, D4) the command is an owed
//! behaviour (`crate::determined`), and its contract names the rows each set effect changes
//! (`crate::plan::behavior_contract`).

use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
use crate::{SynthesisPlan, Target};

/// Why a re-key has no representation outside Rust.
const REKEY: &str = "this target cannot move a record to another identity; a generated seam \
                     writes the row its request names in place, and the identity it is stored \
                     under is not a field it can write";

/// Every re-key, named at the identity it writes, for every target but Rust.
pub(crate) fn refuse(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    if target == Target::Rust {
        return Ok(());
    }
    let mut causes = Vec::new();
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
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
