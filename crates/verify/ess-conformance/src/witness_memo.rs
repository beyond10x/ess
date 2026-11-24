//! One witness search per question, for the length of a synthesis (beyond10x/ess#301).
//!
//! [`crate::witness`]'s candidate search is a function of the model, the command, the guards, the
//! distinction, whether midpoints are added and the copies of the branch under test
//! ([`crate::witness::under_test`]), and nothing else: no clock, no counter, no state
//! carried between calls. Synthesis asks it the same question many times: every arrangement that
//! creates a row runs its creating command again, and a model whose rows own rows arranges each
//! owner once per scenario that needs one. Over `examples/billing` 124 of 332 searches repeated an
//! earlier one; over the #301 reference model, 4,491 of 9,576 in one pass.
//!
//! [`memoise`] holds each answer for as long as the model it was asked of is borrowed, so the
//! second asking is a copy of the first answer. The suite is unchanged by construction: the answer
//! returned is the one the search returned.
//!
//! A command is known by its content, not by its address ([`asked`]): the commands
//! `synthesize::caller` builds for each caller question, and the acting commands of a caller
//! reading searched against the arranging reading, are fresh values, and were searched again for
//! every question although many are equal in every field. Every debug build checks the key: a
//! sample of the answers given from memory is compared with the search run again ([`checked`]).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_domain::name::QualifiedName;
use ess_primitives::facts::FactPath;
use ess_primitives::predicate::Predicate;

use crate::witness::{Distinction, Searched};

thread_local! {
    /// The answers held, innermost model last. A stack because a synthesis of one model may
    /// synthesize another inside it (`synthesize::caller` reads the model once per caller
    /// assignment), and an answer about one model is no answer about another.
    static MEMO: RefCell<Vec<Memo>> = const { RefCell::new(Vec::new()) };
    #[cfg(test)]
    static WORK: RefCell<Option<Work>> = const { RefCell::new(None) };
    #[cfg(test)]
    static GENERATIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// The answers one model's searches gave, by command, distinction and midpoint flag, then by the
/// guards searched over. `Predicate` has no order, so the last level is compared one by one; it
/// holds the few guard sets one command is searched with.
struct Memo {
    /// The model, compared by address only. It is borrowed for as long as this entry is on the
    /// stack ([`Memoised`]), so the address cannot be reused by another model meanwhile.
    ir: *const EssIr,
    /// The `Debug` rendering of the model's own declaration of each name asked about by a command
    /// that is not that declaration ([`asked`]), made once.
    declared: BTreeMap<QualifiedName, String>,
    /// Every command asked about that is not the model's own declaration of its name, by name,
    /// each with its `Debug` rendering: [`Asked::Other`] is a position in its name's list.
    others: BTreeMap<QualifiedName, Vec<(ResolvedCommand, String)>>,
    /// Each answer with the guards it was searched over, and whether it was searched for the
    /// model's own declaration found by its address, the only case the memory had before
    /// commands were told apart by content.
    answers: BTreeMap<Question, Vec<(Vec<Predicate>, Searched, bool)>>,
    /// Which memory this is, counted on the thread: two models one after another may share an
    /// address, and the test counter tells their searches apart by this.
    #[cfg(test)]
    generation: u64,
}

impl Memo {
    fn new(ir: *const EssIr) -> Self {
        Self {
            ir,
            declared: BTreeMap::new(),
            others: BTreeMap::new(),
            answers: BTreeMap::new(),
            #[cfg(test)]
            generation: GENERATIONS.with(|count| {
                count.set(count.get() + 1);
                count.get()
            }),
        }
    }
}

/// Which command of its name a search was asked about ([`asked`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Asked {
    /// The model's own declaration, or a command equal to it in every field.
    Declared,
    /// The command at this position of [`Memo::others`] for the name.
    Other(usize),
}

/// The command, which of its name it is, the distinction and midpoint flag a search was asked
/// with, and the input paths the branch under test copies where it is a branch of that command
/// ([`crate::witness::under_test`]).
type Question = (QualifiedName, Asked, Distinction, bool, BTreeSet<FactPath>);

/// Holds witness answers for one model until dropped. See [`memoise`].
pub(crate) struct Memoised<'ir> {
    pushed: bool,
    model: PhantomData<&'ir EssIr>,
}

impl Drop for Memoised<'_> {
    fn drop(&mut self) {
        if self.pushed {
            let top = MEMO.with(|memo| memo.borrow_mut().pop());
            if let Some(top) = top {
                park(top);
            }
        }
    }
}

thread_local! {
    /// The models whose answers outlast each synthesis of them ([`Keeping`]), innermost scope
    /// last, by address: the answers while they are parked, `None` while the model is held.
    static KEPT: RefCell<Vec<BTreeMap<usize, Option<Memo>>>> = const { RefCell::new(Vec::new()) };
}

/// Keeps the answers about each model it holds past every synthesis of that model, until dropped:
/// a caller reading synthesized whole and again per command it focuses on asks the same searches
/// of the same model (`synthesize::caller`). The guard holds the models, so no address it keeps can
/// be taken by another model while their answers are parked.
pub(crate) struct Keeping {
    models: Vec<std::sync::Arc<EssIr>>,
    /// A Keeping pops the stack of the thread that opened it, so it is not sent to another thread.
    _thread: std::marker::PhantomData<*const ()>,
}

impl Keeping {
    /// A new innermost scope, holding no model yet.
    pub(crate) fn open() -> Self {
        KEPT.with(|kept| kept.borrow_mut().push(BTreeMap::new()));
        Self {
            models: Vec::new(),
            _thread: std::marker::PhantomData,
        }
    }

    /// Keeps the answers about `ir` for as long as this guard lives.
    pub(crate) fn keep(&mut self, ir: &std::sync::Arc<EssIr>) {
        KEPT.with(|kept| {
            if let Some(scope) = kept.borrow_mut().last_mut() {
                scope.entry(address(ir)).or_insert(None);
            }
        });
        self.models.push(std::sync::Arc::clone(ir));
    }
}

impl Drop for Keeping {
    fn drop(&mut self) {
        // The answers go before the models they are about.
        KEPT.with(|kept| {
            kept.borrow_mut().pop();
        });
    }
}

fn address(ir: &EssIr) -> usize {
    std::ptr::from_ref(ir) as usize
}

/// `ir`'s parked answers, held innermost again, where [`Keeping`] holds it.
fn recall(ir: &EssIr) -> bool {
    let parked = KEPT.with(|kept| {
        kept.borrow_mut()
            .last_mut()
            .and_then(|scope| scope.get_mut(&address(ir)))
            .and_then(Option::take)
    });
    let Some(parked) = parked else {
        return false;
    };
    MEMO.with(|memo| memo.borrow_mut().push(parked));
    true
}

/// Parks `memo` where [`Keeping`] holds its model; drops it otherwise.
fn park(memo: Memo) {
    KEPT.with(|kept| {
        if let Some(slot) = kept
            .borrow_mut()
            .last_mut()
            .and_then(|scope| scope.get_mut(&(memo.ir as usize)))
        {
            *slot = Some(memo);
        }
    });
}

/// Answers each witness search about `ir` once while the result is held, and again from the first
/// answer after that. Nested for the same model, the outer holder keeps answering.
pub(crate) fn memoise(ir: &EssIr) -> Memoised<'_> {
    if recall(ir) {
        return Memoised {
            pushed: true,
            model: PhantomData,
        };
    }
    let at = std::ptr::from_ref(ir);
    let pushed = MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        if memo.last().is_some_and(|top| std::ptr::eq(top.ir, at)) {
            return false;
        }
        memo.push(Memo::new(at));
        true
    });
    Memoised {
        pushed,
        model: PhantomData,
    }
}

/// Which command `command` is among those asked of the held model, or `None` where no model is
/// held or the one held is not `ir`: then the search is not answered from memory.
///
/// The model's own declaration of the name, by address, is [`Asked::Declared`] at no cost. Any
/// other value is told apart by its whole content: its `Debug` rendering, which shows every field
/// down to a number's spelling and the sign of a zero that `==` treats as one value, and `==`
/// beside it. A command equal in both to the declaration is the declaration; one equal to a
/// command asked before is that command. So a command `synthesize::caller` builds afresh for each
/// question, or one read from another caller reading of the same model, shares the answers of
/// every equal one, and two commands differing anywhere never do. The flag says whether
/// `command` is the declaration by address.
fn asked(ir: &EssIr, command: &ResolvedCommand) -> Option<(Asked, bool)> {
    let declared = ir.commands().get(&command.name);
    MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        let top = memo.last_mut().filter(|top| std::ptr::eq(top.ir, ir))?;
        if declared.is_some_and(|declared| std::ptr::eq(declared, command)) {
            return Some((Asked::Declared, true));
        }
        let shown = format!("{command:?}");
        if let Some(declared) = declared {
            let declared_shown = top
                .declared
                .entry(command.name.clone())
                .or_insert_with(|| format!("{declared:?}"));
            if *declared_shown == shown && declared == command {
                return Some((Asked::Declared, false));
            }
        }
        let others = top.others.entry(command.name.clone()).or_default();
        if let Some(at) = others
            .iter()
            .position(|(other, other_shown)| *other_shown == shown && other == command)
        {
            return Some((Asked::Other(at), false));
        }
        others.push((command.clone(), shown));
        Some((Asked::Other(others.len() - 1), false))
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
    let Some((asked, by_address)) = asked(ir, command) else {
        executed(ir, command, guards, distinction, between);
        return search();
    };
    let key = (
        command.name.clone(),
        asked,
        distinction,
        between,
        crate::witness::copied_under_test(command),
    );
    let same = |asked: &[Predicate]| {
        asked.len() == guards.len() && asked.iter().zip(guards).all(|(a, b)| a == *b)
    };
    let known = MEMO.with(|memo| {
        memo.borrow().last().and_then(|top| {
            top.answers.get(&key).and_then(|asked| {
                asked
                    .iter()
                    .find(|(searched, _, _)| same(searched))
                    .map(|(_, answer, stored_by_address)| (answer.clone(), *stored_by_address))
            })
        })
    });
    if let Some((known, stored_by_address)) = known {
        // An answer stored and asked for the declaration found by address is the memory as it was
        // before commands were told apart by content; every other is checked, sampled.
        #[cfg(debug_assertions)]
        if (!by_address || !stored_by_address) && sampled() {
            checked(command, &known, &search());
        }
        #[cfg(not(debug_assertions))]
        let _ = stored_by_address;
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
                by_address,
            ));
        }
    });
    answer
}

/// How many answers from memory across two command values a thread has every one of checked by
/// the debug guard ([`checked`]); after these, one in [`CHECK_EVERY`]. Counted per thread, so
/// every test checks its first ones, and a test synthesizing many models, each reading of which is
/// a memory of its own, does not search most of them twice.
#[cfg(debug_assertions)]
const CHECK_FIRST: usize = 64;

/// After [`CHECK_FIRST`], every how many such answers of one thread are checked.
#[cfg(debug_assertions)]
const CHECK_EVERY: usize = 64;

#[cfg(debug_assertions)]
thread_local! {
    /// How many answers from memory across two command values this thread gave ([`sampled`]).
    static SHARED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Whether this answer from memory is one the debug guard checks, counting it.
#[cfg(debug_assertions)]
fn sampled() -> bool {
    SHARED.with(|shared| {
        shared.set(shared.get() + 1);
        shared.get() <= CHECK_FIRST || shared.get() % CHECK_EVERY == 0
    })
}

/// The key-completeness guard of every debug build and test: an answer from memory equals the
/// search run again, down to its `Debug` rendering. A key that missed something the search reads
/// gives a later question the answer of an earlier, different one, and fails here.
#[cfg(debug_assertions)]
fn checked(command: &ResolvedCommand, known: &Searched, again: &Searched) {
    let (known, again) = (format!("{known:?}"), format!("{again:?}"));
    assert!(
        known == again,
        "a witness search about `{}` answered from memory differs from the same search run \
         again, so the memory's key misses something the search reads:\nremembered: \
         {known}\nsearched again: {again}",
        command.name
    );
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
                generation: MEMO.with(|memo| {
                    memo.borrow()
                        .last()
                        .filter(|top| std::ptr::eq(top.ir, ir))
                        .map_or(0, |top| top.generation)
                }),
                command: format!("{command:?}"),
                distinction,
                between,
                copied: crate::witness::copied_under_test(command),
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
    generation: u64,
    /// The command's whole content, as its `Debug` rendering.
    command: String,
    distinction: Distinction,
    between: bool,
    copied: BTreeSet<FactPath>,
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
    // `WORK` and `GENERATIONS` are per thread, and a key holds the model's address.
    let out = crate::synthesize::with_jobs(std::num::NonZeroUsize::MIN, run);
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

    /// Tickets a desk opens, owned by the opening caller's desk: every command but `Open` refuses
    /// a caller who is not the owner, and `Open` copies a list of structs and a struct whole.
    const DESK: &str = "format: ess/37\nsystem: shop\nversion: v1\ndomain: shop.desk\n\
        summary: Tickets owned by the desk that opened them.\n\
        types:\n\
        \x20 - {name: shop.desk.TicketId, kind: newtype, of: String}\n\
        \x20 - {name: shop.desk.DeskId, kind: newtype, of: String}\n\
        \x20 - name: shop.desk.Line\n    kind: struct\n    fields:\n\
        \x20     - {name: sku, type: String}\n      - {name: qty, type: Integer}\n\
        \x20     - {name: memo, type: \"Optional<String>\"}\n\
        \x20 - name: shop.desk.Place\n    kind: struct\n    fields:\n\
        \x20     - {name: street, type: String}\n      - {name: zip, type: String}\n\
        entities:\n  - name: shop.desk.Ticket\n    identity: {name: ticket_id, type: shop.desk.TicketId}\n\
        \x20   fields:\n      - {name: desk_id, type: shop.desk.DeskId}\n\
        \x20     - {name: lines, type: \"List<shop.desk.Line>\"}\n\
        \x20     - {name: place, type: shop.desk.Place}\n\
        \x20   lifecycle:\n      initial: Open\n      states: [Open, Closed]\n      terminal: [Closed]\n\
        \x20     transitions:\n        - {name: move, from: [Open], to: Open}\n\
        \x20       - {name: close, from: [Open], to: Closed}\n\
        views:\n  - name: shop.desk.Tickets\n    source: shop.desk.Ticket\n    consistency: read_your_writes\n\
        \x20   fields:\n      - {name: ticket_id, type: shop.desk.TicketId}\n\
        \x20     - {name: desk_id, type: shop.desk.DeskId}\n\
        \x20     - {name: lines, type: \"List<shop.desk.Line>\"}\n\
        \x20     - {name: place, type: shop.desk.Place}\n\
        \x20     - {name: state, type: shop.desk.Ticket.State}\n\
        actors:\n  - name: shop.desk.Clerk\n    attributes:\n      - {name: desk_id, type: shop.desk.DeskId}\n\
        \x20   may: [shop.desk.Open, shop.desk.Move, shop.desk.Close]\n\
        errors:\n  - {name: shop.desk.NoSuchTicket, fields: [{name: ticket_id, type: shop.desk.TicketId}]}\n\
        \x20 - {name: shop.desk.NotOwner, fields: [{name: ticket_id, type: shop.desk.TicketId}]}\n\
        \x20 - {name: shop.desk.StateConflict, fields: [{name: ticket_id, type: shop.desk.TicketId}]}\n\
        events:\n  - {name: shop.desk.Opened, fields: [{name: ticket_id, type: shop.desk.TicketId}]}\n\
        \x20 - {name: shop.desk.Moved, fields: [{name: ticket_id, type: shop.desk.TicketId}, {name: place, type: shop.desk.Place}]}\n\
        \x20 - {name: shop.desk.Closed, fields: [{name: ticket_id, type: shop.desk.TicketId}]}\n\
        commands:\n  - name: shop.desk.Open\n    input:\n      - {name: ticket_id, type: shop.desk.TicketId}\n\
        \x20     - {name: lines, type: \"List<shop.desk.Line>\"}\n      - {name: place, type: shop.desk.Place}\n\
        \x20   outcomes:\n      - name: opened\n        creates: shop.desk.Ticket\n        instance: ticket_id\n\
        \x20       sets: {desk_id: {caller: desk_id}, lines: input.lines, place: input.place}\n\
        \x20       emits: [shop.desk.Opened]\n        payload: {shop.desk.Opened: {ticket_id: input.ticket_id}}\n\
        \x20 - name: shop.desk.Move\n    input:\n      - {name: ticket_id, type: shop.desk.TicketId}\n\
        \x20     - {name: place, type: shop.desk.Place}\n    outcomes:\n      - name: no-ticket\n\
        \x20       unknown_instance: true\n        error: shop.desk.NoSuchTicket\n      - name: not-owner\n\
        \x20       when_subject:\n          predicate: desk_id != caller.desk_id\n        error: shop.desk.NotOwner\n\
        \x20       payload: {shop.desk.NotOwner: {ticket_id: input.ticket_id}}\n      - name: moved\n\
        \x20       moves: shop.desk.Ticket.move\n        instance: ticket_id\n        sets: {place: input.place}\n\
        \x20       emits: [shop.desk.Moved]\n        payload: {shop.desk.Moved: {ticket_id: input.ticket_id, place: input.place}}\n\
        \x20     - name: wrong-state\n        wrong_state: true\n        error: shop.desk.StateConflict\n\
        \x20 - name: shop.desk.Close\n    input:\n      - {name: ticket_id, type: shop.desk.TicketId}\n\
        \x20   outcomes:\n      - name: no-ticket\n        unknown_instance: true\n        error: shop.desk.NoSuchTicket\n\
        \x20     - name: not-owner\n        when_subject:\n          predicate: desk_id != caller.desk_id\n\
        \x20       error: shop.desk.NotOwner\n        payload: {shop.desk.NotOwner: {ticket_id: input.ticket_id}}\n\
        \x20     - name: closed\n        moves: shop.desk.Ticket.close\n        instance: ticket_id\n\
        \x20       emits: [shop.desk.Closed]\n        payload: {shop.desk.Closed: {ticket_id: input.ticket_id}}\n\
        \x20     - name: wrong-state\n        wrong_state: true\n        error: shop.desk.StateConflict\n";

    fn compiled(text: &str) -> EssIr {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
        let specification = Specification::assemble([(Source::new("desk.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
        compile(&specification, &SourceMap::new())
            .unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
    }

    /// The same work guard over a model whose actors carry an attribute its commands compare:
    /// synthesis reads it once per caller assignment, and the commands of one reading are
    /// searched against another's model and rebuilt for each caller question, fresh values each
    /// time. A command equal in every field to one already searched over the same model is
    /// answered from that search, so no search of one model runs twice for one question. On the
    /// base, which knew a command only by address, 62 of 638 searches here repeated one.
    #[test]
    fn caller_readings_run_each_witness_search_of_a_model_once() {
        let ir = compiled(DESK);
        let (synthesis, executed, repeated) = counted(|| synthesize(&ir));
        assert!(
            synthesis.refusals.is_empty(),
            "the desk synthesizes without refusals: {:#?}",
            synthesis.refusals
        );
        assert!(executed > 0, "the desk ran no witness search at all");
        assert_eq!(
            repeated, 0,
            "{repeated} of {executed} witness searches repeated a question already answered \
             about the same model"
        );
    }
}
