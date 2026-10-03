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
//!
//! # The faults the specification declares
//!
//! [`record_injected`] records the same workload with [`FaultInjection::Declared`]: it injects every
//! fault the specification declares and no other, each as one more move the seed schedules.
//!
//! | declared by | injected as | written as |
//! |---|---|---|
//! | a binding's `delivery: at_least_once` | the event that binding reacts to is delivered a second time ([`ConformanceTarget::redeliver_event`]), at a later tick of the seed's choosing | nothing: a delivery is not a client's call |
//! | a command's `replays:` branch | the client sends the same request again, unchanged, while or after the first is in flight | a second operation whose [`Operation::retry_of`] names the first |
//! | a command's `external:` branch | the answer is **delayed** past the client's wait (the target executes it) or never arrives (it never does), the seed drawing which or neither | [`Completion::Indeterminate`] |
//!
//! An event is delivered again only where **every** binding reacting to it declares
//! `at_least_once`: a second delivery reaches everything the event reaches, and a binding declaring
//! `at_most_once` has said it is never delivered twice. A `replays:` branch is itself declared
//! `external:` — whether the retained input matches is not the input's to decide — and it is the
//! retry that exercises it, not a delay. Nothing is injected into the prefix: it arranges the
//! subjects the clients share, one call at a time, and a creation nobody saw answer would leave the
//! clients with nothing to address. A restart of the target is never injected: nothing declares
//! support for one (design decision 6).
//!
//! The injections draw from their own sequence of the seed, and only when [`FaultInjection::Declared`]
//! asks for them; [`record`] draws nothing more, so one seed without injection records exactly the
//! history it recorded before injection existed. [`Injected`] counts what was injected and names
//! the declared branches the injected calls reached.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCondition};
use ess_domain::binding::Delivery;
use ess_domain::name::QualifiedName as ModelName;
use ess_domain::view::Consistency;
use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;
use serde::Serialize;

use crate::history::{Completion, History, HistoryFormat, Operation, QualifiedName, MAX_INTEGER};
use crate::record::{
    command_request, correlation, created_identity, uuid, Call, Draw, Interleaved, RecordError,
    UNDECLARED,
};
use crate::scenario::{EventRef, SuiteProvenance, ViewRef};
use crate::target::{
    ConformanceTarget, Deadline, RedeliveryRequest, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult,
};

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

/// Whether a recording injects the faults the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FaultInjection {
    /// Nothing: every call answers, every event is delivered once, no request is sent twice.
    #[default]
    None,
    /// Every fault the specification declares, and no other — see the
    /// [module documentation](self).
    Declared,
}

/// What a recording injected, and what the injected calls reached.
///
/// Every map is keyed by what declared the fault, so a count under a name the specification does
/// not declare the fault for is a count of an undeclared fault.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Injected {
    /// Second deliveries, by the binding that declares `at_least_once`. One delivery reaches every
    /// binding reacting to the event, and is counted under each.
    pub redeliveries: BTreeMap<String, u64>,
    /// Second deliveries the target refused, by event, with nothing delivered.
    pub refused: BTreeMap<String, u64>,
    /// Client retries, by the command that declares `replays:`.
    pub retries: BTreeMap<String, u64>,
    /// Calls answered after the client stopped waiting, by the command declaring `external:`.
    pub delayed: BTreeMap<String, u64>,
    /// Calls never answered, and never executed, by the command declaring `external:`.
    pub unanswered: BTreeMap<String, u64>,
    /// The declared branch each retried or delayed call's answer took, as `command/outcome`, and
    /// how many times.
    pub reached: BTreeMap<String, u64>,
}

impl Injected {
    /// How many faults were injected, of every kind.
    pub fn total(&self) -> u64 {
        [
            &self.redeliveries,
            &self.retries,
            &self.delayed,
            &self.unanswered,
        ]
        .iter()
        .flat_map(|counts| counts.values())
        .sum()
    }
}

/// A recorded history and what was injected into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// The history.
    pub history: History,
    /// What was injected, all zero under [`FaultInjection::None`].
    pub injected: Injected,
}

/// How an injected external fault ends a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Timeout {
    /// The target answers after the client stopped waiting.
    Delayed,
    /// The target never executes it.
    Unanswered,
}

/// A move in flight.
enum Pending<P> {
    Call {
        pending: P,
        command: String,
        subject_key: Option<String>,
        timeout: Option<Timeout>,
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

/// A client's retry of the call it invoked last.
enum Retry<P> {
    /// Not yet sent: the request, unchanged, and the operation it sends again.
    Waiting {
        request: SemanticCommandRequest,
        command: String,
        subject_key: Option<String>,
        original: usize,
    },
    /// Sent.
    Sent(InFlight<P>),
}

/// An event waiting to be delivered a second time, and the bindings it reaches.
struct Redelivery {
    event: EventRef,
    bindings: Vec<String>,
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
    /// The injections' own sequence, present only under [`FaultInjection::Declared`].
    inject: Option<Draw>,
    /// Whether the prefix is running, when nothing is injected into a call.
    arranging: bool,
    redeliveries: Vec<Redelivery>,
    delivered: u64,
    retries: Vec<Option<Retry<T::Pending>>>,
    injected: Injected,
}

/// Adds one to `counts[key]`.
fn count(counts: &mut BTreeMap<String, u64>, key: &str) {
    *counts.entry(key.to_owned()).or_default() += 1;
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
            retry_of: None,
        });
        index
    }

    /// Whether the command declares a `replays:` branch, which a client retry exercises.
    fn replays(&self, command: &str) -> bool {
        ModelName::new(command)
            .ok()
            .and_then(|name| self.ir.commands().get(&name))
            .is_some_and(|spec| {
                spec.outcomes
                    .iter()
                    .any(|outcome| outcome.replays.is_some())
            })
    }

    /// Whether the command declares an `external:` branch other than a `replays:` one.
    fn external(&self, command: &str) -> bool {
        ModelName::new(command)
            .ok()
            .and_then(|name| self.ir.commands().get(&name))
            .is_some_and(|spec| {
                spec.outcomes.iter().any(|outcome| {
                    outcome.replays.is_none()
                        && matches!(
                            outcome.condition,
                            ResolvedCondition::External { .. }
                                | ResolvedCondition::ExternalWhen { .. }
                        )
                })
            })
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
        let mut timeout = None;
        if !self.arranging && self.inject.is_some() {
            if self.retries[client].is_none() && self.replays(&call.command) {
                self.retries[client] = Some(Retry::Waiting {
                    request: request.clone(),
                    command: call.command.clone(),
                    subject_key: subject_key.clone(),
                    original: index,
                });
            }
            if self.external(&call.command) {
                if let Some(draw) = self.inject.as_mut() {
                    timeout = match draw.below(3) {
                        1 => Some(Timeout::Delayed),
                        2 => Some(Timeout::Unanswered),
                        _ => None,
                    };
                }
            }
        }
        Ok(InFlight {
            pending: Pending::Call {
                pending: self.target.invoke(request),
                command: call.command.clone(),
                subject_key,
                timeout,
            },
            client,
            index,
        })
    }

    /// Sends a client's retry: the request it sent, unchanged, as a new operation.
    fn invoke_retry(
        &mut self,
        client: usize,
        request: SemanticCommandRequest,
        command: &str,
        subject_key: Option<String>,
        original: usize,
    ) -> InFlight<T::Pending> {
        let index = self.reserve(client, command);
        self.operations[index].retry_of = Some(self.operations[original].operation_id.clone());
        count(&mut self.injected.retries, command);
        InFlight {
            pending: Pending::Call {
                pending: self.target.invoke(request),
                command: command.to_owned(),
                subject_key,
                timeout: None,
            },
            client,
            index,
        }
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

    /// Queues a second delivery of every event `result` published that only `at_least_once`
    /// bindings react to.
    fn schedule_redeliveries(&mut self, result: &SemanticCommandResult) {
        if self.inject.is_none() {
            return;
        }
        for occurrence in &result.direct_events {
            let reacting: Vec<(&str, Delivery)> =
                self.ir
                    .bindings()
                    .values()
                    .filter(|binding| {
                        binding.cause.event().is_some_and(|event| {
                            self.ir.event(event).name == *occurrence.event.name()
                        })
                    })
                    .map(|binding| (binding.name.as_str(), binding.delivery))
                    .collect();
            if reacting.is_empty()
                || reacting
                    .iter()
                    .any(|(_, delivery)| *delivery != Delivery::AtLeastOnce)
            {
                continue;
            }
            self.redeliveries.push(Redelivery {
                event: occurrence.event.clone(),
                bindings: reacting
                    .iter()
                    .map(|(name, _)| (*name).to_owned())
                    .collect(),
            });
        }
    }

    /// Delivers the queued event at `slot` a second time.
    fn redeliver(&mut self, slot: usize) {
        let Redelivery { event, bindings } = self.redeliveries.remove(slot);
        self.delivered += 1;
        let request = RedeliveryRequest {
            event: event.clone(),
            correlation: CorrelationId::new(format!("redelivery-{}", self.delivered))
                .unwrap_or_else(|error| panic!("a counter-shaped correlation is valid: {error}")),
        };
        if self.views.redeliver_event(request).is_ok() {
            for binding in &bindings {
                count(&mut self.injected.redeliveries, binding);
            }
        } else {
            count(&mut self.injected.refused, &event.to_string());
        }
    }

    /// Writes the answer into the reserved operation; returns what a creating command published.
    fn complete(&mut self, flight: InFlight<T::Pending>) -> Option<String> {
        match flight.pending {
            Pending::Call {
                pending,
                command,
                subject_key,
                timeout,
            } => {
                if timeout == Some(Timeout::Unanswered) {
                    // The client stops waiting and the target never executes the call.
                    drop(pending);
                    self.tick();
                    count(&mut self.injected.unanswered, &command);
                    self.operations[flight.index].subject_key = subject_key.unwrap_or_default();
                    return None;
                }
                let answer = self.target.complete(pending);
                let returned_at = self.tick();
                let Ok(result) = answer else {
                    self.operations[flight.index].subject_key = subject_key.unwrap_or_default();
                    return None;
                };
                self.schedule_redeliveries(&result);
                let retried = self.operations[flight.index].retry_of.is_some();
                let created = created_identity(self.ir, &command, &result);
                let outcome = result
                    .outcome
                    .as_ref()
                    .map_or_else(|| UNDECLARED.to_owned(), ToString::to_string);
                if timeout == Some(Timeout::Delayed) {
                    // Executed, and answered after the client stopped waiting: the call stays
                    // `Indeterminate`, and the instance its late answer named is still written,
                    // so a read that shows it is not a read of something nobody asked for.
                    count(&mut self.injected.delayed, &command);
                    count(&mut self.injected.reached, &outcome);
                    self.operations[flight.index].subject_key =
                        subject_key.or(created).unwrap_or_default();
                    return None;
                }
                if retried {
                    count(&mut self.injected.reached, &outcome);
                }
                if let Some(token) = &result.consistency {
                    self.tokens[flight.client] = Some(token.clone());
                }
                let written = result
                    .outcome
                    .as_ref()
                    .map_or_else(|| UNDECLARED.to_owned(), |taken| taken.outcome.to_string());
                let operation = &mut self.operations[flight.index];
                operation.subject_key = subject_key.or_else(|| created.clone()).unwrap_or_default();
                answered(operation, returned_at, &written);
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
    record_with(ir, target, views, workload, seed, FaultInjection::None)
        .map(|recorded| recorded.history)
}

/// [`record`], with every fault the specification declares injected, and what was injected.
///
/// Second deliveries go to `views`, the implementation seen through [`ConformanceTarget`]. See
/// the [module documentation](self) for what is injected and how it is written.
///
/// # Errors
///
/// As [`record`].
pub fn record_injected<T: Interleaved, V: ConformanceTarget>(
    ir: &EssIr,
    target: &T,
    views: &V,
    workload: &Workload,
    seed: u64,
) -> Result<Recorded, RecordError> {
    record_with(ir, target, views, workload, seed, FaultInjection::Declared)
}

/// The injections' sequence is the seed's own, moved away from the interleaving's.
const INJECTION_STREAM: u64 = 0x1f83_d9ab_fb41_bd6b;

/// One move the seed may pick.
#[derive(Debug, Clone, Copy)]
enum Move {
    /// A client invokes its next act, or receives the answer to the one in flight.
    Client(usize),
    /// A client sends its retry, or receives the retry's answer.
    Retry(usize),
    /// A queued event is delivered a second time.
    Redeliver(usize),
}

/// [`record`] or [`record_injected`], as `injection` says.
///
/// # Errors
///
/// As [`record`].
pub fn record_with<T: Interleaved, V: ConformanceTarget>(
    ir: &EssIr,
    target: &T,
    views: &V,
    workload: &Workload,
    seed: u64,
    injection: FaultInjection,
) -> Result<Recorded, RecordError> {
    crate::record::refuse_one_time(ir)?;
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
        inject: (injection == FaultInjection::Declared).then_some(Draw(seed ^ INJECTION_STREAM)),
        arranging: true,
        redeliveries: Vec::new(),
        delivered: 0,
        retries: (0..clients).map(|_| None).collect(),
        injected: Injected::default(),
    };
    for call in &workload.prefix {
        let flight = recording.invoke_call(0, call)?;
        let created = recording.complete(flight);
        recording.created.push(created);
    }
    recording.arranging = false;

    let mut draw = Draw(seed);
    let mut next: Vec<usize> = vec![0; workload.clients.len()];
    let mut flying: Vec<Option<InFlight<T::Pending>>> =
        workload.clients.iter().map(|_| None).collect();
    loop {
        // The clients' moves first, in client order, exactly as a recording without injection
        // enables them; then each client's retry; then each queued second delivery.
        let mut enabled: Vec<Move> = (0..workload.clients.len())
            .filter(|&client| {
                flying[client].is_some()
                    || recording.retries[client].is_none()
                        && next[client] < workload.clients[client].len()
            })
            .map(Move::Client)
            .collect();
        enabled.extend(
            (0..workload.clients.len())
                .filter(|&client| recording.retries[client].is_some())
                .map(Move::Retry),
        );
        enabled.extend((0..recording.redeliveries.len()).map(Move::Redeliver));
        if enabled.is_empty() {
            break;
        }
        match enabled[draw.below(enabled.len())] {
            Move::Client(client) => {
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
            Move::Retry(client) => match recording.retries[client].take() {
                Some(Retry::Waiting {
                    request,
                    command,
                    subject_key,
                    original,
                }) => {
                    let sent =
                        recording.invoke_retry(client, request, &command, subject_key, original);
                    recording.retries[client] = Some(Retry::Sent(sent));
                }
                Some(Retry::Sent(flight)) => {
                    recording.complete(flight);
                }
                None => {}
            },
            Move::Redeliver(slot) => recording.redeliver(slot),
        }
    }

    Ok(Recorded {
        history: History {
            format: HistoryFormat::EssHistory1,
            history_id: uuid(seed & 0xffff_ffff_ffff),
            spec_digest: SuiteProvenance::of(ir).spec_digest,
            seed,
            clients: clients as u64,
            operations: recording.operations,
        },
        injected: recording.injected,
    })
}
