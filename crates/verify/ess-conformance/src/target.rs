//! What an implementation offers so that a suite can be run against it.
//!
//! Design §7. A [`ConformanceTarget`] is the only thing a runner talks to, and the whole value of
//! the interface is what it refuses to contain. Every method below answers a question some ESS
//! construct asks; nothing below answers a question only a test would ask.
//!
//! # Every method traces to a declared ESS concept
//!
//! | method | the construct that obliges it | the step that reaches it |
//! |---|---|---|
//! | [`identity`](ConformanceTarget::identity) | none — it names the implementation under test, which a report needs (§30) and a specification never mentions | — |
//! | [`begin_scenario`](ConformanceTarget::begin_scenario) / [`end_scenario`](ConformanceTarget::end_scenario) | scenario isolation (§8): observations from one scenario may not satisfy another | every scenario |
//! | [`execute_command`](ConformanceTarget::execute_command) | `commands:`, their `outcomes:`, the `error:` a branch declares and what it `emits:` | [`ExecuteCommand`](crate::scenario::ScenarioStep::ExecuteCommand) |
//! | [`query_view`](ConformanceTarget::query_view) | `views:` and their `consistency:` | [`QueryView`](crate::scenario::ScenarioStep::QueryView), [`EventuallyView`](crate::scenario::ScenarioStep::EventuallyView) |
//! | [`observe_events`](ConformanceTarget::observe_events) | `events:` a component `publishes:`, observed away from the command that caused them | [`EventuallyEvent`](crate::scenario::ScenarioStep::EventuallyEvent) |
//! | [`configure_external_outcome`](ConformanceTarget::configure_external_outcome) | an outcome declared `external:` (§12) | [`ConfigureExternalOutcome`](crate::scenario::ScenarioStep::ConfigureExternalOutcome) |
//! | [`redeliver_event`](ConformanceTarget::redeliver_event) | a binding's `delivery: at_least_once` (§17) | [`RedeliverEvent`](crate::scenario::ScenarioStep::RedeliverEvent) |
//! | [`observe_invocations`](ConformanceTarget::observe_invocations) | a binding's `mapping:` (§16) | [`ExpectInvocation`](crate::scenario::ScenarioStep::ExpectInvocation) |
//! | [`mark_instant`](ConformanceTarget::mark_instant) | none — it names the instant a duration claim is measured from, which the suite may not invent | [`MarkInstant`](crate::scenario::ScenarioStep::MarkInstant) |
//! | [`observe_elapsed`](ConformanceTarget::observe_elapsed) | a timer, a wrap-up window, a TTL: a length of time the system's own behaviour turns on | [`ExpectNotBefore`](crate::scenario::ScenarioStep::ExpectNotBefore), [`ExpectWithin`](crate::scenario::ScenarioStep::ExpectWithin), [`ExpectQuiet`](crate::scenario::ScenarioStep::ExpectQuiet) |
//!
//! Seven of those eleven are §7's. The four that are not were added because a step in the closed
//! vocabulary could not otherwise be executed at all, and each is argued on its own method. None is
//! a shortcut past a semantic: `redeliver_event` is the only way to perform the claim the word
//! `at_least_once` makes, `observe_invocations` is the one §16 explicitly refuses to require, and
//! the last two are what a duration claim needs from a system that owns its own clock. Those three
//! are the methods with a default body that answers [`TargetError::Unsupported`], which is what
//! keeps a target written against the earlier interface compiling — and what stops it being read as
//! agreeing with a claim it never checked.
//!
//! # What is deliberately absent
//!
//! No clock, no seed, no id source — §7 says so and §37 says why: the runner owns every source of
//! variation and hands it to the target. A target that needs a correlation id or a deadline is
//! **given** one in the request.
//!
//! [`observe_elapsed`](ConformanceTarget::observe_elapsed) does not put a clock here, and the
//! distinction is worth stating because it looks like one. The runner still reads no clock of the
//! target's and sets none: it hands over a *length* the specification stated and receives a
//! *measurement* the target stands behind. Which clock produced the measurement — a wall clock
//! waited on, a logical clock advanced — is the one thing the interface deliberately does not ask,
//! because requiring either would exclude half the systems that have a timer worth testing.
//!
//! And no assertion. There is no `assert_the_binding_worked`, no `tell_me_whether_escalation_happened`
//! and no `reset_for_test` beyond the isolation §8 requires. A target reports what it observed; the
//! runner decides whether the specification is satisfied. The moment a method answers a *question the
//! suite is supposed to ask*, the suite has stopped checking the implementation and started asking it
//! for its own verdict.
//!
//! # Synchronous, and that is open decision D2 taken
//!
//! §27 writes the trait `async` and records that nothing in this workspace can drive one:
//! `aep_contract::testing::block_on` polls a future with a no-op waker and panics after a million
//! polls, which is right for a backend whose futures are ready immediately and wrong for a target
//! that really yields. The options were a synchronous runner, a new executor dependency, or pushing
//! the executor onto adopters. This takes the first, which §27 lists as the default: no new
//! dependency, and a transport target blocks internally — which §15 already asks of it, because
//! waiting is the target's job.

use std::collections::BTreeMap;
use std::fmt;

use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;

use crate::scenario::{
    BindingRef, CommandRef, Elapsed, ErrorRef, EventRef, InstantName, OutcomeRef, ScenarioId,
    ViewRef,
};

// ---- the trait -------------------------------------------------------------------------------

/// An implementation a conformance suite can be run against.
///
/// See the [module documentation](self) for why it has exactly these methods and no assertion among
/// them.
pub trait ConformanceTarget {
    /// Which implementation this is, for the report that becomes evidence (§30).
    fn identity(&self) -> Result<ImplementationIdentity, TargetError>;

    /// Opens an isolated execution context for one scenario (§8).
    ///
    /// *How* is the target's business — a fresh in-memory runtime, a transaction, a tenant, a
    /// temporary schema. The interface requires only that observations made in one scenario cannot
    /// satisfy another.
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError>;

    /// Invokes a command and reports what the specification says is observable of it (§9).
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError>;

    /// Reads a view, no fresher than the request demands (§14).
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError>;

    /// Reports the occurrences of an event this context has published (§13).
    ///
    /// The request carries a deadline. §15 puts the waiting here, in the only layer that knows what
    /// it is waiting for: return when the answer is available or when the deadline has passed, and
    /// never make the caller sleep.
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError>;

    /// Forces the next answer of an outcome the input cannot decide (§12).
    ///
    /// A test adapter control, not a runtime capability the specification claims: no predicate over
    /// a recipient and a template says whether a provider will accept the mail, so the answer is
    /// injected rather than constructed. It applies to the **next** invocation of that command and
    /// then lapses, because that is what the step says.
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError>;

    /// Delivers an event this context has already published to its bindings a second time (§17).
    ///
    /// The eighth method, and §7's seven cannot express it. `delivery: at_least_once` is precisely
    /// the claim that a transport may deliver one occurrence more than once and the handler
    /// survives it; a suite that never delivers twice never tests the only thing that word says.
    /// Re-running the upstream command instead would test something else — two commands are two
    /// occurrences, which every implementation handles by handling each once.
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError>;

    /// Reports the command invocations a binding made (§16).
    ///
    /// The ninth method, and the one §16 warns about: command tracing "may be additional evidence,
    /// but it should not become a requirement for every implementation". So it has a default body
    /// that answers [`TargetError::Unsupported`], and a target that cannot see its own bindings'
    /// invocations implements the other eight and reports `unsupported` for exactly one scenario
    /// (§28) — `<binding>/binding/mapping` — while still proving the flow, the delivery and the
    /// failure policy.
    ///
    /// The alternative was to leave the mapping unchecked, and a swapped mapping is the one clause
    /// of a binding that is silently wrong: `recipient: event.contact` and
    /// `recipient: event.alternate_contact` are the same shape, the same types and two different
    /// systems.
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported(
            format!(
                "the invocations `{}` made of `{}`",
                request.binding, request.command
            ),
            "this target does not expose the commands its bindings invoke",
        ))
    }

    /// Records the instant this scenario has reached, under a name a later claim measures from.
    ///
    /// The tenth method, and the first of the two an elapsed-time claim needs. It is not an
    /// assertion and cannot fail an implementation: marking is *arranging*, exactly as
    /// [`CaptureInstance`](crate::scenario::ScenarioStep::MarkInstant)'s sibling is, and a target
    /// that cannot mark reports the scenario `unsupported` rather than wrong.
    ///
    /// Its default body says so. A target implementing the nine that came before it keeps compiling
    /// and keeps meaning what it meant; what it cannot do is silently agree with a claim about time.
    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            format!("marking the instant `{}`", request.instant),
            "this target keeps no clock a scenario can measure from",
        ))
    }

    /// Lets a window close and reports what its clock read, and what it published on the way.
    ///
    /// The eleventh method, and the one the whole elapsed-time feature rests on. §7's rule holds
    /// here as everywhere: this reports an **observation**, never a verdict. The target says how
    /// much time passed and how many times the watched event appeared; whether that satisfies
    /// *within*, *not before* or *not within N of X* is the runner's decision, made against the
    /// suite.
    ///
    /// # The three clocks, and why the interface names none of them
    ///
    /// A duration claim can be checked three ways, and each is wrong as a *requirement*. Waiting on
    /// a wall clock is real and makes every suite slow and flaky. Advancing a logical clock is
    /// deterministic and most systems have none to advance. Reporting an elapsed measurement after
    /// the fact is honest and, on its own, cannot make the twenty seconds of DEV-741's experiment
    /// actually happen.
    ///
    /// So the interface asks for the third and permits the first two to produce it. `hold` says how
    /// much of the window must have closed *before you answer*, and how the target gets there is its
    /// own business: an end-to-end target sleeps, an in-memory target advances a clock it owns and
    /// returns immediately, and both answer the same question truthfully. What no target may do is
    /// stay silent and be read as agreeing — the default body below is
    /// [`TargetError::Unsupported`], the runner records `unsupported`, and §28 makes an
    /// `unsupported` scenario fail conformance. A window nobody held is never a window that passed.
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        Err(TargetError::unsupported(
            format!(
                "letting {} pass since the instant `{}`",
                request.hold, request.instant
            ),
            "this target cannot make time pass or say how much has",
        ))
    }

    /// Closes the scenario's execution context (§8).
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError>;
}

// ---- identity and context --------------------------------------------------------------------

/// Which implementation was under test.
///
/// A conformance verdict is a claim about one implementation against one specification, and a report
/// that cannot name the implementation attests nothing. The specification's half of that identity is
/// [`SuiteProvenance`](crate::scenario::SuiteProvenance), which the suite already carries.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ImplementationIdentity {
    /// What the implementation is called.
    pub name: String,
    /// Which build of it answered.
    pub version: String,
}

impl ImplementationIdentity {
    /// Names an implementation and the build that ran.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

impl fmt::Display for ImplementationIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

/// The isolated execution context one scenario runs in (§8).
///
/// Both fields come from the runner. The correlation id is minted by the runner's id source rather
/// than by the target, which is §37's rule and the reason two runs of one suite can be compared: an
/// id the target invented would differ between runs for no semantic reason.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ScenarioContext {
    /// Which scenario.
    pub scenario: ScenarioId,
    /// The activity every request in this scenario belongs to.
    pub correlation: CorrelationId,
}

impl ScenarioContext {
    /// A context for one scenario.
    pub fn new(scenario: ScenarioId, correlation: CorrelationId) -> Self {
        Self {
            scenario,
            correlation,
        }
    }
}

/// The instant after which a bounded observation may give up.
///
/// Minted by the runner from its clock and its configured budget, and handed to the target so that
/// the *waiting* happens where §15 puts it. It is not a duration to sleep for: a target that can
/// answer immediately answers immediately, and one that cannot returns what it has when this instant
/// has passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Deadline(Timestamp);

impl Deadline {
    /// A deadline at `at`.
    pub fn at(at: Timestamp) -> Self {
        Self(at)
    }

    /// The instant it expires.
    pub fn instant(self) -> Timestamp {
        self.0
    }

    /// `true` when `now` is at or past it.
    pub fn has_passed(self, now: Timestamp) -> bool {
        now >= self.0
    }
}

impl fmt::Display for Deadline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---- commands --------------------------------------------------------------------------------

/// A command to invoke, as the specification names it (§9).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SemanticCommandRequest {
    /// Which command.
    pub command: CommandRef,
    /// As whom, where the specification grants commands to actors.
    pub actor: Option<crate::scenario::ActorRef>,
    /// The input, by declared field name, with every reference already resolved by the runner.
    pub input: BTreeMap<String, Node>,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
}

/// What a command invocation is observable as (§9).
///
/// Not implementation internals: a branch, the declared error it carries, a consistency token and
/// the events the command itself published.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SemanticCommandResult {
    /// The declared branch the command took.
    ///
    /// # `None` is a finding about the specification, not an escape hatch
    ///
    /// §19 makes the *absence* of a transition semantics: `CancelInvoice` must not move a `Paid`
    /// invoice. It then says "the exact rejection mechanism must come from the declared
    /// command/error semantics", and the model can now supply one: a command declares a
    /// [`wrong_state:`](ess_domain::command::OutcomeCondition::WrongState) branch naming the error
    /// it reports when its subject is somewhere none of its moves start from. Both examples do, so
    /// a target answers that branch rather than nothing, and the illegal-move scenarios require it
    /// by name.
    ///
    /// `None` remains, for the refusals **no** declared branch covers — a command issued against an
    /// instance the target has never seen, which is a different question the specification does not
    /// ask. A target that refuses for a reason its specification does not model answers `None`, and
    /// the scenarios assert only what the specification does say. `None` is not a way past an
    /// assertion — [`ExpectOutcome`](crate::scenario::ScenarioStep::ExpectOutcome) fails against it,
    /// naming that no declared outcome was reached — and it is not a [`TargetError`], because a
    /// lifecycle refusal is correct behaviour rather than an adapter failure.
    pub outcome: Option<OutcomeRef>,
    /// The declared error the branch carries, where it declares one.
    pub error: Option<DeclaredErrorValue>,
    /// The token a later read may demand a view no older than (§14).
    pub consistency: Option<ConsistencyToken>,
    /// The events this invocation itself published.
    ///
    /// What every assertion after an [`ExecuteCommand`](crate::scenario::ScenarioStep::ExecuteCommand) reads.
    /// Consequences that reach the system by a binding are not here: they are observed through
    /// [`observe_events`](ConformanceTarget::observe_events), because they are the other
    /// component's, and requiring them of the caller's result would be a transport assumption (§41).
    pub direct_events: Vec<ObservedEvent>,
}

impl SemanticCommandResult {
    /// A command that took `outcome` and published nothing.
    pub fn took(outcome: OutcomeRef) -> Self {
        Self {
            outcome: Some(outcome),
            error: None,
            consistency: None,
            direct_events: Vec::new(),
        }
    }

    /// A command that reached no declared outcome — see [`SemanticCommandResult::outcome`].
    pub fn undeclared() -> Self {
        Self {
            outcome: None,
            error: None,
            consistency: None,
            direct_events: Vec::new(),
        }
    }

    /// The same result, carrying a declared error.
    #[must_use]
    pub fn with_error(mut self, error: DeclaredErrorValue) -> Self {
        self.error = Some(error);
        self
    }

    /// The same result, carrying a consistency token.
    #[must_use]
    pub fn with_consistency(mut self, token: ConsistencyToken) -> Self {
        self.consistency = Some(token);
        self
    }

    /// The same result, carrying an event the invocation published.
    #[must_use]
    pub fn emitting(mut self, event: ObservedEvent) -> Self {
        self.direct_events.push(event);
        self
    }
}

/// A declared error, and what it carried.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DeclaredErrorValue {
    /// Which declared error.
    pub error: ErrorRef,
    /// Its payload, by declared field name.
    pub fields: BTreeMap<String, Node>,
}

impl DeclaredErrorValue {
    /// A declared error with no payload.
    pub fn new(error: ErrorRef) -> Self {
        Self {
            error,
            fields: BTreeMap::new(),
        }
    }

    /// The same error, carrying one field.
    #[must_use]
    pub fn with(mut self, field: impl Into<String>, value: Node) -> Self {
        self.fields.insert(field.into(), value);
        self
    }
}

// ---- events ----------------------------------------------------------------------------------

/// One occurrence of a declared event (§13).
///
/// Where the target read it from — an in-memory sink, a test consumer, an event table, a callback —
/// is the adapter's business. Transport headers are not modelled here, because the specification
/// does not model them (§41).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ObservedEvent {
    /// Which event.
    pub event: EventRef,
    /// Its payload, by declared field name.
    pub payload: BTreeMap<String, Node>,
    /// The activity it belongs to, where the target propagates one.
    pub correlation: Option<CorrelationId>,
    /// Where it sits in the target's own publication order, where the target has one.
    pub sequence: Option<u64>,
}

impl ObservedEvent {
    /// An occurrence of `event` with no payload.
    pub fn new(event: EventRef) -> Self {
        Self {
            event,
            payload: BTreeMap::new(),
            correlation: None,
            sequence: None,
        }
    }

    /// The same occurrence, carrying one payload field.
    #[must_use]
    pub fn with(mut self, field: impl Into<String>, value: Node) -> Self {
        self.payload.insert(field.into(), value);
        self
    }

    /// The same occurrence, attributed to an activity.
    #[must_use]
    pub fn in_activity(mut self, correlation: CorrelationId) -> Self {
        self.correlation = Some(correlation);
        self
    }

    /// The same occurrence, at a position in the target's publication order.
    #[must_use]
    pub fn at(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }
}

/// A request to observe the occurrences of one event (§13).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EventObservationRequest {
    /// Which event the caller is waiting for.
    ///
    /// Named, rather than "everything you have", because §15 makes the target responsible for
    /// waiting and a target cannot wait for something it has not been told about.
    pub event: EventRef,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
    /// When the target may stop waiting.
    pub deadline: Deadline,
}

// ---- elapsed time ------------------------------------------------------------------------------

/// Naming the instant the scenario has reached, so a later claim can be measured from it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InstantMark {
    /// What later claims call it.
    pub instant: InstantName,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
}

/// A request for what a target's clock reads, and for what happened while it got there.
///
/// # Why `hold` and the reading are one request
///
/// Because splitting them would be a race the runner could not see. "Let twenty seconds pass" and
/// "tell me how long has passed" as two calls admits a target that returns immediately from the
/// first and truthfully answers `20000` from the second, having done nothing in between; the point
/// of the hold is what runs *during* it. One request means the target that answers is the target
/// that waited, and the answer is about the wait.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ElapsedObservationRequest {
    /// The instant the window opens at, named by an earlier [`InstantMark`].
    pub instant: InstantName,
    /// How much of the window to let close before answering.
    ///
    /// Zero asks for the reading as it stands, which is what a *within* claim wants: it measures
    /// what already happened rather than making anything happen.
    pub hold: Elapsed,
    /// The event whose publication inside the window the runner is asking about, where there is one.
    ///
    /// A question, not an assertion — the target says what it saw and the runner decides what that
    /// means, which is the line §7 draws through this whole interface. It is here rather than on
    /// [`observe_events`](ConformanceTarget::observe_events) because that method answers *what has
    /// this activity published*, with no notion of when, and the whole claim here is *when*.
    pub watching: Option<EventRef>,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
}

/// What a target's clock read, and what it published while the window was open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ElapsedObservation {
    /// How much has passed since the marked instant, in milliseconds.
    ///
    /// The target's own measurement on its own clock, of whichever kind it keeps: a wall clock it
    /// waited on, or one it advanced. Milliseconds because a measurement rounded towards the claim
    /// is how a window that did not close gets reported as one that did.
    pub elapsed_ms: u64,
    /// How many times the watched event was published inside the window.
    ///
    /// Zero when nothing was watched, which is not the same fact and is never read as one: the
    /// runner only asks the question when a claim needs it.
    pub published: usize,
}

/// A request to deliver an already-published event to its bindings again (§17).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RedeliveryRequest {
    /// Which event to deliver again.
    ///
    /// It does not say which binding: an event reaches everything that reacts to it, and naming one
    /// would be a delivery the transport does not have.
    pub event: EventRef,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
}

// ---- views -----------------------------------------------------------------------------------

/// A request to read a view (§14).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SemanticViewRequest {
    /// Which view.
    pub view: ViewRef,
    /// The value bound to each parameter the view declares, resolved.
    ///
    /// Empty for a view with no `params:`, which is most of them. Complete for one that has them:
    /// `ess-domain` checks that the filter reads every declared parameter, and the runner refuses
    /// to send a request with one it could not resolve — a query with a made-up parameter reads a
    /// different set of rows, and every assertion after it is about the wrong thing.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, Node>,
    /// How fresh the read has to be.
    ///
    /// `ess_primitives::consistency::QueryConsistency`, not a second pair of consistency types: §14
    /// records that both already ship, and a parallel pair would buy nothing but a translation table
    /// for the two to drift across.
    pub consistency: QueryConsistency,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
    /// When the target may stop waiting for the freshness it was asked for.
    pub deadline: Deadline,
}

/// What a view holds (§14).
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct SemanticViewResult {
    /// The rows, each a value per projected field name.
    pub rows: Vec<ViewRow>,
}

impl SemanticViewResult {
    /// A view holding these rows.
    pub fn of(rows: impl IntoIterator<Item = ViewRow>) -> Self {
        Self {
            rows: rows.into_iter().collect(),
        }
    }
}

/// One row of a view: a value per projected field name.
pub type ViewRow = BTreeMap<String, Node>;

// ---- invocations -----------------------------------------------------------------------------

/// A request for the invocations one binding made (§16).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InvocationObservationRequest {
    /// Whose invocations.
    pub binding: BindingRef,
    /// The command it invokes.
    pub command: CommandRef,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
    /// When the target may stop waiting.
    pub deadline: Deadline,
}

/// One command a binding invoked, and what it passed (§16).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ObservedInvocation {
    /// Which binding invoked it.
    pub binding: BindingRef,
    /// Which command.
    pub command: CommandRef,
    /// What each input received, by declared field name.
    pub input: BTreeMap<String, Node>,
}

impl ObservedInvocation {
    /// An invocation of `command` by `binding` with no input.
    pub fn new(binding: BindingRef, command: CommandRef) -> Self {
        Self {
            binding,
            command,
            input: BTreeMap::new(),
        }
    }

    /// The same invocation, carrying one input field.
    #[must_use]
    pub fn with(mut self, field: impl Into<String>, value: Node) -> Self {
        self.input.insert(field.into(), value);
        self
    }
}

// ---- external outcomes -----------------------------------------------------------------------

/// A request to force an outcome the input cannot decide (§12).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExternalOutcomeControl {
    /// The outcome the adapter must produce next. The command is part of the reference.
    pub force: OutcomeRef,
    /// The scenario this belongs to.
    pub correlation: CorrelationId,
}

// ---- failure ---------------------------------------------------------------------------------

/// Why a target could not answer.
///
/// Two cases, because §28 gives them two different verdicts and collapsing them loses the one that
/// matters: an [`Unsupported`](Self::Unsupported) observation is a permanent property of the target
/// and makes conformance **fail** rather than quietly skip, while
/// [`Unavailable`](Self::Unavailable) is the runner or the adapter failing to execute a check at all
/// and is reported as `error`.
///
/// Neither is a declared domain rejection. §9 is explicit: an implementation that surfaces
/// `CreateInvoice(amount = -1)` as an untyped infrastructure error rather than as
/// `rejected` + `InvalidAmount` is non-conformant, and returning a `TargetError` for it is exactly
/// that defect. A refusal the specification declares is a
/// [`SemanticCommandResult`], never an error here.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetError {
    /// The target cannot expose this observation at all.
    Unsupported {
        /// What was asked for.
        observation: String,
        /// Why the target cannot answer it.
        why: String,
    },
    /// The target could not carry out the request this time.
    Unavailable {
        /// What was attempted.
        operation: String,
        /// What went wrong, in the target's own vocabulary.
        detail: String,
    },
}

impl TargetError {
    /// An observation this target cannot expose.
    pub fn unsupported(observation: impl Into<String>, why: impl Into<String>) -> Self {
        Self::Unsupported {
            observation: observation.into(),
            why: why.into(),
        }
    }

    /// A request this target could not carry out.
    pub fn unavailable(operation: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Unavailable {
            operation: operation.into(),
            detail: detail.into(),
        }
    }

    /// `true` for the case §28 calls `unsupported`.
    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported { .. })
    }
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { observation, why } => {
                write!(f, "cannot expose {observation}: {why}")
            }
            Self::Unavailable { operation, detail } => write!(f, "{operation} failed: {detail}"),
        }
    }
}

impl std::error::Error for TargetError {}
