//! A deterministic recorder: two to four clients driven against one target, written as
//! `ess-history/1`.
//!
//! The Go and TypeScript runners that drive an adopter's target concurrently are a later story
//! (`story:concurrent-explorer-runner`). Until they exist, the checker in [`crate::linearize`] needs
//! histories recorded against the Rust targets, and this is where they come from. Nothing here runs
//! a thread or reads a clock: a seed picks, at every tick of a logical clock, which client acts next,
//! so one seed over one workload and one target records one history, byte for byte.
//!
//! # Invoke and return are two moves
//!
//! A [`ConformanceTarget`] answers a command in one call, which is one instant. A concurrent client
//! sees two: the call leaves at its invoke instant and the answer arrives at its return instant, and
//! other clients' calls happen in between. [`Interleaved`] is that split. [`Atomic`] gives it to any
//! target by doing all of the work at the return instant — which is a correct implementation: an
//! atomic call takes effect at one point between its invoke and its return, and that is exactly what
//! linearizability asks. A target that does part of its work at the invoke instant and part at the
//! return instant, without a lock between them, is one the history can catch
//! ([`Fault::LostUpdate`](crate::faulty::Fault::LostUpdate)).
//!
//! # The shape of a run
//!
//! PULSE's: a sequential prefix, then a parallel suffix. [`Workload::prefix`] runs on client 0 one
//! call at a time, so the subjects the suffix acts on exist before it starts; each list in
//! [`Workload::clients`] is one client's calls, in order, and a client never has two calls in
//! flight. At every tick the seed picks one enabled move — a client with no call in flight invokes
//! its next call, or a client with one in flight receives its answer.

use std::collections::BTreeMap;
use std::fmt;

use ess_compiler::ir::{EssIr, ResolvedEffect, ResolvedInstance};
use ess_domain::name::QualifiedName as ModelName;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

use crate::history::{
    Completion, History, HistoryFormat, Operation, QualifiedName, Uuid, MAX_INTEGER,
};
use crate::scenario::{CommandRef, SuiteProvenance};
use crate::target::{
    ConformanceTarget, SemanticCommandRequest, SemanticCommandResult, TargetError,
};

/// A target whose calls have an invoke and a return, with other clients' calls in between.
pub trait Interleaved {
    /// What a call in flight carries from its invoke to its return.
    type Pending;

    /// The call leaves the client.
    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending;

    /// The answer arrives. An error is a call that never answered.
    ///
    /// # Errors
    ///
    /// [`TargetError`] where the target gave no answer; the recorder writes the call
    /// [`Completion::Indeterminate`].
    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError>;
}

/// Any target, with every call taking effect at its return instant.
#[derive(Debug)]
pub struct Atomic<'t, T>(pub &'t T);

impl<T: ConformanceTarget> Interleaved for Atomic<'_, T> {
    type Pending = SemanticCommandRequest;

    fn invoke(&self, request: SemanticCommandRequest) -> Self::Pending {
        request
    }

    fn complete(&self, pending: Self::Pending) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(pending)
    }
}

/// Which instance a call acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// The call creates it; its identity is read from the event the model says publishes it.
    Creates,
    /// The instance the prefix call at this index created.
    Created(usize),
}

/// One call a client makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    /// The command, as the model names it.
    pub command: String,
    /// The input, without the field that names the subject: the recorder fills that one.
    pub input: BTreeMap<String, Node>,
    /// Which instance it acts on.
    pub subject: Subject,
}

impl Call {
    /// A call of `command` with `input` on `subject`.
    pub fn new(
        command: impl Into<String>,
        input: BTreeMap<String, Node>,
        subject: Subject,
    ) -> Self {
        Self {
            command: command.into(),
            input,
            subject,
        }
    }
}

/// What the clients do: a sequential prefix, then one list of calls per client.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Workload {
    /// Run on client 0 before any other client starts, one call at a time.
    pub prefix: Vec<Call>,
    /// One list per client, each in the order that client makes its calls.
    pub clients: Vec<Vec<Call>>,
}

/// Why a workload could not be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// The workload names no client, or more than [`MAX_INTEGER`].
    NoClients,
    /// A call names a command the model does not declare.
    UnknownCommand(String),
    /// A call acts on a prefix call that created nothing.
    NoSuchSubject {
        /// The prefix index named.
        prefix: usize,
    },
    /// A call names its subject in a command that takes none.
    NoSubjectField(String),
    /// The seed is above [`MAX_INTEGER`], which `ess-history/1` cannot carry.
    SeedOutOfRange(u64),
}

impl fmt::Display for RecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoClients => write!(formatter, "the workload names no client"),
            Self::UnknownCommand(command) => {
                write!(formatter, "the model declares no command `{command}`")
            }
            Self::NoSuchSubject { prefix } => {
                write!(formatter, "prefix call {prefix} created no instance")
            }
            Self::NoSubjectField(command) => write!(
                formatter,
                "`{command}` takes no input field that names an existing instance"
            ),
            Self::SeedOutOfRange(seed) => write!(
                formatter,
                "the seed {seed} is above {MAX_INTEGER}, the largest `ess-history/1` carries"
            ),
        }
    }
}

impl std::error::Error for RecordError {}

/// The seed's sequence: `SplitMix64`, written out so no crate is needed and no platform varies it.
#[derive(Debug, Clone)]
struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        // `bound` is at most a handful of clients, so the modulo bias is immaterial and, more to the
        // point, deterministic.
        usize::try_from(self.next() % bound as u64).unwrap_or(0)
    }
}

/// The identity the recorder writes for the `n`th thing it names.
fn uuid(n: u64) -> Uuid {
    Uuid::new(format!("00000000-0000-4000-8000-{n:012x}"))
        .unwrap_or_else(|error| panic!("a counter-shaped UUID is canonical: {error}"))
}

/// The recorder's working state for one run.
struct Recording<'a, T: Interleaved> {
    ir: &'a EssIr,
    target: &'a T,
    clock: u64,
    operations: Vec<Operation>,
    created: Vec<Option<String>>,
}

/// A call in flight.
struct InFlight<P> {
    pending: P,
    command: String,
    subject_key: Option<String>,
    index: usize,
}

impl<T: Interleaved> Recording<'_, T> {
    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    /// Builds the request for `call`, filling the field that names its subject.
    fn request(
        &self,
        call: &Call,
    ) -> Result<(SemanticCommandRequest, Option<String>), RecordError> {
        let name = ModelName::new(&call.command)
            .map_err(|_| RecordError::UnknownCommand(call.command.clone()))?;
        let spec = self
            .ir
            .commands()
            .get(&name)
            .ok_or_else(|| RecordError::UnknownCommand(call.command.clone()))?;
        let mut input = call.input.clone();
        let subject_key = match call.subject {
            Subject::Creates => None,
            Subject::Created(prefix) => {
                let key = self
                    .created
                    .get(prefix)
                    .cloned()
                    .flatten()
                    .ok_or(RecordError::NoSuchSubject { prefix })?;
                let field = spec
                    .outcomes
                    .iter()
                    .filter_map(|outcome| outcome.subject.as_ref())
                    .find_map(|subject| match &subject.instance {
                        ResolvedInstance::Supplied { field } => Some(field.name.clone()),
                        ResolvedInstance::Observed { .. } => None,
                    })
                    .ok_or_else(|| RecordError::NoSubjectField(call.command.clone()))?;
                input.insert(field, Node::Text(key.clone()));
                Some(key)
            }
        };
        let request = SemanticCommandRequest {
            command: CommandRef::new(name),
            actor: None,
            input,
            correlation: CorrelationId::new(format!("history-{}", self.operations.len() + 1))
                .unwrap_or_else(|error| panic!("a counter-shaped correlation is valid: {error}")),
        };
        Ok((request, subject_key))
    }

    /// The identity a creating command's answer published, where the model says where it is.
    fn created_identity(&self, command: &str, result: &SemanticCommandResult) -> Option<String> {
        let name = ModelName::new(command).ok()?;
        let spec = self.ir.commands().get(&name)?;
        let taken = result.outcome.as_ref()?;
        let outcome = spec
            .outcomes
            .iter()
            .find(|outcome| outcome.name == taken.outcome)?;
        let subject = outcome.subject.as_ref()?;
        let (ResolvedEffect::Creates, ResolvedInstance::Observed { event, field }) =
            (&subject.effect, &subject.instance)
        else {
            return None;
        };
        let event = &self.ir.event(event).name;
        result
            .direct_events
            .iter()
            .find(|occurrence| occurrence.event.name() == event)
            .and_then(|occurrence| occurrence.payload.get(&field.name))
            .and_then(Node::as_text)
            .map(ToOwned::to_owned)
    }

    fn invoke(&mut self, client: u64, call: &Call) -> Result<InFlight<T::Pending>, RecordError> {
        let (request, subject_key) = self.request(call)?;
        let invoked_at = self.tick();
        let index = self.operations.len();
        // Reserve the operation's place in invoke order; `complete` fills it in.
        self.operations.push(Operation {
            operation_id: uuid(index as u64 + 1),
            client,
            command: QualifiedName::new(call.command.clone())
                .unwrap_or_else(|error| panic!("a declared command has a name: {error}")),
            subject_key: String::new(),
            invoked_at,
            returned_at: None,
            completion: Completion::Indeterminate,
            outcome: None,
        });
        Ok(InFlight {
            pending: self.target.invoke(request),
            command: call.command.clone(),
            subject_key,
            index,
        })
    }

    /// Writes the answer into the operation `flight` reserved, and returns the identity a creating
    /// command published.
    fn complete(&mut self, flight: InFlight<T::Pending>) -> Option<String> {
        let answer = self.target.complete(flight.pending);
        let returned_at = self.tick();
        let Ok(result) = answer else {
            // No answer: the operation stays `Indeterminate`, with no return instant and no outcome.
            self.operations[flight.index].subject_key = flight.subject_key.unwrap_or_default();
            return None;
        };
        let created = self.created_identity(&flight.command, &result);
        // A result naming no declared branch is written under a name no branch can have, so the
        // checker finds no step that answers it.
        let outcome = result
            .outcome
            .as_ref()
            .map_or_else(|| UNDECLARED.to_owned(), |taken| taken.outcome.to_string());
        let operation = &mut self.operations[flight.index];
        operation.subject_key = flight
            .subject_key
            .or_else(|| created.clone())
            .unwrap_or_default();
        operation.returned_at = Some(returned_at);
        operation.completion = Completion::Returned;
        operation.outcome = Some(
            QualifiedName::new(outcome)
                .unwrap_or_else(|error| panic!("an outcome has a name: {error}")),
        );
        created
    }
}

/// The outcome the recorder writes for an answer that names no declared branch.
pub const UNDECLARED: &str = "<undeclared>";

/// Records `workload` against `target`, interleaved by `seed`.
///
/// Operations are written in invoke order and numbered from 1; instants come from one logical
/// clock that advances on every invoke and every return. The history carries the digest of `ir`.
///
/// # Errors
///
/// [`RecordError`] where the workload names something the model does not declare.
pub fn record<T: Interleaved>(
    ir: &EssIr,
    target: &T,
    workload: &Workload,
    seed: u64,
) -> Result<History, RecordError> {
    let clients = workload.clients.len().max(1) as u64;
    if workload.clients.is_empty() && workload.prefix.is_empty() || clients > MAX_INTEGER {
        return Err(RecordError::NoClients);
    }
    if seed > MAX_INTEGER {
        return Err(RecordError::SeedOutOfRange(seed));
    }
    let mut recording = Recording {
        ir,
        target,
        clock: 0,
        operations: Vec::new(),
        created: Vec::new(),
    };
    for call in &workload.prefix {
        let flight = recording.invoke(0, call)?;
        let created = recording.complete(flight);
        recording.created.push(created);
    }

    let mut draw = Draw(seed);
    let mut next: Vec<usize> = vec![0; workload.clients.len()];
    let mut flying: Vec<Option<InFlight<T::Pending>>> =
        workload.clients.iter().map(|_| None).collect();
    loop {
        let enabled: Vec<usize> = (0..workload.clients.len())
            .filter(|&client| {
                flying[client].is_some() || next[client] < workload.clients[client].len()
            })
            .collect();
        if enabled.is_empty() {
            break;
        }
        let client = enabled[draw.below(enabled.len())];
        if let Some(flight) = flying[client].take() {
            recording.complete(flight);
        } else {
            let call = &workload.clients[client][next[client]];
            next[client] += 1;
            flying[client] = Some(recording.invoke(client as u64, call)?);
        }
    }

    Ok(History {
        format: HistoryFormat::EssHistory1,
        history_id: uuid(seed & 0xffff_ffff_ffff),
        spec_digest: SuiteProvenance::of(ir).spec_digest,
        seed,
        clients,
        operations: recording.operations,
    })
}
