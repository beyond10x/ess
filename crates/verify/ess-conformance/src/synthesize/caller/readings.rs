//! The readings a caller synthesis merges, planned before any is computed.
//!
//! A reading is one synthesis of one rewritten model under one focus. [`plan`] lists every reading
//! the merge of [`super::synthesize`] consumes, in the order the merge consumes them, and
//! [`read_all`] computes a list of jobs and returns the results in that order, whoever computes
//! them. A reading is a function of its explicit inputs (the model, the callers, the seeds and the
//! entry), so the merge reads results and runs no reading in the middle of itself, bar the two
//! that depend on a merge result (`one_row_acted_on` and the rotation re-run).

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    #[cfg(test)]
    if let Reading::About {
        command,
        back: false,
        ..
    } = reading
    {
        assert!(
            !PANIC_IN.with(|seam| seam.borrow().as_deref() == Some(command.to_string().as_str())),
            "a test panic in the reading of `{command}`"
        );
    }
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
///
/// With more than one job ([`super::super::jobs`]) the jobs run on scoped threads: each uniform
/// model's jobs, in plan order, on one thread that keeps that model's memory (its lane), every
/// other job on whichever thread is free. The results are merged by index on the calling thread,
/// and every job runs to its end, so the panic that surfaces is the lowest-indexed one, as on one
/// thread.
pub(super) fn read_all<'m, J: Sync, R: Send>(
    jobs: &[J],
    lane: impl Fn(&J) -> Option<&'m Arc<EssIr>> + Sync,
    read: impl Fn(&J) -> R + Sync,
) -> Vec<R> {
    let threads = super::super::jobs().get();
    if threads == 1 || jobs.len() < 2 {
        return jobs.iter().map(read).collect();
    }
    let mut lanes: Vec<(&Arc<EssIr>, Vec<usize>)> = Vec::new();
    let mut pool: Vec<usize> = Vec::new();
    for (index, job) in jobs.iter().enumerate() {
        match lane(job) {
            Some(model) => match lanes.iter_mut().find(|(held, _)| Arc::ptr_eq(held, model)) {
                Some((_, indices)) => indices.push(index),
                None => lanes.push((model, vec![index])),
            },
            None => pool.push(index),
        }
    }
    let count = lanes.len().max(threads.min(lanes.len() + pool.len()));
    let context = super::super::threads::Context::capture();
    let cursor = AtomicUsize::new(0);
    let (lanes, pool, cursor) = (&lanes, &pool, &cursor);
    let joined = run_workers(count, &context, |worker| {
        let mut taken: Taken<R> = Vec::new();
        let mut run = |index: usize| {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read(&jobs[index])));
            taken.push((index, result));
        };
        if let Some((model, indices)) = lanes.get(worker) {
            let _kept = Kept::open(&[*model]);
            for &index in indices {
                run(index);
            }
        }
        while let Some(&index) = pool.get(cursor.fetch_add(1, Ordering::Relaxed)) {
            run(index);
        }
        taken
    });
    let mut results: Vec<Option<std::thread::Result<R>>> = (0..jobs.len()).map(|_| None).collect();
    let mut ran = 0;
    for worker in joined {
        let (taken, work) = match worker {
            Ok(done) => done,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        work.add();
        for (index, result) in taken {
            ran += 1;
            results[index] = Some(result);
        }
    }
    super::super::add_off_thread(ran);
    results
        .into_iter()
        .map(|result| match result.expect("every job ran") {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        })
        .collect()
}

/// Starts `count` scoped threads, each running `body(worker)` inside `context`, and joins them in
/// worker order with the work each counted. The one place a reading's thread is made, so the
/// context a worker holds is the one this sets.
fn run_workers<T: Send>(
    count: usize,
    context: &super::super::threads::Context,
    body: impl Fn(usize) -> T + Sync,
) -> Vec<std::thread::Result<(T, super::super::threads::Work)>> {
    #[cfg(test)]
    let panic_in = PANIC_IN.with(|seam| seam.borrow().clone());
    #[cfg(test)]
    let panic_in = &panic_in;
    let body = &body;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..count)
            .map(|worker| {
                std::thread::Builder::new()
                    .name(format!("ess-synth-{worker}"))
                    .stack_size(8 << 20)
                    .spawn_scoped(scope, move || {
                        context.enter(|| {
                            #[cfg(test)]
                            PANIC_IN.with(|seam| *seam.borrow_mut() = panic_in.clone());
                            // A reading never starts threads of its own.
                            super::super::with_jobs(std::num::NonZeroUsize::MIN, || {
                                (body(worker), super::super::threads::Work::taken())
                            })
                        })
                    })
                    .expect("a worker thread starts")
            })
            .collect();
        handles
            .into_iter()
            .map(std::thread::ScopedJoinHandle::join)
            .collect()
    })
}

/// The results one worker holds, by job index.
type Taken<R> = Vec<(usize, std::thread::Result<R>)>;

#[cfg(test)]
thread_local! {
    /// The command whose first `About` reading panics, for the panic test only.
    pub(super) static PANIC_IN: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
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

const _: () = {
    const fn sync<T: Sync>() {}
    sync::<super::Assigned<'static>>();
    sync::<super::Callers>();
};

#[cfg(test)]
mod tests {
    use ess_domain::command::precedence::{phase_order, with_phase_order, Phase};

    use super::super::super::threads::Context;
    use super::super::super::{dropped_write, with_dropped_write, DroppedWrite};
    use super::run_workers;

    /// What each of `count` workers started by [`run_workers`] reads: the dropped write, through
    /// the accessor the synthesis reads it with, and the phase order.
    fn held(count: usize) -> Vec<(Option<DroppedWrite>, [Phase; 8])> {
        let context = Context::capture();
        run_workers(count, &context, |_| (dropped_write(), phase_order()))
            .into_iter()
            .map(|worker| worker.expect("the worker ends").0)
            .collect()
    }

    /// A worker started by the runner `read_all` uses holds the dropped write and the phase order
    /// of the thread that planned the readings, and none of either where that thread set none. Two
    /// fields, so the proof does not rest on one setter.
    #[test]
    fn every_worker_holds_the_context_of_the_thread_that_started_it() {
        let write = DroppedWrite {
            command: "shop.order.Place".to_owned(),
            outcome: "placed".to_owned(),
            target: "note".to_owned(),
            source: "note".to_owned(),
        };
        let mut exchanged = Phase::PRECEDENCE;
        exchanged.swap(0, 1);
        for count in [2, 4] {
            let carried = with_dropped_write(Some(write.clone()), || {
                with_phase_order(exchanged, || held(count))
            });
            assert_eq!(carried.len(), count);
            for (worker, (dropped, order)) in carried.iter().enumerate() {
                assert_eq!(
                    dropped.as_ref(),
                    Some(&write),
                    "worker {worker} of {count} holds the dropped write"
                );
                assert_eq!(
                    *order, exchanged,
                    "worker {worker} of {count} holds the phase order"
                );
            }
            for (worker, (dropped, order)) in held(count).iter().enumerate() {
                assert_eq!(
                    dropped, &None,
                    "worker {worker} of {count} holds no dropped write"
                );
                assert_eq!(*order, Phase::PRECEDENCE, "worker {worker} of {count}");
            }
        }
    }
}
