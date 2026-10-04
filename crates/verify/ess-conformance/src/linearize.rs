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
//!   `external:` branch ([`Externals::Open`](crate::interpret::execute::Externals::Open)), and
//!   every input the history does not record (below).
//!   Each next state is a branch of the search.
//! * A state already reached with the same set of operations ordered is not searched again.
//! * Row-local histories are partitioned by subject, joining retry components as described below.
//!   A command which may read related rows or select a set puts the history into one shared
//!   partition: those operations do not commute merely because their addressed subjects differ.
//! * The private store preserves the provenance, type and presence of unrecorded generated values.
//!   A proven complete path wins over unresolved alternatives; exhaustion without such a path is
//!   a violation only if no unresolved alternative remains. Exhausted budget remains Unknown.
//!
//! # What a returned operation must answer
//!
//! A `Returned` operation's step must take the branch it recorded, by outcome name. An
//! `Indeterminate` operation's step may take any branch — or none, because a call that never
//! answered may never have happened.
//!
//! # A retried request
//!
//! A runner injecting the faults a specification declares sends a request to a command declaring
//! a `replays` branch a second time, and writes the second operation's
//! [`retry_of`](Operation::retry_of); an operation and its retries are one request. The search state
//! records, beside the model's store, which requests have taken the branch a `replays:` branch
//! retains, and holds every operation of a request to the declaration in every order it tries:
//!
//! * the origin branch is taken at most once per request — by an operation that answered it, or by
//!   one that never answered and is read as having taken it. A second is the request applied
//!   twice, and that order is not one the model allows;
//! * a `replays:` branch — answered, or taken by an operation that never answered — is available
//!   only once the request's origin branch has been taken earlier in that order, and changes
//!   nothing. A replay with nothing of its request taken before it is not one the model allows.
//! * an operation recording a `decision_time` (`ess-history/2`) made a decision, so a `replays:`
//!   branch never answers it: a retained answer delivered again carries no instant of its own.
//!
//! Because the rule spans the request, every subject its operations name — an original that never
//! answered names the instance it would have created, a retry that created another names that one
//! — is searched in one partition, named by the least of them, with every other operation on those
//! subjects; an operation of the request that names none, a replay of a generated identity, is
//! searched there too. Where two operations of one request both answered the origin branch, the
//! violation is reported before any search, naming the subject the second answer created and the
//! request's operations.
//!
//! # The instant a decision observed
//!
//! An `ess-history/2` operation may record the instant its command decision observed
//! ([`Operation::decision_time`]). Every alternative and replay of that operation the search tries
//! reads that one instant as `now`, unchanged. An operation recording none reads no clock: a guard
//! that needs one is Unknown, which leaves that alternative unresolved — never a violation, and
//! never a pass by trying instants nobody recorded. `invoked_at` and `returned_at` are ordering
//! coordinates and are never read as `now`.
//!
//! # What `ess-history/1` does not record, and how the search reads it
//!
//! An operation records its command, its subject and its outcome, and **not its input**. So the
//! search asks whether *some* input explains it: the candidate inputs [`witness::candidates`] builds
//! for the command from its declared input types and the literals its guards write — the same ones
//! synthesis submits — with the field that names the subject set to the recorded `subject_key`.
//! Every distinct next state any of them reaches is a branch. A creating command's new identity is
//! the `subject_key`, given to the interpreter as the value of the event field `instance:` names
//! ([`Generated::Recorded`]). Other implementation-assigned values stay abstract in this private
//! history entrypoint. A validated witness can prove their constrained domain inhabited; its value
//! is discarded and cannot decide a later guard. Public native generated-value policies retain
//! their concrete semantics.
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
//! A read of a view is not an operation of the search: it is judged after it, at the level its
//! view declares ([`Consistency`] is `read_your_writes` or `eventual`; ESS declares no linearizable
//! read). The match over [`Consistency`] is exhaustive, so a level added later has to be decided
//! here.
//!
//! What a read answered is [`Operation::rows`], the identity of each row. What the model says a
//! view holds in a state is, for each subject, whether its instance exists and its lifecycle state
//! passes the view's filter. So one read is judged subject by subject — for each subject, whether
//! it is a row — which is weaker than asking for one snapshot of every subject at once, and a
//! violation of it is a violation of either.
//!
//! A read is judged against **every** state some complete linearization of the subject's
//! operations passes through, not against the one order the search found: where two calls overlap,
//! either order may be the one that happened. Each such state is a set of operations ordered so
//! far and the model state they leave; an `Indeterminate` operation contributes both of its
//! branches, took effect and never happened. For one read, of one subject, a state explains it
//! when it shows the subject as the read does and:
//!
//! * every operation it has ordered was invoked no later than the read returned — a read shows
//!   nothing no write had yet been asked for;
//! * `read_your_writes`, per client session: it has ordered every operation the reading client
//!   made on the subject that returned before the read was invoked. A read no such state explains,
//!   and an earlier one does, is [`Anomaly::StaleRead`] (Jepsen Elle's session vocabulary);
//! * `eventual`: nothing more, until the writes have settled. The writes **stop** at the latest
//!   return of any command, where every command answered; a history with a command that never
//!   answered has no such point, because that command may take effect at any time. A session is
//!   one client's reads of one view. Its reads invoked after the writes stop are counted in invoke
//!   order, and every one after the first `settle` ([`check_settled`]; [`DEFAULT_SETTLE`] for
//!   [`check`]) must show a state in which every operation is ordered — what the linearized
//!   commands produce — or it is [`Anomaly::NotConverged`]. `settle` counts reads, not instants:
//!   instants compare only by order, so an order-preserving change of clock changes no verdict. A
//!   read before that may be behind, because an eventual projection promises convergence, not
//!   convergence by the next read; it is judged as the level allows and listed in
//!   [`Checked::not_judged`] with the reason [`BEFORE_SETTLE`] or [`BEFORE_WRITES_STOP`], so a
//!   history in which no read was judged for convergence says so.
//!
//! A read no allowed state explains, where no earlier one does either, or that shows a row no
//! recorded operation addresses, is [`Anomaly::FutureRead`]. A read that lists one identity twice
//! is [`Anomaly::DuplicateRow`]: a view that is not an aggregate (aggregates are not judged) has one
//! row per instance, and the language declares no view with repeated rows. Reads are judged only when every
//! partition is linearizable: a command violation is reported first, and an `Unknown` — from the
//! search, or from the budget running out while every state is collected — leaves them unjudged.
//!
//! A read the checker cannot judge is listed in [`Checked::not_judged`] with its reason: one that
//! never answered, one that records no rows, a read of a view that takes parameters (a history
//! records none), of an aggregate view (its rows are groups, not instances), or of a view whose
//! filter the lifecycle state alone does not decide (a history records no field values).
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

use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedEffect, ResolvedInstance, ResolvedView};
use ess_compiler::ir::{ResolvedBody, ResolvedTypeRef};
use ess_domain::entity::StateName;
use ess_domain::name::QualifiedName as ModelName;
use ess_domain::types::Primitive;
use ess_domain::view::Consistency;
use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::Truth;
use serde::Serialize;

use crate::history::{Completion, History, HistoryFormat, Operation, ReturnBound, Verdict};
use crate::input::TypedFacts;
use crate::interpret::execute::history::{self as history_execution, State};
use crate::interpret::execute::{Generated, GeneratedSlot, Undetermined};
use crate::witness::{self, Distinction};

/// The search budget when none is named: this many executions of the model.
pub const DEFAULT_BUDGET: u64 = 1_000_000;

/// A read this check does not judge, the level its view declares, and why it is not judged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotJudged {
    /// The operation.
    pub operation_id: String,
    /// The view it reads.
    pub view: String,
    /// The consistency the view declares, as written in a document.
    pub consistency: String,
    /// Why the read cannot be judged.
    pub reason: String,
}

/// What a judged read shows that its view's declared consistency does not allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Anomaly {
    /// `read_your_writes`: the read shows a subject as it was before the reading client's own last
    /// write on it.
    StaleRead,
    /// `eventual`: a read after a session's first `settle` reads after the writes stopped does not
    /// show what the linearized commands produced.
    NotConverged,
    /// The read shows a subject as no write invoked before the read returned left it, or shows a
    /// row no recorded operation addresses.
    FutureRead,
    /// The read lists one instance twice. A view that is not an aggregate holds one row per
    /// instance of the entity it projects, so no state of the model answers it.
    DuplicateRow,
}

impl Anomaly {
    /// As written in a report.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StaleRead => "stale-read",
            Self::NotConverged => "not-converged",
            Self::FutureRead => "future-read",
            Self::DuplicateRow => "duplicate-row",
        }
    }
}

/// A read its view's declared consistency does not allow: which client, which read, and what it
/// showed of which subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadViolation {
    /// The read.
    pub operation_id: String,
    /// The client that made it.
    pub client: u64,
    /// The view it reads.
    pub view: String,
    /// The consistency the view declares, as written in a document.
    pub consistency: String,
    /// The subject the read shows wrongly.
    pub subject_key: String,
    /// Whether the read shows that subject as a row.
    pub shown: bool,
    /// What is wrong with it.
    pub anomaly: Anomaly,
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
    /// model accepted, by operation id. For a read violation, one order the search found for the
    /// subject the read shows wrongly; the read was judged against every order. Empty for
    /// `Linearizable`.
    pub linearization: Vec<String>,
    /// How many subject partitions were searched.
    pub partitions: usize,
    /// The reads that cannot be judged, in document order.
    pub not_judged: Vec<NotJudged>,
    /// How many reads were judged at their view's consistency.
    pub judged: usize,
    /// For a `Violation` a read decided: that read.
    pub read: Option<ReadViolation>,
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
    /// For an operation of a command declaring `replays:`, the request it sends ([`RequestPlan`]).
    request: Option<Request>,
}

/// One read of a view the checker judges, with everything judging it needs.
struct ViewRead<'h> {
    operation: &'h Operation,
    view: String,
    consistency: Consistency,
    /// The entity the view projects.
    entity: &'h ModelName,
    /// The lifecycle states whose instances the view's filter admits.
    admits: BTreeSet<&'h StateName>,
    /// What it answered, as a set.
    rows: BTreeSet<&'h str>,
    /// The first identity it answered more than once, where it did.
    duplicate: Option<&'h str>,
}

/// The model-side reading of every operation: commands by subject, judged reads, and the reads
/// that cannot be judged.
struct Split<'h> {
    partitions: BTreeMap<Partition<'h>, Vec<Prepared<'h>>>,
    reads: Vec<ViewRead<'h>>,
    not_judged: Vec<NotJudged>,
}

/// A shared interaction group is not a fabricated subject identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Partition<'h> {
    Subject(&'h str),
    Shared,
}

impl Partition<'_> {
    fn diagnostic(self, operations: &[Prepared<'_>]) -> String {
        match self {
            Self::Subject(subject) => subject.to_owned(),
            Self::Shared => operations
                .iter()
                .map(|prepared| prepared.operation.subject_key.as_str())
                .filter(|subject| !subject.is_empty())
                .min()
                .unwrap_or("")
                .to_owned(),
        }
    }
}

/// A conservative grouping dependency, never a command refusal or branch-selection authority.
fn reads_other_rows(command: &ResolvedCommand) -> bool {
    use ess_compiler::ir::{ResolvedCondition, ResolvedPayloadValue};
    fn cross(value: &ResolvedPayloadValue) -> bool {
        match value {
            ResolvedPayloadValue::RelatedField { .. } => true,
            ResolvedPayloadValue::Struct { fields } => {
                fields.iter().any(|field| cross(&field.value))
            }
            ResolvedPayloadValue::ResponseField { .. }
            | ResolvedPayloadValue::Generated
            | ResolvedPayloadValue::InputField { .. }
            | ResolvedPayloadValue::Literal { .. }
            | ResolvedPayloadValue::Cleared
            | ResolvedPayloadValue::SubjectField { .. }
            | ResolvedPayloadValue::Increment { .. }
            | ResolvedPayloadValue::InputOrGenerated { .. }
            | ResolvedPayloadValue::CallerAttribute { .. }
            | ResolvedPayloadValue::ChangedCount => false,
        }
    }
    command.outcomes.iter().any(|outcome| {
        matches!(outcome.condition, ResolvedCondition::Related { .. })
            || outcome.instances.is_some()
            || !outcome.affects.is_empty()
            || outcome.sets.iter().any(|field| cross(&field.value))
            || outcome
                .error_payload
                .iter()
                .any(|field| cross(&field.value))
            || outcome
                .payload
                .iter()
                .any(|payload| payload.fields.iter().any(|field| cross(&field.value)))
    })
}

/// The read `operation` of `view`, where it can be judged; why not, where it cannot.
fn view_read<'h>(
    ir: &'h EssIr,
    view: &'h ResolvedView,
    operation: &'h Operation,
) -> Result<ViewRead<'h>, String> {
    if operation.completion != Completion::Returned {
        return Err("the read never answered".to_owned());
    }
    let Some(rows) = &operation.rows else {
        return Err("the read records no rows".to_owned());
    };
    if !view.params.is_empty() {
        return Err("the view takes parameters, and a history records none".to_owned());
    }
    if view.is_aggregate() {
        return Err("an aggregate view's rows are groups, not instances".to_owned());
    }
    let entity = ir.entity(&view.source);
    if !history_text_identity(ir, &entity.identity.type_ref) {
        return Err(
            "ess-history/1 records text row identities; this entity requires nontext identities"
                .into(),
        );
    }
    let mut admits = BTreeSet::new();
    for state in &entity.lifecycle.states {
        let admitted = match &view.filter {
            None => true,
            Some(filter) => {
                let mut facts = TypedFacts::new(ir, &view.fields, FactStore::new());
                facts.set(
                    FactPath::new(STATE)
                        .unwrap_or_else(|error| panic!("`{STATE}` is a fact path: {error}")),
                    FactValue::text(state.as_str()),
                );
                match filter.evaluate(&facts) {
                    Truth::True => true,
                    Truth::False => false,
                    Truth::Unknown => {
                        return Err(
                            "its filter is not decided by the lifecycle state alone, and a \
                             history records no field values"
                                .to_owned(),
                        )
                    }
                }
            }
        };
        if admitted {
            admits.insert(state);
        }
    }
    Ok(ViewRead {
        operation,
        view: view.name.to_string(),
        consistency: view.consistency,
        entity: &entity.name,
        admits,
        rows: rows.iter().map(String::as_str).collect(),
        duplicate: {
            let mut seen = BTreeSet::new();
            rows.iter()
                .map(String::as_str)
                .find(|row| !seen.insert(*row))
        },
    })
}

/// The fact a view filter reads an instance's lifecycle state at.
const STATE: &str = "state";

/// `true` when `operation` names a view `ir` declares.
fn reads_a_view(ir: &EssIr, operation: &Operation) -> bool {
    ModelName::new(operation.command.as_str()).is_ok_and(|name| ir.views().contains_key(&name))
}

fn split<'h>(ir: &'h EssIr, history: &'h History) -> Result<Split<'h>, CheckRefusal> {
    let mut inputs: BTreeMap<&ModelName, Vec<BTreeMap<String, Node>>> = BTreeMap::new();
    let mut partitions: BTreeMap<Partition<'h>, Vec<Prepared<'h>>> = BTreeMap::new();
    let shared = history.operations.iter().any(|operation| {
        ModelName::new(operation.command.as_str())
            .ok()
            .and_then(|name| ir.commands().get(&name))
            .is_some_and(reads_other_rows)
    });
    let mut reads = Vec::new();
    let mut not_judged = Vec::new();
    let mut plan = request_plan(ir, history);
    for operation in &history.operations {
        let unknown = || CheckRefusal::UnknownOperation {
            operation_id: operation.operation_id.as_str().to_owned(),
            command: operation.command.as_str().to_owned(),
        };
        let name = ModelName::new(operation.command.as_str()).map_err(|_| unknown())?;
        if let Some(view) = ir.views().get(&name) {
            match view_read(ir, view, operation) {
                Ok(read) => reads.push(read),
                Err(reason) => not_judged.push(NotJudged {
                    operation_id: operation.operation_id.as_str().to_owned(),
                    view: name.to_string(),
                    consistency: view.consistency.as_str().to_owned(),
                    reason,
                }),
            }
            continue;
        }
        let (key, command) = ir.commands().get_key_value(&name).ok_or_else(unknown)?;
        if command
            .outcomes
            .iter()
            .filter_map(|outcome| outcome.subject.as_ref())
            .any(|subject| {
                !history_text_identity(ir, &ir.entity(&subject.entity).identity.type_ref)
            })
        {
            return Err(CheckRefusal::Model {
                operation_id: operation.operation_id.as_str().into(),
                why: "ess-history/1 records text subject identities; this command requires nontext identities".into(),
            });
        }
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
        let id = operation.operation_id.as_str();
        partitions
            .entry(if shared {
                Partition::Shared
            } else {
                Partition::Subject(
                    plan.partition
                        .get(id)
                        .copied()
                        .unwrap_or(operation.subject_key.as_str()),
                )
            })
            .or_default()
            .push(Prepared {
                operation,
                name: key.clone(),
                inputs,
                generated,
                request: plan.requests.remove(id),
            });
    }
    Ok(Split {
        partitions,
        reads,
        not_judged,
    })
}

/// Whether this declaration admits history's actual text identity representation.
/// Json histories retain text as text; numeric JSON cannot be recovered from a string.
fn history_text_identity(ir: &EssIr, identity: &ResolvedTypeRef) -> bool {
    let mut current = identity.required();
    for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
        return match current {
            ResolvedTypeRef::Primitive { name } => matches!(
                name,
                Primitive::String
                    | Primitive::Uuid
                    | Primitive::Timestamp
                    | Primitive::Duration
                    | Primitive::Json
            ),
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => {
                    current = of.required();
                    continue;
                }
                ResolvedBody::Enum { .. } => true,
                _ => false,
            },
            _ => false,
        };
    }
    false
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

/// The state a search holds: the model's store, and the requests whose origin branch the order so
/// far has taken ([`Request::id`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Held {
    store: State,
    taken: BTreeSet<usize>,
}

/// Every state the model can be in after `prepared` from `held`, answering what it recorded.
///
/// An operation of a request to a command declaring `replays:` is held to the request besides:
/// its origin branch is taken at most once per request, and a `replays:` branch only once it has
/// been, whether the operation answered it or, never answering, may have taken it.
struct Alternatives<T> {
    proven: Vec<T>,
    unresolved: Option<CheckRefusal>,
    exhausted: bool,
}

impl<T> Alternatives<T> {
    /// Diagnostics may describe impossibility only when every alternative is resolved.
    fn determined(self) -> Result<Vec<T>, CheckRefusal> {
        self.unresolved.map_or(Ok(self.proven), Err)
    }
}

fn step(
    ir: &EssIr,
    held: &Held,
    prepared: &Prepared<'_>,
) -> Result<Alternatives<Held>, CheckRefusal> {
    let operation = prepared.operation;
    let store = &held.store;
    let mut next: Vec<Held> = Vec::new();
    let mut unresolved = None;
    if operation.completion == Completion::Indeterminate {
        // It may never have happened.
        next.push(held.clone());
    }
    for input in &prepared.inputs {
        let steps = match history_execution::execute(
            ir,
            store,
            &prepared.name,
            input,
            &prepared.generated,
            operation.operation_id.as_str(),
            operation.decision_time,
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
        if let Some(why) = steps.unresolved {
            unresolved.get_or_insert_with(|| CheckRefusal::Model {
                operation_id: operation.operation_id.as_str().to_owned(),
                why: why.to_string(),
            });
        }
        for taken in steps.proven {
            let answers = match (&operation.completion, &operation.outcome) {
                (Completion::Returned, Some(recorded)) => taken
                    .outcome
                    .as_ref()
                    .is_some_and(|outcome| outcome.outcome.as_str() == recorded.as_str()),
                _ => true,
            };
            if !answers {
                continue;
            }
            let mut taken_now = held.taken.clone();
            if let Some(request) = &prepared.request {
                let branch = taken
                    .outcome
                    .as_ref()
                    .map_or("", |outcome| outcome.outcome.as_str());
                if request.origins.contains(branch) && !taken_now.insert(request.id) {
                    // The request's origin branch was taken already: applied twice.
                    continue;
                }
                if request.replays.contains(branch) && !held.taken.contains(&request.id) {
                    // Nothing of this request was retained to replay.
                    continue;
                }
                if request.replays.contains(branch) && operation.decision_time.is_some() {
                    // A retained answer delivered again decides nothing; an operation recording a
                    // decision instant made a decision, and a replay is not one (`ess-history/2`).
                    continue;
                }
            }
            let state = Held {
                store: taken.next,
                taken: taken_now,
            };
            if !next.contains(&state) {
                next.push(state);
            }
        }
    }
    Ok(Alternatives {
        proven: next,
        unresolved,
        exhausted: false,
    })
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
    /// The order found, by operation index.
    Linearizable(Vec<usize>),
    Violation(Vec<usize>),
    Unknown(Vec<usize>),
    Unresolved(CheckRefusal),
}

/// One level of the depth-first search: the state reached, and the moves from it not yet tried.
struct Frame {
    done: Done,
    moves: Vec<(usize, Held)>,
    next: usize,
}

/// The operations of one partition in the fixed order moves are tried in: invoke order, then
/// document order.
fn move_order(operations: &[Prepared<'_>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..operations.len()).collect();
    order.sort_by_key(|&index| (operations[index].operation.invoked_at, index));
    order
}

/// The moves from one state: every operation nothing still outside the order returned before,
/// with every state it can leave. `None` when the budget ran out.
fn next_moves(
    ir: &EssIr,
    operations: &[Prepared<'_>],
    order: &[usize],
    done: &Done,
    store: &Held,
    budget: &mut u64,
    spent: &mut u64,
) -> Result<Option<Alternatives<(usize, Held)>>, CheckRefusal> {
    let mut found: Vec<(usize, Held)> = Vec::new();
    let mut unresolved = None;
    for &index in order {
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
            return Ok((!found.is_empty()).then_some(Alternatives {
                proven: found,
                unresolved,
                exhausted: true,
            }));
        }
        *budget -= 1;
        *spent += 1;
        let steps = step(ir, store, &operations[index])?;
        if unresolved.is_none() {
            unresolved = steps.unresolved;
        }
        for next in steps.proven {
            found.push((index, next));
        }
    }
    Ok(Some(Alternatives {
        proven: found,
        unresolved,
        exhausted: false,
    }))
}

/// Searches one partition, spending from `budget`.
fn search(
    ir: &EssIr,
    operations: &[Prepared<'_>],
    budget: &mut u64,
    spent: &mut u64,
) -> Result<Outcome, CheckRefusal> {
    let order = move_order(operations);
    let total = operations.len();

    let mut seen: BTreeMap<Done, Vec<Held>> = BTreeMap::new();
    let mut path: Vec<usize> = Vec::new();
    let mut longest: Vec<usize> = Vec::new();

    let start = Done::new(total);
    let Some(first) = next_moves(
        ir,
        operations,
        &order,
        &start,
        &Held::default(),
        budget,
        spent,
    )?
    else {
        return Ok(Outcome::Unknown(longest));
    };
    if total == 0 {
        return Ok(Outcome::Linearizable(Vec::new()));
    }
    let mut unresolved = first.unresolved;
    let mut exhausted = first.exhausted;
    let mut stack = vec![Frame {
        done: start,
        moves: first.proven,
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
            return Ok(Outcome::Linearizable(path));
        }
        let Some(found) = next_moves(ir, operations, &order, &done, &store, budget, spent)? else {
            exhausted = true;
            path.pop();
            continue;
        };
        exhausted |= found.exhausted;
        if unresolved.is_none() {
            unresolved = found.unresolved;
        }
        stack.push(Frame {
            done,
            moves: found.proven,
            next: 0,
        });
    }
    if exhausted {
        Ok(Outcome::Unknown(longest))
    } else {
        Ok(unresolved.map_or(Outcome::Violation(longest), Outcome::Unresolved))
    }
}

// ---- every state a linearization passes through -----------------------------------------------

/// One state some complete linearization of a partition passes through: the operations ordered
/// so far, and the state they leave.
struct Reached {
    done: Done,
    store: State,
    /// The latest invoke instant among the operations ordered so far; 0 when there are none.
    latest: u64,
    /// Whether every operation of the partition is ordered.
    full: bool,
}

/// The walk behind [`reach`]: each node's liveness, cached by the set ordered and the state.
struct Walk<'a, 'h> {
    ir: &'a EssIr,
    operations: &'a [Prepared<'h>],
    order: Vec<usize>,
    budget: &'a mut u64,
    spent: &'a mut u64,
    live: BTreeMap<Done, Vec<(Held, bool)>>,
    unresolved: Option<CheckRefusal>,
}

impl Walk<'_, '_> {
    /// Whether some complete linearization passes through (`done`, `store`); `None` when the
    /// budget ran out.
    fn visit(&mut self, done: &Done, store: &Held) -> Result<Option<bool>, CheckRefusal> {
        if let Some(known) = self
            .live
            .get(done)
            .and_then(|states| states.iter().find(|(held, _)| held == store))
        {
            return Ok(Some(known.1));
        }
        let complete = (0..self.operations.len()).all(|index| done.has(index));
        let mut live = complete;
        if !complete {
            let Some(moves) = next_moves(
                self.ir,
                self.operations,
                &self.order,
                done,
                store,
                self.budget,
                self.spent,
            )?
            else {
                return Ok(None);
            };
            if self.unresolved.is_none() {
                self.unresolved = moves.unresolved;
            }
            if moves.exhausted {
                return Ok(None);
            }
            for (index, next) in moves.proven {
                match self.visit(&done.with(index), &next)? {
                    None => return Ok(None),
                    Some(child) => live |= child,
                }
            }
        }
        self.live
            .entry(done.clone())
            .or_default()
            .push((store.clone(), live));
        Ok(Some(live))
    }
}

/// Every state some complete linearization of the partition passes through, including the empty
/// start; `None` when the budget ran out.
///
/// Every order the search would accept, not the one it found: a read is explained by any of them.
/// An `Indeterminate` operation contributes both of its branches — it took effect, or it never
/// happened — because [`step`] returns both.
fn reach(
    ir: &EssIr,
    operations: &[Prepared<'_>],
    budget: &mut u64,
    spent: &mut u64,
) -> Result<Option<Alternatives<Reached>>, CheckRefusal> {
    let total = operations.len();
    let mut walk = Walk {
        ir,
        operations,
        order: move_order(operations),
        budget,
        spent,
        live: BTreeMap::new(),
        unresolved: None,
    };
    if walk.visit(&Done::new(total), &Held::default())?.is_none() {
        return Ok(None);
    }
    let mut reached = Vec::new();
    for (done, states) in walk.live {
        let ordered: Vec<usize> = (0..total).filter(|&index| done.has(index)).collect();
        let latest = ordered
            .iter()
            .map(|&index| operations[index].operation.invoked_at)
            .max()
            .unwrap_or(0);
        let full = ordered.len() == total;
        for (held, live) in states {
            if live {
                reached.push(Reached {
                    done: done.clone(),
                    store: held.store,
                    latest,
                    full,
                });
            }
        }
    }
    Ok(Some(Alternatives {
        proven: reached,
        unresolved: walk.unresolved,
        exhausted: false,
    }))
}

/// How many of a session's reads of an `eventual` view, invoked after the writes stop, may still
/// be behind, when none is named. Convergence is judged on the reads after them.
///
/// A count of reads, not a span of instants: `ess-history/1` instants compare only by order, so
/// an order-preserving change of clock changes no verdict. Two is what the billing reference at
/// `Billing::DEFAULT_LAG` needs — its projection catches up after two further reads by anyone,
/// and a session's third read after the writes is invoked after its first two answered — and the
/// default doubles it.
pub const DEFAULT_SETTLE: u64 = 4;

/// Searches `history` for an order of its operations the model of `ir` accepts, spending at most
/// `budget` executions of the model, and judges its view reads with [`DEFAULT_SETTLE`].
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
pub fn check(ir: &EssIr, history: &History, budget: u64) -> Result<Checked, CheckRefusal> {
    check_settled(ir, history, budget, DEFAULT_SETTLE)
}

/// [`check`], with convergence of an `eventual` view judged only on a session's reads after its
/// first `settle` reads of that view invoked after the writes stop.
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
#[allow(
    clippy::too_many_lines,
    reason = "the checker keeps reachability and view judgments under one shared budget"
)]
pub fn check_settled(
    ir: &EssIr,
    history: &History,
    budget: u64,
    settle: u64,
) -> Result<Checked, CheckRefusal> {
    let split = split(ir, history)?;
    if let Some(checked) = applied_twice(ir, history, &split) {
        return Ok(checked);
    }
    let mut remaining = budget;
    let mut spent = 0;
    let mut unknown: Option<(String, Vec<String>)> = None;
    let ids = |operations: &[Prepared<'_>], indices: &[usize]| -> Vec<String> {
        indices
            .iter()
            .map(|&index| operations[index].operation.operation_id.as_str().to_owned())
            .collect()
    };
    let mut found: BTreeMap<Partition<'_>, Vec<String>> = BTreeMap::new();
    let mut unresolved = None;
    for (subject_key, operations) in &split.partitions {
        match search(ir, operations, &mut remaining, &mut spent)? {
            Outcome::Linearizable(order) => {
                found.insert(*subject_key, ids(operations, &order));
            }
            Outcome::Violation(longest) => {
                return Ok(Checked {
                    verdict: Verdict::Violation,
                    steps: spent,
                    subject_key: Some(subject_key.diagnostic(operations)),
                    linearization: ids(operations, &longest),
                    partitions: split.partitions.len(),
                    not_judged: split.not_judged,
                    judged: 0,
                    read: None,
                });
            }
            Outcome::Unknown(longest) => {
                if unknown.is_none() {
                    unknown = Some((
                        subject_key.diagnostic(operations),
                        ids(operations, &longest),
                    ));
                }
            }
            Outcome::Unresolved(why) => {
                unresolved.get_or_insert(why);
            }
        }
    }
    let unknown_now =
        |subject_key: String, linearization: Vec<String>, spent: u64, split: Split| Checked {
            verdict: Verdict::Unknown,
            steps: spent,
            subject_key: Some(subject_key),
            linearization,
            partitions: split.partitions.len(),
            not_judged: split.not_judged,
            judged: 0,
            read: None,
        };
    if let Some((subject_key, longest)) = unknown {
        return Ok(unknown_now(subject_key, longest, spent, split));
    }
    if let Some(why) = unresolved {
        return Err(why);
    }
    let mut reached: BTreeMap<Partition<'_>, Vec<Reached>> = BTreeMap::new();
    let mut unresolved_reads = BTreeMap::new();
    if !split.reads.is_empty() {
        for (subject_key, operations) in &split.partitions {
            let Some(states) = reach(ir, operations, &mut remaining, &mut spent)? else {
                let order = found.remove(subject_key).unwrap_or_default();
                return Ok(unknown_now(
                    subject_key.diagnostic(operations),
                    order,
                    spent,
                    split,
                ));
            };
            reached.insert(*subject_key, states.proven);
            if let Some(why) = states.unresolved {
                unresolved_reads.insert(*subject_key, why);
            }
        }
    }
    let convergence = convergence(&split, settle);
    let subjects = subjects(&split);
    let judged_reads = judge(&split, &subjects, &reached, &convergence, &unresolved_reads)?;
    let (verdict, subject_key, linearization, read) = match judged_reads {
        Some(violation) => (
            Verdict::Violation,
            Some(violation.subject_key.clone()),
            subjects
                .get(violation.subject_key.as_str())
                .and_then(|partition| found.remove(partition))
                .unwrap_or_default(),
            Some(violation),
        ),
        None => (Verdict::Linearizable, None, Vec::new(), None),
    };
    let judged = split.reads.len()
        - convergence
            .values()
            .filter(|judged| judged.is_err())
            .count();
    let not_judged = listed(history, split.not_judged, &split.reads, &convergence);
    Ok(Checked {
        verdict,
        steps: spent,
        subject_key,
        linearization,
        partitions: split.partitions.len(),
        not_judged,
        judged,
        read,
    })
}

/// Every request in `history`, in document order: the operation that first sent it, by lower-case
/// id, and every operation that sent it — the first and each retry ([`Operation::retry_of`]). A
/// retry naming an operation the history does not hold is its own request.
fn requests(history: &History) -> Vec<(String, Vec<&Operation>)> {
    let by_id: BTreeMap<String, &Operation> = history
        .operations
        .iter()
        .map(|operation| {
            (
                operation.operation_id.as_str().to_ascii_lowercase(),
                operation,
            )
        })
        .collect();
    let root = |operation: &Operation| -> String {
        let mut current = operation.operation_id.as_str().to_ascii_lowercase();
        let mut steps = 0;
        while let Some(original) = by_id
            .get(&current)
            .and_then(|operation| operation.retry_of.as_ref())
            .map(|original| original.as_str().to_ascii_lowercase())
            .filter(|original| by_id.contains_key(original))
        {
            current = original;
            steps += 1;
            if steps > history.operations.len() {
                break;
            }
        }
        current
    };
    let mut requests: Vec<(String, Vec<&Operation>)> = Vec::new();
    for operation in &history.operations {
        let request = root(operation);
        match requests.iter_mut().find(|(known, _)| *known == request) {
            Some((_, members)) => members.push(operation),
            None => requests.push((request, vec![operation])),
        }
    }
    requests
}

/// The branches of `command` a `replays:` branch retains, and the `replays:` branches themselves.
fn retained_branches(command: &ResolvedCommand) -> (BTreeSet<&str>, BTreeSet<&str>) {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| {
            outcome
                .replays
                .as_ref()
                .map(|replay| (replay.origin.as_str(), outcome.name.as_str()))
        })
        .unzip()
}

/// Whether `operation` answered one of `branches`.
fn answered_one_of(operation: &Operation, branches: &BTreeSet<&str>) -> bool {
    operation.completion == Completion::Returned
        && operation
            .outcome
            .as_ref()
            .is_some_and(|outcome| branches.contains(outcome.as_str()))
}

/// One request to a command declaring `replays:`, as each of its operations is stepped with it.
#[derive(Debug, Clone)]
struct Request {
    /// The request's place among the history's requests: what [`Held::taken`] records.
    id: usize,
    /// The branches a `replays:` branch retains.
    origins: BTreeSet<String>,
    /// The `replays:` branches.
    replays: BTreeSet<String>,
}

/// Where each operation of a request to a command declaring `replays:` is searched, and the
/// request it belongs to, by operation id.
///
/// One request's operations may name different subjects — an original that never answered
/// names the instance it would have created, and a retry that created another names that one — and
/// the search holds them to one rule across all of them (at most one origin answer, a replay only
/// after it). So every subject one request's operations name is searched in one partition, with
/// every other operation on those subjects, and an operation of the request that names no subject
/// (a replay of a generated identity) is searched there too. The partition is named by the least of
/// those subjects.
struct RequestPlan<'h> {
    requests: BTreeMap<&'h str, Request>,
    partition: BTreeMap<&'h str, &'h str>,
}

fn request_plan<'h>(ir: &EssIr, history: &'h History) -> RequestPlan<'h> {
    let mut plan = RequestPlan {
        requests: BTreeMap::new(),
        partition: BTreeMap::new(),
    };
    let mut classes: Vec<BTreeSet<&'h str>> = Vec::new();
    let mut members_of: Vec<(Vec<&'h Operation>, BTreeSet<&'h str>)> = Vec::new();
    for (id, (_, members)) in requests(history).into_iter().enumerate() {
        let Some(command) = ModelName::new(members[0].command.as_str())
            .ok()
            .and_then(|name| ir.commands().get(&name))
        else {
            continue;
        };
        let (origins, replays) = retained_branches(command);
        if origins.is_empty() {
            continue;
        }
        let request = Request {
            id,
            origins: origins.into_iter().map(ToOwned::to_owned).collect(),
            replays: replays.into_iter().map(ToOwned::to_owned).collect(),
        };
        for operation in &members {
            plan.requests
                .insert(operation.operation_id.as_str(), request.clone());
        }
        let subjects: BTreeSet<&'h str> = members
            .iter()
            .map(|operation| operation.subject_key.as_str())
            .filter(|subject| !subject.is_empty())
            .collect();
        let mut merged = subjects.clone();
        classes.retain(|class| {
            if class.is_disjoint(&subjects) {
                true
            } else {
                merged.extend(class.iter().copied());
                false
            }
        });
        if !merged.is_empty() {
            classes.push(merged);
        }
        members_of.push((members, subjects));
    }
    let name_of = |subject: &str| {
        classes
            .iter()
            .find(|class| class.contains(subject))
            .and_then(|class| class.first().copied())
    };
    for operation in &history.operations {
        if let Some(name) = name_of(operation.subject_key.as_str()) {
            plan.partition.insert(operation.operation_id.as_str(), name);
        }
    }
    for (members, subjects) in &members_of {
        let Some(name) = subjects.first().and_then(|subject| name_of(subject)) else {
            continue;
        };
        for operation in members {
            plan.partition.insert(operation.operation_id.as_str(), name);
        }
    }
    plan
}

/// The first request a client sent more than once whose origin branch answered more than once, as
/// a violation naming the subject the second such answer created and every operation of the
/// request, by id.
///
/// A retry ([`Operation::retry_of`]) sends one logical request again. Its command declares a
/// `replays` branch, which is what the specification says a request already answered by the
/// branch it replays answers: the retained result, with no second effect. So of one request's
/// operations, at most one may answer that origin branch; each other may answer the replay, or
/// never answer. Two origin answers are one request applied twice, whatever the search would make
/// of them as two independent calls. A retry naming an operation the history does not hold is its
/// own request.
fn applied_twice(ir: &EssIr, history: &History, split: &Split<'_>) -> Option<Checked> {
    for (_, members) in requests(history)
        .iter()
        .filter(|(_, members)| members.len() > 1)
    {
        let Some(command) = ModelName::new(members[0].command.as_str())
            .ok()
            .and_then(|name| ir.commands().get(&name))
        else {
            continue;
        };
        let (origins, _) = retained_branches(command);
        let applied: Vec<&&Operation> = members
            .iter()
            .filter(|operation| answered_one_of(operation, &origins))
            .collect();
        if let [_, second, ..] = applied.as_slice() {
            return Some(Checked {
                verdict: Verdict::Violation,
                steps: 0,
                subject_key: Some(second.subject_key.clone()),
                linearization: members
                    .iter()
                    .map(|operation| operation.operation_id.as_str().to_owned())
                    .collect(),
                partitions: split.partitions.len(),
                not_judged: split.not_judged.clone(),
                judged: 0,
                read: None,
            });
        }
    }
    None
}

/// `not_judged` with every read judged for less than its level promises added — each `eventual`
/// read `convergence` did not judge for convergence — in document order.
fn listed(
    history: &History,
    mut not_judged: Vec<NotJudged>,
    reads: &[ViewRead<'_>],
    convergence: &BTreeMap<&str, Result<(), String>>,
) -> Vec<NotJudged> {
    for read in reads {
        if let Some(Err(reason)) = convergence.get(read.operation.operation_id.as_str()) {
            not_judged.push(NotJudged {
                operation_id: read.operation.operation_id.as_str().to_owned(),
                view: read.view.clone(),
                consistency: read.consistency.as_str().to_owned(),
                reason: reason.clone(),
            });
        }
    }
    let position: BTreeMap<&str, usize> = history
        .operations
        .iter()
        .enumerate()
        .map(|(index, operation)| (operation.operation_id.as_str(), index))
        .collect();
    not_judged.sort_by_key(|read| position.get(read.operation_id.as_str()).copied());
    not_judged
}

// ---- judging the reads -----------------------------------------------------------------------

/// `true` when `operation` returned before `instant`.
fn returned_before(operation: &Operation, instant: u64) -> bool {
    operation.return_bound() < ReturnBound::At(instant)
}

/// The reason code of an `eventual` read not judged for convergence because the session had not
/// yet read `settle` times after the writes stopped.
pub const BEFORE_SETTLE: &str = "before-settle";

/// The reason code of an `eventual` read not judged for convergence because it was invoked before
/// the writes stopped, or the writes never stopped.
pub const BEFORE_WRITES_STOP: &str = "before-writes-stop";

/// For every `eventual` read, by operation id: `Ok` where it is judged for convergence, and the
/// reason where it is not. A `read_your_writes` read has no entry.
///
/// The writes stop at the latest return of any command, where every command answered. A read is
/// after it when it was invoked after that return — an order, never a difference of instants. A
/// session is one client's reads of one view; its reads after the writes stop are counted in
/// invoke order, and the first `settle` of them may still be behind.
fn convergence<'h>(split: &Split<'h>, settle: u64) -> BTreeMap<&'h str, Result<(), String>> {
    let stopped = split.partitions.values().flatten().try_fold(
        None,
        |latest: Option<&Operation>, prepared| {
            let operation = prepared.operation;
            (operation.completion == Completion::Returned)
                .then(|| match latest {
                    Some(held) if held.return_bound() >= operation.return_bound() => held,
                    _ => operation,
                })
                .map(Some)
        },
    );
    let mut sessions: BTreeMap<(u64, &str), Vec<&Operation>> = BTreeMap::new();
    let mut decided = BTreeMap::new();
    for read in &split.reads {
        if read.consistency != Consistency::Eventual {
            continue;
        }
        let operation = read.operation;
        let id = operation.operation_id.as_str();
        match stopped {
            None => {
                decided.insert(
                    id,
                    Err(format!(
                        "{BEFORE_WRITES_STOP}: a command never answered, so the writes never \
                         stop and convergence cannot be judged"
                    )),
                );
            }
            Some(last) if last.is_some_and(|last| !returned_before(last, operation.invoked_at)) => {
                decided.insert(
                    id,
                    Err(format!(
                        "{BEFORE_WRITES_STOP}: invoked before the last write returned, so it may \
                         show any earlier state"
                    )),
                );
            }
            Some(_) => sessions
                .entry((operation.client, read.view.as_str()))
                .or_default()
                .push(operation),
        }
    }
    for reads in sessions.values_mut() {
        reads.sort_by_key(|operation| operation.invoked_at);
        for (count, operation) in reads.iter().enumerate() {
            let judged = if (count as u64) < settle {
                Err(format!(
                    "{BEFORE_SETTLE}: read {} of its session after the writes stopped, within \
                     the first {settle} that may still be behind",
                    count + 1
                ))
            } else {
                Ok(())
            };
            decided.insert(operation.operation_id.as_str(), judged);
        }
    }
    decided
}

/// The first read, in document order, its view's declared consistency does not allow.
/// Every subject a partition's operations name, with the partition it is searched in, and each
/// partition under its own name: one subject is one partition, except where one request's
/// operations name several ([`RequestPlan`]).
fn subjects<'s>(split: &Split<'s>) -> BTreeMap<&'s str, Partition<'s>> {
    let mut subjects = BTreeMap::new();
    for (partition, operations) in &split.partitions {
        // A subject partition is named by its subject, the empty string included: a `String`
        // identity may be empty, and a read showing that row is explained by its partition.
        if let Partition::Subject(subject) = partition {
            subjects.insert(*subject, *partition);
        }
        for prepared in operations {
            let subject = prepared.operation.subject_key.as_str();
            if !subject.is_empty() {
                subjects.insert(subject, *partition);
            }
        }
    }
    subjects
}

fn judge(
    split: &Split<'_>,
    subjects: &BTreeMap<&str, Partition<'_>>,
    reached: &BTreeMap<Partition<'_>, Vec<Reached>>,
    convergence: &BTreeMap<&str, Result<(), String>>,
    unresolved: &BTreeMap<Partition<'_>, CheckRefusal>,
) -> Result<Option<ReadViolation>, CheckRefusal> {
    for read in &split.reads {
        let operation = read.operation;
        let returned = operation.returned_at.unwrap_or(u64::MAX);
        let converges = matches!(
            convergence.get(operation.operation_id.as_str()),
            Some(Ok(()))
        );
        let violation = |subject: &str, shown: bool, anomaly: Anomaly| ReadViolation {
            operation_id: operation.operation_id.as_str().to_owned(),
            client: operation.client,
            view: read.view.clone(),
            consistency: read.consistency.as_str().to_owned(),
            subject_key: subject.to_owned(),
            shown,
            anomaly,
        };
        // A row-level view holds one row per instance; the same identity twice is no state.
        if let Some(row) = read.duplicate {
            return Ok(Some(violation(row, true, Anomaly::DuplicateRow)));
        }
        for row in &read.rows {
            if !subjects.contains_key(row) {
                return Ok(Some(violation(row, true, Anomaly::FutureRead)));
            }
        }
        for (subject, name) in subjects {
            let Some(states) = reached.get(name) else {
                continue;
            };
            let shown = read.rows.contains(subject);
            let partition = &split.partitions[name];
            // Exhaustive on purpose: a level added to the language is decided here, not defaulted.
            // The reader's own operations on this subject that returned before it read.
            let own: Vec<usize> = match read.consistency {
                Consistency::ReadYourWrites => partition
                    .iter()
                    .enumerate()
                    .filter(|(_, prepared)| {
                        let own = prepared.operation;
                        own.client == operation.client
                            && (*name == Partition::Shared
                                || own.subject_key == *subject
                                || own.subject_key.is_empty())
                            && own.completion == Completion::Returned
                            && returned_before(own, operation.invoked_at)
                    })
                    .map(|(index, _)| index)
                    .collect(),
                Consistency::Eventual => Vec::new(),
            };
            let answers = |state: &&Reached| {
                state
                    .store
                    .lifecycle(read.entity, subject)
                    .is_some_and(|state| read.admits.contains(state))
                    == shown
            };
            let asked = |state: &&Reached| state.latest <= returned;
            let covers = |state: &&Reached| own.iter().all(|&index| state.done.has(index));
            if converges {
                if states
                    .iter()
                    .filter(|state| state.full)
                    .any(|state| answers(&state))
                {
                    continue;
                }
                if let Some(why) = unresolved.get(name) {
                    return Err(why.clone());
                }
                return Ok(Some(violation(subject, shown, Anomaly::NotConverged)));
            }
            if states
                .iter()
                .any(|state| asked(&state) && covers(&state) && answers(&state))
            {
                continue;
            }
            if let Some(why) = unresolved.get(name) {
                return Err(why.clone());
            }
            let anomaly = if states.iter().any(|state| asked(&state) && answers(&state)) {
                Anomaly::StaleRead
            } else {
                Anomaly::FutureRead
            };
            return Ok(Some(violation(subject, shown, anomaly)));
        }
    }
    Ok(None)
}

// ---- shrinking -------------------------------------------------------------------------------

/// `history` holding only the operations `keep` admits, in document order.
fn only(history: &History, keep: impl Fn(&Operation) -> bool) -> History {
    let mut kept = history.clone();
    kept.operations.retain(|operation| keep(operation));
    // A part of a history written as `ess-history/2` is written as format 2 only while one of its
    // operations still records a decision time.
    if kept.format == HistoryFormat::EssHistory2 {
        kept.format = HistoryFormat::for_operations(&kept.operations);
    }
    kept
}

/// How many operations of the partition that decided a command violation no order explains: that
/// partition's commands less its longest partial linearization.
///
/// Independent of the order the search tries moves in. A violation is only reported after the
/// search has reached every state it can, so the longest linearization is the longest there is; its
/// length, unlike which operations it holds, is a property of the history.
fn unexplained(ir: &EssIr, history: &History, checked: &Checked) -> usize {
    let commands = history
        .operations
        .iter()
        .filter(|operation| {
            Some(operation.subject_key.as_str()) == checked.subject_key.as_deref()
                && !reads_a_view(ir, operation)
        })
        .count();
    commands.saturating_sub(checked.linearization.len())
}

/// What a trial must still be for a removal to be kept.
#[derive(Debug, Clone)]
enum Same {
    /// A command violation with no more unexplained operations than this.
    Command(usize),
    /// A violation by this read, of this subject, of this kind.
    Read(ReadViolation),
}

impl Same {
    /// What `candidate` is, where it is still the same violation.
    fn still(
        &self,
        ir: &EssIr,
        candidate: &History,
        budget: u64,
        settle: u64,
    ) -> Result<Option<Self>, CheckRefusal> {
        if candidate.operations.is_empty() {
            return Ok(None);
        }
        let checked = check_settled(ir, candidate, budget, settle)?;
        if checked.verdict != Verdict::Violation {
            return Ok(None);
        }
        Ok(match (self, &checked.read) {
            (Self::Command(bound), None) => {
                let count = unexplained(ir, candidate, &checked);
                (count <= *bound).then_some(Self::Command(count))
            }
            (Self::Read(read), Some(found))
                if found.operation_id == read.operation_id
                    && found.subject_key == read.subject_key
                    && found.anomaly == read.anomaly =>
            {
                Some(self.clone())
            }
            _ => None,
        })
    }
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
///    P-compositionality makes safe. For a command violation every read goes too; for a read
///    violation the read stays, answering only for that subject;
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
/// * a removal is kept only when the result is still a violation of the same kind: for a command
///   violation, the number of operations no order explains has not grown since the last removal
///   kept; for a read violation, the same read still shows the same subject with the same
///   [`Anomaly`].
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
    shrink_settled(ir, history, budget, DEFAULT_SETTLE)
}

/// [`shrink`], judging every trial with [`check_settled`] at `settle`.
///
/// # Errors
///
/// [`CheckRefusal`] where a trial cannot be checked.
pub fn shrink_settled(
    ir: &EssIr,
    history: &History,
    budget: u64,
    settle: u64,
) -> Result<History, CheckRefusal> {
    let checked = check_settled(ir, history, budget, settle)?;
    if checked.verdict != Verdict::Violation {
        return Ok(history.clone());
    }
    let subject = checked.subject_key.as_deref();
    let (mut same, partition) = match &checked.read {
        None => (
            Same::Command(unexplained(ir, history, &checked)),
            only(history, |operation| {
                Some(operation.subject_key.as_str()) == subject && !reads_a_view(ir, operation)
            }),
        ),
        Some(read) => {
            let is_read =
                |operation: &Operation| operation.operation_id.as_str() == read.operation_id;
            let mut partition = only(history, |operation| {
                is_read(operation)
                    || (Some(operation.subject_key.as_str()) == subject
                        && !reads_a_view(ir, operation))
            });
            for operation in &mut partition.operations {
                if is_read(operation) {
                    if let Some(rows) = &mut operation.rows {
                        rows.retain(|row| *row == read.subject_key);
                    }
                }
            }
            (Same::Read(read.clone()), partition)
        }
    };
    let mut current = history.clone();
    if let Some(next) = same.still(ir, &partition, budget, settle)? {
        current = partition;
        same = next;
    }
    loop {
        let mut removed = false;
        let clients: BTreeSet<u64> = current.operations.iter().map(|it| it.client).collect();
        for client in clients {
            let Some(candidate) = without(&current, |operation| operation.client == client) else {
                continue;
            };
            if let Some(next) = same.still(ir, &candidate, budget, settle)? {
                current = candidate;
                same = next;
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
            if let Some(next) = same.still(ir, &candidate, budget, settle)? {
                current = candidate;
                same = next;
                removed = true;
            }
        }
        if !removed {
            break;
        }
    }
    let relabelled = relabel(&current);
    if same.still(ir, &relabelled, budget, settle)?.is_some() {
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

// ---- what a page draws -----------------------------------------------------------------------

/// Every subject partition, in subject order, with the order of its operations the search found
/// — a complete order where one exists, otherwise the longest partial one — by operation id.
///
/// The same search [`check`] runs, over every partition rather than up to the first violation,
/// spending at most `budget` executions of the model between them. A partition searched after the
/// budget ran out has the longest order found before it did, which may be empty.
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
pub fn orders(
    ir: &EssIr,
    history: &History,
    budget: u64,
) -> Result<Vec<(String, Vec<String>)>, CheckRefusal> {
    let split = split(ir, history)?;
    let mut remaining = budget;
    let mut spent = 0;
    let mut found = Vec::new();
    for (subject_key, operations) in &split.partitions {
        let order = match search(ir, operations, &mut remaining, &mut spent)? {
            Outcome::Linearizable(order) | Outcome::Violation(order) | Outcome::Unknown(order) => {
                order
            }
            Outcome::Unresolved(why) => return Err(why),
        };
        found.push((
            subject_key.diagnostic(operations),
            order
                .iter()
                .map(|&index| operations[index].operation.operation_id.as_str().to_owned())
                .collect(),
        ));
    }
    Ok(found)
}

/// One side of a [`Conflict`]: an operation, the state its recorded answer needed its subject in,
/// and the state an order supplied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConflictSide {
    /// The operation.
    pub operation_id: String,
    /// The subject's lifecycle state in each model state from which the operation answers what it
    /// recorded, sorted; `absent` where no instance with that identity is held.
    pub required: Vec<String>,
    /// The subject's lifecycle state in each model state the order supplied to it, sorted, spelt
    /// as `required` is.
    pub supplied: Vec<String>,
}

/// The two operations a command violation turns on.
///
/// The longest partial linearization places `against` and cannot then place `failing`: ordered
/// right after `against`, `failing` finds its subject in `failing.supplied` and needed
/// `failing.required`. The other way round — `failing` where `against` stands, which the recorded
/// instants allow — leaves `against` the state in `against.supplied`, from which it cannot answer
/// what it recorded either; it needed `against.required`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Conflict {
    /// The subject whose partition decided the violation.
    pub subject_key: String,
    /// The first operation, in invoke order, the longest order could not place.
    pub failing: ConflictSide,
    /// The latest operation of the longest order from before which `failing` answers what it
    /// recorded.
    pub against: ConflictSide,
}

/// The subject's lifecycle state in each of `stores`, sorted and without repeats.
fn subject_states<'s>(stores: impl IntoIterator<Item = &'s State>, subject: &str) -> Vec<String> {
    let mut states = BTreeSet::new();
    for store in stores {
        let mut held = store
            .text_states()
            .filter(|(identity, _)| *identity == subject)
            .peekable();
        if held.peek().is_none() {
            states.insert("absent".to_owned());
        }
        for (_, state) in held {
            states.insert(state.as_str().to_owned());
        }
    }
    states.into_iter().collect()
}

/// For a command violation `checked` found in `history`: the two operations it turns on, and the
/// state each needed. `None` for any other verdict, for a read violation, and where no single
/// operation of the longest order is one the failing operation conflicts with — where the failure
/// takes more than two operations to explain, or the recorded instants forbid the swap.
///
/// Replays [`Checked::linearization`] through the model, keeping every state each prefix can
/// leave, then walks it back from the end to the last placed operation `against` such that the
/// first unplaced operation may be ordered before it (nothing placed from `against` on returned
/// before it was invoked), answers what it recorded from the state before `against` and not from
/// the state after, and leaves a state from which `against` cannot answer what it recorded. It
/// does not spend a search budget; it executes the model a bounded number of times per state of
/// the replay.
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
#[allow(
    clippy::too_many_lines,
    reason = "conflict extraction keeps all history completion cases in one exhaustive pass"
)]
pub fn conflict(
    ir: &EssIr,
    history: &History,
    checked: &Checked,
) -> Result<Option<Conflict>, CheckRefusal> {
    if checked.verdict != Verdict::Violation || checked.read.is_some() {
        return Ok(None);
    }
    let Some(subject) = checked.subject_key.as_deref() else {
        return Ok(None);
    };
    let split = split(ir, history)?;
    let Some(operations) = split
        .partitions
        .get(&Partition::Shared)
        .or_else(|| split.partitions.get(&Partition::Subject(subject)))
    else {
        return Ok(None);
    };
    let Some(placed) = checked
        .linearization
        .iter()
        .map(|id| {
            operations
                .iter()
                .position(|prepared| prepared.operation.operation_id.as_str() == id)
        })
        .collect::<Option<Vec<usize>>>()
    else {
        return Ok(None);
    };
    let Some(failing) = move_order(operations)
        .into_iter()
        .find(|index| !placed.contains(index))
    else {
        return Ok(None);
    };
    let mut prefixes: Vec<Vec<Held>> = vec![vec![Held::default()]];
    for &index in &placed {
        let mut next: Vec<Held> = Vec::new();
        for store in prefixes.last().into_iter().flatten() {
            for reached in step(ir, store, &operations[index])?.determined()? {
                if !next.contains(&reached) {
                    next.push(reached);
                }
            }
        }
        prefixes.push(next);
    }
    let id = |index: usize| operations[index].operation.operation_id.as_str().to_owned();
    let answers_from = |stores: &[Held], index: usize| -> Result<bool, CheckRefusal> {
        for store in stores {
            if !step(ir, store, &operations[index])?
                .determined()?
                .is_empty()
            {
                return Ok(true);
            }
        }
        Ok(false)
    };
    for at in (0..placed.len()).rev() {
        let against = placed[at];
        // Moving `failing` before `against` moves it before everything placed after `against` too;
        // an operation there that returned before `failing` was invoked forbids it.
        if placed[at..]
            .iter()
            .any(|&index| precedes(operations[index].operation, operations[failing].operation))
            || precedes(operations[failing].operation, operations[against].operation)
        {
            continue;
        }
        let before = &prefixes[at];
        let after = &prefixes[at + 1];
        if answers_from(after, failing)? {
            continue;
        }
        let mut explained: Vec<&Held> = Vec::new();
        let mut instead: Vec<Held> = Vec::new();
        for store in before {
            let next = step(ir, store, &operations[failing])?.determined()?;
            if !next.is_empty() {
                explained.push(store);
                instead.extend(next);
            }
        }
        if explained.is_empty() || answers_from(&instead, against)? {
            continue;
        }
        let mut needed: Vec<&Held> = Vec::new();
        for store in before {
            if !step(ir, store, &operations[against])?
                .determined()?
                .is_empty()
            {
                needed.push(store);
            }
        }
        return Ok(Some(Conflict {
            subject_key: subject.to_owned(),
            failing: ConflictSide {
                operation_id: id(failing),
                required: subject_states(explained.into_iter().map(|held| &held.store), subject),
                supplied: subject_states(after.iter().map(|held| &held.store), subject),
            },
            against: ConflictSide {
                operation_id: id(against),
                required: subject_states(needed.into_iter().map(|held| &held.store), subject),
                supplied: subject_states(instead.iter().map(|held| &held.store), subject),
            },
        }));
    }
    Ok(None)
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
    /// The reads not judged here, each with its reason.
    pub not_judged: Vec<NotJudged>,
    /// How many reads were judged at their view's consistency. Not written in the JSON report,
    /// whose fields are those of `check-history`'s first release plus `read`.
    #[serde(skip)]
    pub judged: usize,
    /// For a violation a read decided: the client, the read and what it showed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<ReadViolation>,
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
    report_settled(ir, history, budget, DEFAULT_SETTLE)
}

/// [`report`], with convergence judged at `settle` ([`check_settled`]).
///
/// # Errors
///
/// [`CheckRefusal`] where the history cannot be checked against this model at all.
pub fn report_settled(
    ir: &EssIr,
    history: &History,
    budget: u64,
    settle: u64,
) -> Result<Report, CheckRefusal> {
    let checked = check_settled(ir, history, budget, settle)?;
    let shrunk = match checked.verdict {
        Verdict::Violation => Some(shrink_settled(ir, history, budget, settle)?),
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
        judged: checked.judged,
        read: checked.read,
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
            self.operations - self.not_judged.len() - self.judged,
            self.partitions,
            self.steps,
            self.budget
        );
        if self.judged > 0 {
            let _ = writeln!(
                text,
                "judged {} view read(s) at their declared consistency",
                self.judged
            );
        }
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
        if let Some(read) = &self.read {
            let _ = writeln!(
                text,
                "read violation: client {} read {} of `{}`, declared {}: {} — it {} `{}`",
                read.client,
                read.operation_id,
                read.view,
                read.consistency,
                read.anomaly.as_str(),
                if read.shown { "shows" } else { "does not show" },
                read.subject_key
            );
        }
        for read in &self.not_judged {
            let _ = writeln!(
                text,
                "not judged: {} reads `{}`, declared {}: {}",
                read.operation_id, read.view, read.consistency, read.reason
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
                    "  client {} [{}, {}] {} `{}` -> {}{}{}",
                    operation.client,
                    operation.invoked_at,
                    returned,
                    operation.command.as_str(),
                    operation.subject_key,
                    operation
                        .outcome
                        .as_ref()
                        .map_or("no answer", |outcome| outcome.as_str()),
                    operation
                        .rows
                        .as_ref()
                        .map_or_else(String::new, |rows| format!(" [{}]", rows.join(", "))),
                    operation
                        .retry_of
                        .as_ref()
                        .map_or_else(String::new, |original| format!(
                            " (a retry of {})",
                            original.as_str()
                        )),
                );
            }
        }
        text
    }
}
