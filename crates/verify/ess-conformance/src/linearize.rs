//! Is there an order of the recorded operations the specification's own model accepts?
//!
//! `ess verify conform check-history` reads an `ess-history/1` document ([`crate::history`]) and
//! searches for a sequential order of its operations that the interpreter
//! ([`crate::interpret::execute`]) accepts, answer for answer. This is the only checker in ESS: the
//! Go and TypeScript explorers record histories and call it (design decision 1).
//!
//! # The search
//!
//! Wing and Gong's, with Lowe's cache, as Porcupine arranges it (read as an algorithm reference; no
//! code is taken, design decision 2):
//!
//! * An operation may come next in the order when no operation still outside the order returned
//!   before it was invoked. An operation that never answered returns after every other one
//!   ([`ReturnBound::AfterEveryOther`], decision 4), so it never holds another back.
//! * The model is **nondeterministic**: one step can leave more than one next state — an
//!   `external:` branch ([`Externals::Open`]), and every input the history does not record (below).
//!   Each next state is a branch of the search.
//! * A state already reached with the same set of operations ordered is not searched again.
//! * The history is **partitioned by subject** (P-compositionality): the interpreter's store keys
//!   every instance by its identity and no step reads another instance, so operations on different
//!   subjects commute and each subject's operations are searched alone. The history is linearizable
//!   exactly when every partition is.
//!
//! # What a returned operation must answer
//!
//! A `Returned` operation's step must take the branch it recorded, by outcome name. An
//! `Indeterminate` operation's step may take any branch — or none, because a call that never
//! answered may never have happened.
//!
//! # What `ess-history/1` does not record, and how the search reads it
//!
//! An operation records its command, its subject and its outcome, and **not its input**. So the
//! search asks whether *some* input explains it: the candidate inputs [`witness::candidates`] builds
//! for the command from its declared input types and the literals its guards write — the same ones
//! synthesis submits — with the field that names the subject set to the recorded `subject_key`.
//! Every distinct next state any of them reaches is a branch. A creating command's new identity is
//! the `subject_key`, given to the interpreter as the value of the event field `instance:` names
//! ([`Generated::Recorded`]); every other value the implementation assigns is left to the model's own
//! counter.
//!
//! Because `ess-history/1` records no inputs, neither verdict is exact. A `Violation` says no
//! candidate input explains the history, and an input outside the candidates might have. A
//! `Linearizable` says some candidate input explains every answer — not that the input the client
//! actually sent does, so a fault that shows only in how an answer depends on its input (an amount
//! refused that should have been accepted, say) can be judged `Linearizable`. The witness strategy
//! is the same bounded one the synthesized suite already stands on; recording inputs is a change
//! to the format, not to this checker.
//!
//! # Views
//!
//! This story checks reads of views declared `Current`. `ess/…` declares no such level — a view is
//! `read_your_writes` or `eventual` ([`Consistency`]) — so every operation that names a view is
//! **not judged here**: it is taken out of the search and listed in [`Checked::not_judged`] with
//! the level it declares, for `story:session-and-eventual-view-checks`. The match over
//! [`Consistency`] is exhaustive, so a level added later has to be decided here.
//!
//! # The budget
//!
//! One step of the budget is one execution of the model: one operation tried from one state.
//! Nothing reads a clock, so one history and one budget give one verdict and one shrunk history,
//! byte for byte. A search that spends its budget is [`Verdict::Unknown`], which is never a pass
//! (decision 5).
//!
//! # Shrinking
//!
//! A violation is shrunk to a smaller history that is still the same violation: first to the subject
//! partition that decided it, then whole clients are removed one at a time, then single operations,
//! until no removal is left that keeps it. Each trial is a full check with the same budget, and
//! [`shrink`] says what "the same" means — a removal that turns the violation into a different one
//! (a call on an instance whose creation was taken out) is not kept.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};

use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedEffect, ResolvedInstance};
use ess_domain::name::QualifiedName as ModelName;
use ess_domain::view::Consistency;
use ess_primitives::node::Node;
use serde::Serialize;

use crate::history::{Completion, History, Operation, ReturnBound, Verdict};
use crate::interpret::execute::{
    execute_generating, Externals, Generated, GeneratedSlot, Store, Undetermined,
};
use crate::witness::{self, Distinction};

/// The search budget when none is named: this many executions of the model.
pub const DEFAULT_BUDGET: u64 = 1_000_000;

/// A read this check does not judge, and the level its view declares.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotJudged {
    /// The operation.
    pub operation_id: String,
    /// The view it reads.
    pub view: String,
    /// The consistency the view declares, as written in a document.
    pub consistency: String,
}

/// What searching one history concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    /// The verdict.
    pub verdict: Verdict,
    /// How many of the budget's steps the search spent.
    pub steps: u64,
    /// The subject whose partition decided a `Violation` or an `Unknown`.
    pub subject_key: Option<String>,
    /// For a `Violation` or an `Unknown`: the longest order of that partition's operations the
    /// model accepted, by operation id. Empty for `Linearizable`.
    pub linearization: Vec<String>,
    /// How many subject partitions were searched.
    pub partitions: usize,
    /// The operations taken out of the search, in document order.
    pub not_judged: Vec<NotJudged>,
}

/// Why a history cannot be checked against this model at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckRefusal {
    /// An operation names neither a command nor a view the model declares.
    UnknownOperation {
        /// The operation.
        operation_id: String,
        /// What it names.
        command: String,
    },
    /// No candidate input can be built for a command.
    NoInput {
        /// The command.
        command: String,
        /// Why, from the witness strategy.
        why: String,
    },
    /// The interpreter cannot decide a step the search needs.
    Model {
        /// The operation being tried.
        operation_id: String,
        /// What the interpreter said.
        why: String,
    },
}

impl CheckRefusal {
    /// The refusal's stable name.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownOperation { .. } => "check.unknown-operation",
            Self::NoInput { .. } => "check.no-input",
            Self::Model { .. } => "check.model-undetermined",
        }
    }
}

impl fmt::Display for CheckRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::UnknownOperation {
                operation_id,
                command,
            } => write!(
                formatter,
                "{code}: operation {operation_id} names `{command}`, which the model declares as \
                 neither a command nor a view"
            ),
            Self::NoInput { command, why } => write!(
                formatter,
                "{code}: no candidate input can be built for `{command}`: {why}"
            ),
            Self::Model { operation_id, why } => write!(
                formatter,
                "{code}: the model cannot decide operation {operation_id}: {why}"
            ),
        }
    }
}

impl std::error::Error for CheckRefusal {}

// ---- preparing the operations --------------------------------------------------------------------

/// One judged operation, with everything a step of it needs.
struct Prepared<'h> {
    operation: &'h Operation,
    name: ModelName,
    inputs: Vec<BTreeMap<String, Node>>,
    generated: Generated,
}

/// The model-side reading of every operation: judged ones by subject, not-judged ones listed.
struct Split<'h> {
    partitions: BTreeMap<&'h str, Vec<Prepared<'h>>>,
    not_judged: Vec<NotJudged>,
}

fn split<'h>(ir: &'h EssIr, history: &'h History) -> Result<Split<'h>, CheckRefusal> {
    let mut inputs: BTreeMap<&ModelName, Vec<BTreeMap<String, Node>>> = BTreeMap::new();
    let mut partitions: BTreeMap<&str, Vec<Prepared<'h>>> = BTreeMap::new();
    let mut not_judged = Vec::new();
    for operation in &history.operations {
        let unknown = || CheckRefusal::UnknownOperation {
            operation_id: operation.operation_id.as_str().to_owned(),
            command: operation.command.as_str().to_owned(),
        };
        let name = ModelName::new(operation.command.as_str()).map_err(|_| unknown())?;
        if let Some(view) = ir.views().get(&name) {
            // Exhaustive on purpose: a level added to the language is decided here, not defaulted.
            match view.consistency {
                Consistency::ReadYourWrites | Consistency::Eventual => {
                    not_judged.push(NotJudged {
                        operation_id: operation.operation_id.as_str().to_owned(),
                        view: name.to_string(),
                        consistency: view.consistency.as_str().to_owned(),
                    });
                    continue;
                }
            }
        }
        let (key, command) = ir.commands().get_key_value(&name).ok_or_else(unknown)?;
        if !inputs.contains_key(key) {
            inputs.insert(key, candidates(ir, command)?);
        }
        let subject = Node::Text(operation.subject_key.clone());
        let inputs = inputs[key]
            .iter()
            .map(|input| {
                let mut input = input.clone();
                for field in supplied(command) {
                    input.insert(field.to_owned(), subject.clone());
                }
                input
            })
            .fold(Vec::new(), |mut distinct, input| {
                if !distinct.contains(&input) {
                    distinct.push(input);
                }
                distinct
            });
        let generated = Generated::Recorded(
            observed(command)
                .map(|slot| (slot, subject.clone()))
                .collect(),
        );
        partitions
            .entry(operation.subject_key.as_str())
            .or_default()
            .push(Prepared {
                operation,
                name: key.clone(),
                inputs,
                generated,
            });
    }
    Ok(Split {
        partitions,
        not_judged,
    })
}

/// Every candidate input the witness strategy builds for `command`.
fn candidates(
    ir: &EssIr,
    command: &ResolvedCommand,
) -> Result<Vec<BTreeMap<String, Node>>, CheckRefusal> {
    let guards: Vec<_> = command.outcomes.iter().filter_map(crate::when).collect();
    let built = witness::candidates(ir, command, &guards, Distinction::PLAIN).map_err(|gap| {
        CheckRefusal::NoInput {
            command: command.name.to_string(),
            why: gap.to_string(),
        }
    })?;
    if built.is_empty() {
        return Err(CheckRefusal::NoInput {
            command: command.name.to_string(),
            why: "the declared input types admit none of the bounded candidates".to_owned(),
        });
    }
    Ok(built)
}

/// The input fields any branch of `command` names its subject in.
fn supplied(command: &ResolvedCommand) -> BTreeSet<&str> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .filter_map(|subject| match &subject.instance {
            ResolvedInstance::Supplied { field } => Some(field.name.as_str()),
            ResolvedInstance::Observed { .. } => None,
        })
        .collect()
}

/// The event fields any creating branch of `command` publishes its new identity in.
fn observed(command: &ResolvedCommand) -> impl Iterator<Item = GeneratedSlot> + '_ {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .filter_map(|subject| match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, ResolvedInstance::Observed { event, field }) => {
                Some(GeneratedSlot::new(event.name().clone(), field.name.clone()))
            }
            _ => None,
        })
}

// ---- one step --------------------------------------------------------------------------------

/// Every state the model can be in after `prepared` from `store`, answering what it recorded.
fn step(ir: &EssIr, store: &Store, prepared: &Prepared<'_>) -> Result<Vec<Store>, CheckRefusal> {
    let operation = prepared.operation;
    let mut next: Vec<Store> = Vec::new();
    if operation.completion == Completion::Indeterminate {
        // It may never have happened.
        next.push(store.clone());
    }
    for input in &prepared.inputs {
        let steps = match execute_generating(
            ir,
            store,
            &prepared.name,
            input,
            &Externals::Open,
            &prepared.generated,
        ) {
            Ok(steps) => steps,
            // The request is not one the model's branches describe from here — an identity already
            // held, a subject key that is not of the identity's type: this input explains nothing.
            Err(Undetermined::Request(_)) => continue,
            Err(why) => {
                return Err(CheckRefusal::Model {
                    operation_id: operation.operation_id.as_str().to_owned(),
                    why: why.to_string(),
                })
            }
        };
        for taken in steps {
            let answers = match (&operation.completion, &operation.outcome) {
                (Completion::Returned, Some(recorded)) => taken
                    .outcome
                    .as_ref()
                    .is_some_and(|outcome| outcome.outcome.as_str() == recorded.as_str()),
                _ => true,
            };
            if answers && !next.contains(&taken.next) {
                next.push(taken.next);
            }
        }
    }
    Ok(next)
}

// ---- the search ------------------------------------------------------------------------------

/// A set of operation indices, as a key the cache can hold.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Done(Vec<u64>);

impl Done {
    fn new(size: usize) -> Self {
        Self(vec![0; size.div_ceil(64)])
    }
    fn has(&self, index: usize) -> bool {
        self.0[index / 64] & (1 << (index % 64)) != 0
    }
    fn with(&self, index: usize) -> Self {
        let mut next = self.clone();
        next.0[index / 64] |= 1 << (index % 64);
        next
    }
}

/// How one partition's search ended.
enum Outcome {
    Linearizable,
    Violation(Vec<usize>),
    Unknown(Vec<usize>),
}

/// One level of the depth-first search: the state reached, and the moves from it not yet tried.
struct Frame {
    done: Done,
    moves: Vec<(usize, Store)>,
    next: usize,
}

/// Searches one partition, spending from `budget`.
fn search(
    ir: &EssIr,
    operations: &[Prepared<'_>],
    budget: &mut u64,
    spent: &mut u64,
) -> Result<Outcome, CheckRefusal> {
    // Invoke order, then document order, so the moves are tried in one fixed order.
    let mut order: Vec<usize> = (0..operations.len()).collect();
    order.sort_by_key(|&index| (operations[index].operation.invoked_at, index));
    let total = operations.len();

    let mut seen: BTreeMap<Done, Vec<Store>> = BTreeMap::new();
    let mut path: Vec<usize> = Vec::new();
    let mut longest: Vec<usize> = Vec::new();

    // The moves from one state: every operation nothing still outside the order returned before.
    let moves = |done: &Done, store: &Store, budget: &mut u64, spent: &mut u64| {
        let mut found: Vec<(usize, Store)> = Vec::new();
        for &index in &order {
            if done.has(index) {
                continue;
            }
            let invoked = operations[index].operation.invoked_at;
            let held_back = order.iter().any(|&other| {
                other != index
                    && !done.has(other)
                    && operations[other].operation.return_bound() < ReturnBound::At(invoked)
            });
            if held_back {
                continue;
            }
            if *budget == 0 {
                return Ok(None);
            }
            *budget -= 1;
            *spent += 1;
            for next in step(ir, store, &operations[index])? {
                found.push((index, next));
            }
        }
        Ok::<_, CheckRefusal>(Some(found))
    };

    let start = Done::new(total);
    let Some(first) = moves(&start, &Store::default(), budget, spent)? else {
        return Ok(Outcome::Unknown(longest));
    };
    if total == 0 {
        return Ok(Outcome::Linearizable);
    }
    let mut stack = vec![Frame {
        done: start,
        moves: first,
        next: 0,
    }];
    while let Some(frame) = stack.last_mut() {
        let Some((index, store)) = frame.moves.get(frame.next).cloned() else {
            stack.pop();
            path.pop();
            continue;
        };
        frame.next += 1;
        let done = frame.done.with(index);
        let known = seen.entry(done.clone()).or_default();
        if known.contains(&store) {
            continue;
        }
        known.push(store.clone());
        path.push(index);
        if path.len() > longest.len() {
            longest.clone_from(&path);
        }
        if path.len() == total {
            return Ok(Outcome::Linearizable);
        }
        let Some(found) = moves(&done, &store, budget, spent)? else {
            return Ok(Outcome::Unknown(longest));
        };
        stack.push(Frame {
            done,
            moves: found,
            next: 0,
        });
    }
    Ok(Outcome::Violation(longest))
}

/// Searches `history` for an order of its operations the model of `ir` accepts, spending at most
/// `budget` executions of the model.
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
pub fn check(ir: &EssIr, history: &History, budget: u64) -> Result<Checked, CheckRefusal> {
    let split = split(ir, history)?;
    let mut remaining = budget;
    let mut spent = 0;
    let mut unknown: Option<(String, Vec<String>)> = None;
    let ids = |operations: &[Prepared<'_>], indices: Vec<usize>| -> Vec<String> {
        indices
            .into_iter()
            .map(|index| operations[index].operation.operation_id.as_str().to_owned())
            .collect()
    };
    for (subject_key, operations) in &split.partitions {
        match search(ir, operations, &mut remaining, &mut spent)? {
            Outcome::Linearizable => {}
            Outcome::Violation(longest) => {
                return Ok(Checked {
                    verdict: Verdict::Violation,
                    steps: spent,
                    subject_key: Some((*subject_key).to_owned()),
                    linearization: ids(operations, longest),
                    partitions: split.partitions.len(),
                    not_judged: split.not_judged,
                });
            }
            Outcome::Unknown(longest) => {
                if unknown.is_none() {
                    unknown = Some(((*subject_key).to_owned(), ids(operations, longest)));
                }
            }
        }
    }
    let (verdict, subject_key, linearization) = match unknown {
        Some((subject_key, longest)) => (Verdict::Unknown, Some(subject_key), longest),
        None => (Verdict::Linearizable, None, Vec::new()),
    };
    Ok(Checked {
        verdict,
        steps: spent,
        subject_key,
        linearization,
        partitions: split.partitions.len(),
        not_judged: split.not_judged,
    })
}

// ---- shrinking -------------------------------------------------------------------------------

/// `history` holding only the operations `keep` admits, in document order.
fn only(history: &History, keep: impl Fn(&Operation) -> bool) -> History {
    let mut kept = history.clone();
    kept.operations.retain(|operation| keep(operation));
    kept
}

/// How many judged operations of the partition that decided a violation no order explains: that
/// partition's operations less its longest partial linearization.
///
/// Independent of the order the search tries moves in. A violation is only reported after the
/// search has reached every state it can, so the longest linearization is the longest there is; its
/// length, unlike which operations it holds, is a property of the history.
fn unexplained(history: &History, checked: &Checked) -> usize {
    let not_judged: BTreeSet<&str> = checked
        .not_judged
        .iter()
        .map(|read| read.operation_id.as_str())
        .collect();
    let judged = history
        .operations
        .iter()
        .filter(|operation| {
            Some(operation.subject_key.as_str()) == checked.subject_key.as_deref()
                && !not_judged.contains(operation.operation_id.as_str())
        })
        .count();
    judged.saturating_sub(checked.linearization.len())
}

/// The unexplained count of `candidate` when it is still a violation with no more unexplained
/// operations than `bound`.
fn no_worse(
    ir: &EssIr,
    candidate: &History,
    bound: usize,
    budget: u64,
) -> Result<Option<usize>, CheckRefusal> {
    if candidate.operations.is_empty() {
        return Ok(None);
    }
    let checked = check(ir, candidate, budget)?;
    if checked.verdict != Verdict::Violation {
        return Ok(None);
    }
    let count = unexplained(candidate, &checked);
    Ok((count <= bound).then_some(count))
}

/// `true` when `earlier` returned before `later` was invoked.
fn precedes(earlier: &Operation, later: &Operation) -> bool {
    earlier.return_bound() < ReturnBound::At(later.invoked_at)
}

/// `current` without the operations `drop` names, where none of them precedes an operation that
/// stays; `None` otherwise.
fn without(current: &History, drop: impl Fn(&Operation) -> bool) -> Option<History> {
    let (gone, kept): (Vec<&Operation>, Vec<&Operation>) = current
        .operations
        .iter()
        .partition(|operation| drop(operation));
    let needed = gone
        .iter()
        .any(|removed| kept.iter().any(|stays| precedes(removed, stays)));
    (!gone.is_empty() && !needed).then(|| only(current, |operation| !drop(operation)))
}

/// A smaller history that is still the violation `history` is.
///
/// Three moves, each kept only while what is left is still that violation:
///
/// 1. every operation outside the subject partition that decided the violation is dropped, which
///    P-compositionality makes safe, and so is every read not judged here;
/// 2. whole clients are removed one at a time;
/// 3. single operations are removed one at a time, until no one more can be.
///
/// "Still that violation" is more than "still a violation". Taking out the creation turns every
/// later call on the instance into a call on an instance nobody created, which is a violation too,
/// and a different one: a shrinker that took it would report a single payment of an invoice that
/// was never created and lose the race it was asked to show. Two rules keep it, and neither depends
/// on the order the search tries its moves in:
///
/// * an operation is removed only when no operation that stays was invoked after it returned — the
///   setup a race runs after goes only once the race itself is gone, as a program's earlier calls
///   are kept while a later one needs them;
/// * a removal is kept only when the result is still a violation and the number of operations no
///   order explains has not grown since the last removal kept.
///
/// Moves 2 and 3 repeat until neither removes anything. Last, the sequential prefix — the setup
/// every client raced after — is moved onto a client that is left, and the clients are numbered
/// from 0 in their original order, where the result is still that violation. A history that is not
/// a violation is returned unchanged.
///
/// # Errors
///
/// [`CheckRefusal`] where a trial cannot be checked.
pub fn shrink(ir: &EssIr, history: &History, budget: u64) -> Result<History, CheckRefusal> {
    let checked = check(ir, history, budget)?;
    if checked.verdict != Verdict::Violation {
        return Ok(history.clone());
    }
    let mut bound = unexplained(history, &checked);
    let not_judged: BTreeSet<&str> = checked
        .not_judged
        .iter()
        .map(|read| read.operation_id.as_str())
        .collect();
    let mut current = history.clone();
    let partition = only(history, |operation| {
        Some(operation.subject_key.as_str()) == checked.subject_key.as_deref()
            && !not_judged.contains(operation.operation_id.as_str())
    });
    if let Some(count) = no_worse(ir, &partition, bound, budget)? {
        current = partition;
        bound = count;
    }
    loop {
        let mut removed = false;
        let clients: BTreeSet<u64> = current.operations.iter().map(|it| it.client).collect();
        for client in clients {
            let Some(candidate) = without(&current, |operation| operation.client == client) else {
                continue;
            };
            if let Some(count) = no_worse(ir, &candidate, bound, budget)? {
                current = candidate;
                bound = count;
                removed = true;
                break;
            }
        }
        if removed {
            continue;
        }
        let ids: Vec<String> = current
            .operations
            .iter()
            .map(|operation| operation.operation_id.as_str().to_owned())
            .collect();
        for id in ids {
            let Some(candidate) =
                without(&current, |operation| operation.operation_id.as_str() == id)
            else {
                continue;
            };
            if let Some(count) = no_worse(ir, &candidate, bound, budget)? {
                current = candidate;
                bound = count;
                removed = true;
            }
        }
        if !removed {
            break;
        }
    }
    let relabelled = relabel(&current);
    if no_worse(ir, &relabelled, bound, budget)?.is_some() {
        current = relabelled;
    }
    Ok(current)
}

/// `history` with its sequential prefix moved onto one remaining client, and its clients numbered
/// densely from 0 in their original order.
///
/// The sequential prefix is the operations, in invoke order, each of which returned before the next
/// was invoked — the setup a recorder runs before its clients race. The search reads only instants,
/// never clients, so which client ran the prefix changes no verdict; moving it onto the client of
/// the first operation after it keeps every client to one call at a time, because the whole prefix
/// returned before that operation was invoked.
fn relabel(history: &History) -> History {
    let mut order: Vec<usize> = (0..history.operations.len()).collect();
    order.sort_by_key(|&index| (history.operations[index].invoked_at, index));
    let mut prefix = 0;
    while prefix < order.len() {
        let operation = &history.operations[order[prefix]];
        let next = order
            .get(prefix + 1)
            .map(|&index| history.operations[index].invoked_at);
        let before_next = match (operation.return_bound(), next) {
            (ReturnBound::At(returned), Some(invoked)) => returned < invoked,
            (ReturnBound::At(_), None) => true,
            (ReturnBound::AfterEveryOther, _) => false,
        };
        if !before_next {
            break;
        }
        prefix += 1;
    }
    let mut relabelled = history.clone();
    if let Some(&first) = order.get(prefix).or_else(|| order.first()) {
        let onto = history.operations[first].client;
        for &index in &order[..prefix] {
            relabelled.operations[index].client = onto;
        }
    }
    let clients: Vec<u64> = relabelled
        .operations
        .iter()
        .map(|it| it.client)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    for operation in &mut relabelled.operations {
        operation.client = clients
            .iter()
            .position(|client| *client == operation.client)
            .map_or(0, |position| position as u64);
    }
    relabelled.clients = (clients.len() as u64).max(1);
    relabelled
}

// ---- the report ------------------------------------------------------------------------------

/// What `check-history` prints: the verdict, the longest partial linearization, and for a
/// violation the shrunk history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The verdict.
    pub verdict: Verdict,
    /// The budget the search had, in executions of the model.
    pub budget: u64,
    /// How many of them it spent.
    pub steps: u64,
    /// How many operations the history holds.
    pub operations: usize,
    /// How many subject partitions were searched.
    pub partitions: usize,
    /// The subject whose partition decided a `Violation` or an `Unknown`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_key: Option<String>,
    /// The longest order of that partition's operations the model accepted, by operation id.
    pub linearization: Vec<String>,
    /// The reads not judged here.
    pub not_judged: Vec<NotJudged>,
    /// For a violation, the shrunk history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shrunk: Option<History>,
}

/// Checks `history`, and shrinks it where it is a violation.
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
pub fn report(ir: &EssIr, history: &History, budget: u64) -> Result<Report, CheckRefusal> {
    let checked = check(ir, history, budget)?;
    let shrunk = match checked.verdict {
        Verdict::Violation => Some(shrink(ir, history, budget)?),
        Verdict::Linearizable | Verdict::Unknown => None,
    };
    Ok(Report {
        verdict: checked.verdict,
        budget,
        steps: checked.steps,
        operations: history.operations.len(),
        partitions: checked.partitions,
        subject_key: checked.subject_key,
        linearization: checked.linearization,
        not_judged: checked.not_judged,
        shrunk,
    })
}

impl Report {
    /// The process exit status the verdict is: 0 linearizable, 1 violation, 3 unknown.
    pub fn exit_code(&self) -> u8 {
        match self.verdict {
            Verdict::Linearizable => 0,
            Verdict::Violation => 1,
            Verdict::Unknown => 3,
        }
    }

    /// The report as pretty JSON with one trailing newline; the field order is fixed.
    ///
    /// # Panics
    ///
    /// It does not: every field serializes.
    pub fn to_json(&self) -> String {
        let mut text = serde_json::to_string_pretty(self)
            .unwrap_or_else(|error| panic!("a report serializes: {error}"));
        text.push('\n');
        text
    }

    /// The report as text.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        let verdict = match self.verdict {
            Verdict::Linearizable => "Linearizable",
            Verdict::Violation => "Violation",
            Verdict::Unknown => "Unknown",
        };
        let _ = writeln!(text, "verdict: {verdict}");
        let _ = writeln!(
            text,
            "searched {} operation(s) in {} subject partition(s), {} of {} step(s)",
            self.operations - self.not_judged.len(),
            self.partitions,
            self.steps,
            self.budget
        );
        if self.verdict == Verdict::Unknown {
            let _ = writeln!(
                text,
                "the budget ran out before the search finished; Unknown is not a pass"
            );
        }
        if let Some(subject_key) = &self.subject_key {
            let _ = writeln!(
                text,
                "longest partial linearization, subject `{subject_key}`: {}",
                if self.linearization.is_empty() {
                    "none".to_owned()
                } else {
                    self.linearization.join(", ")
                }
            );
        }
        for read in &self.not_judged {
            let _ = writeln!(
                text,
                "not judged: {} reads `{}`, declared {}",
                read.operation_id, read.view, read.consistency
            );
        }
        if let Some(shrunk) = &self.shrunk {
            let _ = writeln!(
                text,
                "shrunk history: {} client(s), {} operation(s)",
                shrunk.clients,
                shrunk.operations.len()
            );
            for operation in &shrunk.operations {
                let returned = match operation.return_bound() {
                    ReturnBound::At(instant) => instant.to_string(),
                    ReturnBound::AfterEveryOther => "never".to_owned(),
                };
                let _ = writeln!(
                    text,
                    "  client {} [{}, {}] {} `{}` -> {}",
                    operation.client,
                    operation.invoked_at,
                    returned,
                    operation.command.as_str(),
                    operation.subject_key,
                    operation
                        .outcome
                        .as_ref()
                        .map_or("no answer", |outcome| outcome.as_str()),
                );
            }
        }
        text
    }
}
