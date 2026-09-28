//! A deterministic recorder whose clients read views as well as call commands, each client a
//! session.
//!
//! [`crate::record`] drives commands only, which is all a linearizability check of commands needs.
//! A view's declared consistency is a claim about **reads** — `read_your_writes` is a promise to
//! one client about its own writes, `eventual` a promise about what every client reads once the
//! writes stop — so a history that is to hold a view to it has to carry reads, and each read has
//! to be made the way a client in a session makes it. This recorder is [`crate::record`]'s
//! interleaving (the same seed sequence, the same logical clock, the same sequential prefix on
//! client 0) with one more kind of move, [`Act::Read`].
//!
//! # A read, as a session makes it
//!
//! A client keeps the consistency token its own last command answered with. A read of a
//! `read_your_writes` view demands that token (`AtLeast`), and a client with none reads at
//! `Current`; a read of an `eventual` view is always at `Current`, because demanding a token would
//! make the target wait until it caught up and there would be nothing eventual left to observe.
//! The prefix runs on client 0, so its tokens are client 0's.
//!
//! A read is invoked at one instant and answered at a later one, like a call; the target is asked
//! at the return instant. What it answered is written as [`Operation::rows`]: the identity of each
//! row, read off the field that carries the identity of the entity the view projects. A read
//! whose rows do not all carry it is written without rows, which the checker lists as not judged
//! rather than guessing. A read's `outcome` is [`READ`] and its `subject_key` is empty: a view is
//! not one instance's.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_domain::name::QualifiedName as ModelName;
use ess_domain::view::Consistency;
use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;

use crate::history::{Completion, History, HistoryFormat, Operation, QualifiedName, MAX_INTEGER};
use crate::record::{
    command_request, correlation, created_identity, uuid, Call, Draw, Interleaved, RecordError,
    UNDECLARED,
};
use crate::scenario::{SuiteProvenance, ViewRef};
use crate::target::{ConformanceTarget, Deadline, SemanticViewRequest, SemanticViewResult};

/// The outcome written for every answered read.
pub const READ: &str = "read";

/// One thing a client does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// A command.
    Call(Call),
    /// A read of the view this names, as the model names it.
    Read(String),
}

/// What the clients do: a sequential prefix of commands on client 0, then one list per client.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Workload {
    /// Run on client 0 before any other client starts, one call at a time.
    pub prefix: Vec<Call>,
    /// One list per client, each in the order that client acts.
    pub clients: Vec<Vec<Act>>,
}

/// A move in flight.
enum Pending<P> {
    Call {
        pending: P,
        command: String,
        subject_key: Option<String>,
    },
    Read {
        request: SemanticViewRequest,
        identity: String,
    },
}

struct InFlight<P> {
    pending: Pending<P>,
    client: usize,
    index: usize,
}

struct Recording<'a, T: Interleaved, V> {
    ir: &'a EssIr,
    target: &'a T,
    views: &'a V,
    clock: u64,
    operations: Vec<Operation>,
    created: Vec<Option<String>>,
    /// Each client's last consistency token.
    tokens: Vec<Option<ConsistencyToken>>,
}

impl<T: Interleaved, V: ConformanceTarget> Recording<'_, T, V> {
    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    /// Reserves an operation's place in invoke order; `complete` fills it in.
    fn reserve(&mut self, client: usize, name: &str) -> usize {
        let invoked_at = self.tick();
        let index = self.operations.len();
        self.operations.push(Operation {
            operation_id: uuid(index as u64 + 1),
            client: client as u64,
            command: QualifiedName::new(name.to_owned())
                .unwrap_or_else(|error| panic!("a declared name is not empty: {error}")),
            subject_key: String::new(),
            invoked_at,
            returned_at: None,
            completion: Completion::Indeterminate,
            outcome: None,
            rows: None,
        });
        index
    }

    fn invoke_call(
        &mut self,
        client: usize,
        call: &Call,
    ) -> Result<InFlight<T::Pending>, RecordError> {
        let (request, subject_key) = command_request(
            self.ir,
            &self.created,
            call,
            correlation(self.operations.len() + 1),
        )?;
        let index = self.reserve(client, &call.command);
        Ok(InFlight {
            pending: Pending::Call {
                pending: self.target.invoke(request),
                command: call.command.clone(),
                subject_key,
            },
            client,
            index,
        })
    }

    fn invoke_read(
        &mut self,
        client: usize,
        view: &str,
    ) -> Result<InFlight<T::Pending>, RecordError> {
        let unknown = || RecordError::UnknownCommand(view.to_owned());
        let name = ModelName::new(view).map_err(|_| unknown())?;
        let spec = self.ir.views().get(&name).ok_or_else(unknown)?;
        let consistency = match (spec.consistency, &self.tokens[client]) {
            (Consistency::ReadYourWrites, Some(token)) => QueryConsistency::at_least(token.clone()),
            (Consistency::ReadYourWrites, None) | (Consistency::Eventual, _) => {
                QueryConsistency::Current
            }
        };
        let identity = self.ir.entity(&spec.source).identity.name.clone();
        let request = SemanticViewRequest {
            view: ViewRef::new(name),
            params: BTreeMap::new(),
            consistency,
            correlation: correlation(self.operations.len() + 1),
            deadline: Deadline::at(Timestamp::from_epoch_millis(u64::MAX)),
        };
        let index = self.reserve(client, view);
        Ok(InFlight {
            pending: Pending::Read { request, identity },
            client,
            index,
        })
    }

    /// Writes the answer into the reserved operation; returns what a creating command published.
    fn complete(&mut self, flight: InFlight<T::Pending>) -> Option<String> {
        match flight.pending {
            Pending::Call {
                pending,
                command,
                subject_key,
            } => {
                let answer = self.target.complete(pending);
                let returned_at = self.tick();
                let Ok(result) = answer else {
                    self.operations[flight.index].subject_key = subject_key.unwrap_or_default();
                    return None;
                };
                if let Some(token) = &result.consistency {
                    self.tokens[flight.client] = Some(token.clone());
                }
                let created = created_identity(self.ir, &command, &result);
                let outcome = result
                    .outcome
                    .as_ref()
                    .map_or_else(|| UNDECLARED.to_owned(), |taken| taken.outcome.to_string());
                let operation = &mut self.operations[flight.index];
                operation.subject_key = subject_key.or_else(|| created.clone()).unwrap_or_default();
                answered(operation, returned_at, &outcome);
                created
            }
            Pending::Read { request, identity } => {
                let answer = self.views.query_view(request);
                let returned_at = self.tick();
                if let Ok(result) = answer {
                    let operation = &mut self.operations[flight.index];
                    answered(operation, returned_at, READ);
                    operation.rows = rows(&result, &identity);
                }
                None
            }
        }
    }
}

fn answered(operation: &mut Operation, returned_at: u64, outcome: &str) {
    operation.returned_at = Some(returned_at);
    operation.completion = Completion::Returned;
    operation.outcome = Some(
        QualifiedName::new(outcome.to_owned())
            .unwrap_or_else(|error| panic!("an outcome has a name: {error}")),
    );
}

/// The identity of every row, or `None` where one of them does not carry it as text.
fn rows(result: &SemanticViewResult, identity: &str) -> Option<Vec<String>> {
    result
        .rows
        .iter()
        .map(|row| {
            row.get(identity)
                .and_then(Node::as_text)
                .map(ToOwned::to_owned)
        })
        .collect()
}

/// Records `workload` against `target`, sending every read to `views`, interleaved by `seed`.
///
/// Operations are written in invoke order and numbered from 1; instants come from one logical
/// clock that advances on every invoke and every return. `views` is the same implementation as
/// `target` seen through [`ConformanceTarget`] — the reference itself where `target` is
/// [`Atomic`](crate::record::Atomic) over it.
///
/// # Errors
///
/// [`RecordError`] where the workload names something the model does not declare. A read of a
/// view the model does not declare is [`RecordError::UnknownCommand`], the name being neither.
pub fn record<T: Interleaved, V: ConformanceTarget>(
    ir: &EssIr,
    target: &T,
    views: &V,
    workload: &Workload,
    seed: u64,
) -> Result<History, RecordError> {
    let clients = workload.clients.len().max(1);
    if workload.clients.is_empty() && workload.prefix.is_empty() || clients as u64 > MAX_INTEGER {
        return Err(RecordError::NoClients);
    }
    if seed > MAX_INTEGER {
        return Err(RecordError::SeedOutOfRange(seed));
    }
    let mut recording = Recording {
        ir,
        target,
        views,
        clock: 0,
        operations: Vec::new(),
        created: Vec::new(),
        tokens: vec![None; clients],
    };
    for call in &workload.prefix {
        let flight = recording.invoke_call(0, call)?;
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
            let act = &workload.clients[client][next[client]];
            next[client] += 1;
            flying[client] = Some(match act {
                Act::Call(call) => recording.invoke_call(client, call)?,
                Act::Read(view) => recording.invoke_read(client, view)?,
            });
        }
    }

    Ok(History {
        format: HistoryFormat::EssHistory1,
        history_id: uuid(seed & 0xffff_ffff_ffff),
        spec_digest: SuiteProvenance::of(ir).spec_digest,
        seed,
        clients: clients as u64,
        operations: recording.operations,
    })
}
