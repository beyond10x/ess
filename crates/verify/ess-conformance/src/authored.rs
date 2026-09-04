//! The scenarios a person wrote, checked against the model and compiled into the same suite.
//!
//! # What this is for
//!
//! [`synthesize`](mod@crate::synthesize) writes the scenarios a specification *obliges*: a branch is
//! declared, so a suite owes a check about it. That covers everything a model determines and, by
//! construction, nothing it does not — and it says so, because a construct it cannot witness comes
//! back as a [`Refusal`](crate::Refusal) rather than as a silence. One of those refusals reads *the
//! contract is declared; the algorithm is not*, and it is the reason this module exists.
//!
//! A router's matching order, a scorer's tie-break, the rule that decides which of two waiting calls
//! is dispatched first: the model can declare the command, the view and the types, and it will never
//! derive the algorithm. Somebody has to write that down. Before this module the only places to
//! write it were a bespoke runner in each consuming repository — where a scenario naming a field the
//! model no longer declares fails at run time, separately, in each of them — or nowhere.
//!
//! So an authored scenario is a scenario like any other, with two differences that are the whole
//! point:
//!
//! * **It is checked against the model before it runs.** Every command, actor, outcome, event,
//!   error, view, entity, field, enum variant and lifecycle state it names is resolved at compile
//!   time, and a name the model does not declare is refused *by name*, with the file it was read
//!   from. That is the value a bespoke runner cannot offer: today a scenario naming a field that was
//!   renamed last week keeps passing until something executes it.
//! * **It is told apart from a generated one, everywhere.** [`ScenarioId::Authored`] is its own id
//!   shape, so the two populations cannot collide in a suite and cannot be confused in a report. A
//!   coverage number that counted a person's assertion as an obligation the specification derived
//!   would be describing a model that does not exist.
//!
//! Everything else is deliberately identical. An authored scenario compiles into the same
//! [`ConformanceScenario`] over the same closed [`ScenarioStep`] vocabulary and lands in the same
//! `ess-conformance/3` document synthesis writes. A verb that needed its own runner would be a
//! second definition of what a scenario means, which is the failure [`scenario`](crate::scenario)
//! exists to prevent.
//!
//! # The document
//!
//! ```yaml
//! type: ess-scenario/1
//! domain: billing.invoice
//! scenario: two-issued-invoices-rank-latest-first
//! summary: The most recently issued invoice is the first row of OutstandingInvoices.
//!
//! arrange:
//!   - instance: earlier
//!     entity: billing.invoice.Invoice
//!
//! timeline:
//!   - at: 2026-01-05T09:00:00Z
//!     command: billing.invoice.CreateInvoice
//!     actor: billing.invoice.Customer
//!     input:
//!       account_id: 3f1d5b7e-0000-4000-8000-000000000001
//!       customer_email: earlier@example.test
//!       amount: {amount: 120, currency: EUR}
//!     outcome: accepted
//!     events:
//!       - event: billing.invoice.InvoiceCreated
//!         payload: {customer_email: earlier@example.test}
//!     capture: {instance: earlier, event: billing.invoice.InvoiceCreated, field: invoice_id}
//!
//! assert:
//!   - view: billing.invoice.OutstandingInvoices
//!     at:
//!       row: first
//!       fields: {invoice_id: {$instance: earlier}}
//! ```
//!
//! # Elapsed time
//!
//! An act may name its instant, and a later act may say what must be true of the time between them.
//!
//! ```yaml
//! timeline:
//!   - at: 2026-01-05T09:00:13Z
//!     command: acd.routing.BridgeEntry
//!     mark: bridged
//!
//!   - at: 2026-01-05T09:00:33Z
//!     command: acd.routing.EnterQueue
//!     elapsed:
//!       - since: bridged
//!         not_before: PT20S
//!       - since: bridged
//!         quiet: {for: PT20S, events: [acd.routing.CallEnded]}
//! ```
//!
//! Three bounds, one per window, and each names the instant it opens at:
//!
//! | bound | what it claims | what the target is asked |
//! |---|---|---|
//! | `not_before: PT20S` | this act does not happen until twenty seconds after `since` | let the window close, then say how much passed |
//! | `within: PT5S` | this act happened no later than five seconds after `since` | say how much has passed |
//! | `quiet: {for: …, events: […]}` | none of these appeared in the window after `since` | let the window close, then say how often each appeared |
//!
//! **The anchor is written and never inferred.** Every other assertion in this format is anchored by
//! position — it is about the act it sits on. A window is not, and the suite this was built for is
//! the reason: its bounded negatives opened "wherever the preceding block happened to end", so
//! inserting one arrangement step silently moved five of them. `since:` names a `mark:`, a window
//! measured from an unmarked instant is refused, and an act cannot mark and measure at the same
//! instant.
//!
//! **The file's own instants are held to the claim.** `at:` still reaches no runner, but it is no
//! longer decorative: `not_before: PT20S` in a file whose two instants are five seconds apart is a
//! document that says two different things, and it is refused rather than compiled into whichever
//! one the compiler happened to read.
//!
//! **What a target has to be able to do, and what happens when it cannot.** A window is checked by
//! asking the target how much time passed since a marked instant, having first asked it to let the
//! window close. Whether it gets there by waiting on a wall clock or by advancing one it owns is its
//! business and this format has no opinion; what it may not do is stay quiet and be read as
//! agreeing. A target that implements neither method answers `unsupported`, the scenario is reported
//! `unsupported`, and §28 makes the run fail. There is no path on which an unheld window passes.
//! # Three decisions the format makes, and why
//!
//! **The timeline carries an explicit instant, and it has to ascend.** A list is ordered by where
//! its entries sit on the page, which is a fact nobody can see in a diff: move two lines and the
//! scenario means something else, silently. `at:` states the order the author meant, the compiler
//! refuses a file whose instants do not strictly ascend, and the steps come out in the order the
//! instants say. It reaches no runner — §37 puts every clock on the runner's side and a suite that
//! carried a deadline would mean something different on a slower machine — so what it buys is the
//! review, not the execution.
//!
//! **The author states the claim; the model states how to check it.** Whether a view assertion is
//! [`ExpectView`](ScenarioStep::ExpectView) or
//! [`EventuallyView`](ScenarioStep::EventuallyView) is read off
//! [`ResolvedView::assertion_style`](ess_compiler::ir::ResolvedView::assertion_style), never written
//! in the scenario, for the reason §14 gives: a choice made per assertion is a choice made wrong
//! eventually. The same goes for the ranking an
//! [`At`](ViewExpectation::At) or a [`Ranked`](ViewExpectation::Ranked) is relative to — the view
//! declares `order_by:`, so restating it here would be a second copy that can disagree, and a view
//! that declares none makes a position meaningless and is refused instead.
//!
//! **A reference is spelled with a sigil.** `{$instance: earlier}` and
//! `{$observed: {event: …, field: …}}` rather than a bare mapping, because a declared struct may
//! perfectly well have a field called `instance` — the argument [`ScenarioValue`] already makes
//! about the suite's own encoding. A `$` cannot begin an ESS field name (`Field::PATTERN` is
//! `^[A-Za-z][A-Za-z0-9_]*$`), so the two can never be confused, and a plain value is written
//! exactly as the model's own documents write one.
//!
//! # What is deliberately not here
//!
//! * **An implementation-specific assertion.** The step vocabulary is closed (see
//!   [`ScenarioStep`]), and it stays closed for an authored scenario: a claim this format cannot
//!   express is a semantic the specification does not have, and the answer is §18's — refuse, and
//!   say the model is incomplete.
//! * **A value a run produced, compared as a literal.** An event's payload and an error's fields are
//!   compared field by field against values the suite carries, which is what the steps that hold
//!   them declare; a reference is refused there rather than silently dropped.
//! * **A clock the runner honours.** `at:` orders the file, bounds the durations claimed against it,
//!   and stops there. Nothing here says what time it is, and a scenario's meaning does not change
//!   with the machine that runs it: an [`Elapsed`] is a length the specification's own timers wait,
//!   which reads the same everywhere, and it is measured by the only party that has a clock.
//! * **A producer that halted.** No bound here says an ordered scan *stopped*, only that nothing was
//!   published for a stated length of time — which is a claim about a window, not about a stream
//!   having ended. The two look alike and are not: a prefix read in silence is still a prefix. That
//!   is a different feature.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_compiler::diagnostic::Code;
use ess_compiler::ir::{EssIr, ResolvedField, ResolvedTypeRef};
use ess_domain::command::OutcomeName;
use ess_domain::name::QualifiedName;
use ess_domain::view::{AssertionStyle, Ranking};
use ess_primitives::error::ParseError;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use ess_primitives::time::{CivilDate, Timestamp};
use serde::Deserialize as _;

use crate::input::{bind, resolve_path, Completeness, ShapeError, ShapeErrors};
use crate::scenario::{
    ActorRef, AuthoredName, CommandRef, ConformanceScenario, DeclaredTypeRef, DomainRef, Elapsed,
    EntityRef, ErrorRef, EssSemanticRef, EventRef, InstanceName, InstantName, OutcomeRef, Position,
    ScenarioId, ScenarioPurpose, ScenarioStep, ScenarioValue, ViewExpectation, ViewRef,
};
use crate::synthesize::{payload_shape, reachable_types};

// ---- the document ------------------------------------------------------------------------------

/// The format this module reads.
pub const FORMAT: &str = "ess-scenario/1";

/// One file an author wrote, and where it was read from.
///
/// Text rather than a path, because compiling is not reading: this crate holds no file system, the
/// same way it holds no clock. [`origin`](Self::origin) is only ever printed — it is what a refusal
/// names so an author knows which file to open — so it is whatever spelling the caller would like a
/// reader to see.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Source {
    /// Where it came from, as a reader should see it.
    pub origin: String,
    /// What it says.
    pub text: String,
}

impl Source {
    /// One source.
    pub fn new(origin: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            origin: origin.into(),
            text: text.into(),
        }
    }
}

/// An authored scenario, as the document says it.
///
/// Unknown keys are refused: a mistyped `timelime:` that parsed into an empty timeline would be a
/// scenario that silently checks nothing, which is the one failure a passing run cannot show.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    /// The document format, `ess-scenario/1`.
    #[serde(rename = "type")]
    pub format: String,
    /// The bounded context the scenario is written about.
    pub domain: String,
    /// What the author calls it.
    pub scenario: AuthoredName,
    /// What it proves, in one line.
    pub summary: ScenarioPurpose,
    /// The instances the timeline may bind.
    #[serde(default)]
    pub arrange: Vec<Arrangement>,
    /// What happens, in the order the instants say.
    #[serde(default)]
    pub timeline: Vec<Act>,
    /// What must hold of the views afterwards.
    #[serde(default)]
    pub assert: Vec<Assertion>,
}

/// One instance the scenario acts on, and whose it is.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arrangement {
    /// What the timeline calls it.
    pub instance: InstanceName,
    /// The declared entity it is one of.
    pub entity: String,
}

/// One command, at one instant, and everything required of it.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Act {
    /// When, as `YYYY-MM-DDTHH:MM:SSZ`. Orders the file; reaches no runner.
    pub at: Moment,
    /// The declared command to invoke.
    pub command: String,
    /// As whom, where the specification grants commands to actors.
    #[serde(default)]
    pub actor: Option<String>,
    /// The input, by declared field name.
    #[serde(default)]
    pub input: BTreeMap<String, Written>,
    /// The declared branch it must take.
    #[serde(default)]
    pub outcome: Option<String>,
    /// The declared error it must report, and what it must carry.
    #[serde(default)]
    pub error: Option<ErrorClaim>,
    /// The occurrences it must publish.
    #[serde(default)]
    pub events: Vec<EventClaim>,
    /// The occurrences it must not publish.
    #[serde(default)]
    pub no_events: Vec<String>,
    /// The identity to bind, so later steps can name the instance this act brought into existence.
    #[serde(default)]
    pub capture: Option<Capture>,
    /// What must be true of the time between an earlier instant and this act.
    ///
    /// A list, because an act may legitimately carry more than one — "twenty seconds after the
    /// bridge, and nothing was offered in them" is two claims about one gap, and folding them into
    /// one key would be the [`AmbiguousClaim`](Cause::AmbiguousClaim) mistake in a new place.
    #[serde(default)]
    pub elapsed: Vec<Window>,
    /// The name a later window measures from this act's instant.
    ///
    /// Marked *after* the act, so a window cannot open at the act it is written on: that window has
    /// no width and the claim in it cannot fail.
    #[serde(default)]
    pub mark: Option<InstantName>,
}

/// One claim about the time between a marked instant and the act that carries it.
///
/// Exactly one of the three bounds, for the reason [`Assertion`] states about its six: two would be
/// two claims filed as one, and none would be a claim that cannot fail.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    /// The instant the window opens at — a name an earlier act marked.
    ///
    /// Required, and there is no default. The suite this format was extended for had a negative
    /// whose window opened wherever the preceding block happened to end, so inserting one step
    /// moved five assertions and no diff showed it. An anchor a reader has to reconstruct is an
    /// anchor that has already moved.
    pub since: InstantName,
    /// This act does not happen until at least this much has passed since `since`.
    #[serde(default, deserialize_with = "written_length")]
    pub not_before: Option<Elapsed>,
    /// This act happens no later than this much after `since`.
    #[serde(default, deserialize_with = "written_length")]
    pub within: Option<Elapsed>,
    /// These occurrences do not appear in a window of this length after `since`.
    #[serde(default)]
    pub quiet: Option<Quiet>,
}

/// A stretch of time in which named occurrences must not appear.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quiet {
    /// How long the window stays open.
    #[serde(rename = "for", deserialize_with = "required_length")]
    pub duration: Elapsed,
    /// The declared events that must not appear in it.
    pub events: Vec<String>,
}

/// A length of time as an author writes it: `PT20S`.
///
/// The written spelling and the persisted one are deliberately different, and each is right where it
/// is. A person writes `PT20S`, which says what it is; a suite writes `20`, which every runner in
/// every language can read without a date library. One value, two audiences, and the conversion
/// happens once — here — rather than in each runner.
fn required_length<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Elapsed, D::Error> {
    let raw = String::deserialize(deserializer)?;
    Elapsed::parse(&raw).map_err(serde::de::Error::custom)
}

/// The same, where the key is optional.
fn written_length<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Elapsed>, D::Error> {
    Option::<String>::deserialize(deserializer)?
        .map(|raw| Elapsed::parse(&raw).map_err(serde::de::Error::custom))
        .transpose()
}

impl Window {
    /// The bounds this window states, which has to be exactly one.
    fn stated(&self) -> Vec<&'static str> {
        let mut stated = Vec::new();
        if self.not_before.is_some() {
            stated.push("not_before");
        }
        if self.within.is_some() {
            stated.push("within");
        }
        if self.quiet.is_some() {
            stated.push("quiet");
        }
        stated
    }

    /// The length of the window, whichever bound states it.
    fn length(&self) -> Option<Elapsed> {
        self.not_before
            .or(self.within)
            .or_else(|| self.quiet.as_ref().map(|quiet| quiet.duration))
    }
}

/// The declared error a branch reports, and the fields to compare.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorClaim {
    /// Which declared error.
    pub name: String,
    /// The payload fields to compare, by name. Partial, as the step is.
    #[serde(default)]
    pub fields: BTreeMap<String, Written>,
}

/// An occurrence the act must publish, and the fields to compare.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventClaim {
    /// Which declared event.
    pub event: String,
    /// The payload fields to compare, by name. Partial, as the step is.
    #[serde(default)]
    pub payload: BTreeMap<String, Written>,
}

/// Binding the identity an act published, so later steps can name it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    /// The name later steps refer to it by; one the arrangement declares.
    pub instance: InstanceName,
    /// The event carrying the identity.
    pub event: String,
    /// The field of that event's payload.
    pub field: String,
}

/// One claim about one view.
///
/// Exactly one of the six expectation keys, because two would be two assertions filed as one and
/// none would be an assertion that cannot fail.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    /// Which declared view.
    pub view: String,
    /// The value bound to each parameter the view declares.
    #[serde(default)]
    pub params: BTreeMap<String, Written>,
    /// A row with these field values is present.
    #[serde(default)]
    pub contains: Option<BTreeMap<String, Written>>,
    /// No row with these field values is present.
    #[serde(default)]
    pub excludes: Option<BTreeMap<String, Written>>,
    /// The view holds a number of rows inside these bounds.
    #[serde(default)]
    pub counts: Option<Counts>,
    /// The rows are in the order the view declares.
    #[serde(default)]
    pub ranked: Option<bool>,
    /// The row at this position holds these field values.
    #[serde(default)]
    pub at: Option<At>,
    /// Every row satisfies this predicate, and there is at least one.
    #[serde(default)]
    pub satisfies: Option<Predicate>,
}

/// How many rows a view may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    /// The fewest.
    #[serde(default)]
    pub at_least: Option<usize>,
    /// The most.
    #[serde(default)]
    pub at_most: Option<usize>,
}

/// A claim about one row of a view, by position in the order the view declares.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct At {
    /// Which row.
    pub row: Row,
    /// The fields that row must match. Partial, as the step is.
    #[serde(default)]
    pub fields: BTreeMap<String, Written>,
}

/// Which row an [`At`] is about: `first`, `last`, or an index counting from zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    /// The row the declared order puts first.
    First,
    /// The row the declared order puts last.
    Last,
    /// The row at this index.
    Nth(usize),
}

impl Row {
    /// The suite's own spelling of it.
    fn position(self) -> Position {
        match self {
            Self::First => Position::First,
            Self::Last => Position::Last,
            Self::Nth(index) => Position::Nth { index },
        }
    }
}

impl<'de> serde::Deserialize<'de> for Row {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Node::deserialize(deserializer)? {
            Node::Text(word) if word == "first" => Ok(Self::First),
            Node::Text(word) if word == "last" => Ok(Self::Last),
            Node::Number(index) if index.is_integral() && index.get() >= 0.0 => {
                // The cast is guarded by the two tests above it: integral and not negative.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Ok(Self::Nth(index.get() as usize))
            }
            other => Err(serde::de::Error::custom(format!(
                "`row` is `first`, `last` or a whole number from zero, not {}",
                other.type_name()
            ))),
        }
    }
}

/// An instant in the timeline, written `YYYY-MM-DDTHH:MM:SSZ`.
///
/// UTC, to the second, and zero-padded — the spelling
/// [`Timestamp::iso_8601`](ess_primitives::time::Timestamp::iso_8601) already writes everywhere else
/// in this workspace. Fixed width and one time zone are what make the ordering of the written form
/// and the ordering of the instant the same thing, so a reader comparing two lines of a file reaches
/// the answer the compiler reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment(Timestamp);

impl Moment {
    /// Parses one.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        let reject = |reason: &str| {
            ParseError::reference(
                "instant",
                value,
                format!("{reason}; instants are written `YYYY-MM-DDTHH:MM:SSZ`, in UTC"),
            )
        };
        let (date, clock) = value.split_once('T').ok_or_else(|| reject("has no `T`"))?;
        let clock = clock
            .strip_suffix('Z')
            .ok_or_else(|| reject("does not end in `Z`"))?;
        let date = CivilDate::parse(date).map_err(|_| reject("names no date"))?;
        let parts: Vec<&str> = clock.split(':').collect();
        let [hours, minutes, seconds] = parts.as_slice() else {
            return Err(reject("names no time of day"));
        };
        let number = |part: &str, limit: u64| -> Result<u64, ParseError> {
            if part.len() != 2 {
                return Err(reject("is not zero-padded to two digits"));
            }
            let value = part
                .parse::<u64>()
                .map_err(|_| reject("has a time of day that is not a number"))?;
            if value >= limit {
                return Err(reject("has a time of day outside the clock"));
            }
            Ok(value)
        };
        let seconds_into_day =
            number(hours, 24)? * 3600 + number(minutes, 60)? * 60 + number(seconds, 60)?;
        Ok(Self(Timestamp::from_epoch_millis(
            date.to_timestamp().epoch_millis() + seconds_into_day * 1000,
        )))
    }
}

impl Moment {
    /// How much of the file's own timeline sits between an earlier instant and this one.
    ///
    /// What makes `at:` load-bearing for the first time. It still reaches no runner — see the
    /// module documentation — but it is now the thing a duration claim is held against, so a
    /// scenario cannot say "twenty seconds" in one line and "five" in the two lines that bracket it.
    ///
    /// Saturating, and the arm never runs: the timeline's instants strictly ascend, and a window is
    /// only ever measured from one an earlier act marked. Written this way rather than with an
    /// `unwrap` because an underflow here would be a wrong number rather than a panic.
    fn since(self, opened: Self) -> Elapsed {
        let millis = self
            .0
            .epoch_millis()
            .saturating_sub(opened.0.epoch_millis());
        // Both instants are whole seconds, so the division is exact, and the cast cannot lose:
        // `Moment::parse` builds from a `CivilDate` and a time of day inside one clock.
        Elapsed::seconds(u32::try_from(millis / 1000).unwrap_or(u32::MAX))
    }
}

impl fmt::Display for Moment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.iso_8601())
    }
}

impl<'de> serde::Deserialize<'de> for Moment {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

/// A value as an author writes it: a literal, or a reference to something the run produced.
///
/// The authored spelling of [`ScenarioValue`], and the tagging argument is that type's own: a
/// declared struct may have a field called `instance`, so an untagged mapping would be read as a
/// reference. `$` cannot begin an ESS field name, so the sigil is a tag no document can collide
/// with, and a literal is written exactly as the model's own documents write one.
#[derive(Debug, Clone, PartialEq)]
pub enum Written {
    /// A value the author chose.
    Literal(Node),
    /// The identity bound under this name earlier in the scenario.
    Instance(InstanceName),
    /// Whatever this event carried in this field, earlier in this scenario.
    Observed {
        /// The event that published it.
        event: String,
        /// The field of its payload.
        field: String,
    },
}

impl Written {
    /// The sigil that begins a reference rather than a value.
    pub const INSTANCE: &'static str = "$instance";
    /// The sigil that names a value the run itself produced.
    pub const OBSERVED: &'static str = "$observed";

    /// The literal it carries, where it is one.
    fn literal(&self) -> Option<&Node> {
        match self {
            Self::Literal(node) => Some(node),
            _ => None,
        }
    }
}

impl<'de> serde::Deserialize<'de> for Written {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let node = Node::deserialize(deserializer)?;
        let Some((key, value)) = node.as_single_entry() else {
            return Ok(Self::Literal(node));
        };
        match key {
            Self::INSTANCE => {
                let name = value.as_text().ok_or_else(|| {
                    serde::de::Error::custom(format!("`{}` names an instance", Self::INSTANCE))
                })?;
                InstanceName::new(name)
                    .map(Self::Instance)
                    .map_err(serde::de::Error::custom)
            }
            Self::OBSERVED => {
                let entries = value.as_map().ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "`{}` is written `{{event: …, field: …}}`",
                        Self::OBSERVED
                    ))
                })?;
                let text = |name: &str| {
                    entries
                        .get(name)
                        .and_then(Node::as_text)
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| {
                            serde::de::Error::custom(format!(
                                "`{}` names no `{name}`",
                                Self::OBSERVED
                            ))
                        })
                };
                Ok(Self::Observed {
                    event: text("event")?,
                    field: text("field")?,
                })
            }
            _ => Ok(Self::Literal(node)),
        }
    }
}

// ---- the result --------------------------------------------------------------------------------

/// Everything a set of authored files compiled to, and everything it did not.
///
/// Both halves in one value, for the reason [`Synthesis`](crate::Synthesis) states: a caller that
/// wants only the scenarios is a caller that has decided the refused ones do not matter, and making
/// that take a second line of code is the point.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Authoring {
    /// The scenarios that compiled, by the id they are filed under.
    pub scenarios: BTreeMap<ScenarioId, ConformanceScenario>,
    /// Every file that produced none, and why.
    pub refusals: Vec<Refusal>,
}

impl Authoring {
    /// `true` when every authored file produced a scenario.
    pub fn is_complete(&self) -> bool {
        self.refusals.is_empty()
    }

    /// Every refusal carrying this code.
    pub fn refused(&self, code: Code) -> impl Iterator<Item = &Refusal> {
        self.refusals
            .iter()
            .filter(move |refusal| refusal.code() == code)
    }
}

/// One authored scenario that got no place in the suite, and why.
///
/// The shape §36 asks of a synthesis refusal, said about a file instead of a construct: a stable
/// code, a structured body, and the thing that caused it — which here is a name somebody typed, so
/// the file it was typed in is part of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The file it was read from.
    pub origin: String,
    /// The scenario the file would have produced, where it said enough to name one.
    pub scenario: Option<ScenarioId>,
    /// Why, as fields rather than as a sentence.
    pub cause: Cause,
}

impl Refusal {
    /// Its stable code.
    pub fn code(&self) -> Code {
        self.cause.code()
    }

    /// What would have to change for the scenario to compile.
    pub fn hint(&self) -> &'static str {
        self.cause.hint()
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scenario {
            Some(id) => writeln!(f, "refusal[{}]: `{id}` in {}", self.code(), self.origin),
            None => writeln!(f, "refusal[{}]: {}", self.code(), self.origin),
        }?;
        for line in self.cause.to_string().lines() {
            writeln!(f, "  {line}")?;
        }
        write!(f, "  help: {}", self.hint())
    }
}

/// Which surface a value was written against, for a message that names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Surface {
    /// A command's input.
    Input(CommandRef),
    /// An event's payload.
    Payload(EventRef),
    /// A declared error's fields.
    Error(ErrorRef),
    /// A view's rows.
    Row(ViewRef),
    /// A view's declared parameters.
    Params(ViewRef),
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(command) => write!(f, "the input of `{command}`"),
            Self::Payload(event) => write!(f, "the payload of `{event}`"),
            Self::Error(error) => write!(f, "the fields of `{error}`"),
            Self::Row(view) => write!(f, "a row of `{view}`"),
            Self::Params(view) => write!(f, "the parameters of `{view}`"),
        }
    }
}

/// Why one authored scenario did not compile.
///
/// One variant per distinct way a scenario can fail to typecheck against the model, because a code
/// is what a harness matches on and a repair instruction that covers two mistakes is a repair
/// instruction for neither.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cause {
    /// The file is not a document this format can read.
    Unreadable {
        /// What the reader said.
        detail: String,
    },
    /// The document claims a format this build does not implement.
    UnsupportedFormat {
        /// What it claims.
        found: String,
    },
    /// Two files produce the same scenario id.
    Duplicate {
        /// The other file.
        first: String,
    },
    /// The model declares no such bounded context.
    UndeclaredDomain {
        /// The name written.
        domain: String,
        /// What the model declares.
        declared: Vec<String>,
    },
    /// The model declares no such entity.
    UndeclaredEntity {
        /// The name written.
        entity: String,
    },
    /// The model declares no such command.
    UndeclaredCommand {
        /// The name written.
        command: String,
    },
    /// That command declares no such branch.
    UndeclaredOutcome {
        /// The command.
        command: CommandRef,
        /// The branch written.
        outcome: String,
        /// The branches it declares.
        declared: Vec<String>,
    },
    /// The model declares no such actor.
    UndeclaredActor {
        /// The name written.
        actor: String,
    },
    /// The actor is declared, and the specification does not grant it this command.
    ActorMayNot {
        /// Who.
        actor: ActorRef,
        /// What.
        command: CommandRef,
    },
    /// The model declares no such event.
    UndeclaredEvent {
        /// The name written.
        event: String,
    },
    /// The model declares no such error.
    UndeclaredError {
        /// The name written.
        error: String,
    },
    /// The model declares no such view.
    UndeclaredView {
        /// The name written.
        view: String,
    },
    /// A value is supplied under a name the surface does not declare.
    UndeclaredField {
        /// Where.
        surface: Surface,
        /// The path within it; empty at its root.
        at: String,
        /// The name written.
        field: String,
    },
    /// A declared field that has to be supplied is not.
    MissingField {
        /// Where.
        surface: Surface,
        /// The path within it; empty at its root.
        at: String,
        /// The name of what is missing.
        field: String,
    },
    /// A value is not of the shape its declared type calls for.
    ValueRejected {
        /// Where.
        surface: Surface,
        /// The reader's own account of it.
        detail: String,
    },
    /// A value names something a declared enum does not have as a variant.
    UndeclaredVariant {
        /// Where.
        surface: Surface,
        /// The declared type.
        declared_by: DeclaredTypeRef,
        /// The name written.
        value: String,
        /// What the model declares.
        variants: Vec<String>,
    },
    /// A value names a lifecycle state the entity does not declare.
    UndeclaredState {
        /// Whose lifecycle.
        entity: EntityRef,
        /// The name written.
        state: String,
        /// The states it declares.
        declared: Vec<String>,
    },
    /// A reference names an instance the arrangement does not declare.
    UnarrangedInstance {
        /// The name written.
        instance: InstanceName,
        /// What the arrangement declares.
        declared: Vec<String>,
    },
    /// A reference names an instance nothing has bound yet.
    UnboundInstance {
        /// The name written.
        instance: InstanceName,
    },
    /// A reference reads an event no earlier act required.
    Unobserved {
        /// The event.
        event: EventRef,
    },
    /// A reference sits where the suite compares values it carries itself.
    NotComparable {
        /// Where.
        surface: Surface,
        /// The field it was written for.
        field: String,
    },
    /// An instance is bound to a field of a type that is not its entity's identity.
    InstanceMistyped {
        /// The reference.
        instance: InstanceName,
        /// Whose it is.
        entity: EntityRef,
        /// Where it was written.
        surface: Surface,
        /// The field.
        field: String,
        /// The type that field declares.
        declared: String,
        /// The type the entity's identity has.
        identity: String,
    },
    /// The timeline's instants do not strictly ascend.
    UnorderedTimeline {
        /// The instant that goes backwards.
        at: Moment,
        /// The one before it.
        after: Moment,
    },
    /// A position or an order is asserted of a view that declares no order.
    Unordered {
        /// Which view.
        view: ViewRef,
    },
    /// An assertion states other than exactly one claim.
    AmbiguousClaim {
        /// Which view.
        view: ViewRef,
        /// The claims it states.
        stated: Vec<&'static str>,
    },
    /// A predicate reads something the view does not publish.
    UnreadablePredicate {
        /// Which view.
        view: ViewRef,
        /// The path it reads.
        path: String,
    },
    /// The scenario runs no command and asserts nothing.
    NothingHappens,
    /// A window is measured from an instant nothing marked before it.
    UnmarkedInstant {
        /// The name written.
        instant: InstantName,
        /// What earlier acts have marked.
        marked: Vec<String>,
    },
    /// Two acts mark the same instant.
    DuplicateInstant {
        /// The name written twice.
        instant: InstantName,
        /// The instant the first one named.
        first: Moment,
    },
    /// A window has no width.
    VacuousWindow {
        /// The instant it would have opened at.
        instant: InstantName,
    },
    /// A bounded negative forbids no occurrence.
    QuietAboutNothing {
        /// The instant the window opens at.
        instant: InstantName,
        /// How long it stays open.
        duration: Elapsed,
    },
    /// The timeline's own instants say something the window contradicts.
    WindowContradictsTimeline {
        /// The instant the window opens at.
        instant: InstantName,
        /// The bound the window states.
        stated: &'static str,
        /// The length it states.
        elapsed: Elapsed,
        /// What the file's own instants put between the two acts.
        written: Elapsed,
    },
    /// A window states other than exactly one bound.
    AmbiguousWindow {
        /// The instant it opens at.
        instant: InstantName,
        /// The bounds it states.
        stated: Vec<&'static str>,
    },
}

impl Cause {
    /// The family every refusal here belongs to.
    const FAMILY: &'static str = "AUTHOR";

    /// Its stable code.
    ///
    /// Derived from the variant rather than stored beside it, so a code cannot come to name a body
    /// other than its own.
    pub fn code(&self) -> Code {
        Code::new(
            Self::FAMILY,
            match self {
                Self::Unreadable { .. } => 1,
                Self::UnsupportedFormat { .. } => 2,
                Self::Duplicate { .. } => 3,
                Self::UndeclaredDomain { .. } => 4,
                Self::UndeclaredEntity { .. } => 5,
                Self::UndeclaredCommand { .. } => 6,
                Self::UndeclaredOutcome { .. } => 7,
                Self::UndeclaredActor { .. } => 8,
                Self::ActorMayNot { .. } => 9,
                Self::UndeclaredEvent { .. } => 10,
                Self::UndeclaredError { .. } => 11,
                Self::UndeclaredView { .. } => 12,
                Self::UndeclaredField { .. } => 13,
                Self::MissingField { .. } => 14,
                Self::ValueRejected { .. } => 15,
                Self::UndeclaredVariant { .. } => 16,
                Self::UndeclaredState { .. } => 17,
                Self::UnarrangedInstance { .. } => 18,
                Self::UnboundInstance { .. } => 19,
                Self::Unobserved { .. } => 20,
                Self::NotComparable { .. } => 21,
                Self::InstanceMistyped { .. } => 22,
                Self::UnorderedTimeline { .. } => 23,
                Self::Unordered { .. } => 24,
                Self::AmbiguousClaim { .. } => 25,
                Self::UnreadablePredicate { .. } => 26,
                Self::NothingHappens => 27,
                Self::UnmarkedInstant { .. } => 28,
                Self::DuplicateInstant { .. } => 29,
                Self::VacuousWindow { .. } => 30,
                Self::QuietAboutNothing { .. } => 31,
                Self::WindowContradictsTimeline { .. } => 32,
                Self::AmbiguousWindow { .. } => 33,
            },
        )
    }

    /// What would have to change for the scenario to compile.
    ///
    /// One arm per cause, and long because there are thirty-three of them — the same argument
    /// [`Display`](fmt::Display) makes below. A reader comparing two repairs reads them side by
    /// side or not at all, and splitting the list would put half of it somewhere else.
    #[allow(clippy::too_many_lines)]
    pub fn hint(&self) -> &'static str {
        match self {
            Self::Unreadable { .. } => {
                "the document is YAML with the keys `type`, `domain`, `scenario` and `summary`; a \
                 key it does not know is refused rather than ignored"
            }
            Self::UnsupportedFormat { .. } => "write `type: ess-scenario/1`",
            Self::Duplicate { .. } => {
                "two files name one scenario in one domain; rename one of them"
            }
            Self::UndeclaredDomain { .. } => {
                "name a bounded context the specification declares, or add it to the model"
            }
            Self::UndeclaredEntity { .. } => {
                "name an entity the specification declares; an authored scenario acts on the model's \
                 own instances and invents none"
            }
            Self::UndeclaredCommand { .. } => {
                "name a command the specification declares; a scenario that invokes anything else is \
                 checking a system this model does not describe"
            }
            Self::UndeclaredOutcome { .. } => {
                "name one of the branches the command declares, or declare the branch you meant"
            }
            Self::UndeclaredActor { .. } => "name an actor the specification declares, or drop `actor:`",
            Self::ActorMayNot { .. } => {
                "grant the command to this actor with `may:`, or act as one that already has it"
            }
            Self::UndeclaredEvent { .. } => "name an event the specification declares",
            Self::UndeclaredError { .. } => "name a declared error the specification declares",
            Self::UndeclaredView { .. } => "name a view the specification declares",
            Self::UndeclaredField { .. } => {
                "name a field the construct declares; the scenario and the model disagree about \
                 what it has"
            }
            Self::MissingField { .. } => {
                "supply the field; a command is invoked with all of its input, and one left out is a \
                 call that could not be made"
            }
            Self::ValueRejected { .. } => "write a value of the type the model declares there",
            Self::UndeclaredVariant { .. } => {
                "write one of the variants the enum declares; the set is closed"
            }
            Self::UndeclaredState { .. } => {
                "write one of the states the entity's lifecycle declares"
            }
            Self::UnarrangedInstance { .. } => {
                "declare the instance under `arrange:`, so the scenario says whose it is"
            }
            Self::UnboundInstance { .. } => {
                "capture the instance from an event before the step that names it; a suite carries \
                 no identity of its own"
            }
            Self::Unobserved { .. } => {
                "require the event in an earlier act; a value is read off an occurrence the run \
                 produced, and this scenario has not required one"
            }
            Self::NotComparable { .. } => {
                "write the value the field must hold; an event's payload and an error's fields are \
                 compared against values the suite carries"
            }
            Self::InstanceMistyped { .. } => {
                "bind the instance to a field typed as the entity's identity, or arrange the entity \
                 whose identity this field carries"
            }
            Self::UnorderedTimeline { .. } => {
                "give each act an instant later than the one before it; the file's order is the \
                 scenario's order and `at:` is what states it"
            }
            Self::Unordered { .. } => {
                "declare `order_by:` on the view, or assert `contains:` instead; a position in an \
                 unordered view names a different row on every read"
            }
            Self::AmbiguousClaim { .. } => {
                "state exactly one of `contains`, `excludes`, `counts`, `ranked`, `at` or \
                 `satisfies` per assertion"
            }
            Self::UnreadablePredicate { .. } => {
                "read only fields the view projects, or project the field the predicate reads"
            }
            Self::NothingHappens => {
                "give the scenario a timeline; a scenario that runs nothing is a check that cannot \
                 fail"
            }
            Self::UnmarkedInstant { .. } => {
                "write `mark:` on the earlier act the window opens at; a duration is measured from \
                 an instant somebody named, never from wherever the step before it happened to end"
            }
            Self::DuplicateInstant { .. } => {
                "give the two acts different names; one name for two instants is a window whose \
                 length depends on which one a reader had in mind"
            }
            Self::VacuousWindow { .. } => {
                "give the window a length; a window of no seconds is a claim about no time, and \
                 there is no target it can fail against"
            }
            Self::QuietAboutNothing { .. } => {
                "name the events the window must stay clear of; a bounded negative that forbids \
                 nothing is satisfied by every implementation there is"
            }
            Self::WindowContradictsTimeline { .. } => {
                "move the act's `at:` so the file's own instants agree with the claim, or state \
                 the length the file already shows; the timeline is what a reader believes"
            }
            Self::AmbiguousWindow { .. } => {
                "state exactly one of `not_before`, `within` or `quiet` per window, and write a \
                 second window for a second claim"
            }
        }
    }
}

impl fmt::Display for Cause {
    /// One arm per cause, and long because there are thirty-three of them. Splitting it would put
    /// half the wording somewhere a reader comparing two refusals has to go and find.
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { detail } => write!(f, "unreadable: {detail}"),
            Self::UnsupportedFormat { found } => {
                write!(f, "`{found}` is not `{FORMAT}`")
            }
            Self::Duplicate { first } => {
                write!(f, "the same scenario is already declared in {first}")
            }
            Self::UndeclaredDomain { domain, declared } => write!(
                f,
                "`{domain}` is not a domain this specification declares; it declares {}",
                declared.join(", ")
            ),
            Self::UndeclaredEntity { entity } => {
                write!(f, "`{entity}` is not an entity this specification declares")
            }
            Self::UndeclaredCommand { command } => {
                write!(
                    f,
                    "`{command}` is not a command this specification declares"
                )
            }
            Self::UndeclaredOutcome {
                command,
                outcome,
                declared,
            } => write!(
                f,
                "`{command}` declares no outcome `{outcome}`; it declares {}",
                declared.join(", ")
            ),
            Self::UndeclaredActor { actor } => {
                write!(f, "`{actor}` is not an actor this specification declares")
            }
            Self::ActorMayNot { actor, command } => {
                write!(f, "`{actor}` is not granted `{command}`")
            }
            Self::UndeclaredEvent { event } => {
                write!(f, "`{event}` is not an event this specification declares")
            }
            Self::UndeclaredError { error } => {
                write!(f, "`{error}` is not an error this specification declares")
            }
            Self::UndeclaredView { view } => {
                write!(f, "`{view}` is not a view this specification declares")
            }
            Self::UndeclaredField { surface, at, field } => {
                let at = if at.is_empty() {
                    String::new()
                } else {
                    format!(", at `{at}`")
                };
                write!(f, "{surface} declares no `{field}`{at}")
            }
            Self::MissingField { surface, at, field } => {
                let at = if at.is_empty() {
                    String::new()
                } else {
                    format!(", at `{at}`")
                };
                write!(
                    f,
                    "{surface} requires `{field}`, and nothing supplies it{at}"
                )
            }
            Self::ValueRejected { surface, detail } => {
                write!(f, "{surface}: {detail}")
            }
            Self::UndeclaredVariant {
                surface,
                declared_by,
                value,
                variants,
            } => write!(
                f,
                "{surface}: `{value}` is not a variant of `{declared_by}`; it declares {}",
                variants.join(", ")
            ),
            Self::UndeclaredState {
                entity,
                state,
                declared,
            } => write!(
                f,
                "`{state}` is not a state of `{entity}`; its lifecycle declares {}",
                declared.join(", ")
            ),
            Self::UnarrangedInstance { instance, declared } => {
                let declared = if declared.is_empty() {
                    "nothing".to_owned()
                } else {
                    declared.join(", ")
                };
                write!(
                    f,
                    "`{instance}` is not arranged; `arrange:` declares {declared}"
                )
            }
            Self::UnboundInstance { instance } => write!(
                f,
                "`{instance}` is named before anything binds it; no earlier act captures it"
            ),
            Self::Unobserved { event } => write!(
                f,
                "`{event}` is read before anything requires it in this scenario"
            ),
            Self::NotComparable { surface, field } => write!(
                f,
                "{surface}: `{field}` is written as a reference, and only a value can be compared \
                 there"
            ),
            Self::InstanceMistyped {
                instance,
                entity,
                surface,
                field,
                declared,
                identity,
            } => write!(
                f,
                "{surface}: `{field}` is `{declared}`, and `{instance}` is a `{entity}`, whose \
                 identity is `{identity}`"
            ),
            Self::UnorderedTimeline { at, after } => {
                write!(f, "`{at}` does not come after `{after}`")
            }
            Self::Unordered { view } => {
                write!(f, "`{view}` declares no `order_by:`")
            }
            Self::AmbiguousClaim { view, stated } => {
                if stated.is_empty() {
                    write!(f, "the assertion about `{view}` states no claim")
                } else {
                    write!(
                        f,
                        "the assertion about `{view}` states {} claims: {}",
                        stated.len(),
                        stated.join(", ")
                    )
                }
            }
            Self::UnreadablePredicate { view, path } => {
                write!(f, "`{view}` publishes nothing at `{path}`")
            }
            Self::NothingHappens => f.write_str("the timeline is empty"),
            Self::UnmarkedInstant { instant, marked } => {
                let marked = if marked.is_empty() {
                    "nothing is marked before it".to_owned()
                } else {
                    format!("the instants marked before it are {}", marked.join(", "))
                };
                write!(f, "no earlier act marks the instant `{instant}`; {marked}")
            }
            Self::DuplicateInstant { instant, first } => {
                write!(f, "the instant `{instant}` is already marked, at `{first}`")
            }
            Self::VacuousWindow { instant } => {
                write!(f, "the window opening at `{instant}` is `PT0S` long")
            }
            Self::QuietAboutNothing { instant, duration } => write!(
                f,
                "the window of {duration} after `{instant}` names no event it must stay clear of"
            ),
            Self::WindowContradictsTimeline {
                instant,
                stated,
                elapsed,
                written,
            } => write!(
                f,
                "`{stated}: {elapsed}` since `{instant}`, and the timeline writes {written} \
                 between the two acts"
            ),
            Self::AmbiguousWindow { instant, stated } => {
                if stated.is_empty() {
                    write!(f, "the window opening at `{instant}` states no bound")
                } else {
                    write!(
                        f,
                        "the window opening at `{instant}` states {} bounds: {}",
                        stated.len(),
                        stated.join(", ")
                    )
                }
            }
        }
    }
}

// ---- compilation -------------------------------------------------------------------------------

/// Compiles authored files against the model they are written about.
///
/// Deterministic in both directions: the sources are compiled in the order of their origins, and
/// every collection the result carries is ordered, so two compilations of one set of files produce
/// one set of bytes. A file that does not compile contributes refusals and no scenario — never a
/// half-built one, because a scenario missing the step that could not be compiled is a check that
/// passes for the wrong reason.
pub fn compile(ir: &EssIr, sources: &[Source]) -> Authoring {
    let mut ordered: Vec<&Source> = sources.iter().collect();
    ordered.sort_by(|left, right| left.origin.cmp(&right.origin));

    let mut authoring = Authoring::default();
    let mut seen: BTreeMap<ScenarioId, String> = BTreeMap::new();
    for source in ordered {
        match compile_one(ir, source, &seen) {
            Ok((id, scenario)) => {
                seen.insert(id.clone(), source.origin.clone());
                authoring.scenarios.insert(id, scenario);
            }
            Err(refusals) => authoring.refusals.extend(refusals),
        }
    }
    authoring
}

/// One file, or every reason it produced nothing.
fn compile_one(
    ir: &EssIr,
    source: &Source,
    seen: &BTreeMap<ScenarioId, String>,
) -> Result<(ScenarioId, ConformanceScenario), Vec<Refusal>> {
    let bare = |cause: Cause| {
        vec![Refusal {
            origin: source.origin.clone(),
            scenario: None,
            cause,
        }]
    };

    let document: Document = serde_yaml::from_str(&source.text).map_err(|error| {
        bare(Cause::Unreadable {
            detail: error.to_string(),
        })
    })?;
    if document.format != FORMAT {
        return Err(bare(Cause::UnsupportedFormat {
            found: document.format,
        }));
    }
    let Ok(domain) = QualifiedName::new(&document.domain) else {
        return Err(bare(undeclared_domain(ir, &document.domain)));
    };
    if !ir.domains().contains_key(&domain) {
        return Err(bare(undeclared_domain(ir, &document.domain)));
    }
    let id = ScenarioId::Authored {
        domain: DomainRef::new(domain),
        name: document.scenario.clone(),
    };
    if let Some(first) = seen.get(&id) {
        return Err(vec![Refusal {
            origin: source.origin.clone(),
            scenario: Some(id),
            cause: Cause::Duplicate {
                first: first.clone(),
            },
        }]);
    }

    let mut compiler = Compiler {
        ir,
        origin: &source.origin,
        id: id.clone(),
        refusals: Vec::new(),
        arranged: BTreeMap::new(),
        bound: BTreeSet::new(),
        observed: BTreeSet::new(),
        marked: BTreeMap::new(),
        steps: Vec::new(),
        source: BTreeSet::new(),
        types: BTreeSet::new(),
    };
    compiler.run(&document);
    if !compiler.refusals.is_empty() {
        return Err(compiler.refusals);
    }

    let mut dependencies = compiler.source;
    dependencies.insert(EssSemanticRef::from(match &id {
        ScenarioId::Authored { domain, .. } => domain.clone(),
        // `id` is built two statements above and is nothing else.
        other => unreachable!("`{other}` is an authored scenario id"),
    }));
    dependencies.extend(compiler.types.into_iter().map(EssSemanticRef::from));
    Ok((
        id,
        ConformanceScenario::new(document.summary, compiler.steps, dependencies),
    ))
}

/// The refusal for a domain the model does not declare, with the ones it does.
fn undeclared_domain(ir: &EssIr, written: &str) -> Cause {
    Cause::UndeclaredDomain {
        domain: written.to_owned(),
        declared: ir.domains().keys().map(ToString::to_string).collect(),
    }
}

/// One scenario in flight: what it has resolved, bound and built so far.
struct Compiler<'a> {
    ir: &'a EssIr,
    origin: &'a str,
    id: ScenarioId,
    refusals: Vec<Refusal>,
    /// The entity each arranged instance belongs to.
    arranged: BTreeMap<InstanceName, EntityRef>,
    /// The instances an earlier act has captured.
    bound: BTreeSet<InstanceName>,
    /// The events an earlier act has required.
    observed: BTreeSet<EventRef>,
    /// The instant each earlier act marked, and where the file puts it.
    marked: BTreeMap<InstantName, Moment>,
    steps: Vec<ScenarioStep>,
    source: BTreeSet<EssSemanticRef>,
    types: BTreeSet<DeclaredTypeRef>,
}

impl Compiler<'_> {
    /// Records a refusal about the scenario being compiled.
    fn refuse(&mut self, cause: Cause) {
        self.refusals.push(Refusal {
            origin: self.origin.to_owned(),
            scenario: Some(self.id.clone()),
            cause,
        });
    }

    /// Resolves a written name, refusing it by name when the model declares nothing under it.
    fn declared<'ir, T>(
        &mut self,
        written: &str,
        table: &'ir BTreeMap<QualifiedName, T>,
        cause: impl FnOnce(String) -> Cause,
    ) -> Option<(QualifiedName, &'ir T)> {
        let resolved = QualifiedName::new(written)
            .ok()
            .and_then(|name| table.get_key_value(&name));
        let Some((name, declared)) = resolved else {
            self.refuse(cause(written.to_owned()));
            return None;
        };
        Some((name.clone(), declared))
    }

    /// Every declared type the fields reach, so the scenario's `source` names what it depends on.
    fn reach(&mut self, fields: &[ResolvedField]) {
        for field in fields {
            reachable_types(self.ir, &field.type_ref, &mut self.types);
        }
    }

    /// Compiles the whole document.
    fn run(&mut self, document: &Document) {
        for arrangement in &document.arrange {
            let Some((name, _)) =
                self.declared(&arrangement.entity, self.ir.entities(), |entity| {
                    Cause::UndeclaredEntity { entity }
                })
            else {
                continue;
            };
            let entity = EntityRef::new(name);
            self.source.insert(entity.clone().into());
            self.arranged.insert(arrangement.instance.clone(), entity);
        }

        let mut previous: Option<Moment> = None;
        for act in &document.timeline {
            if let Some(after) = previous {
                if act.at <= after {
                    self.refuse(Cause::UnorderedTimeline { at: act.at, after });
                }
            }
            previous = Some(act.at);
            self.act(act);
        }
        for assertion in &document.assert {
            self.assertion(assertion);
        }
        if document.timeline.is_empty() {
            self.refuse(Cause::NothingHappens);
        }
    }

    /// One act: the command, what it must answer, and what it binds.
    fn act(&mut self, act: &Act) {
        // The windows come first, and that is the semantics rather than a filing decision: a claim
        // about the time before an act is a claim the act is not allowed to be the cause of. Emitted
        // after `ExecuteCommand`, "not before twenty seconds" would be checked against a system the
        // step it is guarding had already changed.
        for window in &act.elapsed {
            self.window(window, act.at);
        }
        let Some((name, command)) = self.declared(&act.command, self.ir.commands(), |command| {
            Cause::UndeclaredCommand { command }
        }) else {
            self.mark(act);
            return;
        };
        let command_ref = CommandRef::new(name.clone());
        self.source.insert(command_ref.clone().into());
        // Cloned out of the IR so the borrow ends here: every refusal below takes `&mut self`.
        let input_fields = command.input.clone();
        let outcomes: Vec<OutcomeName> =
            command.outcomes.iter().map(|it| it.name.clone()).collect();
        self.reach(&input_fields);

        let actor = act
            .actor
            .as_ref()
            .and_then(|written| self.actor(written, &command_ref));
        let input = self.values(
            &act.input,
            &input_fields,
            &Surface::Input(command_ref.clone()),
            Completeness::Total,
        );
        self.steps.push(ScenarioStep::ExecuteCommand {
            command: command_ref.clone(),
            actor,
            input,
        });

        if let Some(written) = &act.outcome {
            match OutcomeName::new(written)
                .ok()
                .filter(|outcome| outcomes.contains(outcome))
            {
                Some(outcome) => {
                    let reference = OutcomeRef::new(command_ref.clone(), outcome);
                    self.source.insert(reference.clone().into());
                    self.steps
                        .push(ScenarioStep::ExpectOutcome { outcome: reference });
                }
                None => self.refuse(Cause::UndeclaredOutcome {
                    command: command_ref.clone(),
                    outcome: written.clone(),
                    declared: outcomes.iter().map(ToString::to_string).collect(),
                }),
            }
        }

        if let Some(claim) = &act.error {
            self.error(claim);
        }
        for claim in &act.events {
            self.event(claim);
        }
        for written in &act.no_events {
            if let Some((name, _)) = self.declared(written, self.ir.events(), |event| {
                Cause::UndeclaredEvent { event }
            }) {
                let event = EventRef::new(name);
                self.source.insert(event.clone().into());
                self.steps.push(ScenarioStep::ExpectNoEvent { event });
            }
        }
        if let Some(capture) = &act.capture {
            self.capture(capture);
        }
        self.mark(act);
    }

    /// Names this act's instant, so a later window can open at it.
    ///
    /// After everything else the act does, which is what stops a window opening at the act that
    /// carries it: the two would be the same instant, and a window of no width is a claim that
    /// cannot fail.
    fn mark(&mut self, act: &Act) {
        let Some(instant) = &act.mark else {
            return;
        };
        if let Some(first) = self.marked.get(instant).copied() {
            self.refuse(Cause::DuplicateInstant {
                instant: instant.clone(),
                first,
            });
            return;
        }
        self.marked.insert(instant.clone(), act.at);
        self.steps.push(ScenarioStep::MarkInstant {
            instant: instant.clone(),
        });
    }

    /// One claim about the time between a marked instant and this act.
    ///
    /// # Why the file's own instants are checked against the claim
    ///
    /// `at:` used to order the file and nothing else, and that made every duration in a scenario a
    /// gap between two comments. It still reaches no runner — a suite that carried an instant would
    /// mean something different on a machine with a different clock — but a claim that contradicts
    /// the gap the author wrote is a document that says two things, and the reader believes the
    /// timeline. So the two are held to each other here, at compile time, where the disagreement
    /// costs a refusal instead of a wrong verdict.
    fn window(&mut self, window: &Window, at: Moment) {
        let stated = window.stated();
        if stated.len() != 1 {
            self.refuse(Cause::AmbiguousWindow {
                instant: window.since.clone(),
                stated,
            });
            return;
        }
        // `stated` has exactly one entry, so `length` has a value.
        let Some(elapsed) = window.length() else {
            return;
        };
        if elapsed.is_empty() {
            self.refuse(Cause::VacuousWindow {
                instant: window.since.clone(),
            });
            return;
        }
        let Some(opened) = self.marked.get(&window.since).copied() else {
            self.refuse(Cause::UnmarkedInstant {
                instant: window.since.clone(),
                marked: self.marked.keys().map(ToString::to_string).collect(),
            });
            return;
        };
        let written = at.since(opened);
        // `not_before` and `quiet` both need the window closed by the time the act runs; `within`
        // needs the act inside it. One comparison per bound, against the gap the file writes.
        let contradicted = match stated[0] {
            "within" => written.get() > elapsed.get(),
            _ => written.get() < elapsed.get(),
        };
        if contradicted {
            self.refuse(Cause::WindowContradictsTimeline {
                instant: window.since.clone(),
                stated: stated[0],
                elapsed,
                written,
            });
            return;
        }

        if let Some(quiet) = &window.quiet {
            if quiet.events.is_empty() {
                self.refuse(Cause::QuietAboutNothing {
                    instant: window.since.clone(),
                    duration: quiet.duration,
                });
                return;
            }
            for written in &quiet.events {
                if let Some((name, _)) = self.declared(written, self.ir.events(), |event| {
                    Cause::UndeclaredEvent { event }
                }) {
                    let event = EventRef::new(name);
                    self.source.insert(event.clone().into());
                    self.steps.push(ScenarioStep::ExpectQuiet {
                        event,
                        instant: window.since.clone(),
                        elapsed: quiet.duration,
                    });
                }
            }
            return;
        }
        self.steps.push(if window.within.is_some() {
            ScenarioStep::ExpectWithin {
                instant: window.since.clone(),
                elapsed,
            }
        } else {
            ScenarioStep::ExpectNotBefore {
                instant: window.since.clone(),
                elapsed,
            }
        });
    }

    /// The actor an act runs as, where the specification grants it the command.
    fn actor(&mut self, written: &str, command: &CommandRef) -> Option<ActorRef> {
        let (name, declared) = self.declared(written, self.ir.actors(), |actor| {
            Cause::UndeclaredActor { actor }
        })?;
        let granted = declared
            .may
            .iter()
            .any(|handle| handle.name() == command.name());
        let actor = ActorRef::new(name);
        if !granted {
            self.refuse(Cause::ActorMayNot {
                actor: actor.clone(),
                command: command.clone(),
            });
            return None;
        }
        self.source.insert(actor.clone().into());
        Some(actor)
    }

    /// The declared error a branch reports, and the fields to compare.
    fn error(&mut self, claim: &ErrorClaim) {
        let Some((name, declared)) = self.declared(&claim.name, self.ir.errors(), |error| {
            Cause::UndeclaredError { error }
        }) else {
            return;
        };
        let error = ErrorRef::new(name);
        let fields = declared.fields.clone();
        self.reach(&fields);
        self.source.insert(error.clone().into());
        let compared = self.literals(&claim.fields, &fields, &Surface::Error(error.clone()));
        self.steps.push(ScenarioStep::ExpectError {
            error,
            fields: compared,
        });
    }

    /// An occurrence an act must publish, with the declared shape every occurrence carries.
    fn event(&mut self, claim: &EventClaim) {
        let Some((name, declared)) = self.declared(&claim.event, self.ir.events(), |event| {
            Cause::UndeclaredEvent { event }
        }) else {
            return;
        };
        let event = EventRef::new(name);
        let fields = declared.fields.clone();
        self.reach(&fields);
        self.source.insert(event.clone().into());
        let payload = self.literals(&claim.payload, &fields, &Surface::Payload(event.clone()));
        let shape = payload_shape(self.ir, &event);
        self.observed.insert(event.clone());
        self.steps.push(ScenarioStep::ExpectEvent {
            event,
            payload,
            shape,
        });
    }

    /// Binding an identity an act published.
    fn capture(&mut self, capture: &Capture) {
        let Some(entity) = self.arranged.get(&capture.instance).cloned() else {
            let declared = self.arranged.keys().map(ToString::to_string).collect();
            self.refuse(Cause::UnarrangedInstance {
                instance: capture.instance.clone(),
                declared,
            });
            return;
        };
        let Some((name, declared)) = self.declared(&capture.event, self.ir.events(), |event| {
            Cause::UndeclaredEvent { event }
        }) else {
            return;
        };
        let event = EventRef::new(name);
        if !declared.fields.iter().any(|it| it.name == capture.field) {
            self.refuse(Cause::UndeclaredField {
                surface: Surface::Payload(event),
                at: String::new(),
                field: capture.field.clone(),
            });
            return;
        }
        self.source.insert(event.clone().into());
        self.bound.insert(capture.instance.clone());
        self.steps.push(ScenarioStep::CaptureInstance {
            instance: capture.instance.clone(),
            entity,
            event,
            field: capture.field.clone(),
        });
    }

    /// One claim about one view.
    fn assertion(&mut self, assertion: &Assertion) {
        let Some((name, declared)) = self.declared(&assertion.view, self.ir.views(), |view| {
            Cause::UndeclaredView { view }
        }) else {
            return;
        };
        let view = ViewRef::new(name);
        let fields = declared.fields.clone();
        let params = declared.params.clone();
        let order_by = declared.order_by.clone();
        let style = declared.assertion_style;
        self.reach(&fields);
        self.source.insert(view.clone().into());

        let stated = assertion.stated();
        if stated.len() != 1 {
            self.refuse(Cause::AmbiguousClaim {
                view: view.clone(),
                stated,
            });
            return;
        }
        let bound = self.values(
            &assertion.params,
            &params,
            &Surface::Params(view.clone()),
            Completeness::Total,
        );
        let Some(expectation) = self.expectation(assertion, &view, &fields, &order_by) else {
            return;
        };
        match style {
            AssertionStyle::Expect => {
                self.steps.push(ScenarioStep::QueryView {
                    view: view.clone(),
                    params: bound,
                });
                self.steps
                    .push(ScenarioStep::ExpectView { view, expectation });
            }
            AssertionStyle::Eventually => self.steps.push(ScenarioStep::EventuallyView {
                view,
                params: bound,
                expectation,
            }),
        }
    }

    /// The one claim an assertion states, in the suite's own vocabulary.
    fn expectation(
        &mut self,
        assertion: &Assertion,
        view: &ViewRef,
        fields: &[ResolvedField],
        order_by: &[Ranking],
    ) -> Option<ViewExpectation> {
        let surface = Surface::Row(view.clone());
        if let Some(written) = &assertion.contains {
            return Some(ViewExpectation::Contains {
                fields: self.values(written, fields, &surface, Completeness::Partial),
            });
        }
        if let Some(written) = &assertion.excludes {
            return Some(ViewExpectation::Excludes {
                fields: self.values(written, fields, &surface, Completeness::Partial),
            });
        }
        if let Some(counts) = &assertion.counts {
            return Some(ViewExpectation::Counts {
                at_least: counts.at_least,
                at_most: counts.at_most,
            });
        }
        if let Some(predicate) = &assertion.satisfies {
            for path in predicate.fact_paths() {
                if !resolve_path(self.ir, fields, path).is_scalar() {
                    self.refuse(Cause::UnreadablePredicate {
                        view: view.clone(),
                        path: path.to_string(),
                    });
                    return None;
                }
            }
            return Some(ViewExpectation::Satisfies {
                predicate: predicate.clone(),
            });
        }
        // The last two are claims about an order, and the order is the view's. A view that declares
        // none makes both meaningless: "the rows are in order" is satisfied by any order, and "the
        // first row" is a different row on every read.
        if order_by.is_empty() {
            self.refuse(Cause::Unordered { view: view.clone() });
            return None;
        }
        if assertion.ranked == Some(true) {
            return Some(ViewExpectation::Ranked {
                order_by: order_by.to_vec(),
            });
        }
        let at = assertion.at.as_ref()?;
        Some(ViewExpectation::At {
            order_by: order_by.to_vec(),
            position: at.row.position(),
            fields: self.values(&at.fields, fields, &surface, Completeness::Partial),
        })
    }

    /// Values and references, checked against what the surface declares.
    fn values(
        &mut self,
        written: &BTreeMap<String, Written>,
        fields: &[ResolvedField],
        surface: &Surface,
        completeness: Completeness,
    ) -> BTreeMap<String, ScenarioValue> {
        let mut resolved = BTreeMap::new();
        let mut literals = BTreeMap::new();
        let mut referenced = BTreeSet::new();

        for (field, value) in written {
            match value {
                Written::Literal(node) => {
                    literals.insert(field.clone(), node.clone());
                }
                Written::Instance(instance) => {
                    referenced.insert(field.clone());
                    if let Some(value) = self.instance(instance, fields, surface, field) {
                        resolved.insert(field.clone(), value);
                    }
                }
                Written::Observed { event, field: read } => {
                    referenced.insert(field.clone());
                    if let Some(value) = self.observed(event, read) {
                        resolved.insert(field.clone(), value);
                    }
                }
            }
        }
        // A reference occupies a declared field as much as a value does, so the completeness check
        // below is run against the fields it does not occupy — otherwise every instance-valued input
        // would also be reported missing.
        let remaining: Vec<ResolvedField> = fields
            .iter()
            .filter(|field| !referenced.contains(&field.name))
            .cloned()
            .collect();
        for field in &referenced {
            if !fields.iter().any(|declared| &declared.name == field) {
                self.refuse(Cause::UndeclaredField {
                    surface: surface.clone(),
                    at: String::new(),
                    field: field.clone(),
                });
            }
        }
        if let Err(errors) = bind(self.ir, &remaining, &literals, completeness) {
            self.shape(&errors, surface);
        }
        for (field, node) in literals {
            resolved.insert(field, ScenarioValue::literal(node));
        }
        resolved
    }

    /// The literal half only, for a step that compares values the suite carries.
    fn literals(
        &mut self,
        written: &BTreeMap<String, Written>,
        fields: &[ResolvedField],
        surface: &Surface,
    ) -> BTreeMap<String, Node> {
        let mut literals = BTreeMap::new();
        for (field, value) in written {
            match value.literal() {
                Some(node) => {
                    literals.insert(field.clone(), node.clone());
                }
                None => self.refuse(Cause::NotComparable {
                    surface: surface.clone(),
                    field: field.clone(),
                }),
            }
        }
        if let Err(errors) = bind(self.ir, fields, &literals, Completeness::Partial) {
            self.shape(&errors, surface);
        }
        literals
    }

    /// An instance reference, checked against the arrangement and the field it fills.
    fn instance(
        &mut self,
        instance: &InstanceName,
        fields: &[ResolvedField],
        surface: &Surface,
        field: &str,
    ) -> Option<ScenarioValue> {
        let Some(entity) = self.arranged.get(instance).cloned() else {
            let declared = self.arranged.keys().map(ToString::to_string).collect();
            self.refuse(Cause::UnarrangedInstance {
                instance: instance.clone(),
                declared,
            });
            return None;
        };
        if !self.bound.contains(instance) {
            self.refuse(Cause::UnboundInstance {
                instance: instance.clone(),
            });
            return None;
        }
        // The identity has a declared type, so a field that cannot hold one is a mistake the model
        // can see: `PayInvoice` takes an invoice id and an amount, and binding the invoice to the
        // amount is a scenario nothing would ever have executed.
        if let Some(declared) = fields.iter().find(|it| it.name == field) {
            let identity = self.ir.entity_identity(&entity);
            if let Some(identity) = identity {
                if declared.type_ref.required() != identity.required() {
                    self.refuse(Cause::InstanceMistyped {
                        instance: instance.clone(),
                        entity,
                        surface: surface.clone(),
                        field: field.to_owned(),
                        declared: declared.type_ref.to_string(),
                        identity: identity.to_string(),
                    });
                    return None;
                }
            }
        }
        Some(ScenarioValue::instance(instance.clone()))
    }

    /// A value the run itself published, read off an occurrence an earlier act required.
    fn observed(&mut self, written: &str, field: &str) -> Option<ScenarioValue> {
        let (name, declared) = self.declared(written, self.ir.events(), |event| {
            Cause::UndeclaredEvent { event }
        })?;
        let event = EventRef::new(name);
        if !declared.fields.iter().any(|it| it.name == field) {
            self.refuse(Cause::UndeclaredField {
                surface: Surface::Payload(event),
                at: String::new(),
                field: field.to_owned(),
            });
            return None;
        }
        if !self.observed.contains(&event) {
            self.refuse(Cause::Unobserved { event });
            return None;
        }
        self.source.insert(event.clone().into());
        Some(ScenarioValue::observed(event, field))
    }

    /// Every way a supplied value failed to be a value of its declared type, in this vocabulary.
    fn shape(&mut self, errors: &ShapeErrors, surface: &Surface) {
        for error in errors.iter() {
            let cause = match error {
                ShapeError::MissingField { at, field } => Cause::MissingField {
                    surface: surface.clone(),
                    at: at.clone(),
                    field: field.clone(),
                },
                ShapeError::UndeclaredField { at, field } => Cause::UndeclaredField {
                    surface: surface.clone(),
                    at: at.clone(),
                    field: field.clone(),
                },
                ShapeError::UndeclaredVariant {
                    declared_by,
                    value,
                    variants,
                    ..
                } => self.variant(surface, declared_by, value, variants),
                other => Cause::ValueRejected {
                    surface: surface.clone(),
                    detail: other.to_string(),
                },
            };
            self.refuse(cause);
        }
    }

    /// A name a closed set does not have — as a lifecycle state where the set is one.
    ///
    /// A state machine's states reach the model as an enum like any other, and a reader who wrote
    /// `Payed` needs to be told which states the *entity* has, not which variants an anonymous type
    /// declares. So the two are told apart here rather than reported as one.
    fn variant(
        &self,
        surface: &Surface,
        declared_by: &str,
        value: &str,
        variants: &[String],
    ) -> Cause {
        let name = QualifiedName::new(declared_by).ok();
        let entity = name.as_ref().and_then(|name| {
            self.ir
                .entities()
                .iter()
                .find(|(_, entity)| entity.state_type.name() == name)
                .map(|(entity, _)| EntityRef::new(entity.clone()))
        });
        match (entity, name) {
            (Some(entity), _) => Cause::UndeclaredState {
                entity,
                state: value.to_owned(),
                declared: variants.to_vec(),
            },
            (None, Some(name)) => Cause::UndeclaredVariant {
                surface: surface.clone(),
                declared_by: DeclaredTypeRef::new(name),
                value: value.to_owned(),
                variants: variants.to_vec(),
            },
            // `declared_by` is written by the flattener out of a resolved type's own name, so it is
            // a qualified name; the arm keeps the match total without inventing one.
            (None, None) => Cause::ValueRejected {
                surface: surface.clone(),
                detail: format!("`{value}` is not a variant of `{declared_by}`"),
            },
        }
    }
}

impl Assertion {
    /// The claims this assertion states, which has to be exactly one.
    fn stated(&self) -> Vec<&'static str> {
        let mut stated = Vec::new();
        if self.contains.is_some() {
            stated.push("contains");
        }
        if self.excludes.is_some() {
            stated.push("excludes");
        }
        if self.counts.is_some() {
            stated.push("counts");
        }
        if self.ranked == Some(true) {
            stated.push("ranked");
        }
        if self.at.is_some() {
            stated.push("at");
        }
        if self.satisfies.is_some() {
            stated.push("satisfies");
        }
        stated
    }
}

/// The type an entity's identity has, where the reference names one this IR declares.
trait Identity {
    /// The identity's declared type.
    fn entity_identity(&self, entity: &EntityRef) -> Option<&ResolvedTypeRef>;
}

impl Identity for EssIr {
    fn entity_identity(&self, entity: &EntityRef) -> Option<&ResolvedTypeRef> {
        self.entities()
            .get(entity.name())
            .map(|declared| &declared.identity.type_ref)
    }
}
