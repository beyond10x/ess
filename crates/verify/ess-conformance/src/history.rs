//! `ess-history/1` and `ess-history/2`: one recorded concurrent run, as a document the checker
//! reads.
//!
//! A concurrent runner drives an adopter's target from several clients at once and records, per
//! operation, which client called which command on which subject, when the call was invoked, when
//! it returned and what it answered. `ess-history/2` adds one optional operation field,
//! `decision_time`: the UTC instant the command's decision observed
//! ([`crate::occurrence_clock`], beyond10x/ess#244). A writer selects format 2 exactly when an
//! operation records one ([`HistoryFormat::for_operations`]) and refuses to write format 1 with one,
//! so a document written before the field existed keeps its bytes and its meaning. The document is
//! declared by `models/concurrent-history/`;
//! the types here are that declaration in Rust, and `crates/edge/ess-xtask/tests/history_model.rs`
//! fails when the two disagree in a field or an enum value. `schemas/ess-history.schema.json` is
//! the same shape for the Go and TypeScript writers, held to the model by the same test.
//!
//! [`read`] is the one way in. It refuses, each by a named [`HistoryRefusal`]:
//!
//! 1. bytes that are not exactly this document — an unknown, missing or repeated field at either
//!    level, the document or an operation written as a JSON array rather than an object, a value
//!    outside its type, a number written with a decimal point or an exponent, a `decision_time`
//!    that is not the one spelling of an instant, written `null`, or carried by an
//!    `ess-history/1` document ([`HistoryRefusal::Malformed`], the detail naming which) — or
//!    another `format`, a future major included. [`read_format1`] is the retained format 1 reader: it
//!    refuses `ess-history/2` by its format, before any operation is read;
//! 2. a history recorded against a specification other than the one being checked, before any
//!    operation is looked at;
//! 3. an operation whose completion disagrees with its instants, its outcome or its rows, one on a
//!    client the history does not count, two operations with one identity (compared without regard
//!    to hexadecimal case), and a UUID not written in lower case.
//!
//! A [`Completion::Returned`] operation carries both its return instant and its outcome. An
//! operation with no answer is [`Completion::Indeterminate`]. It carries no return instant and
//! no outcome: a timed-out call may still take effect, so its return is read as later than every
//! other operation's ([`ReturnBound::AfterEveryOther`], design decision 4). Reading it as failed
//! produces false violations.
//!
//! Instants (`invoked_at`, `returned_at`) are readings of a monotonic integer clock the writer
//! chooses. Its unit is not fixed: the checker compares instants only by order, so milliseconds,
//! nanoseconds or a logical counter all serve, provided one history uses one clock. They are never
//! read as the current time; `decision_time` is the only UTC instant a history records. Every integer
//! in the document — instants, `seed`, `clients`, `client` — is at most [`MAX_INTEGER`]
//! (2^53 − 1), the largest a TypeScript writer can write exactly; a larger one is refused by name.
//!
//! [`read`] is the only way from bytes to a [`History`]: the public types serialize and do not
//! deserialize, so no caller can obtain one that skipped these checks.
//!
//! An `Optional` field may be absent or `null`; both read as `None` — except `decision_time`, which
//! is absent where no decision was observed and is never `null`. Everything else has one
//! spelling, and one document reads to one value whatever the order of its keys.

use std::collections::BTreeSet;
use std::fmt;

use serde::de::{self, Deserializer};
use serde::ser::{self, SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

pub use crate::occurrence_clock::DecisionInstant;
pub use ess_primitives::evidence::SpecDigest;

/// The `format` of a history whose operations record no decision time, which every writer before
/// `ess-history/2` wrote.
pub const HISTORY_FORMAT: &str = "ess-history/1";

/// The `format` of a history at least one of whose operations records a `decision_time`.
pub const HISTORY_FORMAT_2: &str = "ess-history/2";

/// The largest integer an `ess-history/1` document carries: 2^53 − 1.
///
/// Every integer above it is refused by [`HistoryRefusal::IntegerOutOfRange`], because a
/// TypeScript writer holds numbers as doubles and cannot write a larger one exactly.
pub const MAX_INTEGER: u64 = 9_007_199_254_740_991;

/// `concurrent.history.HistoryFormat`: the closed set of history formats this build reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HistoryFormat {
    /// `ess-history/1`.
    #[serde(rename = "ess-history/1")]
    EssHistory1,
    /// `ess-history/2`: an operation may carry `decision_time`.
    #[serde(rename = "ess-history/2")]
    EssHistory2,
}

impl HistoryFormat {
    /// The format a writer selects for `operations`: `ess-history/2` exactly when one of them
    /// records a decision time.
    #[must_use]
    pub fn for_operations(operations: &[Operation]) -> Self {
        if operations
            .iter()
            .any(|operation| operation.decision_time.is_some())
        {
            Self::EssHistory2
        } else {
            Self::EssHistory1
        }
    }

    /// The `format` string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EssHistory1 => HISTORY_FORMAT,
            Self::EssHistory2 => HISTORY_FORMAT_2,
        }
    }

    fn named(text: &str) -> Option<Self> {
        [Self::EssHistory1, Self::EssHistory2]
            .into_iter()
            .find(|format| format.as_str() == text)
    }
}

/// `concurrent.history.QualifiedName`: a command or outcome name, never empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct QualifiedName(String);

impl QualifiedName {
    /// Admits a non-empty name.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.is_empty() {
            return Err("a qualified name must not be empty".to_owned());
        }
        Ok(Self(value))
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for QualifiedName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// A `Uuid` identity in the canonical hyphenated form (`ess_primitives::facts::is_canonical_uuid`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Uuid(String);

impl Uuid {
    /// Admits a canonical hyphenated UUID.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if !ess_primitives::facts::is_canonical_uuid(&value) {
            return Err(format!("`{value}` is not a canonical hyphenated UUID"));
        }
        Ok(Self(value))
    }

    /// The identity as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the identity is in its one admitted spelling: lower-case hexadecimal.
    ///
    /// Parsing admits either case so that [`read`] can compare identities without regard to case
    /// and then refuse an upper-case spelling by name.
    pub fn is_lower_case(&self) -> bool {
        !self.0.bytes().any(|byte| byte.is_ascii_uppercase())
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// `concurrent.history.Completion`: whether an operation answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Completion {
    /// The call answered at `returned_at`.
    Returned,
    /// The call never answered; it may still have taken effect.
    Indeterminate,
}

/// `concurrent.history.Verdict`: what checking a history concludes.
///
/// `Unknown` is never a pass (design decision 5): a search that exhausts its budget says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Verdict {
    /// A sequential order the specification accepts exists.
    Linearizable,
    /// No sequential order the specification accepts exists.
    Violation,
    /// The search did not finish.
    Unknown,
}

/// `concurrent.history.History`: one recorded concurrent run.
///
/// [`read`] is the only way to obtain one from bytes: this type serializes, and does not
/// deserialize, so every document passes the reader's checks. What the reader admits on the wire
/// is the private `HistoryWire`, held to the same model as this type.
///
/// ```compile_fail,E0277
/// let _: ess_conformance::history::History = serde_json::from_str("{}").unwrap();
/// ```
///
/// The same call on a type that does deserialize compiles, so the failure above is the missing
/// `Deserialize`, not a missing crate:
///
/// ```
/// let completion: ess_conformance::history::Completion =
///     serde_json::from_str("\"Returned\"").unwrap();
/// assert_eq!(completion, ess_conformance::history::Completion::Returned);
/// ```
///
/// Writing one refuses `ess-history/1` forced onto an operation that records a decision time:
///
/// ```
/// use ess_conformance::history::{read, HistoryFormat, SpecDigest};
/// let digest = SpecDigest::new("ab".repeat(32)).unwrap();
/// let bytes = br#"{"format":"ess-history/2","history_id":"00000000-0000-4000-8000-000000000001","spec_digest":"abababababababababababababababababababababababababababababababab","seed":1,"clients":1,"operations":[{"operation_id":"00000000-0000-4000-8000-000000000002","client":0,"command":"demo.c.Run","subject_key":"","invoked_at":1,"returned_at":2,"completion":"Returned","outcome":"ran","decision_time":"2026-10-04T12:00:00Z"}]}"#;
/// let mut history = read(bytes, &digest).unwrap();
/// assert_eq!(serde_json::to_vec(&history).unwrap(), bytes);
/// history.format = HistoryFormat::EssHistory1;
/// assert!(serde_json::to_vec(&history).is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    /// [`HistoryFormat::EssHistory2`] where an operation records a decision time, otherwise
    /// [`HistoryFormat::EssHistory1`] ([`HistoryFormat::for_operations`]).
    pub format: HistoryFormat,
    /// The run's identity.
    pub history_id: Uuid,
    /// The digest of the compiled specification the run was driven from.
    pub spec_digest: SpecDigest,
    /// The seed the run was drawn from, at most [`MAX_INTEGER`].
    pub seed: u64,
    /// How many clients drove the target; every operation's `client` is below it.
    pub clients: u64,
    /// Every recorded operation, in the order written.
    pub operations: Vec<Operation>,
}

/// `concurrent.history.Operation`: one call, from invoke to return.
///
/// The model's `history_id` on an operation is the `via` of `History.operations`; in the document
/// it is carried by nesting the operation inside its history. Like [`History`], it serializes and
/// is read only through [`read`].
///
/// ```compile_fail,E0277
/// let _: ess_conformance::history::Operation = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    /// The operation's identity, unique within its history.
    pub operation_id: Uuid,
    /// The client that made the call, `0..clients`.
    pub client: u64,
    /// The command called.
    pub command: QualifiedName,
    /// The identity of the subject the command addressed.
    pub subject_key: String,
    /// When the call was invoked, on the writer's monotonic integer clock.
    pub invoked_at: u64,
    /// When it returned, on the same clock; present exactly when `completion` is `Returned`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returned_at: Option<u64>,
    /// Whether the call answered.
    pub completion: Completion,
    /// The outcome it answered with; present exactly when `completion` is `Returned`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<QualifiedName>,
    /// For a read of a view: the identity of each row it answered with, in the order answered.
    ///
    /// Written only on a `Returned` read, and only where every row carries the identity of the
    /// instance it projects. Absent on a command, and on a read that could not record it; the
    /// checker judges no read without it ([`crate::linearize`]). A document written before the
    /// field existed reads unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<String>>,
    /// For a client retry: the operation whose request this one sent again, unchanged.
    ///
    /// A retry is written only by a runner injecting the faults a specification declares, and only
    /// for a command declaring a `replays` branch: one logical request, sent twice, is answered by
    /// its origin branch at most once and by the retained result otherwise
    /// ([`crate::linearize`]). It names an earlier operation of the same client and command. Absent
    /// on every other operation; a document written before the field existed reads unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_of: Option<Uuid>,
    /// The instant this operation's command decision observed (`ess-history/2`), at full
    /// precision.
    ///
    /// Taken from the command's own decision edge ([`crate::target::RecordedCommandCompletion`]):
    /// never from the request, the recorder's clock or `invoked_at`/`returned_at`. Absent where no
    /// decision was observed — a call refused before its decision edge, a retained answer
    /// delivered again, a target recording none — which the checker reads as no evidence, never
    /// as an instant. An `Indeterminate` operation carries one when its decision began.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_time: Option<DecisionInstant>,
}

impl Serialize for History {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.format == HistoryFormat::EssHistory1 {
            if let Some(operation) = self
                .operations
                .iter()
                .find(|operation| operation.decision_time.is_some())
            {
                return Err(ser::Error::custom(format!(
                    "`{HISTORY_FORMAT}` carries no `decision_time`, and operation {} records one; \
                     write `{HISTORY_FORMAT_2}`",
                    operation.operation_id.as_str()
                )));
            }
        }
        let mut document = serializer.serialize_struct("History", 6)?;
        document.serialize_field("format", &self.format)?;
        document.serialize_field("history_id", &self.history_id)?;
        document.serialize_field("spec_digest", &self.spec_digest)?;
        document.serialize_field("seed", &self.seed)?;
        document.serialize_field("clients", &self.clients)?;
        document.serialize_field("operations", &self.operations)?;
        document.end()
    }
}

/// What [`read`] admits for a [`History`]: the same fields, deserialized only from a JSON object.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryWire {
    format: HistoryFormat,
    history_id: Uuid,
    spec_digest: SpecDigest,
    seed: u64,
    clients: u64,
    #[serde(deserialize_with = "objects")]
    operations: Vec<Operation>,
}

impl From<HistoryWire> for History {
    fn from(wire: HistoryWire) -> Self {
        Self {
            format: wire.format,
            history_id: wire.history_id,
            spec_digest: wire.spec_digest,
            seed: wire.seed,
            clients: wire.clients,
            operations: wire.operations,
        }
    }
}

/// What [`read`] admits for an [`Operation`]. An absent `Option` field reads as `None` without
/// `default`: serde's derive treats a missing `Option` that way.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationWire {
    operation_id: Uuid,
    client: u64,
    command: QualifiedName,
    subject_key: String,
    invoked_at: u64,
    returned_at: Option<u64>,
    completion: Completion,
    outcome: Option<QualifiedName>,
    rows: Option<Vec<String>>,
    retry_of: Option<Uuid>,
    decision_time: Option<DecisionInstant>,
}

impl From<OperationWire> for Operation {
    fn from(wire: OperationWire) -> Self {
        Self {
            operation_id: wire.operation_id,
            client: wire.client,
            command: wire.command,
            subject_key: wire.subject_key,
            invoked_at: wire.invoked_at,
            returned_at: wire.returned_at,
            completion: wire.completion,
            outcome: wire.outcome,
            rows: wire.rows,
            retry_of: wire.retry_of,
            decision_time: wire.decision_time,
        }
    }
}

/// When an operation is read as having returned.
///
/// Ordered: every [`At`](Self::At) precedes [`AfterEveryOther`](Self::AfterEveryOther).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReturnBound {
    /// It returned at this instant.
    At(u64),
    /// It never answered, so it is read as returning after every other operation.
    AfterEveryOther,
}

impl Operation {
    /// When this operation is read as having returned (design decision 4).
    ///
    /// An admitted operation is `Returned` with a return instant or `Indeterminate` without one,
    /// so the completion alone decides.
    pub fn return_bound(&self) -> ReturnBound {
        match (self.completion, self.returned_at) {
            (Completion::Returned, Some(instant)) => ReturnBound::At(instant),
            _ => ReturnBound::AfterEveryOther,
        }
    }
}

/// Every way [`read`] refuses a history, each under its own name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryRefusal {
    /// The bytes are not an `ess-history/1` document: not JSON, an unknown or missing field, or a
    /// value outside its declared type.
    Malformed {
        /// What the parser reported.
        detail: String,
    },
    /// The `format` is absent, or names a format this build does not read.
    UnsupportedFormat {
        /// The `format` string, if there was one.
        found: Option<String>,
    },
    /// The history was recorded against another specification.
    SpecDigestMismatch {
        /// The digest of the compiled IR being checked.
        expected: SpecDigest,
        /// The digest the history carries.
        found: SpecDigest,
    },
    /// The history counts no clients.
    NoClients,
    /// Two operations carry one identity.
    DuplicateOperation {
        /// The repeated identity.
        operation_id: String,
    },
    /// An operation names a client at or above `clients`.
    ClientOutOfRange {
        /// The operation.
        operation_id: String,
        /// The client it names.
        client: u64,
        /// The history's client count.
        clients: u64,
    },
    /// An operation is `Returned` and carries no return instant.
    ReturnedWithoutReturnInstant {
        /// The operation.
        operation_id: String,
    },
    /// An operation's return instant precedes its invoke instant.
    ReturnBeforeInvoke {
        /// The operation.
        operation_id: String,
        /// When it was invoked.
        invoked_at: u64,
        /// When it is recorded as having returned.
        returned_at: u64,
    },
    /// An operation is `Indeterminate` and carries a return instant.
    IndeterminateWithReturnInstant {
        /// The operation.
        operation_id: String,
    },
    /// An operation is `Indeterminate` and carries an outcome.
    IndeterminateWithOutcome {
        /// The operation.
        operation_id: String,
    },
    /// An operation is `Indeterminate` and carries `rows`: a read that never answered answered no
    /// rows.
    IndeterminateWithRows {
        /// The operation.
        operation_id: String,
    },
    /// An operation is `Returned` and carries no outcome.
    ReturnedWithoutOutcome {
        /// The operation.
        operation_id: String,
    },
    /// A UUID is written with upper-case hexadecimal; its one spelling is lower case.
    NonCanonicalUuid {
        /// The UUID as written.
        value: String,
    },
    /// An integer is above [`MAX_INTEGER`].
    IntegerOutOfRange {
        /// Where it was written: `seed`, `clients` or `operations[<i>].<field>`.
        field: String,
        /// The value written.
        value: u64,
    },
    /// An operation's `retry_of` names no earlier operation of the same client and command: a
    /// retry sends again a request its own client already sent.
    RetryOfUnknown {
        /// The retry.
        operation_id: String,
        /// What it names.
        retry_of: String,
    },
}

impl HistoryRefusal {
    /// The refusal's stable name: its entry in [`HistoryRefusal::CATALOGUE`].
    pub fn code(&self) -> &'static str {
        self.catalogue_entry().key
    }
}

crate::authored::diagnostic_catalogue! {
    impl HistoryRefusal => &'static str {
        Self::Malformed { .. } => "history.malformed",
            "The bytes are not an `ess-history/1` or `ess-history/2` document: not JSON, an \
             unknown or missing field, a value outside its declared type, or a `decision_time` \
             that is `null`, not the one spelling of an instant, or in an `ess-history/1` \
             document.",
            "Write the document as `schemas/ess-history.schema.json` describes; the detail names \
             the field or the position.";
        Self::UnsupportedFormat { .. } => "history.unsupported-format",
            "The `format` is absent, or names a format this build does not read.",
            "Write `format: ess-history/1`, or `ess-history/2` where an operation records a \
             `decision_time`.";
        Self::SpecDigestMismatch { .. } => "history.spec-digest-mismatch",
            "The history was recorded against another specification.",
            "Check the history against the specification it was recorded against, or record it \
             again against this one.";
        Self::NoClients => "history.no-clients",
            "The history counts no clients.",
            "Set `clients` to the number of clients that sent operations.";
        Self::DuplicateOperation { .. } => "history.duplicate-operation",
            "Two operations carry one identity.",
            "Give every operation its own identity.";
        Self::ClientOutOfRange { .. } => "history.client-out-of-range",
            "An operation names a client at or above `clients`.",
            "Number clients from 0 to `clients` minus 1, or raise `clients`.";
        Self::ReturnedWithoutReturnInstant { .. } => "history.returned-without-return-instant",
            "An operation is `Returned` and carries no return instant.",
            "Record `returned_at` for an operation that returned, or mark it `Indeterminate`.";
        Self::ReturnBeforeInvoke { .. } => "history.return-before-invoke",
            "An operation's return instant precedes its invoke instant.",
            "Record instants from one clock, so an operation returns at or after it was invoked.";
        Self::IndeterminateWithReturnInstant { .. } => "history.indeterminate-with-return-instant",
            "An operation is `Indeterminate` and carries a return instant.",
            "Drop `returned_at`, or mark the operation `Returned` if it did return.";
        Self::IndeterminateWithOutcome { .. } => "history.indeterminate-with-outcome",
            "An operation is `Indeterminate` and carries an outcome.",
            "Drop `outcome`, or mark the operation `Returned` if it did return.";
        Self::IndeterminateWithRows { .. } => "history.indeterminate-with-rows",
            "An operation is `Indeterminate` and carries `rows`: a read that never answered \
             answered no rows.",
            "Drop `rows`, or mark the operation `Returned` if it did return.";
        Self::ReturnedWithoutOutcome { .. } => "history.returned-without-outcome",
            "An operation is `Returned` and carries no outcome.",
            "Record the outcome the operation returned.";
        Self::NonCanonicalUuid { .. } => "history.non-canonical-uuid",
            "A UUID is written with upper-case hexadecimal; its one spelling is lower case.",
            "Write the UUID in lower case.";
        Self::IntegerOutOfRange { .. } => "history.integer-out-of-range",
            "An integer is above the largest integer a history admits.",
            "Keep the value at or below the bound the message states.";
        Self::RetryOfUnknown { .. } => "history.retry-of-unknown",
            "An operation's `retry_of` names no earlier operation of the same client and command.",
            "Name the earlier operation this one sends again, from the same client and command, \
             or drop `retry_of`.";
    }
}

impl fmt::Display for HistoryRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Malformed { detail } => write!(formatter, "{code}: {detail}"),
            Self::UnsupportedFormat { found: Some(found) } => write!(
                formatter,
                "{code}: `{found}` is not a history format this reader reads"
            ),
            Self::UnsupportedFormat { found: None } => write!(
                formatter,
                "{code}: no `format` string; expected `{HISTORY_FORMAT}` or `{HISTORY_FORMAT_2}`"
            ),
            Self::SpecDigestMismatch { expected, found } => write!(
                formatter,
                "{code}: the history was recorded against specification {found}, not {expected}"
            ),
            Self::NoClients => write!(formatter, "{code}: `clients` is 0"),
            Self::DuplicateOperation { operation_id } => {
                write!(formatter, "{code}: operation {operation_id} appears twice")
            }
            Self::ClientOutOfRange {
                operation_id,
                client,
                clients,
            } => write!(
                formatter,
                "{code}: operation {operation_id} names client {client} of {clients}"
            ),
            Self::ReturnedWithoutReturnInstant { operation_id } => write!(
                formatter,
                "{code}: operation {operation_id} is `Returned` with no `returned_at`"
            ),
            Self::ReturnBeforeInvoke {
                operation_id,
                invoked_at,
                returned_at,
            } => write!(
                formatter,
                "{code}: operation {operation_id} returned at {returned_at}, before it was \
                 invoked at {invoked_at}"
            ),
            Self::IndeterminateWithReturnInstant { operation_id } => write!(
                formatter,
                "{code}: operation {operation_id} is `Indeterminate` and carries `returned_at`"
            ),
            Self::IndeterminateWithOutcome { operation_id } => write!(
                formatter,
                "{code}: operation {operation_id} is `Indeterminate` and carries `outcome`"
            ),
            Self::IndeterminateWithRows { operation_id } => write!(
                formatter,
                "{code}: operation {operation_id} is `Indeterminate` and carries `rows`"
            ),
            Self::ReturnedWithoutOutcome { operation_id } => write!(
                formatter,
                "{code}: operation {operation_id} is `Returned` with no `outcome`"
            ),
            Self::NonCanonicalUuid { value } => write!(
                formatter,
                "{code}: `{value}` is not written in lower-case hexadecimal"
            ),
            Self::IntegerOutOfRange { field, value } => write!(
                formatter,
                "{code}: `{field}` is {value}, above the largest admitted integer {MAX_INTEGER}"
            ),
            Self::RetryOfUnknown {
                operation_id,
                retry_of,
            } => write!(
                formatter,
                "{code}: operation {operation_id} is a retry of {retry_of}, which is no earlier \
                 operation of the same client and command"
            ),
        }
    }
}

impl std::error::Error for HistoryRefusal {}

/// Only the `format`, read before anything else so another format is refused by that name.
#[derive(Deserialize)]
struct FormatProbe {
    #[serde(default)]
    format: Option<serde_json::Value>,
}

/// A `T` read only from a JSON object.
///
/// serde's derived struct visitor also accepts a JSON array in field order, which is not an
/// `ess-history/1` document. This asks the deserializer for a map and hands the map to `T`'s own
/// derived visitor, so unknown, missing and repeated fields are still refused by that visitor.
struct ObjectOnly<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for ObjectOnly<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visit<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Visit<T> {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
                T::deserialize(de::value::MapAccessDeserializer::new(map))
            }
        }

        deserializer
            .deserialize_map(Visit(std::marker::PhantomData))
            .map(ObjectOnly)
    }
}

/// `History.operations`: an array whose every element is a JSON object.
fn objects<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Operation>, D::Error> {
    Vec::<ObjectOnly<OperationWire>>::deserialize(deserializer).map(|operations| {
        operations
            .into_iter()
            .map(|ObjectOnly(wire)| Operation::from(wire))
            .collect()
    })
}

/// Refuses an integer above [`MAX_INTEGER`], naming where it was written.
fn in_range(field: impl FnOnce() -> String, value: u64) -> Result<(), HistoryRefusal> {
    if value <= MAX_INTEGER {
        Ok(())
    } else {
        Err(HistoryRefusal::IntegerOutOfRange {
            field: field(),
            value,
        })
    }
}

/// Reads one `ess-history/1` or `ess-history/2` document recorded against the specification whose
/// digest is `expected`.
///
/// The format is decided first, the shape second, the digest third and each operation last, so a
/// history recorded against another specification is refused by
/// [`HistoryRefusal::SpecDigestMismatch`] whatever its operations hold. When several operations
/// are wrong, the first in document order is reported; within one operation, a repeated identity
/// is reported before a non-canonical spelling of it.
pub fn read(bytes: &[u8], expected: &SpecDigest) -> Result<History, HistoryRefusal> {
    read_admitting(
        bytes,
        expected,
        &[HistoryFormat::EssHistory1, HistoryFormat::EssHistory2],
    )
}

/// The retained `ess-history/1` reader: [`read`] for a consumer that holds no decision-time
/// authority. `ess-history/2` is refused by its format before any operation is read.
pub fn read_format1(bytes: &[u8], expected: &SpecDigest) -> Result<History, HistoryRefusal> {
    read_admitting(bytes, expected, &[HistoryFormat::EssHistory1])
}

fn read_admitting(
    bytes: &[u8],
    expected: &SpecDigest,
    admitted: &[HistoryFormat],
) -> Result<History, HistoryRefusal> {
    let malformed = |error: serde_json::Error| HistoryRefusal::Malformed {
        detail: error.to_string(),
    };
    let ObjectOnly(probe): ObjectOnly<FormatProbe> =
        serde_json::from_slice(bytes).map_err(malformed)?;
    match probe.format {
        Some(serde_json::Value::String(format))
            if HistoryFormat::named(&format).is_some_and(|named| admitted.contains(&named)) => {}
        Some(serde_json::Value::String(format)) => {
            return Err(HistoryRefusal::UnsupportedFormat {
                found: Some(format),
            });
        }
        _ => return Err(HistoryRefusal::UnsupportedFormat { found: None }),
    }
    null_decision_time(bytes)?;
    let ObjectOnly(wire): ObjectOnly<HistoryWire> =
        serde_json::from_slice(bytes).map_err(malformed)?;
    let history = History::from(wire);
    if history.format == HistoryFormat::EssHistory1 {
        if let Some(operation) = history
            .operations
            .iter()
            .find(|operation| operation.decision_time.is_some())
        {
            return Err(HistoryRefusal::Malformed {
                detail: format!(
                    "operation {} records a `decision_time`, which `{HISTORY_FORMAT}` does not \
                     carry; write `{HISTORY_FORMAT_2}`",
                    operation.operation_id.as_str()
                ),
            });
        }
    }
    if &history.spec_digest != expected {
        return Err(HistoryRefusal::SpecDigestMismatch {
            expected: expected.clone(),
            found: history.spec_digest,
        });
    }
    in_range(|| "seed".to_owned(), history.seed)?;
    in_range(|| "clients".to_owned(), history.clients)?;
    lower_case(&history.history_id)?;
    if history.clients == 0 {
        return Err(HistoryRefusal::NoClients);
    }
    let mut seen = BTreeSet::new();
    for (index, operation) in history.operations.iter().enumerate() {
        if !seen.insert(operation.operation_id.as_str().to_ascii_lowercase()) {
            return Err(HistoryRefusal::DuplicateOperation {
                operation_id: operation.operation_id.as_str().to_owned(),
            });
        }
        lower_case(&operation.operation_id)?;
        let at = |field: &'static str| move || format!("operations[{index}].{field}");
        in_range(at("client"), operation.client)?;
        in_range(at("invoked_at"), operation.invoked_at)?;
        if let Some(returned_at) = operation.returned_at {
            in_range(at("returned_at"), returned_at)?;
        }
        check(operation, history.clients)?;
        if let Some(original) = &operation.retry_of {
            lower_case(original)?;
            let sent_before = history.operations[..index].iter().any(|earlier| {
                earlier
                    .operation_id
                    .as_str()
                    .eq_ignore_ascii_case(original.as_str())
                    && earlier.client == operation.client
                    && earlier.command == operation.command
            });
            if !sent_before {
                return Err(HistoryRefusal::RetryOfUnknown {
                    operation_id: operation.operation_id.as_str().to_owned(),
                    retry_of: original.as_str().to_owned(),
                });
            }
        }
    }
    Ok(history)
}

/// Refuses an operation writing `decision_time` as `null`, which serde would read as absent.
///
/// Only the key's presence is read here; the operation's shape is the derived reader's.
fn null_decision_time(bytes: &[u8]) -> Result<(), HistoryRefusal> {
    #[derive(Deserialize)]
    struct Operations {
        #[serde(default)]
        operations: Option<serde_json::Value>,
    }
    let Ok(Operations {
        operations: Some(serde_json::Value::Array(operations)),
    }) = serde_json::from_slice::<Operations>(bytes)
    else {
        return Ok(());
    };
    match operations.iter().position(|operation| {
        operation
            .get("decision_time")
            .is_some_and(serde_json::Value::is_null)
    }) {
        Some(operation) => Err(HistoryRefusal::Malformed {
            detail: format!(
                "`operations[{operation}].decision_time` is `null`; leave it out where no decision \
                 was observed"
            ),
        }),
        None => Ok(()),
    }
}

/// Refuses a UUID written with upper-case hexadecimal.
fn lower_case(identity: &Uuid) -> Result<(), HistoryRefusal> {
    if identity.is_lower_case() {
        Ok(())
    } else {
        Err(HistoryRefusal::NonCanonicalUuid {
            value: identity.as_str().to_owned(),
        })
    }
}

/// One operation's own consistency: its client, and its completion against its instants and its
/// outcome.
fn check(operation: &Operation, clients: u64) -> Result<(), HistoryRefusal> {
    let operation_id = || operation.operation_id.as_str().to_owned();
    if operation.client >= clients {
        return Err(HistoryRefusal::ClientOutOfRange {
            operation_id: operation_id(),
            client: operation.client,
            clients,
        });
    }
    match (operation.completion, operation.returned_at) {
        (Completion::Returned, None) => Err(HistoryRefusal::ReturnedWithoutReturnInstant {
            operation_id: operation_id(),
        }),
        (Completion::Returned, Some(returned_at)) if returned_at < operation.invoked_at => {
            Err(HistoryRefusal::ReturnBeforeInvoke {
                operation_id: operation_id(),
                invoked_at: operation.invoked_at,
                returned_at,
            })
        }
        (Completion::Returned, Some(_)) if operation.outcome.is_none() => {
            Err(HistoryRefusal::ReturnedWithoutOutcome {
                operation_id: operation_id(),
            })
        }
        (Completion::Indeterminate, Some(_)) => {
            Err(HistoryRefusal::IndeterminateWithReturnInstant {
                operation_id: operation_id(),
            })
        }
        (Completion::Indeterminate, None) if operation.outcome.is_some() => {
            Err(HistoryRefusal::IndeterminateWithOutcome {
                operation_id: operation_id(),
            })
        }
        (Completion::Indeterminate, None) if operation.rows.is_some() => {
            Err(HistoryRefusal::IndeterminateWithRows {
                operation_id: operation_id(),
            })
        }
        (Completion::Returned, Some(_)) | (Completion::Indeterminate, None) => Ok(()),
    }
}
