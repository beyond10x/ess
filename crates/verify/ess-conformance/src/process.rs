//! A conformance target in another process, spoken to over `ess-conformance-target/1`.
//!
//! `docs/design/conformance-target-process.md` is the binding design. [`ProcessTarget`] asks every
//! question of [`ConformanceTarget`] as one JSON object per line and reads one JSON answer per line,
//! in lockstep, over a [`Peer`]; [`serve`] is the other end, answering such lines from any
//! `ConformanceTarget`. The child process itself is the caller's: this crate starts nothing, and a
//! `Peer` is whatever carries the lines.
//!
//! # What is read, and what is never shown
//!
//! Requests carry every number as its exact value ([`Number::exact_text`](ess_primitives::facts::Number::exact_text)),
//! and answers are read through an exact reader ([`Node::from_json_exact`](ess_primitives::node::Node::from_json_exact)'s
//! number door) that is closed outside value maps: an unknown, missing, repeated or mistyped member
//! is a fault. A fault is described by its kind, the request's `id`, its operation and a byte
//! offset only ([`ProtocolFault`]): nothing this module returns, displays or debug-prints holds a
//! byte of an answer line that broke the protocol.
//!
//! The strings a target supplies that are shown as text, the handshake's implementation `name` and
//! `version` and every string of a `target_error`, are bounded ([`MAX_IDENTITY`], refused past it;
//! [`MAX_TARGET_TEXT`], cut past it) and have every control character (Cc), line or paragraph
//! separator and bidirectional override or isolate written as its escape, `\u{1b}` style, before
//! they are kept. An in-process target's strings are not touched.
//!
//! # Lifecycle
//!
//! A fault ends the current scenario: the peer is killed, every further call of that scenario answers
//! the fault without contacting anything (`end_scenario` answers success), and the next scenario's
//! first call respawns the peer and handshakes again, requiring the same implementation identity. A
//! restart that fails ends every remaining scenario with that failure, and nothing is respawned
//! again.

mod exact;
mod serve;
mod wire;

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;

use ess_primitives::node::Node;

use crate::scenario::ActorRef;
use crate::target::{
    AbsentInputRequest, AwaitAnswer, AwaitRequest, ConformanceTarget, ElapsedObservation,
    ElapsedObservationRequest, EntitySetupRequest, EventDeliveryRequest, EventObservationRequest,
    ExternalOutcomeControl, ImplementationIdentity, InstantMark, InvocationObservationRequest,
    NowHold, ObservedEvent, ObservedInvocation, OrderedScan, OrderedScanRequest, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};

pub use serve::{serve, ServeError};

/// The protocol's format name and version, which a handshake's `speaks` names.
pub const TARGET_FORMAT: &str = "ess-conformance-target/1";

/// The versions this build reads, oldest first, as a handshake request's `reads` lists them.
pub const READS: &[&str] = &[TARGET_FORMAT];

/// The longest answer line read, in bytes before its `\n`: 16 MiB.
pub const MAX_LINE: usize = 16_777_216;

/// The longest implementation `name` or `version` a handshake may name, in bytes; a longer one is
/// [`HandshakeRefusal::IdentityTooLong`].
pub const MAX_IDENTITY: usize = 256;

/// The longest string of a `target_error` answer kept whole, in bytes; a longer one is cut at a
/// character boundary and ends ` … (<n> bytes more)`.
pub const MAX_TARGET_TEXT: usize = 4096;

// ---- operations ----------------------------------------------------------------------------------

macro_rules! operations {
    ($($(#[$doc:meta])* $variant:ident = $name:literal,)*) => {
        /// One operation of the protocol: the handshake, or a method of [`ConformanceTarget`] in its
        /// own `snake_case`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Op {
            $($(#[$doc])* $variant,)*
        }

        impl Op {
            /// Every operation, in the design page's catalogue order.
            pub const ALL: &'static [Op] = &[$(Op::$variant,)*];

            /// The `op` member that names it.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Op::$variant => $name,)*
                }
            }

            /// The operation an `op` member names, if this build knows it.
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($name => Some(Op::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

operations! {
    /// `identity`, answered once per process before anything else.
    Handshake = "handshake",
    /// [`ConformanceTarget::fixture_values`].
    FixtureValues = "fixture_values",
    /// [`ConformanceTarget::open_periodic`].
    OpenPeriodic = "open_periodic",
    /// [`ConformanceTarget::observe_periodic`].
    ObservePeriodic = "observe_periodic",
    /// [`ConformanceTarget::close_periodic`].
    ClosePeriodic = "close_periodic",
    /// [`ConformanceTarget::observe_clock_reading`].
    ObserveClockReading = "observe_clock_reading",
    /// [`ConformanceTarget::establish_entity`].
    EstablishEntity = "establish_entity",
    /// [`ConformanceTarget::begin_scenario`].
    BeginScenario = "begin_scenario",
    /// [`ConformanceTarget::execute_command`].
    ExecuteCommand = "execute_command",
    /// [`ConformanceTarget::execute_command_without_input`].
    ExecuteCommandWithoutInput = "execute_command_without_input",
    /// [`ConformanceTarget::query_view`].
    QueryView = "query_view",
    /// [`ConformanceTarget::query_view_as`].
    QueryViewAs = "query_view_as",
    /// [`ConformanceTarget::query_view_anonymous`].
    QueryViewAnonymous = "query_view_anonymous",
    /// [`ConformanceTarget::observe_events`].
    ObserveEvents = "observe_events",
    /// [`ConformanceTarget::configure_external_outcome`].
    ConfigureExternalOutcome = "configure_external_outcome",
    /// [`ConformanceTarget::configure_external_outcome_repeatedly`].
    ConfigureExternalOutcomeRepeatedly = "configure_external_outcome_repeatedly",
    /// [`ConformanceTarget::redeliver_event`].
    RedeliverEvent = "redeliver_event",
    /// [`ConformanceTarget::deliver_event`].
    DeliverEvent = "deliver_event",
    /// [`ConformanceTarget::observe_invocations`].
    ObserveInvocations = "observe_invocations",
    /// [`ConformanceTarget::mark_instant`].
    MarkInstant = "mark_instant",
    /// [`ConformanceTarget::hold_now`].
    HoldNow = "hold_now",
    /// [`ConformanceTarget::observe_elapsed`].
    ObserveElapsed = "observe_elapsed",
    /// [`ConformanceTarget::scan_view`].
    ScanView = "scan_view",
    /// [`ConformanceTarget::scan_view_as`].
    ScanViewAs = "scan_view_as",
    /// [`ConformanceTarget::scan_view_anonymous`].
    ScanViewAnonymous = "scan_view_anonymous",
    /// [`ConformanceTarget::await_outcome`].
    AwaitOutcome = "await_outcome",
    /// [`ConformanceTarget::end_scenario`].
    EndScenario = "end_scenario",
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

// ---- the transport -------------------------------------------------------------------------------

/// What carries the lines: a child process in `ess`, memory in a test.
///
/// Lines are passed without their `\n`. Every method is called from one thread, one request at a
/// time.
pub trait Peer {
    /// Writes one request line, appending the `\n`.
    fn send(&mut self, line: &str) -> Result<(), PeerFault>;

    /// Reads the next complete line, without its `\n`, waiting at most `limit_ms` wall-clock
    /// milliseconds. A line longer than [`MAX_LINE`] is [`PeerFault::LineTooLong`].
    fn receive(&mut self, limit_ms: u64) -> Result<Vec<u8>, PeerFault>;

    /// Ends the peer and everything it started, and reaps it. Called after every fault and before a
    /// refused handshake returns; calling it on an ended peer does nothing.
    fn kill(&mut self);

    /// Starts the peer afresh after [`kill`](Self::kill), for the next scenario.
    fn respawn(&mut self) -> Result<(), PeerFault>;
}

/// How a [`Peer`] failed to carry a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerFault {
    /// The process exited while a request was outstanding.
    Exited(Exit),
    /// Its standard output reached end of file while a request was outstanding.
    ClosedStdout,
    /// No complete line within the limit.
    Timeout,
    /// More than [`MAX_LINE`] bytes with no `\n`.
    LineTooLong,
    /// The process could not be started again.
    NotStarted(std::io::ErrorKind),
}

/// How a process ended, as far as the operating system says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// It exited with this status.
    Code(i32),
    /// A signal of this number ended it.
    Signal(i32),
    /// Its status could not be read.
    Unknown,
}

impl fmt::Display for Exit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(code) => write!(f, "status {code}"),
            Self::Signal(signal) => write!(f, "signal {signal}"),
            Self::Unknown => f.write_str("status unknown"),
        }
    }
}

// ---- faults --------------------------------------------------------------------------------------

/// A broken exchange, by kind, request `id`, operation and byte offset only.
///
/// Its [`Display`](fmt::Display) is the `detail` the design page's fault table gives, and it becomes
/// [`TargetError::Unavailable`] with the operation's name. It holds no byte of the answer line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolFault {
    kind: FaultKind,
    id: u64,
    op: Op,
    limit_ms: u64,
}

/// The kinds of [`ProtocolFault`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    /// The process exited while the request was outstanding.
    Exited(Exit),
    /// Its standard output reached end of file while the request was outstanding.
    ClosedStdout,
    /// No complete line within the request's `limit_ms`.
    Timeout,
    /// More than [`MAX_LINE`] bytes with no `\n`.
    LineTooLong,
    /// The line is not UTF-8 from this byte on.
    NotUtf8 {
        /// The first byte that is not.
        at: usize,
    },
    /// The line is not one JSON value, or is blank.
    NotJson {
        /// Where reading stopped.
        at: usize,
    },
    /// Valid JSON that breaks the envelope or the operation's answer shape.
    NotAnAnswer {
        /// The value that does not match.
        at: usize,
    },
    /// The answer names another request's `id`.
    WrongId {
        /// The `id` it names.
        named: u64,
    },
    /// A number the exact reader refuses: past binary64, over 512 characters, or an exponent past
    /// ±512.
    NumberOutOfRange {
        /// Where the number starts.
        at: usize,
    },
    /// The process could not be started again for a restart.
    NotStarted(std::io::ErrorKind),
}

impl FaultKind {
    /// The kind's name, as the design page's fault table spells it.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Exited(_) => "exited",
            Self::ClosedStdout => "closed-stdout",
            Self::Timeout => "timeout",
            Self::LineTooLong => "line-too-long",
            Self::NotUtf8 { .. } => "not-utf8",
            Self::NotJson { .. } => "not-json",
            Self::NotAnAnswer { .. } => "not-an-answer",
            Self::WrongId { .. } => "wrong-id",
            Self::NumberOutOfRange { .. } => "number-out-of-range",
            Self::NotStarted(_) => "not-started",
        }
    }
}

impl From<PeerFault> for FaultKind {
    fn from(fault: PeerFault) -> Self {
        match fault {
            PeerFault::Exited(exit) => Self::Exited(exit),
            PeerFault::ClosedStdout => Self::ClosedStdout,
            PeerFault::Timeout => Self::Timeout,
            PeerFault::LineTooLong => Self::LineTooLong,
            PeerFault::NotStarted(kind) => Self::NotStarted(kind),
        }
    }
}

impl ProtocolFault {
    /// What broke.
    pub fn kind(&self) -> FaultKind {
        self.kind
    }

    /// The outstanding request's `id`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// The outstanding request's operation.
    pub fn op(&self) -> Op {
        self.op
    }

    /// The fault as the target error the runner records: [`TargetError::Unavailable`] naming the
    /// operation, with this fault's text as its detail.
    pub fn to_target_error(&self) -> TargetError {
        TargetError::unavailable(self.op.name(), self.to_string())
    }
}

impl fmt::Display for ProtocolFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (id, op) = (self.id, self.op);
        match self.kind {
            FaultKind::Exited(exit) => {
                write!(
                    f,
                    "target process exited ({exit}) before answering {id} ({op})"
                )
            }
            FaultKind::ClosedStdout => write!(
                f,
                "target process closed its standard output before answering {id} ({op})"
            ),
            FaultKind::Timeout => {
                write!(f, "no answer to {id} ({op}) within {} ms", self.limit_ms)
            }
            FaultKind::LineTooLong => {
                write!(f, "answer to {id} ({op}) exceeds {MAX_LINE} bytes")
            }
            FaultKind::NotUtf8 { at } => {
                write!(f, "answer to {id} ({op}) is not UTF-8 at byte {at}")
            }
            FaultKind::NotJson { at } => {
                write!(f, "answer to {id} ({op}) is not JSON at byte {at}")
            }
            FaultKind::NotAnAnswer { at } => write!(
                f,
                "answer to {id} ({op}) does not match its operation at byte {at}"
            ),
            FaultKind::WrongId { named } => write!(
                f,
                "answer names id {named} while {id} ({op}) is outstanding"
            ),
            FaultKind::NumberOutOfRange { at } => write!(
                f,
                "answer to {id} ({op}) holds a number outside the domain at byte {at}"
            ),
            FaultKind::NotStarted(kind) => write!(
                f,
                "target process could not be started again ({kind:?}) before {id} ({op})"
            ),
        }
    }
}

impl std::error::Error for ProtocolFault {}

/// Why a handshake did not admit the target.
///
/// For the first handshake, `ess` refuses the run (exit 2, no report). For a restart's, it is the
/// failure every remaining scenario ends with. It holds no value from the answer: a `speaks` that is
/// not a version of this format is described, not quoted, and a `target_error` by its kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeRefusal {
    /// The exchange itself broke.
    Fault(ProtocolFault),
    /// The target speaks a version this build does not read.
    Version {
        /// The version of [`TARGET_FORMAT`]'s family it names, when `speaks` names one at all.
        speaks: Option<u32>,
    },
    /// The target answered the handshake with a `target_error` of this kind.
    Refused {
        /// `unsupported`, `unavailable` or `not_granted`.
        kind: &'static str,
    },
    /// A restarted target names another implementation than the first did.
    OtherImplementation,
    /// The handshake's implementation `name` or `version` is longer than [`MAX_IDENTITY`] bytes.
    IdentityTooLong {
        /// `name` or `version`.
        member: &'static str,
        /// Its length in bytes.
        bytes: usize,
    },
}

impl HandshakeRefusal {
    /// The refusal's kind: a fault kind's name, `version`, `target-error`, `other-implementation`
    /// or `identity-too-long`.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Fault(fault) => fault.kind.name(),
            Self::Version { .. } => "version",
            Self::Refused { .. } => "target-error",
            Self::OtherImplementation => "other-implementation",
            Self::IdentityTooLong { .. } => "identity-too-long",
        }
    }
}

impl fmt::Display for HandshakeRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reads = READS.join(", ");
        match self {
            Self::Fault(fault) => write!(
                f,
                "the target's handshake failed ({}): {fault}",
                fault.kind.name()
            ),
            Self::Version {
                speaks: Some(version),
            } => write!(
                f,
                "target speaks ess-conformance-target/{version}; this ess reads {reads}"
            ),
            Self::Version { speaks: None } => write!(
                f,
                "target speaks no version of ess-conformance-target; this ess reads {reads}"
            ),
            Self::Refused { kind } => write!(
                f,
                "the target answered the handshake with target_error `{kind}`"
            ),
            Self::OtherImplementation => f.write_str(
                "the restarted target names another implementation than the run's; one report \
                 names one implementation",
            ),
            Self::IdentityTooLong { member, bytes } => write!(
                f,
                "the target's handshake names an implementation {member} of {bytes} bytes; at \
                 most {MAX_IDENTITY} are read"
            ),
        }
    }
}

impl std::error::Error for HandshakeRefusal {}

/// The suite a handshake names, from its provenance and its admitted bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HandshakeSuite {
    /// The system the suite checks.
    pub system: String,
    /// The version of its specification.
    pub specification_version: String,
    /// The suite's format.
    pub suite_version: crate::scenario::SuiteFormat,
    /// The digest of the model it was synthesized from.
    pub spec_digest: ess_primitives::evidence::SpecDigest,
    /// The component it is scoped to, where it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// [`AdmittedSuite::digest`](crate::AdmittedSuite::digest) of its bytes.
    pub suite_digest: String,
}

impl HandshakeSuite {
    /// The handshake's account of an admitted suite.
    pub fn of(admitted: &crate::AdmittedSuite) -> Self {
        let provenance = &admitted.suite().provenance;
        Self {
            system: provenance.system.clone(),
            specification_version: provenance.specification_version.clone(),
            suite_version: provenance.suite_version,
            spec_digest: provenance.spec_digest.clone(),
            component: provenance.component.clone(),
            suite_digest: admitted.digest().to_owned(),
        }
    }
}

// ---- the process target --------------------------------------------------------------------------

/// A [`ConformanceTarget`] answered over a [`Peer`] in `ess-conformance-target/1`.
///
/// Made by [`connect`](Self::connect), which handshakes first. See the
/// [module documentation](self) for the lifecycle.
pub struct ProcessTarget<P: Peer> {
    timeout_ms: u64,
    suite: HandshakeSuite,
    identity: ImplementationIdentity,
    link: RefCell<Link<P>>,
}

struct Link<P> {
    peer: P,
    next_id: u64,
    live: bool,
    scenario: Option<ScenarioContext>,
    fault: Option<(Option<ScenarioContext>, TargetError)>,
    dead: Option<TargetError>,
}

/// How one exchange failed: the protocol broke, or the target answered `target_error`.
enum Failure {
    Fault(ProtocolFault),
    Target(TargetError),
}

impl<P: Peer> ProcessTarget<P> {
    /// Handshakes over `peer`, a freshly started target, and makes the target of the run.
    ///
    /// `timeout_ms` is the wall-clock wait for one answer, to which an operation adds the time it
    /// is asked to let pass (the design page's "Time limits"). On a refusal the peer has been
    /// killed.
    pub fn connect(
        peer: P,
        suite: HandshakeSuite,
        timeout_ms: u64,
    ) -> Result<Self, HandshakeRefusal> {
        let mut link = Link {
            peer,
            next_id: 0,
            live: true,
            scenario: None,
            fault: None,
            dead: None,
        };
        match handshake(&mut link, timeout_ms, &suite) {
            Ok(identity) => Ok(Self {
                timeout_ms,
                suite,
                identity,
                link: RefCell::new(link),
            }),
            Err(refusal) => {
                link.peer.kill();
                Err(refusal)
            }
        }
    }

    /// The peer back, for the caller to end it at the end of the run.
    pub fn into_peer(self) -> P {
        self.link.into_inner().peer
    }

    /// The implementation the first handshake named.
    pub fn implementation(&self) -> &ImplementationIdentity {
        &self.identity
    }

    /// One operation: the lifecycle rules, then one exchange.
    fn call<T>(
        &self,
        op: Op,
        extra_ms: u64,
        opener: Option<&ScenarioContext>,
        request: &impl serde::Serialize,
        decode: impl FnOnce(&exact::Value) -> Result<T, exact::Bad>,
    ) -> Result<T, TargetError> {
        let mut link = self.link.borrow_mut();
        if let Some(dead) = &link.dead {
            return Err(dead.clone());
        }
        // A scenario's first call ends the fault of the scenario before it.
        let next_scenario = match (opener, &link.fault) {
            (Some(context), Some((faulted_in, _))) => faulted_in.as_ref() != Some(context),
            _ => false,
        };
        if next_scenario {
            link.fault = None;
        }
        if let Some((_, error)) = &link.fault {
            return Err(error.clone());
        }
        if let Some(context) = opener {
            link.scenario = Some(context.clone());
        }
        if !link.live {
            self.restart(&mut link)?;
        }
        let limit_ms = self.timeout_ms.saturating_add(extra_ms);
        match round_trip(&mut link, op, limit_ms, request, decode) {
            Ok(answer) => Ok(answer),
            Err(Failure::Target(error)) => Err(error),
            Err(Failure::Fault(fault)) => {
                let error = fault.to_target_error();
                link.peer.kill();
                link.live = false;
                link.fault = Some((link.scenario.clone(), error.clone()));
                Err(error)
            }
        }
    }

    /// A fresh process and a fresh handshake, naming the run's implementation.
    fn restart(&self, link: &mut Link<P>) -> Result<(), TargetError> {
        let attempt = if let Err(fault) = link.peer.respawn() {
            Err(HandshakeRefusal::Fault(ProtocolFault {
                kind: fault.into(),
                id: 0,
                op: Op::Handshake,
                limit_ms: self.timeout_ms,
            }))
        } else {
            link.next_id = 0;
            handshake(link, self.timeout_ms, &self.suite).and_then(|identity| {
                if identity == self.identity {
                    Ok(())
                } else {
                    Err(HandshakeRefusal::OtherImplementation)
                }
            })
        };
        match attempt {
            Ok(()) => {
                link.live = true;
                Ok(())
            }
            Err(refusal) => {
                link.peer.kill();
                link.live = false;
                let error = TargetError::unavailable(Op::Handshake.name(), refusal.to_string());
                link.dead = Some(error.clone());
                Err(error)
            }
        }
    }

    /// A read with a fallback: `first`, then `then` with the same request when `first` is answered
    /// `unsupported`, as the trait's default bodies delegate.
    fn with_fallback<T>(
        first: impl FnOnce() -> Result<T, TargetError>,
        then: impl FnOnce() -> Result<T, TargetError>,
    ) -> Result<T, TargetError> {
        match first() {
            Err(error) if error.is_unsupported() => then(),
            answer => answer,
        }
    }
}

/// The handshake exchange, on a peer whose next id is 0.
fn handshake<P: Peer>(
    link: &mut Link<P>,
    timeout_ms: u64,
    suite: &HandshakeSuite,
) -> Result<ImplementationIdentity, HandshakeRefusal> {
    let request = wire::HandshakeRequest {
        reads: READS,
        suite,
    };
    match round_trip(
        link,
        Op::Handshake,
        timeout_ms,
        &request,
        wire::handshake_answer,
    ) {
        Ok((speaks, identity)) => {
            if READS.contains(&speaks.as_str()) {
                identity
            } else {
                Err(HandshakeRefusal::Version {
                    speaks: version_of(&speaks),
                })
            }
        }
        Err(Failure::Fault(fault)) => Err(HandshakeRefusal::Fault(fault)),
        Err(Failure::Target(error)) => Err(HandshakeRefusal::Refused {
            kind: match error {
                TargetError::Unsupported { .. } => "unsupported",
                TargetError::Unavailable { .. } => "unavailable",
                TargetError::NotGranted { .. } => "not_granted",
            },
        }),
    }
}

/// The version a `speaks` names, when it names one of this format's family.
fn version_of(speaks: &str) -> Option<u32> {
    let digits = speaks.strip_prefix("ess-conformance-target/")?;
    if digits.is_empty() || digits.len() > 9 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

#[derive(serde::Serialize)]
struct Envelope<'a, R: serde::Serialize> {
    id: u64,
    op: &'static str,
    limit_ms: u64,
    request: &'a R,
}

/// Writes one request, reads one answer and decodes it.
fn round_trip<P: Peer, T>(
    link: &mut Link<P>,
    op: Op,
    limit_ms: u64,
    request: &impl serde::Serialize,
    decode: impl FnOnce(&exact::Value) -> Result<T, exact::Bad>,
) -> Result<T, Failure> {
    let id = link.next_id;
    link.next_id = link.next_id.saturating_add(1);
    let fault = |kind: FaultKind| {
        Failure::Fault(ProtocolFault {
            kind,
            id,
            op,
            limit_ms,
        })
    };
    let written = serde_json::to_string(&Envelope {
        id,
        op: op.name(),
        limit_ms,
        request,
    })
    .map_err(|_| {
        Failure::Target(TargetError::unavailable(
            op.name(),
            "the request could not be written as JSON",
        ))
    })?;
    link.peer.send(&written).map_err(|f| fault(f.into()))?;
    let bytes = link.peer.receive(limit_ms).map_err(|f| fault(f.into()))?;
    if bytes.len() > MAX_LINE {
        return Err(fault(FaultKind::LineTooLong));
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| {
        fault(FaultKind::NotUtf8 {
            at: e.valid_up_to(),
        })
    })?;
    let bad = |bad: exact::Bad| match bad {
        exact::Bad::NotJson(at) => fault(FaultKind::NotJson { at }),
        exact::Bad::Number(at) => fault(FaultKind::NumberOutOfRange { at }),
        exact::Bad::Shape(at) => fault(FaultKind::NotAnAnswer { at }),
    };
    let value = exact::parse(text).map_err(bad)?;
    let exact::Kind::Map(members) = &value.kind else {
        return Err(bad(exact::Bad::Shape(value.at)));
    };
    let named = members
        .get("id")
        .ok_or(exact::Bad::Shape(value.at))
        .and_then(exact::unsigned)
        .map_err(bad)?;
    if named != id {
        return Err(fault(FaultKind::WrongId { named }));
    }
    exact::object(&value, &["id"], &["ok", "target_error"]).map_err(bad)?;
    match (members.get("ok"), members.get("target_error")) {
        (Some(ok), None) => decode(ok).map_err(bad),
        (None, Some(error)) => Err(Failure::Target(wire::target_error(error).map_err(bad)?)),
        _ => Err(bad(exact::Bad::Shape(value.at))),
    }
}

impl<P: Peer> ConformanceTarget for ProcessTarget<P> {
    fn fixture_values(
        &self,
        scenario: &ScenarioContext,
        contract: &crate::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        self.call(
            Op::FixtureValues,
            0,
            Some(scenario),
            &wire::FixtureRequest { scenario, contract },
            exact::node_map,
        )
    }

    fn open_periodic(
        &self,
        request: crate::periodic::Open,
    ) -> Result<crate::periodic::Opened, TargetError> {
        self.call(Op::OpenPeriodic, 0, None, &request, wire::opened)
    }

    fn observe_periodic(
        &self,
        request: crate::periodic::Observe,
    ) -> Result<crate::periodic::Observation, TargetError> {
        let extra = request.elapsed.hold.millis();
        self.call(
            Op::ObservePeriodic,
            extra,
            None,
            &request,
            wire::observation,
        )
    }

    fn close_periodic(
        &self,
        scope: crate::periodic::Scope,
    ) -> Result<crate::periodic::Closed, TargetError> {
        self.call(Op::ClosePeriodic, 0, None, &scope, wire::closed)
    }

    fn observe_clock_reading(
        &self,
        request: crate::reading::ReadingObservationRequest,
    ) -> Result<ess_domain::reading::ReadingEvidence, TargetError> {
        self.call(Op::ObserveClockReading, 0, None, &request, exact::serde)
    }

    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        let wire = wire::EntityRequest::of(&request);
        self.call(Op::EstablishEntity, 0, None, &wire, wire::unit)
    }

    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(self.identity.clone())
    }

    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.call(Op::BeginScenario, 0, Some(scenario), scenario, wire::unit)
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let wire = wire::CommandRequest::of(&request);
        self.call(Op::ExecuteCommand, 0, None, &wire, wire::command_result)
    }

    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let wire = wire::AbsentInputWire::of(&request);
        self.call(
            Op::ExecuteCommandWithoutInput,
            0,
            None,
            &wire,
            wire::command_result,
        )
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let wire = wire::ViewRequest::of(&request, None);
        self.call(Op::QueryView, 0, None, &wire, wire::view_result)
    }

    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        Self::with_fallback(
            || {
                let wire = wire::ViewRequest::of(&request, Some(reader));
                self.call(Op::QueryViewAs, 0, None, &wire, wire::view_result)
            },
            || self.query_view(request.clone()),
        )
    }

    fn query_view_anonymous(
        &self,
        request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        Self::with_fallback(
            || {
                let wire = wire::ViewRequest::of(&request, None);
                self.call(Op::QueryViewAnonymous, 0, None, &wire, wire::view_result)
            },
            || self.query_view(request.clone()),
        )
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.call(Op::ObserveEvents, 0, None, &request, wire::events)
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.call(Op::ConfigureExternalOutcome, 0, None, &request, wire::unit)
    }

    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        let wire = wire::RepeatedOutcomeRequest {
            force: &request.force,
            correlation: &request.correlation,
            times,
        };
        self.call(
            Op::ConfigureExternalOutcomeRepeatedly,
            0,
            None,
            &wire,
            wire::unit,
        )
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.call(Op::RedeliverEvent, 0, None, &request, wire::unit)
    }

    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        let wire = wire::DeliveryRequest::of(&request);
        self.call(Op::DeliverEvent, 0, None, &wire, wire::unit)
    }

    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.call(Op::ObserveInvocations, 0, None, &request, wire::invocations)
    }

    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.call(Op::MarkInstant, 0, None, &request, wire::unit)
    }

    fn hold_now(&self, request: NowHold) -> Result<(), TargetError> {
        self.call(Op::HoldNow, 0, None, &request, wire::unit)
    }

    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        let extra = request.hold.millis();
        self.call(Op::ObserveElapsed, extra, None, &request, wire::elapsed)
    }

    fn scan_view(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        let wire = wire::ScanRequest::of(&request, None);
        self.call(Op::ScanView, 0, None, &wire, wire::scan)
    }

    fn scan_view_as(
        &self,
        request: OrderedScanRequest,
        reader: &ActorRef,
    ) -> Result<OrderedScan, TargetError> {
        Self::with_fallback(
            || {
                let wire = wire::ScanRequest::of(&request, Some(reader));
                self.call(Op::ScanViewAs, 0, None, &wire, wire::scan)
            },
            || self.scan_view(request.clone()),
        )
    }

    fn scan_view_anonymous(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        Self::with_fallback(
            || {
                let wire = wire::ScanRequest::of(&request, None);
                self.call(Op::ScanViewAnonymous, 0, None, &wire, wire::scan)
            },
            || self.scan_view(request.clone()),
        )
    }

    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        let extra = request.within.map_or(0, crate::scenario::Elapsed::millis);
        let wire = wire::AwaitWire::of(&request);
        self.call(Op::AwaitOutcome, extra, None, &wire, wire::await_answer)
    }

    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        {
            let link = self.link.borrow();
            if link.dead.is_some() || link.fault.is_some() {
                // The killed process closed the context; the scenario already carries its one
                // error check.
                return Ok(());
            }
        }
        self.call(Op::EndScenario, 0, None, scenario, wire::unit)
    }
}
