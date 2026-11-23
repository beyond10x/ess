//! The readings a caller synthesis merges, planned before any is computed.
//!
//! A reading is one synthesis of one rewritten model under one focus. [`plan`] lists every reading
//! the merge of [`super::synthesize`] consumes, in the order the merge consumes them, and
//! [`read_all`] computes a list of jobs and returns the results in that order, whoever computes
//! them. A reading is a function of its explicit inputs (the model, the callers, the seeds and the
//! entry), so the merge reads results and runs no reading in the middle of itself, bar the two
//! that depend on a merge result (`one_row_acted_on` and the rotation re-run).

use std::collections::BTreeSet;
use std::sync::Arc;

use ess_compiler::ir::EssIr;
use ess_domain::name::QualifiedName;

use super::{run, run_seeded, Assigned, InvocationModels, Who};
use crate::synthesize::{Focus, Synthesis};

/// One synthesis the merge reads.
pub(super) enum Reading<'q> {
    /// The whole model read as `Who` sends every command; seeded for `First` only.
    Whole(Who),
    /// One command's run for the mixed-caller loop: sent as `Second` with every other command
    /// sent as `First` (`back: false`), or the reverse (`back: true`); a whole synthesis where
    /// `whole`, focused on the command otherwise.
    About {
        command: &'q QualifiedName,
        back: bool,
        whole: bool,
    },
    /// `merge_cross`'s run of one command: arranged as `First` and acted as `Second`, seeded
    /// (`reversed: false`), or the reverse, seed-free.
    Cross {
        command: &'q QualifiedName,
        reversed: bool,
    },
}

/// Every reading of the merge, in the order the merge consumes them: the two whole readings, then
/// the two focused runs of each mixed command, then the two crossed runs of each, in name order.
pub(super) fn plan(mixed: &BTreeSet<QualifiedName>, unfocused: bool) -> Vec<Reading<'_>> {
    let mut plan = vec![Reading::Whole(Who::First), Reading::Whole(Who::Second)];
    for command in mixed {
        for back in [false, true] {
            plan.push(Reading::About {
                command,
                back,
                whole: unfocused,
            });
        }
    }
    for command in mixed {
        for reversed in [false, true] {
            plan.push(Reading::Cross { command, reversed });
        }
    }
    plan
}

/// The two whole runs of a command's pair, read inline where its focused runs sent a rotated command.
pub(super) fn read_about_whole(
    assigned: &Assigned<'_>,
    seeds: &crate::synthesize::AdmittedSeeds,
    command: &QualifiedName,
) -> (Synthesis, Synthesis) {
    let whole = |back| {
        read(
            assigned,
            seeds,
            &Reading::About {
                command,
                back,
                whole: true,
            },
        )
    };
    (whole(false), whole(true))
}

/// The assignment an [`Reading::About`] reads: `command` sent as `Second`, every other as `First`,
/// or the reverse.
fn about_sent(command: &QualifiedName, back: bool) -> impl Fn(&QualifiedName) -> Who + '_ {
    move |name| {
        let sent = if name == command {
            Who::Second
        } else {
            Who::First
        };
        if back {
            sent.other()
        } else {
            sent
        }
    }
}

/// The synthesis of one reading.
pub(super) fn read(
    assigned: &Assigned<'_>,
    seeds: &crate::synthesize::AdmittedSeeds,
    reading: &Reading<'_>,
) -> Synthesis {
    match reading {
        Reading::Whole(Who::First) => run_seeded(assigned, &|_| Who::First, Focus::Whole, seeds),
        Reading::Whole(Who::Second) => run(assigned, &|_| Who::Second, Focus::Whole),
        Reading::About {
            command,
            back,
            whole,
        } => {
            let sent = about_sent(command, *back);
            let focus = if *whole {
                Focus::Whole
            } else {
                Focus::About(command)
            };
            if *back {
                run(assigned, &sent, focus)
            } else {
                run_seeded(assigned, &sent, focus, seeds)
            }
        }
        Reading::Cross { command, reversed } => {
            let (ir, callers) = (assigned.ir, assigned.callers);
            let first = assigned.model(&|_| Who::First);
            let second = assigned.model(&|_| Who::Second);
            let models = if *reversed {
                InvocationModels {
                    arrangement: &second,
                    acting: &first,
                    credentials: Some((ir, callers, Who::Second)),
                    seeds: &crate::synthesize::seeds::EMPTY,
                }
            } else {
                InvocationModels {
                    arrangement: &first,
                    acting: &second,
                    credentials: Some((ir, callers, Who::First)),
                    seeds,
                }
            };
            crate::synthesize::synthesize_invocations(&models, Focus::About(command))
        }
    }
}

impl Assigned<'_> {
    /// The uniform model the reading's arrangement is, or `None` where it is not one.
    pub(super) fn lane(&self, reading: &Reading<'_>) -> Option<&Arc<EssIr>> {
        match reading {
            Reading::Whole(who) => self.uniform.get(&self.key(&|_| *who)),
            Reading::About { command, back, .. } => {
                self.uniform.get(&self.key(&about_sent(command, *back)))
            }
            Reading::Cross { reversed, .. } => {
                let who = if *reversed { Who::Second } else { Who::First };
                self.uniform.get(&self.key(&|_| who))
            }
        }
    }
}

/// Computes every job and returns the results in the order of `jobs`.
///
/// `lane` names the uniform model a job reads, which a caller computing jobs on several threads
/// keeps on one of them.
pub(super) fn read_all<'m, J, R>(
    jobs: &[J],
    lane: impl Fn(&J) -> Option<&'m Arc<EssIr>>,
    read: impl Fn(&J) -> R,
) -> Vec<R> {
    let _ = &lane;
    jobs.iter().map(read).collect()
}

/// The memory the two uniform models keep across their syntheses, for as long as it lives.
///
/// It is not `Send`: dropping it pops the stack of the thread that opened it.
pub(super) struct Kept {
    _made: crate::synthesize::made::Keeping,
    _witnessed: crate::witness_memo::Keeping,
}

impl Kept {
    pub(super) fn open(models: &[&Arc<EssIr>]) -> Self {
        let mut made = crate::synthesize::made::Keeping::open();
        let mut witnessed = crate::witness_memo::Keeping::open();
        for model in models {
            made.keep(model);
        }
        for model in models {
            witnessed.keep(model);
        }
        Self {
            _made: made,
            _witnessed: witnessed,
        }
    }
}
