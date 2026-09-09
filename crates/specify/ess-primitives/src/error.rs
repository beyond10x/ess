//! Parse and validation errors.
//!
//! The two error types mirror the two-stage document model:
//!
//! * [`ParseError`] — a value is not well formed (bad identifier, unparsable predicate).
//!   Raised while deserializing, and therefore reported by [`serde`] with document context.
//! * [`ValidationError`] — a document is well formed but not semantically valid (a
//!   transition points at a state that does not exist). Raised by `TryFrom` conversions and
//!   collected into [`ValidationErrors`] so that one run reports every problem, not the
//!   first.

use std::fmt;
use std::fmt::Write as _;

/// A value that is not well formed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// An identifier violates its charset rule.
    #[error("invalid {kind} identifier {value:?}: {reason}")]
    Identifier {
        /// What kind of identifier was expected, such as `principle`.
        kind: &'static str,
        /// The offending value.
        value: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A version reference could not be parsed.
    #[error("invalid {kind} reference {value:?}: {reason}")]
    Reference {
        /// What kind of reference was expected, such as `protocol`.
        kind: &'static str,
        /// The offending value.
        value: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A predicate expression could not be parsed.
    #[error("cannot parse predicate {expression:?}: {reason}")]
    Predicate {
        /// The offending expression.
        expression: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A capability string is not a known capability.
    #[error("unknown capability {value:?}: {reason}")]
    Capability {
        /// The offending value.
        value: String,
        /// Why it was rejected, including the accepted names where the list is short enough.
        reason: String,
    },

    /// A nested construct is written deeper than the parser will walk.
    ///
    /// Its own variant rather than a [`Self::Identifier`] with a different sentence in it, because
    /// this is the one parse failure a caller has to be able to recognise mechanically: it is the
    /// refusal that stands between a document and a stack overflow, and a harness that cannot tell
    /// it apart from "you misspelt a type" cannot report it as what it is. `kind` names the
    /// construct and `limit` states the depth, so the message never has to be read for either.
    #[error("{kind} nesting is deeper than {limit} levels: {value}")]
    TooDeep {
        /// What nests, such as `type` or `predicate`.
        kind: &'static str,
        /// The offending value, truncated: what is being refused is unbounded by definition.
        value: String,
        /// The greatest depth the parser accepts.
        limit: usize,
    },

    /// A document fragment has the wrong shape.
    #[error("{location}: expected {expected}, found {found}")]
    Shape {
        /// Where in the document the problem is, in dotted form.
        location: String,
        /// What was expected.
        expected: String,
        /// What was found.
        found: String,
    },
}

impl ParseError {
    /// Builds a [`ParseError::Identifier`].
    pub fn identifier(kind: &'static str, value: &str, reason: String) -> Self {
        Self::Identifier {
            kind,
            value: value.to_owned(),
            reason,
        }
    }

    /// Builds a [`ParseError::Reference`].
    pub fn reference(kind: &'static str, value: &str, reason: impl Into<String>) -> Self {
        Self::Reference {
            kind,
            value: value.to_owned(),
            reason: reason.into(),
        }
    }

    /// Builds a [`ParseError::Predicate`].
    pub fn predicate(expression: &str, reason: impl Into<String>) -> Self {
        Self::Predicate {
            expression: expression.to_owned(),
            reason: reason.into(),
        }
    }

    /// Builds a [`ParseError::Capability`].
    pub fn capability(value: &str, reason: impl Into<String>) -> Self {
        Self::Capability {
            value: value.to_owned(),
            reason: reason.into(),
        }
    }

    /// Builds a [`ParseError::TooDeep`], quoting at most [`Self::ECHO_LIMIT`] bytes of the value.
    ///
    /// Truncated because the value that triggers this is the one value in the format with no length
    /// bound — a type reference nested ten thousand deep is a ten-thousand-character string, and an
    /// error that echoes it whole turns a refusal into a second denial of service.
    pub fn too_deep(kind: &'static str, value: &str, limit: usize) -> Self {
        Self::TooDeep {
            kind,
            value: echoable(value),
            limit,
        }
    }

    /// How much of an offending value an error message repeats.
    pub const ECHO_LIMIT: usize = 60;

    /// Builds a [`ParseError::Shape`].
    pub fn shape(
        location: impl Into<String>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::Shape {
            location: location.into(),
            expected: expected.into(),
            found: found.into(),
        }
    }
}

/// `value` as an error message may repeat it: quoted, and cut at a character boundary.
fn echoable(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= ParseError::ECHO_LIMIT {
        return format!("{trimmed:?}");
    }
    let head: String = trimmed.chars().take(ParseError::ECHO_LIMIT).collect();
    format!("{head:?}…")
}

/// Declares every validation code once.
///
/// The wire string and the [`ValidationCode::ALL`] list are generated from the same line as the
/// variant, because both were previously maintained by hand and both had silently fallen behind:
/// five codes existed, were emitted, and were absent from the list the tests iterate — so the guard
/// that was supposed to catch exactly that reported success.
macro_rules! validation_codes {
    ($( $(#[$attribute:meta])* $variant:ident => $wire:literal, )*) => {
        /// Stable machine-readable classification of a semantic validation failure.
        ///
        /// Codes are part of the public interface: harnesses and tests match on them rather than on
        /// message text.
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
        )]
        #[serde(rename_all = "snake_case")]
        #[non_exhaustive]
        pub enum ValidationCode {
            $( $(#[$attribute])* $variant, )*
        }

        impl ValidationCode {
            /// Every code this build can produce, in declaration order.
            ///
            /// Generated, so it cannot fall behind the enum.
            pub const ALL: &'static [Self] = &[ $( Self::$variant, )* ];

            /// The code as it appears in output, such as `unreachable_state`.
            pub fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $wire, )*
                }
            }
        }
    };
}

validation_codes! {
    /// A workflow declares no states.
    EmptyWorkflow => "empty_workflow",

    /// The initial state does not exist.
    UnknownInitialState => "unknown_initial_state",

    /// A transition, obligation or override references a state that does not exist.
    UnknownState => "unknown_state",

    /// A non-terminal state has no outgoing transition, so execution would wedge.
    DeadEndState => "dead_end_state",

    /// A state cannot be reached from the initial state.
    UnreachableState => "unreachable_state",

    /// Two transitions share the same source and target.
    DuplicateTransition => "duplicate_transition",

    /// A referenced principle is not in the registry.
    UnknownPrinciple => "unknown_principle",

    /// The same principle is listed twice.
    DuplicatePrinciple => "duplicate_principle",

    /// A referenced profile is not in the registry.
    UnknownProfile => "unknown_profile",

    /// A referenced workflow is not in the registry.
    UnknownWorkflow => "unknown_workflow",

    /// A referenced protocol is not in the registry.
    UnknownProtocol => "unknown_protocol",

    /// A document requires a protocol major version this build does not implement.
    UnsupportedProtocolVersion => "unsupported_protocol_version",

    /// A document is written in a specification format version this build does not implement.
    ///
    /// Distinct from [`Self::UnsupportedProtocolVersion`], which is about the protocol a project
    /// executes: `ess/2` names how a document is written, not what governs the work, and a tool
    /// told the wrong one goes looking for the wrong upgrade.
    UnsupportedFormatVersion => "unsupported_format_version",

    /// A document uses a construct this build does not implement.
    ///
    /// Distinct from [`Self::UnsupportedFormatVersion`], which is about the version a document is
    /// written in: this one is a legal-looking thing the reader will implement later, and telling
    /// the two apart is the difference between "upgrade the tool" and "write it another way".
    UnsupportedConstruct => "unsupported_construct",

    /// A profile or task references a capability the protocol does not declare.
    UndeclaredCapability => "undeclared_capability",

    /// A requirement references an evidence kind the protocol does not declare.
    UndeclaredEvidenceKind => "undeclared_evidence_kind",

    /// Rollback is required for a state marked irreversible.
    RollbackOnIrreversibleState => "rollback_on_irreversible_state",

    /// A capability is both needed and explicitly denied.
    CapabilityConflict => "capability_conflict",

    /// Production mutation is allowed without an approval requirement.
    ProductionWriteWithoutApproval => "production_write_without_approval",

    /// A condition reads something nothing makes available.
    ///
    /// In a protocol, a predicate references a fact the protocol does not declare observable. In a
    /// specification, a command's guard or a view's filter reads a field its subject does not have.
    /// Both are conditions nobody can decide.
    UnobservableFact => "unobservable_fact",

    /// An obligation is timed against a phase no state declares.
    UnknownPhase => "unknown_phase",

    /// A version mismatch between a pinned reference and the registry entry.
    VersionMismatch => "version_mismatch",

    /// A rollback failure policy is declared with no way to identify what to roll back to.
    IncompleteRollbackPolicy => "incomplete_rollback_policy",

    /// Something references itself where that cannot mean anything.
    SelfReference => "self_reference",

    /// Something that exists to change state changes nothing.
    ///
    /// A command whose acceptance would produce a revision nobody can explain; a specified outcome
    /// that neither emits an event nor names an error, so nothing about it is observable.
    EmptyChange => "empty_change",

    /// A refusal is recorded together with a change.
    ///
    /// An audit record that says an action was refused and also records a change; a specified
    /// outcome that reports an error and also emits. A refused command changes nothing, so either
    /// is two outcomes wearing one name.
    RefusalMutatedState => "refusal_mutated_state",

    /// An audit record says an entity changed without recording what changed.
    UnreconstructableChange => "unreconstructable_change",

    /// Something that settles an outcome does not say on what.
    ///
    /// A record of a decision that does not say what was decided; a specified outcome decided
    /// outside the input that names no cause, which leaves nobody able to reproduce it.
    UnexplainedDecision => "unexplained_decision",

    /// An audit record's redaction fields contradict each other.
    RedactionInconsistent => "redaction_inconsistent",

    /// An event's declared type does not match what its payload asserts.
    EventPayloadMismatch => "event_payload_mismatch",

    /// An event names a subject without the revision it describes, or the reverse.
    IncompleteEventSubject => "incomplete_event_subject",

    /// An event caused by a command does not name that command as its cause.
    MissingCausation => "missing_causation",

    /// Something written where a reference belongs is not one, and differs from a recognised form
    /// by a typo.
    ///
    /// Distinct from [`Self::UndeclaredReference`], where the reference is well formed and names
    /// nothing. Here the text was not read as a reference at all — `evnt.customer_email` becomes a
    /// literal string and is sent as one — so "not declared" would be a false statement about a
    /// name nobody looked up.
    MisspelledReference => "misspelled_reference",

    /// A reference names something nothing declares.
    ///
    /// Distinct from [`Self::UnknownState`], which is about workflow states specifically: a
    /// specification refers to types, events, errors, entities, fields and domains too, and a tool
    /// reading `unknown_state` off a missing event learns the wrong thing about what to fix.
    UndeclaredReference => "undeclared_reference",

    /// The same name is declared twice, and neither declaration can be said to win.
    DuplicateDeclaration => "duplicate_declaration",

    /// A declaration declares nothing that could have an effect: no fields, no outcomes, no
    /// states, nothing to enforce, no statement of what being finished means.
    EmptyDeclaration => "empty_declaration",

    /// A document does not make a declaration it is required to make: a required key is absent, or
    /// nothing declares the thing every other source contributes to.
    ///
    /// Distinct from [`Self::EmptyDeclaration`], which is about a declaration that is there and
    /// says nothing. Here there is nothing to read at all, and the repair is to write one.
    MissingDeclaration => "missing_declaration",

    /// Two declarations are each well formed and cannot both hold.
    ConflictingDeclaration => "conflicting_declaration",

    /// A declared value is not the kind of thing its position requires.
    ///
    /// A declared type disagreeing with the type it must match; a task naming something that is not
    /// an artifact reference; an absolute path where only a relative one has a meaning.
    TypeMismatch => "type_mismatch",

    /// A set of conditional branches leaves some input with no branch, so what happens to it is
    /// unspecified.
    ///
    /// Distinct from [`Self::DeadEndState`], which is about a state machine with no way onward: the
    /// branches here are each fine, and it is the gap between them that nobody has decided.
    NonExhaustiveBranches => "non_exhaustive_branches",

    /// Every branch of a declaration is decided outside its input, so no caller can reach one by
    /// choosing what to send.
    ///
    /// Distinct from [`Self::UnreachableState`], which is about a graph nothing walks into, and
    /// from [`Self::EmptyChange`], which is about a branch that does nothing: these branches do
    /// something, and nothing a caller can write selects between them.
    UnreachableBranch => "unreachable_branch",
}

impl fmt::Display for ValidationCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What kind of construct a refusal is about.
///
/// The typed half of what [`ValidationError::location`] used to be spelled as prose. A consumer
/// reading a diagnostic needs the layer the defect is in — `ESS-COMMAND-001`'s `COMMAND` — and
/// deriving that by splitting a human-facing path on `.` and `[` makes the emitted code a function
/// of how a producer wrote a `format!`. See `docs/design/review-typed-diagnostics.md`.
///
/// [`Self::as_str`] is the token the producers already write at the head of a location, so a
/// rendered path is byte-identical to the string it replaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ConstructKind {
    /// A named type.
    Type,
    /// A declared conversion between two types.
    Conversion,
    /// An entity or its lifecycle.
    Entity,
    /// A command.
    Command,
    /// An event.
    Event,
    /// A declared error.
    Error,
    /// A view.
    View,
    /// An actor.
    Actor,
    /// A binding, including its mapping.
    Binding,
    /// A component.
    Component,
    /// The topology.
    Topology,
    /// A bounded context.
    Domain,
    /// The specification as a whole, or something with no better home.
    Specification,
}

impl ConstructKind {
    /// The token this kind renders as at the head of a location, such as `command`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Conversion => "conversion",
            Self::Entity => "entity",
            Self::Command => "command",
            Self::Event => "event",
            Self::Error => "error",
            Self::View => "view",
            Self::Actor => "actor",
            Self::Binding => "binding",
            Self::Component => "component",
            Self::Topology => "topology",
            Self::Domain => "domain",
            Self::Specification => "spec",
        }
    }
}

impl fmt::Display for ConstructKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One step of the path from a construct to the member a refusal is about.
///
/// The distinction a dotted string cannot carry. `outcomes` is a key of the schema and `accepted`
/// is a name its author chose, and a consumer looking for the line an author wrote has to know
/// which is which. The compiler used to guess, with a fifty-entry stop-list of keys that never name
/// a declaration; the producer already knows, so it says so here instead.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment {
    /// A key of the specification language, such as `outcomes` or `payload`.
    Key(String),
    /// A name the author wrote, such as an outcome name or a field name.
    Name(String),
    /// A positional element of a list, such as the second declared input.
    Index(usize),
}

/// The construct a refusal is about: its kind, its qualified name, and the path to the member.
///
/// Built left to right — `ConstructRef::new(kind, name).key("outcomes").named("accepted")` — and
/// rendered by [`Self::render`] into the string [`ValidationError::location`] has always carried.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstructRef {
    kind: ConstructKind,
    name: String,
    members: Vec<Segment>,
}

impl ConstructRef {
    /// The construct itself, with no member path yet.
    pub fn new(kind: ConstructKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
            members: Vec::new(),
        }
    }

    /// What kind of construct this is.
    pub fn kind(&self) -> ConstructKind {
        self.kind
    }

    /// The construct's qualified name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The path from the construct to the member, in order.
    pub fn members(&self) -> &[Segment] {
        &self.members
    }

    /// Descends through a key of the specification language.
    #[must_use]
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.members.push(Segment::Key(key.into()));
        self
    }

    /// Descends through a name the author wrote.
    #[must_use]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.members.push(Segment::Name(name.into()));
        self
    }

    /// Descends into a positional element.
    #[must_use]
    pub fn index(mut self, index: usize) -> Self {
        self.members.push(Segment::Index(index));
        self
    }

    /// The document path, exactly as a producer used to write it with `format!`.
    ///
    /// The one definition of a sited refusal's [`ValidationError::location`]. Keeping it in one
    /// function is what makes "the adopter-facing string does not move" a property that can be
    /// checked, rather than a promise made once per call site.
    pub fn render(&self) -> String {
        let mut rendered = String::with_capacity(self.name.len() + 16);
        rendered.push_str(self.kind.as_str());
        rendered.push('.');
        rendered.push_str(&self.name);
        for member in &self.members {
            match member {
                Segment::Key(key) => {
                    rendered.push('.');
                    rendered.push_str(key);
                }
                Segment::Name(name) => {
                    rendered.push('.');
                    rendered.push_str(name);
                }
                Segment::Index(index) => {
                    let _ = write!(rendered, "[{index}]");
                }
            }
        }
        rendered
    }
}

impl fmt::Display for ConstructRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

/// Where a construct was written, when the producer knows.
///
/// Nothing supplies one yet: `serde_yaml` discards positions for a semantic error, so the compiler
/// still finds the line by searching the source text. The type exists because the first producer
/// that can supply a position needs no change to the bridge that consumes it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxSpan {
    /// The file, as the specification labelled it.
    pub source: String,
    /// The line, 1-based.
    pub line: usize,
    /// The column, 1-based.
    pub column: usize,
}

/// What a refusal is about, independently of how it is worded.
///
/// The rule's identity is [`ValidationError::code`] and always was; what was missing is the
/// construct and the position, both of which used to be recovered by parsing prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// The construct and the member path within it.
    pub construct: ConstructRef,
    /// Where it was written, when that is known.
    pub span: Option<SyntaxSpan>,
}

/// One semantic validation failure.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ValidationError {
    /// Stable classification.
    pub code: ValidationCode,
    /// Where the problem is, in dotted document form, such as `workflow.transitions[3].to`.
    ///
    /// Adopter-facing output. On a refusal built by [`Self::at`] it is exactly
    /// [`ConstructRef::render`] of that refusal's site and is never read back to recover a fact.
    pub location: String,
    /// What is wrong.
    pub message: String,
    /// How to fix it, when there is an obvious remedy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// The typed site, on producers that have been migrated to one.
    ///
    /// Private, so that nothing can set a site without also setting the [`Self::location`] it
    /// renders; [`Self::at`] is the only way in, and it sets both from one value.
    ///
    /// `#[serde(skip)]`: this struct is `deny_unknown_fields` and appears in no generated schema,
    /// so serializing the site would make it a wire-visible field change for a fact that is only
    /// ever consumed in-process by `ess-compiler`'s bridge. The serialized bytes do not move.
    ///
    /// Boxed: a `ValidationError` is returned in `Result::Err` by `TypeRegistry::insert` and its
    /// siblings, and an unboxed `Site` puts the error variant at 176 bytes — which
    /// `clippy::result_large_err` refuses, and rightly: the site is present on a minority of
    /// refusals and absent on every success.
    #[serde(skip)]
    site: Option<Box<Site>>,
}

impl ValidationError {
    /// Builds a validation error from a document path written as a string.
    ///
    /// The untyped path, kept while the migration in `docs/design/review-typed-diagnostics.md`
    /// runs. A refusal built this way carries no [`Self::site`], and the compiler falls back to
    /// deriving its family and its source line from `location`.
    pub fn new(
        code: ValidationCode,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            location: location.into(),
            message: message.into(),
            hint: None,
            site: None,
        }
    }

    /// Builds a validation error about a construct, rendering its location from that construct.
    pub fn at(construct: ConstructRef, code: ValidationCode, message: impl Into<String>) -> Self {
        Self {
            code,
            location: construct.render(),
            message: message.into(),
            hint: None,
            site: Some(Box::new(Site {
                construct,
                span: None,
            })),
        }
    }

    /// The typed site, on a refusal built by [`Self::at`].
    pub fn site(&self) -> Option<&Site> {
        self.site.as_deref()
    }

    /// Records where the construct was written.
    ///
    /// Ignored on a refusal that has no site: a position without a construct is the heuristic this
    /// story exists to remove, wearing a typed hat.
    #[must_use]
    pub fn with_span(mut self, span: SyntaxSpan) -> Self {
        if let Some(site) = self.site.as_mut() {
            site.span = Some(span);
        }
        self
    }

    /// Attaches a remediation hint.
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.location, self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, " (hint: {hint})")?;
        }
        Ok(())
    }
}

/// Every semantic validation failure found in one document or resolution.
///
/// Validation accumulates: a document with four broken references reports four errors, so a
/// caller does not have to fix and re-run four times.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(transparent)]
pub struct ValidationErrors(Vec<ValidationError>);

impl ValidationErrors {
    /// An empty error set.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Records a failure.
    pub fn push(&mut self, error: ValidationError) {
        self.0.push(error);
    }

    /// Records a failure and returns `self`, for builder-style accumulation.
    #[must_use]
    pub fn with(mut self, error: ValidationError) -> Self {
        self.push(error);
        self
    }

    /// Absorbs another error set.
    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    /// `true` when nothing failed.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The number of failures.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The failures, in discovery order.
    pub fn as_slice(&self) -> &[ValidationError] {
        &self.0
    }

    /// `Ok(value)` when nothing failed, otherwise the error set.
    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }

    /// `true` when any failure carries `code`.
    pub fn contains(&self, code: ValidationCode) -> bool {
        self.0.iter().any(|error| error.code == code)
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ValidationError> for ValidationErrors {
    fn from(error: ValidationError) -> Self {
        Self(vec![error])
    }
}

impl IntoIterator for ValidationErrors {
    type Item = ValidationError;
    type IntoIter = std::vec::IntoIter<ValidationError>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.len() {
            0 => f.write_str("no validation errors"),
            1 => write!(f, "{}", self.0[0]),
            n => {
                writeln!(f, "{n} validation errors:")?;
                for error in &self.0 {
                    writeln!(f, "  - {error}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ValidationErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_has_a_distinct_stable_string() {
        let mut seen: Vec<&str> = Vec::new();
        for code in ValidationCode::ALL {
            let rendered = code.as_str();
            assert!(
                !rendered.is_empty()
                    && rendered.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{code:?} renders as {rendered:?}, which is not a snake_case code"
            );
            assert!(
                !seen.contains(&rendered),
                "two codes both render as {rendered:?}; a caller matching on it cannot tell them apart"
            );
            seen.push(rendered);
        }
    }

    #[test]
    fn the_serialised_form_matches_the_string_form() {
        for code in ValidationCode::ALL {
            let json = serde_json::to_string(code).expect("serialises");
            assert_eq!(
                json,
                format!("\"{}\"", code.as_str()),
                "a code's wire form and its display form must not drift apart"
            );
        }
    }

    #[test]
    fn errors_accumulate_and_report_every_problem() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());
        errors.push(ValidationError::new(
            ValidationCode::UnknownState,
            "workflow.transitions[0].to",
            "`ghost` is not a declared state",
        ));
        errors.push(
            ValidationError::new(
                ValidationCode::DeadEndState,
                "workflow.states.a",
                "`a` has no outgoing transition",
            )
            .with_hint("add a transition, or mark it terminal"),
        );

        assert_eq!(errors.len(), 2);
        assert!(errors.contains(ValidationCode::DeadEndState));
        assert!(!errors.contains(ValidationCode::UnreachableState));

        let rendered = errors.to_string();
        assert!(rendered.contains("2 validation errors"), "{rendered}");
        assert!(rendered.contains("hint: add a transition"), "{rendered}");
    }

    /// The rendering rule, over every shape a producer writes today.
    ///
    /// `location` is the adopter-facing string and is pinned by the guide, by 48 in-crate
    /// assertions and by `ess specify validate`'s own output. It must be a function of the typed
    /// site and nothing else, so the table is the definition and this is the check.
    #[test]
    fn a_construct_reference_renders_the_location_string_its_producer_used_to_write() {
        let cases: Vec<(ConstructRef, &str)> = vec![
            (
                ConstructRef::new(ConstructKind::Command, "shop.orders.PlaceOrder"),
                "command.shop.orders.PlaceOrder",
            ),
            (
                ConstructRef::new(ConstructKind::Command, "shop.orders.PlaceOrder").key("outcomes"),
                "command.shop.orders.PlaceOrder.outcomes",
            ),
            (
                ConstructRef::new(ConstructKind::Command, "shop.orders.PlaceOrder")
                    .key("outcomes")
                    .named("placed"),
                "command.shop.orders.PlaceOrder.outcomes.placed",
            ),
            (
                ConstructRef::new(ConstructKind::Command, "shop.repeat.FileOne")
                    .key("input")
                    .index(1),
                "command.shop.repeat.FileOne.input[1]",
            ),
            (
                ConstructRef::new(ConstructKind::Command, "shop.cross.Announce")
                    .key("outcomes")
                    .named("announced")
                    .key("payload")
                    .named("shop.cross.Announced")
                    .named("headline"),
                "command.shop.cross.Announce.outcomes.announced.payload.shop.cross.Announced.headline",
            ),
            (
                ConstructRef::new(ConstructKind::Type, "shop.orders.OrderId").key("of"),
                "type.shop.orders.OrderId.of",
            ),
        ];
        for (construct, expected) in cases {
            assert_eq!(construct.render(), expected);
        }
    }

    /// The invariant that keeps the rendered string honest: nothing may set one without the other.
    #[test]
    fn a_sited_error_renders_its_location_from_its_site_and_a_string_error_has_no_site() {
        let construct = ConstructRef::new(ConstructKind::Command, "shop.orders.PlaceOrder")
            .key("outcomes")
            .named("placed")
            .key("emits");
        let sited = ValidationError::at(
            construct.clone(),
            ValidationCode::UndeclaredReference,
            "`shop.orders.Missing` is not a declared event",
        );
        assert_eq!(sited.location, construct.render());
        assert_eq!(
            sited.location,
            "command.shop.orders.PlaceOrder.outcomes.placed.emits"
        );
        let site = sited.site().expect("a sited error carries its site");
        assert_eq!(site.construct.kind(), ConstructKind::Command);
        assert_eq!(site.construct.name(), "shop.orders.PlaceOrder");
        assert_eq!(
            site.construct.members(),
            &[
                Segment::Key("outcomes".to_owned()),
                Segment::Name("placed".to_owned()),
                Segment::Key("emits".to_owned()),
            ]
        );
        assert!(site.span.is_none(), "no producer has a parser position yet");

        let string_only = ValidationError::new(
            ValidationCode::UndeclaredReference,
            "command.shop.orders.PlaceOrder.outcomes.placed.emits",
            "`shop.orders.Missing` is not a declared event",
        );
        assert!(string_only.site().is_none());
        assert_eq!(sited.location, string_only.location);
    }

    /// The site travels in-process only, so the serialized bytes and the schema do not move.
    #[test]
    fn a_site_does_not_reach_the_serialized_form() {
        let sited = ValidationError::at(
            ConstructRef::new(ConstructKind::Command, "shop.orders.PlaceOrder").key("outcomes"),
            ValidationCode::EmptyDeclaration,
            "declares no outcomes",
        )
        .with_span(SyntaxSpan {
            source: "orders.yaml".to_owned(),
            line: 12,
            column: 5,
        });
        let string_only = ValidationError::new(
            ValidationCode::EmptyDeclaration,
            "command.shop.orders.PlaceOrder.outcomes",
            "declares no outcomes",
        );
        assert_eq!(
            serde_json::to_string(&sited).expect("serialises"),
            serde_json::to_string(&string_only).expect("serialises"),
            "a serialized ValidationError must be byte-identical with and without a site"
        );
    }
}
