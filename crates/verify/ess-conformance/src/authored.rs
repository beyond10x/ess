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
//! # An early stop
//!
//! An assertion may say that a reader of an ordered view took a number of rows, said stop, and that
//! the producer stopped too.
//!
//! ```yaml
//! assert:
//!   - view: acd.matching.SlotOrder
//!     params: {group_id: {$instance: group}}
//!     halts_after: 2
//! ```
//!
//! **It is the seventh claim key and not a seventh row shape, and that is the whole design.** The
//! other six — `contains`, `excludes`, `counts`, `ranked`, `at`, `satisfies` — are predicates over
//! the rows a read returned, and this one cannot be. Two implementations that return identical rows
//! differ on it: one reads the whole listing, hands back the first two and discards the rest; one
//! pulls two and stops. Nothing in the result tells them apart, which is exactly why this format
//! could not state the claim before and why a prefix assertion written in its place would be green
//! for a reason unrelated to what it says.
//!
//! So it changes the **read** rather than the expectation. It compiles to
//! [`ExpectHalt`](ScenarioStep::ExpectHalt) — or [`EventuallyHalt`](ScenarioStep::EventuallyHalt),
//! read off the view's declared consistency as every other claim about a view is — and that step
//! carries the view and its parameters, because the read and the claim are one act.
//!
//! **What a target is asked, and what happens when it cannot answer.** Read the view in its declared
//! order, hand rows to a reader that takes `halts_after` of them and then says stop, and report two
//! facts about the read: how many rows the ordered *source* produced, and whether the reader's stop
//! is what ended it. Both are needed. The count alone is satisfied by a listing that happened to
//! hold exactly that many rows and ran out; the flag alone is satisfied by a target that
//! materialised everything and then broke out of a loop over the copy. A target that cannot read a
//! view a row at a time answers `unsupported`, the scenario is reported `unsupported`, and §28 makes
//! the run fail. There is no path on which a scan nobody stopped passes.
//!
//! **`halts_after: 0` is refused.** A reader that takes no row never sees one and therefore never
//! says stop, and what an honest source produces before being refused its first row differs between
//! two implementations that are both right.
//!
//! # An external answer
//!
//! No input decides a branch the specification declares `external:` (§12): whether a provider
//! accepts the mail is the provider's answer. So an act that names one under `outcome:` states that
//! answer for its call, and compiles to a
//! [`ConfigureExternalOutcome`](ScenarioStep::ConfigureExternalOutcome) for that branch immediately
//! before its [`ExecuteCommand`](ScenarioStep::ExecuteCommand) — the step synthesis writes for the
//! same branch — so a target is told what to answer rather than left to happen upon it. An act
//! naming a branch the input decides compiles to no such step.
//!
//! A claim that holds only on an external answer the act does not state expects that answer anyway,
//! and is refused ([`ExternalAnswerUnstated`](Cause::ExternalAnswerUnstated)), naming every branch
//! it could mean, whether or not `outcome:` is written. The claims read are the act's error and
//! direct response, each event it claims published and each it claims absent. The answers reached
//! are its own command's and those of every command a binding invokes from what it publishes,
//! transitively; a binding's escalation needs its invoked command to fail. Only the act's own
//! external branch written under `outcome:` is stated — an authored act has no key for the answer
//! a binding's call gives — and a command the act never reaches exempts nothing.
//!
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
//! `^_*[A-Za-z][A-Za-z0-9_]*$`), so the two can never be confused, and a plain value is written
//! exactly as the model's own documents write one.
//!
//! **A reference may sit inside a value** (beyond10x/ess#242): `ring_sequence: [{$instance: a},
//! {$instance: b}]` for a `List<ReleaseRingId>`, a map value, a struct member, at any depth,
//! wherever the declared type at that position is the instance's identity type. It compiles to a
//! [`List`](ScenarioValue::List) or [`Members`](ScenarioValue::Members) the runner resolves element
//! by element, and at a position of any other type it is refused, naming the position.
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
//! * **A count of rows a target read on its own account.** [`halts_after`](Assertion::halts_after)
//!   below says a reader *stopped a producer*, which is a claim about an implementation's own
//!   iteration; nothing here asks how many rows a target read for its own reasons, because no
//!   specification construct obliges a number.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_compiler::diagnostic::Code;
use ess_compiler::ir::{
    EssIr, ResolvedBinding, ResolvedBody, ResolvedCommand, ResolvedField, ResolvedOutcome,
    ResolvedTypeRef,
};
use ess_domain::command::{fixture_inputs::FixtureName, OutcomeName, TestStrategy};
use ess_domain::name::QualifiedName;
use ess_domain::view::{AssertionStyle, Ranking};
use ess_primitives::error::ParseError;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use ess_primitives::time::{CivilDate, Timestamp};
use serde::Deserialize as _;

use crate::input::{bind, projection_target, Completeness, ShapeError, ShapeErrors};
use crate::scenario::{
    ActorRef, AuthoredName, CommandRef, ConformanceScenario, DeclaredTypeRef, DomainRef, Elapsed,
    EntityRef, ErrorRef, EssSemanticRef, EventRef, InstanceName, InstantName, OutcomeRef, Position,
    ScenarioId, ScenarioPurpose, ScenarioStep, ScenarioValue, ViewExpectation, ViewRef,
};
use crate::synthesize::{payload_shape, reachable_types};

// ---- the diagnostic catalogue ------------------------------------------------------------------

/// Declares a refusal type's diagnostic catalogue and the exhaustive match that maps a value to its
/// entry, from one list.
///
/// ```ignore
/// diagnostic_catalogue! {
///     impl Refusal => u16 {
///         Self::Unreadable { .. } => 1, "the file is not a document", "write YAML";
///         Self::NothingHappens => 2, "the scenario runs nothing", "give it a timeline";
///     }
/// }
/// ```
///
/// Generates `Refusal::CATALOGUE`, every entry in the order written, and
/// `Refusal::catalogue_entry(&self)`, which matches `self` against the patterns in the same order.
/// A variant no pattern covers does not compile, so a code cannot be emitted without an entry; and
/// because both are expanded from the same arm, an entry cannot say one thing in the catalogue and
/// another at the site. Two patterns may share a variant when a field decides the code. An empty
/// meaning or repair is refused at compile time. `website/docs/reference/diagnostics.md` is
/// rendered from the catalogues by `cargo xtask diagnostics`.
macro_rules! diagnostic_catalogue {
    (
        impl $type:ty => $key:ty {
            $( $pattern:pat => $value:expr, $meaning:expr, $repair:expr; )*
        }
    ) => {
        impl $type {
            /// Every diagnostic this type can carry, with what it means and how to repair it.
            pub const CATALOGUE: &'static [ess_compiler::diagnostic::CatalogueEntry<$key>] = &[
                $(
                    ess_compiler::diagnostic::CatalogueEntry {
                        key: $value,
                        meaning: $meaning,
                        repair: $repair,
                    },
                )*
            ];

            /// This value's entry in [`Self::CATALOGUE`].
            pub fn catalogue_entry(&self) -> &'static ess_compiler::diagnostic::CatalogueEntry<$key> {
                match self {
                    $(
                        $pattern => &ess_compiler::diagnostic::CatalogueEntry {
                            key: $value,
                            meaning: $meaning,
                            repair: $repair,
                        },
                    )*
                }
            }
        }

        const _: () = {
            $(
                assert!(
                    !$meaning.is_empty() && !$repair.is_empty(),
                    "a catalogued diagnostic has an empty meaning or repair"
                );
            )*
        };
    };
}

pub(crate) use diagnostic_catalogue;

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
    /// Independently provisioned values and their source-owned types (scenario format 3).
    #[serde(default)]
    pub fixtures: BTreeMap<FixtureName, ess_domain::TypeRef>,
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
    /// Actual upstream-owned state to establish (`ess-scenario/2` or newer).
    #[serde(default, deserialize_with = "deserialize_entity_setup")]
    pub setup: Option<EntitySetup>,
}

fn deserialize_entity_setup<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<EntitySetup>, D::Error> {
    <EntitySetup as serde::Deserialize>::deserialize(deserializer).map(Some)
}

/// Literal entity state, validated against the model before it becomes a target request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntitySetup {
    /// The identity value in the entity's declared identity type.
    pub identity: Node,
    /// All required entity fields; absence and present null remain distinct.
    pub fields: BTreeMap<String, Node>,
    /// A declared lifecycle state, without an invented transition history.
    pub state: ess_domain::entity::StateName,
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
    /// The refusal it must meet instead of running, where it expects one: `refused: not_granted`,
    /// the standard refusal for an actor no grant admits (beyond10x/ess#265). Available in
    /// ess-scenario/4. The act's `actor:` must then name a declared actor the specification does
    /// not grant the command, and the act claims nothing only a command that ran could answer.
    #[serde(default)]
    pub refused: Option<Refused>,
    /// Literal assertions over the actual direct return; `{}` checks its closed shape only.
    /// Available in ess-scenario/4. Values never come from the target's own post-state.
    #[serde(default, deserialize_with = "deserialize_response_claim")]
    pub response: Option<BTreeMap<String, Node>>,
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

/// A refusal an act expects in place of any branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refused {
    /// The standard refusal for an actor no grant admits, answered before the command runs
    /// (beyond10x/ess#265).
    NotGranted,
}

fn deserialize_response_claim<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, Node>>, D::Error> {
    <BTreeMap<String, Node> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

/// One claim about the time between a marked instant and the act that carries it.
///
/// Exactly one of the three bounds, for the reason [`Assertion`] states about its seven: two would
/// be two claims filed as one, and none would be a claim that cannot fail.
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
/// Exactly one of the seven expectation keys, because two would be two assertions filed as one and
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
    /// A reader of this view takes this many rows, says stop, and the producer stops too.
    ///
    /// The one claim here that is not about the rows. See the module documentation for why it
    /// could not be filed beside the other six and had to change the read instead.
    #[serde(default)]
    pub halts_after: Option<usize>,
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
    /// A typed value resolved independently before the scenario starts.
    Fixture(FixtureName),
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
    /// The sigil naming a pre-execution fixture.
    pub const FIXTURE: &'static str = "$fixture";
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
            Self::FIXTURE => {
                let name = value
                    .as_text()
                    .ok_or_else(|| serde::de::Error::custom("$fixture names a fixture"))?;
                FixtureName::new(name)
                    .map(Self::Fixture)
                    .map_err(serde::de::Error::custom)
            }
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
    /// An entity's explicitly arranged state.
    Entity(EntityRef),
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
            Self::Entity(entity) => write!(f, "the arranged state of `{entity}`"),
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
    /// The model needs a finite codec outside the current conformance contract.
    UnsupportedBinary64 {
        /// Every unsupported model field/type position.
        locations: Vec<String>,
    },
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
    /// An assertion's paths resolve, but its operands or quantified target are ill-typed.
    InvalidPredicate {
        /// The row owner.
        view: ViewRef,
        /// The shared semantic error, with its owner and full expression.
        diagnostic: ess_domain::expression::ExpressionError,
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
    /// An early stop is claimed after no rows at all.
    HaltsAtNothing {
        /// Which view.
        view: ViewRef,
    },
    /// An act claims what only an external answer it does not state satisfies.
    ///
    /// No input decides an external branch (§12), so the claim holds only if the target is told
    /// which answer to give. An act states one answer — its own command's external branch, named
    /// under `outcome:` — and the claim needs another: a sibling of that command, or a branch of a
    /// command a binding invokes from what the act publishes. Refused whether or not `outcome:` is
    /// written.
    ExternalAnswerUnstated {
        /// The command the act invokes.
        command: CommandRef,
        /// What it claims, written as a reader sees it: an error, a direct response, an event
        /// published or an event absent.
        claim: String,
        /// Every external branch, as `command/branch`, whose answer the claim could mean, in the
        /// order the act reaches them.
        branches: Vec<String>,
    },
    /// The act expects the refusal an ungranted actor gets, and the specification grants the actor
    /// the command (beyond10x/ess#265).
    ActorGranted {
        /// Who.
        actor: ActorRef,
        /// What.
        command: CommandRef,
    },
    /// The act expects the refusal an ungranted actor gets, and names no actor to refuse or claims
    /// what only a command that ran could answer (beyond10x/ess#265).
    RefusalContradicted {
        /// The command the act invokes.
        command: CommandRef,
        /// What contradicts the refusal, as a reader sees it: a missing `actor:`, and each key
        /// only a command that ran answers — `outcome:`, `error:`, `response:`, `events:`,
        /// `capture:` — in that order.
        claims: Vec<String>,
    },
    /// The act sends with no `actor:` a command a served component accepts and the specification
    /// grants to no actor, so every caller is refused it (beyond10x/ess#265), and the act requires
    /// what no conforming surface does.
    UngrantedOnServedSurface {
        /// The command.
        command: CommandRef,
    },
}

/// Whether `code` is one a refusal of one authored file can carry: every cause [`Cause::code`]
/// numbers but `ESS-AUTHOR-036`, which refuses the model before any file is read and so names no
/// source. Coverage admission reads this; the Go, TypeScript and browser readers restate it.
pub(crate) fn names_file_refusal(code: &str) -> bool {
    Cause::CATALOGUE
        .iter()
        .filter(|entry| entry.key != 36)
        .any(|entry| code == Code::new(Cause::FAMILY, entry.key).to_string())
}

/// Whether a served component (`reached_by: network`) accepts `command` and no declared actor is
/// granted it: every caller is refused it (beyond10x/ess#265).
fn ungranted_on_served_surface(ir: &EssIr, command: &ess_domain::name::QualifiedName) -> bool {
    let served = ir.components().values().any(|component| {
        component.reached_by == ess_domain::component::Reach::Network
            && component
                .accepts
                .iter()
                .any(|accepted| accepted.name() == command)
    });
    served
        && !ir
            .actors()
            .values()
            .any(|actor| actor.may.iter().any(|granted| granted.name() == command))
}

/// Whether no input decides `outcome` (§12): the test strategy an `external:` branch carries.
fn is_external(outcome: &ResolvedOutcome) -> bool {
    outcome.test_strategy == TestStrategy::InjectFault
}

/// `command/branch`, as an [`OutcomeRef`] is written.
fn branch(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> String {
    OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone()).to_string()
}

/// The external branches of the act's own command that alone make `produces` true of its answer.
///
/// Empty when the written branch produces it, or when a branch the input decides can.
fn own_only_external(
    command: &ResolvedCommand,
    written: Option<&ResolvedOutcome>,
    produces: impl Fn(&ResolvedOutcome) -> bool,
) -> Vec<String> {
    if written.is_some_and(&produces) {
        return Vec::new();
    }
    let producing: Vec<&ResolvedOutcome> = command
        .outcomes
        .iter()
        .filter(|outcome| produces(outcome))
        .collect();
    if producing.iter().any(|outcome| !is_external(outcome)) {
        return Vec::new();
    }
    producing
        .into_iter()
        .map(|outcome| branch(command, outcome))
        .collect()
}

/// `names` once each, in the order first met.
fn in_order<'a>(names: impl Iterator<Item = &'a String>) -> Vec<String> {
    let mut once: Vec<String> = Vec::new();
    for name in names {
        if !once.contains(name) {
            once.push(name.clone());
        }
    }
    once
}

impl Cause {
    /// The family every refusal here belongs to.
    pub const FAMILY: &'static str = "AUTHOR";

    /// Its stable code.
    ///
    /// Derived from the variant rather than stored beside it, so a code cannot come to name a body
    /// other than its own: the number is the variant's entry in [`Cause::CATALOGUE`].
    pub fn code(&self) -> Code {
        Code::new(Self::FAMILY, self.catalogue_entry().key)
    }

    /// What would have to change for the scenario to compile.
    ///
    /// The repair in the variant's [`Cause::CATALOGUE`] entry, so the `help:` line and the published
    /// reference say the same thing.
    pub fn hint(&self) -> &'static str {
        self.catalogue_entry().repair
    }
}

// One arm per cause, and long because there are forty of them. A reader comparing two
// repairs reads them side by side or not at all, and splitting the list would put half of it
// somewhere else. The meaning is the variant's own first line of documentation.
diagnostic_catalogue! {
    impl Cause => u16 {
        Self::Unreadable { .. } => 1,
            "The file is not a document this format can read.",
            "the document is YAML with the keys `type`, `domain`, `scenario` and `summary`; a \
             key it does not know is refused rather than ignored";
        Self::UnsupportedFormat { .. } => 2,
            "The document claims a format this build does not implement.",
            "write `type: ess-scenario/1`, `ess-scenario/2` for entity setup, `ess-scenario/3` for \
             fixture values, or `ess-scenario/4` for direct responses";
        Self::Duplicate { .. } => 3,
            "Two files produce the same scenario id.",
            "two files name one scenario in one domain; rename one of them";
        Self::UndeclaredDomain { .. } => 4,
            "The model declares no such bounded context.",
            "name a bounded context the specification declares, or add it to the model";
        Self::UndeclaredEntity { .. } => 5,
            "The model declares no such entity.",
            "name an entity the specification declares; an authored scenario acts on the model's \
             own instances and invents none";
        Self::UndeclaredCommand { .. } => 6,
            "The model declares no such command.",
            "name a command the specification declares; a scenario that invokes anything else is \
             checking a system this model does not describe";
        Self::UndeclaredOutcome { .. } => 7,
            "That command declares no such branch.",
            "name one of the branches the command declares, or declare the branch you meant";
        Self::UndeclaredActor { .. } => 8,
            "The model declares no such actor.",
            "name an actor the specification declares, or drop `actor:`";
        Self::ActorMayNot { .. } => 9,
            "The actor is declared, and the specification does not grant it this command.",
            "grant the command to this actor with `may:`, act as one that already has it, or \
             write `refused: not_granted` where the act expects the refusal";
        Self::UndeclaredEvent { .. } => 10,
            "The model declares no such event.",
            "name an event the specification declares";
        Self::UndeclaredError { .. } => 11,
            "The model declares no such error.",
            "name a declared error the specification declares";
        Self::UndeclaredView { .. } => 12,
            "The model declares no such view.",
            "name a view the specification declares";
        Self::UndeclaredField { .. } => 13,
            "A value is supplied under a name the surface does not declare.",
            "name a field the construct declares; the scenario and the model disagree about \
             what it has";
        Self::MissingField { .. } => 14,
            "A declared field that has to be supplied is not.",
            "supply the field; a command is invoked with all of its input, and one left out is a \
             call that could not be made";
        Self::ValueRejected { .. } => 15,
            "A value is not of the shape its declared type calls for.",
            "write a value of the type the model declares there";
        Self::UndeclaredVariant { .. } => 16,
            "A value names something a declared enum does not have as a variant.",
            "write one of the variants the enum declares; the set is closed";
        Self::UndeclaredState { .. } => 17,
            "A value names a lifecycle state the entity does not declare.",
            "write one of the states the entity's lifecycle declares";
        Self::UnarrangedInstance { .. } => 18,
            "A reference names an instance the arrangement does not declare.",
            "declare the instance under `arrange:`, so the scenario says whose it is";
        Self::UnboundInstance { .. } => 19,
            "A reference names an instance nothing has bound yet.",
            "capture the instance from an event before the step that names it; a suite carries \
             no identity of its own";
        Self::Unobserved { .. } => 20,
            "A reference reads an event no earlier act required.",
            "require the event in an earlier act; a value is read off an occurrence the run \
             produced, and this scenario has not required one";
        Self::NotComparable { .. } => 21,
            "A reference sits where the suite compares values it carries itself.",
            "write the value the field must hold; an event's payload and an error's fields are \
             compared against values the suite carries";
        Self::InstanceMistyped { .. } => 22,
            "An instance is bound to a field of a type that is not its entity's identity.",
            "bind the instance to a field typed as the entity's identity, or arrange the entity \
             whose identity this field carries";
        Self::UnorderedTimeline { .. } => 23,
            "The timeline's instants do not strictly ascend.",
            "give each act an instant later than the one before it; the file's order is the \
             scenario's order and `at:` is what states it";
        Self::Unordered { .. } => 24,
            "A position or an order is asserted of a view that declares no order.",
            "declare `order_by:` on the view, or assert `contains:` instead; a position in an \
             unordered view names a different row on every read";
        Self::AmbiguousClaim { .. } => 25,
            "An assertion states other than exactly one claim.",
            "state exactly one of `contains`, `excludes`, `counts`, `ranked`, `at`, \
             `satisfies` or `halts_after` per assertion";
        Self::UnreadablePredicate { .. } => 26,
            "A predicate reads something the view does not publish.",
            "read only fields the view projects, or project the field the predicate reads";
        Self::NothingHappens => 27,
            "The scenario runs no command and asserts nothing.",
            "give the scenario a timeline; a scenario that runs nothing is a check that cannot \
             fail";
        Self::UnmarkedInstant { .. } => 28,
            "A window is measured from an instant nothing marked before it.",
            "write `mark:` on the earlier act the window opens at; a duration is measured from \
             an instant somebody named, never from wherever the step before it happened to end";
        Self::DuplicateInstant { .. } => 29,
            "Two acts mark the same instant.",
            "give the two acts different names; one name for two instants is a window whose \
             length depends on which one a reader had in mind";
        Self::VacuousWindow { .. } => 30,
            "A window has no width.",
            "give the window a length; a window of no seconds is a claim about no time, and \
             there is no target it can fail against";
        Self::QuietAboutNothing { .. } => 31,
            "A bounded negative forbids no occurrence.",
            "name the events the window must stay clear of; a bounded negative that forbids \
             nothing is satisfied by every implementation there is";
        Self::WindowContradictsTimeline { .. } => 32,
            "The timeline's own instants say something the window contradicts.",
            "move the act's `at:` so the file's own instants agree with the claim, or state \
             the length the file already shows; the timeline is what a reader believes";
        Self::AmbiguousWindow { .. } => 33,
            "A window states other than exactly one bound.",
            "state exactly one of `not_before`, `within` or `quiet` per window, and write a \
             second window for a second claim";
        Self::HaltsAtNothing { .. } => 34,
            "An early stop is claimed after no rows at all.",
            "say how many rows the reader takes before it stops; a reader that takes none never \
             sees a row and so never says stop, and what a source produced before being refused \
             its first row is a different answer in two implementations that are both right";
        Self::InvalidPredicate { .. } => 35,
            "An assertion's paths resolve, but its operands or quantified target are ill-typed.",
            "use compatible scalar operands and quantify only over a declared List or Map";
        Self::UnsupportedBinary64 { .. } => 36,
            "The model needs a finite codec outside the current conformance contract.",
            "use checked format-5 normalization; finite Binary64 conformance codecs are not \
             implemented";
        Self::ExternalAnswerUnstated { .. } => 37,
            "An act claims what only an external answer it does not state satisfies.",
            "no input the act sends decides an external branch, so name the act's own one \
             under `outcome:` and the suite configures that answer for its call; an answer a \
             command invoked through a binding must give cannot be stated in an authored act, \
             so drop the claim there and leave it to synthesis";
        Self::ActorGranted { .. } => 38,
            "An act expects the refusal an ungranted actor gets, and the actor holds the grant.",
            "act as an actor the specification does not grant the command, or drop `refused: \
             not_granted` and claim what the command answers";
        Self::RefusalContradicted { .. } => 39,
            "An act expects the refusal an ungranted actor gets, and names no actor or claims \
             what only a command that ran answers.",
            "a refused command takes no branch, reports no declared error and returns nothing: \
             keep `actor:` beside `refused: not_granted`, drop `outcome`, `error`, `response`, \
             `events` and `capture`, and list under `no_events:` what must not appear anywhere in \
             the target's log after the refused send";
        Self::UngrantedOnServedSurface { .. } => 40,
            "An act sends a served command no actor is granted, which every caller is refused.",
            "grant the command to an actor with `may:` and send the act as that actor, or write \
             `refused: not_granted` with an `actor:` the specification declares";
    }
}

impl fmt::Display for Cause {
    /// One arm per cause, and long because there are thirty-four of them. Splitting it would put
    /// half the wording somewhere a reader comparing two refusals has to go and find.
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedBinary64 { locations } => write!(
                f,
                "{}: Binary64 requires a qualified conformance codec",
                locations.join(", ")
            ),
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
            Self::ActorGranted { actor, command } => write!(
                f,
                "`{actor}` is granted `{command}`, so it is not refused as not granted"
            ),
            Self::UngrantedOnServedSurface { command } => write!(
                f,
                "`{command}` is accepted by a served component and no declared actor is granted \
                 it, so every caller is refused it and it cannot run"
            ),
            Self::RefusalContradicted { command, claims } => write!(
                f,
                "`{command}` is expected refused as not granted, which contradicts {}",
                claims.join(", ")
            ),
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
            Self::InvalidPredicate { view, diagnostic } => {
                write!(
                    f,
                    "the assertion over a row of `{view}` is ill-typed: {}",
                    diagnostic.message
                )
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
            Self::HaltsAtNothing { view } => write!(
                f,
                "the read of `{view}` is claimed to stop after no rows at all"
            ),
            Self::ExternalAnswerUnstated {
                command,
                claim,
                branches,
            } => write!(
                f,
                "the act on `{command}` claims {claim}, which holds only if {} {} answers, and \
                 the act states no such answer",
                if branches.len() == 1 {
                    "the external branch"
                } else {
                    "one of the external branches"
                },
                branches
                    .iter()
                    .map(|branch| format!("`{branch}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
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
    if let Err(error) = crate::admission::model(ir) {
        return Authoring {
            scenarios: BTreeMap::new(),
            refusals: vec![Refusal {
                origin: "model".to_owned(),
                scenario: None,
                cause: Cause::UnsupportedBinary64 {
                    locations: error.issues.into_iter().map(|issue| issue.path).collect(),
                },
            }],
        };
    }
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

/// The newtypes of a compiled model that may key a map, for reading type spellings against it.
///
/// The same declarations the specification was read with: each newtype with what it wraps,
/// spelled as a primitive or as the declared type's name.
fn map_key_newtypes(ir: &EssIr) -> ess_domain::types::MapKeyNewtypes {
    use ess_compiler::ir::ResolvedBody;
    ess_domain::types::MapKeyNewtypes::from_declarations(ir.types().values().filter_map(
        |declared| match &declared.body {
            ResolvedBody::Newtype {
                of: ResolvedTypeRef::Primitive { name },
                ..
            } => Some((declared.name.clone(), name.as_str().to_owned())),
            ResolvedBody::Newtype {
                of: ResolvedTypeRef::Declared { name },
                ..
            } => Some((declared.name.clone(), ir.named_type(name).name.to_string())),
            _ => None,
        },
    ))
}

/// One file, or every reason it produced nothing.
pub(crate) fn compile_one(
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

    // A fixture is declared in the specification's own type spellings, so a map keyed by one of
    // the model's newtypes (beyond10x/ess#143) reads here exactly as it did there.
    let document: Document = map_key_newtypes(ir)
        .scope(|| serde_yaml::from_str(&source.text))
        .map_err(|error| {
            bare(Cause::Unreadable {
                detail: error.to_string(),
            })
        })?;
    if let Some(cause) = document_format_refusal(&document) {
        return Err(bare(cause));
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
        fixtures: document.fixtures.clone(),
        used_fixtures: BTreeSet::new(),
        fixture_format: matches!(
            document.format.as_str(),
            "ess-scenario/3" | "ess-scenario/4"
        ),
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
    if compiler.used_fixtures != document.fixtures.keys().cloned().collect() {
        compiler.refuse(Cause::Unreadable {
            detail: "fixture declarations must exactly match the referenced names".into(),
        });
    }
    if !document.fixtures.is_empty() {
        match crate::fixtures::Contract::of(ir, document.fixtures.clone()) {
            Ok(fixtures) => compiler
                .steps
                .insert(0, ScenarioStep::ResolveFixtures { fixtures }),
            Err(detail) => compiler.refuse(Cause::Unreadable { detail }),
        }
    }
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

fn document_format_refusal(document: &Document) -> Option<Cause> {
    if !matches!(
        document.format.as_str(),
        FORMAT | "ess-scenario/2" | "ess-scenario/3" | "ess-scenario/4"
    ) {
        return Some(Cause::UnsupportedFormat {
            found: document.format.clone(),
        });
    }
    if document.format == FORMAT && document.arrange.iter().any(|item| item.setup.is_some()) {
        return Some(Cause::Unreadable {
            detail: "entity setup requires type: ess-scenario/2".into(),
        });
    }
    if !matches!(
        document.format.as_str(),
        "ess-scenario/3" | "ess-scenario/4"
    ) && !document.fixtures.is_empty()
    {
        return Some(Cause::Unreadable {
            detail: "fixture declarations require type: ess-scenario/3".into(),
        });
    }
    if document.format != "ess-scenario/4"
        && document.timeline.iter().any(|act| act.response.is_some())
    {
        return Some(Cause::Unreadable {
            detail: "direct response assertions require type: ess-scenario/4".into(),
        });
    }
    if document.format != "ess-scenario/4"
        && document.timeline.iter().any(|act| act.refused.is_some())
    {
        return Some(Cause::Unreadable {
            detail: "`refused: not_granted` requires type: ess-scenario/4".into(),
        });
    }
    None
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
    fixtures: BTreeMap<FixtureName, ess_domain::TypeRef>,
    used_fixtures: BTreeSet<FixtureName>,
    fixture_format: bool,
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
        let mut established: Vec<(EntityRef, Node)> = Vec::new();
        for arrangement in &document.arrange {
            let Some((name, declared)) =
                self.declared(&arrangement.entity, self.ir.entities(), |entity| {
                    Cause::UndeclaredEntity { entity }
                })
            else {
                continue;
            };
            let entity = EntityRef::new(name);
            self.source.insert(entity.clone().into());
            if self.arranged.contains_key(&arrangement.instance) {
                self.refuse(Cause::ValueRejected {
                    surface: Surface::Entity(entity),
                    detail: format!("duplicate instance `{}`", arrangement.instance),
                });
                continue;
            }
            self.arranged
                .insert(arrangement.instance.clone(), entity.clone());
            if let Some(setup) = &arrangement.setup {
                if established.contains(&(entity.clone(), setup.identity.clone())) {
                    self.refuse(Cause::ValueRejected {
                        surface: Surface::Entity(entity),
                        detail: "duplicate qualified entity identity".into(),
                    });
                    continue;
                }
                if let Err(detail) = crate::input::validate_entity_setup(
                    self.ir,
                    &entity,
                    &setup.identity,
                    &setup.fields,
                    &setup.state,
                ) {
                    self.refuse(Cause::ValueRejected {
                        surface: Surface::Entity(entity),
                        detail,
                    });
                    continue;
                }
                let mut fields = declared.fields.clone();
                fields.push(declared.identity.clone());
                self.reach(&fields);
                established.push((entity.clone(), setup.identity.clone()));
                self.bound.insert(arrangement.instance.clone());
                self.steps.push(ScenarioStep::EstablishEntity {
                    instance: arrangement.instance.clone(),
                    entity,
                    identity: setup.identity.clone(),
                    fields: setup.fields.clone(),
                    state: setup.state.clone(),
                });
            }
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
        if document.timeline.is_empty() && (established.is_empty() || document.assert.is_empty()) {
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

        if act.refused == Some(Refused::NotGranted) {
            self.refused_act(act, &command_ref, &input_fields);
            return;
        }
        let actor = self.sender(act, &command_ref);
        let input = self.values(
            &act.input,
            &input_fields,
            &Surface::Input(command_ref.clone()),
            Completeness::Total,
        );
        self.external_answer(&command_ref, command, act);
        self.steps.push(ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
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
        self.unstated_external(&command_ref, command, act);
        let returning = match &act.outcome {
            Some(name) => command
                .outcomes
                .iter()
                .any(|outcome| outcome.name.as_str() == name && outcome.returns),
            None => command.outcomes.iter().all(|outcome| outcome.returns),
        };
        if act.response.is_some() || returning {
            self.reach(&command.response);
            let outcome = act
                .outcome
                .as_ref()
                .and_then(|name| name.parse().ok())
                .map(|name| OutcomeRef::new(command_ref.clone(), name));
            match crate::direct_response::Observation::of(
                self.ir,
                command,
                outcome,
                act.response.clone().unwrap_or_default(),
            ) {
                Ok(response) => self
                    .steps
                    .push(ScenarioStep::ExpectDirectResponse { response }),
                Err(detail) => self.refuse(Cause::Unreadable { detail }),
            }
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

    /// Arms the answer an act naming an external branch states, immediately before its call.
    ///
    /// §12: no input decides an external branch, so naming one under `outcome:` is the answer the
    /// target must give for this call. Armed after the act's windows and before its
    /// `ExecuteCommand`, as synthesis arms it, so nothing earlier can spend it. A branch the input
    /// decides arms nothing.
    fn external_answer(&mut self, command_ref: &CommandRef, command: &ResolvedCommand, act: &Act) {
        if let Some(external) = act.outcome.as_ref().and_then(|written| {
            command.outcomes.iter().find(|outcome| {
                outcome.name.as_str() == written
                    && outcome.test_strategy == TestStrategy::InjectFault
            })
        }) {
            self.steps.push(ScenarioStep::ConfigureExternalOutcome {
                force: OutcomeRef::new(command_ref.clone(), external.name.clone()),
                times: None,
            });
        }
    }

    /// Refuses each claim of an act that only an external answer the act does not state satisfies.
    ///
    /// No input decides an external branch (§12), so a claim that holds only on one expects an
    /// answer the target is never told to give. The act states one answer: an external branch of its
    /// own command written under `outcome:`, which [`Self::external_answer`] arms. Every other
    /// external branch the act reaches — a sibling of its own command, or a branch of a command a
    /// binding invokes from what the act publishes, however many bindings along — is unstated.
    ///
    /// Read whether or not `outcome:` is written, for every claim the act makes of its own call:
    /// its error and its direct response (of its own command's branches), each event it claims
    /// published (a branch's `emits:` or a binding's escalation, anywhere downstream), and each
    /// event it claims absent. Every such claim is refused on its own, naming every branch it could
    /// mean. What an act never reaches exempts nothing.
    fn unstated_external(
        &mut self,
        command_ref: &CommandRef,
        command: &ResolvedCommand,
        act: &Act,
    ) {
        let written = act.outcome.as_ref().map(|name| {
            command
                .outcomes
                .iter()
                .find(|outcome| outcome.name.as_str() == name)
        });
        // A misspelt branch is refused as `ESS-AUTHOR-011`; which answer it meant is not guessed.
        let written = match written {
            Some(None) => return,
            Some(Some(outcome)) => Some(outcome),
            None => None,
        };
        let mut unstated: Vec<(String, Vec<String>)> = Vec::new();

        // The error and the response are the act's own call's answer.
        if let Some(error) = act
            .error
            .as_ref()
            .and_then(|claim| QualifiedName::new(&claim.name).ok())
        {
            let branches = own_only_external(command, written, |outcome| {
                outcome.error.as_ref().is_some_and(|it| it.name() == &error)
            });
            unstated.push((format!("error `{error}`"), branches));
        }
        if act.response.is_some() {
            let branches = own_only_external(command, written, |outcome| outcome.returns);
            unstated.push(("a direct response".to_owned(), branches));
        }

        let published = self.published(command, written);
        for event in act
            .events
            .iter()
            .filter_map(|claim| QualifiedName::new(&claim.event).ok())
        {
            let needs = published.get(&event).map(Vec::as_slice).unwrap_or_default();
            let branches = if needs.iter().any(BTreeSet::is_empty) {
                Vec::new()
            } else {
                in_order(needs.iter().flatten())
            };
            unstated.push((format!("event `{event}`"), branches));
        }
        for event in act
            .no_events
            .iter()
            .filter_map(|written| QualifiedName::new(written).ok())
        {
            let branches = self.avoiding(command, written, &event);
            unstated.push((format!("no event `{event}`"), branches));
        }

        for (claim, branches) in unstated {
            if !branches.is_empty() {
                self.refuse(Cause::ExternalAnswerUnstated {
                    command: command_ref.clone(),
                    claim,
                    branches,
                });
            }
        }
    }

    /// Every event the act can set off, each with the external answers each way to it needs.
    ///
    /// Walks from the act's own branches through every binding an emitted event triggers, into
    /// every branch of the command it invokes, transitively. A branch the input decides needs no
    /// answer; the written `outcome:` needs none either, since it is stated. A binding's escalation
    /// is published when its invoked command fails — an external branch, or a branch reporting an
    /// error. An event with an empty need set is reached without an unstated answer.
    fn published(
        &self,
        command: &ResolvedCommand,
        written: Option<&ResolvedOutcome>,
    ) -> BTreeMap<QualifiedName, Vec<BTreeSet<String>>> {
        let mut published: BTreeMap<QualifiedName, Vec<BTreeSet<String>>> = BTreeMap::new();
        let mut seen: BTreeSet<(String, BTreeSet<String>)> = BTreeSet::new();
        let mut pending: Vec<(&ResolvedCommand, &ResolvedOutcome, BTreeSet<String>)> = command
            .outcomes
            .iter()
            .map(|outcome| {
                let stated = written.is_some_and(|it| it.name == outcome.name);
                let needs = if stated || !is_external(outcome) {
                    BTreeSet::new()
                } else {
                    [branch(command, outcome)].into_iter().collect()
                };
                (command, outcome, needs)
            })
            .collect();
        pending.reverse();
        while let Some((at, outcome, needs)) = pending.pop() {
            if !seen.insert((branch(at, outcome), needs.clone())) {
                continue;
            }
            for emitted in &outcome.emits {
                published
                    .entry(emitted.name().clone())
                    .or_default()
                    .push(needs.clone());
                for (invoked, binding) in self.triggered(emitted.name()) {
                    let mut next = Vec::new();
                    for downstream in &invoked.outcomes {
                        let mut further = needs.clone();
                        if is_external(downstream) {
                            further.insert(branch(invoked, downstream));
                        }
                        if let Some(escalation) = &binding.escalation {
                            if is_external(downstream) || downstream.error.is_some() {
                                published
                                    .entry(escalation.name().clone())
                                    .or_default()
                                    .push(further.clone());
                            }
                        }
                        next.push((invoked, downstream, further));
                    }
                    pending.extend(next.into_iter().rev());
                }
            }
        }
        published
    }

    /// The external branches whose answer alone keeps `event` unpublished, when every answer the
    /// input decides publishes it.
    ///
    /// Empty when some decided answer leaves it out (the claim can hold without an unstated answer)
    /// or when no answer at all does (the claim is false, which is not this refusal's to say).
    fn avoiding(
        &self,
        command: &ResolvedCommand,
        written: Option<&ResolvedOutcome>,
        event: &QualifiedName,
    ) -> Vec<String> {
        let decided: Vec<&ResolvedOutcome> = match written {
            Some(outcome) => vec![outcome],
            None => command
                .outcomes
                .iter()
                .filter(|outcome| !is_external(outcome))
                .collect(),
        };
        let mut chain = Vec::new();
        let mut visiting = BTreeSet::new();
        if decided.is_empty()
            || !decided
                .iter()
                .all(|outcome| self.must_publish(outcome, event, &mut visiting, &mut chain))
        {
            return Vec::new();
        }
        let mut names = Vec::new();
        let mut consider = |at: &ResolvedCommand, skip: Option<&ResolvedOutcome>| {
            for outcome in &at.outcomes {
                if is_external(outcome)
                    && skip.is_none_or(|it| it.name != outcome.name)
                    && !self.must_publish(outcome, event, &mut BTreeSet::new(), &mut Vec::new())
                {
                    names.push(branch(at, outcome));
                }
            }
        };
        consider(command, written);
        for at in chain {
            consider(at, None);
        }
        in_order(names.iter())
    }

    /// Whether taking `outcome` publishes `event` whatever each command a binding invokes after it
    /// answers, among the answers the input decides; `chain` gains each such command.
    fn must_publish<'ir>(
        &'ir self,
        outcome: &ResolvedOutcome,
        event: &QualifiedName,
        visiting: &mut BTreeSet<QualifiedName>,
        chain: &mut Vec<&'ir ResolvedCommand>,
    ) -> bool {
        if outcome.emits.iter().any(|it| it.name() == event) {
            return true;
        }
        for emitted in &outcome.emits {
            for (invoked, _) in self.triggered(emitted.name()) {
                if !visiting.insert(invoked.name.clone()) {
                    continue;
                }
                let decided: Vec<&ResolvedOutcome> = invoked
                    .outcomes
                    .iter()
                    .filter(|it| !is_external(it))
                    .collect();
                let mut found = Vec::new();
                let must = !decided.is_empty()
                    && decided
                        .iter()
                        .all(|it| self.must_publish(it, event, visiting, &mut found));
                visiting.remove(&invoked.name);
                if must {
                    chain.push(invoked);
                    chain.extend(found);
                    return true;
                }
            }
        }
        false
    }

    /// Each binding `event` triggers, with the command it invokes.
    fn triggered(&self, event: &QualifiedName) -> Vec<(&ResolvedCommand, &ResolvedBinding)> {
        self.ir
            .bindings()
            .values()
            .filter_map(|binding| {
                binding
                    .cause
                    .event()
                    .filter(|it| it.name() == event)
                    .and_then(|_| self.ir.commands().get(binding.command.name()))
                    .map(|invoked| (invoked, binding))
            })
            .collect()
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

    /// An act that expects the refusal an ungranted actor gets (beyond10x/ess#265): the command sent
    /// as that actor, [`ExpectNotGranted`](ScenarioStep::ExpectNotGranted), and whatever it claims
    /// absent.
    ///
    /// The one place an actor the specification does not grant the command is accepted, and only
    /// because the act says it must be refused. Everything only a command that ran could answer is
    /// refused beside it, and so is an act that names no actor to refuse: a suite step sent as no
    /// actor is sent as whoever the target is configured to be, which says nothing about a grant.
    fn refused_act(
        &mut self,
        act: &Act,
        command_ref: &CommandRef,
        input_fields: &[ess_compiler::ir::ResolvedField],
    ) {
        let mut claims = Vec::new();
        if act.actor.is_none() {
            claims.push("a missing `actor:`".to_owned());
        }
        for (key, written) in [
            ("outcome", act.outcome.is_some()),
            ("error", act.error.is_some()),
            ("response", act.response.is_some()),
            ("events", !act.events.is_empty()),
            ("capture", act.capture.is_some()),
        ] {
            if written {
                claims.push(format!("`{key}:`"));
            }
        }
        if !claims.is_empty() {
            self.refuse(Cause::RefusalContradicted {
                command: command_ref.clone(),
                claims,
            });
        }
        let actor = act
            .actor
            .as_ref()
            .and_then(|written| self.ungranted(written, command_ref));
        let input = self.values(
            &act.input,
            input_fields,
            &Surface::Input(command_ref.clone()),
            Completeness::Total,
        );
        self.steps.push(ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: command_ref.clone(),
            actor: actor.clone(),
            input,
        });
        // `no_events:` here is about the whole log, not the refused send's direct events: a refusal
        // has none, so only the log can show a command that ran before it was refused.
        let mut unpublished = Vec::new();
        for written in &act.no_events {
            if let Some((name, _)) = self.declared(written, self.ir.events(), |event| {
                Cause::UndeclaredEvent { event }
            }) {
                let event = EventRef::new(name);
                self.source.insert(event.clone().into());
                unpublished.push(event);
            }
        }
        if let Some(actor) = actor {
            self.steps
                .push(ScenarioStep::ExpectNotGranted { actor, unpublished });
        }
        self.mark(act);
    }

    /// The actor an act that expects its command to run is sent as, where the specification grants
    /// it the command.
    ///
    /// An act sent as no actor whose served command no actor is granted is refused: every caller is
    /// refused that command (beyond10x/ess#265), so the act requires what no conforming surface
    /// does.
    fn sender(&mut self, act: &Act, command: &CommandRef) -> Option<ActorRef> {
        let Some(written) = act.actor.as_ref() else {
            if ungranted_on_served_surface(self.ir, command.name()) {
                self.refuse(Cause::UngrantedOnServedSurface {
                    command: command.clone(),
                });
            }
            return None;
        };
        self.actor(written, command)
    }

    /// The actor an act expecting the refusal is sent as, where the specification declares it and
    /// does not grant it the command.
    fn ungranted(&mut self, written: &str, command: &CommandRef) -> Option<ActorRef> {
        let (name, declared) = self.declared(written, self.ir.actors(), |actor| {
            Cause::UndeclaredActor { actor }
        })?;
        let granted = declared
            .may
            .iter()
            .any(|handle| handle.name() == command.name());
        let actor = ActorRef::new(name);
        if granted {
            self.refuse(Cause::ActorGranted {
                actor,
                command: command.clone(),
            });
            return None;
        }
        self.source.insert(actor.clone().into());
        Some(actor)
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
        let (fixtures, literals): (BTreeMap<_, _>, BTreeMap<_, _>) = claim
            .payload
            .iter()
            .map(|(field, value)| (field.clone(), value.clone()))
            .partition(|(_, value)| matches!(value, Written::Fixture(_)));
        let payload = self.literals(&literals, &fields, &Surface::Payload(event.clone()));
        let mut dynamic = self.values(
            &fixtures,
            &fields,
            &Surface::Payload(event.clone()),
            Completeness::Partial,
        );
        let shape = payload_shape(self.ir, &event);
        self.observed.insert(event.clone());
        if dynamic.is_empty() {
            self.steps.push(ScenarioStep::ExpectEvent {
                event,
                payload,
                shape,
            });
        } else {
            dynamic.extend(
                payload
                    .into_iter()
                    .map(|(key, value)| (key, ScenarioValue::literal(value))),
            );
            self.steps.push(ScenarioStep::ExpectEventValues {
                event,
                payload: dynamic,
                shape,
            });
        }
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
        if let Some(after) = assertion.halts_after {
            self.halt(&view, bound, after, &order_by, style);
            return;
        }
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

    /// An early stop: the reader takes `after` rows, says stop, and the producer stops too.
    ///
    /// # Why this is not a seventh [`ViewExpectation`]
    ///
    /// Because the other six are decided against the rows a read returned, and this one cannot be.
    /// Two implementations that return the same rows differ on it — one pulls three, hands two on
    /// and throws the third away; one pulls two and stops — so a claim filed where the rows are the
    /// evidence is a claim that passes for a reason unrelated to itself. It changes the *read*
    /// instead, which is why it compiles to a step carrying the view and its parameters rather than
    /// to an expectation about a query some earlier step made.
    ///
    /// # Why the view has to declare an order
    ///
    /// `ESS-AUTHOR-024`, in a new place and for its own reason. A halt after two rows of an
    /// unordered listing says the reader stopped and says nothing about *what it read*, because the
    /// two rows are different rows on every read. The claim these scenarios exist to make is that a
    /// consumer of an ordered listing did not have to see the rest of it.
    fn halt(
        &mut self,
        view: &ViewRef,
        params: BTreeMap<String, ScenarioValue>,
        after: usize,
        order_by: &[Ranking],
        style: AssertionStyle,
    ) {
        if after == 0 {
            self.refuse(Cause::HaltsAtNothing { view: view.clone() });
            return;
        }
        if order_by.is_empty() {
            self.refuse(Cause::Unordered { view: view.clone() });
            return;
        }
        // The style is the model's, exactly as it is for every other claim about a view: a listing
        // the specification calls `eventual` may not hold the rows yet, and a scan of one that has
        // not caught up runs out before the reader stops it, which is lag rather than a wrong
        // implementation.
        self.steps.push(match style {
            AssertionStyle::Expect => ScenarioStep::ExpectHalt {
                view: view.clone(),
                params,
                after,
            },
            AssertionStyle::Eventually => ScenarioStep::EventuallyHalt {
                view: view.clone(),
                params,
                after,
            },
        });
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
            let checked = ess_compiler::expression::check_predicate(
                self.ir,
                fields,
                predicate,
                &format!("authored row {view}.satisfies"),
            );
            if let Some(diagnostic) = checked.errors.first() {
                if let Some(path) = &diagnostic.path {
                    self.refuse(Cause::UnreadablePredicate {
                        view: view.clone(),
                        path: path.to_string(),
                    });
                } else {
                    self.refuse(Cause::InvalidPredicate {
                        view: view.clone(),
                        diagnostic: diagnostic.clone(),
                    });
                }
                return None;
            }
            // A `defined()` or `missing()` of an `Optional` aggregate reads its presence, which a
            // row publishes as synthesis reads it (beyond10x/ess#176); such a suite takes
            // suite/26 when its format is selected against the model.
            let mut presence = BTreeSet::new();
            crate::input::presence_reads(predicate, &mut presence);
            for read in &checked.reads {
                if !projection_target(self.ir, &read.resolution).is_scalar()
                    && !crate::input::aggregate_presence(self.ir, read, &presence)
                {
                    self.refuse(Cause::UnreadablePredicate {
                        view: view.clone(),
                        path: read.path.to_string(),
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
        // A structured literal holding a reference is shape-checked whole; its references, here.
        let mut structured = BTreeMap::new();
        let mut positions = Vec::new();

        for (field, value) in written {
            match value {
                Written::Literal(node) if holds_reference(node) => {
                    let value = self.structured(
                        node,
                        Slot::of_field(fields, field),
                        surface,
                        &Place::field(field),
                        0,
                        false,
                        &mut positions,
                    );
                    structured.insert(field.clone(), value);
                    literals.insert(field.clone(), node.clone());
                }
                Written::Fixture(fixture) => {
                    referenced.insert(field.clone());
                    let target = fields.iter().find(|declared| declared.name == *field);
                    let compatible =
                        target
                            .zip(self.fixtures.get(fixture))
                            .is_some_and(|(target, source)| {
                                crate::fixtures::assignable(
                                    source,
                                    &crate::accessor::unresolve(&target.type_ref),
                                )
                            });
                    if !self.fixture_format || !compatible {
                        self.refuse(Cause::Unreadable { detail: format!("fixture {fixture} needs scenario/3, a declared type, and a compatible target field {field}") });
                    } else {
                        self.used_fixtures.insert(fixture.clone());
                        resolved.insert(
                            field.clone(),
                            ScenarioValue::Fixture {
                                fixture: fixture.clone(),
                            },
                        );
                    }
                }
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
            let errors = errors.without(|at| {
                positions.iter().any(|position: &String| {
                    at == position
                        || at
                            .strip_prefix(position.as_str())
                            .is_some_and(|rest| rest.starts_with('.'))
                })
            });
            if let Some(errors) = errors {
                self.shape(&errors, surface);
            }
        }
        for (field, node) in literals {
            let value = structured
                .remove(&field)
                .unwrap_or_else(|| ScenarioValue::literal(node));
            resolved.insert(field, value);
        }
        resolved
    }

    /// A literal that holds `{$instance: …}` below its top, checked position by position against
    /// the declared type there (beyond10x/ess#242).
    ///
    /// A reference is admitted where the declared type is the instance's identity type, exactly
    /// as a whole instance-valued field is, and refused anywhere else as
    /// [`InstanceMistyped`](Cause::InstanceMistyped), naming the position — `ring_sequence[1]`,
    /// `pair.primary`, `by_stage[canary]`, `target.value`. A union's payload is typed by the
    /// variant its tag names. A part that holds no reference stays a literal, so only what has to
    /// be resolved at run time is written as a [`List`](ScenarioValue::List) or
    /// [`Members`](ScenarioValue::Members).
    ///
    /// Every reference's position is pushed to `positions` as the shape check spells it, so what
    /// that check says about the mapping the reference is written as is not reported a second
    /// time: the reference is answered here, admitted or refused.
    ///
    /// A reference that nothing can place — at a member the model does not declare, or inside a
    /// value of the wrong shape — is refused where the shape check does not look, which is
    /// `unchecked`: inside a union, the shape check reads no member (it reads a map's values since
    /// beyond10x/ess#240).
    /// Elsewhere the shape check names the member or the shape, and this walk says nothing more,
    /// so one mistake is one refusal. Either way no reference reaches a target as the mapping it
    /// is written as.
    #[allow(clippy::too_many_arguments)]
    fn structured(
        &mut self,
        node: &Node,
        declared: Slot,
        surface: &Surface,
        place: &Place,
        depth: usize,
        unchecked: bool,
        positions: &mut Vec<String>,
    ) -> ScenarioValue {
        if depth > ess_domain::types::MAX_TYPE_DEPTH {
            return ScenarioValue::literal(node.clone());
        }
        if let Some(written) = reference(node) {
            let check = match &declared {
                Slot::Typed(type_ref) => (Some(type_ref), type_ref.to_string()),
                Slot::Inside(container) => (None, container.clone()),
                // Refused where the member or the shape is: by the shape check, or by the walk
                // over the value that holds it when the shape check does not look there.
                Slot::Undeclared => return ScenarioValue::literal(node.clone()),
            };
            positions.push(place.fact.clone());
            let instance = match InstanceName::new(written) {
                Ok(instance) => instance,
                Err(error) => {
                    self.refuse(Cause::Unreadable {
                        detail: format!(
                            "{}: `{}` names an instance: {error}",
                            place.written,
                            Written::INSTANCE
                        ),
                    });
                    return ScenarioValue::literal(node.clone());
                }
            };
            return self
                .instance_at(&instance, Some(check), surface, &place.written)
                .unwrap_or_else(|| ScenarioValue::literal(node.clone()));
        }
        if !holds_reference(node) {
            return ScenarioValue::literal(node.clone());
        }
        let (container, expected) = match declared {
            Slot::Typed(type_ref) => (
                Container::of(self.ir, &type_ref, 0),
                Some(type_ref.to_string()),
            ),
            Slot::Inside(container) => (Container::Opaque(container), None),
            Slot::Undeclared => (Container::Undeclared, None),
        };
        // A union is a mapping of its tag and its payload, and the shape check reads neither.
        let unchecked = unchecked || matches!(container, Container::Union { .. });
        if unchecked && !container.fits(node) {
            self.refuse(Cause::ValueRejected {
                surface: surface.clone(),
                detail: format!(
                    "{}: expected {}, found {}",
                    place.written,
                    expected.as_deref().unwrap_or("a declared value"),
                    node.type_name()
                ),
            });
        }
        match node {
            Node::Seq(items) => ScenarioValue::List {
                items: items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        let position = match &container {
                            Container::List(of) => Slot::Typed(of.clone()),
                            other => other.inside(),
                        };
                        self.structured(
                            item,
                            position,
                            surface,
                            &place.index(index),
                            depth + 1,
                            unchecked,
                            positions,
                        )
                    })
                    .collect(),
            },
            Node::Map(entries) => ScenarioValue::Members {
                members: entries
                    .iter()
                    .map(|(key, member)| {
                        let (position, child, inner) = self
                            .member(&container, entries, key, member, surface, place, unchecked);
                        let value = self.structured(
                            member,
                            position,
                            surface,
                            &child,
                            depth + 1,
                            inner,
                            positions,
                        );
                        (key.clone(), value)
                    })
                    .collect(),
            },
            // Only a sequence or a mapping can hold a reference below its top.
            _ => ScenarioValue::literal(node.clone()),
        }
    }

    /// Where member `key` of a mapping sits in `container`, how it is written, and whether the
    /// shape check leaves it unread. A member holding a reference that nothing declares is refused
    /// here when the shape check does not read it, naming the member.
    #[allow(clippy::too_many_arguments)]
    fn member(
        &mut self,
        container: &Container,
        entries: &BTreeMap<String, Node>,
        key: &str,
        member: &Node,
        surface: &Surface,
        place: &Place,
        unchecked: bool,
    ) -> (Slot, Place, bool) {
        let (slot, child, inner) = match container {
            Container::Struct(fields) => (
                fields
                    .iter()
                    .find(|field| field.name == key)
                    .map_or(Slot::Undeclared, |field| {
                        Slot::Typed(field.type_ref.clone())
                    }),
                place.member(key),
                unchecked,
            ),
            // The shape check reads a map's values at their ordinal in key order (beyond10x/ess#240),
            // so a value is checked there as a list element is.
            Container::Map(value) => {
                let ordinal = entries.keys().position(|at| at == key).unwrap_or_default();
                (
                    Slot::Typed(value.clone()),
                    place.key(key, ordinal),
                    unchecked,
                )
            }
            Container::Union {
                name,
                tag,
                variants,
            } => {
                let payload = if tag == "value" { "content" } else { "value" };
                let slot = if key == tag {
                    Slot::Inside(name.clone())
                } else if key == payload {
                    let variant = entries.get(tag).and_then(Node::as_text);
                    if let Some(type_ref) = variant.and_then(|variant| variants.get(variant)) {
                        Slot::Typed(type_ref.clone())
                    } else {
                        if holds_reference(member) {
                            self.refuse(Cause::ValueRejected {
                                surface: surface.clone(),
                                detail: format!(
                                    "{}: `{}` names no variant of `{name}`; it declares {}",
                                    place.written,
                                    variant.unwrap_or("nothing"),
                                    variants.keys().cloned().collect::<Vec<_>>().join(", ")
                                ),
                            });
                        }
                        Slot::Undeclared
                    }
                } else {
                    Slot::Undeclared
                };
                (slot, place.member(key), true)
            }
            other => (other.inside(), place.member(key), unchecked),
        };
        let undeclared = matches!(slot, Slot::Undeclared)
            && unchecked
            && holds_reference(member)
            && match container {
                Container::Struct(_) => true,
                Container::Union { tag, .. } => {
                    key != tag && key != if tag == "value" { "content" } else { "value" }
                }
                _ => false,
            };
        if undeclared {
            self.refuse(Cause::UndeclaredField {
                surface: surface.clone(),
                at: place.written.clone(),
                field: key.to_owned(),
            });
        }
        (slot, child, inner)
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
                // A reference inside a compared value is the same claim the format cannot make as
                // a whole-field one, refused by where it sits (beyond10x/ess#242) rather than read
                // as a mapping the value was supposed to equal.
                Some(node) if holds_reference(node) => {
                    let mut places = Vec::new();
                    references(node, &Place::field(field), &mut places);
                    for place in places {
                        self.refuse(Cause::NotComparable {
                            surface: surface.clone(),
                            field: place,
                        });
                    }
                }
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
        let declared = fields
            .iter()
            .find(|it| it.name == field)
            .map(|it| (Some(&it.type_ref), it.type_ref.to_string()));
        self.instance_at(instance, declared, surface, field)
    }

    /// An instance reference at `position`, checked against the arrangement and, where `declared`
    /// says what the position holds, against its type: the type there, if the model declares one
    /// the position has, and how the position's type is written.
    fn instance_at(
        &mut self,
        instance: &InstanceName,
        declared: Option<(Option<&ResolvedTypeRef>, String)>,
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
        if let Some((type_ref, written)) = declared {
            let identity = self.ir.entity_identity(&entity);
            if let Some(identity) = identity {
                if type_ref.is_none_or(|type_ref| type_ref.required() != identity.required()) {
                    self.refuse(Cause::InstanceMistyped {
                        instance: instance.clone(),
                        entity,
                        surface: surface.clone(),
                        field: field.to_owned(),
                        declared: written,
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

// ---- references inside a structured literal (beyond10x/ess#242) --------------------------------

/// The instance a node names, where it is written `{$instance: name}`.
fn reference(node: &Node) -> Option<&str> {
    match node.as_single_entry() {
        Some((Written::INSTANCE, name)) => name.as_text(),
        _ => None,
    }
}

/// Whether `{$instance: …}` is written anywhere in `node`, itself included.
fn holds_reference(node: &Node) -> bool {
    reference(node).is_some()
        || match node {
            Node::Seq(items) => items.iter().any(holds_reference),
            Node::Map(entries) => entries.values().any(holds_reference),
            _ => false,
        }
}

/// Where each `{$instance: …}` in `node` is written, as [`Place::written`] spells it.
fn references(node: &Node, place: &Place, found: &mut Vec<String>) {
    if reference(node).is_some() {
        found.push(place.written.clone());
        return;
    }
    match node {
        Node::Seq(items) => {
            for (index, item) in items.iter().enumerate() {
                references(item, &place.index(index), found);
            }
        }
        Node::Map(entries) => {
            for (key, entry) in entries {
                references(entry, &place.member(key), found);
            }
        }
        _ => {}
    }
}

/// Where a part of a structured literal sits in the declared type.
enum Slot {
    /// At a member the model declares, of this type.
    Typed(ResolvedTypeRef),
    /// Inside a value whose parts the model does not type one by one — a union, `Json` — which
    /// is named, because no part of it is an identity.
    Inside(String),
    /// At a member the model does not declare, which the shape check reports.
    Undeclared,
}

impl Slot {
    /// Where input field `name` sits: at its declared type, or nowhere the model declares.
    fn of_field(fields: &[ResolvedField], name: &str) -> Self {
        fields
            .iter()
            .find(|declared| declared.name == name)
            .map_or(Self::Undeclared, |declared| {
                Self::Typed(declared.type_ref.clone())
            })
    }
}

/// What a declared type is made of, for a structured literal walking into it.
enum Container {
    /// A list of this.
    List(ResolvedTypeRef),
    /// A map whose values are this.
    Map(ResolvedTypeRef),
    /// A struct with these members.
    Struct(Vec<ResolvedField>),
    /// A union: its tag field, and each variant's payload type by tag value.
    Union {
        /// The union's type, as written.
        name: String,
        /// The field carrying the variant's name.
        tag: String,
        /// The payload type of each variant.
        variants: BTreeMap<String, ResolvedTypeRef>,
    },
    /// A value whose parts are not typed one by one, by the name of its type.
    Opaque(String),
    /// Nothing the model declares.
    Undeclared,
}

impl Container {
    /// Whether `node` has the shape this container is: a sequence for a list, a mapping for a
    /// map, a struct or a union. An opaque or undeclared container is answered elsewhere.
    fn fits(&self, node: &Node) -> bool {
        matches!(
            (self, node),
            (Self::List(_), Node::Seq(_))
                | (
                    Self::Map(_) | Self::Struct(_) | Self::Union { .. },
                    Node::Map(_)
                )
                | (Self::Opaque(_) | Self::Undeclared, _)
        )
    }

    /// What `type_ref` is made of, through every `Optional` and newtype around it.
    fn of(ir: &EssIr, type_ref: &ResolvedTypeRef, depth: usize) -> Self {
        if depth > ess_domain::types::MAX_TYPE_DEPTH {
            return Self::Undeclared;
        }
        match type_ref {
            ResolvedTypeRef::Optional { of } => Self::of(ir, of, depth + 1),
            ResolvedTypeRef::List { of } => Self::List(of.as_ref().clone()),
            ResolvedTypeRef::Map { value, .. } => Self::Map(value.as_ref().clone()),
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => Self::of(ir, of, depth + 1),
                ResolvedBody::Struct { fields, .. } => Self::Struct(fields.clone()),
                ResolvedBody::Union { tag, variants } => Self::Union {
                    name: type_ref.to_string(),
                    tag: tag.clone(),
                    variants: variants.clone(),
                },
                ResolvedBody::Enum { .. } => Self::Opaque(type_ref.to_string()),
            },
            ResolvedTypeRef::Primitive { .. } => Self::Opaque(type_ref.to_string()),
        }
    }

    /// Where a part of a value of this shape sits, when the part is not one the shape has: inside
    /// an opaque value, or where the shape check reports the mismatch.
    fn inside(&self) -> Slot {
        match self {
            Self::Opaque(name) => Slot::Inside(name.clone()),
            Self::List(_)
            | Self::Map(_)
            | Self::Struct(_)
            | Self::Union { .. }
            | Self::Undeclared => Slot::Undeclared,
        }
    }
}

/// A position inside an input field, written for a person and as the shape check spells it.
struct Place {
    /// `ring_sequence[1]`, `pair.primary`, `by_stage[canary]`.
    written: String,
    /// `ring_sequence.1`, `pair.primary`: the path a shape error names.
    fact: String,
}

impl Place {
    /// The field itself.
    fn field(name: &str) -> Self {
        Self {
            written: name.to_owned(),
            fact: name.to_owned(),
        }
    }

    /// Element `index` of a list.
    fn index(&self, index: usize) -> Self {
        Self {
            written: format!("{}[{index}]", self.written),
            fact: format!("{}.{index}", self.fact),
        }
    }

    /// Member `name` of a struct.
    fn member(&self, name: &str) -> Self {
        Self {
            written: format!("{}.{name}", self.written),
            fact: format!("{}.{name}", self.fact),
        }
    }

    /// The value at `key` of a map, which the shape check names by its `ordinal` in key order.
    fn key(&self, key: &str, ordinal: usize) -> Self {
        Self {
            written: format!("{}[{key}]", self.written),
            fact: format!("{}.{ordinal}", self.fact),
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
        if self.halts_after.is_some() {
            stated.push("halts_after");
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
