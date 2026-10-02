//! One witness search per question, for the length of a synthesis (beyond10x/ess#301).
//!
//! [`crate::witness`]'s candidate search is a function of the model, the command, the guards, the
//! distinction and whether midpoints are added, and nothing else: no clock, no counter, no state
//! carried between calls. Synthesis asks it the same question many times: every arrangement that
//! creates a row runs its creating command again, and a model whose rows own rows arranges each
//! owner once per scenario that needs one. Over `examples/billing` 124 of 332 searches repeated an
//! earlier one; over the #301 reference model, 4,491 of 9,576 in one pass.
//!
//! [`memoise`] holds each answer for as long as the model it was asked of is borrowed, so the
//! second asking is a copy of the first answer. The suite is unchanged by construction: the answer
//! returned is the one the search returned.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_domain::name::QualifiedName;
use ess_primitives::predicate::Predicate;

use crate::witness::{Distinction, Searched};

thread_local! {
    /// The answers held, innermost model last. A stack because a synthesis of one model may
    /// synthesize another inside it (`synthesize::caller` reads the model once per caller
    /// assignment), and an answer about one model is no answer about another.
    static MEMO: RefCell<Vec<Memo>> = const { RefCell::new(Vec::new()) };
    #[cfg(test)]
    static WORK: RefCell<Option<Work>> = const { RefCell::new(None) };
}

/// The answers one model's searches gave, by command, distinction and midpoint flag, then by the
/// guards searched over. `Predicate` has no order, so the last level is compared one by one; it
/// holds the few guard sets one command is searched with.
struct Memo {
    /// The model, compared by address only. It is borrowed for as long as this entry is on the
    /// stack ([`Memoised`]), so the address cannot be reused by another model meanwhile.
    ir: *const EssIr,
    answers: BTreeMap<Question, Vec<(Vec<Predicate>, Searched)>>,
}

/// The command, distinction and midpoint flag a search was asked with.
type Question = (QualifiedName, Distinction, bool);

/// Holds witness answers for one model until dropped. See [`memoise`].
pub(crate) struct Memoised<'ir> {
    pushed: bool,
    model: PhantomData<&'ir EssIr>,
}

impl Drop for Memoised<'_> {
    fn drop(&mut self) {
        if self.pushed {
            MEMO.with(|memo| {
                memo.borrow_mut().pop();
            });
        }
    }
}

/// Answers each witness search about `ir` once while the result is held, and again from the first
/// answer after that. Nested for the same model, the outer holder keeps answering.
pub(crate) fn memoise(ir: &EssIr) -> Memoised<'_> {
    let at = std::ptr::from_ref(ir);
    let pushed = MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        if memo.last().is_some_and(|top| std::ptr::eq(top.ir, at)) {
            return false;
        }
        memo.push(Memo {
            ir: at,
            answers: BTreeMap::new(),
        });
        true
    });
    Memoised {
        pushed,
        model: PhantomData,
    }
}

/// Whether a search about `command` may be answered from memory: a model is being held and
/// `command` is the one it declares under that name. A command built for one question
/// (`synthesize::caller` probes with one) is searched every time.
fn held(ir: &EssIr, command: &ResolvedCommand) -> bool {
    ir.commands()
        .get(&command.name)
        .is_some_and(|declared| std::ptr::eq(declared, command))
        && MEMO.with(|memo| {
            memo.borrow()
                .last()
                .is_some_and(|top| std::ptr::eq(top.ir, ir))
        })
}

/// `search`'s answer for this question: from memory where it was asked before, otherwise from
/// `search`, remembered where the model is held.
pub(crate) fn answer(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    distinction: Distinction,
    between: bool,
    search: impl FnOnce() -> Searched,
) -> Searched {
    if !held(ir, command) {
        executed(ir, command, guards, distinction, between);
        return search();
    }
    let key = (command.name.clone(), distinction, between);
    let same = |asked: &[Predicate]| {
        asked.len() == guards.len() && asked.iter().zip(guards).all(|(a, b)| a == *b)
    };
    let known = MEMO.with(|memo| {
        memo.borrow().last().and_then(|top| {
            top.answers.get(&key).and_then(|asked| {
                asked
                    .iter()
                    .find(|(searched, _)| same(searched))
                    .map(|(_, answer)| answer.clone())
            })
        })
    });
    if let Some(known) = known {
        return known;
    }
    executed(ir, command, guards, distinction, between);
    // No borrow is held across the search: it may itself ask the memory.
    let answer = search();
    MEMO.with(|memo| {
        if let Some(top) = memo.borrow_mut().last_mut() {
            top.answers.entry(key).or_default().push((
                guards.iter().map(|guard| (*guard).clone()).collect(),
                answer.clone(),
            ));
        }
    });
    answer
}

/// Records one search the witness module actually ran (a test-only work counter).
#[cfg(test)]
pub(crate) fn executed(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    distinction: Distinction,
    between: bool,
) {
    WORK.with(|work| {
        if let Some(work) = work.borrow_mut().as_mut() {
            let key = Key {
                ir: std::ptr::from_ref(ir) as usize,
                command: command.name.to_string(),
                distinction,
                between,
                guards: guards.iter().map(|guard| (*guard).clone()).collect(),
            };
            if work.seen.contains(&key) {
                work.repeated += 1;
            } else {
                work.seen.push(key);
            }
            work.executed += 1;
        }
    });
}

#[cfg(not(test))]
#[inline]
pub(crate) fn executed(
    _ir: &EssIr,
    _command: &ResolvedCommand,
    _guards: &[&Predicate],
    _distinction: Distinction,
    _between: bool,
) {
}

#[cfg(test)]
#[derive(PartialEq)]
struct Key {
    ir: usize,
    command: String,
    distinction: Distinction,
    between: bool,
    guards: Vec<Predicate>,
}

/// How many witness searches a run executed, and how many of them asked a question already answered.
#[cfg(test)]
#[derive(Default)]
struct Work {
    executed: usize,
    repeated: usize,
    seen: Vec<Key>,
}

#[cfg(test)]
fn counted<T>(run: impl FnOnce() -> T) -> (T, usize, usize) {
    WORK.with(|work| *work.borrow_mut() = Some(Work::default()));
    let out = run();
    let work = WORK
        .with(|work| work.borrow_mut().take())
        .expect("the counter was installed");
    (out, work.executed, work.repeated)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use ess_compiler::resolve::compile;
    use ess_compiler::source::SourceMap;
    use ess_domain::spec::{RawSpecFile, Specification};
    use ess_domain::system::Source;

    use super::*;
    use crate::synthesize::synthesize;

    fn example(name: &str) -> EssIr {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../examples")
            .join(name)
            .canonicalize()
            .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
        let mut found: Vec<PathBuf> = Vec::new();
        let mut pending = vec![base.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).expect("the example is readable") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().is_some_and(|it| it == "yaml") {
                    found.push(path);
                }
            }
        }
        found.sort();
        let mut sources = SourceMap::new();
        let mut parsed = Vec::new();
        for path in found {
            let label = path
                .strip_prefix(&base)
                .expect("inside the example")
                .display()
                .to_string();
            let text = std::fs::read_to_string(&path).expect("readable");
            let raw = RawSpecFile::parse(&text)
                .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
            sources.insert(label.clone(), text);
            parsed.push((Source::new(label), raw));
        }
        let specification = Specification::assemble(parsed)
            .unwrap_or_else(|errors| panic!("`{name}` validates:\n{errors}"));
        compile(&specification, &sources)
            .unwrap_or_else(|diagnostics| panic!("`{name}` resolves:\n{diagnostics}"))
    }

    /// The synthesis work guard (beyond10x/ess#301): over the repository's own example models, no
    /// witness search runs twice for one question. A search that is asked again is answered from
    /// the first run, so synthesis does work in proportion to the distinct questions its scenarios
    /// ask, and a change that re-runs searches per arrangement fails here rather than as minutes on
    /// a large model. A count, not a clock, so a loaded CI host cannot trip it.
    #[test]
    fn synthesis_runs_each_witness_search_once() {
        for name in ["billing", "gatepass", "oracle-fixture"] {
            let ir = example(name);
            let (synthesis, executed, repeated) = counted(|| synthesize(&ir));
            assert!(
                !synthesis.suite.scenarios.is_empty(),
                "`{name}` synthesizes scenarios"
            );
            assert!(executed > 0, "`{name}` ran no witness search at all");
            assert_eq!(
                repeated, 0,
                "`{name}`: {repeated} of {executed} witness searches repeated a question already \
                 answered in the same synthesis"
            );
        }
    }
}
