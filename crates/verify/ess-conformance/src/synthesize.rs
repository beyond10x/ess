//! Turning a resolved specification into the suite it obliges an implementation to pass.
//!
//! Design §10 through §20, and the contract §36 draws around them: this is derivation from an
//! [`EssIr`], like every projection in `ess-gen`, and it is the one derivation that may **refuse**.
//!
//! ```text
//! EssIr ──► ConformanceSuite   what an implementation must satisfy
//!       └─► Vec<Refusal>       what this specification does not yet say enough for
//! ```
//!
//! # A refusal is a result, not a defect
//!
//! [`synthesize`] returns both, always, and the reason is §36's: a suite that quietly holds fewer
//! checks than the specification requires is the "generated tests are green" failure the whole
//! milestone exists to rule out, and unlike a refusal, nothing about it is visible in a passing run.
//! So every construct that gets no scenario appears in [`Synthesis::refusals`] saying which
//! construct, why, and what would have to change — the shape §28 uses for a required semantic a
//! target cannot expose, applied one stage earlier.
//!
//! # Three decisions this module does not take
//!
//! Each was taken once, in the model, and asking again here is the regression the design names by
//! name: *a decision made per projection is a decision made wrong eventually*.
//!
//! | question | answered by | never by |
//! |---|---|---|
//! | can an input reach this branch? | [`ResolvedOutcome::test_strategy`] | re-inspecting the predicate |
//! | `expect` or `eventually`? | [`ResolvedView::assertion_style`] | re-reading the consistency word |
//! | does this candidate satisfy the guard? | [`InputFacts::decide`](crate::InputFacts::decide) | a second evaluator |
//!
//! # `Unknown` refuses
//!
//! §11, and invariant 5 read from the generator's side. A candidate whose guard evaluates to
//! `True` is kept and one that evaluates to `False` is discarded, but `Unknown` ends the search:
//! five of its six causes are properties of the specification that no value can change, so retrying
//! spends the whole budget on a specification defect and then reports it as a flaky test.
//!
//! # What is asserted, and what is deliberately not
//!
//! An outcome scenario asserts the branch, the declared error, every event the branch emits — with
//! the payload **shape** the specification declares for it — and, first class per §10, every event
//! the specification declares that this branch does **not** emit.
//!
//! Where the line falls is worth stating, because it is the model's line and not a budget:
//!
//! | about an event's payload | asserted | why |
//! |---|---|---|
//! | the declared fields are present | yes | the event declares them |
//! | each holds a value of its declared type | yes | the event declares that too |
//! | a field carries a particular *value* | where the outcome's `payload:` determines it | the declaration says which input or literal fills it |
//! | no undeclared field is present | **no** | nothing in the model closes an event's payload |
//!
//! `InvoiceCreated.amount == CreateInvoice.amount` reads like a reading and, without a
//! declaration, is a match on a shared field name; a specification that called the input `total`
//! would make the same suite wrong. So the values follow from the outcome's `payload:` — the
//! shape a binding's `mapping:` already has for a command input — and from nothing else: a field
//! with no declared source stays undetermined, covered by the shape and by no value, which is a
//! fact about the specification this suite shows rather than an error. A declared error's fields
//! have no such construct yet and are asserted by name and never by value.
//!
//! # A lifecycle scenario is a sequence over one instance
//!
//! §19's two classes — the move that must be possible, and the move that must not — are sequences:
//! bring an instance into existence, drive it to the state in question, then act. Every step after
//! the first has to say *which* instance, and until an outcome declared
//! [`instance:`](ess_domain::command::Subject::instance) nothing in the model could. Both classes
//! were refused wholesale; both are synthesised now.
//!
//! What makes it honest rather than convenient is where the identity comes from. Nothing here
//! fabricates one: the arrangement runs the outcome the specification says *creates* the entity, and
//! binds the identity out of the event that outcome declares publishes it — through
//! [`ScenarioStep::CaptureInstance`] — so the value is the target's, and the suite carries a
//! reference to it rather than a guess at it.
//!
//! # What a scenario pins beyond the branch it names
//!
//! beyond10x/ess#111 audited a real implementation and found mutants of declared behaviour that
//! changed no verdict. Four rules close them, each reusing steps the format already has:
//!
//! | declared | pinned by |
//! |---|---|
//! | a move's `to` | the row is required in that state wherever the view projects `state` (`lifecycle_state`) |
//! | every state in a move's `from` | one further instance per further source, moved and read back (`other_sources`) |
//! | a `sets:` entry of an update | a witness off the value the row already holds (`freshened`) |
//! | an ordered comparison with a literal | the branch at the boundary, its default at the neighbour across it (`boundaries`) |
//!
//! # A binding is four claims, and each one fails on its own
//!
//! §16 through §18. *When this event occurs, invoke this command, with this mapping, delivered this
//! way, and on failure do this* — so a binding produces four scenarios rather than one, under the
//! four [`BindingAspect`] keys, and each says what it can prove from
//! the declared semantics and nothing more:
//!
//! | aspect | what it requires | why that and not more |
//! |---|---|---|
//! | `flow` | the event happens, and the invoked command's branch publishes what it declares | §16: prove the flow through the resulting event, never by observing the internal command |
//! | `mapping` | each input receives the value the binding names for it | the only clause a document can get *silently* wrong — see [`ScenarioStep::ExpectInvocation`] |
//! | `delivery` | under `at_least_once`, the same event delivered twice still leaves the consequence observable; under `at_most_once`, nothing | §17: `at_least_once` permits duplicates, so a count is a test that fails a correct target, and `at_most_once` permits no second arrival to test |
//! | `on-failure` | the declared policy is observable, with the failure forced | §18: force it, and assert what the model says follows |
//!
//! Two of them produce no scenario at all where the model says so, and that is the model being
//! honest rather than this crate being short. `on_failure: drop` means the work is lost and nobody
//! is told, so there is nothing to observe; §18 names the response — refuse the check rather than
//! invent one — and [`BindingGap::PolicySilent`] is it. `delivery: at_most_once` says the event is
//! delivered once and never again, so the redelivery §17 asks for is a thing the specification
//! states will not happen; [`BindingGap::DeliverySingleAttempt`] refuses it for that reason, and
//! [`ScenarioStep::RedeliverEvent`] never appears for such a binding.
//!
//! # An invariant is asserted where a view publishes what it reads
//!
//! §20, at the level it asks for: *evaluate invariants after successful state-changing commands,
//! against observable entity or view state, where witnesses are available*. So one scenario per
//! entity per state-changing branch, running that branch and then requiring that every row of a view
//! satisfies the entity's invariants — [`ViewExpectation::Satisfies`], carrying the predicate the
//! specification wrote.
//!
//! Where the witness is missing, it is missing *by construction*: an entity's invariant reads the
//! entity's fields and a view publishes only what it declares, so an invariant over a field no view
//! publishes is unobservable no matter how good the runner is. That is a fact about the
//! specification, and [`RefusalCause::InvariantUnobservable`] says so with the paths in hand.
//!
//! # A wrong-state refusal is asserted where the command declares one
//!
//! §19 asks for two things of an illegal move, and until `wrong_state:` existed the model could
//! express only one. "Must not reach `Cancelled`" was asserted; "the exact rejection mechanism must
//! come from the declared command/error semantics" was not, because a command had no way to say what
//! it answers when its subject is in a state its transitions do not run from. An implementation that
//! refused with the wrong error, or with an untyped infrastructure failure, passed.
//!
//! It can say so now, and it says exactly one thing:
//! [`OutcomeCondition::WrongState`](ess_domain::command::OutcomeCondition::WrongState) names the
//! `error:` and nothing else. The **states** stay where they were already declared — a transition's
//! `from` set — and [`EssIr::wrong_states`] does the subtraction, so no author writes an absence down
//! and no projection re-derives it. Where a command declares the branch, an illegal-move scenario
//! requires it *and* the declared error; where it does not,
//! [`RefusalCause::RefusalUndeclared`] is recorded beside the scenario, which is the same
//! arrangement as before for the specifications that have not adopted the construct.
//!
//! # What an outcome determines, and what is therefore asserted
//!
//! Two blocks on an outcome relate the input a command was given to something an observer can
//! read, and each licenses a class of assertion that was a refusal before it existed:
//!
//! | block | fills | what is asserted with it | what is asserted without it |
//! |---|---|---|---|
//! | `payload:` | a field of an event the branch emits | the field carries the value the input supplied | the field is present and of its declared type |
//! | `sets:` | a field of the entity the branch acts on | every view that projects that field holds a row carrying the value | the row is found by its identity and nothing is said about what it holds |
//!
//! `sets:` is read in two steps, `settled` and `shown`: a source that is a literal the target's
//! representation cannot be written as, that crosses a declared conversion, or that fills a field
//! no view projects at the entity's own type is left out rather than guessed at, because in each
//! the value the row holds is not the value sent. A literal over text or an enum variant IS the
//! value, so `campaign_id: ""` is asserted as the empty string rather than dropped. Where the source
//! is an input field the arrangement pointed at a row it created — the field carrying the subject's
//! owner — what is asserted is that reference: the row holds *that* owner's id, which is a claim
//! about a value neither the suite nor the specification can spell.
//!
//! # What the model cannot say yet, and what is therefore not asserted
//!
//! One gap left, reported rather than left to be noticed: **which** row a ranked view puts first.
//!
//! The values are now known — `sets:` says where each row's ranking value came from, and the
//! scenario chose it — so the order over the rows *this scenario made* is computable. What is not
//! known is whether they are the only rows. §8 permits a target to be shared between scenarios, so
//! a row somebody else put there could outrank all of them, and `first` would be a claim about the
//! other user of the target rather than about the implementation. It is the same reading that makes
//! [`ViewExpectation::Counts`] a floor and never a ceiling.
//!
//! What would close it is a view saying it holds only what the scenario running it put there.
//! Until then [`ViewExpectation::Ranked`] asserts that the rows are in the declared order and
//! [`ViewExpectation::Contains`] asserts what each of them holds, which is together every part of
//! the claim except the one about the rows nobody in this scenario made.
//!
//! A value object's invariants are the third: `billing.invoice.Money` says `amount >= 0` of every
//! `Money` in the system, which is a claim about a *type* rather than about an instance at rest, and
//! rebasing it onto every entity field that reaches that type is a walk this slice does not do. That
//! one is a gap in this crate rather than in the model, and it is
//! [`RefusalCause::NotSynthesisedYet`].

mod absent_input;
mod aggregate;
mod binding_condition;
mod binding_effects;
mod bounded_retry;
mod caller;
mod delivery_context;
mod disclosure;
mod existence;
mod grant;
mod identity;
mod paging;
mod related;
mod related_guard;
mod set_effects;
mod singleton;
mod subject_fact;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use ess_compiler::diagnostic::Code;
use ess_compiler::ir::{
    Driver, EntityHandle, EssIr, ResolvedBinding, ResolvedBody, ResolvedCommand, ResolvedComponent,
    ResolvedCondition, ResolvedEffect, ResolvedFailure, ResolvedInstance, ResolvedOutcome,
    ResolvedPayloadValue, ResolvedSubject, ResolvedType, ResolvedTypeRef, ResolvedView,
};
use ess_domain::binding::Delivery;
use ess_domain::command::{OutcomeName, TestStrategy};
use ess_domain::entity::{Cardinality, EntitySpec, Invariant, StateName};
use ess_domain::name::QualifiedName;
use ess_domain::types::MAX_TYPE_DEPTH;
use ess_domain::view::AssertionStyle;
use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate, Quantified, Truth};
use ess_primitives::time::Rfc3339Instant;

use crate::decision::{when, Decision, Unevaluable};
use crate::input::{flatten, predicate_projectable, resolve_path, ShapeErrors};
use crate::scenario::{
    ActorRef, BindingAspect, BindingRef, CommandRef, ComponentRef, ConformanceScenario,
    ConformanceSuite, DeclaredTypeRef, EntityRef, ErrorRef, EssSemanticRef, EventRef, Holds,
    InstanceName, LeafShape, OutcomeRef, PayloadShape, ScenarioId, ScenarioPurpose, ScenarioStep,
    ScenarioValue, SuiteProvenance, TransitionRef, ViewExpectation, ViewRef,
};
use crate::witness::{candidates, Distinction, WitnessGap, MAX_CANDIDATES};

/// Everything one specification obliges an implementation to pass, and everything it does not say
/// enough for.
///
/// Both halves, in one value, because they are one answer. A caller that wants only the suite is a
/// caller that has decided refusals do not matter, and making that take a second line of code is
/// the point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Synthesis {
    /// The scenarios that could be synthesised.
    pub suite: ConformanceSuite,
    /// Every construct that could not, in the order the model declares them.
    pub refusals: Vec<Refusal>,
    /// The scenarios the specification obliges and this suite does not hold, because it was
    /// scoped to one component and they need another.
    ///
    /// Empty for a whole-system suite. Not a refusal: the specification said enough, and the
    /// scenario exists — in the suite for the component that realises what it needs. Listed
    /// rather than dropped for the reason refusals are listed: a suite that quietly holds fewer
    /// checks than the specification demands is the one failure a passing run cannot show.
    pub outside: Vec<Outside>,
    /// What the specification leaves undeclared that no scenario can therefore hold an
    /// implementation to, in the order the model declares the constructs.
    ///
    /// Not a refusal either: a refusal is a construct the specification states and this suite
    /// cannot check, and a note is a question the specification does not answer at all. Listed
    /// for the reason both of the others are — a silence is invisible in a passing run.
    pub notes: Vec<Note>,
}

/// A question the specification leaves unanswered, so that no scenario is owed for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// A command acting on an input-named instance declares neither a not-found outcome nor a
    /// `wrong_state` outcome, so what it answers when that identity names no record is undeclared
    /// (`docs/design/typed-literals-and-unknown-instances.md`, section 2).
    UnknownInstanceUnanswered {
        /// The command.
        command: CommandRef,
    },
    /// A command acting on an input-named instance declares more than one outcome that could be
    /// its not-found answer, so which one an unknown identity takes is undeclared
    /// (`docs/design/typed-literals-and-unknown-instances.md`, section 2a).
    UnknownInstanceAmbiguous {
        /// The command.
        command: CommandRef,
        /// The candidate outcomes, in declaration order.
        outcomes: Vec<OutcomeName>,
    },
    /// A wrong-state refusal whose subject the declared views publish only in part, so its scenario
    /// observes what they publish and nothing more (beyond10x/ess#132). No view says how the rest
    /// of the row reads, so a refusal that changed it is not checked.
    PartialObservation {
        /// The refusal scenario.
        scenario: ScenarioId,
        /// The subject fields no view lets it observe, in name order.
        unobserved: Vec<String>,
    },
    /// A branch whose `sets:` read one input into two targets while a same-typed input feeds none —
    /// the pair a `sets-retarget` mutant joins — is sent with the two inputs equal, because no input
    /// its guards leave tells them apart (beyond10x/ess#202). Its scenario then cannot tell the
    /// model it was synthesized from from the one that writes the unread input.
    UnseparatedSources {
        /// The branch's scenario.
        scenario: ScenarioId,
        /// The input read into two or more targets.
        source: String,
        /// The same-typed input no target reads.
        unread: String,
    },
    /// Two branches whose guards may both hold, where the declared precedence answers with `first`
    /// (`docs/design/input-guard-overlap-precedence.md`), and `first`'s scenario sends no input in
    /// that overlap (beyond10x/ess#217). A target answering `other` there is not failed by it.
    UnwitnessedOverlap {
        /// The scenario of the branch taken first.
        scenario: ScenarioId,
        /// The branch taken first: an input-guarded refusal, or the accepting branch declared
        /// first.
        first: OutcomeName,
        /// The branch it is taken before.
        other: OutcomeName,
        /// Why no input in the overlap is sent.
        gap: OverlapGap,
    },
    /// A scenario sending a command that reads the caller keeps its first run only: the run with
    /// the two callers' roles swapped would send a caller-supplied identity again, and the input's
    /// type has too few values to give it one no scenario sends (beyond10x/ess#275). A target that
    /// answers one caller by name is not failed by it.
    UnswappedCallers {
        /// The scenario.
        scenario: ScenarioId,
        /// The input that becomes the created identity.
        input: String,
        /// Its declared type.
        type_ref: String,
    },
    /// The mixed-caller scenario is retained, but its reversed-role counterpart cannot compose.
    CrossCallerUnswapped {
        /// The retained scenario.
        scenario: ScenarioId,
        /// The concrete arrangement or observation restriction.
        reason: &'static str,
    },
    /// The ordinary witness remains, but no source-authorized mixed caller witness composes.
    CrossCallerUnwitnessed {
        /// The retained ordinary scenario.
        scenario: ScenarioId,
        /// Why crossing callers cannot retain its source-selected expectations.
        reason: String,
    },
    /// A branch copying fields of the row its `when_related:` guard reads (`{related: …}` through
    /// the same input) whose scenario arranges no second row the guard accepts holding other values
    /// there (beyond10x/ess#270). A target that copies from a row the guard accepts, rather than
    /// the row the input names, is not failed by it.
    UnaccompaniedRelatedCopy {
        /// The branch's scenario.
        scenario: ScenarioId,
        /// The fields copied, in the order first read.
        fields: Vec<String>,
    },
    /// Every declared actor holds the grant for a command, so no actor is refused it and no
    /// `…/grant/denied` scenario is owed (beyond10x/ess#265). The coverage fact that stands in for
    /// the scenario a command some actor lacks the grant for gets.
    GrantedToEveryActor {
        /// The command.
        command: CommandRef,
    },
    /// The model declares actors and serves no component (`reached_by: network`), so no
    /// `…/grant/denied` scenario is owed (beyond10x/ess#265): the standard refusal is the served
    /// contract's, and where nothing is served enforcing a grant is the caller's, against the
    /// generated grant table. The coverage fact that stands in for the scenarios a served model gets.
    GrantEnforcedByCaller,
    /// A command a served component accepts is granted to no declared actor, so every caller is
    /// refused it, and no scenario may send it expecting it to run (beyond10x/ess#265). Its
    /// `…/grant/denied` scenario is its only witness; the scenarios synthesis would otherwise have
    /// sent it in, as no actor, are withheld and named here.
    GrantedToNoActor {
        /// The command.
        command: CommandRef,
        /// The scenarios withheld, in id order.
        withheld: Vec<ScenarioId>,
    },
    /// A command a served component accepts, which some declared actor lacks the grant for, has no
    /// scenario that sends it into an accepting branch, so no `…/grant/denied` scenario is
    /// synthesized for it (beyond10x/ess#265). A refusal asked of a send the command would have
    /// refused anyway shows nothing about the grant.
    GrantDeniedUnwitnessed {
        /// The command.
        command: CommandRef,
    },
    /// A command a served component accepts is held by two or more actors, and at least one of them
    /// carries attributes, so its scenarios keep the actor synthesis chose rather than being sent
    /// by each granted actor in turn (beyond10x/ess#265): their expectations are read for that
    /// actor's values. Not every actor granted the command is shown to send it.
    GrantRotationSkipped {
        /// The command.
        command: CommandRef,
        /// The actors holding it that carry attributes, in name order.
        attributed: Vec<ActorRef>,
    },
}

impl fmt::Display for Note {
    #[allow(clippy::too_many_lines)] // Keep the complete note vocabulary in one exhaustive match.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownInstanceUnanswered { command } => write!(
                f,
                "`{command}` declares no not-found and no `wrong_state` outcome, so what it \
                 answers when its `instance:` names no record is undeclared and not checked"
            ),
            Self::UnknownInstanceAmbiguous { command, outcomes } => {
                let names: Vec<String> = outcomes.iter().map(|name| format!("`{name}`")).collect();
                write!(
                    f,
                    "`{command}` declares {} as externally decided refusals reporting its \
                     `instance:` identity, so which one it answers when that identity names no \
                     record is undeclared and not checked",
                    names.join(" and ")
                )
            }
            Self::PartialObservation {
                scenario,
                unobserved,
            } => {
                let names: Vec<String> =
                    unobserved.iter().map(|name| format!("`{name}`")).collect();
                write!(
                    f,
                    "`{scenario}` observes the refused subject through what its views publish; no \
                     view publishes {}, so a refusal that changed {} is not checked, and a field \
                     published only by an `eventual` view is checked against an eventually \
                     consistent read, which a projection that has not yet caught up with a wrong \
                     change still passes",
                    names.join(", "),
                    if names.len() == 1 { "it" } else { "them" }
                )
            }
            Self::UnseparatedSources {
                scenario,
                source,
                unread,
            } => write!(
                f,
                "`{scenario}` sends `{source}` and `{unread}` with one value: `{source}` feeds two \
                 or more `sets:` targets while `{unread}` feeds none, and no input its guards leave \
                 tells them apart, so a model writing `{unread}` where this one writes `{source}` \
                 passes it too"
            ),
            Self::UnwitnessedOverlap {
                scenario,
                first,
                other,
                gap,
            } => {
                let why = match gap {
                    OverlapGap::Unreached => {
                        "no candidate input tried lies where both guards hold and nothing taken \
                         before `{first}` does, and the candidates do not show that no input does"
                    }
                    OverlapGap::Unsent => {
                        "an input lies there, and the scenario is arranged by a search that does \
                         not send it (a held state that admits it only beside another branch, a \
                         stored row, a replay or a preserved subject)"
                    }
                };
                write!(
                    f,
                    "`{scenario}` sends no input that both `{first}` and `{other}` select, where \
                     the declared precedence answers `{first}`: {}; a target answering `{other}` \
                     there is not failed",
                    why.replace("{first}", first.as_str())
                )
            }
            Self::UnswappedCallers {
                scenario,
                input,
                type_ref,
            } => write!(
                f,
                "`{scenario}` runs once, with the callers' roles as first assigned: its run with \
                 them swapped would send `{input}` again, and `{type_ref}` has too few values to \
                 give it an identity no other scenario sends, so a target that answers one caller \
                 by name is not failed by it"
            ),
            Self::CrossCallerUnswapped { scenario, reason } => write!(f,
                "`{scenario}` retains its mixed-caller witness without an appended reversed-role run: {reason}"),
            Self::CrossCallerUnwitnessed { scenario, reason } => write!(f,
                "`{scenario}` retains its ordinary witness without a mixed-caller same-row witness: {reason}"),
            Self::UnaccompaniedRelatedCopy { scenario, fields } => {
                let names: Vec<String> = fields.iter().map(|name| format!("`{name}`")).collect();
                write!(
                    f,
                    "`{scenario}` copies {} from the row its `when_related:` guard reads, and no \
                     second row the guard accepts holding another value there could be arranged, \
                     so a target copying from a row the guard accepts rather than the row named \
                     is not failed by it",
                    names.join(", ")
                )
            }
            Self::GrantedToEveryActor { .. }
            | Self::GrantDeniedUnwitnessed { .. }
            | Self::GrantRotationSkipped { .. }
            | Self::GrantedToNoActor { .. }
            | Self::GrantEnforcedByCaller => self.grant(f),
        }
    }
}

impl Note {
    /// The notes about who may send what on a served surface (beyond10x/ess#265), kept apart so
    /// each rendering stays beside its own variant.
    fn grant(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GrantedToEveryActor { command } => write!(
                f,
                "every declared actor may invoke `{command}`, so no actor is refused it and no \
                 `{command}/grant/denied` scenario is owed"
            ),
            Self::GrantDeniedUnwitnessed { command } => write!(
                f,
                "no scenario sends `{command}` into a branch that accepts it, so `{command}/grant/\
                 denied` is not synthesized: a refusal of a send the command would have refused \
                 anyway shows nothing about the grant"
            ),
            Self::GrantRotationSkipped {
                command,
                attributed,
            } => {
                let names: Vec<String> = attributed.iter().map(|actor| format!("`{actor}`")).collect();
                write!(
                    f,
                    "`{command}` is held by {}, which carry attributes its scenarios are read for, so \
                     they keep the actor synthesis chose and not every actor granted `{command}` is \
                     shown to send it",
                    names.join(", ")
                )
            }
            Self::GrantedToNoActor { command, withheld } => {
                write!(
                    f,
                    "no declared actor may invoke `{command}`, which a served component accepts, so \
                     every caller is refused it and `{command}/grant/denied` is its only witness"
                )?;
                if !withheld.is_empty() {
                    let names: Vec<String> =
                        withheld.iter().map(|id| format!("`{id}`")).collect();
                    write!(f, "; withheld: {}", names.join(", "))?;
                }
                Ok(())
            }
            Self::GrantEnforcedByCaller => f.write_str(
                "the specification declares actors and serves no component, so enforcing a grant is \
                 the caller's, against the generated grant table, and no `<command>/grant/denied` \
                 scenario is owed",
            ),
            _ => Ok(()),
        }
    }
}

impl Synthesis {
    /// `true` when every construct of the specification produced a scenario.
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

/// One construct that got no scenario, and why.
///
/// The shape §36 asks for — "a stable code, a structured body, and the ESS element that caused it",
/// the same shape `ess-compiler` uses for a bad document, because a coding agent consumes both as
/// repair instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The ESS element that has no scenario.
    pub subject: EssSemanticRef,
    /// The scenario that would have existed, where the refusal is about one.
    ///
    /// `None` for a construct whose scenario has no single id — a binding has two aspects, and
    /// neither is synthesised yet.
    pub scenario: Option<ScenarioId>,
    /// Why, as fields rather than as a sentence.
    pub cause: RefusalCause,
}

impl Refusal {
    /// A refusal about the scenario that would have carried `id`.
    fn about(id: &ScenarioId, cause: RefusalCause) -> Self {
        Self {
            subject: subject_of(id),
            scenario: Some(id.clone()),
            cause,
        }
    }

    /// Its stable code.
    pub fn code(&self) -> Code {
        self.cause.code()
    }

    /// What would have to change for this construct to be testable.
    pub fn hint(&self) -> &'static str {
        self.cause.hint()
    }
}

impl fmt::Display for Refusal {
    /// Names the scenario that is missing, not only the construct it is about.
    ///
    /// The id is what a fault matrix and a stored report key on, and it is the only thing that
    /// tells two refusals about one entity apart: `Invoice/state/Paid/refuses/CancelInvoice` and
    /// `Invoice/state/Paid/refuses/IssueInvoice` share a subject and are different checks.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scenario {
            // The scenario exists; one absence it should witness is missing from it (ess/22,
            // beyond10x/ess#285).
            Some(id) if matches!(self.cause, RefusalCause::AbsenceUnwitnessed { .. }) => writeln!(
                f,
                "refusal[{}]: {} has a scenario `{id}` that leaves an absent reference unwitnessed",
                self.code(),
                self.subject
            ),
            Some(id) => writeln!(
                f,
                "refusal[{}]: {} has no scenario `{id}`",
                self.code(),
                self.subject
            ),
            None => writeln!(
                f,
                "refusal[{}]: {} has no scenario",
                self.code(),
                self.subject
            ),
        }?;
        for line in self.cause.to_string().lines() {
            writeln!(f, "  {line}")?;
        }
        write!(f, "  help: {}", self.hint())
    }
}

/// Why one construct got no scenario.
///
/// Five outcomes of synthesis, one statement about this build, and three drift alarms. The split
/// matters to whoever reads a refusal: the first five are answered by editing the specification,
/// the sixth by a later slice of this crate, and the last three cannot happen — they are here so
/// that if they ever do, they arrive as a named refusal rather than as a scenario nobody wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalCause {
    /// No value of the command's declared input type could be constructed at all.
    NoWitness(WitnessGap),
    /// The guard could not be decided against a candidate, and no other candidate would change it.
    ///
    /// §11's `Unknown`, carrying every leaf that could not be decided and
    /// [why](crate::Reason) — which is the difference between "supply the value" and
    /// "fix the specification".
    GuardUnevaluable(Unevaluable),
    /// Every candidate this synthesizer knows how to try was refuted.
    ///
    /// A valid predicate that is not constructively satisfiable by generate-and-check. §11 names
    /// the repair — a constraint solver — as a later extension, so this is a refusal rather than a
    /// longer search.
    GuardUnsatisfiable {
        /// What had to hold, as it reads.
        predicate: String,
        /// How many candidates were decided against it.
        tried: usize,
    },
    /// The scenario needed an instance of an entity, and the specification cannot arrange one.
    ///
    /// The model now says which field names an instance, so "the invoice the previous step created"
    /// is expressible and §19's scenarios are synthesised. What is left here is the case where the
    /// specification cannot *produce* the instance the scenario needs: nothing creates the entity at
    /// all, no sequence of declared moves reaches the state, or a command on the only route has no
    /// witness. Each is a property of the specification, and each is reported rather than papered
    /// over with a fabricated identity — which would be a test that fails a correct implementation.
    InstanceRequired {
        /// Whose instance.
        entity: EntityRef,
        /// What the scenario would have needed of it.
        need: InstanceNeed,
        /// Why the specification cannot produce one.
        reason: Unreachable,
    },
    /// A view's filter could not be decided against the state the scenario reaches.
    ///
    /// A synthesised scenario knows one fact about the entity it created: the state its lifecycle
    /// starts in. A filter reading anything else is undecidable here for the same reason a guard
    /// over an unbound path is, and asserting the view either way would be a guess.
    ViewUndecidable {
        /// Which view.
        view: ViewRef,
        /// Its filter, as it reads.
        filter: String,
        /// The state the entity is in when the assertion would run.
        state: StateName,
        /// The paths the filter reads that nothing binds.
        unbound: Vec<FactPath>,
    },
    /// A view declares an order and the scenario cannot put two rows in it.
    ///
    /// [`ViewExpectation::Ranked`] compares adjacent pairs, so a view holding fewer than two rows
    /// holds its order for every implementation there is. That is deliberate on the assertion's side
    /// — an order over no rows is simply held, and demanding a row would make every ordering claim a
    /// non-emptiness claim as well — and it is exactly why the *scenario* has to arrange the second
    /// row instead. Where the specification cannot produce one, the honest result is this refusal
    /// and no assertion, because an assertion no implementation can fail is worse than a missing
    /// one: it looks like a present check in every report that reads it.
    ///
    /// The instances are arranged the way every other instance in a suite is — the declared creating
    /// outcome, then the declared moves that reach a state the view's filter admits. There is no
    /// seeding route around that, and there is deliberately none: a target asked to put a row
    /// somewhere would be a target answering a question only a test asks.
    OrderUnwitnessed {
        /// The view whose order is not asserted.
        view: ViewRef,
        /// Whose instances it holds.
        entity: EntityRef,
        /// How many rows the scenario could put there.
        arranged: usize,
        /// Why it could not arrange another.
        reason: Unreachable,
    },
    /// A construct this build does not synthesise yet.
    ///
    /// Visible rather than silent, deliberately. A reader of a suite cannot tell an unimplemented
    /// slice from a specification with nothing to check, and §36 rules that ambiguity out for
    /// refusals; the same reasoning applies to a gap this crate has not closed.
    NotSynthesisedYet {
        /// What is missing.
        construct: &'static str,
        /// Where it is specified.
        sections: &'static str,
    },
    /// A binding clause with nothing a scenario could observe.
    ///
    /// Not one reason but six, because the repairs differ: one of them is a policy that publishes
    /// nothing *on purpose* (§18), and the others are shapes a specification can be edited out of.
    BindingUnobservable {
        /// Which binding.
        binding: BindingRef,
        /// Why the clause has no witness.
        gap: BindingGap,
    },
    /// An entity invariant nothing observable reads.
    ///
    /// §20 evaluates invariants "against observable entity/view state where possible", and this is
    /// where that qualifier bites. An entity's invariant reads the entity's **fields**; a view
    /// publishes only what it **declares**. `weight_grams >= 0` on an entity whose views publish
    /// `order_id` and `contact` is therefore unobservable by construction — no runner and no witness
    /// generator can close it, and asserting it against something else would be asserting a
    /// different claim.
    InvariantUnobservable {
        /// Whose invariant.
        entity: EntityRef,
        /// The condition, as the author wrote it.
        invariant: String,
        /// The paths it reads that no view of the entity publishes.
        ///
        /// Empty when every path *is* published and the problem is the other one: no view holds an
        /// instance in the state the scenario reaches, so there would be no row to read. A path in
        /// [`unassertable`](Self::InvariantUnobservable::unassertable) is not here: a view does
        /// publish its field.
        unpublished: Vec<FactPath>,
        /// The `.count` reads — of a list or of a text — whose field some view does publish, and
        /// which no view-row assertion in this suite format can carry
        /// (`docs/design/string-alphabet-and-length.md`, section 3). Before this field they were
        /// reported as published by no view, which blamed the view for the suite format.
        unassertable: Vec<FactPath>,
        /// The state the entity is in when the assertion would run.
        state: StateName,
    },
    /// §19's rejection mechanism, which this command does not declare.
    ///
    /// A command attempted against an instance in a state none of its transitions run from **is**
    /// refused — that is exactly what makes the combination illegal — and this command declares no
    /// outcome and no error for it. So the scenario that exists asserts what it can, that nothing
    /// the specification declares was published, and cannot assert what §19 asks for: "the exact
    /// rejection mechanism must come from the declared command/error semantics", and "do not
    /// generate vague *operation fails* tests if the domain declares a specific error".
    ///
    /// **The model can express it now**, which is what changed: a `wrong_state:` outcome names the
    /// error the command reports, and the states it answers in stay implied by the transitions it
    /// does not run from. So this is no longer a gap in the model — it is a specification that has
    /// not said what its command does, and the repair is one branch in the document.
    ///
    /// It is a refusal beside a scenario rather than instead of one, exactly as
    /// [`InvariantUnobservable`](Self::InvariantUnobservable) is: the scenario is worth having, and
    /// a reader who cannot see that it asserts less than the section asks for will read a thin check
    /// as a thick one.
    RefusalUndeclared {
        /// Whose instance.
        entity: EntityRef,
        /// The state it is resting in when the command arrives.
        state: StateName,
        /// The command that is not honoured there.
        command: CommandRef,
    },
    /// A value object's declared invariants, with nowhere observable to read them.
    ///
    /// The value-object reading of §20's "against observable entity/view state where possible",
    /// and the cause that replaced this family's `NotSynthesisedYet` refusal when wave 6.5
    /// delivered the slice: what remains refused is what genuinely has no witness. Two shapes land
    /// here, told apart by [`at`](Self::ValueInvariantUnwitnessed::at) — no view publishes a field
    /// position that can answer the type at all, or a position exists and no declared outcome can
    /// put a row where the view shows one.
    ValueInvariantUnwitnessed {
        /// The type whose invariants they are.
        value: DeclaredTypeRef,
        /// The conditions, as the author wrote them.
        invariants: Vec<String>,
        /// The position that exists and cannot be arranged, where one does; `None` when no view
        /// publishes an answerable position at all — including a value held only inside a list, a
        /// map, a union or an `Optional`, none of which a fact path every row must answer can
        /// reach.
        at: Option<(ViewRef, String)>,
    },
    /// An aggregate view whose rows this scenario cannot keep apart from every other scenario's.
    ///
    /// An aggregate is an exact number, and §8 lets a target be shared, so a row another scenario
    /// made landing in the same group turns `3` into `4`. The scenario scopes its groups by a key
    /// value no other scenario produces, which needs a `String` or `Uuid` group key, or a parameter
    /// compared with one, that the creating command sets from its input
    /// (`docs/design/aggregate-views.md`, "Scoping"). Without one, an exact aggregate would be a
    /// claim about the target's other users.
    AggregateUnscoped {
        /// The aggregate view.
        view: ViewRef,
    },
    /// An aggregate view the arrangement cannot produce rows for as the page's pattern requires.
    ///
    /// A parameter read other than by one top-level equality conjunct, more than seven inputs, a
    /// field the search cannot set, or a filter truth it cannot reach. `reason` names which.
    AggregateUnwitnessed {
        /// The aggregate view.
        view: ViewRef,
        /// What could not be arranged, naming the field or the row.
        reason: String,
    },
    /// Two scenarios claimed one id. A drift alarm: `ess-domain` refuses a duplicated declaration.
    DuplicateScenario,
    /// The outcome's strategy and its condition disagree about how a scenario reaches the branch.
    ///
    /// A drift alarm. [`TestStrategy`] is computed from the condition, so the two cannot disagree
    /// unless one of them changes without the other. Two shapes reach it: a
    /// [`ConstructInput`](TestStrategy::ConstructInput) branch that declares no guard, and an
    /// [`ArrangeState`](TestStrategy::ArrangeState) branch asked for the input that reaches it —
    /// nothing reaches that one by choosing an input, and the illegal-move family sends the input
    /// the *moving* branch would have taken.
    ///
    /// One shape is not drift: an [`ObserveSubjectFact`](TestStrategy::ObserveSubjectFact) branch
    /// asked for an input by a family that cannot arrange the row it reads. The wrong-state family
    /// no longer asks (beyond10x/ess#173); where another still does, the text says so in the
    /// author's terms rather than as an alarm.
    StrategyWithoutGuard {
        /// What the strategy said.
        strategy: TestStrategy,
    },
    /// A guard compares a `.count` with a number whose boundary lies past what this synthesizer
    /// builds (`docs/design/string-alphabet-and-length.md`, section 4).
    ///
    /// In place of [`GuardUnsatisfiable`](Self::GuardUnsatisfiable) exactly when an outcome has no
    /// witness and one of its guards compares some `.count` with a literal whose `⌊v⌋ + 1` exceeds
    /// [`MAX_COUNT_WITNESS`](crate::witness::MAX_COUNT_WITNESS): the bound is stated rather than
    /// searched past, and the repair is an authored scenario, not a change to the input.
    CountUnwitnessed {
        /// The `.count` the guard reads.
        path: FactPath,
        /// The literal it is compared with, as written.
        literal: String,
        /// The most characters or elements a witness is built with.
        bound: usize,
    },
    /// A witness this synthesizer built is not a value of the input's declared type.
    ///
    /// A drift alarm, and the one that matters most: it means the witness walk and the flattener's
    /// walk have come to disagree about what a type accepts, which would otherwise surface as a
    /// guard that mysteriously cannot be decided.
    WitnessRejected(ShapeErrors),
    /// A reference a branch's `{related: …}` value follows may be absent, and no arrangement leaves
    /// it absent to witness the absent value (ess/22, beyond10x/ess#285). The branch's scenario
    /// stands; this names the coverage it lacks.
    AbsenceUnwitnessed {
        /// The reference, as `<entity>.<field>` or `input.<field>`.
        reference: String,
        /// Why it could not be left absent.
        reason: String,
    },
    /// A synthesized step requires a branch for an input the guards answer otherwise
    /// (beyond10x/ess#280): an input-guarded refusal, or an accepting `when:` branch declared
    /// before it, claims the input first, or the branch's own `when:` refutes it.
    ///
    /// A drift alarm, read off the finished suite by [`precedence_contradictions`]: under the
    /// precedence order (`docs/design/cross-record-and-stored-field-guards.md`) an input-guarded
    /// refusal answers before every accepting branch and every refusal declared after it, and the
    /// first declared accepting `when:` branch whose guard holds answers before the later ones, so
    /// such a step fails every target that honours the specification. The scenario is withdrawn
    /// rather than emitted.
    PrecedenceContradicted {
        /// The branch the step requires.
        required: OutcomeName,
        /// The branch that answers the input first, or `None` where the required branch's own
        /// guard refutes it.
        first: Option<OutcomeName>,
        /// The guard that decides it — `first`'s, or the required branch's own — as it reads.
        guard: String,
        /// The input the step sends, by field, as it reads.
        input: String,
    },
}

/// `RefusalCause::PrecedenceContradicted`'s number in the `SYNTH` family, the next after
/// [`COUNT_UNWITNESSED`].
pub const PRECEDENCE_CONTRADICTED: u16 = 19;

/// `RefusalCause::AbsenceUnwitnessed`'s number in the `SYNTH` family, the next after
/// [`PRECEDENCE_CONTRADICTED`] (ess/22, beyond10x/ess#285).
pub const ABSENCE_UNWITNESSED: u16 = 20;

/// The repair for a family asked to send a command guarded by a related row (ess/18, #211): only the
/// command's own outcome scenarios and its drivers arrange that row, and any other family has none
/// to point it at.
const RELATED_UNARRANGED: &str = "the command is selected by the row its `when_related:` guard \
     reads — the one its input names, or the one a stored field of the subject it addresses names \
     (ess/22) — which this scenario family does not arrange; cover it with an authored scenario \
     (ess-scenario/1)";

/// `RefusalCause::CountUnwitnessed`'s number in the `SYNTH` family, the next after
/// [`crate::aggregate::UNWITNESSED`].
pub const COUNT_UNWITNESSED: u16 = 18;

/// The refusal for an outcome no candidate reaches: `ESS-SYNTH-018` when one of its guards
/// compares a `.count` with a literal past [`MAX_COUNT_WITNESS`](crate::witness::MAX_COUNT_WITNESS),
/// which no candidate was built to decide, and otherwise `ESS-SYNTH-003`.
pub(crate) fn unsatisfied(guards: &[&Predicate], predicate: String, tried: usize) -> RefusalCause {
    fn past_the_cap(predicate: &Predicate) -> Option<(FactPath, String)> {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                children.iter().find_map(past_the_cap)
            }
            Predicate::Not(inner) => past_the_cap(inner),
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                past_the_cap(&quantified.body)
            }
            Predicate::Compare { left, right, .. } => [(left, right), (right, left)]
                .into_iter()
                .find_map(|(fact, literal)| match (fact, literal) {
                    (Operand::Fact(path), Operand::Literal(value))
                        if path.segments().last().is_some_and(|last| last == "count") =>
                    {
                        let number = value.as_number()?.get();
                        #[allow(clippy::cast_precision_loss)]
                        let cap = crate::witness::MAX_COUNT_WITNESS as f64;
                        (number.floor() + 1.0 > cap).then(|| (path.clone(), value.to_string()))
                    }
                    _ => None,
                }),
            _ => None,
        }
    }
    match guards.iter().find_map(|guard| past_the_cap(guard)) {
        Some((path, literal)) => RefusalCause::CountUnwitnessed {
            path,
            literal,
            bound: crate::witness::MAX_COUNT_WITNESS,
        },
        None => RefusalCause::GuardUnsatisfiable { predicate, tried },
    }
}

impl RefusalCause {
    /// The family every refusal here belongs to.
    pub const FAMILY: &'static str = "SYNTH";

    /// Its stable code.
    ///
    /// Derived from the variant rather than stored beside it, so a code cannot come to name a body
    /// other than its own: the number is the variant's entry in [`RefusalCause::CATALOGUE`].
    pub fn code(&self) -> Code {
        Code::new(Self::FAMILY, self.catalogue_entry().key)
    }

    /// What would have to change for the construct to be testable.
    #[allow(clippy::too_many_lines)]
    pub fn hint(&self) -> &'static str {
        match self {
            // A gap in what a view lets a scenario observe is repaired by declaring a view, not by
            // changing a value type (beyond10x/ess#132). Every such reason names the view it needs,
            // and says `immediate` where only a `read_your_writes` one will do, or names the
            // `eventual` alternative where one would do as well (beyond10x/ess#172).
            Self::NoWitness(gap) if gap.reason.contains("or an `eventual` one") => {
                "declare a view of the entity, `read_your_writes` or `eventual`, with no filter and \
                 no parameters, that projects its identity, its state and the fields named at the \
                 entity's types"
            }
            Self::NoWitness(gap)
                if gap.reason.contains(" view") && gap.reason.contains("immediate") =>
            {
                "declare a `read_your_writes` view of the entity that projects its identity, its \
                 state and the fields named; an `eventual` view cannot be read at the moment this \
                 observation is made"
            }
            Self::NoWitness(gap) if gap.reason.contains(" view") => {
                "declare a view of the entity that projects its identity and its state; a \
                 wrong-state refusal is observed through any identity view, `eventual` included"
            }
            Self::NoWitness(_) => {
                "give the field a type that has a finite value, or drop it from the command's input"
            }
            Self::GuardUnevaluable(_) => {
                "the guard reads something no input can supply; correct the path or the type it \
                 walks into"
            }
            Self::GuardUnsatisfiable { .. } => {
                "write the branch's condition over values a candidate can carry, or supply a \
                 fixture for it"
            }
            Self::InstanceRequired { reason, .. } => reason.hint(),
            Self::ViewUndecidable { .. } => {
                "filter the view on the entity's state, which is what a generated scenario knows \
                 after the command it ran"
            }
            Self::OrderUnwitnessed { .. } => {
                "declare an outcome that can leave a second instance where this view shows one, or \
                 drop `order_by:`; an order over one row is a claim no implementation can fail"
            }
            Self::AggregateUnscoped { .. } => {
                "group by, or filter by a parameter over, a `String` or `Uuid` field the creating \
                 command sets from its input"
            }
            Self::AggregateUnwitnessed { .. } => {
                "let the creating command set every field the view groups by or aggregates from \
                 its input, and read a parameter only as `field == param.name` at the top of the \
                 filter"
            }
            Self::NotSynthesisedYet { .. } => "a later slice of `ess-conformance` synthesises this",
            Self::BindingUnobservable { gap, .. } => gap.hint(),
            Self::InvariantUnobservable {
                unpublished,
                unassertable,
                ..
            } => {
                if !unassertable.is_empty() {
                    "a `.count` is not asserted on a view row in this suite format; the value is \
                     held to it where it is built, on command input and setup"
                } else if unpublished.is_empty() {
                    "declare a view that holds an instance in this state, or the invariant cannot \
                     be read after this branch"
                } else {
                    "publish the fields the invariant reads in a view of this entity, or state the \
                     invariant over what one already publishes"
                }
            }
            Self::RefusalUndeclared { .. } => {
                "give the command a `wrong_state:` outcome naming the error it reports; the states \
                 it answers in are already declared, as the states its transitions do not run from"
            }
            Self::ValueInvariantUnwitnessed { at: None, .. } => {
                "publish a field that holds a value of this type in some view — outside a list, a \
                 map, a union and an `Optional` — or state the claim as an entity invariant over \
                 what a view already publishes"
            }
            Self::ValueInvariantUnwitnessed { at: Some(_), .. } => {
                "declare an outcome that leaves an instance in a state this view's filter holds, \
                 or widen the filter"
            }
            Self::DuplicateScenario => {
                "two declarations produced one scenario id; rename one of them"
            }
            // A `when_subject:` branch is reached by arranging the row it reads, and a family that
            // can only choose an input for it has nothing to choose with: not drift, a family this
            // synthesizer cannot yet build for that branch (beyond10x/ess#173).
            Self::StrategyWithoutGuard {
                strategy: TestStrategy::ObserveSubjectFact,
            } => {
                "the branch is selected by the subject's stored fields, which this scenario family \
                 cannot arrange; cover it with an authored scenario (ess-scenario/1)"
            }
            Self::StrategyWithoutGuard {
                strategy: TestStrategy::ArrangeRelatedRow,
            } => RELATED_UNARRANGED,
            Self::StrategyWithoutGuard { .. } => {
                "`TestStrategy` and `OutcomeCondition` have drifted apart in `ess-domain`"
            }
            Self::CountUnwitnessed { .. } => {
                "the guard compares `.count` with a value above the 1024 this synthesizer builds; \
                 cover the branch with an authored scenario (ess-scenario/1), or lower the bound"
            }
            Self::WitnessRejected(_) => {
                "the witness walk and the flattener disagree about this type; they read one table"
            }
            Self::PrecedenceContradicted { .. } => {
                "nothing to change in the specification; this is a defect in ess to report, with \
                 the specification that produced it"
            }
            Self::AbsenceUnwitnessed { .. } => ABSENCE_REPAIR,
        }
    }
}

/// The repair for an absent reference no arrangement leaves absent (ess/22, beyond10x/ess#285).
const ABSENCE_REPAIR: &str = "fill the reference from an Optional input that the branch, or the \
     branch creating the row that holds it, can be sent without; or cover the absent value with an \
     authored scenario (ess-scenario/1)";

// The code, the meaning and the repair of every synthesis refusal, from one list. A `help:` line
// may be more specific than the repair here, because some causes carry the reason that decides
// it; the repair here is what holds for every refusal under the code. Code 10 names every binding
// gap but one, so a new gap is a compile error until somebody decides which code it carries.
crate::authored::diagnostic_catalogue! {
    impl RefusalCause => u16 {
        Self::NoWitness(_) => 1,
            "No value of the command's declared input type could be constructed at all.",
            "give the field a type that has a finite value, or drop it from the command's input; \
             where the reason names a view, declare the view of the entity it asks for, \
             projecting its identity, its state and the fields named";
        Self::GuardUnevaluable(_) => 2,
            "The guard could not be decided against a candidate, and no other candidate would \
             change it.",
            "the guard reads something no input can supply; correct the path or the type it \
             walks into";
        Self::GuardUnsatisfiable { .. } => 3,
            "Every candidate this synthesizer knows how to try was refuted.",
            "write the branch's condition over values a candidate can carry, or supply a \
             fixture for it";
        Self::InstanceRequired { .. } => 4,
            "The scenario needed an instance of an entity, and the specification cannot arrange \
             one.",
            "give some outcome `creates:` for the entity, or declare a transition and an outcome \
             that reach the state; where the route runs through a branch no input reaches, \
             repair that branch's own refusal";
        Self::ViewUndecidable { .. } => 5,
            "A view's filter could not be decided against the state the scenario reaches.",
            "filter the view on the entity's state, which is what a generated scenario knows \
             after the command it ran";
        Self::NotSynthesisedYet { .. } => 6,
            "A construct this build does not synthesise yet.",
            "nothing to change in the specification; cover the construct with an authored \
             scenario until a later release synthesises it";
        Self::DuplicateScenario => 7,
            "Two declarations produced one scenario id.",
            "two declarations produced one scenario id; rename one of them";
        Self::StrategyWithoutGuard { .. } => 8,
            "The branch is selected in a way this scenario family cannot arrange: by the \
             subject's stored fields, or by a row of another entity (`when_related:`).",
            "cover the branch with an authored scenario (ess-scenario/1); where the `help:` line \
             says two parts of ess have drifted apart, that is a defect in ess to report";
        Self::WitnessRejected(_) => 9,
            "A value synthesis built is not a value of the input's declared type.",
            "nothing to change in the specification; this is a defect in ess to report, with the \
             specification that produced it";
        Self::BindingUnobservable {
            gap:
                BindingGap::NothingPublishes { .. }
                | BindingGap::BranchUndecided { .. }
                | BindingGap::NothingPublished { .. }
                | BindingGap::NothingMapped { .. }
                | BindingGap::NoForcibleFailure { .. }
                | BindingGap::PolicySilent
                | BindingGap::DeliverySingleAttempt
                | BindingGap::RetriedUnforcible { .. }
                | BindingGap::FinalUnforcible { .. }
                | BindingGap::ArrangementSetsOff { .. }
                | BindingGap::DestinationIdentityUnavailable { .. }
                | BindingGap::DestinationIneligible { .. }
                | BindingGap::DestinationUnreachable { .. }
                | BindingGap::UnchangedUnobservable { .. }
                | BindingGap::EffectUnsettled { .. }
                | BindingGap::ConditionUnarranged { .. },
            ..
        } => 10,
            "A binding clause has nothing a scenario could observe.",
            "follow the `help:` line, which names the gap: an event nothing emits, a flow with \
             no consequence, an invocation that maps nothing, a failure no scenario can force, \
             or a `drop` or `at_most_once` policy that is unobservable by design";
        Self::InvariantUnobservable { .. } => 11,
            "An entity invariant nothing observable reads.",
            "publish the fields the invariant reads in a view of this entity, or state the \
             invariant over what one already publishes";
        Self::RefusalUndeclared { .. } => 12,
            "A command attempted in a state none of its transitions run from declares no outcome \
             for that refusal.",
            "give the command a `wrong_state:` outcome naming the error it reports; the states \
             it answers in are already declared, as the states its transitions do not run from";
        Self::ValueInvariantUnwitnessed { .. } => 13,
            "A value object's declared invariants, with nowhere observable to read them.",
            "publish a field that holds a value of this type in some view, outside a list, a \
             map, a union and an `Optional`; or declare an outcome that leaves an instance in a \
             state the view's filter holds";
        Self::OrderUnwitnessed { .. } => 14,
            "A view declares an order and the scenario cannot put two rows in it.",
            "declare an outcome that can leave a second instance where this view shows one, or \
             drop `order_by:`; an order over one row is a claim no implementation can fail";
        Self::BindingUnobservable {
            gap: BindingGap::AccessorObservation { .. },
            ..
        } => 15,
            "A binding's mapping uses a native accessor whose expected result cannot be \
             reconstructed from what a scenario observes.",
            "provide a separately executable host conversion or an unambiguous observable \
             assignment";
        Self::AggregateUnscoped { .. } => crate::aggregate::UNSCOPED,
            "An aggregate view whose rows this scenario cannot keep apart from every other \
             scenario's.",
            "group by, or filter by a parameter over, a `String` or `Uuid` field the creating \
             command sets from its input";
        Self::AggregateUnwitnessed { .. } => crate::aggregate::UNWITNESSED,
            "An aggregate view the arrangement cannot produce rows for as the page's pattern \
             requires.",
            "let the creating command set every field the view groups by or aggregates from \
             its input, and read a parameter only as `field == param.name` at the top of the \
             filter";
        Self::CountUnwitnessed { .. } => COUNT_UNWITNESSED,
            "A guard compares a `.count` with a number whose boundary lies past what this \
             synthesizer builds.",
            "the guard compares `.count` with a value above the 1024 this synthesizer builds; \
             cover the branch with an authored scenario (ess-scenario/1), or lower the bound";
        Self::PrecedenceContradicted { .. } => PRECEDENCE_CONTRADICTED,
            "A synthesized step requires a branch for an input its own guard refutes, or another \
             branch answers first under the precedence order.",
            "nothing to change in the specification; this is a defect in ess to report, with the \
             specification that produced it";
        Self::AbsenceUnwitnessed { .. } => ABSENCE_UNWITNESSED,
            "A reference a `{related: …}` value follows may be absent, and no arrangement leaves \
             it absent to witness the absent value; the branch's scenario stands without it.",
            "fill the reference from an Optional input that the branch, or the branch creating \
             the row that holds it, can be sent without; or cover the absent value with an \
             authored scenario (ess-scenario/1)";
    }
}

impl fmt::Display for RefusalCause {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoWitness(gap) => write!(f, "no witness: {gap}"),
            Self::GuardUnevaluable(refusal) => write!(f, "{refusal}"),
            Self::GuardUnsatisfiable { predicate, tried } => write!(
                f,
                "no candidate of the {tried} tried satisfies `{predicate}`"
            ),
            Self::InstanceRequired {
                entity,
                need,
                reason,
            } => write!(f, "it needs an instance of `{entity}` {need}, and {reason}"),
            Self::ViewUndecidable {
                view,
                filter,
                state,
                unbound,
            } => {
                write!(
                    f,
                    "`{view}` filters on `{filter}`, which is undecided for an entity in `{state}`"
                )?;
                for path in unbound {
                    write!(f, "\n  - `{path}` is bound by nothing a scenario knows")?;
                }
                Ok(())
            }
            Self::OrderUnwitnessed {
                view,
                entity,
                arranged,
                reason,
            } => write!(
                f,
                "`{view}` declares an order and this scenario can put {arranged} row(s) in it: \
                 comparing two rows needs two `{entity}`s, and {reason}"
            ),
            Self::NotSynthesisedYet {
                construct,
                sections,
            } => write!(
                f,
                "{construct} is specified in {sections} and not synthesised yet"
            ),
            Self::RefusalUndeclared {
                entity,
                state,
                command,
            } => write!(
                f,
                "`{command}` on a `{entity}` in `{state}` is refused and the specification does \
                 not say how: no `wrong_state:` outcome and no declared error, so the scenario can \
                 only require that nothing happened"
            ),
            Self::AggregateUnscoped { .. } | Self::AggregateUnwitnessed { .. } => {
                aggregate_refusal(f, self)
            }
            Self::CountUnwitnessed {
                path,
                literal,
                bound,
            } => write!(
                f,
                "`{path}` is compared with {literal}, and deciding that needs a text or a list \
                 longer than the {bound} characters or elements this synthesizer builds"
            ),
            Self::DuplicateScenario => f.write_str("a second scenario claimed this id"),
            Self::StrategyWithoutGuard {
                strategy: strategy @ TestStrategy::ObserveSubjectFact,
            } => write!(
                f,
                "its strategy is `{strategy}`: the subject's stored fields select it, and this \
                 scenario asked for an input that reaches it"
            ),
            Self::StrategyWithoutGuard {
                strategy: strategy @ TestStrategy::ArrangeRelatedRow,
            } => write!(
                f,
                "its command's strategy is `{strategy}`: the row its `when_related:` guard reads \
                 — the one its input names, or the one a stored field of the subject it addresses \
                 names (ess/22) — selects its branch, and this scenario family arranges none"
            ),
            Self::StrategyWithoutGuard { strategy } => {
                write!(f, "its strategy is `{strategy}` and it declares no guard")
            }
            Self::WitnessRejected(errors) => {
                write!(
                    f,
                    "the witness is not a value of the input's type:\n{errors}"
                )
            }
            Self::BindingUnobservable { binding, gap } => write!(f, "`{binding}` {gap}"),
            Self::PrecedenceContradicted {
                required,
                first: Some(first),
                guard,
                input,
            } => write!(
                f,
                "a step requires `{required}` for {input}, which `{first}` ({guard}) answers \
                 first under the precedence order"
            ),
            Self::PrecedenceContradicted {
                required,
                first: None,
                guard,
                input,
            } => write!(
                f,
                "a step requires `{required}` for {input}, which its own guard ({guard}) refutes"
            ),
            Self::AbsenceUnwitnessed { reference, reason } => {
                write!(f, "no arrangement leaves `{reference}` absent: {reason}")
            }
            Self::InvariantUnobservable { .. } => invariant_unobservable(f, self),
            Self::ValueInvariantUnwitnessed {
                value,
                invariants,
                at,
            } => value_unwitnessed(f, value, invariants, at.as_ref()),
        }
    }
}

/// Renders [`RefusalCause::InvariantUnobservable`]: no row in the state, or the paths no view row
/// can be held to — a `.count` first, then what no view publishes.
fn invariant_unobservable(f: &mut fmt::Formatter<'_>, cause: &RefusalCause) -> fmt::Result {
    let RefusalCause::InvariantUnobservable {
        entity,
        invariant,
        unpublished,
        unassertable,
        state,
    } = cause
    else {
        unreachable!("called for InvariantUnobservable only")
    };
    if unpublished.is_empty() && unassertable.is_empty() {
        return write!(
            f,
            "`{invariant}` cannot be read after this branch: no view of `{entity}` holds an \
             instance in `{state}`"
        );
    }
    if unassertable.is_empty() {
        write!(
            f,
            "`{invariant}` reads what no view of `{entity}` publishes"
        )?;
    } else {
        write!(
            f,
            "`{invariant}` reads what no view row of `{entity}` can be held to"
        )?;
    }
    for path in unassertable {
        write!(
            f,
            "\n  - `{path}` is a `.count`, which a view row does not assert in this suite format"
        )?;
    }
    for path in unpublished {
        write!(f, "\n  - `{path}` is published by no view of the entity")?;
    }
    Ok(())
}

/// Renders the two aggregate view refusals.
fn aggregate_refusal(f: &mut fmt::Formatter<'_>, cause: &RefusalCause) -> fmt::Result {
    match cause {
        RefusalCause::AggregateUnscoped { view } => write!(
            f,
            "`{view}` reports exact aggregates, and no group key or parameter lets this scenario \
             keep its rows apart from every other scenario's on a shared target"
        ),
        RefusalCause::AggregateUnwitnessed { view, reason } => {
            write!(f, "`{view}` cannot be arranged: {reason}")
        }
        _ => Ok(()),
    }
}

/// Renders [`RefusalCause::ValueInvariantUnwitnessed`], quoting the author's own conditions.
fn value_unwitnessed(
    f: &mut fmt::Formatter<'_>,
    value: &DeclaredTypeRef,
    invariants: &[String],
    at: Option<&(ViewRef, String)>,
) -> fmt::Result {
    match at {
        None => {
            write!(
                f,
                "no view publishes a field position that can answer what `{value}` declares of \
                 every value"
            )?;
            // A `.count` is never projectable onto a row, so an invariant reading one has no
            // position whatever the views publish; saying so names the real limit.
            let counts: BTreeSet<String> = invariants
                .iter()
                .filter_map(|statement| Predicate::parse_expression(statement).ok())
                .flat_map(|predicate| {
                    predicate
                        .fact_paths()
                        .into_iter()
                        .filter(|path| path.segments().last().is_some_and(|last| last == "count"))
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                })
                .collect();
            if !counts.is_empty() {
                write!(
                    f,
                    "; `{}` is a `.count`, which a view row does not assert in this suite format",
                    counts.into_iter().collect::<Vec<_>>().join("`, `")
                )?;
            }
        }
        Some((view, field)) => write!(
            f,
            "`{view}.{field}` holds a `{value}` and no declared outcome leaves a row there for \
             its invariants to be read off"
        )?,
    }
    for invariant in invariants {
        write!(f, "\n  - `{invariant}`")?;
    }
    Ok(())
}

/// Why one clause of a binding has no scenario.
///
/// Ten shapes, and the split that matters is between the eight a specification author can edit
/// and [`PolicySilent`](Self::PolicySilent) and
/// [`DeliverySingleAttempt`](Self::DeliverySingleAttempt), which are decisions the author already
/// made and for which the model deliberately gives nothing to observe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingGap {
    /// A native accessor whose expectation is not reconstructible from observations.
    AccessorObservation {
        /// Exact unsupported observation or conversion contract.
        reason: String,
    },
    /// No command outcome emits the event the binding reacts to.
    ///
    /// Legal: an event may arrive from outside the specification, which is why `ess-domain` does not
    /// refuse it. It still means nothing in a scenario can make the binding fire.
    NothingPublishes {
        /// The event nothing emits.
        event: EventRef,
    },
    /// Which branch the invoked command takes depends on the values the event carries.
    ///
    /// A binding fills the command's input from the event, and no generator knows what the upstream
    /// implementation will publish there — so with two branches an input decides between, a scenario
    /// cannot say which one to require. Deciding it by inspecting the guard would be a claim about a
    /// value nobody has.
    BranchUndecided {
        /// The invoked command.
        command: CommandRef,
        /// The branches a scenario cannot choose between, in declaration order.
        branches: Vec<OutcomeName>,
    },
    /// The branch the binding reaches publishes nothing.
    ///
    /// §16 proves a flow through the event the invoked command publishes. A branch that emits none
    /// leaves the flow with no observable consequence at all.
    NothingPublished {
        /// The branch that publishes nothing.
        outcome: OutcomeRef,
    },
    /// The binding fills no input, so there is no mapping to check.
    NothingMapped {
        /// The invoked command.
        command: CommandRef,
    },
    /// No branch of the invoked command can be made to fail.
    ///
    /// A failure policy is only observable once the failure has been forced, and the only branch a
    /// scenario can force is one the specification declares `external:` (§12). Without one, the
    /// declared policy is a word nothing exercises.
    NoForcibleFailure {
        /// The invoked command.
        command: CommandRef,
    },
    /// The declared policy is `drop`, which publishes nothing on purpose.
    ///
    /// §18's rule, and the one refusal in this family that is not a defect: "give up silently" is
    /// the whole content of the word, and `ess-domain` records why an event here would be wrong —
    /// it would make the policy a notification, which is a different decision that already has a
    /// name. So the check is refused rather than invented, and what a reader learns is that this
    /// binding's failure path is *by declaration* unprovable.
    PolicySilent,
    /// The declared guarantee is `at_most_once`, which has no second delivery to perform.
    ///
    /// §17's scenario is the redelivery: the same event arrives twice and the consequence is still
    /// observable. `at_most_once` says the opposite — one attempt, nothing delivers it again — so
    /// there is no second arrival to arrange, and arranging one anyway would ask a conformant
    /// implementation to do the thing its specification says it does not do. The word's remaining
    /// content, that a lost attempt is not retried here, is what `on_failure` declares and what the
    /// `on-failure` scenario beside this one already checks.
    ///
    /// The consequence for a target: an `at_most_once` binding puts no
    /// [`RedeliverEvent`](crate::scenario::ScenarioStep::RedeliverEvent) in the suite, so
    /// [`redeliver_event`](crate::target::ConformanceTarget::redeliver_event) is never reached for
    /// it. A system all of whose bindings deliver at most once owes that method nothing.
    DeliverySingleAttempt,
    /// The binding states a bounded retry, and every refusal of the invoked command a scenario can
    /// force is `final`, so none can be forced on every attempt to exhaust the bound.
    RetriedUnforcible {
        /// The invoked command.
        command: CommandRef,
    },
    /// The binding names `final` refusals, and none of them can be forced.
    FinalUnforcible {
        /// The invoked command.
        command: CommandRef,
    },
    /// Every command that publishes the binding's event needs an arrangement that publishes it
    /// too, so the binding would be invoked before the counted attempts begin.
    ArrangementSetsOff {
        /// The event the binding reacts to.
        event: EventRef,
    },
    /// The invoked command acts on an existing row, and no identity for it is knowable before the
    /// trigger runs (beyond10x/ess#267): a field the implementation mints, a value only the
    /// response carries, or a mapping no arranged row can stand behind. Synthesis does not guess
    /// the next identity, and does not arrange a row after the binding has started.
    DestinationIdentityUnavailable {
        /// The invoked command.
        command: CommandRef,
        /// Its input naming the row.
        input: String,
        /// Why that identity is not knowable beforehand.
        why: &'static str,
    },
    /// The trigger leaves the row it acts on in a state no accepting branch of the invoked command
    /// admits, so the binding can only be refused there.
    DestinationIneligible {
        /// The invoked command.
        command: CommandRef,
        /// The state the row is in when the binding is delivered.
        state: StateName,
    },
    /// The row the invoked command acts on cannot be arranged in a state its accepting branch
    /// admits.
    DestinationUnreachable {
        /// The row's entity.
        entity: EntityRef,
        /// The first state tried.
        state: StateName,
        /// Why.
        reason: Box<Unreachable>,
    },
    /// The policy is `drop`, and the row the binding addresses cannot be read before the trigger
    /// and compared after it, which is what tells a dropped failure from a success (beyond10x/ess#267).
    UnchangedUnobservable {
        /// Why not.
        why: &'static str,
    },
    /// The branch sets off a binding on its own row, and where the row comes to rest cannot be
    /// named: two bindings at once, a branch an input decides, a deletion or a cycle
    /// (beyond10x/ess#266). The scenario keeps what the branch itself does and asserts no view of
    /// the row.
    EffectUnsettled {
        /// The branch that sets the binding off.
        outcome: OutcomeRef,
        /// Why the chain does not settle.
        why: String,
    },
    /// The binding's event-payload condition (ess/22, beyond10x/ess#268) cannot be made to hold,
    /// or to fail, by a trigger this synthesis can vary without changing what else it does.
    ConditionUnarranged {
        /// Which side, and why.
        why: String,
    },
}

impl BindingGap {
    /// What would have to change.
    fn hint(&self) -> &'static str {
        match self {
            Self::AccessorObservation { .. } => {
                "provide a separately executable host conversion or an unambiguous observable assignment"
            }
            Self::NothingPublishes { .. } => {
                "give some command outcome `emits:` for this event; a binding on an event nothing \
                 publishes can never fire"
            }
            Self::BranchUndecided { .. } => {
                "leave the invoked command one branch an input does not choose, or declare the \
                 others `external:` so a scenario can force them"
            }
            Self::NothingPublished { .. } => {
                "emit an event from the branch the binding reaches; a flow with no consequence is a \
                 flow nothing can observe"
            }
            Self::NothingMapped { .. } => {
                "map the command's inputs from the event, or drop the binding: an invocation that \
                 carries nothing carries nothing to check"
            }
            Self::NoForcibleFailure { .. } => {
                "declare the branch that fails `external:`, which is what lets a scenario force it"
            }
            Self::PolicySilent => {
                "`drop` is unobservable by design; write `escalate:` with an event if the failure \
                 has to be provable"
            }
            Self::DeliverySingleAttempt => {
                "`at_most_once` has no redelivery by design; write `at_least_once` if the transport \
                 really may deliver the event again"
            }
            Self::RetriedUnforcible { .. } => {
                "declare the refusal the retry repeats `external:` and leave it out of `final:`, \
                 which is what lets a scenario force it on every attempt"
            }
            Self::FinalUnforcible { .. } => {
                "declare the final refusal `external:`, which is what lets a scenario force it"
            }
            Self::ArrangementSetsOff { .. } => {
                "let some command publish the event without an arrangement that publishes it first; \
                 an attempt count is only a count of the attempts the bound made"
            }
            Self::DestinationIdentityUnavailable { .. } => {
                "map the row's identity from an event field the trigger fills from its own input or \
                 from the row it acts on, so a scenario can arrange that row before the trigger"
            }
            Self::DestinationIneligible { .. } => {
                "let the trigger leave the row in a state the invoked command accepts, or cover the \
                 binding with an authored scenario (ess-scenario/1)"
            }
            Self::DestinationUnreachable { .. } => {
                "give the row's entity a route to a state the invoked command accepts; see the \
                 reason"
            }
            Self::UnchangedUnobservable { .. } => {
                "address a row that exists before the trigger and that the trigger leaves alone, \
                 and declare a read_your_writes view showing it by identity; or write `escalate:`"
            }
            Self::EffectUnsettled { .. } => {
                "let at most one binding act on a row at once, through a branch its state decides, \
                 and end every chain; or cover the row with an authored scenario (ess-scenario/1)"
            }
            Self::ConditionUnarranged { .. } => {
                "let the trigger copy every event member the condition reads from an input it \
                 does not guard on, or cover the condition with an authored scenario \
                 (ess-scenario/1)"
            }
        }
    }
}

impl fmt::Display for BindingGap {
    /// Reads as the tail of "`<binding>` …".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessorObservation { reason } => {
                write!(f, "has no accessor observation: {reason}")
            }
            Self::NothingPublishes { event } => {
                write!(f, "reacts to `{event}`, which no outcome emits")
            }
            Self::BranchUndecided { command, branches } => write!(
                f,
                "invokes `{command}`, whose branch is decided by an input the event fills: {}",
                branches
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::NothingPublished { outcome } => {
                write!(f, "invokes `{outcome}`, which publishes nothing")
            }
            Self::NothingMapped { command } => {
                write!(f, "fills none of `{command}`'s input")
            }
            Self::NoForcibleFailure { command } => write!(
                f,
                "declares a failure policy, and no branch of `{command}` can be forced to fail"
            ),
            Self::PolicySilent => {
                f.write_str("gives up silently, which the model publishes nothing for")
            }
            Self::DeliverySingleAttempt => {
                f.write_str("delivers at most once, which has no second delivery to observe")
            }
            Self::RetriedUnforcible { command } => write!(
                f,
                "bounds its retry, and no refusal of `{command}` it retries can be forced"
            ),
            Self::FinalUnforcible { command } => write!(
                f,
                "names final refusals of `{command}`, and none of them can be forced"
            ),
            Self::ArrangementSetsOff { event } => write!(
                f,
                "counts attempts, and every way to publish `{event}` publishes it while arranging"
            ),
            Self::DestinationIdentityUnavailable {
                command,
                input,
                why,
            } => write!(
                f,
                "invokes `{command}` on an existing row named by `{input}`, and no identity for it \
                 is knowable before the trigger: {why}"
            ),
            Self::DestinationIneligible { command, state } => write!(
                f,
                "invokes `{command}` on a row the trigger leaves in `{state}`, which no accepting \
                 branch admits"
            ),
            Self::DestinationUnreachable {
                entity,
                state,
                reason,
            } => write!(
                f,
                "needs a `{entity}` resting where its invoked command accepts it, first `{state}`, \
                 and {reason}"
            ),
            Self::UnchangedUnobservable { why } => write!(
                f,
                "drops a failed attempt, and the row it addresses cannot be shown unchanged: {why}"
            ),
            Self::EffectUnsettled { outcome, why } => write!(
                f,
                "acts on the row `{outcome}` leaves, and where that row comes to rest cannot be \
                 named: {why}"
            ),
            Self::ConditionUnarranged { why } => {
                write!(
                    f,
                    "invokes only when its event-payload condition holds, and {why}"
                )
            }
        }
    }
}

/// Why a scenario could not get the instance it needed.
///
/// Five shapes, and the difference is which line of the specification an author goes and edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unreachable {
    /// No outcome brings an instance of the entity into existence.
    ///
    /// Legal: an entity may arrive from a migration or from a system outside this document, which is
    /// why `ess-domain` does not refuse it. It still means no scenario can act on one.
    NothingCreates,
    /// No sequence of declared, driven transitions reaches the state from where the lifecycle starts.
    ///
    /// A drift alarm. `ess-domain` refuses a state nothing reaches (`unreachable_state`) and a
    /// transition nothing drives (`missing_causation`), so the graph this walk searches and the
    /// graph that check searches are the same one — and a valid specification cannot produce this.
    /// It exists so that if the two ever come apart, the result is a named refusal rather than a
    /// scenario nobody wrote. The reachable shape of "cannot get there" is
    /// [`Unwitnessable`](Self::Unwitnessable).
    NoPath {
        /// Where a new instance begins.
        from: StateName,
    },
    /// A command on the route to the state has no input that reaches the branch that moves it.
    ///
    /// The full reason is on that outcome's own refusal; naming it here keeps one cause in one
    /// place rather than restating it under a second id.
    Unwitnessable {
        /// The branch that could not be reached.
        outcome: OutcomeRef,
    },
    /// The rows are read by their owner, through the link `via`, and the owning relation is
    /// `cardinality: one`: an owner holds one row, so no second row is ever read beside it
    /// (beyond10x/ess#193).
    OwnerHoldsOne {
        /// The field that links a row to its owner.
        via: String,
    },
    /// A move on the route reads a related row through a stored field of the row being arranged
    /// (`when_related: {via: <field>}`, ess/22, beyond10x/ess#304), and no arranging run can set
    /// that field so the move's branch is selected: left out, or naming a row of the related entity
    /// arranged one level deep.
    StoredReference {
        /// The branch the route runs through.
        outcome: Box<OutcomeRef>,
        /// Why neither run can be built.
        why: Box<str>,
    },
    /// Every step that leaves a row in the state sets off a binding that moves the row on
    /// (beyond10x/ess#266): acting on it there would race the binding. Not an executed scenario
    /// and not a skipped green one — the binding's own flow scenario is the witness.
    BoundAway {
        /// The state no row rests in.
        state: StateName,
        /// The binding that moves it on.
        binding: BindingRef,
    },
    /// A step on the route sets off a binding on the row whose rest cannot be named or observed
    /// (beyond10x/ess#266).
    BindingUnsettled {
        /// The binding.
        binding: BindingRef,
        /// The step that sets it off.
        outcome: Box<OutcomeRef>,
        /// Why.
        why: Box<str>,
    },
}

impl Unreachable {
    /// What would have to change.
    fn hint(&self) -> &'static str {
        match self {
            Self::NothingCreates => {
                "give some command outcome `creates:` for this entity; nothing can act on an \
                 instance nothing brings into existence"
            }
            Self::NoPath { .. } => {
                "declare a transition that reaches this state, and an outcome that takes it"
            }
            Self::Unwitnessable { .. } => {
                "the route to this state runs through a branch no input reaches; see that \
                 outcome's own refusal"
            }
            Self::OwnerHoldsOne { .. } => {
                "an owner under `cardinality: one` holds one row, so an order over one owner's \
                 rows has nothing to rank; declare `cardinality: many` if an owner holds several"
            }
            Self::StoredReference { .. } => {
                "an arranging run sends a branch reading a stored reference with the reference \
                 left out, or naming a row of the related entity arranged one level deep; declare \
                 the reference `Optional<…>`, give the related entity a route to a row that \
                 selects the branch, or cover the state with an authored scenario (ess-scenario/1)"
            }
            Self::BoundAway { .. } => {
                "nothing to change if the binding is meant to move every such row on: its flow \
                 scenario witnesses the move; otherwise give the entity a route to the state that \
                 sets no binding off"
            }
            Self::BindingUnsettled { .. } => {
                "let at most one binding act on a row at once, through a branch its state decides, \
                 end every chain, and declare an unfiltered view showing the row's identity and \
                 state"
            }
        }
    }
}

impl fmt::Display for Unreachable {
    /// Reads as the tail of "it needs an instance of `X` …, and …".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingCreates => f.write_str("no outcome creates one"),
            Self::NoPath { from } => {
                write!(f, "no declared move reaches it from `{from}`")
            }
            Self::Unwitnessable { outcome } => {
                write!(
                    f,
                    "the route runs through `{outcome}`, which no input reaches"
                )
            }
            Self::OwnerHoldsOne { via } => write!(
                f,
                "the rows are read by the owner `{via}` names, and the owning relation is \
                 `cardinality: one`, so an owner holds one row"
            ),
            Self::StoredReference { outcome, why } => write!(
                f,
                "the route runs through `{outcome}`, which reads a related row through a stored \
                 field of the row being arranged, and no arranging run sets it to select that \
                 branch: {why}"
            ),
            Self::BoundAway { state, binding } => write!(
                f,
                "every step that leaves it in `{state}` sets off `{binding}`, which moves it on, so \
                 acting on it there would race the binding; `{binding}/binding/flow` is the witness"
            ),
            Self::BindingUnsettled {
                binding,
                outcome,
                why,
            } => write!(
                f,
                "the route runs through `{outcome}`, which sets off `{binding}` on the row, and \
                 where the row comes to rest cannot be named: {why}"
            ),
        }
    }
}

/// What a scenario would have needed of an instance that already exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceNeed {
    /// The outcome moves one along a declared transition.
    Moves {
        /// Which move.
        transition: String,
    },
    /// The outcome changes one without moving it.
    Updates,
    /// The scenario needs one resting in a particular state before it acts.
    InState {
        /// Which state.
        state: StateName,
    },
}

impl fmt::Display for InstanceNeed {
    /// Reads as the tail of "it needs an instance of `X` …".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Moves { transition } => write!(f, "to move along `{transition}`"),
            Self::Updates => f.write_str("to change without moving"),
            Self::InState { state } => write!(f, "resting in `{state}`"),
        }
    }
}

/// Every check one specification obliges an implementation to pass, and every one it cannot.
///
/// Deterministic (§37): the walk is over [`BTreeMap`]s in name order and declaration order, every
/// value it chooses is a function of the model, and nothing here reads a clock or a random device.
/// `tests/synthesis.rs` synthesises the billing example twice and compares bytes.
pub fn synthesize(ir: &EssIr) -> Synthesis {
    // Each witness search is run once per question for the whole synthesis (beyond10x/ess#301).
    let _memoised = crate::witness_memo::memoise(ir);
    // ess/16 (#168): a model whose actors carry attributes is synthesized once per caller
    // assignment, each read with the caller's values written in (`caller::synthesize`).
    let mut synthesis = if caller::uses(ir) {
        caller::synthesize(ir)
    } else {
        synthesize_plain(ir, Focus::Whole)
    };
    grant::cross_caller(ir, &mut synthesis.suite, &mut synthesis.notes);
    // Read off the finished suite: no scenario expects the one row of a singleton entity created
    // twice in a run, whichever family built it (beyond10x/ess#287).
    singleton::withdraw_second_creations(ir, &mut synthesis);
    // A scenario requiring a branch for an input an input-guarded refusal answers first fails every
    // target that honours the specification, so it is withdrawn and refused (beyond10x/ess#280).
    for refusal in precedence_contradictions(ir, &synthesis.suite) {
        if let Some(id) = &refusal.scenario {
            synthesis.suite.scenarios.remove(id);
        }
        synthesis.refusals.push(refusal);
    }
    // A note about an unseparated pair is recorded with its branch's scenario, and later passes
    // (fixtures, clock offsets, caller readings) may drop that scenario; a note naming a scenario
    // the suite does not hold points at nothing, so it goes with it (beyond10x/ess#202).
    let suite = &synthesis.suite;
    synthesis.notes.retain(|note| match note {
        Note::UnseparatedSources { scenario, .. }
        | Note::UnaccompaniedRelatedCopy { scenario, .. } => suite.scenario(scenario).is_some(),
        _ => true,
    });
    // Read off the finished suite, so every path that builds a branch's scenario is held to it.
    let overlaps = unwitnessed_overlaps(ir, &synthesis.suite);
    synthesis.notes.extend(overlaps);
    disclosure::augment(ir, &mut synthesis);
    synthesis
}

/// Which scenarios a synthesis writes.
///
/// A caller assignment that sends one command as the second caller is read only for that command's
/// own scenarios (`caller::synthesize`), and writing every other scenario of the model to throw it
/// away cost a whole synthesis per caller-reading command (beyond10x/ess#301).
#[derive(Clone, Copy)]
enum Focus<'a> {
    /// Every scenario the model obliges.
    Whole,
    /// Only the scenarios about a branch of this command: its `outcome`, `transition`, `refusal`
    /// and `invariant` ids, which every family files from its loop over that one command. Families
    /// that file no such id (bindings, aggregates, value-object invariants) are not run, and the
    /// passes over the finished suite run as they always do, over fewer scenarios.
    About(&'a QualifiedName),
}

impl Focus<'_> {
    /// Whether the scenarios of `command` are written.
    fn takes(self, command: &QualifiedName) -> bool {
        match self {
            Self::Whole => true,
            Self::About(focus) => focus == command,
        }
    }

    /// Whether every scenario is written.
    fn is_whole(self) -> bool {
        matches!(self, Self::Whole)
    }
}

/// [`synthesize`], for a model in which nothing depends on who sends a command.
fn synthesize_plain(ir: &EssIr, focus: Focus<'_>) -> Synthesis {
    synthesize_invocations(&caller::InvocationModels::plain(ir), focus)
}

#[allow(clippy::too_many_lines)] // Keep the ordered scenario-family dispatch together.
fn synthesize_invocations(models: &caller::InvocationModels<'_>, focus: Focus<'_>) -> Synthesis {
    let ir = models.arrangement;
    // A caller assignment's model is another model: its answers are held apart and dropped with it.
    let _memoised = crate::witness_memo::memoise(ir);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
    let mut refusals = Vec::new();
    for (path, subject) in ess_compiler::binary64::uses(ir) {
        refusals.push(Refusal { subject, scenario: None, cause: RefusalCause::NoWitness(WitnessGap {
            path, type_ref: "Binary64".to_owned(), reason: "requires a qualified finite Binary64 suite and codec that this conformance format does not admit",
        }) });
    }
    if !refusals.is_empty() {
        return Synthesis {
            suite,
            refusals,
            outside: Vec::new(),
            notes: Vec::new(),
        };
    }
    let actors = granted_actors(ir);

    let mut unseparated_notes = Vec::new();
    for command in models.acting.commands().values() {
        if !focus.takes(&command.name) {
            continue;
        }
        for outcome in &command.outcomes {
            // A wrong-state branch gets no scenario from here. §10 asks for one scenario per
            // *reachable* outcome, and the states this branch is reachable in are exactly the ones
            // the illegal-move family below already enumerates — one scenario each, against an
            // instance the arrangement really drove there; one more picking one of those states
            // would assert a strict subset of what they assert. Its own `/outcome/` id is filed
            // by `unknown_instances` instead, for the one case that family cannot arrange: an
            // identity naming no record (beyond10x/ess#113).
            // An `unknown_instance:` branch (ess/15) likewise: `unknown_instances` files it, sent
            // for an identity no record carries.
            // An `input_absent:` branch (ess/16) is filed by `absent_input::absent_inputs`, sent with
            // no input at all.
            // A set outcome (ess/16, `instances:`) is filed by `set_effects`, over rows it arranges.
            if matches!(
                outcome.condition,
                ResolvedCondition::WrongState
                    | ResolvedCondition::UnknownInstance
                    | ResolvedCondition::InputAbsent
                    | ResolvedCondition::ExistingInstance
            ) || outcome.instances.is_some()
            {
                continue;
            }
            let Some((id, scenario)) =
                outcome_scenario_in(models, command, outcome, &actors, &mut refusals)
            else {
                continue;
            };
            unseparated_notes.extend(unseparated(command, outcome, &id, &scenario));
            let fields = related_guard::unaccompanied(ir, command, outcome, &actors);
            if !fields.is_empty() {
                unseparated_notes.push(Note::UnaccompaniedRelatedCopy {
                    scenario: id.clone(),
                    fields,
                });
            }
            insert(&mut suite, id, scenario, &mut refusals);
        }
    }
    let mut partial = Vec::new();
    lifecycle(
        models,
        &actors,
        focus,
        &mut suite,
        &mut refusals,
        &mut partial,
    );
    state_refusals(models, &actors, focus, &mut suite, &mut refusals);
    let mut notes = Vec::new();
    unknown_instances(ir, &actors, focus, &mut suite, &mut refusals, &mut notes);
    absent_input::absent_inputs(ir, &actors, focus, &mut suite, &mut refusals);
    existence::existence(models, &actors, focus, &mut suite, &mut refusals);
    set_effects::set_effects(models, &actors, focus, &mut suite, &mut refusals);
    notes.extend(partial);
    notes.extend(unseparated_notes);
    invariants(models, &actors, focus, &mut suite, &mut refusals);
    if focus.is_whole() {
        bindings(ir, &actors, &mut suite, &mut refusals);
        aggregate::aggregates(ir, &actors, &mut suite, &mut refusals);
    }
    grant::denied(ir, &mut suite, &mut refusals, &mut notes);
    preconditions(models, &mut suite);
    for (id, reason) in crate::fixtures::install(ir, &mut suite) {
        suite.scenarios.remove(&id);
        refusals.push(Refusal::about(
            &id,
            RefusalCause::NoWitness(WitnessGap {
                path: reason,
                type_ref: "fixture inputs".into(),
                reason: "typed fixture resolution cannot execute this contract",
            }),
        ));
    }
    for (id, path, reason) in crate::now_offset::install(ir, &mut suite) {
        suite.scenarios.remove(&id);
        refusals.push(Refusal::about(
            &id,
            RefusalCause::NoWitness(WitnessGap {
                path,
                type_ref: "Timestamp".into(),
                reason,
            }),
        ));
    }
    suite.select_fresh_format_for(ir);

    Synthesis {
        suite,
        refusals,
        outside: Vec::new(),
        notes,
    }
}

/// The suite one component can be held to.
///
/// [`synthesize`] obliges the whole system, and a specification with two components obliges two
/// implementations. An implementation of one of them answers `ErrUnsupported` to every scenario
/// about the other — which the runner reports as a skip, and a run with skips in it is a run that
/// cannot say it passed. So the suite for a component holds exactly the scenarios whose every
/// command it accepts or owns the domain of, every event it publishes or owns, and every view it
/// owns; the rest are returned as [`Synthesis::outside`], each with what it needs, so a reader can
/// see which component's suite they belong in.
///
/// A negative event assertion counts for nothing here. `expect_no_event` names every event the
/// specification declares minus the branch's own, and an implementation can say it did not emit an
/// event it has never heard of; requiring the component to publish it would empty every suite.
///
/// The refusals are the whole system's, untouched: a refusal is a fact about the specification, and
/// scoping the suite does not make a construct say more.
///
/// # Errors
///
/// [`UnknownComponent`] when the specification declares no component of that name, carrying the
/// names it does declare.
pub fn synthesize_for(ir: &EssIr, component: &str) -> Result<Synthesis, UnknownComponent> {
    let Some(realised) = ir
        .components()
        .values()
        .find(|declared| declared.name.as_str() == component)
    else {
        return Err(UnknownComponent {
            component: component.to_owned(),
            declared: ir
                .components()
                .keys()
                .map(|name| name.as_str().to_owned())
                .collect(),
        });
    };

    let whole = synthesize(ir);
    let mut provenance = whole.suite.provenance.clone();
    provenance.component = Some(component.to_owned());
    let mut suite = ConformanceSuite::new(provenance);
    let mut outside = Vec::new();
    for (id, scenario) in whole.suite.scenarios {
        let needs = needs_of(ir, realised, &scenario);
        if needs.is_empty() {
            suite
                .insert(id, scenario)
                .expect("the whole suite held each id once, so its subset does too");
        } else {
            outside.push(Outside {
                scenario: id,
                needs,
            });
        }
    }
    suite.select_fresh_format_for(ir);
    Ok(Synthesis {
        suite,
        refusals: whole.refusals,
        outside,
        notes: whole.notes,
    })
}

/// A scenario a component's suite does not hold, and what it would need the component to realise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outside {
    /// The scenario, as the whole-system suite names it.
    pub scenario: ScenarioId,
    /// The commands, events and views another component realises, in name order.
    pub needs: Vec<EssSemanticRef>,
}

impl fmt::Display for Outside {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let needs: Vec<String> = self.needs.iter().map(|need| format!("`{need}`")).collect();
        write!(f, "`{}` needs {}", self.scenario, needs.join(", "))
    }
}

/// A component name the specification does not declare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownComponent {
    /// What was asked for.
    pub component: String,
    /// What the specification declares, in name order.
    pub declared: Vec<String>,
}

impl fmt::Display for UnknownComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no component `{}` is declared", self.component)?;
        if self.declared.is_empty() {
            write!(f, "; the specification declares no components")
        } else {
            let declared: Vec<String> = self
                .declared
                .iter()
                .map(|name| format!("`{name}`"))
                .collect();
            write!(f, "; it declares {}", declared.join(", "))
        }
    }
}

impl std::error::Error for UnknownComponent {}

/// What a scenario asks of an implementation that this component does not realise.
///
/// One entry per distinct construct, in name order, so the list is a function of the scenario and
/// not of the order its steps happen to name things.
pub(crate) fn needs_of(
    ir: &EssIr,
    component: &ResolvedComponent,
    scenario: &ConformanceScenario,
) -> Vec<EssSemanticRef> {
    let mut needs: BTreeSet<EssSemanticRef> = BTreeSet::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::CaptureCommandResult { capture } | ScenarioStep::ExpectReplayResult { capture } => {
                if !handles(ir, component, capture.origin.command.name()) { needs.insert(capture.origin.command.clone().into()); }
            }
            ScenarioStep::ExpectResponsePayload { response } => {
                if !handles(ir, component, response.command.name()) { needs.insert(response.command.clone().into()); }
                if !emits(ir, component, response.event.name()) { needs.insert(response.event.clone().into()); }
            }
            ScenarioStep::ExpectDirectResponse { response } => {
                if !handles(ir, component, response.command.name()) { needs.insert(response.command.clone().into()); }
            }
            ScenarioStep::CheckPeriodic { check } => {
                if !handles(ir, component, check.command.name()) { needs.insert(check.command.clone().into()); }
                if check.periodic.host.owner != component.name { needs.insert(ComponentRef::new(check.periodic.host.owner.clone()).into()); }
            }
            ScenarioStep::ExpectReadingOrder { left, right, .. } => {
                for reference in [left, right] {
                    if !emits(ir, component, reference.event.name()) {
                        needs.insert(reference.event.clone().into());
                    }
                }
            }
            ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExpectInvocation { command, .. }
            | ScenarioStep::ExpectEveryInvocation { command, .. }
            | ScenarioStep::ExpectNoInvocation { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => {
                if !handles(ir, component, command.name()) {
                    needs.insert(command.clone().into());
                }
            }
            ScenarioStep::ConfigureExternalOutcome { force, .. } => {
                if !handles(ir, component, force.command.name()) {
                    needs.insert(force.command.clone().into());
                }
            }
            ScenarioStep::ExpectEvent { event, .. }
            | ScenarioStep::ExpectEventValues { event, .. }
            | ScenarioStep::EventuallyEvent { event, .. }
            | ScenarioStep::RedeliverEvent { event, .. }
            | ScenarioStep::CaptureInstance { event, .. } => {
                if !emits(ir, component, event.name()) {
                    needs.insert(event.clone().into());
                }
            }
            ScenarioStep::QueryView { view, .. }
            | ScenarioStep::SnapshotCompleteSubject { view, .. }
            | ScenarioStep::ExpectCompleteSubjectUnchanged { view }
            | ScenarioStep::SnapshotSubject { view, .. }
            | ScenarioStep::ExpectSubjectUnchanged { view }
            | ScenarioStep::ExpectSubjectAbsent { view, .. }
            | ScenarioStep::SnapshotView { view }
            | ScenarioStep::ExpectViewUnchanged { view }
            | ScenarioStep::ExpectView { view, .. }
            | ScenarioStep::EventuallyView { view, .. }
            // A halt is a *read* of a view, so it needs the view the same way a query does — the
            // component that does not publish it cannot be asked to stop producing it either.
            | ScenarioStep::ExpectHalt { view, .. }
            | ScenarioStep::EventuallyHalt { view, .. } => {
                if !owns_view(ir, component, view.name()) {
                    needs.insert(view.clone().into());
                }
            }
            // About the command the scenario just ran, about an event it must *not* have
            // published, or about how much time passed — none asks the component to realise
            // anything more. `ExpectQuiet` names an event the way `ExpectNoEvent` does, and for the
            // same reason it is here: a component that never emits it satisfies the claim, so
            // requiring it to realise the event would scope the scenario out of the one component
            // it is most obviously about.
            // Fixture setup is an adapter capability, not a declared command/event realization.
            // Keep upstream-backed view witnesses in the component that owns the view.
            // An event from an external channel is emitted by no component: the suite delivers it,
            // so the one that reacts needs nothing more than the command it invokes.
            ScenarioStep::DeliverEvent { .. }
            | ScenarioStep::EstablishEntity { .. }
            | ScenarioStep::ResolveFixtures { .. }
            | ScenarioStep::ExpectNoEvents
            | ScenarioStep::ExpectOutcome { .. }
            | ScenarioStep::ExpectNotGranted { .. }
            | ScenarioStep::ExpectNoError
            | ScenarioStep::ExpectError { .. }
            | ScenarioStep::ExpectNoEvent { .. }
            | ScenarioStep::MarkInstant { .. }
            | ScenarioStep::ExpectNotBefore { .. }
            | ScenarioStep::ExpectWithin { .. }
            | ScenarioStep::ExpectQuiet { .. } => {}
        }
    }
    needs.into_iter().collect()
}

/// Whether a component is the handler of a command: it accepts it, or owns the domain it is in.
pub(crate) fn handles(ir: &EssIr, component: &ResolvedComponent, command: &QualifiedName) -> bool {
    component
        .accepts
        .iter()
        .any(|accepted| accepted.name() == command)
        || ir
            .commands()
            .get(command)
            .is_some_and(|declared| component.owns.contains(&declared.domain))
}

/// Whether a component is where an event comes from: it publishes it, or owns the domain it is in.
fn emits(ir: &EssIr, component: &ResolvedComponent, event: &QualifiedName) -> bool {
    component
        .publishes
        .iter()
        .any(|published| published.name() == event)
        || ir
            .events()
            .get(event)
            .is_some_and(|declared| component.owns.contains(&declared.domain))
}

/// Whether a component projects a view: a view is declared inside a domain, so this is ownership.
pub(crate) fn owns_view(ir: &EssIr, component: &ResolvedComponent, view: &QualifiedName) -> bool {
    ir.views()
        .get(view)
        .is_some_and(|declared| component.owns.contains(&declared.domain))
}

/// One scenario per declared outcome (§10), or the refusal that says why there is none.
fn outcome_scenario_in(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    refusals: &mut Vec<Refusal>,
) -> Option<(ScenarioId, ConformanceScenario)> {
    let id = ScenarioId::Outcome {
        outcome: OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone()),
    };
    let (mut steps, mut source, run) = exercise_as(
        models,
        command,
        outcome,
        actors,
        &id,
        refusals,
        Witness::Full,
    )?;
    let (further, depends) = boundaries(models, command, outcome, actors, &run, &steps);
    steps.extend(further);
    source.extend(depends);
    // ess/16 (#163): the branch again with every input it reads only through `else: <literal>`
    // left out, after everything the full invocation asserts, so the suite asserts both halves of
    // `{input: f, else: <literal>}` — the sent value wins, and the literal stands in for none.
    // Not a refusal where it cannot be built: the full invocation above is the scenario. A
    // replayed branch and a state refusal are arranged by their own searches, and not again.
    let again = outcome.replays.is_none() && !is_state_refusal(command, outcome);
    if let Some((more, depends, _)) = again
        .then(|| {
            exercise_as(
                models,
                command,
                outcome,
                actors,
                &id,
                &mut Vec::new(),
                Witness::LiteralFallbacks,
            )
        })
        .flatten()
    {
        steps.extend(more);
        source.extend(depends);
    }
    // ess/18 (#201): a branch whose `when_subject_state:` lists several states is witnessed in each
    // of them, on a further row per state after the first invocation, so a target that mishandles
    // any listed state fails. A state refusal is witnessed per state by `state_refusals`.
    if let ResolvedCondition::SubjectState { state, .. } = &outcome.condition {
        if state.is_listed() && outcome.subject.is_some() && !is_state_refusal(command, outcome) {
            for (nth, held) in state.iter().enumerate() {
                if run.before.as_ref() == Some(held) {
                    continue;
                }
                if let Some((more, depends, _)) = exercise_as(
                    models,
                    command,
                    outcome,
                    actors,
                    &id,
                    refusals,
                    Witness::Listed(nth),
                ) {
                    steps.extend(more);
                    source.extend(depends);
                }
            }
        }
    }
    let (more, depends) = related_boundaries(models, command, outcome, actors, &id, refusals);
    steps.extend(more);
    source.extend(depends);
    // ess/22 (#304): a branch an absent Optional reference selects is witnessed once more on a
    // further instance, with the reference left out, so a target reading absence as a missing row
    // — or reading some row of the entity — fails this scenario.
    if related_guard::absent_selects(models.arrangement, command, outcome) {
        let nth = related_guard::boundary_goals(models.arrangement, command, outcome).len() + 1;
        if let Some((more, depends, _)) = exercise_as(
            models,
            command,
            outcome,
            actors,
            &id,
            refusals,
            Witness::RelatedAbsent(nth),
        ) {
            steps.extend(more);
            source.extend(depends);
        }
    }
    if again {
        let (more, depends) = absent_references(models, command, outcome, actors, &id, refusals);
        steps.extend(more);
        source.extend(depends);
    }
    Some((
        id,
        ConformanceScenario::new(purpose(command, outcome), steps, source),
    ))
}

/// The absent-reference witnesses of one branch's scenario (ess/22, beyond10x/ess#285): the branch
/// copying a value through a reference that may be absent is run once more per such reference, on
/// a further instance with that reference left absent and every other present, and the copied
/// value is asserted absent — so a target reading absence as a missing row, or copying some row's
/// value anyway, fails the scenario. A run that cannot be built, and a reference no run can leave
/// absent, leave the scenario standing; each is refused under its id naming the reference.
fn absent_references(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> (Vec<ScenarioStep>, BTreeSet<EssSemanticRef>) {
    let ir = models.arrangement;
    let (mut steps, mut source) = (Vec::new(), BTreeSet::new());
    for point in 0..related::absence_points(ir, command, outcome).len() {
        let mut failed = Vec::new();
        let witness = Witness::RelatedValueAbsent(point);
        if let Some((more, depends, _)) =
            exercise_as(models, command, outcome, actors, id, &mut failed, witness)
        {
            steps.extend(more);
            source.extend(depends);
            continue;
        }
        let cause = match failed.into_iter().next() {
            Some(Refusal {
                cause: cause @ RefusalCause::AbsenceUnwitnessed { .. },
                ..
            }) => cause,
            other => RefusalCause::AbsenceUnwitnessed {
                reference: related::point_reference(ir, outcome, point),
                reason: other.map_or_else(
                    || "the run that leaves it absent could not be built".to_owned(),
                    |refusal| refusal.cause.to_string(),
                ),
            },
        };
        refusals.push(Refusal::about(id, cause));
    }
    for (reference, reason) in related::unwitnessable(ir, command, outcome) {
        refusals.push(Refusal::about(
            id,
            RefusalCause::AbsenceUnwitnessed { reference, reason },
        ));
    }
    (steps, source)
}

/// Further related rows one branch's scenario is witnessed on (ess/18, beyond10x/ess#211): a
/// related predicate with two or more connective children is witnessed once more per child, on a
/// further related row isolating it, with whichever branch the command answers there asserted — so
/// a target dropping one conjunct, or one disjunct, fails. A boundary no bounded arrangement reaches
/// is refused under this scenario's id, never dropped.
///
/// Each side of a counter limit (beyond10x/ess#226) is one more such row. One whose search left
/// every row it could without holding it — `open_cards == 4` where nothing raises the counter past
/// 3 — is a row no run of the model holds, and adds none; one the search went past its bound for, or
/// stopped short of for another cause, is refused; so is one whose nearest value a run holds lies
/// past the search's reach ([`subject_fact::Further::Past`]), without a search.
fn related_boundaries(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> (Vec<ScenarioStep>, BTreeSet<EssSemanticRef>) {
    let ir = models.arrangement;
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    if !related_guard::routes(command, outcome) {
        return (steps, source);
    }
    let of = command
        .outcomes
        .iter()
        .position(|branch| branch.name == outcome.name)
        .expect("the outcome is the command's");
    let goals = related_guard::boundary_goals(ir, command, outcome);
    for (goal, ((refuted, held), kind)) in goals.iter().enumerate() {
        if let subject_fact::Further::Past(why) = kind {
            refusals.push(Refusal::about(
                id,
                RefusalCause::GuardUnsatisfiable {
                    predicate: format!(
                        "{}, one side of a stored counter's limit: {why}",
                        related_guard::describe_goal(refuted, held),
                    ),
                    tried: 0,
                },
            ));
            continue;
        }
        let limit = &(*kind == subject_fact::Further::Limit);
        let answering: Vec<&ResolvedOutcome> = command
            .outcomes
            .iter()
            .filter(|branch| related_guard::routes(command, branch))
            .collect();
        let mut causes = Vec::new();
        let found = answering.iter().find_map(|branch| {
            exercise_as(
                models,
                command,
                branch,
                actors,
                id,
                &mut causes,
                Witness::RelatedBoundary { of, goal },
            )
        });
        match found {
            Some((more, depends, _)) => {
                steps.extend(more);
                source.extend(depends);
            }
            None if *limit
                && !causes.is_empty()
                && causes.iter().all(|refusal: &Refusal| {
                    matches!(refusal.cause, RefusalCause::GuardUnsatisfiable { .. })
                        && !subject_fact::is_beyond_reach(&refusal.cause)
                }) => {}
            None if *limit => refusals.push(Refusal::about(
                id,
                RefusalCause::GuardUnsatisfiable {
                    predicate: format!(
                        "{}, one side of a stored counter's limit: {}",
                        related_guard::describe_goal(refuted, held),
                        causes
                            .iter()
                            .map(|refusal| refusal.cause.to_string())
                            .collect::<Vec<_>>()
                            .join("; ")
                    ),
                    tried: answering.len(),
                },
            )),
            None => refusals.push(Refusal::about(
                id,
                RefusalCause::GuardUnsatisfiable {
                    predicate: related_guard::describe_goal(refuted, held),
                    tried: answering.len(),
                },
            )),
        }
    }
    (steps, source)
}

/// Which invocation of a branch a run builds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Witness {
    /// The witness input, every optional input it can send included.
    Full,
    /// A further instance and input, with every optional input the branch reads only through
    /// `{input: f, else: <literal>}` left out ([`without_literal_fallbacks`]). A run that can leave
    /// nothing out is not built.
    LiteralFallbacks,
    /// A further instance arranged in the `n`th state a listed `when_subject_state:` names (ess/18,
    /// beyond10x/ess#201), so every listed state is witnessed and not only the first one reached.
    Listed(usize),
    /// A further related row the `goal`th boundary of the related predicate of branch `of` (by
    /// index) names, and the branch the command answers with on it (ess/18, beyond10x/ess#211,
    /// [`related_guard::boundary_goals`]).
    RelatedBoundary {
        /// The branch whose predicate the boundary belongs to, by its index in the command.
        of: usize,
        /// Which of that predicate's boundaries.
        goal: usize,
    },
    /// A further instance sent with the Optional reference the command's related guards read left
    /// out, between related rows that select a refusal (ess/22, beyond10x/ess#304,
    /// [`related_guard::prepare_absent_in`]), at the `n`th further distinction.
    RelatedAbsent(usize),
    /// A further instance whose `{related: …}` sources follow the `n`th reference that may be
    /// absent ([`related::absence_points`]) left absent, and so copy an absent value (ess/22,
    /// beyond10x/ess#285, [`arranged_with_absent_reference`]).
    RelatedValueAbsent(usize),
}

/// Arrange the instance the branch acts on, run the branch, and assert everything it promises.
///
/// The body §10 and §19 share. An outcome scenario and a transition scenario differ in the id they
/// are filed under and in the sentence they print, and not in what they do — §10's unit is the
/// branch and §19's is the move, and a suite that dropped one because it resembled the other would
/// lose the id a fault matrix refers to.
fn exercise(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> Option<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>, Run)> {
    exercise_as(
        &caller::InvocationModels::plain(ir),
        command,
        outcome,
        actors,
        id,
        refusals,
        Witness::Full,
    )
}

/// Exercise one explicitly interpreted acting invocation and its arrangement.
fn exercise_as(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
    witness: Witness,
) -> Option<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>, Run)> {
    let run = match run_as(models, command, outcome, actors, witness) {
        Ok(run) => run,
        Err(cause) => {
            refusals.push(Refusal::about(id, cause));
            return None;
        }
    };

    exercise_run_in(models, command, outcome, actors, id, refusals, run)
}

/// Assert a branch whose invocation was arranged by the caller, including post-state retries.
fn exercise_run(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
    run: Run,
) -> Option<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>, Run)> {
    exercise_run_in(
        &caller::InvocationModels::plain(ir),
        command,
        outcome,
        actors,
        id,
        refusals,
        run,
    )
}

fn exercise_run_in(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
    run: Run,
) -> Option<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>, Run)> {
    let ir = models.arrangement;
    let emitted: Vec<EventRef> = outcome.emits.iter().map(EventRef::from).collect();
    let absent = not_emitted(ir, &emitted);
    let actor = run.actor.clone();
    // The row at rest once the bindings this branch sets off on it have run, read eventually
    // (beyond10x/ess#266): never the state a binding is about to move it out of.
    let mut run = run;
    let moving = binding_effects::settle_run(ir, outcome, &mut run, id, refusals);
    let mut views = view_expectations(ir, command, outcome, &run, actors, id, refusals);
    if moving {
        views.asserted = binding_effects::eventually(std::mem::take(&mut views.asserted));
    }
    models.mark(caller::InvocationPhase::Arrange, &mut views.arranged);

    let mut steps = run.steps();
    if let Some(error) = &outcome.error {
        let (input, before) = (&run.input, &run.before_settled);
        steps.push(expect_error(ir, outcome, error, input, before));
    }
    for event in &emitted {
        let literals = determined_payload(ir, outcome, event, &run.input, &run.before_settled);
        let mut shape = crate::response::event_shape(ir, event, outcome);
        absent_by_reference_leaves(ir, outcome, event, &run, &mut shape);
        let mut references = determined_identities(
            ir,
            outcome,
            event,
            &run.input,
            &run.before_settled,
            run.instance.as_ref(),
        );
        references.extend(crate::fixtures::event_values(outcome, event, &run.input));
        steps.push(expect_event_step(event, literals, references, shape));
    }
    if outcome.returns {
        match crate::direct_response::Observation::of(
            ir,
            command,
            Some(OutcomeRef::new(
                CommandRef::new(command.name.clone()),
                outcome.name.clone(),
            )),
            BTreeMap::new(),
        ) {
            Ok(response) => steps.push(ScenarioStep::ExpectDirectResponse { response }),
            Err(reason) => {
                refusals.push(Refusal::about(
                    id,
                    RefusalCause::NoWitness(WitnessGap {
                        path: format!("{}.response: {reason}", command.name),
                        type_ref: "command response".into(),
                        reason: "typed direct response observation cannot execute this contract",
                    }),
                ));
                return None;
            }
        }
    }
    match crate::response::Observation::of(ir, command, outcome) {
        Ok(observations) => steps.extend(
            observations
                .into_iter()
                .map(|response| ScenarioStep::ExpectResponsePayload { response }),
        ),
        Err(reason) => {
            refusals.push(Refusal::about(
                id,
                RefusalCause::NoWitness(WitnessGap {
                    path: format!("{}.response: {reason}", command.name),
                    type_ref: "command response".into(),
                    reason: "typed response observation cannot execute this contract",
                }),
            ));
            return None;
        }
    }
    // §10's first-class negative assertion: without it the refusal case passes against an
    // implementation that refuses the command and emits the success event anyway.
    for event in &absent {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    steps.extend(run.after_steps.iter().cloned());
    let mut removed = BTreeSet::new();
    let mut deleted = deletion_witness(ir, command, outcome, &run, &mut removed);
    models.mark(caller::InvocationPhase::Act, &mut deleted);
    steps.extend(deleted);
    // After everything that reads the branch, and before anything that reads a view. Both halves of
    // that are load-bearing. Put later, the arrangement would run after the view it exists to fill;
    // put earlier, its own creating command would publish the first occurrence of the event the
    // branch publishes — and `creates:` says the new identity is *in that event*, so every reference
    // to the instance this scenario is about would resolve to a neighbour's.
    steps.extend(views.arranged);
    steps.extend(views.asserted);

    let mut source = dependencies(ir, command, outcome, &absent, actor, &views.views);
    source.extend(removed);
    source.extend(run.source.iter().cloned());
    source.extend(views.source);
    record_refused(id, &run, refusals);
    Some((steps, source, run))
}

/// Records each further row `run` refused on its own ([`Run::refused`]) under the scenario's id,
/// once however many invocations of the branch arranged it.
fn record_refused(id: &ScenarioId, run: &Run, refusals: &mut Vec<Refusal>) {
    for cause in &run.refused {
        let refusal = Refusal::about(id, cause.clone());
        if !refusals.contains(&refusal) {
            refusals.push(refusal);
        }
    }
}

/// One branch, arranged and run: everything before the assertions that are particular to a family.
///
/// The three families that execute a command all need the same four things — an instance in the
/// state the branch may be taken from, an input that reaches the branch, the invocation, and the
/// requirement that the declared branch was the one taken — and they differ only in what they assert
/// afterwards. Sharing it is not only economy: an arrangement built two ways is two answers to
/// "which invoice is this scenario about", and the second one is wrong eventually.
///
/// [`Run::steps`] is kept separate from the arrangement so a caller can inject something *between*
/// them — which §18's failure scenario needs, because the control it arms must be armed after the
/// arrangement's own commands have run and before the one that triggers the binding.
#[derive(Clone)]
struct Run {
    /// The steps that bring the instance into the state the branch needs.
    setup: Vec<ScenarioStep>,
    /// Forcing the branch where it is externally decided, invoking it, and requiring it.
    invoke: Vec<ScenarioStep>,
    /// The state the subject is in afterwards, where there is a subject.
    after: Option<StateName>,
    /// The state the arrangement left the subject in before the branch, where it arranged one.
    before: Option<StateName>,
    /// What the arrangement bound the instance as, where it arranged one.
    ///
    /// Carried out of the arrangement rather than dropped there, because a view assertion has to
    /// name the row it is about: "the view holds a row" is only the same claim as "the view holds
    /// *this* invoice" while nothing else is using the target.
    instance: Option<InstanceName>,
    /// The actor the command is invoked as, where the specification grants one.
    actor: Option<ActorRef>,
    /// What the invocation supplied, by declared field name.
    ///
    /// Carried out of the arrangement because a payload assertion needs it: an event field the
    /// outcome's `payload:` determines from an input is assertable exactly where this map holds
    /// the literal the suite chose for that input.
    input: BTreeMap<String, ScenarioValue>,
    /// What the arrangement depends on.
    source: BTreeSet<EssSemanticRef>,
    /// Observations of the subject that belong after the branch's own assertions: the row a
    /// branch that names no subject of its own leaves exactly as it was observed before it.
    after_steps: Vec<ScenarioStep>,
    /// What the subject's fields hold once the branch under test has run.
    ///
    /// The arrangement's, with this branch's own `sets:` applied over the top — this branch runs
    /// last, so where both name a field it is this one's value the row will carry.
    settled: BTreeMap<String, Determined>,
    /// What the subject's fields held before the branch under test ran: the arrangement's. An ess/14
    /// payload source that reads the subject is asserted against this.
    before_settled: BTreeMap<String, Determined>,
    /// Further rows of the scenario refused on their own while the scenario stands: a side of a
    /// counter limit no bounded row reaches (beyond10x/ess#226). Each is recorded under the
    /// scenario's id.
    refused: Vec<RefusalCause>,
}

impl Run {
    /// The arrangement and the invocation, in order.
    fn steps(&self) -> Vec<ScenarioStep> {
        let mut steps = self.setup.clone();
        steps.extend(self.invoke.iter().cloned());
        steps
    }
}

/// Arranges the instance a branch acts on and invokes it, or says why neither is possible.
fn run(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Run, RefusalCause> {
    run_as(
        &caller::InvocationModels::plain(ir),
        command,
        outcome,
        actors,
        Witness::Full,
    )
}

/// [`run`], for the invocation `witness` names.
#[allow(clippy::too_many_lines)]
fn run_as(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    witness: Witness,
) -> Result<Run, RefusalCause> {
    let ir = models.arrangement;
    if let Some(replay) = &outcome.replays {
        return run_replay_in(models, command, outcome, replay, actors);
    }
    if is_state_refusal(command, outcome) {
        let subject = command
            .selection_subject(outcome)
            .expect("validated common selection subject");
        let mut first = None;
        for state in &ir.entity(&subject.entity).lifecycle.states {
            match run_state_refusal(ir, command, outcome, state, actors, Distinction::PLAIN) {
                Ok(mut run) => {
                    models.mark_run(&mut run);
                    return Ok(run);
                }
                Err(reason) => {
                    first.get_or_insert(reason);
                }
            }
        }
        return Err(first.expect("nonempty finite lifecycle"));
    }
    // A branch copying its input into an entity whose invariants over it no bounded input meets
    // is refused naming them, never sent an input its own entity refuses (beyond10x/ess#234).
    if let Some((named, tried)) = crate::witness::unmet_invariants(ir, command, outcome, false)
        .map_err(RefusalCause::NoWitness)?
    {
        return Err(RefusalCause::GuardUnsatisfiable {
            predicate: named,
            tried,
        });
    }
    let routed = subject_fact::routes(command, outcome);
    // A command guarded by a related row (ess/18, #211) is arranged with that row, or its absence,
    // for every branch it decides; further witnesses are the boundaries of its predicates alone.
    let related = related_guard::routes(command, outcome);
    let mut related_at = Distinction::PLAIN;
    let (mut setup, input) = if related {
        match witness {
            Witness::Full => related_guard::prepare_at_in(
                models,
                command,
                outcome,
                actors,
                Distinction::PLAIN,
                None,
            )?,
            Witness::RelatedBoundary { of, goal } => {
                let goals = command
                    .outcomes
                    .get(of)
                    .map(|branch| related_guard::boundary_goals(ir, command, branch))
                    .unwrap_or_default();
                let (named, _) = goals.get(goal).ok_or_else(related_guard::unarranged)?;
                related_at = Distinction::further(goal + 1);
                related_guard::prepare_at_in(
                    models,
                    command,
                    outcome,
                    actors,
                    related_at,
                    Some(named),
                )?
            }
            Witness::RelatedAbsent(nth) => {
                related_at = Distinction::further(nth);
                related_guard::prepare_absent_in(models, command, outcome, actors, related_at)?
            }
            Witness::LiteralFallbacks | Witness::Listed(_) | Witness::RelatedValueAbsent(_) => {
                return Err(related_guard::unarranged())
            }
        }
    } else {
        // A guard no input meets beside the invariants of the entity it copies the input into is
        // refused naming them, where the guard alone is met (beyond10x/ess#234).
        arranged_as(ir, command, outcome, actors, routed, witness).map_err(|cause| {
            match (
                &cause,
                crate::witness::unmet_invariants(ir, command, outcome, true),
            ) {
                (RefusalCause::GuardUnsatisfiable { .. }, Ok(Some((named, tried)))) => {
                    RefusalCause::GuardUnsatisfiable {
                        predicate: named,
                        tried,
                    }
                }
                _ => cause,
            }
        })?
    };
    // The input as it is sent, after every arrangement moved it: still within the invariants of
    // the entity the branch copies it into (beyond10x/ess#234).
    if let Some(named) = crate::witness::invariant_broken_by(ir, command, outcome, &input) {
        return Err(RefusalCause::GuardUnsatisfiable {
            predicate: named,
            tried: 1,
        });
    }

    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    let actor = actors.get(&command.name).cloned();

    let mut invoke = Vec::new();
    if outcome.test_strategy == TestStrategy::InjectFault {
        // §12: no predicate over a recipient and a template says whether a provider will accept the
        // mail, so the suite injects the answer rather than inventing an input that produces it.
        invoke.push(ScenarioStep::ConfigureExternalOutcome {
            force: outcome_ref.clone(),
            times: None,
        });
    }
    // A branch reading stored fields that names no subject of its own — a refusal — reads the one
    // its siblings name, and is sent for the row the arrangement made for it.
    let reads = if routed {
        subject_fact::reading(command, outcome)
    } else {
        outcome.subject.as_ref()
    };
    let supplied = supply(
        command,
        &input,
        reads,
        setup.instance.as_ref(),
        &setup.bound,
    );
    invoke.push(ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref,
        actor: actor.clone(),
        input: supplied.clone(),
    });
    invoke.push(ScenarioStep::ExpectOutcome {
        outcome: outcome_ref,
    });
    // A creation guarded by a related row (ess/18, #211) is sent after that row's own creation,
    // which may publish the same event — a folder inside a folder. So the new row is bound here,
    // from this command's events, and later steps name it rather than the first occurrence. So is
    // the row a further run of a creating branch makes for its absent-reference witness (ess/22,
    // beyond10x/ess#285): the scenario already ran the branch once.
    let captured = if let Witness::RelatedValueAbsent(point) = witness {
        related_at = Distinction::further(ABSENT_REFERENCE_WITNESS + point);
        true
    } else {
        false
    };
    if related || captured {
        if let Some(ResolvedSubject {
            entity,
            effect: ResolvedEffect::Creates,
            instance: ResolvedInstance::Observed { event, field },
            ..
        }) = &outcome.subject
        {
            let name = instance_name(&ir.entity(entity).name, related_at);
            invoke.push(ScenarioStep::CaptureInstance {
                instance: name.clone(),
                entity: EntityRef::from(entity),
                event: EventRef::from(event),
                field: field.name.clone(),
            });
            setup.instance = Some(name);
        }
    }

    if (subject_fact::uses(command) || related) && outcome.error.is_none() {
        invoke.push(ScenarioStep::ExpectNoError);
    }
    let (mut after_steps, refused) = if routed {
        subject_fact::around(models, command, outcome, actors, &mut setup, &supplied)?
    } else {
        (Vec::new(), Vec::new())
    };
    accepts_nothing(ir, outcome, &mut setup, &mut invoke, &mut after_steps);
    if outcome
        .subject
        .as_ref()
        .is_some_and(|subject| subject.effect == ResolvedEffect::Preserves)
    {
        if !subject_fact::uses(command) {
            invoke.push(ScenarioStep::ExpectNoError);
        }
        let preservation = subject_fact::preservation(ir, outcome, &setup)?;
        setup.steps.extend(preservation.before);
        invoke.extend(preservation.after);
        setup.source.extend(preservation.source);
    }

    let mut source = setup.source;
    if has_subject_guards(command) {
        if let (Some(instance), Some(state)) = (&setup.instance, &setup.after) {
            let (observed, view) = observe_subject_state(ir, outcome, instance, state)?;
            invoke.extend(observed);
            source.insert(view.into());
        }
    }

    let before_settled = setup.settled.clone();
    let mut settled = setup.settled;
    absorb(
        &mut settled,
        outcome,
        super::synthesize::settled(ir, outcome, &supplied, &before_settled),
    );
    let mut run = Run {
        after_steps,
        setup: setup.steps,
        invoke,
        after: setup.after,
        before: setup.before,
        instance: setup.instance,
        actor,
        input: supplied,
        source,
        before_settled,
        refused,
        settled,
    };
    models.mark_run(&mut run);
    Ok(run)
}

/// The subject arranged for a branch and the input chosen for it, by whichever of the three
/// arrangements the branch's condition calls for — with the input moved off what the row already
/// holds where the branch writes it ([`freshened`]). A branch reading the stored row is arranged
/// by a search that chose the row and the input together, and is left as that search chose it.
fn arranged(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    routed: bool,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    if routed {
        // The stored-row search chose the row and the input together; it is asked for a row the
        // branch's writes change first, and only then for any row (beyond10x/ess#161).
        return subject_fact::prepare(ir, command, outcome, actors);
    }
    let (setup, input) = if has_subject_guards(command)
        && !existence::creates_unknown(outcome)
        && !is_state_input_refusal(command, outcome)
    {
        prepare_state_input(ir, command, outcome, actors)?
    } else {
        (
            prepare(ir, outcome, actors, None)?,
            reach(ir, command, outcome, Distinction::PLAIN)?,
        )
    };
    let input = existence::fresh_created(ir, command, outcome, input, false)?;
    let fresh = freshened(
        ir,
        command,
        outcome,
        input.clone(),
        setup.before.as_ref(),
        &setup.settled,
    );
    // A write the plain arrangement already holds proves nothing about the write (beyond10x/ess#161):
    // a literal `sets:` cannot be moved by the input, so the row is arranged again under a further
    // witness until it held something else, where one does — in the same held state, so a branch
    // selected by that state is still selected.
    let unchanged = unchanged_writes(ir, outcome, &fresh, &setup.settled);
    if unchanged > 0 {
        let held = has_subject_guards(command)
            .then(|| setup.before.clone())
            .flatten();
        for nth in 1..=FRESH_WITNESSES {
            let Ok(mut further) = prepare_in(
                ir,
                outcome,
                actors,
                held.as_ref(),
                Distinction::further(nth),
            ) else {
                continue;
            };
            if let (Some(state), Some(instance)) = (&held, &further.instance) {
                let Ok((observed, view)) = observe_subject_state(ir, outcome, instance, state)
                else {
                    continue;
                };
                further.steps.extend(observed);
                further.source.insert(view.into());
            }
            let moved = freshened(
                ir,
                command,
                outcome,
                input.clone(),
                further.before.as_ref(),
                &further.settled,
            );
            if unchanged_writes(ir, outcome, &moved, &further.settled) < unchanged {
                return Ok((further, moved));
            }
        }
    }
    Ok((setup, fresh))
}

/// How many of a branch's `sets:` entries write the value the existing row already holds, where the
/// scenario can know both: a literal, or the input's own value, against [`held_value`]. Nothing for
/// a branch that acts on no existing row.
fn unchanged_writes(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    input: &BTreeMap<String, Node>,
    settled: &BTreeMap<String, Determined>,
) -> usize {
    if !outcome.subject.as_ref().is_some_and(|subject| {
        matches!(
            subject.effect,
            ResolvedEffect::Updates | ResolvedEffect::Moves { .. }
        )
    }) {
        return 0;
    }
    outcome
        .sets
        .iter()
        .filter(|set| {
            let written = match &set.value {
                ResolvedPayloadValue::Literal { value } => {
                    literal_value(ir, &set.target_type, value, 0)
                }
                ResolvedPayloadValue::InputField { field, .. } => input.get(field).cloned(),
                _ => None,
            };
            written.is_some() && written == held_value(ir, settled, &set.target, &set.target_type)
        })
        .count()
}

/// Whether `outcome` is a refusal of a held-state command that names no subject of its own: the
/// effect-free default (ess/7), or a refusal guarded by a literal held state (ess/18,
/// beyond10x/ess#201). Both read the subject their siblings name, are witnessed on a row arranged in
/// each held state that selects them, and change nothing.
fn is_state_refusal(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    has_subject_guards(command)
        && (state_default(outcome)
            || matches!(outcome.condition, ResolvedCondition::SubjectState { .. }))
        && outcome.error.is_some()
        && outcome.subject.is_none()
        && outcome.replays.is_none()
}

fn run_state_refusal(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    state: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
) -> Result<Run, RefusalCause> {
    let input = reach_in_state(ir, command, outcome, state, distinction)?;
    let subject = command
        .selection_subject(outcome)
        .expect("validated common selection subject");
    let arranged = arrange_first(
        ir,
        &subject.entity,
        std::slice::from_ref(state),
        actors,
        distinction,
        &[],
    )
    .map_err(|reason| RefusalCause::InstanceRequired {
        entity: EntityRef::from(&subject.entity),
        need: InstanceNeed::Updates,
        reason,
    })?;
    let input = supply(
        command,
        &input,
        Some(subject),
        Some(&arranged.instance),
        &BTreeMap::new(),
    );
    let setup = Setup {
        instance: Some(arranged.instance.clone()),
        after: Some(state.clone()),
        settled: arranged.settled.clone(),
        ..Setup::none()
    };
    let preservation = subject_fact::preserve_complete_subject(ir, subject, &setup)?;
    let (observed, view) = observe_selection_subject(
        ir,
        subject,
        &arranged.instance,
        state,
        &outcome.name.to_string(),
    )?;
    let mut steps = arranged.steps;
    steps.extend(observed);
    steps.extend(preservation.before);
    let actor = actors.get(&command.name).cloned();
    let mut invoke = vec![
        ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: CommandRef::new(command.name.clone()),
            actor: actor.clone(),
            input: input.clone(),
        },
        ScenarioStep::ExpectOutcome {
            outcome: OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone()),
        },
        ScenarioStep::ExpectNoEvents,
    ];
    invoke.extend(preservation.after);
    let mut source = arranged.source;
    source.extend(preservation.source);
    source.insert(view.into());
    source.insert(
        OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone()).into(),
    );
    Ok(Run {
        after_steps: Vec::new(),
        setup: steps,
        invoke,
        after: Some(state.clone()),
        before: Some(state.clone()),
        instance: Some(arranged.instance),
        actor,
        input,
        source,
        before_settled: arranged.settled.clone(),
        refused: Vec::new(),
        settled: arranged.settled,
    })
}

/// One `<entity>/state/<S>/refuses/<command>` scenario per held state a subjectless refusal is
/// selected in.
///
/// Every such refusal of the command that some input selects in `S` is witnessed there, in
/// declaration order, each on a row of its own (a further distinction per refusal after the first),
/// so two refusals of one state that the input tells apart (ess/18, beyond10x/ess#201) share the one
/// id the state has and are both asserted, rather than colliding on it.
fn state_refusals(
    models: &caller::InvocationModels<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    focus: Focus<'_>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let ir = models.arrangement;
    for command in models.acting.commands().values() {
        if !focus.takes(&command.name) {
            continue;
        }
        let refusing: Vec<&ResolvedOutcome> = command
            .outcomes
            .iter()
            .filter(|o| is_state_refusal(command, o))
            .collect();
        let Some(first) = refusing.first() else {
            continue;
        };
        let subject = command
            .selection_subject(first)
            .expect("validated common selection subject");
        for state in &ir.entity(&subject.entity).lifecycle.states {
            let id = ScenarioId::Refusal {
                entity: EntityRef::from(&subject.entity),
                state: state.clone(),
                command: CommandRef::new(command.name.clone()),
                refuses: true,
            };
            let mut steps = Vec::new();
            let mut source = BTreeSet::new();
            let mut failed = false;
            let mut witnessed = 0;
            for outcome in &refusing {
                let distinction = if witnessed == 0 {
                    Distinction::PLAIN
                } else {
                    Distinction::further(witnessed)
                };
                match run_state_refusal(ir, command, outcome, state, actors, distinction) {
                    Ok(mut run) => {
                        models.mark_run(&mut run);
                        witnessed += 1;
                        steps.extend(run.steps());
                        steps.push(expect_error(
                            ir,
                            outcome,
                            outcome.error.as_ref().expect("named refusal"),
                            &run.input,
                            &run.before_settled,
                        ));
                        source.extend(run.source);
                    }
                    // No input reaches this refusal in this state: not a refusal of its own, as
                    // before a count guard past the cap was named.
                    Err(
                        RefusalCause::GuardUnsatisfiable { .. }
                        | RefusalCause::CountUnwitnessed { .. },
                    ) => {}
                    Err(cause) => {
                        failed = true;
                        refusals.push(Refusal::about(&id, cause));
                    }
                }
            }
            if steps.is_empty() || failed {
                continue;
            }
            insert(
                suite,
                id,
                ConformanceScenario::new(
                    ScenarioPurpose::new(format!(
                        "`{}` refuses in held state `{state}` without effects",
                        command.name
                    ))
                    .expect("nonempty purpose"),
                    steps,
                    source,
                ),
                refusals,
            );
        }
    }
}

fn run_replay_in(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    replay: &ess_compiler::ir::ResolvedReplay,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Run, RefusalCause> {
    let ir = models.arrangement;
    let creator = &ir.commands()[&command.name];
    let original = creator.replay_origin(replay);
    let mut origin = run(ir, creator, original, actors)?;
    models.mark(caller::InvocationPhase::Arrange, &mut origin.setup);
    models.mark(caller::InvocationPhase::Arrange, &mut origin.invoke);
    let instance = origin.instance.clone().unwrap_or_else(|| {
        instance_name(&ir.entity(&replay.subject.entity).name, Distinction::PLAIN)
    });
    let (eligible_steps, eligible_source) =
        replay_eligibility(ir, command, outcome, &origin, &replay.subject, &instance)?;
    let capture = crate::replay::Observation::of(ir, command, outcome, instance.clone()).map_err(
        |reason| {
            RefusalCause::NoWitness(WitnessGap {
                path: format!("{}.response: {reason}", command.name),
                type_ref: "retained response".into(),
                reason: "exact retained-result observation cannot execute this response contract",
            })
        },
    )?;
    let mut setup = origin.steps();
    setup.push(ScenarioStep::ExpectNoError);
    for event in &original.emits {
        let event = EventRef::from(event);
        setup.push(ScenarioStep::ExpectEvent {
            payload: determined_payload(ir, original, &event, &origin.input, &BTreeMap::new()),
            shape: crate::response::event_shape(ir, &event, original),
            event,
        });
    }
    if let ResolvedInstance::Observed { event, field } = &replay.subject.instance {
        setup.push(ScenarioStep::CaptureInstance {
            instance: instance.clone(),
            entity: EntityRef::from(&replay.subject.entity),
            event: EventRef::from(event),
            field: field.name.clone(),
        });
    }
    setup.push(ScenarioStep::CaptureCommandResult {
        capture: capture.clone(),
    });
    setup.extend(eligible_steps);
    let preservation = subject_fact::preserve_complete_subject(
        ir,
        &replay.subject,
        &Setup {
            instance: Some(instance.clone()),
            after: origin.after.clone(),
            settled: origin.settled.clone(),
            ..Setup::none()
        },
    )?;
    setup.extend(preservation.before);
    let mut invoke = vec![
        ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: capture.replay.command.clone(),
            actor: origin.actor.clone(),
            input: origin.input.clone(),
        },
        ScenarioStep::ExpectOutcome {
            outcome: capture.replay.clone(),
        },
        ScenarioStep::ExpectReplayResult { capture },
    ];
    invoke.extend(preservation.after);
    models.mark(caller::InvocationPhase::Arrange, &mut setup);
    models.mark(caller::InvocationPhase::Act, &mut invoke);
    let mut source = origin.source;
    source.extend(eligible_source);
    source.extend(preservation.source);
    source.insert(
        OutcomeRef::new(CommandRef::new(command.name.clone()), original.name.clone()).into(),
    );
    source.insert(EntityRef::from(&replay.subject.entity).into());
    Ok(Run {
        after_steps: Vec::new(),
        setup,
        invoke,
        instance: Some(instance),
        before: origin.after.clone(),
        after: origin.after,
        actor: origin.actor,
        input: origin.input,
        source,
        before_settled: origin.settled.clone(),
        refused: Vec::new(),
        settled: origin.settled,
    })
}

fn replay_eligibility(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    origin: &Run,
    subject: &ResolvedSubject,
    instance: &InstanceName,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let facts = crate::input::replay_facts(ir, command, &origin.input)
        .map_err(RefusalCause::WitnessRejected)?;
    let held = origin
        .after
        .as_ref()
        .expect("replay origin creates or moves its subject");
    let mut observed = BTreeSet::new();
    // An accepting `when:` branch declared before an external one answers first (beyond10x/ess#217).
    let eligible = match &outcome.condition {
        ResolvedCondition::External { .. } => {
            !claimed_by(&facts, &earlier_accepting(command, outcome))
        }
        ResolvedCondition::ExternalWhen { predicate, .. } => {
            decides(&facts, &[predicate], true)?
                && !claimed_by(&facts, &earlier_accepting(command, outcome))
        }
        _ => {
            let mut selected = Vec::new();
            for branch in command
                .outcomes
                .iter()
                .filter(|branch| !state_default(branch))
            {
                if replay_condition(ir, command, branch, subject, origin, &facts, &mut observed)? {
                    selected.push(branch);
                }
            }
            if selected.is_empty() {
                selected.extend(
                    command
                        .outcomes
                        .iter()
                        .filter(|branch| state_default(branch)),
                );
            }
            selected.len() == 1 && selected[0].name == outcome.name
        }
    };
    if !eligible {
        return Err(RefusalCause::GuardUnsatisfiable {
            predicate: format!(
                "{} selected on the identical original input in post-origin state {held}",
                outcome.name
            ),
            tried: 1,
        });
    }
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    if observed.remove(EntitySpec::STATE) {
        let (observation, view) =
            observe_selection_subject(ir, subject, instance, held, outcome.name.as_str())?;
        steps.extend(observation);
        source.insert(view.into());
    }
    let arrangement = Arrangement {
        instance: instance.clone(),
        state: held.clone(),
        steps: Vec::new(),
        source: BTreeSet::new(),
        settled: origin.settled.clone(),
        unwritten: BTreeSet::new(),
    };
    for field in observed {
        let (observation, view) = subject_fact::observe(ir, &subject.entity, &field, &arrangement)?;
        steps.extend(observation);
        source.insert(view.into());
    }
    Ok((steps, source))
}

fn replay_condition(
    ir: &EssIr,
    command: &ResolvedCommand,
    branch: &ResolvedOutcome,
    subject: &ResolvedSubject,
    origin: &Run,
    facts: &crate::InputFacts<'_>,
    observed: &mut BTreeSet<String>,
) -> Result<bool, RefusalCause> {
    let held = origin
        .after
        .as_ref()
        .expect("replay origin has a post-state");
    let settled = &origin.settled;
    let predicate =
        match &branch.condition {
            ResolvedCondition::When { predicate } => Some(predicate),
            ResolvedCondition::SubjectState { predicate, .. }
            | ResolvedCondition::StateChange { predicate, .. } => {
                observed.insert(EntitySpec::STATE.into());
                if !admits_held_state(&branch.condition, held) {
                    return Ok(false);
                }
                predicate.as_ref()
            }
            ResolvedCondition::SubjectField {
                field,
                equals,
                predicate,
            } => {
                observed.insert(field.clone());
                let actual = settled
                    .get(field)
                    .and_then(|value| value.value.as_literal())
                    .and_then(Node::as_text)
                    .ok_or_else(|| {
                        RefusalCause::NoWitness(WitnessGap {
                        path: format!("{}.{}", subject.entity, field),
                        type_ref: "post-origin subject fact".into(),
                        reason:
                            "the original invocation did not establish the replay selection fact",
                    })
                    })?;
                if actual != equals {
                    return Ok(false);
                }
                predicate.as_ref()
            }
            ResolvedCondition::WrongState => {
                observed.insert(EntitySpec::STATE.into());
                return Ok(ir
                    .wrong_states(command)
                    .get(&subject.entity)
                    .is_some_and(|states| states.contains(held)));
            }
            ResolvedCondition::SubjectPredicate { predicate, input } => {
                observed.extend(subject_fact::read_by(ir, &subject.entity, predicate));
                match subject_fact::guard_truth_with(
                    ir,
                    &subject.entity,
                    settled,
                    &BTreeSet::new(),
                    Some(held),
                    predicate,
                    None,
                ) {
                    Truth::True => {}
                    Truth::False => return Ok(false),
                    Truth::Unknown => return Err(RefusalCause::NoWitness(WitnessGap {
                        path: format!("{}.{predicate}", subject.entity),
                        type_ref: "post-origin subject fact".into(),
                        reason:
                            "the original invocation did not establish the replay selection fact",
                    })),
                }
                input.as_ref()
            }
            // No related row was arranged for the replay: the original's own scenario witnessed it.
            ResolvedCondition::Related { .. } => return Err(related_guard::unarranged()),
            ResolvedCondition::Otherwise
            | ResolvedCondition::External { .. }
            | ResolvedCondition::ExternalWhen { .. }
            | ResolvedCondition::UnknownInstance
            | ResolvedCondition::InputAbsent
            | ResolvedCondition::ExistingInstance => return Ok(false),
        };
    decides(facts, &predicate.into_iter().collect::<Vec<_>>(), true)
}

/// What has to be true before a branch can be run, and what is true of its subject afterwards.
///
/// Empty for a branch that changes no entity. A branch that *creates* its own subject has no
/// instance to arrange — the new row is the scenario's own doing — but it may still have an
/// **owner** to arrange, because a row of an owned entity cannot exist without the row it belongs
/// to. See [`arrange_owner`].
struct Setup {
    /// The steps that bring the instance into the state the branch needs.
    steps: Vec<ScenarioStep>,
    /// What those steps bound the instance as, where they bound one.
    instance: Option<InstanceName>,
    /// Input fields of the branch under test that name something the arrangement created, by field.
    ///
    /// Separate from `instance`, which is the branch's *subject*. This is everything else the input
    /// has to point at — today exactly the field carrying the subject's owner — and it is a map
    /// because the question "which arranged row does this field name" is per field and the subject's
    /// answer is already spoken for.
    bound: BTreeMap<String, InstanceName>,
    /// The constructs the arrangement depends on, so a change to one makes a stored result stale.
    source: BTreeSet<EssSemanticRef>,
    /// The state the subject is in once the branch has been taken, where there is a subject.
    after: Option<StateName>,
    /// The state the arrangement left the subject in, before the branch runs, where it arranged
    /// one. For a `moves:` it is the `from` state the scenario exercises, and a transition with
    /// several is run from the others as well (beyond10x/ess#111).
    before: Option<StateName>,
    /// What the arrangement left in the subject's fields, where the branches it ran said.
    settled: BTreeMap<String, Determined>,
}

impl Setup {
    /// Nothing to arrange, and no entity to assert about afterwards.
    fn none() -> Self {
        Self {
            steps: Vec::new(),
            instance: None,
            bound: BTreeMap::new(),
            source: BTreeSet::new(),
            after: None,
            before: None,
            settled: BTreeMap::new(),
        }
    }
}

/// The instance one branch needs, brought to the state that branch may be taken from.
///
/// `creates:` needs nothing arranged and lands in the lifecycle's `initial`. `moves:` needs an
/// instance resting in one of the transition's `from` states, and takes the first that the
/// specification can actually reach — trying them in name order rather than picking one, so a
/// transition whose only reachable source is the second one still gets a scenario. `updates:` names
/// no state at all, so the cheapest reachable one is where the lifecycle starts.
fn prepare(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    held: Option<&StateName>,
) -> Result<Setup, RefusalCause> {
    prepare_in(ir, outcome, actors, held, Distinction::PLAIN)
}

/// [`prepare_subject`], with the rows a `{related: …}` source reads arranged ahead of it and the
/// subject or the input pointed at them (ess/16, beyond10x/ess#166, [`related::arrange`]). A branch
/// that reads no related row is arranged exactly as before.
fn prepare_in(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    held: Option<&StateName>,
    distinction: Distinction,
) -> Result<Setup, RefusalCause> {
    let setup = prepare_subject(ir, outcome, actors, held, distinction)?;
    related::arrange(ir, outcome, actors, distinction, setup)
}

/// [`prepare`], with the existing subject arranged under `distinction`: a further witness for every
/// act that arranges it, which [`arranged`] asks for where the plain arrangement already holds the
/// value the branch writes (beyond10x/ess#161).
fn prepare_subject(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    held: Option<&StateName>,
    distinction: Distinction,
) -> Result<Setup, RefusalCause> {
    let Some(subject) = &outcome.subject else {
        return Ok(Setup::none());
    };
    let lifecycle = &ir.entity(&subject.entity).lifecycle;
    // Where a creation lands: the state `into:` names (ess/15), or where the lifecycle starts.
    let born = subject.into.as_ref().unwrap_or(&lifecycle.initial);
    let (mut targets, need): (Vec<StateName>, InstanceNeed) = match &subject.effect {
        ResolvedEffect::Creates => {
            // The new row is the scenario's own doing and needs nothing arranged — except the row it
            // belongs to, where it belongs to one.
            let owner = arrange_owner(
                ir,
                outcome,
                &subject.entity,
                actors,
                Distinction::PLAIN,
                &[],
            );
            return Ok(match owner {
                None => Setup {
                    after: Some(born.clone()),
                    ..Setup::none()
                },
                Some((field, arrangement)) => Setup {
                    steps: arrangement.steps,
                    // Not `instance`: that names the branch's subject, and this arrangement made
                    // the subject's *owner*. The subject does not exist yet.
                    bound: [(field, arrangement.instance)].into_iter().collect(),
                    source: arrangement.source,
                    after: Some(born.clone()),
                    // Not the owner's either. `settled` is what the *subject's* fields hold, and
                    // the owner's fields are another row's.
                    settled: BTreeMap::new(),
                    instance: None,
                    before: None,
                },
            });
        }
        ResolvedEffect::Moves { transition } => (
            transition.from.iter().cloned().collect(),
            InstanceNeed::Moves {
                transition: transition.name.clone(),
            },
        ),
        ResolvedEffect::Updates | ResolvedEffect::Preserves | ResolvedEffect::Deletes => {
            (vec![lifecycle.initial.clone()], InstanceNeed::Updates)
        }
    };
    if let Some(state) = held {
        targets = vec![state.clone()];
    }
    // A removed row rests in no state (ess/15): nothing shows it afterwards.
    let after = match &subject.effect {
        ResolvedEffect::Moves { transition } => Some(transition.to.clone()),
        ResolvedEffect::Deletes => None,
        ResolvedEffect::Creates | ResolvedEffect::Updates | ResolvedEffect::Preserves => {
            Some(held.unwrap_or(&lifecycle.initial).clone())
        }
    };

    let arrangement = arrange_first(ir, &subject.entity, &targets, actors, distinction, &[])
        .map_err(|reason| RefusalCause::InstanceRequired {
            entity: EntityRef::from(&subject.entity),
            need,
            reason,
        })?;
    Ok(Setup {
        steps: arrangement.steps,
        instance: Some(arrangement.instance),
        // The branch under test acts on an instance that already exists, so its own input names the
        // subject and nothing else the arrangement built.
        bound: BTreeMap::new(),
        source: arrangement.source,
        after,
        before: Some(arrangement.state),
        settled: arrangement.settled,
    })
}

/// One instance of an entity, resting in a state, and the name the scenario calls it by.
#[derive(Debug, Clone)]
struct Arrangement {
    /// What it is called for the rest of the scenario.
    instance: InstanceName,
    /// The state the steps leave it in.
    ///
    /// Carried out rather than re-derived by the caller: which of several admissible states an
    /// arrangement reached decides which views hold a row for it, and answering that twice is two
    /// answers to one question.
    state: StateName,
    /// The steps that produce it.
    steps: Vec<ScenarioStep>,
    /// What those steps depend on.
    source: BTreeSet<EssSemanticRef>,
    /// What those steps left in the entity's fields, where the branches said.
    ///
    /// Accumulated in step order, so a move that sets a field the creation also set leaves the
    /// later value — which is what the implementation will hold, and so what a declared order
    /// ranks this row by.
    settled: BTreeMap<String, Determined>,
    /// The `Optional` fields no step of the arrangement wrote, known from its creation onward: the
    /// row holds nothing there, so a predicate asking whether one is present reads it as absent
    /// (beyond10x/ess#239). Empty wherever the arrangement's history is not known from its
    /// creation, which leaves such a predicate `Unknown`, as before.
    unwritten: BTreeSet<String>,
}

/// A wrong-state arrangement, the input sent to it, and identities arranged for that input.
type RefusalArrangement = (
    Arrangement,
    BTreeMap<String, Node>,
    BTreeMap<String, InstanceName>,
);

/// The `Optional` fields of `entity` the creating branch `creator` does not write, and nothing but
/// a later act on the row itself can: absent on the row as it leaves it (beyond10x/ess#239).
///
/// A stored field has five writers in the model, and each is either folded in by
/// [`Arrangement::absorb`] or kept out of the set here, where it reads undetermined as before:
///
/// | writer | here |
/// |---|---|
/// | the creator's own `sets:` | not unwritten |
/// | a later `updates:`/`moves:` `sets:` on this row, sent by the scenario | folded in by `absorb` |
/// | an `instances:` outcome's `sets:`, on every row its filter selects | [`written_elsewhere`] |
/// | an `affects:` entry's `sets:`, on every row its filter selects | [`written_elsewhere`] |
/// | the `sets:` of a command a binding invokes, which the target runs unasked | [`written_elsewhere`] |
///
/// The last three reach a row no step of its own arrangement names — a decoy's act, or the target
/// reacting to an event — so no arrangement knows whether one ran on it.
fn unwritten_by(ir: &EssIr, entity: &EntityHandle, creator: &ResolvedOutcome) -> BTreeSet<String> {
    let elsewhere = written_elsewhere(ir, entity);
    ir.entity(entity)
        .fields
        .iter()
        .filter(|field| field.type_ref.is_optional())
        .filter(|field| !creator.sets.iter().any(|set| set.target == field.name))
        .filter(|field| !elsewhere.contains(&field.name))
        .map(|field| field.name.clone())
        .collect()
}

/// The fields of `entity` some writer other than an act the arrangement sends to the row itself
/// can write: every `instances:` and `affects:` `sets:` on the entity, and every `sets:` of a
/// command a binding invokes that updates or moves a row of it. A binding-invoked `creates:` makes
/// a row of its own and writes no other.
fn written_elsewhere(ir: &EssIr, entity: &EntityHandle) -> BTreeSet<String> {
    let bound: BTreeSet<&QualifiedName> = ir
        .bindings()
        .values()
        .map(|binding| &ir.command(&binding.command).name)
        .collect();
    let mut written = BTreeSet::new();
    for command in ir.commands().values() {
        let invoked = bound.contains(&command.name);
        for outcome in &command.outcomes {
            if outcome
                .instances
                .as_ref()
                .is_some_and(|set| &set.entity == entity)
                || (invoked
                    && outcome.subject.as_ref().is_some_and(|subject| {
                        &subject.entity == entity
                            && !matches!(subject.effect, ResolvedEffect::Creates)
                    }))
            {
                written.extend(outcome.sets.iter().map(|set| set.target.clone()));
            }
            for affect in outcome.affects.iter().filter(|a| &a.entity == entity) {
                written.extend(affect.sets.iter().map(|set| set.target.clone()));
            }
        }
    }
    written
}

/// `unwritten` less every field `outcome`'s `sets:` writes: what stays unwritten after it runs.
fn still_unwritten(unwritten: &BTreeSet<String>, outcome: &ResolvedOutcome) -> BTreeSet<String> {
    unwritten
        .iter()
        .filter(|field| !outcome.sets.iter().any(|set| &set.target == *field))
        .cloned()
        .collect()
}

impl Arrangement {
    /// Folds one further act on this row, `outcome`, with what it determined: [`absorb`], and every
    /// field it writes no longer unwritten.
    fn absorb(&mut self, outcome: &ResolvedOutcome, determined: BTreeMap<String, Determined>) {
        absorb(&mut self.settled, outcome, determined);
        self.unwritten = still_unwritten(&self.unwritten, outcome);
    }

    /// The identity this instance is known by for the rest of the scenario.
    fn identity(&self) -> ScenarioValue {
        ScenarioValue::instance(self.instance.clone())
    }

    /// Whether `view` holds this row for a read sending `params` ([`shows_row`]).
    fn shows(
        &self,
        ir: &EssIr,
        view: &ResolvedView,
        params: &BTreeMap<String, ScenarioValue>,
    ) -> Result<bool, Vec<FactPath>> {
        shows_row(
            ir,
            view,
            &self.state,
            &self.settled,
            Some(&self.identity()),
            params,
        )
    }
}

/// The cheapest of several candidate states the specification can actually reach.
///
/// `cancel` may start in `Placed` or in `Held`, and both are legal sources for the same scenario —
/// so the one taken is the one that needs the fewest commands to set up. Every extra command in an
/// arrangement is another way for a scenario to fail for a reason that has nothing to do with what
/// it is testing. Ties go to the lower-named state, and the states arrive in name order, so the
/// choice is a function of the model (§37).
///
/// The error kept is the first one, in name order, so a refusal does not move when an unrelated
/// state is added to the transition's `from` set.
fn arrange_first(
    ir: &EssIr,
    entity: &EntityHandle,
    targets: &[StateName],
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
) -> Result<Arrangement, Unreachable> {
    let mut cheapest: Option<Arrangement> = None;
    let mut first: Option<Unreachable> = None;
    for target in targets {
        match arrange(ir, entity, target, actors, distinction, arranging) {
            Ok(arrangement) => {
                if cheapest
                    .as_ref()
                    .is_none_or(|held| arrangement.steps.len() < held.steps.len())
                {
                    cheapest = Some(arrangement);
                }
            }
            Err(reason) => {
                first.get_or_insert(reason);
            }
        }
    }
    cheapest.ok_or(first.unwrap_or(Unreachable::NothingCreates))
}

/// Bring one instance of `entity` into existence and drive it to `target`.
///
/// Two questions the model can now answer and could not before G21: which outcome brings an instance
/// into existence, and — the one this gate closed — **which field carries its identity**, so the
/// steps that follow can name the instance the first step created rather than inventing one.
///
/// The route is the shortest sequence of declared, driven transitions from the lifecycle's `initial`
/// to `target`. Shortest because a scenario is a fixture, not a tour: every extra command is another
/// way for the arrangement to fail for a reason that has nothing to do with what is being tested.
///
/// `arranging` is the chain of entities this one is being arranged *for* — empty at the top, and one
/// longer at each hop into an owner. It is what stops a specification in which two entities own each
/// other from arranging forever; see [`arrange_owner`].
fn arrange(
    ir: &EssIr,
    entity: &EntityHandle,
    target: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
) -> Result<Arrangement, Unreachable> {
    let all = ir.drivers();
    let drivers: &[Driver<'_>] = all.get(entity).map_or(&[], Vec::as_slice);
    // Every creating branch is a place to start (ess/15, `into:`): the route is the shortest from
    // the state some creation lands in, ties to the first creator in the order `EssIr::drivers`
    // yields them: command name, then declared branch order, since the IR keeps commands by name.
    // A creation that cannot be arranged, or whose route cannot be driven, gives way to the next
    // in that order (beyond10x/ess#198), and the cause kept is the first one's.
    let mut creators = drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .peekable();
    if creators.peek().is_none() {
        return Err(Unreachable::NothingCreates);
    }
    let mut routed: Vec<(&Driver<'_>, Vec<Driver<'_>>)> = creators
        .filter_map(|creator| {
            // From where the new row rests once the bindings its creation sets off have run
            // (beyond10x/ess#266), which `created` observes before the route goes on.
            let start = binding_effects::resting(ir, creator.outcome, born(ir, creator))?;
            route_from(ir, entity, drivers, &start, target).map(|path| (creator, path))
        })
        .collect();
    if routed.is_empty() {
        if let Some(away) = binding_effects::bound_away(ir, drivers, target) {
            return Err(away);
        }
    }
    // Stable: equal lengths keep the drivers' name order.
    routed.sort_by_key(|(_, path)| path.len());
    let mut first: Option<Unreachable> = None;
    for (creator, route) in routed {
        let arranged = created(ir, entity, creator, actors, distinction, arranging, None).and_then(
            |arrangement| {
                advance(
                    ir,
                    entity,
                    arrangement,
                    route,
                    target,
                    actors,
                    distinction,
                    arranging,
                )
            },
        );
        match arranged {
            Ok(arrangement) => return Ok(arrangement),
            Err(reason) => {
                first.get_or_insert(reason);
            }
        }
    }
    Err(first.unwrap_or_else(|| Unreachable::NoPath {
        from: ir.entity(entity).lifecycle.initial.clone(),
    }))
}

/// The second half of [`arrange`]: an instance already created, driven along `route` to `target`.
///
/// Split out so an arrangement that chose the creating command's input itself — an aggregate row
/// whose fields are the page's pattern — reaches its state exactly as every other one does.
#[allow(clippy::too_many_arguments)]
fn advance(
    ir: &EssIr,
    entity: &EntityHandle,
    mut arrangement: Arrangement,
    route: Vec<Driver<'_>>,
    target: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
) -> Result<Arrangement, Unreachable> {
    for driver in route {
        // A move whose command reads the row's stored fields is taken only where the row the
        // arrangement built selects it, so that is checked rather than assumed. Where the plain
        // witness does not, the route is searched again for a row that does — the same search a
        // branch guarded by the stored fields is arranged with.
        // A move whose command reads a related row through a stored field of this row (ess/22,
        // beyond10x/ess#304) is sent with that reference left out, or naming a row arranged for it.
        let moved = if related_guard::stored::field(driver.command).is_some() {
            related_guard::stored::step(ir, &driver, &arrangement, actors, distinction, arranging)
                .map_err(|cause| Unreachable::StoredReference {
                outcome: Box::new(OutcomeRef::new(
                    CommandRef::new(driver.command.name.clone()),
                    driver.outcome.name.clone(),
                )),
                why: match cause {
                    RefusalCause::GuardUnsatisfiable { predicate, .. } => predicate.into(),
                    other => other.to_string().into(),
                },
            })?
        } else if subject_fact::uses(driver.command) {
            match subject_fact::step(ir, entity, &driver, &arrangement, actors) {
                Some(next) => next,
                None if arranging.is_empty() => {
                    return subject_fact::reach_state(ir, entity, target, actors, distinction)
                        .map_err(|_| Unreachable::Unwitnessable {
                            outcome: OutcomeRef::new(
                                CommandRef::new(driver.command.name.clone()),
                                driver.outcome.name.clone(),
                            ),
                        });
                }
                None => {
                    return Err(Unreachable::Unwitnessable {
                        outcome: OutcomeRef::new(
                            CommandRef::new(driver.command.name.clone()),
                            driver.outcome.name.clone(),
                        ),
                    })
                }
            }
        } else {
            let invoked = invoke(
                ir,
                &driver,
                Some(&arrangement.instance),
                Some(&arrangement.state),
                actors,
                distinction,
                // A move acts on a row that already exists, so its input names that row and
                // nothing else: whatever owner it needed was arranged before the row was created.
                &BTreeMap::new(),
                &arranging
                    .iter()
                    .copied()
                    .chain([entity])
                    .collect::<Vec<_>>(),
            )?;
            let mut next = arrangement.clone();
            next.steps.extend(invoked.steps);
            next.source.extend(invoked.source);
            next.absorb(driver.outcome, invoked.settled);
            if let Some(transition) = driver.effect.transition() {
                next.state = transition.to.clone();
            }
            next
        };
        arrangement = moved;
        // Observed where the bindings this move set off on the row leave it, before the next step
        // (beyond10x/ess#266).
        binding_effects::settle_moved(ir, entity, driver.outcome, &mut arrangement)?;
    }
    arrangement.state = target.clone();
    Ok(arrangement)
}

/// One new instance of `entity`, resting where its lifecycle starts, and the name it is bound as.
///
/// The first half of [`arrange`], and the start of every search for a row with particular stored
/// values: `input` is the creating command's input where the caller chose one toward a goal, and
/// `None` where the plain witness the creating branch reaches will do.
fn created(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
    input: Option<&BTreeMap<String, Node>>,
) -> Result<Arrangement, Unreachable> {
    // Its owner first, and everything that owns *that*. An owned row cannot exist without the row it
    // belongs to, so the arrangement that brings one into being is incomplete without it.
    // Not where the owner is the row a related guard reads through the same field (ess/18): that
    // row is arranged once, by the related-row search, as the one the guard needs.
    let related_owner = related_guard::routes(creator.command, creator.outcome)
        && related_guard::owner_is_related(ir, creator.command, creator.outcome);
    let owner = if related_owner {
        None
    } else {
        arrange_owner(ir, creator.outcome, entity, actors, distinction, arranging)
    };
    created_owned(
        ir,
        entity,
        creator,
        actors,
        distinction,
        arranging,
        owner.as_ref(),
        input,
    )
}

/// An owner already in the scenario, as [`created_owned`] takes it: the input field `creator`'s
/// `sets:` names the owner through, and no steps. `None` where `entity` has no owner or `creator`
/// does not name it from its input.
fn under_owner(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    owner: &InstanceName,
) -> Option<(String, Arrangement)> {
    // An owner that holds at most one row is never given a second.
    if !shares_owner(ir, entity) {
        return None;
    }
    filed_under(ir, entity, creator, owner)
}

/// [`under_owner`] without its `cardinality: many` test: for an owner the caller knows holds no row
/// of `entity` yet ([`holds_none`]), which a `cardinality: one` relation admits one of
/// (beyond10x/ess#271).
fn filed_under(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    owner: &InstanceName,
) -> Option<(String, Arrangement)> {
    let belongs = ir.owner_of(entity)?;
    let field = creator
        .outcome
        .sets
        .iter()
        .find_map(|set| match &set.value {
            ResolvedPayloadValue::InputField { field, .. }
                if set.target == belongs.via && set.conversion.is_none() =>
            {
                Some(field.clone())
            }
            _ => None,
        })?;
    Some((
        field,
        Arrangement {
            instance: owner.clone(),
            state: ir.entity(&belongs.owner).lifecycle.initial.clone(),
            steps: Vec::new(),
            source: BTreeSet::new(),
            settled: BTreeMap::new(),
            unwritten: BTreeSet::new(),
        },
    ))
}

/// Whether `steps` bring `owner` into being and file no row of `entity` under it: each step
/// sending a command that creates `entity` is read at the input its `sets:` names the owner
/// through (beyond10x/ess#271). An owner the steps did not create may hold rows they do not show,
/// so it is not known to hold none.
fn holds_none(
    ir: &EssIr,
    entity: &EntityHandle,
    steps: &[ScenarioStep],
    owner: &InstanceName,
) -> bool {
    let Some(belongs) = ir.owner_of(entity) else {
        return false;
    };
    let all = ir.drivers();
    let creators: Vec<(String, String)> = all
        .get(entity)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .filter_map(|driver| {
            driver.outcome.sets.iter().find_map(|set| match &set.value {
                ResolvedPayloadValue::InputField { field, .. } if set.target == belongs.via => {
                    Some((driver.command.name.to_string(), field.clone()))
                }
                _ => None,
            })
        })
        .collect();
    let created = steps.iter().any(
        |step| matches!(step, ScenarioStep::CaptureInstance { instance, .. } if instance == owner),
    );
    created
        && !steps.iter().any(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                creators.iter().any(|(name, field)| {
                    command.to_string() == *name
                        && matches!(input.get(field), Some(ScenarioValue::Instance { instance })
                            if instance == owner)
                })
            }
            _ => false,
        })
}

/// Whether an owner of `entity` may hold several of its rows: the owning relation is
/// `cardinality: many`.
fn shares_owner(ir: &EssIr, entity: &EntityHandle) -> bool {
    ir.owner_of(entity)
        .is_some_and(|owned| owned.relation.cardinality == Cardinality::Many)
}

/// The owner a row was created under, where its link to the owner holds an arranged instance, and
/// the field that link is.
fn owner_of_row<'a>(
    ir: &'a EssIr,
    entity: &EntityHandle,
    settled: &'a BTreeMap<String, Determined>,
) -> Option<(&'a str, &'a InstanceName)> {
    let via = ir.owner_of(entity)?.via;
    match &settled.get(via)?.value {
        ScenarioValue::Instance { instance } => Some((via, instance)),
        _ => None,
    }
}

/// [`created`], under an owner the caller arranged: the input field that names it, and the steps
/// that bring it into being — empty where an earlier row of the same scenario already did, so rows
/// that share an owner share one (beyond10x/ess#193, an aggregate grouped by its link field).
// One argument per thing an arranging run is told, and the witness it is for.
#[allow(clippy::too_many_arguments)]
fn created_owned(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
    owner: Option<&(String, Arrangement)>,
    input: Option<&BTreeMap<String, Node>>,
) -> Result<Arrangement, Unreachable> {
    created_by(ir, entity, creator, distinction, owner, |bound, steps| {
        // Mid-arrangement of `entity`: a creator that needs a related row of an entity already
        // being arranged stops there (ess/18).
        let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([entity]).collect();
        if related_guard::routes(creator.command, creator.outcome) {
            related_guard::drive(
                ir,
                creator,
                None,
                actors,
                distinction,
                (bound, steps),
                input,
                &chain,
            )
            .map_err(|_| Unreachable::Unwitnessable {
                outcome: OutcomeRef::new(
                    CommandRef::new(creator.command.name.clone()),
                    creator.outcome.name.clone(),
                ),
            })
        } else {
            Ok(match input {
                Some(input) => {
                    invoke_created_with(ir, creator, actors, distinction, bound, input, &chain)?
                }
                None => invoke(ir, creator, None, None, actors, distinction, bound, &chain)?,
            })
        }
    })
}

/// [`created_owned`] with the creating run left to `run`, which is told what the owner binds and
/// the steps that arranged it: one new instance of `entity`, captured from the event the creating
/// branch emits. An aggregate view's rows are created through it on a related row the scenario
/// arranged itself (beyond10x/ess#272).
fn created_by<E>(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    distinction: Distinction,
    owner: Option<&(String, Arrangement)>,
    run: impl FnOnce(&BTreeMap<String, InstanceName>, &[ScenarioStep]) -> Result<Invocation, E>,
) -> Result<Arrangement, E> {
    let instance = instance_name(&ir.entity(entity).name, distinction);
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();

    let bound: BTreeMap<String, InstanceName> = match owner {
        None => BTreeMap::new(),
        Some((field, arrangement)) => {
            steps.extend(arrangement.steps.iter().cloned());
            source.extend(arrangement.source.iter().cloned());
            [(field.clone(), arrangement.instance.clone())]
                .into_iter()
                .collect()
        }
    };

    let mut settled = BTreeMap::new();
    let created = run(&bound, &steps)?;
    steps.extend(created.steps);
    source.extend(created.source);
    absorb(&mut settled, creator.outcome, created.settled);
    // Where the identity becomes knowable. `creates:` names a field of an event the branch emits,
    // because the caller could not have named an instance that did not exist when it called — and
    // because §9's command result already carries the events a command emitted, so binding it here
    // asks a target for nothing it was not already going to report.
    let ResolvedInstance::Observed { event, field } = &subject(creator).instance else {
        // `Subject::surface` makes this a function of the verb, and the verb here is `creates`.
        unreachable!("a `creates:` link is observed in an event")
    };
    steps.push(ScenarioStep::CaptureInstance {
        instance: instance.clone(),
        entity: EntityRef::from(entity),
        event: EventRef::from(event),
        field: field.name.clone(),
    });
    source.insert(EventRef::from(event).into());

    let mut arrangement = Arrangement {
        instance,
        state: born(ir, creator).clone(),
        steps,
        source,
        settled,
        unwritten: unwritten_by(ir, entity, creator.outcome),
    };
    // Where the bindings the creation sets off on the new row leave it, observed (beyond10x/ess#266).
    binding_effects::settle_created(ir, entity, creator.outcome, &mut arrangement);
    Ok(arrangement)
}

/// The row an owned row belongs to, and the input field that points the creating command at it.
///
/// **Why a `creates:` can need an arrangement at all.** Until this existed, a created subject was
/// taken to need nothing arranged — the new row is the scenario's own doing — and for a root that is
/// exactly right. For an entity another one declares it `owns`, it is not: `owns` says the far side
/// does not stand on its own, so a suite that creates one without its owner arranges a world the
/// specification says cannot exist. Measured on an adopter's model: every scenario that reached a
/// draft began by saving one against a fabricated agent id, the implementation refused because the
/// agent was not there, and fifteen scenarios reported `unsupported` — no information about the
/// implementation, from a suite that was asking an impossible question.
///
/// **The link is `sets:`, and nothing else.** A relation names the *entity field* that carries the
/// ownership; what this needs is the *command input* that fills it, and the one place the model says
/// which input fills which field is the creating branch's `sets:`. Matching on a shared spelling, or
/// on an input that happens to be typed as the owner's identity, is the invention this repository
/// refuses everywhere else — and a wrong guess here does not fail loudly, it points a scenario at
/// somebody else's row.
///
/// So a model closes this gap by declaring the link it already relies on:
///
/// ```yaml
/// sets:
///   account_id: input.account_id   # the field `owns` is carried by
/// ```
///
/// **Where it answers `None`, the arrangement is what it was before.** Four ways that happens, and
/// none of them is a refusal:
///
/// | | |
/// |---|---|
/// | nothing owns this entity | it is a root, which is not an error (entity-relations design §3) |
/// | `sets:` does not determine the carrying field | the model has not said which input names the owner, and this will not guess |
/// | nothing creates the owner | `examples/billing/` is this case — `Account` is declared, owns `Invoice`, and no command brings one into existence. An arrangement cannot create what the specification never says how to create |
/// | the owner is already being arranged | two entities owning each other, which `validate_relations` does not refuse because neither declaration is wrong alone |
///
/// A refusal was the alternative and is the wrong answer: it would delete every scenario that
/// creates a billing invoice, and those scenarios pass. What is lost by falling back is visible in
/// the suite itself — the arrangement simply has no command creating the owner in it.
fn arrange_owner(
    ir: &EssIr,
    creating: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
) -> Option<(String, Arrangement)> {
    let belongs = ir.owner_of(entity)?;
    // The chain, not a depth count: what makes an owner unarrangeable is that arranging it is
    // already in progress, and a number would have to be right about how deep is deep enough.
    if arranging.contains(&&belongs.owner) || belongs.owner == *entity {
        return None;
    }
    let field = creating.sets.iter().find_map(|set| {
        // A conversion says two types may meet and not what it computes, so what the input holds is
        // not what the field ends up holding — and an owner named through one would be a row nobody
        // can show is the row that was arranged.
        match (
            &set.value,
            set.target == belongs.via,
            set.conversion.is_some(),
        ) {
            (ResolvedPayloadValue::InputField { field, .. }, true, false) => Some(field.clone()),
            _ => None,
        }
    })?;

    let owner = &belongs.owner;
    let initial = ir.entity(owner).lifecycle.initial.clone();
    // Where the lifecycle starts, and no further. The relation says the owner must exist; it says
    // nothing about what state it must be in, and driving it somewhere else would be this function
    // inventing a requirement the model does not have.
    let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([entity]).collect();
    let arrangement = arrange_first(ir, owner, &[initial], actors, distinction, &chain).ok()?;
    Some((field, arrangement))
}

/// One command run as part of an arrangement: reach its branch, and require that it was taken.
///
/// The outcome is asserted rather than assumed, because an arrangement that quietly failed produces
/// a scenario that proves nothing and says it passed — which is the shape of green this whole
/// milestone exists to rule out.
// One argument per thing an arranging run is told: the arranging chain joined the six (ess/18).
#[allow(clippy::too_many_arguments)]
fn invoke(
    ir: &EssIr,
    driver: &Driver<'_>,
    instance: Option<&InstanceName>,
    held: Option<&StateName>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    bound: &BTreeMap<String, InstanceName>,
    arranging: &[&EntityHandle],
) -> Result<Invocation, Unreachable> {
    let command_ref = CommandRef::new(driver.command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), driver.outcome.name.clone());
    // A branch of a command guarded by a related row (ess/18, #211) is run with the row it needs
    // arranged first, as its own scenario runs it.
    if related_guard::routes(driver.command, driver.outcome) {
        return related_guard::drive(
            ir,
            driver,
            instance,
            actors,
            distinction,
            (bound, &[]),
            None,
            arranging,
        )
        .map_err(|_| Unreachable::Unwitnessable {
            outcome: outcome_ref,
        });
    }
    // The cause is not carried up. That branch has a refusal of its own, under its own id, saying
    // exactly why no input reaches it; repeating it here would be one defect reported twice with two
    // repairs to weigh.
    let input = if has_subject_guards(driver.command)
        && !existence::creates_unknown(driver.outcome)
        && driver.outcome.test_strategy != TestStrategy::InjectFault
    {
        held.ok_or(RefusalCause::StrategyWithoutGuard {
            strategy: driver.outcome.test_strategy,
        })
        .and_then(|state| reach_in_state(ir, driver.command, driver.outcome, state, distinction))
    } else {
        reach(ir, driver.command, driver.outcome, distinction)
    }
    .map_err(|_| Unreachable::Unwitnessable {
        outcome: outcome_ref.clone(),
    })?;
    if matches!(driver.effect, ResolvedEffect::Creates) {
        invoke_created_with(ir, driver, actors, distinction, bound, &input, arranging)
    } else {
        Ok(invoke_with(ir, driver, instance, actors, bound, &input))
    }
}

/// A creation used to arrange a later subject has the same related sources as its own outcome
/// scenario. Preserve their captured identities and typed values, with the caller's cycle guard.
fn invoke_created_with(
    ir: &EssIr,
    driver: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    bound: &BTreeMap<String, InstanceName>,
    input: &BTreeMap<String, Node>,
    arranging: &[&EntityHandle],
) -> Result<Invocation, Unreachable> {
    // A payload-only source keeps its existing outer arrangement. This seam is for stored
    // copies needed by a later subject/view; arranging payload-only reads here would duplicate
    // the sources that the later branch already arranges (#166).
    if !driver
        .outcome
        .sets
        .iter()
        .any(|set| matches!(set.value, ResolvedPayloadValue::RelatedField { .. }))
    {
        return Ok(invoke_with(ir, driver, None, actors, bound, input));
    }
    let mut setup = Setup::none();
    setup.bound.clone_from(bound);
    let setup = related::arrange_within(
        ir,
        driver.outcome,
        actors,
        distinction,
        setup,
        None,
        arranging,
    )
    .map_err(|_| Unreachable::Unwitnessable {
        outcome: OutcomeRef::new(
            CommandRef::new(driver.command.name.clone()),
            driver.outcome.name.clone(),
        ),
    })?;
    let mut invoked = invoke_with(ir, driver, None, actors, &setup.bound, input);
    let supplied = supply(
        driver.command,
        input,
        driver.outcome.subject.as_ref(),
        None,
        &setup.bound,
    );
    invoked.settled = settled(ir, driver.outcome, &supplied, &setup.settled);
    let mut steps = setup.steps;
    steps.append(&mut invoked.steps);
    invoked.steps = steps;
    invoked.source.extend(setup.source);
    Ok(invoked)
}

/// One arranging invocation with an input already chosen: its steps, and what its `sets:` leave.
///
/// The half of [`invoke`] that does not choose. Split out so an arrangement can choose the input
/// to a goal — a creating command sent the weight a stored-field guard compares against — and
/// still run it exactly as every other arranging command is run.
fn invoke_with(
    ir: &EssIr,
    driver: &Driver<'_>,
    instance: Option<&InstanceName>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    bound: &BTreeMap<String, InstanceName>,
    input: &BTreeMap<String, Node>,
) -> Invocation {
    let command_ref = CommandRef::new(driver.command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), driver.outcome.name.clone());
    let mut steps = Vec::new();
    if driver.outcome.test_strategy == TestStrategy::InjectFault {
        steps.push(ScenarioStep::ConfigureExternalOutcome {
            force: outcome_ref.clone(),
            times: None,
        });
    }
    let supplied = supply(
        driver.command,
        input,
        driver.outcome.subject.as_ref(),
        instance,
        bound,
    );
    // An arranging act reads no row it can name here; an ess/14 source then determines nothing.
    let settled = settled(ir, driver.outcome, &supplied, &BTreeMap::new());
    steps.push(ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref.clone(),
        actor: actors.get(&driver.command.name).cloned(),
        input: supplied,
    });
    steps.push(ScenarioStep::ExpectOutcome {
        outcome: outcome_ref.clone(),
    });

    let source: BTreeSet<EssSemanticRef> = [command_ref.into(), outcome_ref.into()]
        .into_iter()
        .collect();
    Invocation {
        steps,
        source,
        settled,
    }
}

/// One invoked branch: the steps that run it, what they depend on, and what they left behind.
struct Invocation {
    /// The steps, in order.
    steps: Vec<ScenarioStep>,
    /// The constructs they depend on.
    source: BTreeSet<EssSemanticRef>,
    /// What the branch's `sets:` determined, read against the input this invocation supplied.
    settled: BTreeMap<String, Determined>,
}

/// The shortest sequence of driven transitions from where the lifecycle starts to `target`.
///
/// Breadth-first over the states, with the edges out of each state visited in a fixed order —
/// transition name, then command, then branch — so the route is a function of the model and not of
/// how a map happened to iterate (§37).
///
/// Every caller in this tree now routes from a creation state with [`route_from`]; this stays for the
/// callers the aggregates unit merges in, which route from `initial`.
#[allow(dead_code)]
fn route<'a>(
    ir: &EssIr,
    entity: &EntityHandle,
    drivers: &[Driver<'a>],
    target: &StateName,
) -> Option<Vec<Driver<'a>>> {
    route_from(
        ir,
        entity,
        drivers,
        &ir.entity(entity).lifecycle.initial,
        target,
    )
}

/// The state a creating driver leaves its new row in: `into:` (ess/15), or the lifecycle's initial.
fn born<'a>(ir: &'a EssIr, creator: &Driver<'a>) -> &'a StateName {
    creator
        .outcome
        .subject
        .as_ref()
        .and_then(|subject| subject.into.as_ref())
        .unwrap_or_else(|| &ir.entity(&subject(creator).entity).lifecycle.initial)
}

/// [`route`], from `start` rather than from where the lifecycle starts.
fn route_from<'a>(
    ir: &EssIr,
    entity: &EntityHandle,
    drivers: &[Driver<'a>],
    start: &StateName,
    target: &StateName,
) -> Option<Vec<Driver<'a>>> {
    let _ = ir.entity(entity);
    if start == target {
        return Some(Vec::new());
    }

    let mut edges: BTreeMap<StateName, Vec<(StateName, Driver<'a>)>> = BTreeMap::new();
    for driver in drivers {
        let Some(transition) = driver.effect.transition() else {
            continue;
        };
        for from in &transition.from {
            // An edge exists only where the branch that drives it admits the state it leaves. Both
            // state-reading conditions narrow the `from` set and neither may be ignored: routing
            // through a state the driver refuses produces a path whose own step cannot be reached,
            // and the refusal then names the wrong construct.
            if !admits_held_state(&driver.outcome.condition, from) {
                continue;
            }
            // Where the row rests once the bindings this move sets off on it have run
            // (beyond10x/ess#266): a state a binding moves the row out of is no place to stop, and
            // a move whose rest cannot be named is no edge at all.
            let Some(to) = binding_effects::resting(ir, driver.outcome, &transition.to) else {
                continue;
            };
            edges.entry(from.clone()).or_default().push((to, *driver));
        }
    }
    for outgoing in edges.values_mut() {
        outgoing.sort_by_key(|(to, driver)| {
            (
                driver
                    .effect
                    .transition()
                    .map(|transition| transition.name.clone())
                    .unwrap_or_default(),
                driver.command.name.to_string(),
                driver.outcome.name.to_string(),
                to.to_string(),
            )
        });
    }

    let mut came: BTreeMap<StateName, (StateName, Driver<'a>)> = BTreeMap::new();
    let mut seen: BTreeSet<StateName> = [start.clone()].into();
    let mut queue: VecDeque<StateName> = [start.clone()].into();
    while let Some(state) = queue.pop_front() {
        for (to, driver) in edges.get(&state).map(Vec::as_slice).unwrap_or_default() {
            if !seen.insert(to.clone()) {
                continue;
            }
            came.insert(to.clone(), (state.clone(), *driver));
            if to == target {
                let mut route = Vec::new();
                let mut at = target.clone();
                while let Some((previous, driver)) = came.get(&at) {
                    route.push(*driver);
                    at = previous.clone();
                }
                route.reverse();
                return Some(route);
            }
            queue.push_back(to.clone());
        }
    }
    None
}

/// The subject of a driver's outcome, which a driver always has.
fn subject<'a>(driver: &Driver<'a>) -> &'a ResolvedSubject {
    driver.outcome.subject.as_ref().unwrap_or_else(|| {
        panic!(
            "a driver is an outcome with a subject: {}",
            driver.outcome.name
        )
    })
}

/// What a scenario calls one instance of this entity: its local name, in lower-kebab.
///
/// Derived from the model rather than counted, so two scenarios about one entity use one word and a
/// reader of a suite recognises it. A further instance takes the same word and its number —
/// `invoice`, then `invoice-2` — because a second row arranged so a declared order has a pair to
/// compare is the same kind of thing as the first, and a reader should not have to look up which is
/// which.
fn instance_name(entity: &QualifiedName, distinction: Distinction) -> InstanceName {
    let local = entity.local();
    let mut out = String::with_capacity(local.len() + 4);
    for (index, character) in local.char_indices() {
        if character == '_' || character == '-' {
            out.push('-');
        } else if character.is_ascii_uppercase() {
            if index > 0 && !out.ends_with('-') {
                out.push('-');
            }
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(character);
        }
    }
    // The number goes on last, and on the fallback too: two instances that shared a name would be
    // one instance the runner bound twice, which is a suite that arranges two rows and asserts
    // against one.
    let suffix = match distinction {
        Distinction::PLAIN => String::new(),
        further => format!("-{}", further.get() + 1),
    };
    InstanceName::new(format!("{out}{suffix}"))
        .or_else(|_| InstanceName::new(format!("subject{suffix}")))
        .expect("`subject` and its number are lower-kebab")
}

/// A witness input, with every field that names an arranged row replaced by the row itself.
///
/// The one place a scenario carries a reference rather than a value. Every other field holds what
/// synthesis decided against the branch's guard; these hold "the instance step one created", because
/// no generator can know an identity a target has not assigned yet.
///
/// Two fields can be one: the branch's **subject**, which `instance:` names, and the field carrying
/// the subject's **owner**, which `bound` carries from [`arrange_owner`]. They never collide — a
/// `creates:` names its subject in an *event* and so has no subject field in the input at all, and
/// an owner is arranged for a `creates:` only.
///
/// A guard may not read either field — `ess-domain` refuses that under `unobservable_fact`, since
/// invariant 13 makes an identity opaque — so replacing them cannot invalidate the decision that
/// chose the rest of the input.
fn supply(
    command: &ResolvedCommand,
    input: &BTreeMap<String, Node>,
    subject: Option<&ResolvedSubject>,
    instance: Option<&InstanceName>,
    bound: &BTreeMap<String, InstanceName>,
) -> BTreeMap<String, ScenarioValue> {
    let named = subject.and_then(|subject| match &subject.instance {
        ResolvedInstance::Supplied { field } => Some(field.name.as_str()),
        ResolvedInstance::Observed { .. } => None,
    });
    input
        .iter()
        .map(|(field, value)| {
            let supplied = match (named, instance) {
                (Some(named), Some(subject)) if named == field => {
                    ScenarioValue::instance(subject.clone())
                }
                _ => match bound.get(field) {
                    Some(owner) => ScenarioValue::instance(owner.clone()),
                    None => command.fixture_inputs.get(field).map_or_else(
                        || crate::now_offset::sent(command, field, value),
                        |fixture| ScenarioValue::Fixture {
                            fixture: fixture.clone(),
                        },
                    ),
                },
            };
            (field.clone(), supplied)
        })
        .collect()
}

/// `true` when any branch of this command is chosen by what the subject already holds.
///
/// Both state-reading conditions count. A command carrying only
/// [`StateChange`](ResolvedCondition::StateChange) branches still cannot be reached by constructing
/// an input alone — the state has to be established first — so answering `false` for one sent it
/// down the stateless path, where the held state is never arranged and the branch is selected
/// against a subject resting wherever the creating act left it.
fn has_subject_guards(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectState { .. } | ResolvedCondition::StateChange { .. }
        )
    })
}

/// The held states one branch's condition admits, or `None` where it reads no state at all.
///
/// Read from the IR and never re-derived. `ess-compiler` computes
/// [`StateChange`](ResolvedCondition::StateChange)`::states` as "the move's `from` set, partitioned
/// by whether each state is the one it arrives at", and says in the same breath why it is carried
/// rather than left to the caller: *a consumer arranging a scenario needs the held states this
/// branch admits, and re-deriving them means re-implementing the partition rule beside every
/// generator that asks* (`ess-compiler::ir::ResolvedCondition::StateChange`). This crate is that
/// consumer, so it reads the answer instead of computing a second one that can disagree.
///
/// `None` is not "every state". It means the condition says nothing about the held state, so the
/// caller's own rule applies — which for [`route`] is every `from` the transition declares.
///
/// [`prepare_state_input`] deliberately does NOT consult this. It offers the lifecycle's states
/// filtered by the transition's `from` set and lets [`reach_in_state`] reject the ones the branch
/// refuses, which it does, so narrowing here changed no scenario this crate synthesises — measured
/// 2026-09-16 by reverting the narrowing with every other arm in place. It would change one thing:
/// which state is picked for a branch admitting more than one, and that is a policy choice about
/// arrangement that belongs with whoever owns `when_state_changes:`, not a consequence of reading
/// their partition. The single place the partition decides anything is [`admits_held_state`].
fn admitted_states(condition: &ResolvedCondition) -> Option<BTreeSet<&StateName>> {
    match condition {
        ResolvedCondition::SubjectState { state, .. } => Some(state.iter().collect()),
        ResolvedCondition::StateChange { states, .. } => Some(states.iter().collect()),
        ResolvedCondition::When { .. }
        | ResolvedCondition::Otherwise
        | ResolvedCondition::External { .. }
        | ResolvedCondition::ExternalWhen { .. }
        | ResolvedCondition::WrongState
        | ResolvedCondition::SubjectField { .. }
        | ResolvedCondition::SubjectPredicate { .. }
        | ResolvedCondition::Related { .. }
        | ResolvedCondition::UnknownInstance
        | ResolvedCondition::InputAbsent
        | ResolvedCondition::ExistingInstance => None,
    }
}

/// Whether this branch can be the one taken while the subject rests in `held`.
///
/// A condition that reads no state admits every state, which is why the `None` case answers `true`:
/// the question being asked is whether this branch is *excluded* there.
fn admits_held_state(condition: &ResolvedCondition, held: &StateName) -> bool {
    admitted_states(condition).is_none_or(|states| states.contains(held))
}

/// Whether `outcome` is an input-guarded refusal naming no subject beside branches that read the
/// held state (beyond10x/ess#227).
///
/// It is answered before existence and before the held state (`docs/design/outcome-shapes.md`
/// "Precedence"), so its own scenario reaches it by input alone, as a plain send; the rows it is
/// sent for again, one per held state a sibling runs from, are arranged by
/// `existence::refusals_in_each_held_state`.
pub(crate) fn is_state_input_refusal(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    has_subject_guards(command)
        && is_input_guarded_refusal(outcome)
        && !state_default(outcome)
        && outcome.subject.is_none()
        && outcome.replays.is_none()
}

fn state_default(outcome: &ResolvedOutcome) -> bool {
    matches!(&outcome.condition, ResolvedCondition::Otherwise)
        || matches!(&outcome.condition, ResolvedCondition::When { predicate } if predicate.is_trivially_true())
}

/// Select an input only after every competing branch sees the same held state.
fn reach_in_state(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    held: &StateName,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    let guards: Vec<_> = command.outcomes.iter().filter_map(when).collect();
    let mut tried = 0;
    for inputs in searched_candidates(ir, command, vec![guards.clone()], distinction) {
        let inputs = inputs?;
        for input in &inputs {
            let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
            if selected_in_state(command, outcome, held, &facts)? {
                return Ok(input.clone());
            }
        }
        tried = tried.max(inputs.len());
    }
    Err(unsatisfied(
        &guards,
        format!("{} selected in held state {held}", outcome.name),
        tried,
    ))
}

/// Whether `outcome` is the one branch these input facts select while the subject rests in `held`.
fn selected_in_state(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    held: &StateName,
    facts: &crate::InputFacts<'_>,
) -> Result<bool, RefusalCause> {
    let mut selected = Vec::new();
    for branch in command
        .outcomes
        .iter()
        .filter(|branch| !state_default(branch))
    {
        let predicate = match &branch.condition {
            ResolvedCondition::When { predicate } => Some(predicate),
            // Both state-reading conditions compete here, and a branch whose admitted states
            // exclude `held` is not competing: it is the *uniqueness* below that this feeds, so
            // dropping a branch that does admit `held` would let another one look uniquely
            // selected when two of them match the same (state, input) pair.
            ResolvedCondition::SubjectState { predicate, .. }
            | ResolvedCondition::StateChange { predicate, .. }
                if admits_held_state(&branch.condition, held) =>
            {
                predicate.as_ref()
            }
            _ => continue,
        };
        if let Some(predicate) = predicate {
            if !decides(facts, &[predicate], true)? {
                continue;
            }
        }
        selected.push(branch);
    }
    // An input refusal these facts select answers before any held-state branch (beyond10x/ess#227),
    // as the partition in `ess_domain::command::subject_state` counts it; of two, the first
    // declared answers (`selected` keeps declaration order), as Entity Runtime takes it.
    if let Some(first) = selected
        .iter()
        .copied()
        .find(|branch| is_state_input_refusal(command, branch))
    {
        selected = vec![first];
    }
    if selected.is_empty() {
        selected.extend(
            command
                .outcomes
                .iter()
                .filter(|branch| state_default(branch)),
        );
    }
    Ok(selected.len() == 1 && selected[0].name == outcome.name)
}

/// Choose and establish the state together with the input; neither half proves the other.
fn prepare_state_input(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    if outcome.test_strategy == TestStrategy::InjectFault {
        return Ok((
            prepare(ir, outcome, actors, None)?,
            reach_external(ir, command, outcome, Distinction::PLAIN)?,
        ));
    }
    let subject = outcome
        .subject
        .as_ref()
        .ok_or(RefusalCause::StrategyWithoutGuard {
            strategy: outcome.test_strategy,
        })?;
    let lifecycle = &ir.entity(&subject.entity).lifecycle;
    // Only the states this branch admits are candidates. Falling back to the whole lifecycle for a
    // condition that narrows it is the silent-admit failure: a `StateChange` branch requiring its
    // move to CHANGE the held state would have had the arrival state offered to it, and a scenario
    // that arranges a state the branch refuses proves nothing about the branch it names.
    let states: Vec<_> = match &outcome.condition {
        ResolvedCondition::SubjectState { state, .. } => state.iter().cloned().collect(),
        _ => lifecycle.states.iter().cloned().collect(),
    };
    let mut first = None;
    for state in states {
        if subject
            .effect
            .transition()
            .is_some_and(|transition| !transition.from.contains(&state))
        {
            continue;
        }
        match reach_in_state(ir, command, outcome, &state, Distinction::PLAIN).and_then(|input| {
            let mut setup = prepare(ir, outcome, actors, Some(&state))?;
            let instance = setup
                .instance
                .as_ref()
                .ok_or(RefusalCause::StrategyWithoutGuard {
                    strategy: outcome.test_strategy,
                })?;
            let (observed, view) = observe_subject_state(ir, outcome, instance, &state)?;
            setup.steps.extend(observed);
            setup.source.insert(view.into());
            Ok((setup, input))
        }) {
            Ok(pair) => return Ok(pair),
            Err(reason) => {
                first.get_or_insert(reason);
            }
        }
    }
    Err(first.unwrap_or(RefusalCause::StrategyWithoutGuard {
        strategy: outcome.test_strategy,
    }))
}

/// Observe the actual identity/state pair through a declared immediate projection.
fn observe_subject_state(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    instance: &InstanceName,
    state: &StateName,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let subject = outcome
        .subject
        .as_ref()
        .ok_or(RefusalCause::StrategyWithoutGuard {
            strategy: outcome.test_strategy,
        })?;
    observe_selection_subject(ir, subject, instance, state, &outcome.name.to_string())
}

fn observe_selection_subject(
    ir: &EssIr,
    subject: &ResolvedSubject,
    instance: &InstanceName,
    state: &StateName,
    label: &str,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let entity = ir.entity(&subject.entity);
    let view = ir.views().values().find(|view| {
        !view.is_aggregate() && view.source == subject.entity && paging::read_whole(view) && view.filter.is_none()
            && view.assertion_style == AssertionStyle::Expect
            && view.field(&entity.identity.name).is_some_and(|field| field.type_ref == entity.identity.type_ref)
            && view.field(EntitySpec::STATE).is_some_and(|field| field.type_ref == entity.state_field().type_ref)
    }).ok_or_else(|| RefusalCause::NoWitness(WitnessGap {
        path: format!("{label}.subject_state"),
        type_ref: entity.name.to_string(),
        reason: "subject-state selection requires a declared unfiltered immediate view projecting identity and lifecycle state",
    }))?;
    let name = ViewRef::new(view.name.clone());
    let fields = [
        (
            entity.identity.name.clone(),
            ScenarioValue::instance(instance.clone()),
        ),
        (
            EntitySpec::STATE.to_owned(),
            ScenarioValue::literal(Node::Text(state.to_string())),
        ),
    ]
    .into_iter()
    .collect();
    let mut steps = Vec::new();
    require(
        view,
        &name,
        BTreeMap::new(),
        ViewExpectation::Contains { fields },
        &mut steps,
    );
    Ok((steps, name))
}

fn reach_external(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    let guards = match &outcome.condition {
        ResolvedCondition::ExternalWhen { predicate, .. } => vec![predicate],
        _ => Vec::new(),
    };
    // An input-guarded refusal is taken before the external decision is asked for (beyond10x/ess
    // #178), so the witness refutes every one; searched first over the branch's own guard, as it
    // always was, and over the refusals' guards as well only where that finds none.
    let refusals: Vec<&Predicate> = sibling_refusals(command, outcome)
        .filter_map(when)
        .collect();
    // An accepting `when:` branch declared before it answers an input its guard claims, whatever
    // the provider says (beyond10x/ess#217), so the witness refutes those as well; searched last.
    let earlier = earlier_accepting(command, outcome);
    let mut searches = vec![guards.clone()];
    let mut widened = guards.clone();
    if !refusals.is_empty() {
        widened.extend(refusals.iter().copied());
        searches.push(widened.clone());
    }
    if !earlier.is_empty() {
        widened.extend(earlier.iter().copied());
        searches.push(widened);
    }
    let mut tried = 0;
    let mut shadow = Shadow::default();
    for inputs in searched_candidates(ir, command, searches, distinction) {
        let inputs = inputs?;
        for input in &inputs {
            let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
            if !decides(&facts, &guards, true)? {
                continue;
            }
            if decides(&facts, &refusals, false)? && !claimed_by(&facts, &earlier) {
                return Ok(input.clone());
            }
            shadow.record(command, outcome, &facts)?;
        }
        tried = tried.max(inputs.len());
    }
    let predicate = shadow
        .rendered(&guards)
        .unwrap_or_else(|| rendered(&guards, true));
    Err(unsatisfied(&guards, predicate, tried))
}

/// The input that reaches this branch, decided rather than assumed.
///
/// Branches on [`ResolvedOutcome::test_strategy`] and never on the predicate: the compiler decided
/// reachability once, and asking again here is the divergence the field exists to prevent.
///
/// `distinction` says which instance the input is for. A scenario that arranges a second row in a
/// ranked view runs the same creating outcome twice, and the guard is decided again against the
/// second witness rather than assumed to hold of it: a value that moved may have moved out of the
/// branch, and a suite that sent it anyway would arrange an instance the specification says the
/// command refuses.
fn reach(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    // External setup chooses a declared cause independently of the ordinary state partition.
    // Both guarded and unguarded external outcomes still use their own input eligibility.
    if outcome.test_strategy == TestStrategy::InjectFault {
        return reach_external(ir, command, outcome, distinction);
    }
    // `SubjectState` names exactly one state, so the input can be chosen against it here.
    // `StateChange` names a SET, and picking one of them is a choice about the arrangement rather
    // than about the input — so it deliberately does NOT get an arm: it falls to the refusal below,
    // which `has_subject_guards` now answers for it. `run` and `invoke` both route such a command
    // through `prepare_state_input`, which decides the state and the input together and is the only
    // place allowed to pick.
    // A listed guard (ess/18) is reached in the first state it lists that an input selects it in.
    if let ResolvedCondition::SubjectState { state, .. } = &outcome.condition {
        let mut first = None;
        for held in state.iter() {
            match reach_in_state(ir, command, outcome, held, distinction) {
                Ok(input) => return Ok(input),
                Err(reason) => {
                    first.get_or_insert(reason);
                }
            }
        }
        return Err(first.expect("a held-state guard names at least one state"));
    }
    if has_subject_guards(command)
        && !existence::creates_unknown(outcome)
        && !is_state_input_refusal(command, outcome)
    {
        return Err(RefusalCause::StrategyWithoutGuard {
            strategy: outcome.test_strategy,
        });
    }
    // Every branch of a command guarded by a related row (ess/18, #211) is decided by that row, which
    // only `related_guard::prepare` arranges: an input chosen here would be sent for a row nobody
    // arranged, and a correct implementation would answer another branch.
    if related_guard::uses(command) {
        return Err(related_guard::unarranged());
    }
    let (guards, satisfy) = plain_guards(command, outcome)?;
    let mut tried = 0;
    let mut shadow = Shadow::default();
    let searches = searched_guards(command, outcome, &guards, satisfy);
    for inputs in searched_candidates(ir, command, searches, distinction) {
        let inputs = inputs?;
        for input in &inputs {
            let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
            if admits_plain(command, outcome, &facts, &guards, satisfy)? {
                return Ok(input.clone());
            }
            if satisfy && decides(&facts, &guards, true)? {
                shadow.record(command, outcome, &facts)?;
            }
        }
        tried = tried.max(inputs.len());
    }
    let predicate = shadow
        .rendered(&guards)
        .unwrap_or_else(|| rendered(&guards, satisfy));
    Err(unsatisfied(&guards, predicate, tried))
}

/// Which sibling input-guarded refusals claimed the candidates that satisfied an accepting
/// branch's own guard (beyond10x/ess#178).
///
/// Where every such candidate was claimed by one, the branch is shadowed by the stated precedence,
/// and its refusal names the refusal rather than saying that a satisfiable guard has no witness.
#[derive(Default)]
pub(super) struct Shadow {
    /// The refusals that claimed some candidate, by name.
    claimed: BTreeMap<OutcomeName, String>,
    /// Whether some candidate satisfying the guard was lost to anything other than a refusal.
    otherwise: bool,
    /// Whether the branch shadowed is itself an input-guarded refusal, which the model orders
    /// neither before nor after its siblings.
    refusal: bool,
    /// The accepting branches declared before it that claimed some candidate, by name
    /// (beyond10x/ess#217).
    earlier: BTreeMap<OutcomeName, String>,
}

impl Shadow {
    pub(super) fn record(
        &mut self,
        command: &ResolvedCommand,
        outcome: &ResolvedOutcome,
        facts: &crate::InputFacts<'_>,
    ) -> Result<(), RefusalCause> {
        if outcome.error.is_some() && !is_input_guarded_refusal(outcome) && !is_external(outcome) {
            self.otherwise = true;
            return Ok(());
        }
        self.refusal |= is_input_guarded_refusal(outcome);
        let mut claimed = false;
        for refusal in sibling_refusals(command, outcome) {
            let Some(guard) = when(refusal) else {
                continue;
            };
            if decides(facts, &[guard], true)? {
                claimed = true;
                self.claimed.insert(refusal.name.clone(), guard.to_string());
            }
        }
        if !claimed {
            for earlier in earlier_accepting_branches(command, outcome) {
                let Some(guard) = when(earlier) else {
                    continue;
                };
                if claimed_by(facts, &[guard]) {
                    claimed = true;
                    self.earlier.insert(earlier.name.clone(), guard.to_string());
                }
            }
        }
        self.otherwise |= !claimed;
        Ok(())
    }

    pub(super) fn rendered(&self, guards: &[&Predicate]) -> Option<String> {
        if self.otherwise || (self.claimed.is_empty() && self.earlier.is_empty()) {
            return None;
        }
        if !self.earlier.is_empty() {
            let named = |claims: &BTreeMap<OutcomeName, String>| {
                claims
                    .iter()
                    .map(|(name, guard)| format!("{name} ({guard})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let mut outside = Vec::new();
            if !self.claimed.is_empty() {
                outside.push(format!(
                    "{}, the input-guarded refusal taken first",
                    named(&self.claimed)
                ));
            }
            outside.push(format!(
                "{}, the accepting branch declared first",
                named(&self.earlier)
            ));
            return Some(format!(
                "{} outside {}",
                rendered(guards, true),
                outside.join("; ")
            ));
        }
        let refusals: Vec<String> = self
            .claimed
            .iter()
            .map(|(name, guard)| format!("{name} ({guard})"))
            .collect();
        if self.refusal {
            // Two refusals whose guards overlap: the first declared answers (beyond10x/ess#227
            // adversary pass 1), so an input a refusal declared before this one claims is that
            // refusal's.
            return Some(format!(
                "{} outside {}, an input-guarded refusal declared before it, which answers first",
                rendered(guards, true),
                refusals.join(", ")
            ));
        }
        Some(format!(
            "{} outside {}, the input-guarded refusal taken first",
            rendered(guards, true),
            refusals.join(", ")
        ))
    }
}

/// The guard sets a stateless branch's witness is searched over, in order.
///
/// The branch's own guards first, which is every search there was before beyond10x/ess#178, so a
/// witness that already refuted every sibling refusal stays the one it was. An accepting branch
/// must also refute every sibling input-guarded refusal ([`admits_plain`]), and a candidate search
/// varies only what its guards read — so where none of the first candidates steps out of a refusal
/// (`count < 5` beside `open == false`, and `count` at its base `1`), the search runs again over
/// the refusals' guards as well, which puts `5` on `count`'s ladder. An input-guarded refusal
/// refutes its sibling refusals too, so it is searched the same way.
fn searched_guards<'c>(
    command: &'c ResolvedCommand,
    outcome: &'c ResolvedOutcome,
    guards: &[&'c Predicate],
    satisfy: bool,
) -> Vec<Vec<&'c Predicate>> {
    let mut searches = vec![guards.to_vec()];
    if satisfy && (outcome.error.is_none() || is_input_guarded_refusal(outcome)) {
        let mut widened = guards.to_vec();
        widened.extend(
            sibling_refusals(command, outcome)
                .filter_map(when)
                .filter(|guard| !guards.contains(guard)),
        );
        if widened.len() > guards.len() {
            searches.push(widened.clone());
        }
        // An accepting `when:` branch also steps out of every accepting branch declared before it
        // (beyond10x/ess#217). Searched last, so a witness the earlier searches found is kept, and
        // only beside a default: without one the search above already refutes every sibling.
        let defaulted = command
            .outcomes
            .iter()
            .any(|other| other.test_strategy == TestStrategy::DefaultBranch);
        let earlier: Vec<&Predicate> = earlier_accepting(command, outcome)
            .into_iter()
            .filter(|guard| !widened.contains(guard))
            .collect();
        if defaulted && !earlier.is_empty() {
            widened.extend(earlier);
            searches.push(widened);
        }
    }
    searches
}

/// Whether `outcome` is an input-guarded refusal: a `when:` over the input and an `error:`.
///
/// Such a branch is taken before any accepting branch whose guard it overlaps
/// (`docs/design/input-guard-overlap-precedence.md`).
pub(crate) fn is_input_guarded_refusal(outcome: &ResolvedOutcome) -> bool {
    outcome.error.is_some() && matches!(outcome.condition, ResolvedCondition::When { .. })
}

/// The guard over the input an accepting branch is selected under, where it has one: its `when:`,
/// the `when:` beside its `when_subject:`, or the input guard of an external branch. What an
/// input-guarded refusal can overlap.
pub(crate) fn accepting_input_half(outcome: &ResolvedOutcome) -> Option<&Predicate> {
    if outcome.error.is_some() {
        return None;
    }
    match &outcome.condition {
        ResolvedCondition::When { predicate }
        | ResolvedCondition::ExternalWhen { predicate, .. } => Some(predicate),
        ResolvedCondition::SubjectField { predicate, .. } => predicate.as_ref(),
        ResolvedCondition::SubjectPredicate { input, .. } => input.as_ref(),
        _ => None,
    }
}

/// Every input-guarded refusal of `command` answered before `outcome`: all of them for any other
/// branch, and for an input-guarded refusal the ones declared before it — of two refusals an input
/// selects, the first declared answers, as Entity Runtime takes it (beyond10x/ess#227 adversary
/// pass 1). A refusal's witness refutes these and needs to refute nothing declared after it.
pub(crate) fn sibling_refusals<'c>(
    command: &'c ResolvedCommand,
    outcome: &'c ResolvedOutcome,
) -> impl Iterator<Item = &'c ResolvedOutcome> {
    let before = if is_input_guarded_refusal(outcome) {
        command
            .outcomes
            .iter()
            .position(|other| other.name == outcome.name)
            .unwrap_or(command.outcomes.len())
    } else {
        command.outcomes.len()
    };
    command.outcomes[..before]
        .iter()
        .filter(move |other| other.name != outcome.name && is_input_guarded_refusal(other))
}

/// Whether `outcome` is an accepting branch selected by a plain `when:` over the input: what the
/// declared precedence among accepting guarded branches orders (beyond10x/ess#217).
fn is_input_guarded_accepting(outcome: &ResolvedOutcome) -> bool {
    outcome.error.is_none() && matches!(outcome.condition, ResolvedCondition::When { .. })
}

/// The candidates of each search in turn, then those of the widest search tried between the
/// literals of its `Decimal` leaves ([`crate::witness::candidates_between`]) where that adds any.
///
/// A caller stops at the first input it accepts, so the second pass runs only where the ladders
/// found none — a witness they find is the one it always was — and an interval between two literals
/// less than two apart (`amount > 11.5` beside `amount < 11.6`) is not refused as unreachable.
fn searched_candidates<'c>(
    ir: &'c EssIr,
    command: &'c ResolvedCommand,
    searches: Vec<Vec<&'c Predicate>>,
    distinction: Distinction,
) -> impl Iterator<Item = Result<Vec<BTreeMap<String, Node>>, RefusalCause>> + 'c {
    let widest = searches.last().cloned();
    let plain = searches.into_iter().map(move |searched| {
        candidates(ir, command, &searched, distinction).map_err(RefusalCause::NoWitness)
    });
    let between = widest.into_iter().filter_map(move |searched| {
        crate::witness::candidates_between(ir, command, &searched, distinction)
            .map_err(RefusalCause::NoWitness)
            .transpose()
    });
    plain.chain(between)
}

/// The first candidate satisfying `holds`: over the ladders of `searched`, then between the
/// literals of its `Decimal` leaves ([`searched_candidates`]).
fn first_candidate(
    ir: &EssIr,
    command: &ResolvedCommand,
    searched: &[&Predicate],
    holds: impl Fn(&crate::InputFacts<'_>) -> bool,
) -> Option<BTreeMap<String, Node>> {
    searched_candidates(ir, command, vec![searched.to_vec()], Distinction::PLAIN)
        .filter_map(Result::ok)
        .find_map(|inputs| {
            inputs
                .into_iter()
                .find(|input| flatten(ir, command, input).is_ok_and(|facts| holds(&facts)))
        })
}

/// Why a binding cannot force `forced` without observing the input its mapping supplies, if it
/// cannot: its eligibility is a guard over that input, or an accepting `when:` branch declared
/// before it answers an input that guard claims whatever the provider says (beyond10x/ess#217).
fn forced_eligibility(invoked: &ResolvedCommand, forced: &ResolvedOutcome) -> Option<BindingGap> {
    if matches!(forced.condition, ResolvedCondition::ExternalWhen { .. }) {
        return Some(BindingGap::AccessorObservation {
            reason: "GuardedExternalEligibility: fault eligibility requires an observation of the binding-mapped input".into(),
        });
    }
    if !earlier_accepting(invoked, forced).is_empty() {
        return Some(BindingGap::AccessorObservation {
            reason: "PrecededExternalEligibility: an accepting branch declared before the forced one decides the binding-mapped input first".into(),
        });
    }
    None
}

/// Whether `outcome` is decided by a provider: an `external:` branch, with or without a `when:`.
fn is_external(outcome: &ResolvedOutcome) -> bool {
    matches!(
        outcome.condition,
        ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
    )
}

/// Every accepting `when:` branch declared before `outcome`, where `outcome` is one or is an
/// external branch.
///
/// Among the accepting guarded branches and external branches of one command the first declared
/// whose guard holds answers (`docs/design/input-guard-overlap-precedence.md`) — for an external
/// branch, whatever the provider says, which is Entity Runtime's source order
/// (`ess-entity-runtime` `lower`). So an input one of these claims is not an input that reaches
/// `outcome`. An external branch declared earlier is not one of them: a provider that does not
/// take it passes the input on.
fn earlier_accepting_branches<'c>(
    command: &'c ResolvedCommand,
    outcome: &'c ResolvedOutcome,
) -> Vec<&'c ResolvedOutcome> {
    if !is_input_guarded_accepting(outcome) && !is_external(outcome) {
        return Vec::new();
    }
    command
        .outcomes
        .iter()
        .take_while(|other| other.name != outcome.name)
        .filter(|other| is_input_guarded_accepting(other))
        .collect()
}

/// The guards of [`earlier_accepting_branches`].
fn earlier_accepting<'c>(
    command: &'c ResolvedCommand,
    outcome: &'c ResolvedOutcome,
) -> Vec<&'c Predicate> {
    earlier_accepting_branches(command, outcome)
        .into_iter()
        .filter_map(when)
        .collect()
}

/// Whether one of `guards` decidedly holds of `facts`. A guard these facts leave undecided claims
/// nothing: an overlap synthesis cannot decide is neither witnessed nor refused.
fn claimed_by(facts: &crate::InputFacts<'_>, guards: &[&Predicate]) -> bool {
    guards
        .iter()
        .any(|guard| matches!(facts.decide(guard), Decision::Satisfied))
}

/// Whether one input reaches `outcome`, by exactly the reading [`reach`], [`reach_in_state`] and
/// [`reach_external`] choose their inputs under — so an input changed after it was chosen (an
/// update's witness moved off the row's value, a guard's boundary) is decided again rather than
/// assumed to reach the branch still.
///
/// `held` is the state the subject rests in, which a command whose branches read it needs.
fn selects_branch(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    held: Option<&StateName>,
    input: &BTreeMap<String, Node>,
) -> Result<bool, RefusalCause> {
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    if outcome.test_strategy == TestStrategy::InjectFault {
        // An accepting `when:` branch declared before it answers first (beyond10x/ess#217).
        if claimed_by(&facts, &earlier_accepting(command, outcome)) {
            return Ok(false);
        }
        return match &outcome.condition {
            ResolvedCondition::ExternalWhen { predicate, .. } => {
                decides(&facts, &[predicate], true)
            }
            _ => Ok(true),
        };
    }
    if let ResolvedCondition::SubjectState { state, .. } = &outcome.condition {
        // The held state the arrangement left, where the guard admits it; otherwise the first it
        // lists, which is the state a listed guard (ess/18) is reached in by [`reach`].
        let held = held
            .filter(|held| state.contains(held))
            .or_else(|| state.iter().next())
            .expect("a held-state guard names at least one state");
        return selected_in_state(command, outcome, held, &facts);
    }
    // An input refusal beside held-state branches (beyond10x/ess#227) is decided by its input alone
    // where no held state is given: it is answered before the state is read.
    if has_subject_guards(command)
        && !existence::creates_unknown(outcome)
        && !(held.is_none() && is_state_input_refusal(command, outcome))
    {
        let held = held.ok_or(RefusalCause::StrategyWithoutGuard {
            strategy: outcome.test_strategy,
        })?;
        return selected_in_state(command, outcome, held, &facts);
    }
    let (guards, satisfy) = plain_guards(command, outcome)?;
    admits_plain(command, outcome, &facts, &guards, satisfy)
}

/// The guards a stateless branch's input is decided against, and whether it must satisfy them
/// (its own guard) or refute them (every sibling's, for the default branch).
fn plain_guards<'c>(
    command: &'c ResolvedCommand,
    outcome: &'c ResolvedOutcome,
) -> Result<(Vec<&'c Predicate>, bool), RefusalCause> {
    let strategy = outcome.test_strategy;
    let guards: Vec<&Predicate> = match strategy {
        TestStrategy::ConstructInput => match when(outcome) {
            Some(guard) => vec![guard],
            None => return Err(RefusalCause::StrategyWithoutGuard { strategy }),
        },
        // The default branch is defined relative to every *other* branch, so what a candidate has
        // to do is refute all of them rather than satisfy anything.
        TestStrategy::DefaultBranch => command
            .outcomes
            .iter()
            .filter(|other| other.name != outcome.name)
            .filter_map(when)
            .collect(),
        TestStrategy::InjectFault => Vec::new(),
        // A creating `unknown_instance:` (ess/16) is selected by existence, not by input: like the
        // default branch, its input refutes every sibling guard.
        TestStrategy::SendUnknownIdentity if existence::creates_unknown(outcome) => command
            .outcomes
            .iter()
            .filter(|other| other.name != outcome.name)
            .filter_map(when)
            .collect(),
        // A wrong-state branch is decided by the subject, not by the input, so nothing asks this
        // function for the input that reaches it: `refused_here` sends the input that reaches the
        // *moving* branch and arranges the subject instead. Answering with "no guards" would hand
        // back an arbitrary candidate presented as the one that reaches the branch, which is the
        // invention this crate refuses everywhere else — so it is a drift alarm.
        TestStrategy::ArrangeState
        | TestStrategy::ReplayResult
        | TestStrategy::SendUnknownIdentity
        | TestStrategy::SendNoInput
        | TestStrategy::SendExistingIdentity
        | TestStrategy::ConstructInputInState
        | TestStrategy::ObserveSubjectFact
        | TestStrategy::ArrangeRelatedRow => {
            return Err(RefusalCause::StrategyWithoutGuard { strategy })
        }
    };
    Ok((guards, strategy == TestStrategy::ConstructInput))
}

/// Whether these input facts do what a stateless branch's strategy asks of `guards`, and — for a
/// guarded branch of a command with no default — refute every sibling's guard as well.
///
/// An accepting guarded branch refutes every sibling input-guarded refusal whether or not a default
/// exists (beyond10x/ess#178): such a refusal is taken before any accepting branch it overlaps, so
/// an input satisfying both reaches the refusal, and a scenario requiring the accepting branch for
/// it would fail a target that honours the precedence.
fn admits_plain(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    facts: &crate::InputFacts<'_>,
    guards: &[&Predicate],
    satisfy: bool,
) -> Result<bool, RefusalCause> {
    if !decides(facts, guards, satisfy)? {
        return Ok(false);
    }
    if !satisfy {
        return Ok(true);
    }
    let others: Vec<_> = if command
        .outcomes
        .iter()
        .all(|other| other.test_strategy != TestStrategy::DefaultBranch)
    {
        // A refusal declared after an input-guarded refusal never answers before it, so it is not
        // refuted (beyond10x/ess#227 adversary pass 1).
        let later = |other: &ResolvedOutcome| {
            is_input_guarded_refusal(outcome)
                && is_input_guarded_refusal(other)
                && !sibling_refusals(command, outcome).any(|before| before.name == other.name)
        };
        command
            .outcomes
            .iter()
            .filter(|other| other.name != outcome.name && !later(other))
            .filter_map(when)
            .collect()
    } else if outcome.error.is_none() || is_input_guarded_refusal(outcome) {
        // An input-guarded refusal refutes the refusals declared before it: of two an input
        // selects, the first declared answers ([`sibling_refusals`]). An accepting `when:` branch
        // is not reached by an input an accepting branch declared before it claims
        // (beyond10x/ess#217).
        if claimed_by(facts, &earlier_accepting(command, outcome)) {
            return Ok(false);
        }
        sibling_refusals(command, outcome)
            .filter_map(when)
            .collect()
    } else {
        Vec::new()
    };
    decides(facts, &others, false)
}

/// `true` when this candidate does what the strategy asks of every guard.
///
/// `Unknown` leaves through the `Err`, and it leaves immediately: five of its six causes are
/// properties of the specification, so the next candidate meets the same wall.
fn decides(
    facts: &crate::InputFacts<'_>,
    guards: &[&Predicate],
    satisfy: bool,
) -> Result<bool, RefusalCause> {
    for guard in guards {
        match facts.decide(guard) {
            Decision::Satisfied => {
                if !satisfy {
                    return Ok(false);
                }
            }
            Decision::Refuted(_) => {
                if satisfy {
                    return Ok(false);
                }
            }
            Decision::Unevaluable(refusal) => return Err(RefusalCause::GuardUnevaluable(refusal)),
        }
    }
    Ok(true)
}

/// What a refused search was looking for, for the diagnostic.
fn rendered(guards: &[&Predicate], satisfy: bool) -> String {
    let written: Vec<String> = guards.iter().map(ToString::to_string).collect();
    if satisfy {
        written.join(" and ")
    } else {
        format!("none of: {}", written.join(", "))
    }
}

/// Every event this branch must not publish: everything the specification declares, minus its own.
///
/// # Why the whole specification and not just the sibling branches
///
/// `ESS-CF-NO-EVENT` names the rule "a branch publishes no event it does not declare it emits", and
/// the sibling set is a narrower claim wearing that sentence: it catches a refusal that announces
/// the success it refused, and lets a branch announce anything declared elsewhere. An invoice
/// created and simultaneously announced as cancelled is a defect no downstream consumer survives,
/// and no sibling of `accepted` emits `InvoiceCancelled`.
///
/// # Why it is narrower than "no other event at all", twice over
///
/// **It is about one invocation, not about the scenario.** The check reads
/// `SemanticCommandResult::direct_events` — what §9 says *this* command published — and nothing
/// else. A scenario legitimately causes other events while it runs: an arrangement creates an
/// invoice before the branch under test, and a binding invokes a downstream command whose own
/// events arrive later. Asserting over everything a scenario observed would fail a correct
/// implementation for doing what the specification told it to.
///
/// **It names only declared events.** An implementation may publish occurrences this specification
/// says nothing about — an audit record, a technical heartbeat — and a suite refusing those would
/// be enforcing a rule no document wrote. The model closes a *branch's* emissions, not the
/// system's output.
fn not_emitted(ir: &EssIr, emitted: &[EventRef]) -> Vec<EventRef> {
    ir.events()
        .values()
        .map(|event| EventRef::new(event.name.clone()))
        .filter(|event| !emitted.contains(event))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The payload values this branch's `payload:` declaration determines, where the scenario knows
/// them.
///
/// The other half of [`PayloadShape`]'s argument: a *value* is assertable exactly where some
/// construct says where it comes from, and the outcome's declared payload is that construct. What
/// survives here is narrower than what the declaration determines, because the assertion is a
/// literal comparison:
///
/// * a **literal source** is the text the author wrote — `ess-domain` admits one only onto a field
///   that is text or an enum underneath, so the text is the value;
/// * an **input source** is assertable where the scenario supplied a literal for that input. An
///   input that carries a bound instance is still determined, but its value is the target's to
///   mint and the suite knows it only as a reference — [`ScenarioStep::CaptureInstance`] is how
///   identity already crosses that gap, and a payload literal here would be a guess. The field
///   stays covered by the shape, not by a value.
///
/// A declared conversion does not widen this: the model's conversions permit two types to meet and
/// name no transformation, so the value that went in is the value that comes out — the same
/// reading [`ScenarioStep::ExpectInvocation`] already makes of a binding's mapped field.
fn determined_payload(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    event: &EventRef,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> BTreeMap<String, Node> {
    outcome
        .payload
        .iter()
        .find(|payload| EventRef::from(&payload.event) == *event)
        .map(|determined| determined_fields(ir, &determined.fields, supplied, before))
        .unwrap_or_default()
}

/// The identity-typed payload fields of `event` this run determines, each as the instance it
/// resolves to (beyond10x/ess#273): an input carrying a bound instance, the subject's own identity,
/// and an identity the arrangement settled — a related row's, or one the subject holds.
///
/// The half [`determined_payload`] leaves to the shape: the suite cannot write the identity the
/// target mints as a literal, but it can name the instance that captured it, which the runner
/// resolves before comparing. An implementation that drops such a field, or publishes another
/// identity in its place, then fails the scenario. A field filled through a declared conversion is
/// left out, for the reason [`determined_payload`] states for literals.
fn determined_identities(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    event: &EventRef,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
    subject: Option<&InstanceName>,
) -> BTreeMap<String, ScenarioValue> {
    let subject_identity = outcome
        .subject
        .as_ref()
        .filter(|held| !matches!(held.effect, ResolvedEffect::Creates))
        .map(|held| &ir.entity(&held.entity).identity.name);
    let mut values = BTreeMap::new();
    let Some(payload) = outcome
        .payload
        .iter()
        .find(|payload| EventRef::from(&payload.event) == *event)
    else {
        return values;
    };
    for field in payload.fields.iter().filter(|it| it.conversion.is_none()) {
        let value = match &field.value {
            ResolvedPayloadValue::InputField { field: input, .. } => supplied.get(input).cloned(),
            ResolvedPayloadValue::SubjectField { field: read, .. }
                if Some(read) == subject_identity =>
            {
                subject.cloned().map(ScenarioValue::instance)
            }
            _ => expression_value(ir, field, supplied, before),
        };
        if let Some(value @ ScenarioValue::Instance { .. }) = value {
            values.insert(field.target.clone(), value);
        }
    }
    values
}

/// The expectation that `event` was published: its determined literals and, where this run
/// determines any, its identities ([`determined_identities`]), which make it an
/// `expect_event_values` step (suite/18).
fn expect_event_step(
    event: &EventRef,
    literals: BTreeMap<String, Node>,
    identities: BTreeMap<String, ScenarioValue>,
    shape: PayloadShape,
) -> ScenarioStep {
    if identities.is_empty() {
        return ScenarioStep::ExpectEvent {
            event: event.clone(),
            payload: literals,
            shape,
        };
    }
    let mut payload = identities;
    payload.extend(
        literals
            .into_iter()
            .map(|(key, value)| (key, ScenarioValue::literal(value))),
    );
    ScenarioStep::ExpectEventValues {
        event: event.clone(),
        payload,
        shape,
    }
}

/// The `expect_error` step for the error `outcome` reports, comparing each field the
/// specification gives a source (ess/19, `story:error-payload-sources`) whose value this scenario
/// determines, by the rules [`determined_payload`] reads an event's by.
///
/// `supplied` is what the command under test was sent and `before` what the arrangement left in
/// the row the refusal is answered for. A field with no source, a `{generated: true}` one, and an
/// input carrying a bound instance are not compared — the partial comparison `expect_error`
/// states. An error with no declared source is compared by name alone, as before `ess/19`.
fn expect_error(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    error: &ess_compiler::ir::ErrorHandle,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> ScenarioStep {
    ScenarioStep::ExpectError {
        error: ErrorRef::from(error),
        fields: determined_fields(ir, &outcome.error_payload, supplied, before),
    }
}

/// The values of `fields` a scenario determines: [`determined_payload`]'s reading of one record's
/// sources, whichever record — an event or an error — they fill.
fn determined_fields(
    ir: &EssIr,
    fields: &[ess_compiler::ir::ResolvedPayloadField],
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> BTreeMap<String, Node> {
    let mut values = BTreeMap::new();
    for field in fields {
        match &field.value {
            // `Cleared` is refused on an event payload by `ess-domain`, so it cannot reach here;
            // matched with the other two that determine nothing rather than by a wildcard, so a
            // fifth source has to be decided rather than silently ignored.
            ResolvedPayloadValue::ResponseField { .. }
            | ResolvedPayloadValue::Generated
            | ResolvedPayloadValue::Cleared
            // ess/16 (#167): the rows one scenario changed, asserted by `set_effects` alone.
            | ResolvedPayloadValue::ChangedCount => {}
            // Read as the target's type, as `settled` reads a `sets:` literal: `ess-domain` admits
            // `true`, a whole number and a decimal over the primitives they spell, and asserting
            // their text would fail every implementation that publishes the number.
            ResolvedPayloadValue::Literal { value } => {
                if let Some(read) = literal_value(ir, &field.target_type, value, 0) {
                    values.insert(field.target.clone(), read);
                }
            }
            ResolvedPayloadValue::InputField { field: input, .. } => {
                if let Some(ScenarioValue::Literal { value }) = supplied.get(input) {
                    values.insert(field.target.clone(), value.clone());
                }
            }
            // ess/14: asserted where the arrangement determined what they read, as a literal.
            ResolvedPayloadValue::SubjectField { .. }
            | ResolvedPayloadValue::Increment { .. }
            | ResolvedPayloadValue::InputOrGenerated { .. }
            | ResolvedPayloadValue::Struct { .. }
            | ResolvedPayloadValue::RelatedField { .. }
            | ResolvedPayloadValue::CallerAttribute { .. } => {
                if let Some(ScenarioValue::Literal { value }) =
                    expression_value(ir, field, supplied, before)
                {
                    // ess/22 (#285): a value read through a reference that may be absent is absent
                    // where it is, and an absent top-level field may be published left out or as
                    // `null`, which an exact payload comparison cannot say. The row asserts it.
                    if value == Node::Null && absent_by_reference(&field.value) {
                        continue;
                    }
                    values.insert(field.target.clone(), value);
                } else {
                    // beyond10x/ess#179: a struct with an undetermined leaf is still asserted
                    // leaf by leaf, each under its dotted path (suite/26).
                    values.extend(determined_leaves(
                        ir,
                        field,
                        &field.target,
                        supplied,
                        before,
                    ));
                }
            }
        }
    }
    values
}

/// Requires every top-level field of `event` that a `{related: …}` source copies through a
/// reference this run left absent to be absent — left out or `null`, either spelling (ess/22,
/// beyond10x/ess#285): its leaf is made to admit no present value. The payload cannot say it — an
/// exact `null` there would fail an implementation leaving the field out — and the leaf can,
/// with nothing a runner does not already read.
fn absent_by_reference_leaves(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    event: &EventRef,
    run: &Run,
    shape: &mut PayloadShape,
) {
    let Some(payload) = outcome
        .payload
        .iter()
        .find(|payload| EventRef::from(&payload.event) == *event)
    else {
        return;
    };
    for field in &payload.fields {
        let absent = absent_by_reference(&field.value)
            && expression_value(ir, field, &run.input, &run.before_settled)
                == Some(ScenarioValue::literal(Node::Null));
        if !absent {
            continue;
        }
        let presence = shape
            .leaves()
            .get(&field.target)
            .and_then(|leaf| leaf.presence);
        shape.insert(
            field.target.clone(),
            LeafShape {
                holds: Holds::Enum {
                    variants: Vec::new(),
                },
                optional: true,
                presence,
            },
        );
    }
}

/// Whether `value` is a `{related: …}` read through a reference that may be absent (ess/22,
/// beyond10x/ess#285).
fn absent_by_reference(value: &ResolvedPayloadValue) -> bool {
    matches!(value, ResolvedPayloadValue::RelatedField { via, through, .. }
        if ess_compiler::ir::related_may_be_absent(via, through))
}

/// What the specification declares an event carries, flattened to leaves a runner can check (§13).
///
/// The one payload claim that needs no model change. [`PayloadShape`] argues why the *values* are
/// not here and what the model would have to gain before they could be.
pub(crate) fn payload_shape(ir: &EssIr, event: &EventRef) -> PayloadShape {
    let mut shape = PayloadShape::new();
    // Every `EventRef` a suite carries was minted from this IR's own events, so the lookup finds
    // one; a shape that described nothing would be a silently weaker assertion, which is the one
    // failure mode §36 rules out, so the absence is not quietly tolerated.
    let declared = ir
        .events()
        .get(event.name())
        .unwrap_or_else(|| panic!("`{event}` is an event this specification declares"));
    for field in &declared.fields {
        describe(ir, &field.type_ref, &field.name, false, 0, &mut shape);
        mark_presence(field, &field.name, false, &mut shape);
    }
    shape
}

/// Carries a field's declared presence policy (beyond10x/ess#139) onto the leaf that is the field.
///
/// Only where the leaf is the field itself and no enclosing `Optional` was walked: under an absent
/// parent every leaf is absent, and a runner cannot tell that absence from the field's own. A
/// struct-valued field has no leaf of its own, so its policy is not carried; both omissions are a
/// weaker assertion, never a wrong one.
fn mark_presence(
    field: &ess_compiler::ir::ResolvedField,
    path: &str,
    enclosed: bool,
    shape: &mut PayloadShape,
) {
    let (Some(presence), false, true) = (
        field.naming.presence,
        enclosed,
        field.type_ref.is_optional(),
    ) else {
        return;
    };
    if let Some(leaf) = shape.leaves().get(path).cloned() {
        shape.insert(path, leaf.with_presence(Some(presence)));
    }
}

/// One leaf per scalar the declared type reaches, under the dotted path that names it.
///
/// The same walk [`crate::input`] documents as a table, so a payload and a command input are held to
/// one reading of what `Optional<Money>` exposes. A type that refers to itself contributes nothing
/// past [`MAX_TYPE_DEPTH`]: refusing to describe a leaf is a weaker assertion, never a wrong one.
fn describe(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    path: &str,
    optional: bool,
    depth: usize,
    shape: &mut PayloadShape,
) {
    if depth > MAX_TYPE_DEPTH {
        return;
    }
    let mut leaf = |holds: Holds| {
        let described = LeafShape::required(holds);
        shape.insert(
            path,
            if optional {
                described.optional()
            } else {
                described
            },
        );
    };
    match type_ref {
        // The wrapper marks every leaf underneath absentable, including a struct's fields: a value
        // that is not there does not have fields that are.
        ResolvedTypeRef::Optional { of } => describe(ir, of, path, true, depth + 1, shape),
        ResolvedTypeRef::Primitive { name } => leaf(Holds::Primitive { kind: *name }),
        ResolvedTypeRef::List { .. } => leaf(Holds::List),
        ResolvedTypeRef::Map { .. } => leaf(Holds::Map),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            // Transparent, exactly as it is to a fact path: a newtype names no member, so the path
            // does not grow.
            ResolvedBody::Newtype { of, .. } => describe(ir, of, path, optional, depth + 1, shape),
            ResolvedBody::Enum { variants } => leaf(Holds::Enum {
                variants: variants
                    .iter()
                    .map(|variant| variant.name().to_owned())
                    .collect(),
            }),
            ResolvedBody::Union { .. } => leaf(Holds::Union),
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    let nested = format!("{path}.{}", field.name);
                    describe(ir, &field.type_ref, &nested, optional, depth + 1, shape);
                    mark_presence(field, &nested, optional, shape);
                }
            }
        },
    }
}

/// How many rows a declared order has to be compared against before it is a claim at all.
///
/// Two. [`ViewExpectation::Ranked`] compares adjacent pairs, and a view holding one row has no
/// pair — so an order asserted against one row is an assertion no implementation can fail, which is
/// worse than a missing check because it looks like a present one.
const RANKING_ROWS: usize = 2;

/// The view assertions a branch that changed an entity supports, and what they need arranged.
///
/// Two lists rather than one, because they do not run in the same place. Every assertion after an
/// [`ScenarioStep::ExecuteCommand`] is about *that* invocation, and `creates:` declares that a new
/// identity is published in an event — so a neighbouring instance arranged **before** the branch
/// would publish the first occurrence of that event, and the scenario's own subject would be the
/// second. The arrangement therefore goes between the branch's own assertions and the first read of
/// a view: after everything that reads the command, before anything that reads a projection.
#[derive(Debug, Default)]
struct ViewAssertions {
    /// The further instances a declared order needs, arranged once the branch has been required.
    arranged: Vec<ScenarioStep>,
    /// The reads and the requirements, after it.
    asserted: Vec<ScenarioStep>,
    /// The views named.
    views: BTreeSet<ViewRef>,
    /// What the arrangements depend on.
    source: BTreeSet<EssSemanticRef>,
}

/// The view assertions a branch that changed an entity supports (§14, §20).
///
/// Everything about the subject is read off the `run` rather than re-derived, because three
/// answers to *what did this scenario do to the entity* have to agree:
///
/// * [`Run::after`] is the state it is in once the branch has been taken — the lifecycle's
///   `initial` for a `creates:`, the transition's `to` for a `moves:`, and the state it was
///   arranged in for an `updates:`. Deciding a filter against the wrong state is a wrong assertion
///   rather than a missing one.
/// * [`Run::instance`] is what the arrangement bound it as, and it is what turns "the view holds a
///   row" into "the view holds *this* one" — see [`identifying`].
/// * [`Run::settled`] is what its fields hold, and it is what turns that into "the view holds this
///   one, carrying what it was given" — see [`shown`].
///
/// A branch with no subject changes nothing a view could show, and produces no steps.
///
/// # A declared order costs further instances
///
/// A view that declares `order_by:` is read twice in the same block, and the second read is the one
/// this function has to arrange for. Synthesis makes at most one instance of an entity per scenario
/// otherwise, and an order over one row holds for every implementation — so where a ranked view
/// would see fewer than [`RANKING_ROWS`], further instances are arranged through the declared
/// creating outcome and the declared moves that reach a state the view's filter admits, each built
/// from its own [`Distinction`]. Where the specification cannot produce them,
/// [`RefusalCause::OrderUnwitnessed`] is recorded and the order is **not** asserted: §36's rule is
/// that a check the specification asked for and did not get is named, and this is the one shape
/// where emitting it anyway would have been silently free.
#[allow(clippy::too_many_lines)]
fn view_expectations(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    run: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> ViewAssertions {
    let mut out = ViewAssertions::default();
    let (Some(subject), Some(state)) = (&outcome.subject, run.after.as_ref()) else {
        return out;
    };
    let (instance, settled) = (run.instance.as_ref(), &run.settled);
    // The row a further run of a creating branch made for its absent-reference witness is named
    // by the capture that run made, not by the first event that published an identity (ess/22,
    // beyond10x/ess#285).
    let absent_run = |captured: &InstanceName| {
        (0..related::absence_points(ir, command, outcome).len()).any(|point| {
            *captured
                == instance_name(
                    &ir.entity(&subject.entity).name,
                    Distinction::further(ABSENT_REFERENCE_WITNESS + point),
                )
        })
    };
    let identity = match (instance, &subject.instance) {
        (Some(captured), ResolvedInstance::Observed { .. }) if absent_run(captured) => {
            Some(ScenarioValue::instance(captured.clone()))
        }
        _ => identity_of(subject, instance),
    };
    let identity = identity.as_ref();
    let projections = row_projections(ir);
    let Some(views) = projections.get(&subject.entity) else {
        return out;
    };

    // Decided first, and for every view, because the companions one ranked view needs land in every
    // other view of the same entity — so what each view holds cannot be settled one view at a time.
    let mut decided: Vec<(&&ResolvedView, bool)> = Vec::new();
    for view in views {
        match shows_row(
            ir,
            view,
            state,
            settled,
            identity,
            &bound(ir, view, settled, identity),
        ) {
            // Neither asserted nor counted: an eventual read of a row left as it was proves nothing.
            Ok(_) if awaits_nothing(command, view, run, state) => {}
            Ok(admits) => decided.push((view, admits)),
            Err(unbound) => refusals.push(Refusal::about(
                id,
                RefusalCause::ViewUndecidable {
                    view: ViewRef::new(view.name.clone()),
                    filter: view
                        .filter
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string),
                    state: state.clone(),
                    unbound,
                },
            )),
        }
    }

    let taken = captured(run);
    let context = Beside {
        ir,
        entity: &subject.entity,
        actors,
        settled,
        identity,
        taken: &taken,
    };
    let (mut companions, unwitnessed) = arrange_ranked(&context, &decided, id, refusals);
    let matching = arrange_matching(&context, &decided, &mut companions);
    for companion in &companions {
        out.arranged.extend(companion.steps.iter().cloned());
        out.source.extend(companion.source.iter().cloned());
    }

    for (view, admits_subject) in &decided {
        let name = ViewRef::new(view.name.clone());
        let fields = identifying(ir, subject, instance, view);
        let expectations = if *admits_subject {
            // Every projected field whose value this scenario determined, beside the identity that
            // says which row. A view whose rows all carry the same wrong value passes `Contains`
            // on the identity alone and passes `Ranked` below trivially; this is the assertion
            // that reads them. It is sound on a target §8 permits to be shared, where a claim
            // about which row comes *first* would not be: a row this scenario did not make cannot
            // stop one it did from holding what it was given.
            subject_row(ir, view, outcome, run, &subject.entity, &fields, state)
        } else {
            // A cancelled invoice that stays in `OutstandingInvoices` is the defect the positive
            // assertion cannot see, and an entity that has not reached the filtered state yet is
            // exactly that case at the other end. Without the identity `Excludes {}` reads "holds
            // no rows", which a companion this scenario put there makes false for every target.
            let params = bound(ir, view, settled, identity);
            let filled = companions
                .iter()
                .any(|companion| companion.shows(ir, view, &params) == Ok(true));
            if fields.is_empty() && filled {
                Vec::new()
            } else {
                vec![ViewExpectation::Excludes { fields }]
            }
        };
        let expectations =
            expectations
                .into_iter()
                .chain(
                    matching
                        .get(&name)
                        .into_iter()
                        .flatten()
                        .map(|(fields, contained)| {
                            let fields = fields.clone();
                            if *contained {
                                ViewExpectation::Contains { fields }
                            } else {
                                ViewExpectation::Excludes { fields }
                            }
                        }),
                );
        let params = bound(ir, view, settled, identity);
        for expectation in expectations {
            require(view, &name, params.clone(), expectation, &mut out.asserted);
        }
        // How many rows this scenario put there, as a floor and never as a ceiling. §8 permits a
        // target to be shared as long as scenarios do not interfere, so a row this scenario did not
        // make is legitimate and cannot take one away — where "exactly this many" would be a claim
        // about every other user of the target, which no specification makes.
        //
        // Only from two, because one is what `Contains` above already says: a floor of one beside
        // it is a second spelling of one claim, and two spellings are two things that can disagree.
        let rows = rows_shown(ir, view, *admits_subject, &companions, &params);
        if rows >= RANKING_ROWS {
            require(
                view,
                &name,
                params.clone(),
                ViewExpectation::Counts {
                    at_least: Some(rows),
                    at_most: None,
                },
                &mut out.asserted,
            );
        }
        // A declared order is a second promise about the same read, asserted in the same block as
        // the first: an `eventual` view that is queried again would be a different read, and two
        // reads can disagree about order without either of them being wrong.
        if !view.order_by.is_empty() && !unwitnessed.contains(&name) {
            require(
                view,
                &name,
                params,
                ViewExpectation::Ranked {
                    order_by: view.order_by.clone(),
                },
                &mut out.asserted,
            );
        }
        out.asserted.extend(paging::page_reads(
            ir,
            view,
            settled,
            identity,
            *admits_subject,
            &companions,
            &unwitnessed,
        ));
        out.views.insert(name);
    }
    out
}

/// Every instance name a run's arrangement and branch captured, and the subject's own: a companion
/// numbered into one of them would be a second instance under one name (adversary pass 2 of
/// story:view-filters-witnessed-on-matching-rows).
fn captured(run: &Run) -> BTreeSet<InstanceName> {
    run.steps()
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::CaptureInstance { instance, .. } => Some(instance.clone()),
            _ => None,
        })
        .chain(run.instance.clone())
        .collect()
}

/// What every further row arranged beside one scenario's subject is arranged against.
struct Beside<'a> {
    ir: &'a EssIr,
    /// The subject's entity, which every further row is an instance of.
    entity: &'a EntityHandle,
    actors: &'a BTreeMap<QualifiedName, ActorRef>,
    /// What the subject's fields hold, which binds each view's parameters.
    settled: &'a BTreeMap<String, Determined>,
    /// The subject's identity, which binds a parameter compared with it.
    identity: Option<&'a ScenarioValue>,
    /// Every instance name the scenario already captured before the view assertions.
    taken: &'a BTreeSet<InstanceName>,
}

impl Beside<'_> {
    /// The lowest further distinction whose instance name neither the scenario nor an arranged
    /// companion holds yet. A subject arranged under a further distinction (beyond10x/ess#161) is
    /// `order-2`, and a companion counted from one would be `order-2` as well.
    fn next(&self, companions: &[Arrangement]) -> Distinction {
        let entity = &self.ir.entity(self.entity).name;
        (1..=self.taken.len() + companions.len() + 1)
            .map(Distinction::further)
            .find(|distinction| {
                let name = instance_name(entity, *distinction);
                !self.taken.contains(&name)
                    && companions
                        .iter()
                        .all(|companion| companion.instance != name)
            })
            .unwrap_or(Distinction::UNKNOWN)
    }
}

/// The further instances every ranked view among `decided` needs to hold [`RANKING_ROWS`] rows, and
/// the ranked views that could not be given them, each refused as `OrderUnwitnessed`.
fn arrange_ranked(
    beside: &Beside<'_>,
    decided: &[(&&ResolvedView, bool)],
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> (Vec<Arrangement>, BTreeSet<ViewRef>) {
    let ir = beside.ir;
    let mut companions: Vec<Arrangement> = Vec::new();
    let mut unwitnessed: BTreeSet<ViewRef> = BTreeSet::new();
    for (view, admits_subject) in decided {
        if view.order_by.is_empty() {
            continue;
        }
        let name = ViewRef::new(view.name.clone());
        // The subject's own binding, and it is the right one: a companion counts towards a ranked
        // read only if the *caller's* parameter admits it. A row in another queue is a row this
        // query never asked for.
        let params = bound(ir, view, beside.settled, beside.identity);
        // A list read by the subject's owner (`account_id == param.account`) holds only rows of that
        // owner, so the further rows are created under it (beyond10x/ess#193) — where the owner
        // may hold many. One that holds one never shows a second row, and the order is refused.
        let read_by_owner = owner_of_row(ir, beside.entity, beside.settled)
            .filter(|(via, _)| reads_root(view, via));
        if let (Some((via, _)), false) = (read_by_owner, shares_owner(ir, beside.entity)) {
            if rows_shown(ir, view, *admits_subject, &companions, &params) < RANKING_ROWS {
                refusals.push(Refusal::about(
                    id,
                    RefusalCause::OrderUnwitnessed {
                        view: name.clone(),
                        entity: EntityRef::from(beside.entity),
                        arranged: rows_shown(ir, view, *admits_subject, &companions, &params),
                        reason: Unreachable::OwnerHoldsOne {
                            via: via.to_owned(),
                        },
                    },
                ));
                unwitnessed.insert(name.clone());
                continue;
            }
        }
        let owner = read_by_owner.map(|(_, owner)| owner);
        while rows_shown(ir, view, *admits_subject, &companions, &params)
            < paging::rows_wanted(view)
        {
            let distinction = beside.next(&companions);
            match arrange_beside(
                ir,
                beside.entity,
                view,
                beside.actors,
                distinction,
                owner,
                &params,
            ) {
                Ok(companion) => companions.push(companion),
                // A paged view's further rows are for its pages; its order is witnessed by two.
                Err(_)
                    if rows_shown(ir, view, *admits_subject, &companions, &params)
                        >= RANKING_ROWS =>
                {
                    break
                }
                Err(reason) => {
                    refusals.push(Refusal::about(
                        id,
                        RefusalCause::OrderUnwitnessed {
                            view: name.clone(),
                            entity: EntityRef::from(beside.entity),
                            arranged: rows_shown(ir, view, *admits_subject, &companions, &params),
                            reason,
                        },
                    ));
                    unwitnessed.insert(name.clone());
                    break;
                }
            }
        }
    }
    (companions, unwitnessed)
}

/// Per view, each further row asserted beside the subject's: the fields that name it, and whether
/// the view holds it.
type Matching = BTreeMap<ViewRef, Vec<(BTreeMap<String, ScenarioValue>, bool)>>;

/// What a further row must be for one of [`arrange_matching`]'s cases.
type Wanted<'w> = Box<dyn Fn(&Arrangement) -> bool + 'w>;

/// Per view, a further row asserted beside the subject's, by the name its arrangement bound and
/// whether the view holds it (story:view-filters-witnessed-on-matching-rows).
///
/// Three cases, all only for a view that projects the identity — without it the subject's
/// `Excludes {}` reads "holds no rows" and a second row cannot be told apart from the first:
///
/// - **The subject's row does not meet the filter.** `Excludes` over it alone passes a target
///   that refuses every row, or applies the filter byte for byte; a row the filter admits is
///   arranged and asserted `Contains`. A filter over the state alone is left as it was, and so is
///   one the plain witness already meets in some state it can be arranged into: then another
///   scenario's own subject is that row, and the committed suites keep their bytes.
/// - **The filter folds case**, whatever the subject's row does. A row on which the filter and the
///   same filter compared byte for byte disagree — `WEB` for `web` — is arranged and asserted as
///   the filter decides it: `Contains` under `equals_ignore_case`, `Excludes` under its negation.
///   The plain witness meets a fold only in the case written, which a byte-wise target decides the
///   same way.
/// - **The filter reads an identity, and holds the subject's row.** `id == param.id` asks for the
///   row this scenario made, and a target that ignores the filter returns every row — which, with
///   the subject the only one, passes. A further row the filter refuses is arranged and asserted
///   `Excludes` (beyond10x/ess#193): for a filter over the identity, one under the subject's own
///   owner where that owner may hold many rows, so the identity alone tells them apart; for a
///   filter over the link to the owner, one under another owner. A filter over both gets both. A
///   view that projects the link and not the identity is named by the link, so the second of these
///   is the one case that does not need the identity projected.
///
/// A companion already arranged serves where it answers the case; a new one is pushed onto
/// `companions`, so it lands in every view of the entity and is counted as every companion is. A
/// view no arrangement reaches keeps what it had.
#[allow(clippy::too_many_lines)]
fn arrange_matching(
    beside: &Beside<'_>,
    decided: &[(&&ResolvedView, bool)],
    companions: &mut Vec<Arrangement>,
) -> Matching {
    let ir = beside.ir;
    let mut matching = Matching::new();
    let identity_name = &ir.entity(beside.entity).identity.name;
    let owner = owner_of_row(ir, beside.entity, beside.settled);
    for (view, admits_subject) in decided {
        let identified = view.field(identity_name).is_some();
        // The link to the subject's owner where the filter reads it: a row under another owner is
        // told apart by it, and the view names that row by the link where it projects no identity
        // (beyond10x/ess#193).
        let linked = owner.filter(|(via, _)| reads_root(view, via));
        let link_told = linked.is_some_and(|(via, _)| view.field(via).is_some());
        if !(identified || link_told) || !reads_row_fields(view) {
            continue;
        }
        let params = &bound(ir, view, beside.settled, beside.identity);
        let bytewise = view
            .filter
            .as_ref()
            .filter(|filter| filter.uses_case_fold())
            .map(|filter| ResolvedView {
                filter: Some(byte_exact(filter)),
                ..(**view).clone()
            });
        let held = |row: &Arrangement| row.shows(ir, view, params);
        let under = |row: &Arrangement, via: &str, owner: &InstanceName| {
            row.settled.get(via).map(|held| &held.value)
                == Some(&ScenarioValue::instance(owner.clone()))
        };
        let mut wanted: Vec<(Wanted<'_>, Option<&InstanceName>)> = Vec::new();
        let by_identity = identified && reads_root(view, identity_name);
        let by_copy = identified
            && !singleton::is_singleton(ir, beside.entity)
            && ir.drivers().get(beside.entity).is_some_and(|drivers| {
                drivers
                    .iter()
                    .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
                    .flat_map(|driver| &driver.outcome.sets)
                    .any(|set| {
                        matches!(set.value, ResolvedPayloadValue::RelatedField { .. })
                            && reads_root(view, &set.target)
                    })
            });
        match &bytewise {
            Some(bytewise) if identified => wanted.push((
                Box::new(move |row: &Arrangement| {
                    let bytes = row.shows(ir, bytewise, params);
                    matches!((held(row), bytes), (Ok(folded), Ok(exact)) if folded != exact)
                }),
                None,
            )),
            Some(_) => continue,
            None if *admits_subject && (by_identity || linked.is_some()) => {
                // The filter names the subject's row by an identity. A further row it refuses is
                // asserted `Excludes`:
                //
                // - where it reads the row's identity, under the subject's own owner, so that the
                //   identity is all that tells the two apart and a by-id read answering the
                //   owner's rows fails. Only where the owner may hold many rows; one that holds
                //   one is never given a second, and the row goes under an owner of its own;
                // - where it reads the link, under another owner, so that a read ignoring the
                //   owner fails.
                if by_identity {
                    let shared = owner.filter(|_| shares_owner(ir, beside.entity));
                    wanted.push((
                        Box::new(move |row: &Arrangement| {
                            held(row) == Ok(false)
                                && shared.is_none_or(|(via, owner)| under(row, via, owner))
                        }),
                        shared.map(|(_, owner)| owner),
                    ));
                }
                if let Some((via, owner)) = linked.filter(|_| link_told || identified) {
                    wanted.push((
                        Box::new(move |row: &Arrangement| {
                            held(row) == Ok(false) && !under(row, via, owner)
                        }),
                        None,
                    ));
                }
            }
            None if *admits_subject && by_copy => wanted.push((
                Box::new(move |row: &Arrangement| held(row) == Ok(false)),
                None,
            )),
            None if !identified
                || *admits_subject
                || plain_row_shown(ir, beside.entity, view, beside.actors, params) =>
            {
                continue;
            }
            None => wanted.push((
                Box::new(move |row: &Arrangement| held(row) == Ok(true)),
                None,
            )),
        }
        let mut taken: Vec<usize> = Vec::new();
        for (accept, sibling) in &wanted {
            let at = if let Some(at) = companions.iter().position(accept.as_ref()) {
                at
            } else {
                let distinction = beside.next(companions);
                let Some(companion) = arrange_toward_by(
                    ir,
                    beside.entity,
                    view,
                    beside.actors,
                    distinction,
                    *sibling,
                    accept.as_ref(),
                ) else {
                    continue;
                };
                companions.push(companion);
                companions.len() - 1
            };
            if taken.contains(&at) {
                continue;
            }
            taken.push(at);
            let row = &companions[at];
            let fields = if identified {
                identifying_arranged(ir, beside.entity, &row.instance, view)
            } else {
                linked
                    .and_then(|(via, _)| {
                        row.settled
                            .get(via)
                            .map(|held| (via.to_owned(), held.value.clone()))
                    })
                    .into_iter()
                    .collect()
            };
            matching
                .entry(ViewRef::new(view.name.clone()))
                .or_default()
                .push((fields, held(row) == Ok(true)));
        }
    }
    matching
}

/// Whether a view's filter reads the row field `field`.
fn reads_root(view: &ResolvedView, field: &str) -> bool {
    view.filter.as_ref().is_some_and(|filter| {
        filter
            .fact_paths()
            .into_iter()
            .any(|path| path.namespace() == field)
    })
}

/// Whether a view's filter reads a field of the row: anything but its state and the caller's
/// parameters.
fn reads_row_fields(view: &ResolvedView) -> bool {
    view.filter.as_ref().is_some_and(|filter| {
        filter.fact_paths().into_iter().any(|path| {
            let namespace = path.namespace();
            namespace != EntitySpec::STATE && namespace != ess_domain::view::ViewSpec::PARAM
        })
    })
}

/// Whether the plain witness, arranged into some state of the entity's lifecycle, leaves a row the
/// view's filter admits — the row some scenario's own subject already is.
fn plain_row_shown(
    ir: &EssIr,
    entity: &EntityHandle,
    view: &ResolvedView,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    params: &BTreeMap<String, ScenarioValue>,
) -> bool {
    ir.entity(entity).lifecycle.states.iter().any(|state| {
        arrange(ir, entity, state, actors, Distinction::PLAIN, &[])
            .is_ok_and(|arranged| arranged.shows(ir, view, params) == Ok(true))
    })
}

/// The field match naming an instance an arrangement created and bound as `instance`, where the
/// view projects the entity's identity; empty where it does not.
fn identifying_arranged(
    ir: &EssIr,
    entity: &EntityHandle,
    instance: &InstanceName,
    view: &ResolvedView,
) -> BTreeMap<String, ScenarioValue> {
    let named = ir.entity(entity).identity.name.clone();
    if view.field(&named).is_none() {
        return BTreeMap::new();
    }
    [(named, ScenarioValue::instance(instance.clone()))]
        .into_iter()
        .collect()
}

/// Whether an `eventually` block about `view` after this branch would wait for nothing: the view is
/// `eventual`, and the branch of a command reading stored fields left the arranged row in the state
/// and with every value the view projects as it was, so a projection that never saw the command
/// shows the same row (beyond10x/ess#172). Only for such commands: every other command keeps the
/// suites, and the committed generated ones, it had.
fn awaits_nothing(
    command: &ResolvedCommand,
    view: &ResolvedView,
    run: &Run,
    state: &StateName,
) -> bool {
    subject_fact::uses(command)
        && view.assertion_style == AssertionStyle::Eventually
        && run.before.as_ref() == Some(state)
        && view.fields.iter().all(|field| {
            run.before_settled.get(&field.name).map(|held| &held.value)
                == run.settled.get(&field.name).map(|held| &held.value)
        })
}

/// The value an `ess/14` source leaves in `field`, where the scenario determines it
/// (`docs/design/value-expressions.md`).
///
/// `before` is what the subject's fields held before this act, as the arrangement settled them.
/// `None` is "not determined": a payload field is then covered by its shape alone, and a `sets:`
/// target stops being a claim about the row.
///
/// | source | determined as |
/// |---|---|
/// | `{subject: f}` | what `before` holds for `f` |
/// | `{increment: n}` | `before`'s number for the target plus `n`, exactly |
/// | `{input: f, else: …}` | what the invocation sent for `f`; else the `else:` literal, or nothing |
/// | nested mapping | the struct, where every leaf is a determined literal |
fn expression_value(
    ir: &EssIr,
    field: &ess_compiler::ir::ResolvedPayloadField,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> Option<ScenarioValue> {
    expression_value_at(
        ir,
        field,
        supplied,
        before,
        std::slice::from_ref(&field.target),
    )
}

fn expression_value_at(
    ir: &EssIr,
    field: &ess_compiler::ir::ResolvedPayloadField,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
    target_location: &[String],
) -> Option<ScenarioValue> {
    match &field.value {
        ResolvedPayloadValue::SubjectField { field: read, .. } => {
            before.get(read).map(|held| held.value.clone())
        }
        // ess/16 (#166): the referenced row's value, which `related::arrange` settled — absent
        // where a reference it follows was left absent (ess/22, beyond10x/ess#285).
        ResolvedPayloadValue::RelatedField {
            via,
            through,
            field: read,
            ..
        } => before
            .get(&related::key(via, through, read))
            .map(|held| held.value.clone()),
        ResolvedPayloadValue::Increment { by } => {
            let Node::Number(held) = before_literal_at(before, target_location)? else {
                return None;
            };
            let by = ess_primitives::facts::Number::decimal_literal(by)?;
            held.checked_add(by).map(|sum| ScenarioValue::Literal {
                value: Node::Number(sum),
            })
        }
        // An omitted input leaves the fallback: nothing determined for `{generated: true}`, and the
        // literal read as the target's type for `else: <literal>` (ess/16, #163), so an
        // implementation storing any other default fails the scenario.
        ResolvedPayloadValue::InputOrGenerated {
            field: read,
            otherwise,
            ..
        } => match supplied.get(read) {
            None | Some(ScenarioValue::Literal { value: Node::Null }) => {
                let written = otherwise.as_deref()?;
                literal_value(ir, &field.target_type, written, 0)
                    .map(|value| ScenarioValue::Literal { value })
            }
            Some(value @ ScenarioValue::Literal { .. }) => Some(value.clone()),
            Some(_) => None,
        },
        ResolvedPayloadValue::Struct { fields } => {
            let mut leaves = BTreeMap::new();
            for leaf in fields {
                let mut location = target_location.to_vec();
                location.push(leaf.target.clone());
                leaves.insert(
                    leaf.target.clone(),
                    leaf_value(ir, leaf, supplied, before, &location)?,
                );
            }
            Some(ScenarioValue::Literal {
                value: Node::Map(leaves),
            })
        }
        ResolvedPayloadValue::Literal { .. }
        | ResolvedPayloadValue::InputField { .. }
        | ResolvedPayloadValue::ResponseField { .. }
        | ResolvedPayloadValue::Generated
        | ResolvedPayloadValue::Cleared
        // ess/16 (#168): read only through `caller::synthesize`, which writes the caller's value in.
        | ResolvedPayloadValue::CallerAttribute { .. }
        | ResolvedPayloadValue::ChangedCount => None,
    }
}

fn before_literal_at<'a>(
    before: &'a BTreeMap<String, Determined>,
    location: &[String],
) -> Option<&'a Node> {
    let (root, remaining) = location.split_first()?;
    let ScenarioValue::Literal { value } = &before.get(root)?.value else {
        return None;
    };
    remaining.iter().try_fold(value, |node, member| {
        let Node::Map(fields) = node else {
            return None;
        };
        fields.get(member)
    })
}

/// [`arranged`], or [`arranged_without_fallbacks`], for the invocation `witness` names.
fn arranged_as(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    routed: bool,
    witness: Witness,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    match witness {
        Witness::Full => arranged(ir, command, outcome, actors, routed),
        Witness::LiteralFallbacks => arranged_without_fallbacks(ir, command, outcome, actors),
        Witness::Listed(nth) => arranged_in_listed_state(ir, command, outcome, actors, nth),
        Witness::RelatedValueAbsent(point) => {
            arranged_with_absent_reference(ir, command, outcome, actors, point)
        }
        // Only a command reading a related row builds one, and `run_as` arranges it there.
        Witness::RelatedBoundary { .. } | Witness::RelatedAbsent(_) => {
            Err(related_guard::unarranged())
        }
    }
}

/// The arrangement and input of [`Witness::LiteralFallbacks`]: a further instance, under a
/// distinction no other arrangement of the scenario uses so its rows and identities are its own,
/// and the input with [`without_literal_fallbacks`] applied.
///
/// Only for a branch its input selects: a guard over the held state or the stored row is arranged
/// by searches that fix the plain witness, whose second arrangement would repeat the first one's
/// identities. Such a branch keeps its full invocation alone.
fn arranged_without_fallbacks(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    if subject_fact::routes(command, outcome) || has_subject_guards(command) {
        return Err(no_literal_fallback_run(command));
    }
    let distinction = Distinction::further(FRESH_WITNESSES + 1);
    let setup = prepare_in(ir, outcome, actors, None, distinction)?;
    let input = reach(ir, command, outcome, distinction)?;
    let input = existence::fresh_created(ir, command, outcome, input, true)?;
    let input = freshened(
        ir,
        command,
        outcome,
        input,
        setup.before.as_ref(),
        &setup.settled,
    );
    let omitted = without_literal_fallbacks(ir, command, outcome, &setup, input.clone());
    if omitted == input {
        return Err(no_literal_fallback_run(command));
    }
    Ok((setup, omitted))
}

/// The arrangement and input of [`Witness::Listed`]: a further instance, under a distinction of its
/// own, arranged in and observed at the `nth` state the branch's listed guard names, with an input
/// the branch is selected by there.
fn arranged_in_listed_state(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    nth: usize,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let without = || RefusalCause::StrategyWithoutGuard {
        strategy: outcome.test_strategy,
    };
    let ResolvedCondition::SubjectState { state, .. } = &outcome.condition else {
        return Err(without());
    };
    let held = state.iter().nth(nth).ok_or_else(without)?;
    let distinction = Distinction::further(FRESH_WITNESSES + 2 + nth);
    let input = reach_in_state(ir, command, outcome, held, distinction)?;
    let mut setup = prepare_in(ir, outcome, actors, Some(held), distinction)?;
    let instance = setup.instance.clone().ok_or_else(without)?;
    let (observed, view) = observe_subject_state(ir, outcome, &instance, held)?;
    setup.steps.extend(observed);
    setup.source.insert(view.into());
    Ok((setup, input))
}

/// The distinction the first absent-reference witness is arranged under, and the next ones one
/// apart (ess/22, beyond10x/ess#285): past every further instance a candidate search numbers, and
/// past the literal-fallback and listed-state runs, so their rows and identities are their own.
const ABSENT_REFERENCE_WITNESS: usize = 2 * crate::witness::MAX_CANDIDATES + 1;

/// The arrangement and input of [`Witness::RelatedValueAbsent`] (ess/22, beyond10x/ess#285): a
/// further instance with the `point`th reference its `{related: …}` sources follow that may be
/// absent left absent and every other present ([`related::arrange_absent`]), and the input it
/// leaves out removed.
///
/// Only for a branch its input selects, as for [`Witness::LiteralFallbacks`]: a guard over the
/// held state or the stored row is arranged by searches that fix the plain witness.
fn arranged_with_absent_reference(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    point: usize,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let unarranged = |reason: &str| RefusalCause::AbsenceUnwitnessed {
        reference: related::point_reference(ir, outcome, point),
        reason: reason.to_owned(),
    };
    if subject_fact::routes(command, outcome) || has_subject_guards(command) {
        return Err(unarranged(
            "the branch is chosen by its subject's stored fields, whose search fixes the plain \
             witness and is not run again for a further instance",
        ));
    }
    let points = related::absence_points(ir, command, outcome);
    let Some(&at) = points.get(point) else {
        return Err(unarranged("it is no reference the branch's reads follow"));
    };
    let distinction = Distinction::further(ABSENT_REFERENCE_WITNESS + point);
    let setup = prepare_subject(ir, outcome, actors, None, distinction)?;
    let Some((setup, omitted)) =
        related::arrange_absent(ir, command, outcome, actors, (distinction, at), setup)?
    else {
        return Err(unarranged(
            "the arrangement points it at a row it needs, or the row that stores it was created \
             by a branch that fills it from no Optional input",
        ));
    };
    let input = reach(ir, command, outcome, distinction)?;
    let input = existence::fresh_created(ir, command, outcome, input, true)?;
    let mut input = freshened(
        ir,
        command,
        outcome,
        input,
        setup.before.as_ref(),
        &setup.settled,
    );
    for field in &omitted {
        input.remove(field);
    }
    if !subject_fact::input_selects(ir, command, outcome, &input).unwrap_or(false) {
        return Err(unarranged("the branch is not selected once it is left out"));
    }
    Ok((setup, input))
}

/// Why [`Witness::LiteralFallbacks`] builds nothing. Never reported: the full invocation is the
/// scenario, and this run only adds to it.
fn no_literal_fallback_run(command: &ResolvedCommand) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path: command.name.to_string(),
        type_ref: "optional input".into(),
        reason: "no optional input can be left out for an `else:` literal",
    })
}

/// The input with every optional field left out that this outcome reads only through
/// `{input: f, else: <literal>}` (ess/16, #163), so the scenario asserts the literal.
///
/// A witness sends every optional input it can, so without this no scenario reached the fallback
/// and an implementation storing any other default passed. A field is kept when anything else in
/// the outcome needs it sent — a plain `input.f` into a required target or through a conversion,
/// a `{generated: true}` fallback, the subject's identity, a fixture, an owner the arrangement bound — or when
/// the input no longer selects the branch without it, so a guard that reads the field still
/// decides it. Each field is decided on its own, so one a guard needs does not keep the rest.
fn without_literal_fallbacks(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    setup: &Setup,
    input: BTreeMap<String, Node>,
) -> BTreeMap<String, Node> {
    /// How one value of the outcome reads an input field.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Read {
        /// `{input: f, else: <literal>}`: the literal stands in for an omitted input.
        Fallback,
        /// A plain `input.f` copied, unconverted, into an `Optional` target: an omitted input
        /// leaves that target absent, which the omission run then asserts.
        Nullable,
        /// Anything that needs the value sent: a required target, a conversion, a
        /// `{generated: true}` fallback.
        Needed,
    }
    fn reads<'a>(
        target: &'a ess_compiler::ir::ResolvedPayloadField,
        out: &mut Vec<(&'a str, Read)>,
    ) {
        match &target.value {
            ResolvedPayloadValue::InputOrGenerated {
                field, otherwise, ..
            } => out.push((
                field,
                if otherwise.is_some() {
                    Read::Fallback
                } else {
                    Read::Needed
                },
            )),
            ResolvedPayloadValue::InputField { field, .. } => out.push((
                field,
                if target.target_type.is_optional() && target.conversion.is_none() {
                    Read::Nullable
                } else {
                    Read::Needed
                },
            )),
            ResolvedPayloadValue::RelatedField {
                via: ess_compiler::ir::ResolvedRelatedVia::Input { field, .. },
                ..
            } => out.push((field, Read::Needed)),
            ResolvedPayloadValue::Struct { fields } => {
                for leaf in fields {
                    reads(leaf, out);
                }
            }
            ResolvedPayloadValue::Literal { .. }
            | ResolvedPayloadValue::ResponseField { .. }
            | ResolvedPayloadValue::Generated
            | ResolvedPayloadValue::Cleared
            | ResolvedPayloadValue::SubjectField { .. }
            | ResolvedPayloadValue::RelatedField { .. }
            | ResolvedPayloadValue::Increment { .. }
            | ResolvedPayloadValue::CallerAttribute { .. }
            | ResolvedPayloadValue::ChangedCount => {}
        }
    }
    let (held, bound) = (setup.before.as_ref(), &setup.bound);
    let mut read = Vec::new();
    for field in outcome
        .payload
        .iter()
        .flat_map(|payload| &payload.fields)
        .chain(&outcome.sets)
    {
        reads(field, &mut read);
    }
    let identity = outcome
        .subject
        .as_ref()
        .and_then(|subject| match &subject.instance {
            ResolvedInstance::Supplied { field } => Some(field.name.as_str()),
            ResolvedInstance::Observed { .. } => None,
        });
    let omitted: BTreeSet<&str> = read
        .iter()
        .filter(|(_, how)| *how == Read::Fallback)
        .map(|(field, _)| *field)
        .filter(|field| {
            read.iter()
                .all(|(other, how)| other != field || *how != Read::Needed)
                && identity != Some(*field)
                && !command.fixture_inputs.contains_key(*field)
                && !bound.contains_key(*field)
                && command
                    .input
                    .iter()
                    .any(|declared| declared.name == *field && declared.type_ref.is_optional())
        })
        .collect();
    let mut input = input;
    for field in omitted {
        let mut trimmed = input.clone();
        if trimmed.remove(field).is_none() {
            continue;
        }
        if admitted(ir, command, &trimmed)
            && selects_branch(ir, command, outcome, held, &trimmed).unwrap_or(false)
        {
            input = trimmed;
        }
    }
    input
}

/// The literal one leaf of a nested mapping determines, where the scenario knows it; `None` where
/// it crosses a conversion or its source determines nothing here.
fn leaf_value(
    ir: &EssIr,
    leaf: &ess_compiler::ir::ResolvedPayloadField,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
    target_location: &[String],
) -> Option<Node> {
    if leaf.conversion.is_some() {
        return None;
    }
    match &leaf.value {
        ResolvedPayloadValue::Literal { value } => literal_value(ir, &leaf.target_type, value, 0),
        ResolvedPayloadValue::InputField { field: read, .. } => match supplied.get(read) {
            Some(ScenarioValue::Literal { value }) => Some(value.clone()),
            _ => None,
        },
        _ => match expression_value_at(ir, leaf, supplied, before, target_location)? {
            ScenarioValue::Literal { value } => Some(value),
            _ => None,
        },
    }
}

/// Every determined leaf of a nested mapping whose struct is **not** determined as a whole, keyed
/// by its dotted path under `prefix` (beyond10x/ess#179, `docs/design/value-expressions.md` E5).
///
/// Empty for any other source, and for a struct [`expression_value`] determines whole: that one
/// keeps being asserted as one value, so a suite without an undetermined leaf keeps its bytes. A
/// leaf that is itself a nested mapping is walked the same way. The undetermined leaves are left
/// to the payload shape, which checks their presence and type under the same paths.
fn determined_leaves(
    ir: &EssIr,
    field: &ess_compiler::ir::ResolvedPayloadField,
    prefix: &str,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> BTreeMap<String, Node> {
    let mut out = BTreeMap::new();
    if matches!(field.value, ResolvedPayloadValue::Struct { .. })
        && field.conversion.is_none()
        && expression_value(ir, field, supplied, before).is_none()
    {
        collect_leaves(
            ir,
            field,
            prefix,
            supplied,
            before,
            std::slice::from_ref(&field.target),
            &mut out,
        );
    }
    out
}

/// Every determined scalar leaf under a nested mapping, by dotted path — always down to the leaf,
/// because those are the paths a payload shape names.
fn collect_leaves(
    ir: &EssIr,
    field: &ess_compiler::ir::ResolvedPayloadField,
    prefix: &str,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
    target_location: &[String],
    out: &mut BTreeMap<String, Node>,
) {
    let ResolvedPayloadValue::Struct { fields } = &field.value else {
        return;
    };
    for leaf in fields {
        let path = format!("{prefix}.{}", leaf.target);
        let mut location = target_location.to_vec();
        location.push(leaf.target.clone());
        if matches!(leaf.value, ResolvedPayloadValue::Struct { .. }) {
            if leaf.conversion.is_none() {
                collect_leaves(ir, leaf, &path, supplied, before, &location, out);
            }
        } else if let Some(value) = leaf_value(ir, leaf, supplied, before, &location) {
            flatten_leaf(ir, &leaf.target_type, &path, &value, 0, out);
        }
    }
}

/// One determined value, split into the scalar leaves [`describe`] names for its declared type.
///
/// A struct-typed leaf read whole — `place: input.place` — holds a map, and a payload shape has no
/// leaf at `lead.place`, only `lead.place.city` and `lead.place.code`; so the map is walked by the
/// declared fields, as `describe` walks the type. Where the value does not follow the type — an
/// absent `Optional` struct, a field the map does not carry, a non-map where a struct is declared
/// — that part is left unasserted: a weaker claim, never one the shape contradicts.
fn flatten_leaf(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    path: &str,
    value: &Node,
    depth: usize,
    out: &mut BTreeMap<String, Node>,
) {
    if depth > MAX_TYPE_DEPTH {
        return;
    }
    let body = match type_ref {
        ResolvedTypeRef::Optional { of } => {
            return flatten_leaf(ir, of, path, value, depth + 1, out);
        }
        ResolvedTypeRef::Declared { name } => &ir.named_type(name).body,
        ResolvedTypeRef::Primitive { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => {
            out.insert(path.to_owned(), value.clone());
            return;
        }
    };
    match body {
        ResolvedBody::Newtype { of, .. } => flatten_leaf(ir, of, path, value, depth + 1, out),
        ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => {
            out.insert(path.to_owned(), value.clone());
        }
        ResolvedBody::Struct { fields, .. } => {
            let Node::Map(entries) = value else {
                return;
            };
            for field in fields {
                if let Some(entry) = entries.get(&field.name) {
                    let nested = format!("{path}.{}", field.name);
                    flatten_leaf(ir, &field.type_ref, &nested, entry, depth + 1, out);
                }
            }
        }
    }
}

/// The determined leaves of every partly determined struct this branch's `sets:` writes, where
/// the view projects that field at the entity's own type — the row counterpart of
/// [`determined_leaves`] in a payload (beyond10x/ess#179).
///
/// Read from the branch's own `sets:` rather than from [`settled`]: a dotted path is no entity
/// field, and every reader of `settled` looks fields up by name. A field the row already asserts
/// whole keeps that value.
///
/// Returns the paths of the same structs' **undetermined** leaves that the declared type says are
/// always there — a `{generated: true}` `rank: Integer` — for [`present_leaves`]: a view row
/// carries no payload shape, so their presence is claimed separately.
fn shown_leaves(
    ir: &EssIr,
    view: &ResolvedView,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
    row: &mut BTreeMap<String, ScenarioValue>,
) -> Vec<String> {
    let mut required = Vec::new();
    for field in &outcome.sets {
        if row.contains_key(&field.target)
            || view.field(&field.target).map(|shown| &shown.type_ref) != Some(&field.target_type)
        {
            continue;
        }
        let determined = determined_leaves(ir, field, &field.target, supplied, before);
        if determined.is_empty() {
            continue;
        }
        for (path, value) in determined {
            row.entry(path)
                .or_insert_with(|| ScenarioValue::literal(value));
        }
        undetermined_leaves(
            ir,
            field,
            &field.target,
            supplied,
            before,
            std::slice::from_ref(&field.target),
            &mut required,
        );
    }
    required
}

/// The dotted paths of a nested mapping's leaves that determine no value here and whose declared
/// type is never absent: not `Optional` (through newtypes), and not `Json`, whose `null` is a value.
fn undetermined_leaves(
    ir: &EssIr,
    field: &ess_compiler::ir::ResolvedPayloadField,
    prefix: &str,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
    target_location: &[String],
    out: &mut Vec<String>,
) {
    let ResolvedPayloadValue::Struct { fields } = &field.value else {
        return;
    };
    for leaf in fields {
        let path = format!("{prefix}.{}", leaf.target);
        let mut location = target_location.to_vec();
        location.push(leaf.target.clone());
        if matches!(leaf.value, ResolvedPayloadValue::Struct { .. }) {
            if leaf.conversion.is_none() {
                undetermined_leaves(ir, leaf, &path, supplied, before, &location, out);
            }
        } else if leaf_value(ir, leaf, supplied, before, &location).is_none()
            && !may_be_null(ir, &leaf.target_type, 0)
        {
            out.push(path);
        }
    }
}

/// Whether a value of this type may be `null` or absent in a conforming row.
fn may_be_null(ir: &EssIr, type_ref: &ResolvedTypeRef, depth: usize) -> bool {
    if depth > MAX_TYPE_DEPTH {
        return true;
    }
    match type_ref {
        ResolvedTypeRef::Optional { .. } => true,
        ResolvedTypeRef::Primitive { name } => *name == ess_domain::types::Primitive::Json,
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => may_be_null(ir, of, depth + 1),
            _ => false,
        },
        ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => false,
    }
}

/// The row a view that admits the branch's subject is required to hold, followed by the presence
/// claims for the undetermined leaves of its partly determined structs ([`present_leaves`]).
fn subject_row(
    ir: &EssIr,
    view: &ResolvedView,
    outcome: &ResolvedOutcome,
    run: &Run,
    entity: &EntityHandle,
    identity: &BTreeMap<String, ScenarioValue>,
    state: &StateName,
) -> Vec<ViewExpectation> {
    let mut row = shown(view, identity.clone(), &run.settled);
    let required = shown_leaves(ir, view, outcome, &run.input, &run.before_settled, &mut row);
    row.extend(lifecycle_state(ir, entity, view, state));
    let mut expectations = vec![ViewExpectation::Contains { fields: row }];
    expectations.extend(present_leaves(identity, required));
    expectations
}

/// One `Excludes` per always-present undetermined leaf: no row of **this** subject holds that leaf
/// null or absent (beyond10x/ess#179). The runner reads an absent dotted path as `null`, so a row
/// that never wrote `lead.rank` matches and fails. Nothing when the view does not project the
/// identity: without it the claim would be about every row.
///
/// The leaf's type is not claimed: no view expectation can say "holds an `Integer`". The event
/// payload's shape does; the row's type of a generated leaf is not asserted.
fn present_leaves(
    identity: &BTreeMap<String, ScenarioValue>,
    required: Vec<String>,
) -> Vec<ViewExpectation> {
    if identity.is_empty() {
        return Vec::new();
    }
    required
        .into_iter()
        .map(|path| {
            let mut fields = identity.clone();
            fields.insert(path, ScenarioValue::literal(Node::Null));
            ViewExpectation::Excludes { fields }
        })
        .collect()
}

/// What one invoked branch leaves in the entity's fields, read against what it was supplied.
///
/// The `sets:` block names the field and where its value comes from; `supplied` is what this
/// invocation actually sent. Together they say what the row will hold, which is the relation
/// nothing in the model carried before `sets:` existed.
///
/// A `{cleared: true}` entry is determined and is the one entry with no value to read: the field
/// holds nothing after this branch, which is a claim a row can be checked against, so it is carried
/// as a null. `ess-domain` has already refused it on a field whose type is not `Optional<…>`.
///
/// A **literal** entry is determined exactly where the literal *is* the value — where the target's
/// representation is text or the variants of an enum, both of which are carried as the text the
/// document wrote, so there is nothing to read the literal *as*. `lane_id: ""` over a
/// `String`-backed `LaneId` is the empty string and nothing else, and dropping it left the suite
/// requiring the value an earlier act supplied: measured 2026-09-16 on an adopter's model, where a
/// branch clearing a campaign id produced two scenarios no implementation could pass. See
/// [`spelled_as_text`] for where the reading stops.
///
/// Three kinds of entry are left out rather than guessed at, and each is a thing the model does
/// not determine:
///
/// * a field whose source is a literal the target's representation cannot be written as, where
///   reading the text as the declared type would be a parse no declaration specifies. `ess-domain`
///   refuses such a literal where it is written, so this is a floor and not a working case — see
///   [`spelled_as_text`];
/// * a field that crosses a declared **conversion** — the conversion says the two types may meet
///   and nothing says what it computes, so the value the row holds is not the value sent;
/// * a field the arrangement did not choose a **literal** for — an identity captured from an
///   earlier step is a name for a value rather than the value, and a row carrying it is a claim
///   two reads have to agree on before it says anything about this one.
///
/// The entity field's type is carried out beside the value, because the fourth exclusion needs a
/// view in hand — see [`shown`].
fn settled(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
) -> BTreeMap<String, Determined> {
    let mut out = BTreeMap::new();
    for field in &outcome.sets {
        if field.conversion.is_some() {
            continue;
        }
        let value = match &field.value {
            ResolvedPayloadValue::Cleared => ScenarioValue::Literal { value: Node::Null },
            ResolvedPayloadValue::Literal { value } => {
                let Some(read) = literal_value(ir, &field.target_type, value, 0) else {
                    continue;
                };
                ScenarioValue::Literal { value: read }
            }
            // Whatever the invocation supplied, which is a literal for almost every field and a
            // reference to an arranged row for the two that name one — the branch's subject, and
            // the field carrying its owner. A reference is the *stronger* claim of the two: the row
            // holds the id of the account this scenario created, and asserting that is how a
            // generated suite catches an implementation that files the new row under a different
            // owner. Restricting this to literals dropped the field instead, which asserted nothing
            // about it at all.
            ResolvedPayloadValue::InputField { field: read, .. } => match supplied.get(read) {
                Some(value) => value.clone(),
                // An `Optional` input the invocation left out — absent on the wire, never `null`
                // — leaves the `Optional` field it fills absent, which a row carries as a null
                // exactly as `{cleared: true}` does. It is the one way a scenario can arrange a
                // row that `not defined(field)` selects.
                None if field.target_type.is_optional() => {
                    ScenarioValue::Literal { value: Node::Null }
                }
                None => continue,
            },
            // ess/14 (`docs/design/value-expressions.md`): read against the row before this act.
            ResolvedPayloadValue::SubjectField { .. }
            | ResolvedPayloadValue::Increment { .. }
            | ResolvedPayloadValue::InputOrGenerated { .. }
            | ResolvedPayloadValue::Struct { .. }
            | ResolvedPayloadValue::RelatedField { .. }
            | ResolvedPayloadValue::CallerAttribute { .. } => {
                match expression_value(ir, field, supplied, before) {
                    Some(value) => value,
                    None => continue,
                }
            }
            ResolvedPayloadValue::Generated
            | ResolvedPayloadValue::ResponseField { .. }
            | ResolvedPayloadValue::ChangedCount => continue,
        };
        out.insert(
            field.target.clone(),
            Determined {
                value,
                type_ref: field.target_type.clone(),
            },
        );
    }
    out
}

/// Folds what one act determined into what the acts before it left, invalidating first.
///
/// Every field this act WROTE stops being a claim about the row, whether or not this act determined
/// a value in its place. [`settled`] abstains on a field that crosses a declared **conversion** —
/// the conversion says two types may meet and not what it computes, so the row does not end up
/// holding the value that was sent — and abstaining is a statement about THIS act; leaving the older
/// determination standing turns it into a claim about the row, and the opposite claim. Measured 2026-09-16 on an adopter's model: a `leave` branch
/// writing `campaign_id: ""` produced a suite that went on requiring the campaign id the creating
/// act had supplied, so the scenario failed against an implementation that cleared the field exactly
/// as the specification said to.
///
/// One function rather than the rule restated at each accumulation, because it has to hold at all of
/// them and it did not: [`run`] applied it to the branch under test while [`arrange`] extended
/// straight over the route that reaches it, so a two-command arrangement whose second command
/// overwrote what the first determined kept the first act's value — the same defect one act earlier.
///
/// **Whoever comes to simplify this: the stale claim was INVERTED, not merely unproven.** Measured
/// 2026-09-16 with `ess verify conform synthesize` on a two-command arrangement whose second command
/// wrote a Boolean field the suite could not read: the suite demanded `true` of that row — the
/// witness the *first* command was handed — while the branch that ran last had written the opposite.
/// So it required the negation of what the specification's final write said. An unproven field is a
/// weak test; this was a test that could only pass against an implementation which ignored the
/// branch. The removal has to happen for every act, not only the one under test, and that is why
/// this is a function and not two lines in the caller.
fn absorb(
    settled: &mut BTreeMap<String, Determined>,
    outcome: &ResolvedOutcome,
    determined: BTreeMap<String, Determined>,
) {
    for field in &outcome.sets {
        settled.remove(&field.target);
    }
    settled.extend(determined);
}

/// How many further witnesses an update's `sets:` input is tried at before the plain one is kept.
///
/// Three: a further [`Distinction`] moves every leaf inside its own type, and a two-variant enum
/// or a `Boolean` has its other value at the first. A type with one value has no other, and the
/// plain witness stands — the row cannot be told apart from an unwritten one, and no value can.
const FRESH_WITNESSES: usize = 3;

/// The branch's input, with every field its `sets:` writes moved off the value the row already
/// holds (beyond10x/ess#111).
///
/// An update witnessed at the value its arrangement wrote proves nothing about the write: the
/// setup and the update drew from one witness, so `Relabel` wrote `tier: Low` over `tier: Low`, and
/// an implementation that dropped the entry from `sets:` still showed the row the view expected.
/// So for a branch that acts on a row which already exists — `updates:` and `moves:` — each field
/// a `sets:` entry fills from an input is compared with what the row holds before it runs:
///
/// | the row holds | from |
/// |---|---|
/// | the literal an arranging act settled | [`Run::settled`], the arrangement's |
/// | an enum's first variant, where no act wrote the field | the only value nothing declares and every target's default lands on |
///
/// Where they are equal, the input takes the value a further witness gives that field, and the
/// branch is decided again with it by [`selects_branch`]; the first further witness that differs
/// and still reaches the branch is kept. The model declares no initial value for an entity field,
/// so the first variant is the one reading of "what an unwritten enum holds" a scenario can make
/// without inventing one; every other type's plain witness (`1`, `true`, the path's text) is
/// already away from its zero.
fn freshened(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: BTreeMap<String, Node>,
    held: Option<&StateName>,
    settled: &BTreeMap<String, Determined>,
) -> BTreeMap<String, Node> {
    let Some(subject) = &outcome.subject else {
        return input;
    };
    let mut input = distinguished(ir, command, outcome, input, held);
    if !matches!(
        subject.effect,
        ResolvedEffect::Updates | ResolvedEffect::Moves { .. }
    ) {
        return input;
    }
    for set in &outcome.sets {
        let ResolvedPayloadValue::InputField { field, .. } = &set.value else {
            continue;
        };
        let already = held_value(ir, settled, &set.target, &set.target_type);
        if already.is_none() || input.get(field) != already.as_ref() {
            continue;
        }
        for nth in 1..=FRESH_WITNESSES {
            let Some(moved) = candidates(ir, command, &[], Distinction::further(nth))
                .ok()
                .and_then(|inputs| inputs.into_iter().next())
                .and_then(|mut further| further.remove(field))
            else {
                continue;
            };
            if Some(&moved) == already.as_ref() || equals_a_sibling(command, &input, field, &moved)
            {
                continue;
            }
            let mut next = input.clone();
            next.insert(field.clone(), moved);
            if admitted(ir, command, &next)
                && selects_branch(ir, command, outcome, held, &next).unwrap_or(false)
            {
                input = next;
                break;
            }
        }
    }
    unread_apart(ir, command, outcome, input, held, settled)
}

/// One `sets:` entry a `sets-drop` mutant of the mutation audit removed: `target: input.source`
/// on `command`'s `outcome` (beyond10x/ess#212). Only [`with_dropped_write`] names one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DroppedWrite {
    /// The command, by qualified name.
    pub(crate) command: String,
    /// The outcome.
    pub(crate) outcome: String,
    /// The entity field the dropped entry wrote.
    pub(crate) target: String,
    /// The input field it wrote it from.
    pub(crate) source: String,
}

thread_local! {
    /// The dropped write of the `sets-drop` mutant being synthesized on this thread, if any.
    static DROPPED: std::cell::RefCell<Option<DroppedWrite>> =
        const { std::cell::RefCell::new(None) };
}

/// Runs `synthesize` with `dropped` as the write the model under synthesis no longer makes, and
/// restores what was there before, also on a panic.
///
/// The mutant's model has lost the entry, so nothing in it says that its now unread input once fed
/// that field. [`unread_apart`] reads this to send the input apart from what the row holds there,
/// under whatever names, so a target that still writes it shows another row. Every synthesis
/// outside the mutation audit runs with none, and its bytes are unchanged.
pub(crate) fn with_dropped_write<R>(
    dropped: Option<DroppedWrite>,
    synthesize: impl FnOnce() -> R,
) -> R {
    struct Restore(Option<DroppedWrite>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let previous = self.0.take();
            DROPPED.with(|it| *it.borrow_mut() = previous);
        }
    }
    let previous = DROPPED.with(|it| it.replace(dropped));
    let _restore = Restore(previous);
    synthesize()
}

/// The dropped write [`with_dropped_write`] names for `command`'s `outcome`, if any.
fn dropped_writes(command: &str, outcome: &str) -> Vec<DroppedWrite> {
    DROPPED.with(|it| {
        it.borrow()
            .iter()
            .filter(|it| it.command == command && it.outcome == outcome)
            .cloned()
            .collect()
    })
}

/// The branch's input, with every field no `sets:` entry reads moved apart from what the row holds
/// in the field of the same name and identical declared type, where the branch leaves that field
/// alone (beyond10x/ess#212), and from what it holds in the field a `sets-drop` mutant's dropped
/// entry wrote from it ([`with_dropped_write`]).
///
/// Where the two coincide, the row reads the same whether or not the implementation also wrote the
/// input there, so a target that did — or the model of a `sets-drop` mutant, which leaves a field
/// the declared model writes as it was — passed. The input field naming the instance is never
/// moved, a further witness gives the moved field its value as [`freshened`] takes one, and the
/// branch is decided again by [`selects_branch`]; where nothing can move, the input stands.
///
/// Only the same-named field: it is the write an implementation makes by accident, and the one
/// `title: input.title` drops. Every same-typed field would move inputs in suites whose rows only
/// share a type with them, such as a payment's amount beside the invoice's total, and a choice of
/// witness is all this is — it says nothing about which field an input belongs to.
fn unread_apart(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    mut input: BTreeMap<String, Node>,
    held: Option<&StateName>,
    settled: &BTreeMap<String, Determined>,
) -> BTreeMap<String, Node> {
    let Some(subject) = &outcome.subject else {
        return input;
    };
    let written: BTreeSet<&str> = outcome.sets.iter().map(|set| set.target.as_str()).collect();
    let read: BTreeSet<&str> = outcome
        .sets
        .iter()
        .filter_map(|set| match &set.value {
            ResolvedPayloadValue::InputField { field, .. } => Some(field.as_str()),
            _ => None,
        })
        .collect();
    let identity = subject.instance.field().name.as_str();
    let entity = ir.entity(&subject.entity);
    let dropped = dropped_writes(&command.name.to_string(), &outcome.name.to_string());
    for field in &command.input {
        if read.contains(field.name.as_str()) || field.name == identity {
            continue;
        }
        // The fields a `sets-drop` mutant's dropped entry wrote from this input, whatever its name.
        let wrote: Vec<&str> = dropped
            .iter()
            .filter(|it| it.source == field.name)
            .map(|it| it.target.as_str())
            .collect();
        let stored: Vec<Node> = entity
            .fields
            .iter()
            .filter(|it| {
                !written.contains(it.name.as_str())
                    && ((it.name == field.name && it.type_ref == field.type_ref)
                        || wrote.contains(&it.name.as_str()))
            })
            .filter_map(|it| held_value(ir, settled, &it.name, &it.type_ref))
            .collect();
        if !input
            .get(&field.name)
            .is_some_and(|value| stored.contains(value))
        {
            continue;
        }
        for nth in 1..=FRESH_WITNESSES {
            let Some(moved) = candidates(ir, command, &[], Distinction::further(nth))
                .ok()
                .and_then(|inputs| inputs.into_iter().next())
                .and_then(|mut further| further.remove(&field.name))
            else {
                continue;
            };
            if stored.contains(&moved) || equals_a_sibling(command, &input, &field.name, &moved) {
                continue;
            }
            let mut next = input.clone();
            next.insert(field.name.clone(), moved);
            if admitted(ir, command, &next)
                && selects_branch(ir, command, outcome, held, &next).unwrap_or(false)
            {
                input = next;
                break;
            }
        }
    }
    input
}

/// Whether `value` at `field` would equal what a same-typed sibling input already carries.
fn equals_a_sibling(
    command: &ResolvedCommand,
    input: &BTreeMap<String, Node>,
    field: &str,
    value: &Node,
) -> bool {
    let Some(written) = command.input.iter().find(|input| input.name == field) else {
        return false;
    };
    command
        .input
        .iter()
        .filter(|sibling| sibling.name != field && sibling.type_ref == written.type_ref)
        .any(|sibling| input.get(&sibling.name) == Some(value))
}

/// The branch's input, with every field a `sets:` entry reads moved apart from its same-typed
/// siblings (beyond10x/ess#161).
///
/// `sets: {anonymous: input.anonymous}` beside a second `Boolean` input `recording`, both sent
/// `true`: the row reads `true` whichever input the implementation wrote from, so an implementation
/// — or a `sets-retarget` mutant — that read the sibling passed. So where a source and a sibling of
/// its declared type carry one value, the sibling is moved first, because it is not what this entry
/// writes, and the source only where the sibling cannot move without losing the branch. A further
/// witness gives the moved field its value, as [`freshened`] takes one, and the branch is decided
/// again by [`selects_branch`]; where neither can move the input stands, because no input tells the
/// two apart.
///
/// Three inputs of a two-valued type cannot all differ (beyond10x/ess#202), so the pairs that
/// matter most are separated first: the [`source_pairs`], each `sets:` source beside a same-typed
/// input no `sets:` entry reads. A `sets-retarget` mutant's model leaves the original source unread,
/// and only a witness sending it apart from the new source tells the mutant from the declared
/// model. Once apart — moved, or sent apart already — each such pair is pinned: no later move may
/// give its two inputs one value again. No other pair is pinned.
fn distinguished(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: BTreeMap<String, Node>,
    held: Option<&StateName>,
) -> BTreeMap<String, Node> {
    let keeps = |next: &BTreeMap<String, Node>| {
        selects_branch(ir, command, outcome, held, next).unwrap_or(false)
    };
    let (mut input, pinned) = sources_apart(ir, command, outcome, input, &keeps);
    for set in &outcome.sets {
        let ResolvedPayloadValue::InputField { field, .. } = &set.value else {
            continue;
        };
        let Some(written) = command.input.iter().find(|input| input.name == *field) else {
            continue;
        };
        let siblings: Vec<&String> = command
            .input
            .iter()
            .filter(|sibling| sibling.name != *field && sibling.type_ref == written.type_ref)
            .map(|sibling| &sibling.name)
            .collect();
        for sibling in siblings {
            if !input.contains_key(field) || input.get(field) != input.get(sibling) {
                continue;
            }
            if let Some(next) = moved_apart(ir, command, &input, [sibling, field], &pinned, &keeps)
            {
                input = next;
            }
        }
    }
    input
}

/// Every pair a `sets-retarget` mutant could join, read off the model being synthesized
/// (beyond10x/ess#202): `(source, unread)`, where `source` is an input some `sets:` entry of
/// `outcome` reads and `unread` is an input of the same declared type that no `sets:` entry of it
/// reads. A retarget leaves the original source unread whichever input it retargets to — one
/// feeding two targets, or one only a payload or a guard reads — so every such pair is one a
/// mutant may have joined.
///
/// Pairs come in priority order: first those the model shows joined — `source` feeds two or more
/// targets, or `unread` is named like a target `source` feeds — then the rest, each group in
/// declaration order of `source`, then of `unread`. Where not every pair can be sent apart, the
/// joined ones are the ones kept apart. The flag says which group a pair is in.
pub(super) fn source_pairs<'c>(
    command: &'c ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Vec<(&'c str, &'c str, bool)> {
    let mut fed: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for set in &outcome.sets {
        if let ResolvedPayloadValue::InputField { field, .. } = &set.value {
            fed.entry(field.as_str())
                .or_default()
                .push(set.target.as_str());
        }
    }
    let mut joined = Vec::new();
    let mut rest = Vec::new();
    for source in &command.input {
        let Some(targets) = fed.get(source.name.as_str()) else {
            continue;
        };
        for unread in command.input.iter().filter(|other| {
            other.name != source.name
                && other.type_ref == source.type_ref
                && !fed.contains_key(other.name.as_str())
        }) {
            let (source, unread) = (source.name.as_str(), unread.name.as_str());
            if targets.len() >= 2 || targets.contains(&unread) {
                joined.push((source, unread, true));
            } else {
                rest.push((source, unread, false));
            }
        }
    }
    joined.extend(rest);
    joined
}

/// `input` with every one of the [`source_pairs`] sent apart that can be, in their order, and the
/// pairs that end apart — the pins every later move must keep. For each pair still sent one value,
/// the unread input is moved first, because no `sets:` entry reads it into the row, and the source
/// only where the unread one cannot move; a move is kept only where the input is still admitted,
/// `keeps` it on the branch, and leaves every pair already pinned apart. Used by [`distinguished`]
/// and by the stored-row search's arrangement ([`subject_fact::prepare`]).
pub(super) fn sources_apart<'c>(
    ir: &EssIr,
    command: &'c ResolvedCommand,
    outcome: &ResolvedOutcome,
    mut input: BTreeMap<String, Node>,
    keeps: &dyn Fn(&BTreeMap<String, Node>) -> bool,
) -> (BTreeMap<String, Node>, Vec<(&'c str, &'c str)>) {
    let mut pinned: Vec<(&str, &str)> = Vec::new();
    for (source, unread, _) in source_pairs(command, outcome) {
        if !input.contains_key(source) || !input.contains_key(unread) {
            continue;
        }
        if input.get(source) == input.get(unread) {
            if let Some(next) = moved_apart(ir, command, &input, [unread, source], &pinned, keeps) {
                input = next;
            }
        }
        if input.get(source) != input.get(unread) {
            pinned.push((source, unread));
        }
    }
    (input, pinned)
}

/// The notes for a scenario whose last invocation of `command` sends a joined pair of
/// [`source_pairs`] of `outcome` with one value (beyond10x/ess#202), one note per pair: no input
/// the guards and the pins leave told them apart, so the `sets-retarget` mutant this model may be
/// is not killed by this scenario.
///
/// Only joined pairs are noted — the source feeds two targets, or the unread input is named like
/// the source's target — because those are the pairs a retarget shows. A declared source held
/// equal to an input it never joined (a guard fixing both, or a row whose writes must still
/// change) is a pair the model does not name, and noting it would report a separation nothing lost.
fn unseparated(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    id: &ScenarioId,
    scenario: &ConformanceScenario,
) -> Vec<Note> {
    let Some(sent) = scenario.steps.iter().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand {
            command: sent,
            input,
            ..
        } if sent.to_string() == command.name.to_string() => Some(input),
        _ => None,
    }) else {
        return Vec::new();
    };
    source_pairs(command, outcome)
        .into_iter()
        .filter(|(_, _, joined)| *joined)
        .filter(|(source, unread, _)| {
            match (
                sent.get(*source).and_then(ScenarioValue::as_literal),
                sent.get(*unread).and_then(ScenarioValue::as_literal),
            ) {
                (Some(first), Some(second)) => first == second,
                _ => false,
            }
        })
        .map(|(source, unread, _)| Note::UnseparatedSources {
            scenario: id.clone(),
            source: source.to_owned(),
            unread: unread.to_owned(),
        })
        .collect()
}

/// `input` with the first of `moving` that can move given a further witness's value, where the
/// result is admitted, `keeps` it on the branch, and leaves every `pinned` pair apart; nothing
/// where no such move exists. See [`distinguished`].
fn moved_apart(
    ir: &EssIr,
    command: &ResolvedCommand,
    input: &BTreeMap<String, Node>,
    moving: [&str; 2],
    pinned: &[(&str, &str)],
    keeps: &dyn Fn(&BTreeMap<String, Node>) -> bool,
) -> Option<BTreeMap<String, Node>> {
    for moving in moving {
        for nth in 1..=FRESH_WITNESSES {
            let Some(moved) = candidates(ir, command, &[], Distinction::further(nth))
                .ok()
                .and_then(|inputs| inputs.into_iter().next())
                .and_then(|mut further| further.remove(moving))
            else {
                continue;
            };
            if input.get(moving) == Some(&moved) {
                continue;
            }
            let mut next = input.clone();
            next.insert(moving.to_owned(), moved);
            if pinned
                .iter()
                .any(|(first, second)| next.get(*first) == next.get(*second))
            {
                continue;
            }
            if admitted(ir, command, &next) && keeps(&next) {
                return Some(next);
            }
        }
    }
    None
}

/// What a row holds in `target` before a branch writes it, where a scenario can know: the literal
/// an arranging act settled, or — where no act wrote it — an enum's first variant. See
/// [`freshened`].
fn held_value(
    ir: &EssIr,
    settled: &BTreeMap<String, Determined>,
    target: &str,
    type_ref: &ResolvedTypeRef,
) -> Option<Node> {
    match settled.get(target) {
        Some(Determined {
            value: ScenarioValue::Literal { value },
            ..
        }) => Some(value.clone()),
        Some(_) => None,
        None => first_variant(ir, type_ref, 0),
    }
}

/// The first declared variant of an enum, through newtypes; nothing for any other type, and
/// nothing for an `Optional`, which an unwritten row holds absent.
fn first_variant(ir: &EssIr, type_ref: &ResolvedTypeRef, depth: usize) -> Option<Node> {
    if depth > MAX_TYPE_DEPTH {
        return None;
    }
    match type_ref {
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => first_variant(ir, of, depth + 1),
            ResolvedBody::Enum { variants } => variants
                .first()
                .map(|variant| Node::Text(variant.name().to_owned())),
            ResolvedBody::Union { .. } | ResolvedBody::Struct { .. } => None,
        },
        ResolvedTypeRef::Primitive { .. }
        | ResolvedTypeRef::Optional { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => None,
    }
}

/// Whether every field of an input holds a value its declared type admits — the check every
/// candidate passed before it was offered, made again for one changed after.
fn admitted(ir: &EssIr, command: &ResolvedCommand, input: &BTreeMap<String, Node>) -> bool {
    command
        .input
        .iter()
        .all(|field| match input.get(&field.name) {
            Some(value) => crate::input::validate_typed_value(ir, &field.type_ref, value).is_ok(),
            None => field.type_ref.is_optional(),
        })
}

/// The value a literal written in the document leaves in a field of this type, where it is one.
///
/// The one question that decides whether a `sets:` literal is assertable, and it answers with the
/// **value** rather than a yes: the row will hold `false` and not the text `"false"`, so a reader
/// that only said "assertable" would have produced an assertion comparing a bool against a string,
/// which no implementation can satisfy. That is why this returns a [`Node`].
///
/// | target resolves to | the literal is |
/// |---|---|
/// | text, or a variant of an enum | the text itself |
/// | `Boolean`, `Integer`, `Decimal` | that value, spelled as [`crate::input::primitive_literal`] admits |
/// | `Binary64` | nothing — no admitted literal spelling |
/// | a struct, a union, a list, a map | nothing — a literal is one piece of text |
///
/// **This deliberately duplicates a rule `ess-domain` also enforces, and is not dead for it.**
/// `validate_sets` refuses a literal this would answer `None` for, so in a *compiled* IR the `None`
/// arms are unreachable from a document that validates. They stay because the alternative is
/// trusting an upstream invariant with no local check: where the two ever come apart the choice is
/// between abstaining and asserting a value nobody can read, and abstaining is a weaker claim rather
/// than a wrong one. The primitive half is not duplicated at all — it is
/// [`crate::input::primitive_literal`], the same function `ess-domain`'s own rule was written to
/// agree with, so the spellings cannot drift.
///
/// `Optional` and a newtype are transparent, exactly as they are to [`describe`] and to the
/// flattener — neither has a spelling of its own, so `Optional<LaneId>` answers as `String` does.
/// A type that resolves through itself gets no answer past [`MAX_TYPE_DEPTH`].
fn literal_value(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    written: &str,
    depth: usize,
) -> Option<Node> {
    if depth > MAX_TYPE_DEPTH {
        return None;
    }
    match type_ref {
        ResolvedTypeRef::Primitive { name } => crate::input::primitive_literal(*name, written),
        ResolvedTypeRef::Optional { of } => literal_value(ir, of, written, depth + 1),
        ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => None,
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => literal_value(ir, of, written, depth + 1),
            // Membership was checked against the declared variants where the literal was written;
            // a suite carries a variant as its name, which is what the enum witness sends too.
            ResolvedBody::Enum { .. } => Some(Node::Text(written.to_owned())),
            ResolvedBody::Union { .. } | ResolvedBody::Struct { .. } => None,
        },
    }
}

/// One entity field a scenario determined the value of, and the type the entity holds it at.
#[derive(Debug, Clone, PartialEq)]
struct Determined {
    /// What the row will hold.
    value: ScenarioValue,
    /// The entity field's declared type. A view has to project the field at this type for the row
    /// to carry the value the command supplied rather than something computed from it.
    type_ref: ResolvedTypeRef,
}

/// One row's assertable fields: what identifies it, and what of it this scenario determined.
///
/// Only fields the view actually projects, **at the type the entity holds them at**. A field the
/// view leaves out is not in the row, so asserting it would be a claim about a read nobody
/// performs; a field the view projects at a different type is a value the view computed from the
/// entity's rather than the one the command supplied, and the two are equal only by coincidence.
///
/// The identity wins where both name it: it is read from the arrangement, and a `sets:` source for
/// it says the same thing one step further from the observation.
fn shown(
    view: &ResolvedView,
    mut fields: BTreeMap<String, ScenarioValue>,
    settled: &BTreeMap<String, Determined>,
) -> BTreeMap<String, ScenarioValue> {
    for (name, determined) in settled {
        if view.field(name).map(|field| &field.type_ref) == Some(&determined.type_ref) {
            fields
                .entry(name.clone())
                .or_insert_with(|| determined.value.clone());
        }
    }
    fields
}

/// The lifecycle state a row of this view is required to carry, where the view projects it.
///
/// The one field of the subject every scenario determines whether or not a `sets:` names it: the
/// run says which state the branch leaves the subject in — the lifecycle's `initial` for a
/// `creates:`, the transition's `to` for a `moves:`, the arranged state for an `updates:`. Without
/// it a scenario named "moves an `Order` to `Closed`" read only the identity, and an implementation
/// that left the order where it was passed (beyond10x/ess#111). Only at the entity's own state type:
/// a view projecting `state` as something else computed it.
fn lifecycle_state(
    ir: &EssIr,
    entity: &EntityHandle,
    view: &ResolvedView,
    state: &StateName,
) -> Option<(String, ScenarioValue)> {
    let declared = ir.entity(entity).state_field();
    view.field(EntitySpec::STATE)
        .filter(|field| field.type_ref == declared.type_ref)
        .map(|_| {
            (
                EntitySpec::STATE.to_owned(),
                ScenarioValue::literal(Node::Text(state.to_string())),
            )
        })
}

/// Adds one requirement about one view, in the block that view's consistency decides.
///
/// The style is read, never re-derived: asserting an `eventual` view with `expect` races the
/// projection, and the repair everyone reaches for is a sleep. A second requirement about a view
/// already queried adds no second query — an `eventual` view read twice is two reads, and two reads
/// can disagree without either of them being wrong.
fn require(
    view: &ResolvedView,
    name: &ViewRef,
    params: BTreeMap<String, ScenarioValue>,
    expectation: ViewExpectation,
    steps: &mut Vec<ScenarioStep>,
) {
    match view.assertion_style {
        AssertionStyle::Expect => {
            let queried = steps
                .iter()
                .any(|step| matches!(step, ScenarioStep::QueryView { view, .. } if view == name));
            if !queried {
                steps.push(ScenarioStep::QueryView {
                    view: name.clone(),
                    params: params.clone(),
                });
            }
            steps.push(ScenarioStep::ExpectView {
                view: name.clone(),
                expectation,
            });
        }
        AssertionStyle::Eventually => steps.push(ScenarioStep::EventuallyView {
            view: name.clone(),
            params,
            expectation,
        }),
    }
}

/// How many rows this scenario puts in one view: its subject, where the filter admits it, and every
/// further instance arranged beside it.
///
/// A companion whose state the filter cannot decide counts for nothing. That is the same three-valued
/// reading [`shows`] makes everywhere else — an undecided filter is not a row, and counting one
/// would be the invention §11 rules out.
fn rows_shown(
    ir: &EssIr,
    view: &ResolvedView,
    admits_subject: bool,
    companions: &[Arrangement],
    params: &BTreeMap<String, ScenarioValue>,
) -> usize {
    usize::from(admits_subject)
        + companions
            .iter()
            .filter(|companion| companion.shows(ir, view, params) == Ok(true))
            .count()
}

/// One further instance of an entity, resting where a view's filter admits it.
///
/// The states are the entity's own declared ones, filtered by the view's own declared filter, and
/// the cheapest reachable one wins — the same rule [`arrange_first`] applies to a branch's subject,
/// for the same reason: every extra command is another way for an arrangement to fail for a reason
/// that has nothing to do with what the scenario tests.
fn arrange_beside(
    ir: &EssIr,
    entity: &EntityHandle,
    view: &ResolvedView,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    owner: Option<&InstanceName>,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<Arrangement, Unreachable> {
    let lifecycle = &ir.entity(entity).lifecycle;
    let admitted: Vec<StateName> = lifecycle
        .states
        .iter()
        .filter(|state| shows(ir, view, state, &BTreeMap::new(), params) == Ok(true))
        .cloned()
        .collect();
    if admitted.is_empty() {
        // No state alone admits a row, so the filter reads the row's fields: the instance is
        // created with an input chosen toward it.
        return arrange_toward(ir, entity, view, actors, distinction, owner, params).ok_or_else(
            || Unreachable::NoPath {
                from: lifecycle.initial.clone(),
            },
        );
    }
    arrange_first(ir, entity, &admitted, actors, distinction, &[])
}

/// One further instance whose *fields* a view's filter admits, where no state alone does
/// (story:view-filters-witnessed-on-matching-rows).
///
/// The filter is read onto each creating branch's input through that branch's direct `sets:` —
/// `source` on the row is `input.source` on the command — and handed to the witness search as a
/// guard, so its literals become candidates exactly as a guard's do. A case-insensitive filter's
/// literal is tried in its other ASCII case there, which is the row a target filtering byte for
/// byte drops. The rewrite only proposes: each candidate is created, driven to every state, and
/// kept only where [`shows`] admits the row it actually left, so a path the rewrite could not map
/// costs a candidate and never an assertion. The first candidate that lands a row wins, in its
/// cheapest state; `None` where none does.
///
/// A case-insensitive filter is first searched, over every creating branch, for a row the same
/// filter compared byte for byte refuses — the literal in its other case — and only then for any
/// row it admits: a branch guarded on the literal itself lands `web` byte for byte, which a
/// target filtering bytes admits as well (adversary pass 1 of this story).
fn arrange_toward(
    ir: &EssIr,
    entity: &EntityHandle,
    view: &ResolvedView,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    owner: Option<&InstanceName>,
    params: &BTreeMap<String, ScenarioValue>,
) -> Option<Arrangement> {
    let filter = view.filter.as_ref()?;
    let admitted = |row: &Arrangement| row.shows(ir, view, params) == Ok(true);
    if filter.uses_case_fold() {
        let bytewise = ResolvedView {
            filter: Some(byte_exact(filter)),
            ..view.clone()
        };
        let changed =
            |row: &Arrangement| admitted(row) && row.shows(ir, &bytewise, params) == Ok(false);
        let found = arrange_toward_by(ir, entity, view, actors, distinction, owner, &changed);
        if found.is_some() {
            return found;
        }
    }
    if let Some(bound) = paging::with_bound_params(view, params) {
        let found = arrange_toward_by(ir, entity, &bound, actors, distinction, owner, &admitted);
        if found.is_some() {
            return found;
        }
    }
    arrange_toward_by(ir, entity, view, actors, distinction, owner, &admitted)
}

/// `predicate` with every case-insensitive comparison made byte for byte: `equals_ignore_case`
/// and `in_ignore_case` read as `in` over the same literals.
fn byte_exact(predicate: &Predicate) -> Predicate {
    match predicate {
        Predicate::FoldMatch { path, values, .. } => Predicate::AnyOf {
            path: path.clone(),
            values: values.clone(),
        },
        Predicate::All(children) => Predicate::All(children.iter().map(byte_exact).collect()),
        Predicate::Any(children) => Predicate::Any(children.iter().map(byte_exact).collect()),
        Predicate::Not(inner) => Predicate::Not(Box::new(byte_exact(inner))),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let mapped = Box::new(Quantified {
                over: quantified.over.clone(),
                bind: quantified.bind.clone(),
                body: byte_exact(&quantified.body),
            });
            if matches!(predicate, Predicate::Forall(_)) {
                Predicate::Forall(mapped)
            } else {
                Predicate::Exists(mapped)
            }
        }
        other => other.clone(),
    }
}

/// [`arrange_toward`]'s search, keeping the cheapest row `accept` takes; `view` only lends its filter.
fn arrange_toward_by(
    ir: &EssIr,
    entity: &EntityHandle,
    view: &ResolvedView,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    owner: Option<&InstanceName>,
    accept: &dyn Fn(&Arrangement) -> bool,
) -> Option<Arrangement> {
    let filter = view.filter.as_ref()?;
    arrange_toward_owned(ir, entity, filter, actors, distinction, owner, accept)
}

/// [`arrange_toward_by`], with the filter handed in rather than lent by a view: a set effect's
/// `where:` with its operands written in (ess/16, [`set_effects`]).
fn arrange_toward_filter(
    ir: &EssIr,
    entity: &EntityHandle,
    filter: &Predicate,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    accept: &dyn Fn(&Arrangement) -> bool,
) -> Option<Arrangement> {
    arrange_toward_owned(ir, entity, filter, actors, distinction, None, accept)
}

/// [`arrange_toward_filter`], each row created under `owner` where one is given and the creating
/// branch names its owner through `sets:` (beyond10x/ess#193): a further row beside the subject's,
/// told from it by nothing but its identity, or read by the same owner's list.
fn arrange_toward_owned(
    ir: &EssIr,
    entity: &EntityHandle,
    filter: &Predicate,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    owner: Option<&InstanceName>,
    accept: &dyn Fn(&Arrangement) -> bool,
) -> Option<Arrangement> {
    arrange_toward_bound(
        ir,
        entity,
        filter,
        actors,
        distinction,
        owner,
        &BTreeMap::new(),
        accept,
    )
}

/// The same bounded arrangement search with fields populated from already captured instances.
/// Only a direct, unconverted `sets: field: input.field` proves which creator input to bind.
/// The identities stay symbolic throughout: a generated ID is never replaced by a sample value.
#[allow(clippy::too_many_arguments)]
fn arrange_toward_bound(
    ir: &EssIr,
    entity: &EntityHandle,
    filter: &Predicate,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    owner: Option<&InstanceName>,
    fields: &BTreeMap<String, InstanceName>,
    accept: &dyn Fn(&Arrangement) -> bool,
) -> Option<Arrangement> {
    let all = ir.drivers();
    let drivers: &[Driver<'_>] = all.get(entity).map_or(&[], Vec::as_slice);
    let states = &ir.entity(entity).lifecycle.states;
    for creator in drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
    {
        let mapped: BTreeMap<&str, &str> = creator
            .outcome
            .sets
            .iter()
            .filter(|set| set.conversion.is_none())
            .filter_map(|set| match &set.value {
                ResolvedPayloadValue::InputField { field, .. } => {
                    Some((set.target.as_str(), field.as_str()))
                }
                _ => None,
            })
            .collect();
        let Some(bound) = creator_bindings(creator, &mapped, fields) else {
            continue;
        };
        let onto = |path: &FactPath| match path.segments().split_first() {
            Some((head, tail)) => mapped.get(head.as_str()).map_or_else(
                || path.clone(),
                |field| {
                    FactPath::from_segments(
                        field
                            .split('.')
                            .map(str::to_owned)
                            .chain(tail.iter().cloned()),
                    )
                },
            ),
            None => path.clone(),
        };
        let toward = map_paths(filter, &onto);
        let mut guards = vec![&toward];
        guards.extend(when(creator.outcome));
        let Ok(inputs) = candidates(ir, creator.command, &guards, distinction) else {
            continue;
        };
        for input in inputs {
            if selects_branch(ir, creator.command, creator.outcome, None, &input) != Ok(true) {
                continue;
            }
            let owner =
                owner.or_else(|| ir.owner_of(entity).and_then(|owned| fields.get(owned.via)));
            let under = owner.and_then(|owner| under_owner(ir, entity, creator, owner));
            let start = created_bound(
                ir,
                entity,
                creator,
                actors,
                distinction,
                under,
                &bound,
                &input,
            );
            let Ok(start) = start else {
                continue;
            };
            let mut best: Option<Arrangement> = None;
            for target in states {
                let Some(route) = route_from(ir, entity, drivers, &start.state, target) else {
                    continue;
                };
                let Ok(reached) = advance(
                    ir,
                    entity,
                    start.clone(),
                    route,
                    target,
                    actors,
                    distinction,
                    &[],
                ) else {
                    continue;
                };
                if accept(&reached)
                    && best
                        .as_ref()
                        .is_none_or(|held| reached.steps.len() < held.steps.len())
                {
                    best = Some(reached);
                }
            }
            if best.is_some() {
                return best;
            }
        }
    }
    None
}

/// Map the requested fields onto creator inputs only where the capture does not invalidate a
/// guard proved from literal inputs. Symbolic guard solving is outside this arrangement seam.
fn creator_bindings(
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    fields: &BTreeMap<String, InstanceName>,
) -> Option<BTreeMap<String, InstanceName>> {
    let bound = fields
        .iter()
        .try_fold(BTreeMap::new(), |mut bound, (field, instance)| {
            let input = mapped.get(field.as_str())?;
            if bound
                .insert((*input).to_owned(), instance.clone())
                .is_some_and(|held| held != *instance)
            {
                return None;
            }
            Some(bound)
        })?;
    if !bound.is_empty()
        && (related_guard::routes(creator.command, creator.outcome)
            || creator
                .command
                .outcomes
                .iter()
                .filter_map(|outcome| when(outcome).or_else(|| accepting_input_half(outcome)))
                .any(|guard| {
                    guard
                        .fact_paths()
                        .iter()
                        .any(|path| bound.contains_key(path.namespace()))
                }))
    {
        return None;
    }
    Some(bound)
}

/// Create one row with explicit captured inputs, preserving any declared owner arrangement.
#[allow(clippy::too_many_arguments)]
fn created_bound(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    owner: Option<(String, Arrangement)>,
    bound: &BTreeMap<String, InstanceName>,
    input: &BTreeMap<String, Node>,
) -> Result<Arrangement, Unreachable> {
    if bound.is_empty() {
        return match owner.as_ref() {
            Some(under) => created_owned(
                ir,
                entity,
                creator,
                actors,
                distinction,
                &[],
                Some(under),
                Some(input),
            ),
            None => created(ir, entity, creator, actors, distinction, &[], Some(input)),
        };
    }
    let arranged_owner =
        owner.or_else(|| arrange_owner(ir, creator.outcome, entity, actors, distinction, &[]));
    created_by(
        ir,
        entity,
        creator,
        distinction,
        arranged_owner.as_ref(),
        |owner_bound, _| {
            let mut sent = owner_bound.clone();
            for (field, instance) in bound {
                if sent.get(field).is_some_and(|held| held != instance) {
                    return Err(Unreachable::NothingCreates);
                }
                sent.insert(field.clone(), instance.clone());
            }
            Ok(invoke_with(ir, creator, None, actors, &sent, input))
        },
    )
}

/// The field match that names the instance this scenario is about, where the model publishes one.
///
/// # Why this is a reading and not a name match
///
/// Three declarations meet, and no two of them are being guessed at. The outcome's `instance:` says
/// where the identity of the instance it acts on is — an input field for `moves:` and `updates:`, a
/// field of an emitted event for `creates:` — which is the same declaration
/// [`ScenarioStep::CaptureInstance`] is written from. The entity's `identity:` says what that field
/// is *called*. And `ess-domain` validates every view field against the entity's observable fields
/// by that name, so a view projecting `invoice_id` is projecting the identity rather than something
/// spelled like it.
///
/// # What it is worth
///
/// Without it, `Contains {}` says "the view holds some row" and `Excludes {}` says "the view holds
/// none" — claims that are only equivalent to the intended ones because §8 isolates each scenario.
/// Against a target that shares state with anything else, the first passes on somebody else's row
/// and the second fails on it.
///
/// Empty where the view does not project the identity at all, or where nothing bound one. That is a
/// weaker assertion rather than a wrong one, and not a refusal: the specification did not ask for a
/// check this view cannot carry.
fn identifying(
    ir: &EssIr,
    subject: &ResolvedSubject,
    instance: Option<&InstanceName>,
    view: &ResolvedView,
) -> BTreeMap<String, ScenarioValue> {
    let named = ir.entity(&subject.entity).identity.name.clone();
    if view.field(&named).is_none() {
        return BTreeMap::new();
    }
    let value = match &subject.instance {
        // The caller supplied it, so the scenario knows it as whatever the arrangement bound.
        ResolvedInstance::Supplied { .. } => {
            let Some(bound) = instance else {
                return BTreeMap::new();
            };
            ScenarioValue::instance(bound.clone())
        }
        // The branch published it, and this scenario is the one that ran the branch — bound right
        // after it where an earlier step published the same event (ess/18, a related row of the
        // created entity), and otherwise read from the event itself.
        ResolvedInstance::Observed { event, field } => instance.map_or_else(
            || ScenarioValue::observed(EventRef::from(event), field.name.clone()),
            |bound| ScenarioValue::instance(bound.clone()),
        ),
    };
    [(named, value)].into_iter().collect()
}

/// The identity of the instance a scenario is about, as the scenario refers to it, or `None` where
/// nothing bound one.
fn identity_of(
    subject: &ResolvedSubject,
    instance: Option<&InstanceName>,
) -> Option<ScenarioValue> {
    match &subject.instance {
        // The caller supplied it, so the scenario knows it as whatever the arrangement bound.
        ResolvedInstance::Supplied { .. } => instance.cloned().map(ScenarioValue::instance),
        // The branch published it, and this scenario is the one that ran the branch.
        ResolvedInstance::Observed { event, field } => Some(ScenarioValue::observed(
            EventRef::from(event),
            field.name.clone(),
        )),
    }
}

/// What this scenario supplies for each parameter the view declares.
///
/// Read from what the arrangement settled, never invented: a parameter is compared against a field
/// of the source entity, and the scenario put a value in that field on the way to the state it is
/// asserting. So `queue_id == param.queue_id` asks for the queue the instance was just placed in,
/// and the row it expects is the one it created.
///
/// A parameter compared with the row's identity, or with a link field the arrangement filled with
/// an instance, is sent as that identity: `id == param.id` asks for the row the scenario made, known
/// by `identity` (beyond10x/ess#193, [`identity::param`]).
///
/// A parameter the arrangement did not settle comes back missing, which leaves the filter
/// undecidable and the view refused by name — the same answer this module gives everywhere for a
/// question the model cannot decide.
fn bound(
    ir: &EssIr,
    view: &ResolvedView,
    settled: &BTreeMap<String, Determined>,
    identity: Option<&ScenarioValue>,
) -> BTreeMap<String, ScenarioValue> {
    view.params
        .iter()
        // A paging parameter is sent only by a page read (`paging::page_reads`), never by name.
        .filter(|param| !paging::reads(view, &param.name))
        .filter_map(|param| {
            settled
                .get(&param.name)
                .map(|determined| determined.value.clone())
                .or_else(|| identity::param(ir, view, &param.name, settled, identity))
                .map(|value| (param.name.clone(), value))
        })
        .collect()
}

/// One scalar node as a fact, or nothing when it is not a scalar.
///
/// A parameter compared in a filter is compared against a field, and a field holds a scalar. A
/// sequence or a map returns `None`, which leaves the filter undecidable and the view refused by
/// name rather than admitted on a comparison nobody could evaluate.
fn fact_value(node: &Node) -> Option<FactValue> {
    match node {
        Node::Text(text) => Some(FactValue::text(text.clone())),
        Node::Bool(value) => Some(FactValue::text(value.to_string())),
        Node::Number(number) => FactValue::number(number.get()).ok(),
        Node::Null | Node::Seq(_) | Node::Map(_) => None,
    }
}

/// Whether a view holds a row for an entity in `state`, or the paths that stop the question being
/// answered.
///
/// The one fact a synthesised scenario knows about the entity it just created is where its
/// lifecycle starts, so that is the one fact bound. Three-valued, and the third value refuses:
/// `Unknown` here means the filter reads something no scenario can know, and asserting either way
/// would be the invention §11 rules out.
fn shows(
    ir: &EssIr,
    view: &ResolvedView,
    state: &StateName,
    settled: &BTreeMap<String, Determined>,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<bool, Vec<FactPath>> {
    shows_row(ir, view, state, settled, None, params)
}

/// [`shows`], for a row whose identity the scenario knows: the subject's, or an arranged instance's
/// ([`Arrangement::identity`]). The identity, every link field the arrangement filled with an
/// instance, and every parameter sent as one are bound as opaque tokens, so a filter comparing two
/// of them is decided (beyond10x/ess#193, [`identity`]).
fn shows_row(
    ir: &EssIr,
    view: &ResolvedView,
    state: &StateName,
    settled: &BTreeMap<String, Determined>,
    identity: Option<&ScenarioValue>,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<bool, Vec<FactPath>> {
    let Some(filter) = &view.filter else {
        return Ok(true);
    };
    // A filter reads the source's rows. A row-level view projects them at the source's types, so
    // its own fields type the facts; an aggregate view's fields are group keys and results, and
    // `talk_seconds` in its filter is the row's value and never the sum a field of that name holds.
    let source_fields;
    let typed = if view.is_aggregate() {
        source_fields = ir.entity(&view.source).observable_fields();
        &source_fields
    } else {
        &view.fields
    };
    let mut facts = crate::input::TypedFacts::new(ir, typed, FactStore::new());
    let path = FactPath::new(EntitySpec::STATE)
        .unwrap_or_else(|error| panic!("`{}` is a fact path: {error}", EntitySpec::STATE));
    facts.set(path, FactValue::text(state.as_str()));
    // What the arrangement put in the entity's own fields. The state used to be the only fact a
    // synthesised scenario knew about the instance it had just created; `sets:` made the rest
    // knowable, and a filter over a field this scenario supplied is decidable from what it
    // supplied. Without this, `lane_id == param.lane_id` is `Unknown` however well the parameter
    // is bound, because the left side is the one nothing had answered.
    //
    // Each value is bound at the entity's declared type, as `subject_fact::row_truth_with` binds a
    // row: a struct's leaves and every present struct, list or map inside it, empty or not, are
    // what `defined()` and `missing()` read (beyond10x/ess#176). One field at a time, so a value
    // that is not of its type is left to the scalar reading it had rather than dropping the rest.
    let declared = &ir.entity(&view.source).fields;
    for (name, determined) in settled {
        if let (Ok(path), ScenarioValue::Literal { value }) =
            (FactPath::new(name), &determined.value)
        {
            let one = BTreeMap::from([(name.clone(), value.clone())]);
            match crate::input::bind(ir, declared, &one, crate::input::Completeness::Partial) {
                Ok(bound) => facts.extend(bound),
                Err(_) => {
                    if let Some(fact) = fact_value(value) {
                        facts.set(path, fact);
                    }
                }
            }
        }
    }
    // A parameter is a fact the *caller* supplies, and this scenario is the caller. Binding it is
    // what lets the two sides meet: the arrangement set the row's `lane_id`, so the scenario asks
    // for the lane it just put the instance in. Unbound leaves the filter `Unknown` and the view
    // is refused by name, which is the honest answer.
    for (name, value) in params {
        if let ScenarioValue::Literal { value } = value {
            if let (Ok(path), Some(fact)) = (
                FactPath::new(format!("{}.{name}", ess_domain::view::ViewSpec::PARAM)),
                fact_value(value),
            ) {
                facts.set(path, fact);
            }
        }
    }
    for (path, token) in identity::facts(ir, view, settled, identity, params) {
        facts.set(path, token);
    }

    match filter.evaluate(&facts) {
        Truth::True => Ok(true),
        Truth::False => Ok(false),
        Truth::Unknown => Err(filter
            .fact_paths()
            .into_iter()
            .filter(|path| facts.fact(path).is_none())
            .cloned()
            .collect()),
    }
}

/// Every construct this scenario's result depends on (§37).
///
/// Not what caused it to exist: the types its input mentions and the payloads it asserts are in
/// here too, because a change to one of those makes a stored result stale while a list of causes
/// says nothing.
fn dependencies(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    absent: &[EventRef],
    actor: Option<ActorRef>,
    views: &BTreeSet<ViewRef>,
) -> BTreeSet<EssSemanticRef> {
    let mut source = BTreeSet::new();
    let command_ref = CommandRef::new(command.name.clone());
    source.insert(command_ref.clone().into());
    source.insert(OutcomeRef::new(command_ref, outcome.name.clone()).into());

    let mut types = BTreeSet::new();
    for field in &command.input {
        reachable_types(ir, &field.type_ref, &mut types);
    }
    for handle in &outcome.emits {
        source.insert(EventRef::from(handle).into());
        for field in &ir.event(handle).fields {
            reachable_types(ir, &field.type_ref, &mut types);
        }
    }
    if let Some(handle) = &outcome.error {
        source.insert(ErrorRef::from(handle).into());
        for field in &ir.error(handle).fields {
            reachable_types(ir, &field.type_ref, &mut types);
        }
    }
    // An event asserted *absent* contributes its name and not its shape: the check is that nothing
    // arrived, which no change to the payload can affect.
    for event in absent {
        source.insert(event.clone().into());
    }
    if let Some(subject) = &outcome.subject {
        source.insert(EntityRef::from(&subject.entity).into());
    }
    if let Some(actor) = actor {
        source.insert(actor.into());
    }
    for view in views {
        source.insert(view.clone().into());
    }
    source.extend(types.into_iter().map(EssSemanticRef::from));
    source
}

/// Every declared type a reference reaches, through newtypes, structs and unions.
///
/// The set is the visited guard, so a type that refers to itself terminates rather than recursing.
pub(crate) fn reachable_types(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    found: &mut BTreeSet<DeclaredTypeRef>,
) {
    for handle in type_ref.named_leaves() {
        if !found.insert(DeclaredTypeRef::from(handle)) {
            continue;
        }
        match &ir.named_type(handle).body {
            ResolvedBody::Newtype { of, .. } => reachable_types(ir, of, found),
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    reachable_types(ir, &field.type_ref, found);
                }
            }
            ResolvedBody::Union { variants, .. } => {
                for variant in variants.values() {
                    reachable_types(ir, variant, found);
                }
            }
            ResolvedBody::Enum { .. } => {}
        }
    }
}

/// One line saying what the scenario proves, for the person reading a report.
fn purpose(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> ScenarioPurpose {
    let reached = match outcome.test_strategy {
        TestStrategy::ReplayResult => "an observed originating success and its retained result",
        TestStrategy::ConstructInput => "an input that satisfies that branch's guard",
        TestStrategy::ObserveSubjectFact => {
            "an independently observed subject enum fact and an eligible input"
        }
        TestStrategy::ConstructInputInState => {
            "an input selecting the branch in its established subject state"
        }
        TestStrategy::DefaultBranch => "an input no other branch's guard claims",
        TestStrategy::InjectFault => "the cause it declares as external, injected",
        TestStrategy::ArrangeState => "a subject in a state its moves do not start from",
        TestStrategy::SendUnknownIdentity => "an identity no record carries",
        TestStrategy::SendNoInput => "a request with no input at all",
        TestStrategy::SendExistingIdentity => "a second call with an identity a record carries",
        TestStrategy::ArrangeRelatedRow => "an arranged row of the entity its input names, or none",
    };
    let text = format!(
        "`{}` answers `{}` for {reached}",
        command.name, outcome.name
    );
    clipped(&text)
}

/// The two classes of lifecycle scenario §19 asks for, legal and illegal alike.
///
/// Both are sequences over one instance: bring one into existence, drive it to the state in
/// question, then either take the move and require it happened, or issue the command that must not
/// be honoured there and require that it did not. Neither could be written before an outcome said
/// which field names the instance it acts on.
fn lifecycle(
    models: &caller::InvocationModels<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    focus: Focus<'_>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
    notes: &mut Vec<Note>,
) {
    let ir = models.arrangement;
    for (handle, drivers) in models.acting.drivers() {
        let entity = EntityRef::from(handle);
        let states = &ir.entity(handle).lifecycle;

        for transition in &states.transitions {
            for driver in drivers
                .iter()
                .filter(|driver| driver.takes(&transition.name))
                .filter(|driver| focus.takes(&driver.command.name))
            {
                let id = ScenarioId::Transition {
                    transition: TransitionRef::new(entity.clone(), &transition.name)
                        .unwrap_or_else(|error| {
                            panic!("a declared transition is a single segment: {error}")
                        }),
                    by: OutcomeRef::new(
                        CommandRef::new(driver.command.name.clone()),
                        driver.outcome.name.clone(),
                    ),
                };
                let Some((mut steps, mut source, run)) = exercise_as(
                    models,
                    driver.command,
                    driver.outcome,
                    actors,
                    &id,
                    refusals,
                    Witness::Full,
                ) else {
                    continue;
                };
                let (further, depends) = other_sources(
                    models, driver, transition, actors, &run, &steps, &id, refusals,
                );
                steps.extend(further);
                source.extend(depends);
                let purpose = moving(&entity, transition, driver);
                insert(
                    suite,
                    id,
                    ConformanceScenario::new(purpose, steps, source),
                    refusals,
                );
            }
        }

        // The absence of a transition is itself semantics (§19). Which states those are is not
        // computed here and not computed twice: `EssIr::wrong_states` subtracts the `from` sets of
        // the moves a command declares from the states its entity declares, and the documentation
        // projection reads the same answer to print it on the page.
        let movers: BTreeMap<&QualifiedName, BTreeSet<&StateName>> = drivers
            .iter()
            .filter(|driver| driver.effect.transition().is_some())
            // Explicit held-state partitions already declare selection in every covered state;
            // complementing only their moving branch would invent a wrong-state refusal where
            // an updates branch intentionally preserves the state.
            .filter(|driver| !has_subject_guards(driver.command))
            .map(|driver| (&driver.command.name, driver.command))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .filter_map(|(name, command)| {
                ir.wrong_states(command)
                    .remove(handle)
                    .map(|states| (name, states))
            })
            .collect();
        for state in &states.states {
            for (command, wrong) in &movers {
                if !wrong.contains(state) || !focus.takes(command) {
                    continue;
                }
                // Whether the id reads `refuses` or `accepts` is the command's own claim, read
                // from the branch rather than inferred from what the scenario ends up asserting.
                // A command with no wrong-state branch says nothing, and the scenario is still
                // produced with `RefusalUndeclared` beside it — so the default here is the one
                // every specification written before `refuses:` existed meant.
                let refuses = drivers
                    .iter()
                    .find(|driver| &driver.command.name == *command)
                    .and_then(|driver| {
                        driver
                            .command
                            .outcomes
                            .iter()
                            .find(|outcome| outcome.condition == ResolvedCondition::WrongState)
                            .map(|outcome| outcome.refuses)
                    })
                    .unwrap_or(true);
                let id = ScenarioId::Refusal {
                    entity: entity.clone(),
                    state: state.clone(),
                    command: CommandRef::new((*command).clone()),
                    refuses,
                };
                if let Some(scenario) = wrong_state_scenario(
                    models, handle, &drivers, command, state, actors, &id, refusals, notes,
                ) {
                    insert(suite, id, scenario, refusals);
                }
            }
        }
    }
}

/// The `<entity>/state/<S>/refuses/<command>` scenario of one wrong state, or `None` with the
/// refusal recorded.
///
/// Guarded branches reading `state` select before `wrong_state:` applies (ess/18, the #192
/// ruling): each that may be taken in `S` is witnessed on a row of its own
/// ([`subject_fact::state_answered_rows`]), ahead of the plain wrong-state row, which is written
/// only where some input and row still reach it. A state they answer together, over their input
/// halves, is still this state's scenario, and never dropped.
#[allow(clippy::too_many_arguments)]
fn wrong_state_scenario(
    models: &caller::InvocationModels<'_>,
    handle: &EntityHandle,
    drivers: &[Driver<'_>],
    command: &QualifiedName,
    state: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
    notes: &mut Vec<Note>,
) -> Option<ConformanceScenario> {
    let answered = models.acting.commands().get(command).map_or_else(
        || Ok((Vec::new(), BTreeSet::new())),
        |declared| subject_fact::state_answered_rows(models, declared, handle, state, actors),
    );
    let (rows, depends) = match answered {
        Ok(found) => found,
        Err(cause) => {
            refusals.push(Refusal::about(id, cause));
            return None;
        }
    };
    let mut plain = Vec::new();
    match refused_here(
        models, handle, drivers, command, state, actors, id, &mut plain,
    ) {
        Some((mut scenario, unobserved)) => {
            refusals.extend(plain);
            if !unobserved.is_empty() {
                notes.push(Note::PartialObservation {
                    scenario: id.clone(),
                    unobserved,
                });
            }
            let mut steps = rows;
            steps.append(&mut scenario.steps);
            scenario.steps = steps;
            scenario.source.extend(depends);
            Some(scenario)
        }
        // No input and row reach the plain wrong-state case: the guards answer every one, so
        // their rows are the scenario.
        None if !rows.is_empty()
            && plain.iter().all(|refused| {
                matches!(refused.cause, RefusalCause::GuardUnsatisfiable { .. })
            }) =>
        {
            Some(ConformanceScenario::new(
                ScenarioPurpose::new(format!(
                    "`{command}` in held state `{state}` is answered by the guarded branches \
                     reading it"
                ))
                .expect("nonempty purpose"),
                rows,
                depends,
            ))
        }
        None => {
            refusals.extend(plain);
            None
        }
    }
}

/// Adds a scenario, or records the collision as a refusal rather than losing one of the two.
pub(crate) fn insert(
    suite: &mut ConformanceSuite,
    id: ScenarioId,
    scenario: ConformanceScenario,
    refusals: &mut Vec<Refusal>,
) {
    if let Err(claimed) = suite.insert(id, scenario) {
        refusals.push(Refusal::about(&claimed, RefusalCause::DuplicateScenario));
    }
}

/// The scenario that proves a command is not honoured in a state its moves cannot start from.
///
/// Three decisions are worth stating, because each is the difference between a check and a
/// formality:
///
/// * The input is the one that **would have reached the moving branch**. Sending a value the branch
///   would refuse anyway produces a scenario that passes whether or not the state rule holds.
/// * Where the command declares a [`WrongState`](ResolvedCondition::WrongState) branch, that branch
///   and its `error:` are both required. This is §19's "the exact rejection mechanism must come from
///   the declared command/error semantics", and it is the whole reason the construct exists: without
///   it the only honest assertion was a negative one, so an implementation that refused with the
///   wrong error — or with an untyped infrastructure failure — passed. Where the command declares no
///   such branch the scenario is still produced and [`RefusalCause::RefusalUndeclared`] is recorded
///   beside it, because a thin check that looks like a thick one is the silence §36 rules out.
/// * What is asserted is that **no** event the specification declares was published — not merely
///   that the move's own events were not. No branch that emits was taken here, and every declared
///   event belongs to a branch, so an invocation that publishes one has published something no
///   branch of it licensed. The narrower claim let an unhonoured `CancelInvoice` announce that the
///   invoice was paid, which is the same hole `not_emitted` closes on the branches that *are*
///   declared.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)] // One refusal witness, including explicit arrangement/acting roles.
fn refused_here(
    models: &caller::InvocationModels<'_>,
    handle: &EntityHandle,
    drivers: &[Driver<'_>],
    command: &QualifiedName,
    state: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> Option<(ConformanceScenario, Vec<String>)> {
    let ir = models.arrangement;
    let entity = EntityRef::from(handle);
    let movers: Vec<&Driver<'_>> = drivers
        .iter()
        .filter(|driver| &driver.command.name == command && driver.effect.transition().is_some())
        .collect();
    let attempt = movers.first().copied()?;

    let (arrangement, input, bound) = refusal_arrangement(ir, handle, state, actors, attempt)
        .map_err(|cause| refusals.push(Refusal::about(id, cause)))
        .ok()?;

    let command_ref = CommandRef::new(command.clone());
    let preservation = complete_wrong_state(ir, attempt, &arrangement)
        .map_err(|cause| {
            refusals.push(Refusal::about(id, cause));
        })
        .ok()?;
    let mut steps = arrangement.steps;
    models.mark(caller::InvocationPhase::Arrange, &mut steps);
    if let Some(preservation) = &preservation {
        steps.extend(preservation.before.iter().cloned());
    }
    let supplied = supply(
        attempt.command,
        &input,
        attempt.outcome.subject.as_ref(),
        Some(&arrangement.instance),
        // The command under test moves the row the arrangement already created, so its input
        // names that row; an owner, where there was one, was arranged inside `arrange`.
        &bound,
    );
    steps.push(ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref.clone(),
        actor: actors.get(command).cloned(),
        input: supplied.clone(),
    });

    let mut source = arrangement.source;
    source.insert(command_ref.clone().into());
    source.insert(EntityRef::from(handle).into());
    for driver in &movers {
        source.insert(OutcomeRef::new(command_ref.clone(), driver.outcome.name.clone()).into());
    }
    if let Some(actor) = actors.get(command) {
        source.insert(actor.clone().into());
    }

    let declared = attempt
        .command
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::WrongState);
    // `refuses: false` on that branch: the command is accepted here and the subject does not move.
    // Everything the refusing scenario asserts still holds — the declared branch is taken and no
    // event is published — and one thing does not: there is no error to name, because the command
    // reports none. The domain refuses the two together, so this reads the claim rather than
    // guessing it from `error:` being absent.
    let accepted = declared.is_some_and(|outcome| !outcome.refuses);
    let reported = if let Some(refusal) = declared {
        let branch = OutcomeRef::new(command_ref.clone(), refusal.name.clone());
        steps.push(ScenarioStep::ExpectOutcome {
            outcome: branch.clone(),
        });
        source.insert(branch.into());
        // The declared error, by name and with no invented payload — the same line `exercise` draws
        // for every other refusal, and the reason this family stopped being a "something went
        // wrong" check. The fields its `payload:` determines are compared (ess/19).
        refusal.error.as_ref().map(|error| {
            let named = ErrorRef::from(error);
            let expected = expect_error(ir, refusal, error, &supplied, &arrangement.settled);
            steps.push(expected);
            source.insert(named.clone().into());
            named
        })
    } else {
        // Beside the scenario, never instead of it: what the scenario asserts is real, and it is
        // less than §19 asks for, and only one of those two facts is visible in a passing run.
        refusals.push(Refusal::about(
            id,
            RefusalCause::RefusalUndeclared {
                entity: entity.clone(),
                state: state.clone(),
                command: command_ref,
            },
        ));
        None
    };

    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(forbidden.into_iter().map(EssSemanticRef::from));

    let mut unobserved = Vec::new();
    if let Some(preservation) = preservation {
        steps.push(ScenarioStep::ExpectNoEvents);
        steps.extend(preservation.after);
        source.extend(preservation.source);
        unobserved = preservation.unobserved;
    }

    let text = match (&reported, accepted) {
        (Some(error), _) => format!(
            "`{command}` does not move a `{entity}` that is in `{state}`, and reports `{error}`"
        ),
        (None, true) => format!(
            "`{command}` is accepted on a `{entity}` that is in `{state}`, and does not move it"
        ),
        (None, false) => format!("`{command}` does not move a `{entity}` that is in `{state}`"),
    };
    models.mark(caller::InvocationPhase::Act, &mut steps);
    Some((
        ConformanceScenario::new(clipped(&text), steps, source),
        unobserved,
    ))
}

/// The input field naming the existing instance a branch acts on, where it acts on one.
///
/// A `moves:` or `updates:` branch whose `instance:` is read from input: the only branches an
/// identity naming no record can reach. A creating branch makes its record, a refusal names none,
/// and an observed identity is not chosen by the caller.
fn names_existing(outcome: &ResolvedOutcome) -> Option<&str> {
    let subject = outcome.subject.as_ref()?;
    if !matches!(
        subject.effect,
        ResolvedEffect::Moves { .. } | ResolvedEffect::Updates | ResolvedEffect::Deletes
    ) {
        return None;
    }
    match &subject.instance {
        ResolvedInstance::Supplied { field } => Some(field.name.as_str()),
        ResolvedInstance::Observed { .. } => None,
    }
}

/// The unknown-instance rule, witnessed once per command acting on an input-named instance that
/// declares an answer for it (`docs/design/typed-literals-and-unknown-instances.md`, sections 2
/// and 2a).
///
/// A command whose input selects a branch acting on an existing instance, and whose `instance:`
/// names no record, answers its declared not-found outcome ([`not_found`]) when it declares one.
/// That scenario is filed under the not-found outcome's own id, replacing the scenario that
/// injected its external cause: sending an identity no record carries *is* that cause, witnessed
/// rather than forced. Where no identity is known to be fresh, the injected scenario stays.
///
/// Only a command declaring no not-found outcome falls back to its `wrong_state` branch. That
/// scenario is filed under the `wrong_state` branch's own outcome id, which no scenario held
/// before: the states the branch answers in are the illegal-move family's, one scenario each, and
/// this is the one case that family cannot arrange.
///
/// A command declaring neither has no declared answer, and gets a [`Note`] — not a refusal, because
/// the specification states nothing here that is left unwitnessed. So does a command declaring two
/// or more not-found candidates: which one an unknown identity takes is not stated, and neither is
/// assumed.
fn unknown_instances(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    focus: Focus<'_>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
    notes: &mut Vec<Note>,
) {
    for command in ir.commands().values() {
        if !focus.takes(&command.name) {
            continue;
        }
        let acting: Vec<&ResolvedOutcome> = command
            .outcomes
            .iter()
            .filter(|outcome| names_existing(outcome).is_some())
            .collect();
        if acting.is_empty() || existence::creates_on_unknown(command) {
            continue;
        }
        let command_ref = CommandRef::new(command.name.clone());
        // A declared `unknown_instance:` branch (ess/15) is the first answer, before a not-found
        // refusal and before `wrong_state`.
        if let Some(declared) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::UnknownInstance)
        {
            let id = ScenarioId::Outcome {
                outcome: OutcomeRef::new(command_ref, declared.name.clone()),
            };
            match unknown_instance(ir, command, &acting, declared, actors) {
                Ok(scenario) => insert(suite, id, scenario, refusals),
                Err(cause) => refusals.push(Refusal::about(&id, cause)),
            }
            continue;
        }
        match not_found(ir, command, &acting).as_slice() {
            [] => {}
            [answer] => {
                let id = ScenarioId::Outcome {
                    outcome: OutcomeRef::new(command_ref, answer.name.clone()),
                };
                if let Ok(scenario) = unknown_instance(ir, command, &acting, answer, actors) {
                    suite.scenarios.remove(&id);
                    insert(suite, id, scenario, refusals);
                }
                continue;
            }
            several => {
                notes.push(Note::UnknownInstanceAmbiguous {
                    command: command_ref,
                    outcomes: several.iter().map(|outcome| outcome.name.clone()).collect(),
                });
                continue;
            }
        }
        let Some(declared) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::WrongState)
        else {
            notes.push(Note::UnknownInstanceUnanswered {
                command: command_ref,
            });
            continue;
        };
        let id = ScenarioId::Outcome {
            outcome: OutcomeRef::new(command_ref, declared.name.clone()),
        };
        match unknown_instance(ir, command, &acting, declared, actors) {
            Ok(scenario) => insert(suite, id, scenario, refusals),
            Err(cause) => refusals.push(Refusal::about(&id, cause)),
        }
    }
}

/// The outcomes a command declares as its answer for an identity naming no record.
///
/// An outcome qualifies when it is an externally decided refusal (`external:` with no input guard,
/// acting on no instance, replaying nothing) whose declared error carries a field of the type of an
/// identity the command's acting branches read from input — `not-found`, reporting
/// `DoorNotFound { door_id }` for a command whose moves take `instance: door_id`. Such an error
/// reports the identity it could not find; a sibling external refusal reporting something else
/// (`Jammed { force }`) is about another input and does not qualify.
///
/// The declaration is read, not guessed: no outcome or error name is consulted. More than one
/// qualifying outcome is returned as it is, and the caller treats it as undeclared.
fn not_found<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
    acting: &[&ResolvedOutcome],
) -> Vec<&'a ResolvedOutcome> {
    let identities: BTreeSet<&ResolvedTypeRef> = acting
        .iter()
        .filter_map(|outcome| names_existing(outcome))
        .filter_map(|field| command.input.iter().find(|input| input.name == field))
        .map(|input| input.type_ref.required())
        .collect();
    command
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome.subject.is_none()
                && outcome.replays.is_none()
                && outcome.refuses
                && matches!(outcome.condition, ResolvedCondition::External { .. })
        })
        .filter(|outcome| {
            outcome.error.as_ref().is_some_and(|error| {
                ir.error(error)
                    .fields
                    .iter()
                    .any(|field| identities.contains(field.type_ref.required()))
            })
        })
        .collect()
}

/// The one outcome [`not_found`] reads as `command`'s answer for an identity naming no record, over
/// its branches acting on an input-named instance; `None` where it declares none, or several. The
/// model interpreter answers a never-created identity with it, after an `unknown_instance:` branch
/// and before `wrong_state`, as synthesis does (beyond10x/ess#291).
pub(crate) fn declared_not_found<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
) -> Option<&'a ResolvedOutcome> {
    let acting: Vec<&ResolvedOutcome> = command
        .outcomes
        .iter()
        .filter(|outcome| names_existing(outcome).is_some())
        .collect();
    match not_found(ir, command, &acting).as_slice() {
        [answer] => Some(answer),
        _ => None,
    }
}

/// The scenario itself: the command, sent for an identity no record carries, answering `declared`.
///
/// The input is the one that reaches a branch acting on an existing instance, for the reason
/// [`refused_here`] gives: a branch decided by input first (`PayInvoice/rejected`) answers its
/// own error whatever the instance names, and sending that input would prove nothing about the
/// rule. The identity in it is then replaced by one no other scenario sends.
fn unknown_instance(
    ir: &EssIr,
    command: &ResolvedCommand,
    acting: &[&ResolvedOutcome],
    declared: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let mut last = None;
    let mut reached = None;
    for outcome in acting {
        // A command reading a related row through an Optional input (ess/22, #304) reaches the
        // branch with the reference left out: no related row is read, so none is arranged. One
        // reading it through a stored field of the addressed subject, Optional or not, reads it
        // only after the subject's existence has answered, and an unknown subject has none.
        let input = if related_guard::optional(command)
            || related_guard::stored::field(command).is_some()
        {
            related_guard::absent_input(ir, command, outcome, Distinction::PLAIN)
        } else {
            reach(ir, command, outcome, Distinction::PLAIN)
        };
        match input {
            Ok(input) => {
                reached = Some((*outcome, input));
                break;
            }
            Err(cause) => last = Some(cause),
        }
    }
    let Some((attempt, mut input)) = reached else {
        return Err(last.unwrap_or(RefusalCause::StrategyWithoutGuard {
            strategy: declared.test_strategy,
        }));
    };
    let field = names_existing(attempt).unwrap_or_default();
    let fresh = fresh_identity(ir, command, field, Some(&input))?;
    input.insert(field.to_owned(), fresh);

    let command_ref = CommandRef::new(command.name.clone());
    let branch = OutcomeRef::new(command_ref.clone(), declared.name.clone());
    let supplied = supply(command, &input, None, None, &BTreeMap::new());
    let mut steps = vec![
        ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
            input: supplied.clone(),
        },
        ScenarioStep::ExpectOutcome {
            outcome: branch.clone(),
        },
    ];
    let mut source: BTreeSet<EssSemanticRef> = BTreeSet::new();
    source.insert(command_ref.into());
    source.insert(branch.into());
    if let Some(subject) = &attempt.subject {
        source.insert(EntityRef::from(&subject.entity).into());
    }
    if let Some(actor) = actors.get(&command.name) {
        source.insert(actor.clone().into());
    }
    let reported = declared
        .error
        .as_ref()
        .filter(|_| declared.refuses)
        .map(|error| {
            let named = ErrorRef::from(error);
            // No row carries the identity, so nothing is read from one.
            steps.push(expect_error(
                ir,
                declared,
                error,
                &supplied,
                &BTreeMap::new(),
            ));
            source.insert(named.clone().into());
            named
        });
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(forbidden.into_iter().map(EssSemanticRef::from));

    let text = match &reported {
        Some(error) => format!(
            "`{}` for an identity no record carries takes `{}` and reports `{error}`",
            command.name, declared.name
        ),
        None => format!(
            "`{}` for an identity no record carries takes `{}`",
            command.name, declared.name
        ),
    };
    Ok(ConformanceScenario::new(clipped(&text), steps, source))
}

/// The branch a command answers for an identity no record carries, in the order the design fixes:
/// a declared `unknown_instance:` branch (ess/15), then one declared not-found refusal, then
/// `wrong_state`. `None` where it declares none of them, or two not-found candidates.
fn unknown_answer<'c>(ir: &EssIr, command: &'c ResolvedCommand) -> Option<&'c ResolvedOutcome> {
    if let Some(declared) = command
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::UnknownInstance)
    {
        return Some(declared);
    }
    let acting: Vec<&ResolvedOutcome> = command
        .outcomes
        .iter()
        .filter(|outcome| names_existing(outcome).is_some())
        .collect();
    match not_found(ir, command, &acting).as_slice() {
        [answer] => Some(answer),
        [] => command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::WrongState),
        _ => None,
    }
}

/// What a `deletes:` branch (ess/15) is witnessed by, after its own assertions: every immediate
/// row view of the entity holds no row with the removed identity, and the same command sent for
/// it again takes the unknown-instance answer — a deleted identity is one no record carries.
///
/// Immediate views only, as preservation is (`docs/design/outcome-shapes.md`, open question 2): an
/// eventual view may still show the row, and waiting for its absence is a claim about lag.
fn deletion_witness(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    run: &Run,
    source: &mut BTreeSet<EssSemanticRef>,
) -> Vec<ScenarioStep> {
    let mut steps = Vec::new();
    let Some(subject) = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == ResolvedEffect::Deletes)
    else {
        return steps;
    };
    let projections = row_projections(ir);
    for view in projections.get(&subject.entity).into_iter().flatten() {
        if view.consistency != ess_domain::view::Consistency::ReadYourWrites
            || !paging::read_whole(view)
        {
            continue;
        }
        let identity = identifying(ir, subject, run.instance.as_ref(), view);
        if identity.is_empty() {
            continue;
        }
        let name = ViewRef::new(view.name.clone());
        steps.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        steps.push(ScenarioStep::ExpectSubjectAbsent {
            view: name.clone(),
            subject: identity,
        });
        source.insert(name.into());
    }
    if let Some(answer) = unknown_answer(ir, command) {
        let command_ref = CommandRef::new(command.name.clone());
        let branch = OutcomeRef::new(command_ref.clone(), answer.name.clone());
        steps.push(ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: command_ref,
            actor: run.actor.clone(),
            input: run.input.clone(),
        });
        steps.push(ScenarioStep::ExpectOutcome {
            outcome: branch.clone(),
        });
        source.insert(branch.into());
        if let Some(error) = answer.error.as_ref().filter(|_| answer.refuses) {
            // The row is gone, so nothing is read from one.
            steps.push(expect_error(
                ir,
                answer,
                error,
                &run.input,
                &BTreeMap::new(),
            ));
            source.insert(ErrorRef::from(error).into());
        }
        // The answer publishes nothing it does not declare, so a deleted identity answered with
        // its old event again is caught.
        if answer.emits.is_empty() {
            steps.push(ScenarioStep::ExpectNoEvents);
        }
    }
    steps
}

/// What an `accepts: nothing` branch (ess/15) is witnessed by: no error, no direct event of any
/// name, and every immediate view read whole before and after it holding the same rows.
fn accepts_nothing(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    setup: &mut Setup,
    invoke: &mut Vec<ScenarioStep>,
    after: &mut Vec<ScenarioStep>,
) {
    if !outcome.accepts_nothing {
        return;
    }
    invoke.push(ScenarioStep::ExpectNoError);
    invoke.push(ScenarioStep::ExpectNoEvents);
    for view in whole_views(ir) {
        for steps in [&mut setup.steps, &mut *after] {
            steps.push(ScenarioStep::QueryView {
                view: view.clone(),
                params: BTreeMap::new(),
            });
        }
        setup
            .steps
            .push(ScenarioStep::SnapshotView { view: view.clone() });
        after.push(ScenarioStep::ExpectViewUnchanged { view: view.clone() });
        setup.source.insert(view.into());
    }
}

/// Every immediate view a scenario can read whole: read-your-writes, no parameters. What an
/// `accepts: nothing` branch (ess/15) is required to leave exactly as it was.
fn whole_views(ir: &EssIr) -> Vec<ViewRef> {
    ir.views()
        .values()
        .filter(|view| {
            view.consistency == ess_domain::view::Consistency::ReadYourWrites
                && paging::read_whole(view)
        })
        .map(|view| ViewRef::new(view.name.clone()))
        .collect()
}

/// The system's preconditions (ess/15), prepended to every scenario: each command sent as its
/// resolved actor with its literal and fixture inputs, and required to take its success branch.
fn preconditions(models: &caller::InvocationModels<'_>, suite: &mut ConformanceSuite) {
    let ir = models.arrangement;
    // One entry per precondition, in order: its two steps, and what it would create again — its
    // command and the value it sends for an input that becomes the created identity.
    let mut chunks: Vec<(Vec<ScenarioStep>, Vec<Recreated>)> = Vec::new();
    for precondition in ir.preconditions() {
        let mut prelude = Vec::new();
        let mut creates = Vec::new();
        let command = ir.command(&precondition.command);
        let command_ref = CommandRef::new(command.name.clone());
        let mut input: BTreeMap<String, ScenarioValue> = precondition
            .input
            .iter()
            .map(|(field, value)| (field.clone(), ScenarioValue::literal(value.clone())))
            .collect();
        for (field, fixture) in &precondition.fixtures {
            input.insert(
                field.clone(),
                ScenarioValue::Fixture {
                    fixture: fixture.clone(),
                },
            );
        }
        for field in identity_inputs(command) {
            if let Some(value) = input.get(&field) {
                creates.push((command_ref.clone(), field, value.clone()));
            }
        }
        prelude.push(ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: command_ref.clone(),
            actor: precondition
                .actor
                .as_ref()
                .map(|actor| ActorRef::new(actor.name().clone())),
            input,
        });
        // Always required: `ess-domain` admits a precondition only where its input selects
        // exactly one branch reporting no error, so a refused one fails the scenario as setup.
        prelude.push(ScenarioStep::ExpectOutcome {
            outcome: OutcomeRef::new(command_ref, precondition.outcome.clone()),
        });
        models.mark(caller::InvocationPhase::Arrange, &mut prelude);
        chunks.push((prelude, creates));
    }
    for scenario in suite.scenarios.values_mut() {
        // A scenario that sends a precondition's command for the identity that precondition
        // creates would create it again over an existing record. It keeps the preconditions before
        // that one and drops it and every later one: its own send is what that precondition did,
        // and a later one may depend on it.
        let kept = chunks
            .iter()
            .position(|(_, creates)| recreates(&scenario.steps, creates))
            .unwrap_or(chunks.len());
        let prelude: Vec<ScenarioStep> = chunks[..kept]
            .iter()
            .flat_map(|(steps, _)| steps.iter().cloned())
            .collect();
        if prelude.is_empty() {
            continue;
        }
        let mut steps = prelude.clone();
        steps.append(&mut scenario.steps);
        scenario.steps = steps;
        for step in &prelude {
            match step {
                ScenarioStep::ExecuteCommand { command, .. } => {
                    scenario.source.insert(command.clone().into());
                }
                ScenarioStep::ExpectOutcome { outcome } => {
                    scenario.source.insert(outcome.clone().into());
                }
                _ => {}
            }
        }
    }
}

/// The input fields whose value becomes an identity one of the command's branches creates: the
/// `input.` source its payload declares for the event field the creation publishes the identity in.
fn identity_inputs(command: &ResolvedCommand) -> BTreeSet<String> {
    let mut fields = BTreeSet::new();
    for outcome in &command.outcomes {
        let Some(subject) = &outcome.subject else {
            continue;
        };
        let ResolvedInstance::Observed { event, field } = &subject.instance else {
            continue;
        };
        for payload in outcome
            .payload
            .iter()
            .filter(|payload| &payload.event == event)
        {
            for entry in payload
                .fields
                .iter()
                .filter(|entry| entry.target == field.name)
            {
                if let ResolvedPayloadValue::InputField { field, .. } = &entry.value {
                    fields.insert(field.clone());
                }
            }
        }
    }
    fields
}

/// What a creating precondition would create again: its command, and the value it sends for an
/// input that becomes the created identity.
type Recreated = (CommandRef, String, ScenarioValue);

/// Whether `steps` send a creating precondition's command with the identity it already sends.
fn recreates(steps: &[ScenarioStep], creates: &[Recreated]) -> bool {
    steps.iter().any(|step| {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            return false;
        };
        creates
            .iter()
            .any(|(creating, field, value)| creating == command && input.get(field) == Some(value))
    })
}

/// How far either side of each value a guard's ladder gives a number [`guided_values`] moves it:
/// enough fresh identities inside a guard's interval for every slot a suite draws from one.
const GUIDED_SPAN: u32 = 128;

/// Every guard that reads a command's input: each branch's `when:`, an external branch's input
/// guard, and the input half of a stored-row or related-row guard.
pub(super) fn input_guards(command: &ResolvedCommand) -> Vec<&Predicate> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::ExternalWhen { predicate, .. } => Some(predicate),
            ResolvedCondition::SubjectPredicate { input, .. }
            | ResolvedCondition::Related { input, .. } => input.as_ref(),
            _ => when(outcome),
        })
        .collect()
}

/// Whether `value` may stand for `field` in `input` without changing the branch the command
/// takes (beyond10x/ess#275): a value of the field's type that decides every input guard as
/// `input` decides it and refutes no entity invariant `input` meets.
///
/// Where `input` cannot be flattened — it holds a value only a run binds — there is no decision to
/// keep, and the value is taken.
pub(super) fn keeps_branch(
    ir: &EssIr,
    command: &ResolvedCommand,
    input: &BTreeMap<String, Node>,
    field: &str,
    value: &Node,
) -> bool {
    let Some(declared) = command.input.iter().find(|declared| declared.name == field) else {
        return false;
    };
    if crate::input::validate_typed_value(ir, &declared.type_ref, value).is_err() {
        return false;
    }
    let mut moved = input.clone();
    moved.insert(field.to_owned(), value.clone());
    let Ok(before) = flatten(ir, command, input) else {
        return true;
    };
    let Ok(after) = flatten(ir, command, &moved) else {
        return false;
    };
    let decided = |facts: &crate::InputFacts<'_>, guard: &Predicate| {
        std::mem::discriminant(&facts.decide(guard))
    };
    input_guards(command)
        .into_iter()
        .all(|guard| decided(&before, guard) == decided(&after, guard))
        && command.outcomes.iter().all(|outcome| {
            crate::witness::invariant_broken_by(ir, command, outcome, input).is_some()
                || crate::witness::invariant_broken_by(ir, command, outcome, &moved).is_none()
        })
}

/// The values the input guards' ladder gives `field`: the literals they compare it with and one
/// either side, and the values deciding sign.
fn guard_ladder(ir: &EssIr, command: &ResolvedCommand, field: &str) -> Vec<Node> {
    let guards = input_guards(command);
    if guards.is_empty() {
        return Vec::new();
    }
    let mut ladder: Vec<Node> = Vec::new();
    for input in candidates(ir, command, &guards, Distinction::PLAIN).unwrap_or_default() {
        if let Some(value) = input.get(field) {
            if !ladder.contains(value) {
                ladder.push(value.clone());
            }
        }
    }
    ladder
}

/// The values `field` is tried at where its far witnesses break a guard the input meets: each value
/// of the guards' ladder, then each number of it moved one, two, … up to [`GUIDED_SPAN`] either way,
/// nearest first — values inside every interval the guards' literals bound, derived from what the
/// guards write.
pub(super) fn guided_values(ir: &EssIr, command: &ResolvedCommand, field: &str) -> Vec<Node> {
    let ladder = guard_ladder(ir, command, field);
    let numbers: Vec<f64> = ladder
        .iter()
        .filter_map(|value| match value {
            Node::Number(number) => Some(number.get()),
            _ => None,
        })
        .collect();
    let mut values = ladder;
    for step in 1..=GUIDED_SPAN {
        for number in &numbers {
            for moved in [number - f64::from(step), number + f64::from(step)] {
                if let Ok(moved) = ess_primitives::facts::Number::new(moved) {
                    let moved = Node::Number(moved);
                    if !values.contains(&moved) {
                        values.push(moved);
                    }
                }
            }
        }
    }
    values
}

/// The `nth` value of `field` that keeps the branch `input` takes ([`keeps_branch`]) and that no
/// arrangement sends: neither the far witness of any distinction an arrangement numbers or of
/// [`Distinction::UNKNOWN`], nor a value of the guards' ladder, which an arrangement whose far
/// witness breaks the guard sends instead. `None` where the guards leave fewer than `nth + 1`.
pub(super) fn guided_identity(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    input: &BTreeMap<String, Node>,
    nth: usize,
) -> Option<Node> {
    let mut arranged: Vec<Node> = guard_ladder(ir, command, field);
    for distinction in (0..=MAX_CANDIDATES)
        .map(Distinction::further)
        .chain([Distinction::UNKNOWN])
    {
        if let Some(value) = candidates(ir, command, &[], distinction)
            .ok()
            .and_then(|inputs| inputs.into_iter().next())
            .and_then(|mut input| input.remove(field))
        {
            arranged.push(value);
        }
    }
    guided_values(ir, command, field)
        .into_iter()
        .filter(|value| !arranged.contains(value))
        .filter(|value| keeps_branch(ir, command, input, field, value))
        .nth(nth)
}

/// Why a guarded identity has no fresh value: the guards the input meets leave too few.
pub(super) fn unguided(command: &ResolvedCommand, field: &str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path: field.to_owned(),
        type_ref: command
            .input
            .iter()
            .find(|input| input.name == field)
            .map(|input| input.type_ref.to_string())
            .unwrap_or_default(),
        reason: "has too few values inside the guards the input meets to name an identity no \
                 other scenario sends, so no instance is known to be new or unknown",
    })
}

/// A value of the identity field that no other scenario sends.
///
/// The witness at [`Distinction::UNKNOWN`], checked against the witness at every distinction an
/// arrangement numbers. Where the type has too few values to keep it apart — a `Boolean`, a
/// `Timestamp` a month wide — no identity is known to name no record on a target the scenarios
/// share, and the scenario is refused rather than asserted on a record another scenario made.
///
/// Where that witness would change the branch `input` takes — a guard reads the identity — the
/// first value inside the guards that no arrangement sends is taken instead
/// ([`guided_identity`]), and the scenario is refused where there is none (beyond10x/ess#275).
///
/// An identity whose type has one value is that value, unchecked against the arrangements: the one
/// row of a singleton entity is unknown in a scenario that arranged none (beyond10x/ess#287,
/// [`singleton`]).
fn fresh_identity(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    input: Option<&BTreeMap<String, Node>>,
) -> Result<Node, RefusalCause> {
    let at = |distinction: Distinction| {
        candidates(ir, command, &[], distinction)
            .map_err(RefusalCause::NoWitness)
            .map(|inputs| {
                inputs
                    .into_iter()
                    .next()
                    .and_then(|input| input.get(field).cloned())
            })
    };
    let unfresh = || {
        RefusalCause::NoWitness(WitnessGap {
            path: field.to_owned(),
            type_ref: command
                .input
                .iter()
                .find(|input| input.name == field)
                .map(|input| input.type_ref.to_string())
                .unwrap_or_default(),
            reason: "has too few values to name an identity no other scenario sends, so no \
                     instance is known to be unknown",
        })
    };
    let fresh = at(Distinction::UNKNOWN)?.ok_or_else(unfresh)?;
    if let Some(input) = input {
        if !keeps_branch(ir, command, input, field, &fresh) {
            // The unknown identity takes the first guided value; the existence family's slots
            // take the ones after it.
            return guided_identity(ir, command, field, input, 0)
                .ok_or_else(|| unguided(command, field));
        }
    }
    // The one row of a singleton entity is unknown wherever the scenario arranged none
    // (beyond10x/ess#287): its isolation, not a value apart from every other scenario's, keeps it so.
    if singleton::names_the_one_row(ir, command, field) {
        return Ok(fresh);
    }
    for nth in 0..=MAX_CANDIDATES {
        if at(Distinction::further(nth))?.as_ref() == Some(&fresh) {
            return Err(unfresh());
        }
    }
    Ok(fresh)
}

/// One line saying which move a transition scenario proves, and by which verb.
fn refusal_arrangement(
    ir: &EssIr,
    handle: &EntityHandle,
    state: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    attempt: &Driver<'_>,
) -> Result<RefusalArrangement, RefusalCause> {
    let mut arrangement =
        arrange(ir, handle, state, actors, Distinction::PLAIN, &[]).map_err(|reason| {
            RefusalCause::InstanceRequired {
                entity: EntityRef::from(handle),
                need: InstanceNeed::InState {
                    state: state.clone(),
                },
                reason,
            }
        })?;
    // A command reading stored fields selects no branch from a state it does not run from, so the
    // input is chosen for what it alone decides rather than asked of `reach`, whose stateless arm
    // has no guard to offer a subject-fact branch (beyond10x/ess#173).
    if subject_fact::uses(attempt.command) && !has_subject_guards(attempt.command) {
        return subject_fact::refusal_witness(
            ir,
            handle,
            state,
            actors,
            arrangement,
            attempt.command,
            attempt.outcome,
            Distinction::PLAIN,
        )
        .map(|(arrangement, input)| (arrangement, input, BTreeMap::new()));
    }
    if related_guard::orders_present_related_refusal(ir, attempt.command) {
        let (input, bound) =
            related_guard::wrong_state_overlap(ir, attempt.command, actors, &mut arrangement)?;
        return Ok((arrangement, input, bound));
    }
    let input = reach(ir, attempt.command, attempt.outcome, Distinction::PLAIN)?;
    Ok((arrangement, input, BTreeMap::new()))
}

/// Full refusal observation is an explicit compiler obligation of the new source profile.
fn complete_wrong_state(
    ir: &EssIr,
    attempt: &Driver<'_>,
    arrangement: &Arrangement,
) -> Result<Option<subject_fact::Preservation>, RefusalCause> {
    if !attempt
        .command
        .outcomes
        .iter()
        .any(|outcome| outcome.complete_refusal)
    {
        return Ok(None);
    }
    let setup = Setup {
        instance: Some(arrangement.instance.clone()),
        after: Some(arrangement.state.clone()),
        settled: arrangement.settled.clone(),
        ..Setup::none()
    };
    subject_fact::preserve_refused_subject(ir, subject(attempt), &setup).map(Some)
}

/// One line saying which move a transition scenario proves, and by which verb.
fn moving(
    entity: &EntityRef,
    transition: &ess_domain::entity::Transition,
    driver: &Driver<'_>,
) -> ScenarioPurpose {
    clipped(&format!(
        "`{}` on `{}` moves a `{entity}` to `{}` along `{}`",
        driver.command.name, driver.outcome.name, transition.to, transition.name
    ))
}

/// The instance names a scenario's steps already bind.
fn bound_instances(steps: &[ScenarioStep]) -> BTreeSet<InstanceName> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::CaptureInstance { instance, .. }
            | ScenarioStep::EstablishEntity { instance, .. } => Some(instance.clone()),
            _ => None,
        })
        .collect()
}

/// One further instance of `entity`, resting in `state`, under names no step of the scenario has
/// bound yet — its own and those of whatever it arranged on the way, an owner included.
///
/// The ordinal is searched rather than counted from a known offset, because a scenario's other
/// further instances — a ranked view's companions, a stored-field guard's boundary rows — are
/// numbered by code that does not know about this one.
fn arrange_unbound(
    ir: &EssIr,
    entity: &EntityHandle,
    state: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
) -> Result<Arrangement, Unreachable> {
    for nth in 1..=MAX_CANDIDATES {
        let arrangement = arrange(ir, entity, state, actors, Distinction::further(nth), &[])?;
        let names = bound_instances(&arrangement.steps);
        if names.is_disjoint(taken) {
            taken.extend(names);
            return Ok(arrangement);
        }
    }
    // Unreachable for any scenario this module builds: it binds far fewer than this many names.
    Err(Unreachable::NoPath {
        from: ir.entity(entity).lifecycle.initial.clone(),
    })
}

/// The transition run from each of its other `from` states, each on its own instance
/// (beyond10x/ess#111).
///
/// `close: from [Open, Held]` is two promises, and the scenario [`exercise`] builds keeps one of
/// them: it arranges the cheapest source, so an implementation that dropped `Held` from the move
/// passed. So after it, for every further source the branch admits, one more order is arranged in
/// that state, the command is sent for it, the branch and its events are required, and every row
/// view that shows the result is required to hold it in the state the move arrives at. The id is
/// the transition's, unchanged: the claim is the move, and the move is every source.
///
/// A source no arrangement reaches is recorded as a refusal beside the scenario, never in place of
/// it — the check it would have been is missing, and that is only visible if it is named. A branch
/// that reads the subject's stored fields is arranged by a search of its own and is not repeated
/// here.
#[allow(clippy::too_many_arguments)]
fn other_sources(
    models: &caller::InvocationModels<'_>,
    driver: &Driver<'_>,
    transition: &ess_domain::entity::Transition,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    run: &Run,
    steps: &[ScenarioStep],
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> (Vec<ScenarioStep>, BTreeSet<EssSemanticRef>) {
    let mut out = (Vec::new(), BTreeSet::new());
    let (command, outcome) = (driver.command, driver.outcome);
    let Some(exercised) = &run.before else {
        return out;
    };
    if outcome.replays.is_some() || subject_fact::routes(command, outcome) {
        return out;
    }
    let mut taken = bound_instances(steps);
    for from in &transition.from {
        if from == exercised || !admits_held_state(&outcome.condition, from) {
            continue;
        }
        match from_source(models, driver, from, &transition.to, actors, &mut taken) {
            Ok((further, depends)) => {
                out.0.extend(further);
                out.1.extend(depends);
            }
            Err(cause) => refusals.push(Refusal::about(id, cause)),
        }
    }
    out
}

/// One further source of a transition: arrange an instance in `from`, move it, and require it moved.
#[allow(clippy::too_many_lines)] // One transition witness and its phase-specific observations.
fn from_source(
    models: &caller::InvocationModels<'_>,
    driver: &Driver<'_>,
    from: &StateName,
    to: &StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ir = models.arrangement;
    let (command, outcome) = (driver.command, driver.outcome);
    let subject = subject(driver);
    let arrangement =
        arrange_unbound(ir, &subject.entity, from, actors, taken).map_err(|reason| {
            RefusalCause::InstanceRequired {
                entity: EntityRef::from(&subject.entity),
                need: InstanceNeed::InState {
                    state: from.clone(),
                },
                reason,
            }
        })?;
    let input = if has_subject_guards(command) {
        reach_in_state(ir, command, outcome, from, Distinction::PLAIN)?
    } else {
        reach(ir, command, outcome, Distinction::PLAIN)?
    };
    let input = freshened(
        ir,
        command,
        outcome,
        input,
        Some(from),
        &arrangement.settled,
    );
    let mut steps = arrangement.steps;
    models.mark(caller::InvocationPhase::Arrange, &mut steps);
    let mut source = arrangement.source;
    if has_subject_guards(command) {
        let (observed, view) = observe_subject_state(ir, outcome, &arrangement.instance, from)?;
        steps.extend(observed);
        source.insert(view.into());
    }
    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    let supplied = supply(
        command,
        &input,
        Some(subject),
        Some(&arrangement.instance),
        &BTreeMap::new(),
    );
    if outcome.test_strategy == TestStrategy::InjectFault {
        steps.push(ScenarioStep::ConfigureExternalOutcome {
            force: outcome_ref.clone(),
            times: None,
        });
    }
    steps.push(ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supplied.clone(),
    });
    steps.push(ScenarioStep::ExpectOutcome {
        outcome: outcome_ref.clone(),
    });
    for event in outcome.emits.iter().map(EventRef::from) {
        steps.push(expect_event_step(
            &event,
            determined_payload(ir, outcome, &event, &supplied, &arrangement.settled),
            determined_identities(
                ir,
                outcome,
                &event,
                &supplied,
                &arrangement.settled,
                Some(&arrangement.instance),
            ),
            crate::response::event_shape(ir, &event, outcome),
        ));
    }
    let determined = settled(ir, outcome, &supplied, &arrangement.settled);
    let before = arrangement.settled.clone();
    let mut left = arrangement.settled;
    absorb(&mut left, outcome, determined);
    // The reads go after the command in a block of their own: a `QueryView` the arrangement made
    // is a read of the row before it moved.
    let mut asserted = Vec::new();
    for view in row_projections(ir)
        .get(&subject.entity)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let known = identity_of(subject, Some(&arrangement.instance));
        let params = bound(ir, view, &left, known.as_ref());
        if shows_row(ir, view, to, &left, known.as_ref(), &params) != Ok(true) {
            continue;
        }
        let identity = identifying(ir, subject, Some(&arrangement.instance), view);
        let mut row = shown(view, identity.clone(), &left);
        let required = shown_leaves(ir, view, outcome, &supplied, &before, &mut row);
        row.extend(lifecycle_state(ir, &subject.entity, view, to));
        let name = ViewRef::new(view.name.clone());
        require(
            view,
            &name,
            params.clone(),
            ViewExpectation::Contains { fields: row },
            &mut asserted,
        );
        for claim in present_leaves(&identity, required) {
            require(view, &name, params.clone(), claim, &mut asserted);
        }
        source.insert(name.into());
    }
    steps.extend(asserted);
    models.mark(caller::InvocationPhase::Act, &mut steps);
    source.insert(command_ref.into());
    source.insert(outcome_ref.into());
    Ok((steps, source))
}

/// One ordered comparison of a guard's conjunct against a literal, read with the fact on the left.
struct Bound<'p> {
    /// Which conjunct of the guard it is.
    conjunct: usize,
    /// The input fact it reads.
    path: &'p FactPath,
    /// The operator, with the fact on the left.
    op: ess_primitives::predicate::CompareOp,
    /// The literal it compares against.
    literal: &'p ess_primitives::facts::FactValue,
}

impl Bound<'_> {
    /// The value at the boundary that satisfies the comparison: the literal itself for `>=` and
    /// `<=`, the neighbour inside it for `>` and `<`.
    fn accepting(&self) -> Option<Node> {
        use ess_primitives::predicate::CompareOp;
        if self.reads_current_time() {
            return self.current_time_margin(true);
        }
        match self.op {
            CompareOp::Ge | CompareOp::Le => self.stepped(0),
            CompareOp::Gt => self.stepped(1),
            CompareOp::Lt => self.stepped(-1),
            CompareOp::Eq | CompareOp::Ne => None,
        }
    }

    /// The neighbour across the boundary that refutes it: the literal itself for `>` and `<`, the
    /// neighbour outside it for `>=` and `<=`.
    fn refuting(&self) -> Option<Node> {
        use ess_primitives::predicate::CompareOp;
        if self.reads_current_time() {
            return self.current_time_margin(false);
        }
        match self.op {
            CompareOp::Gt | CompareOp::Lt => self.stepped(0),
            CompareOp::Ge => self.stepped(-1),
            CompareOp::Le => self.stepped(1),
            CompareOp::Eq | CompareOp::Ne => None,
        }
    }

    /// The literal moved `by` steps of its own kind: a whole number for a number — the neighbour
    /// of an `Integer`, and the step the witness ladder already orders a `Decimal` by — and a
    /// second for an instant. Nothing for any other literal: text orders by its bytes, and the
    /// "next" text is not a neighbour a mutant moves a boundary to.
    fn stepped(&self, by: i8) -> Option<Node> {
        if let Some(number) = self.literal.as_number() {
            return ess_primitives::facts::Number::new(number.get() + f64::from(by))
                .ok()
                .map(Node::Number);
        }
        let text = self.literal.as_text()?;
        let instant = Rfc3339Instant::parse_rfc3339(text)?;
        if by == 0 {
            return Some(Node::Text(text.to_owned()));
        }
        instant
            .plus_seconds(i64::from(by))
            .map(|moved| Node::Text(moved.to_rfc3339()))
    }
}

impl Bound<'_> {
    /// Whether the literal is the current-time operand (beyond10x/ess#171).
    fn reads_current_time(&self) -> bool {
        self.literal
            .as_text()
            .is_some_and(|text| ess_primitives::time::CurrentTime::parse(text).is_some())
    }

    /// For a bound against the current time, the value a second from the boundary on the side
    /// asked for — inside it when `accepting`, outside it otherwise — and never the boundary
    /// itself, which a latency flips: see [`crate::now_offset`].
    fn current_time_margin(&self, accepting: bool) -> Option<Node> {
        use ess_primitives::predicate::CompareOp;
        let now = ess_primitives::time::CurrentTime::parse(self.literal.as_text()?)?;
        let inside: i64 = match self.op {
            CompareOp::Gt | CompareOp::Ge => 1,
            CompareOp::Lt | CompareOp::Le => -1,
            CompareOp::Eq | CompareOp::Ne => return None,
        };
        now.at(crate::now_offset::reference())?
            .plus_seconds(if accepting { inside } else { -inside })
            .map(|moved| Node::Text(moved.to_rfc3339()))
    }
}

/// A guard's conjuncts: the leaves of its top-level `all`, flattened, or the guard itself.
fn conjuncts(guard: &Predicate) -> Vec<&Predicate> {
    match guard {
        Predicate::All(children) => children.iter().flat_map(conjuncts).collect(),
        other => vec![other],
    }
}

/// Every conjunct of a guard that compares an input fact with a literal by an order.
fn ordered_bounds<'p>(conjuncts: &[&'p Predicate]) -> Vec<Bound<'p>> {
    use ess_primitives::predicate::CompareOp;
    let mut found = Vec::new();
    for (index, conjunct) in conjuncts.iter().enumerate() {
        let Predicate::Compare { left, op, right } = conjunct else {
            continue;
        };
        let (path, op, literal) = match (left, right) {
            (Operand::Fact(path), Operand::Literal(literal)) => (path, *op, literal),
            (Operand::Literal(literal), Operand::Fact(path)) => (
                path,
                match op {
                    CompareOp::Lt => CompareOp::Gt,
                    CompareOp::Le => CompareOp::Ge,
                    CompareOp::Gt => CompareOp::Lt,
                    CompareOp::Ge => CompareOp::Le,
                    other => *other,
                },
                literal,
            ),
            _ => continue,
        };
        if matches!(op, CompareOp::Eq | CompareOp::Ne) {
            continue;
        }
        found.push(Bound {
            conjunct: index,
            path,
            op,
            literal,
        });
    }
    found
}

/// `input` with the scalar at `path` replaced, or nothing where the path does not land on one.
fn with_leaf(
    input: &BTreeMap<String, Node>,
    path: &FactPath,
    value: Node,
) -> Option<BTreeMap<String, Node>> {
    if let Some(resized) = with_count(input, path, &value) {
        return Some(resized);
    }
    let (first, rest) = path.segments().split_first()?;
    let mut out = input.clone();
    let mut at = out.get_mut(first)?;
    for segment in rest {
        at = match at {
            Node::Map(members) => members.get_mut(segment)?,
            Node::Seq(elements) => elements.get_mut(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    if !matches!(at, Node::Number(_) | Node::Text(_)) {
        return None;
    }
    *at = value;
    Some(out)
}

/// `input` with the text or list that `<parent>.count` reads resized to `length`, or nothing where
/// the path reads no count or the parent is neither (beyond10x/ess#160).
///
/// `digits.count > 64` reads a length, not a leaf, so a boundary on it is moved by resizing what it
/// counts: a text keeps its own characters, cycled or cut, so an alphabet or pattern that admitted
/// it still admits the prefix; a list repeats or drops its last elements. Lengths above
/// [`MAX_COUNT_WITNESS`](crate::witness::MAX_COUNT_WITNESS) are not built, for the reason the
/// witness ladder does not build them.
fn with_count(
    input: &BTreeMap<String, Node>,
    path: &FactPath,
    length: &Node,
) -> Option<BTreeMap<String, Node>> {
    let (last, parent) = path.segments().split_last()?;
    if last != "count" || parent.is_empty() {
        return None;
    }
    let Node::Number(number) = length else {
        return None;
    };
    let wanted = number.get();
    if wanted.fract() != 0.0
        || !(0.0..=f64::from(u32::try_from(crate::witness::MAX_COUNT_WITNESS).ok()?))
            .contains(&wanted)
    {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let wanted = wanted as usize;
    let (first, rest) = parent.split_first()?;
    let mut out = input.clone();
    let mut at = out.get_mut(first)?;
    for segment in rest {
        at = match at {
            Node::Map(members) => members.get_mut(segment)?,
            Node::Seq(elements) => elements.get_mut(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    match at {
        Node::Text(text) => {
            let own: Vec<char> = text.chars().collect();
            if own.is_empty() && wanted > 0 {
                return None;
            }
            *text = own.iter().copied().cycle().take(wanted).collect();
        }
        Node::Seq(elements) => {
            let last = elements.last().cloned();
            if elements.len() > wanted {
                elements.truncate(wanted);
            } else {
                let filler = last?;
                elements.resize(wanted, filler);
            }
        }
        // A map only shrinks: a new entry needs a key of the declared key type, which this
        // type-blind resize cannot spell (beyond10x/ess#196).
        Node::Map(entries) if entries.len() >= wanted => {
            while entries.len() > wanted {
                entries.pop_last();
            }
        }
        _ => return None,
    }
    Some(out)
}

/// The first candidate for `guard` whose facts `holds` accepts, among the inputs the witness search
/// builds from the guard's own literals (beyond10x/ess#155).
fn first_where(
    ir: &EssIr,
    command: &ResolvedCommand,
    guard: &Predicate,
    holds: impl Fn(&crate::InputFacts<'_>) -> bool,
) -> Vec<BTreeMap<String, Node>> {
    let Ok(inputs) = candidates(ir, command, &[guard], Distinction::PLAIN) else {
        return Vec::new();
    };
    inputs
        .into_iter()
        .filter(|input| flatten(ir, command, input).is_ok_and(|facts| holds(&facts)))
        .collect()
}

/// Whether exactly the child at `index` decides `alone` and every other child the opposite.
fn exactly_one(
    facts: &crate::InputFacts<'_>,
    children: &[Predicate],
    index: usize,
    alone: bool,
) -> bool {
    children.iter().enumerate().all(|(other, child)| {
        let satisfied = match facts.decide(child) {
            Decision::Satisfied => true,
            Decision::Refuted(_) => false,
            Decision::Unevaluable(_) => return false,
        };
        satisfied == ((other == index) == alone)
    })
}

/// One candidate input, by field.
type Row = BTreeMap<String, Node>;

/// One further row per child of a connective with two or more children, where the child decides
/// `alone` and every other child the opposite (beyond10x/ess#155) — skipping a child the plain
/// witness or an earlier row already isolates, and one no candidate isolates.
#[allow(clippy::too_many_arguments)]
fn one_per_child(
    ir: &EssIr,
    command: &ResolvedCommand,
    guard: &Predicate,
    children: &[Predicate],
    alone: bool,
    primary: &Row,
    rows: &mut Vec<Row>,
    keep: &dyn Fn(Row, &mut Vec<Row>),
) {
    if children.len() < 2 {
        return;
    }
    for index in 0..children.len() {
        let isolates = |facts: &crate::InputFacts<'_>| exactly_one(facts, children, index, alone);
        if flatten(ir, command, primary).is_ok_and(|facts| isolates(&facts))
            || rows
                .iter()
                .any(|row| flatten(ir, command, row).is_ok_and(|facts| isolates(&facts)))
        {
            continue;
        }
        let before = rows.len();
        for candidate in first_where(ir, command, guard, isolates) {
            keep(candidate, rows);
            if rows.len() > before {
                break;
            }
        }
    }
}

/// The inputs a branch is further witnessed at so the boundaries of its guards are pinned
/// (beyond10x/ess#111), each distinct from `primary` and from each other.
///
/// `when: items >= 0` was accepted at `1` and refused at `-1`, so a target that moved the boundary
/// to `> 0` passed; `n >= 1`, one conjunct of two, was refused only through the other conjunct.
/// For each conjunct comparing an input fact with a literal by an order:
///
/// | branch | witnessed at | every other conjunct |
/// |---|---|---|
/// | the guarded branch itself | the boundary value that satisfies it — `0` for `>= 0`, `1` for `> 0` | as its plain witness left them |
/// | the default of a guarded sibling | the neighbour across it that refutes it — `-1` for `>= 0`, `0` for `> 0` | satisfied, so this conjunct is the one refusing |
///
/// Every row is decided again by [`selects_branch`] before it is kept, and a row equal to the plain
/// witness is not sent twice. Numbers step by one, instants by a second; text and equality have no
/// neighbour a boundary moves to.
fn boundary_inputs(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    primary: &BTreeMap<String, Node>,
) -> Vec<BTreeMap<String, Node>> {
    let mut rows: Vec<BTreeMap<String, Node>> = Vec::new();
    let keep = |candidate: BTreeMap<String, Node>, rows: &mut Vec<_>| {
        if &candidate != primary
            && !rows.contains(&candidate)
            && admitted(ir, command, &candidate)
            && selects_branch(ir, command, outcome, None, &candidate).unwrap_or(false)
        {
            rows.push(candidate);
        }
    };
    match outcome.test_strategy {
        TestStrategy::ConstructInput => {
            let Some(guard) = when(outcome) else {
                return rows;
            };
            let conjuncts = conjuncts(guard);
            for bound in ordered_bounds(&conjuncts) {
                if let Some(candidate) = bound
                    .accepting()
                    .and_then(|value| with_leaf(primary, bound.path, value))
                {
                    keep(candidate, &mut rows);
                }
            }
            // Each disjunct of an `any` holding alone (beyond10x/ess#155): a witness satisfying
            // every disjunct at once is also a witness of the `all` a connective mutant writes.
            for conjunct in &conjuncts {
                let Predicate::Any(children) = conjunct else {
                    continue;
                };
                one_per_child(
                    ir, command, guard, children, true, primary, &mut rows, &keep,
                );
            }
            for candidate in overlap_inputs(ir, command, outcome) {
                keep(candidate, &mut rows);
            }
        }
        TestStrategy::DefaultBranch => {
            for sibling in command.outcomes.iter().filter(|sibling| {
                sibling.name != outcome.name
                    && sibling.test_strategy == TestStrategy::ConstructInput
            }) {
                let Some(guard) = when(sibling) else {
                    continue;
                };
                // The sibling's own witness is the base a boundary is moved from, so every other
                // conjunct stays satisfied. Where the sibling has none (a guard nothing satisfies,
                // such as `digits.count < 0`), or its witness cannot be resized (an empty text
                // grown to a length), the default's own witness is the base (beyond10x/ess#160).
                let reference = reach(ir, command, sibling, Distinction::PLAIN).ok();
                let conjuncts = conjuncts(guard);
                for bound in ordered_bounds(&conjuncts) {
                    let Some(candidate) = bound.refuting().and_then(|value| {
                        reference
                            .as_ref()
                            .and_then(|base| with_leaf(base, bound.path, value.clone()))
                            .or_else(|| with_leaf(primary, bound.path, value))
                    }) else {
                        continue;
                    };
                    let Ok(facts) = flatten(ir, command, &candidate) else {
                        continue;
                    };
                    let alone = conjuncts.iter().enumerate().all(|(index, conjunct)| {
                        let decided = facts.decide(conjunct);
                        if index == bound.conjunct {
                            matches!(decided, Decision::Refuted(_))
                        } else {
                            matches!(decided, Decision::Satisfied)
                        }
                    });
                    if alone {
                        keep(candidate, &mut rows);
                    }
                }
                // Each conjunct of an `all` failing alone, whatever it compares (beyond10x/ess#155):
                // a default witnessed only where every conjunct fails is also the default of the
                // `any` a connective mutant writes. The ordered bounds above already cover the
                // conjuncts they reach at their neighbours; a conjunct already refuted alone by the
                // plain witness or an earlier row adds nothing.
                let owned: Vec<Predicate> = conjuncts.iter().map(|it| (*it).clone()).collect();
                one_per_child(ir, command, guard, &owned, false, primary, &mut rows, &keep);
                // A case-insensitive guard's literal in a case only Unicode folding equates with it
                // (beyond10x/ess#140): ASCII folding refutes it, so the default is sent it, and a target
                // that folds Unicode takes the guarded branch and fails.
                for (path, value) in crate::witness::unicode_refutations(guard) {
                    if let Some(candidate) = reference
                        .as_ref()
                        .and_then(|base| with_leaf(base, &path, value.clone()))
                        .or_else(|| with_leaf(primary, &path, value))
                    {
                        keep(candidate, &mut rows);
                    }
                }
            }
        }
        _ => {}
    }
    rows
}

/// One input region the declared precedence answers with `first` although `other` holds there too
/// (`docs/design/input-guard-overlap-precedence.md`).
///
/// Two kinds, one rule — the branch taken first is required wherever both guards hold:
///
/// | `first` | `other` | `refuted`: what must not hold, or it would answer instead |
/// |---|---|---|
/// | an input-guarded refusal (beyond10x/ess#178) | an accepting branch with an input guard: a plain `when:`, the `when:` beside a `when_subject:`, an external branch's `when:` | every other input-guarded refusal, which the model orders neither before nor after it |
/// | an accepting `when:` branch (beyond10x/ess#217) | an accepting `when:` branch declared after it | every input-guarded refusal, and every accepting `when:` branch declared before `first` |
pub(super) struct Overlap<'c> {
    /// The branch the precedence answers with.
    pub(super) first: &'c ResolvedOutcome,
    /// The branch it is taken before.
    pub(super) other: &'c ResolvedOutcome,
    /// `first`'s guard and `other`'s input guard.
    both: [&'c Predicate; 2],
    /// The guards that must not hold.
    refuted: Vec<&'c Predicate>,
}

impl<'c> Overlap<'c> {
    /// Every guard a candidate for this overlap is searched over.
    fn searched(&self) -> Vec<&'c Predicate> {
        let mut searched = self.both.to_vec();
        searched.extend(self.refuted.iter().copied());
        searched
    }

    /// Whether these input facts lie in the overlap: both guards hold and nothing taken before
    /// `first` does.
    fn holds(&self, facts: &crate::InputFacts<'_>) -> bool {
        decides(facts, &self.both, true).unwrap_or(false)
            && decides(facts, &self.refuted, false).unwrap_or(false)
    }
}

/// Every [`Overlap`] `outcome` is the `first` of, in declaration order of the other branch.
pub(super) fn overlaps<'c>(
    command: &'c ResolvedCommand,
    outcome: &'c ResolvedOutcome,
) -> Vec<Overlap<'c>> {
    let Some(own) = when(outcome) else {
        return Vec::new();
    };
    let refusals: Vec<&Predicate> = sibling_refusals(command, outcome)
        .filter_map(when)
        .collect();
    if is_input_guarded_refusal(outcome) {
        return command
            .outcomes
            .iter()
            .filter_map(|other| {
                accepting_input_half(other).map(|guard| Overlap {
                    first: outcome,
                    other,
                    both: [own, guard],
                    refuted: refusals.clone(),
                })
            })
            .collect();
    }
    if !is_input_guarded_accepting(outcome) {
        return Vec::new();
    }
    let mut refuted = refusals;
    refuted.extend(earlier_accepting(command, outcome));
    command
        .outcomes
        .iter()
        .skip_while(|other| other.name != outcome.name)
        .skip(1)
        .filter(|other| is_input_guarded_accepting(other))
        .filter_map(|other| {
            when(other).map(|guard| Overlap {
                first: outcome,
                other,
                both: [own, guard],
                refuted: refuted.clone(),
            })
        })
        .collect()
}

/// The first candidate in `overlap` that `also` accepts: over the ladders of every guard it reads
/// and `extra`, then between the literals of its `Decimal` leaves, so an overlap two literals less
/// than two apart bound (`11 < amount < 12`) is reached ([`first_candidate`]).
fn overlap_witness(
    ir: &EssIr,
    command: &ResolvedCommand,
    overlap: &Overlap<'_>,
    extra: &[&Predicate],
    also: impl Fn(&crate::InputFacts<'_>) -> bool,
) -> Option<BTreeMap<String, Node>> {
    let mut searched = overlap.searched();
    for guard in extra {
        if !searched.contains(guard) {
            searched.push(guard);
        }
    }
    first_candidate(ir, command, &searched, |facts| {
        overlap.holds(facts) && also(facts)
    })
}

/// The inputs a branch is further witnessed at because it is the `first` of an [`Overlap`]: one
/// per overlap, the first candidate in it, each sent once.
///
/// For an input-guarded refusal (beyond10x/ess#178): `closed: open == false` and `id-required:
/// ticket_id == ""` both hold of `{ticket_id: "", open: false}`, and the accepting branch cannot
/// read the identity to step aside (ESS-COMMAND-003), so sending that input and requiring the
/// refusal is what fails a target that reads `open` first. No row is arranged here: this is the
/// refusal sent as a plain invocation, which a refusal over the identity always is
/// ([`subject_fact::routes`]); a refusal sent for an arranged row gets its overlap rows from
/// `subject_fact::overlaps`.
///
/// For an accepting `when:` branch (beyond10x/ess#217): `small: amount < 100` before `flagged:
/// amount > 50` answers `amount: 75`, so `small` is sent an input in the overlap and required
/// there, which fails a target answering `flagged`.
///
/// An overlap no candidate reaches adds no row here; [`unwitnessed_overlaps`] records it as a
/// [`Note::UnwitnessedOverlap`] unless the candidates show it empty.
fn overlap_inputs(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Vec<BTreeMap<String, Node>> {
    let mut rows = Vec::new();
    for overlap in overlaps(command, outcome) {
        if let Some(input) = overlap_witness(ir, command, &overlap, &[], |_| true)
            .filter(|input| !rows.contains(input))
        {
            rows.push(input);
        }
    }
    rows
}

/// [`overlap_inputs`] for a command whose branches also read the held state, sent while the subject
/// rests in `held` (beyond10x/ess#217).
///
/// A branch guarded by the held state that admits `held` and holds of the input would compete with
/// both, so the input refutes every such branch — or, where one has no guard beyond the state, no
/// input reaches the overlap in `held`, and [`unwitnessed_overlaps`] records that.
fn overlap_inputs_in_state(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    held: &StateName,
) -> Vec<BTreeMap<String, Node>> {
    let guards: Vec<&Predicate> = command.outcomes.iter().filter_map(when).collect();
    let mut rows = Vec::new();
    for overlap in overlaps(command, outcome) {
        let competing = |facts: &crate::InputFacts<'_>| {
            command.outcomes.iter().any(|branch| {
                branch.name != overlap.first.name
                    && branch.name != overlap.other.name
                    && !state_default(branch)
                    && matches!(
                        branch.condition,
                        ResolvedCondition::SubjectState { .. }
                            | ResolvedCondition::StateChange { .. }
                    )
                    && admits_held_state(&branch.condition, held)
                    && when(branch)
                        .is_none_or(|guard| !matches!(facts.decide(guard), Decision::Refuted(_)))
            })
        };
        if let Some(input) =
            overlap_witness(ir, command, &overlap, &guards, |facts| !competing(facts))
                .filter(|input| !rows.contains(input))
        {
            rows.push(input);
        }
    }
    rows
}

/// Why an `Overlap` the suite does not send is recorded rather than refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlapGap {
    /// No candidate tried lies in the overlap, and the candidates do not cover every region the
    /// guards' literals divide the input into, so the overlap is not shown empty either.
    Unreached,
    /// A candidate lies in it, and the scenario of the branch taken first does not send one: the
    /// scenario is arranged by a search that sends no overlap input — a held state the branch
    /// cannot be sent the overlap in, a stored row, a replay or a preserved subject.
    Unsent,
}

/// Every scenario of `suite` with a step that requires a branch for an input an input-guarded
/// refusal answers first, one [`RefusalCause::PrecedenceContradicted`] each, naming the first such
/// step (beyond10x/ess#280).
///
/// Read off the finished suite, as `unwitnessed_overlaps` is, so every family that sends a command
/// is held to it whichever generator chose the input. Under the precedence order
/// (`docs/design/cross-record-and-stored-field-guards.md`) an input-guarded refusal is taken before
/// every accepting branch and every refusal declared after it (`sibling_refusals`); a step
/// requiring one of those for an input such a refusal's guard decidedly holds of fails every target
/// that honours the specification. #280's `set` scenario sent `{scope: Budget, provider:
/// "provider"}`, which `provider-not-allowed: {scope: Budget, provider: {exists: true}}` claims.
///
/// Only what the step sends literally is read. A guard reading a field sent as a reference to an
/// arranged row is not decided — the identity is opaque, and its absence from the literal facts
/// says nothing about its presence — and a guard the literals leave undecided claims nothing. On a
/// command guarded by a related row, `existing_instance:` and the `exists: false` branch answer
/// before any input refusal, so a step requiring either is not checked.
pub fn precedence_contradictions(ir: &EssIr, suite: &ConformanceSuite) -> Vec<Refusal> {
    suite
        .scenarios
        .iter()
        .filter_map(|(id, scenario)| {
            contradicted(ir, &scenario.steps).map(|cause| Refusal::about(id, cause))
        })
        .collect()
}

/// The first step of `steps` [`precedence_contradictions`] refuses, where one is.
fn contradicted(ir: &EssIr, steps: &[ScenarioStep]) -> Option<RefusalCause> {
    let mut steps = steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand {
            command: invoked,
            input,
            ..
        } = step
        else {
            continue;
        };
        let Some(ScenarioStep::ExpectOutcome { outcome: required }) = steps.peek() else {
            continue;
        };
        let Some(command) = ir
            .commands()
            .values()
            .find(|command| &command.name == invoked.name())
        else {
            continue;
        };
        let Some(outcome) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name == required.outcome)
        else {
            continue;
        };
        if related_guard::uses(command)
            && matches!(
                outcome.condition,
                ResolvedCondition::ExistingInstance
                    | ResolvedCondition::Related {
                        test: ess_compiler::ir::ResolvedRelatedTest::Absent,
                        ..
                    }
            )
        {
            continue;
        }
        let opaque: BTreeSet<&str> = input
            .iter()
            .filter(|(_, value)| value.as_literal().is_none())
            .map(|(field, _)| field.as_str())
            .collect();
        let Ok(facts) = crate::input::replay_facts(ir, command, input) else {
            continue;
        };
        // What the literals decide of `guard`; nothing where it reads an opaque field or anything
        // but the command's input — a caller attribute, a binder — which `defined()` would read
        // as absent.
        let decided = |guard: &Predicate| {
            let input_only = guard.fact_paths().iter().all(|path| {
                path.segments().first().is_some_and(|root| {
                    !opaque.contains(root.as_str())
                        && command.input.iter().any(|field| field.name == *root)
                })
            });
            input_only.then(|| facts.decide(guard))
        };
        let contradiction = |first: Option<&ResolvedOutcome>, guard: &Predicate| {
            RefusalCause::PrecedenceContradicted {
                required: outcome.name.clone(),
                first: first.map(|first| first.name.clone()),
                guard: guard.to_string(),
                input: sent_as_text(input),
            }
        };
        // Every branch answered before it: the input-guarded refusals, then the accepting `when:`
        // branches declared before an accepting `when:` or external one (beyond10x/ess#217).
        let before =
            sibling_refusals(command, outcome).chain(earlier_accepting_branches(command, outcome));
        for first in before {
            let Some(guard) = when(first) else {
                continue;
            };
            if matches!(decided(guard), Some(Decision::Satisfied)) {
                return Some(contradiction(Some(first), guard));
            }
        }
        // The branch's own `when:` over the input, where that alone selects it.
        if let ResolvedCondition::When { predicate } = &outcome.condition {
            if matches!(decided(predicate), Some(Decision::Refuted(_))) {
                return Some(contradiction(None, predicate));
            }
        }
    }
    None
}

/// `input` as it reads in a refusal: each field and its literal, or `<reference>` for a value
/// that names an arranged row.
fn sent_as_text(input: &BTreeMap<String, ScenarioValue>) -> String {
    let fields: Vec<String> = input
        .iter()
        .map(|(field, value)| match value.as_literal() {
            Some(literal) => format!("{field}: {literal}"),
            None => format!("{field}: <reference>"),
        })
        .collect();
    format!("{{{}}}", fields.join(", "))
}

/// A [`Note::UnwitnessedOverlap`] for every [`Overlap`] whose `first` has a scenario that sends no
/// input in it, unless the candidates show the overlap empty (beyond10x/ess#217).
///
/// This is the check, not a second generator: it reads the suite the passes above built, so a path
/// that forgets to send an overlap is recorded whichever path it is. An input a scenario sends is
/// read by its literal values; a value that names an arranged row stands in as the plain witness's
/// placeholder, which no input guard of these reads.
fn unwitnessed_overlaps(ir: &EssIr, suite: &ConformanceSuite) -> Vec<Note> {
    let mut notes = Vec::new();
    for command in ir.commands().values() {
        let placeholder = candidates(ir, command, &[], Distinction::PLAIN)
            .ok()
            .and_then(|inputs| inputs.into_iter().next())
            .unwrap_or_default();
        for outcome in &command.outcomes {
            let found = overlaps(command, outcome);
            if found.is_empty() {
                continue;
            }
            let id = ScenarioId::Outcome {
                outcome: OutcomeRef::new(
                    CommandRef::new(command.name.clone()),
                    outcome.name.clone(),
                ),
            };
            let Some(scenario) = suite.scenario(&id) else {
                // No scenario for the branch is a refusal already, which names it.
                continue;
            };
            let sent = sent_requiring(ir, command, outcome, &scenario.steps, &placeholder);
            for overlap in found {
                if sent
                    .iter()
                    .any(|input| flatten(ir, command, input).is_ok_and(|f| overlap.holds(&f)))
                {
                    continue;
                }
                let gap = if overlap_witness(ir, command, &overlap, &[], |_| true).is_some() {
                    OverlapGap::Unsent
                } else if crate::witness::exhausts(ir, command, &overlap.searched()) {
                    // Every region was tried and none lies in both: the overlap is empty.
                    continue;
                } else {
                    OverlapGap::Unreached
                };
                notes.push(Note::UnwitnessedOverlap {
                    scenario: id.clone(),
                    first: overlap.first.name.clone(),
                    other: overlap.other.name.clone(),
                    gap,
                });
            }
        }
    }
    notes
}

/// The inputs `steps` send `command` with and require `outcome` of, by literal value.
fn sent_requiring(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    steps: &[ScenarioStep],
    placeholder: &BTreeMap<String, Node>,
) -> Vec<BTreeMap<String, Node>> {
    let mut sent = Vec::new();
    let mut steps = steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand {
            command: invoked,
            input,
            ..
        } = step
        else {
            continue;
        };
        if invoked.name() != &command.name {
            continue;
        }
        let Some(ScenarioStep::ExpectOutcome { outcome: required }) = steps.peek() else {
            continue;
        };
        if required.outcome != outcome.name {
            continue;
        }
        let mut values = placeholder.clone();
        for (field, value) in input {
            if let ScenarioValue::Literal { value } = value {
                values.insert(field.clone(), value.clone());
            }
        }
        if flatten(ir, command, &values).is_ok() {
            sent.push(values);
        }
    }
    sent
}

/// The branch sent again at each of its [`boundary_inputs`], after everything else it asserts.
///
/// Each further invocation requires the branch and, where it declares one, its error — which is
/// what a moved boundary changes. A `moves:` branch acts on a fresh instance in the state the
/// scenario's own subject was arranged in, because that one has already moved; an `updates:`
/// branch acts on the scenario's subject again, which an update leaves where it was. Only for the
/// branches whose input alone selects them: a guard over the held state or the stored row is
/// arranged by searches of their own, and an externally decided or replayed branch has no boundary
/// the input moves.
#[allow(clippy::too_many_lines)] // Keep each boundary's arrangement beside its acting invocation.
fn boundaries(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    run: &Run,
    steps: &[ScenarioStep],
) -> (Vec<ScenarioStep>, BTreeSet<EssSemanticRef>) {
    let ir = models.arrangement;
    let mut out = (Vec::new(), BTreeSet::new());
    if outcome.replays.is_some()
        || subject_fact::routes(command, outcome)
        || outcome
            .subject
            .as_ref()
            .is_some_and(|subject| subject.effect == ResolvedEffect::Preserves)
    {
        return out;
    }
    // A command whose branches also read the held state gets no boundary rows — its search is
    // arranged per state — but a branch taken first in an overlap is sent the overlap in the state
    // its scenario arranged (beyond10x/ess#217).
    let held = if has_subject_guards(command) {
        match &run.before {
            Some(held) if !overlaps(command, outcome).is_empty() => Some(held.clone()),
            _ => return out,
        }
    } else {
        None
    };
    let plain = match &held {
        Some(held) => reach_in_state(ir, command, outcome, held, Distinction::PLAIN),
        None => reach(ir, command, outcome, Distinction::PLAIN),
    };
    let Ok(plain) = plain else {
        return out;
    };
    // The input the scenario actually sent, as values: a reference to an arranged row is a name
    // for an identity no guard can read, so the plain witness's placeholder stands in for it.
    let primary: BTreeMap<String, Node> = run
        .input
        .iter()
        .filter_map(|(field, value)| match value {
            ScenarioValue::Literal { value } => Some((field.clone(), value.clone())),
            _ => plain
                .get(field)
                .map(|placeholder| (field.clone(), placeholder.clone())),
        })
        .collect();
    let rows = match &held {
        Some(held) => overlap_inputs_in_state(ir, command, outcome, held)
            .into_iter()
            .filter(|row| row != &primary && admitted(ir, command, row))
            .collect(),
        None => boundary_inputs(ir, command, outcome, &primary),
    };
    if rows.is_empty() {
        return out;
    }
    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    let mut taken = bound_instances(steps);
    for row in rows {
        let mut supplied = run.input.clone();
        let mut settled = BTreeMap::new();
        for (field, value) in &row {
            if primary.get(field) != Some(value) {
                supplied.insert(field.clone(), ScenarioValue::literal(value.clone()));
            }
        }
        // A field the row leaves out is an absent optional the row was decided with, so the
        // primary's value is not carried over: `defined(provider)` refuted by the row would hold
        // again of what is sent (beyond10x/ess#280).
        for field in primary.keys().filter(|field| !row.contains_key(*field)) {
            supplied.remove(field);
        }
        if let Some(subject) = outcome
            .subject
            .as_ref()
            .filter(|subject| subject.effect.transition().is_some())
        {
            let (Some(before), ResolvedInstance::Supplied { field }) =
                (&run.before, &subject.instance)
            else {
                continue;
            };
            let Ok(mut arrangement) =
                arrange_unbound(ir, &subject.entity, before, actors, &mut taken)
            else {
                continue;
            };
            models.mark(caller::InvocationPhase::Arrange, &mut arrangement.steps);
            out.0.extend(arrangement.steps);
            out.1.extend(arrangement.source);
            settled = arrangement.settled;
            supplied.insert(
                field.name.clone(),
                ScenarioValue::instance(arrangement.instance),
            );
        }
        let expected = outcome
            .error
            .as_ref()
            .map(|error| expect_error(ir, outcome, error, &supplied, &settled));
        out.0.push(ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
            input: supplied,
        });
        out.0.push(ScenarioStep::ExpectOutcome {
            outcome: outcome_ref.clone(),
        });
        out.0.extend(expected);
    }
    out.1.insert(command_ref.into());
    out.1.insert(outcome_ref.into());
    models.mark(caller::InvocationPhase::Act, &mut out.0);
    out
}

/// A purpose cut to one line, which is what [`ScenarioPurpose`] accepts.
fn clipped(text: &str) -> ScenarioPurpose {
    let clipped: String = text.chars().take(ScenarioPurpose::MAX_LENGTH).collect();
    ScenarioPurpose::new(clipped)
        .unwrap_or_else(|error| panic!("a synthesised purpose is one line: {error}"))
}

/// What must still hold of an entity once a branch has changed one (§20).
///
/// One scenario per entity per state-changing branch: run the branch, then require that a view of
/// the entity satisfies every invariant it declares. Not one per invariant — see
/// [`ScenarioId::Invariant`] for why an invariant has no name to be keyed by, and what carrying them
/// together costs.
///
/// A branch with no subject changes no entity, and an entity that declares no invariant has nothing
/// to check; neither is a refusal, because neither is a check the specification asked for and did
/// not get.
fn invariants(
    models: &caller::InvocationModels<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    focus: Focus<'_>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let ir = models.arrangement;
    let projections = row_projections(ir);
    for command in models.acting.commands().values() {
        if !focus.takes(&command.name) {
            continue;
        }
        for outcome in &command.outcomes {
            let Some(subject) = &outcome.subject else {
                continue;
            };
            if ir.entity(&subject.entity).invariants.is_empty() {
                continue;
            }
            let id = ScenarioId::Invariant {
                entity: EntityRef::from(&subject.entity),
                after: OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone()),
            };
            let Some(scenario) = holds_after(
                models,
                command,
                outcome,
                &projections,
                actors,
                &id,
                refusals,
            ) else {
                continue;
            };
            insert(suite, id, scenario, refusals);
        }
    }
    if focus.is_whole() {
        value_object_invariants(ir, actors, suite, refusals);
    }
}

/// The scenario that runs one branch and then reads the entity's invariants off a view.
///
/// `None` where the branch cannot be run at all, and where **every** invariant was refused: a
/// scenario that executes a command and asserts nothing about what it was written to check is the
/// shape of green this milestone exists to rule out. A scenario that could assert some of them is
/// still worth having, and the ones it could not appear as refusals beside it.
#[allow(clippy::too_many_arguments)]
fn holds_after(
    models: &caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    projections: &BTreeMap<&EntityHandle, Vec<&ResolvedView>>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> Option<ConformanceScenario> {
    let ir = models.arrangement;
    let subject = outcome.subject.as_ref()?;
    let entity = ir.entity(&subject.entity);
    let entity_ref = EntityRef::from(&subject.entity);
    let run = match run_as(models, command, outcome, actors, Witness::Full) {
        Ok(run) => run,
        Err(cause) => {
            refusals.push(Refusal::about(id, cause));
            return None;
        }
    };
    // The state the branch leaves the instance in, which is what decides whether a filtered view
    // holds a row to read the invariant off. A branch with a subject always has one.
    let state = run.after.clone()?;
    let known = identity_of(subject, run.instance.as_ref());
    let views = projections
        .get(&subject.entity)
        .map(Vec::as_slice)
        .unwrap_or_default();

    let mut steps = run.steps();
    let mut named: BTreeSet<ViewRef> = BTreeSet::new();
    for invariant in &entity.invariants {
        let witnesses = witnesses_for(ir, invariant, views, &state, &run.settled, known.as_ref());
        if witnesses.is_empty() {
            let (unpublished, unassertable) =
                unobserved(ir, invariant, &entity.observable_fields(), views);
            refusals.push(Refusal::about(
                id,
                RefusalCause::InvariantUnobservable {
                    entity: entity_ref.clone(),
                    invariant: invariant.statement.clone(),
                    unpublished,
                    unassertable,
                    state: state.clone(),
                },
            ));
            continue;
        }
        for view in witnesses {
            steps.extend(assert_satisfied(
                view,
                bound(ir, view, &run.settled, known.as_ref()),
                invariant.predicate.clone(),
            ));
            named.insert(ViewRef::new(view.name.clone()));
        }
    }
    if named.is_empty() {
        return None;
    }

    let mut source: BTreeSet<EssSemanticRef> = run.source.clone();
    let command_ref = CommandRef::new(command.name.clone());
    source.insert(command_ref.clone().into());
    source.insert(OutcomeRef::new(command_ref, outcome.name.clone()).into());
    source.insert(entity_ref.clone().into());
    if let Some(actor) = run.actor.clone() {
        source.insert(actor.into());
    }
    source.extend(named.into_iter().map(EssSemanticRef::from));
    source.extend(
        input_types(ir, command)
            .into_iter()
            .map(EssSemanticRef::from),
    );

    let text = format!(
        "a `{entity_ref}` still satisfies what it declares after `{}` on `{}`",
        command.name, outcome.name
    );
    Some(ConformanceScenario::new(clipped(&text), steps, source))
}

/// Requiring one condition of every row of one view, in the block that view's consistency decides.
///
/// The style is read off [`ResolvedView::assertion_style`], exactly as §14 requires everywhere else:
/// an invariant asserted immediately against an `eventual` projection races it, and the repair
/// everyone reaches for is a sleep. Takes the predicate rather than an [`Invariant`] because two
/// families arrive here — an entity's invariant as written, and a value object's invariant rebased
/// onto the field position that holds one — and the view does not care which.
fn assert_satisfied(
    view: &ResolvedView,
    params: BTreeMap<String, ScenarioValue>,
    predicate: Predicate,
) -> Vec<ScenarioStep> {
    let name = ViewRef::new(view.name.clone());
    let expectation = ViewExpectation::Satisfies { predicate };
    match view.assertion_style {
        AssertionStyle::Expect => vec![
            ScenarioStep::QueryView {
                view: name.clone(),
                params,
            },
            ScenarioStep::ExpectView {
                view: name,
                expectation,
            },
        ],
        AssertionStyle::Eventually => vec![ScenarioStep::EventuallyView {
            view: name,
            params,
            expectation,
        }],
    }
}

/// §20 read on a value object: what a type declares of every value must hold at every field
/// position observable state holds one.
///
/// A value object's invariant is a claim about a *type* — every `Money` in the system — rather
/// than about one instance at rest, so it is not keyed by an entity and an outcome the way the
/// family above is. The observable surface a runner can hold it to is a **view field position**:
/// `amount >= 0` on `Money`, read at `InvoiceById.total`, becomes `total.amount >= 0` over every
/// row — the rebasing this function performs, and the one place a value object's own words are
/// rewritten, so [`rebased`] carries the argument for why the rewrite is a reading and not an
/// inference.
///
/// One scenario per (type, view, position), because two positions are two checks: a projection can
/// corrupt `OutstandingInvoices.total` while `InvoiceById.total` stays right, and one id for both
/// would report the first as the second. Which command arranges a row is content, not identity —
/// the first declared outcome in name order that can put an instance where the view shows it.
///
/// A type nothing observable holds keeps a refusal, as it always had one — the cause just stopped
/// being "not synthesised yet" and became the honest one: no view publishes a position that can
/// answer it. A position that exists but that no declared outcome can put a row into is refused
/// per position, under the scenario id that is missing.
fn value_object_invariants(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for declared in ir.types().values() {
        let invariants = match &declared.body {
            ResolvedBody::Newtype { invariants, .. } | ResolvedBody::Struct { invariants, .. } => {
                invariants
            }
            ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => continue,
        };
        if invariants.is_empty() {
            continue;
        }
        let positions = positions_of(ir, declared, invariants);
        if positions.is_empty() {
            refusals.push(Refusal {
                subject: DeclaredTypeRef::new(declared.name.clone()).into(),
                scenario: None,
                cause: RefusalCause::ValueInvariantUnwitnessed {
                    value: DeclaredTypeRef::new(declared.name.clone()),
                    invariants: invariants
                        .iter()
                        .map(|invariant| invariant.statement.clone())
                        .collect(),
                    at: None,
                },
            });
            continue;
        }
        for (view, field) in positions {
            let id = ScenarioId::ValueInvariant {
                value: DeclaredTypeRef::new(declared.name.clone()),
                at: ViewRef::new(view.name.clone()),
                field: field.clone(),
            };
            match holds_at(ir, declared, invariants, view, &field, actors) {
                Ok(scenario) => insert(suite, id, scenario, refusals),
                Err(cause) => refusals.push(Refusal::about(&id, cause)),
            }
        }
    }
}

/// Every view field position that can answer this type's invariants: reached through newtypes and
/// structs, and with every rebased path landing on a scalar.
///
/// What is deliberately *not* a position, and why each absence is a reading of the model rather
/// than a shortcut:
///
/// * inside a `List` or a `Map` — a fact path has no index or key selector, the rule
///   [`resolve_path`] already applies to every other surface;
/// * inside a `Union` — which variant a row holds is the row's business, so no single path is one
///   every row must answer;
/// * under an `Optional` — a row may hold nothing there, the predicate would evaluate `Unknown`,
///   and `Unknown` stops a runner rather than passing (invariant 5);
/// * past [`MAX_TYPE_DEPTH`] — the bound every type walk in this workspace shares.
fn positions_of<'a>(
    ir: &'a EssIr,
    declared: &ResolvedType,
    invariants: &[Invariant],
) -> Vec<(&'a ResolvedView, String)> {
    let mut positions = Vec::new();
    for view in ir.views().values().filter(|view| !view.is_aggregate()) {
        for field in &view.fields {
            let mut found = Vec::new();
            reaches(
                ir,
                &field.type_ref,
                &declared.name,
                vec![field.name.clone()],
                0,
                &mut found,
            );
            for prefix in found {
                let answerable = invariants.iter().all(|invariant| {
                    predicate_projectable(
                        ir,
                        &view.fields,
                        &rebased(&invariant.predicate, &prefix, &declared.body),
                    )
                });
                if answerable {
                    positions.push((view, prefix));
                }
            }
        }
    }
    positions
}

/// Collects every dotted prefix under `at` that reaches the type named `wanted`.
fn reaches(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    wanted: &QualifiedName,
    at: Vec<String>,
    depth: usize,
    found: &mut Vec<String>,
) {
    if depth > MAX_TYPE_DEPTH {
        return;
    }
    match type_ref {
        ResolvedTypeRef::Declared { name } if name.name() == wanted => found.push(at.join(".")),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            // A newtype is transparent, exactly as every other walk reads one.
            ResolvedBody::Newtype { of, .. } => reaches(ir, of, wanted, at, depth + 1, found),
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    let mut deeper = at.clone();
                    deeper.push(field.name.clone());
                    reaches(ir, &field.type_ref, wanted, deeper, depth + 1, found);
                }
            }
            ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => {}
        },
        ResolvedTypeRef::Optional { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. }
        | ResolvedTypeRef::Primitive { .. } => {}
    }
}

/// The invariant's predicate, re-rooted at the field position that holds the value.
///
/// A reading, not an inference: the type's own declaration says what every value satisfies, the
/// view's declaration says a value of the type sits at `prefix`, and composing the two is exactly
/// what "its invariants hold wherever one is" means. A struct's invariant reads its own fields, so
/// each path gains the prefix; a newtype's reads [`value`](ess_domain::types::NamedType::VALUE),
/// the pseudo-field naming the representation it wraps, and at a position the position *is* the
/// value — so `value` maps to the prefix itself.
fn rebased(predicate: &Predicate, prefix: &str, body: &ResolvedBody) -> Predicate {
    let strip_value = matches!(body, ResolvedBody::Newtype { .. });
    let onto = |path: &FactPath| {
        let mut segments: Vec<String> = prefix.split('.').map(str::to_owned).collect();
        let tail = if strip_value {
            &path.segments()[1..]
        } else {
            path.segments()
        };
        segments.extend(tail.iter().cloned());
        FactPath::from_segments(segments)
    };
    map_paths(predicate, &onto)
}

/// One predicate with every *free* fact path rewritten, and nothing else touched.
///
/// `dyn` rather than a generic, because a quantifier has to descend with a different rewrite from
/// the one it was handed — one that leaves its binder alone — and two closures written at different
/// places are two types.
fn map_paths(predicate: &Predicate, onto: &dyn Fn(&FactPath) -> FactPath) -> Predicate {
    let operand = |it: &Operand| match it {
        Operand::Fact(path) => Operand::Fact(onto(path)),
        Operand::Literal(value) => Operand::Literal(value.clone()),
    };
    match predicate {
        Predicate::Always => Predicate::Always,
        Predicate::Never => Predicate::Never,
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .map(|child| map_paths(child, onto))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .iter()
                .map(|child| map_paths(child, onto))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(map_paths(inner, onto))),
        Predicate::Compare { left, op, right } => Predicate::Compare {
            left: operand(left),
            op: *op,
            right: operand(right),
        },
        Predicate::Truthy(path) => Predicate::Truthy(onto(path)),
        Predicate::Defined(path) => Predicate::Defined(onto(path)),
        Predicate::AnyOf { path, values } => Predicate::AnyOf {
            path: onto(path),
            values: values.clone(),
        },
        Predicate::NoneOf { path, values } => Predicate::NoneOf {
            path: onto(path),
            values: values.clone(),
        },
        Predicate::TextMatch { path, op, value } => Predicate::TextMatch {
            path: onto(path),
            op: *op,
            value: value.clone(),
        },
        Predicate::FoldMatch { path, op, values } => Predicate::FoldMatch {
            path: onto(path),
            op: *op,
            values: values.clone(),
        },
        // The collection is a model path and moves with the rest. The binder is not: re-rooting
        // `slot.left` at a field position would produce a path naming a field that does not exist,
        // and it would do it silently, which is the failure this whole module refuses elsewhere.
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let bind = quantified.bind.clone();
            let inner = move |path: &FactPath| {
                if path.namespace() == bind {
                    path.clone()
                } else {
                    onto(path)
                }
            };
            let mapped = Box::new(Quantified {
                over: onto(&quantified.over),
                bind: quantified.bind.clone(),
                body: map_paths(&quantified.body, &inner),
            });
            if matches!(predicate, Predicate::Forall(_)) {
                Predicate::Forall(mapped)
            } else {
                Predicate::Exists(mapped)
            }
        }
    }
}

/// The scenario that puts a row where the view shows it and reads the type's invariants off the
/// position, or why none can be arranged.
///
/// The arranging branch is the first declared outcome, in (command, outcome) order, whose subject
/// is the view's source entity, whose arrangement succeeds, and whose after-state the view shows —
/// the same "first that the specification can actually reach" reading [`arrange_first`] takes, so
/// an outcome that cannot be arranged is skipped rather than fatal. Which branch it is is content,
/// not identity: see [`ScenarioId::ValueInvariant`].
fn holds_at(
    ir: &EssIr,
    declared: &ResolvedType,
    invariants: &[Invariant],
    view: &ResolvedView,
    field: &str,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            let Some(subject) = &outcome.subject else {
                continue;
            };
            if subject.entity != view.source {
                continue;
            }
            let Ok(run) = run(ir, command, outcome, actors) else {
                // That branch cannot be arranged, and carries its own refusal under its own id;
                // the next declared branch may still put a row where this check needs one.
                continue;
            };
            let Some(state) = run.after.clone() else {
                continue;
            };
            let known = identity_of(subject, run.instance.as_ref());
            let params = bound(ir, view, &run.settled, known.as_ref());
            if !matches!(
                shows_row(ir, view, &state, &run.settled, known.as_ref(), &params),
                Ok(true)
            ) {
                continue;
            }

            let mut steps = run.steps();
            for invariant in invariants {
                steps.extend(assert_satisfied(
                    view,
                    params.clone(),
                    rebased(&invariant.predicate, field, &declared.body),
                ));
            }

            let mut source: BTreeSet<EssSemanticRef> = run.source.clone();
            let command_ref = CommandRef::new(command.name.clone());
            source.insert(command_ref.clone().into());
            source.insert(OutcomeRef::new(command_ref, outcome.name.clone()).into());
            source.insert(EntityRef::from(&view.source).into());
            source.insert(ViewRef::new(view.name.clone()).into());
            if let Some(actor) = run.actor.clone() {
                source.insert(actor.into());
            }
            source.extend(
                input_types(ir, command)
                    .into_iter()
                    .map(EssSemanticRef::from),
            );
            // The value's own type closure: the subject itself, then what its body reaches —
            // spelled out because a handle is mintable only by the compiler, and the subject is
            // already a name.
            let mut of_value: BTreeSet<DeclaredTypeRef> = BTreeSet::new();
            of_value.insert(DeclaredTypeRef::new(declared.name.clone()));
            match &declared.body {
                ResolvedBody::Newtype { of, .. } => reachable_types(ir, of, &mut of_value),
                ResolvedBody::Struct { fields, .. } => {
                    for member in fields {
                        reachable_types(ir, &member.type_ref, &mut of_value);
                    }
                }
                ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => {}
            }
            source.extend(of_value.into_iter().map(EssSemanticRef::from));

            let text = format!(
                "every `{}` that `{}` publishes at `{field}` satisfies what the type declares",
                declared.name, view.name
            );
            return Ok(ConformanceScenario::new(clipped(&text), steps, source));
        }
    }
    Err(RefusalCause::ValueInvariantUnwitnessed {
        value: DeclaredTypeRef::new(declared.name.clone()),
        invariants: invariants
            .iter()
            .map(|invariant| invariant.statement.clone())
            .collect(),
        at: Some((ViewRef::new(view.name.clone()), field.to_owned())),
    })
}

/// Every view that can answer this invariant about an instance resting in `state`.
///
/// Two conditions, and both are readings of the model rather than guesses about a runner. The view
/// has to **publish what the invariant reads** — every path resolved against the view's own declared
/// fields, by the same walk a guard's path takes through a command's input — and its filter has to
/// **hold an instance in this state**, because an assertion about every row of an empty view is an
/// assertion nothing can fail.
///
/// A view whose filter cannot be decided against the one fact a scenario knows is not a witness
/// either. That view already has its own refusal under the outcome scenario, saying so with the
/// paths; repeating it here would be one defect reported twice.
fn witnesses_for<'a>(
    ir: &EssIr,
    invariant: &Invariant,
    views: &[&'a ResolvedView],
    state: &StateName,
    settled: &BTreeMap<String, Determined>,
    identity: Option<&ScenarioValue>,
) -> Vec<&'a ResolvedView> {
    views
        .iter()
        .filter(|view| {
            predicate_projectable(ir, &view.fields, &invariant.predicate)
                && matches!(
                    shows_row(
                        ir,
                        view,
                        state,
                        settled,
                        identity,
                        &bound(ir, view, settled, identity)
                    ),
                    Ok(true)
                )
        })
        .copied()
        .collect()
}

/// Every path an invariant reads that no view of the entity resolves to a scalar, split in two:
/// those no view declares at all, and those some view declares as a collection or a text length —
/// a `.count` no view-row assertion in this suite format carries.
///
/// The difference between "no view holds a row here" and "no view could ever answer this", which is
/// the difference between a filter an author might widen and a field an author has to publish —
/// and, for the second list, neither: the field is published and the suite format is the limit
/// (`docs/design/string-alphabet-and-length.md`, section 3).
fn unobserved(
    ir: &EssIr,
    invariant: &Invariant,
    fields: &[ess_compiler::ir::ResolvedField],
    views: &[&ResolvedView],
) -> (Vec<FactPath>, Vec<FactPath>) {
    let reads: BTreeSet<FactPath> = ess_compiler::expression::check_predicate(
        ir,
        fields,
        &invariant.predicate,
        "entity invariant",
    )
    .reads
    .into_iter()
    .filter(|read| read.free)
    .map(|read| read.path)
    .filter(|path| {
        !views
            .iter()
            .any(|view| resolve_path(ir, &view.fields, path).is_scalar())
    })
    .collect();
    reads.into_iter().partition(|path| {
        !views.iter().any(|view| {
            matches!(
                resolve_path(ir, &view.fields, path),
                crate::input::Target::Aggregate("a collection" | "a text length")
            )
        })
    })
}

/// Every declared type a command's input reaches.
fn input_types(ir: &EssIr, command: &ResolvedCommand) -> BTreeSet<DeclaredTypeRef> {
    let mut types = BTreeSet::new();
    for field in command.input.iter().chain(&command.response) {
        reachable_types(ir, &field.type_ref, &mut types);
    }
    types
}

/// The four claims a binding makes, one scenario each (§16, §17, §18).
///
/// Every one of them starts the same way — make the event happen — and that alone is more than a
/// preamble: the trigger is a whole outcome scenario's worth of arrangement, because the event a
/// binding reacts to is published by a command that may itself need an instance driven into a state.
///
/// Where the trigger cannot be produced at all, the refusal is about the **binding** rather than
/// about one of its four scenarios: none of the four exists, and four copies of one reason is a
/// diagnostic a reader has to deduplicate by hand.
fn bindings(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for binding in ir.bindings().values() {
        if binding.cause.periodic().is_some() {
            crate::periodic::synthesize(ir, binding, suite, refusals);
            continue;
        }
        if binding.context.is_some() {
            delivery_context::synthesize(ir, binding, suite, refusals);
            continue;
        }
        // A conditioned binding (ess/22, beyond10x/ess#268) runs only for a payload its condition
        // holds for, so every publishing branch is a candidate trigger.
        if let Some(condition) = &binding.condition {
            conditioned(ir, binding, condition, actors, suite, refusals);
            continue;
        }
        let subject = BindingRef::new(binding.name.clone());
        let event_handle = binding.cause.event().expect("event cause");
        let Some((publisher, published_by)) = publisher(ir, event_handle) else {
            refusals.push(Refusal {
                subject: subject.clone().into(),
                scenario: None,
                cause: RefusalCause::BindingUnobservable {
                    binding: subject,
                    gap: BindingGap::NothingPublishes {
                        event: EventRef::from(event_handle),
                    },
                },
            });
            continue;
        };
        match run(ir, publisher, published_by, actors) {
            Ok(run) => {
                let trigger = binding_condition::Trigger {
                    publisher,
                    published_by,
                    run,
                };
                binding_aspects(ir, binding, &trigger, actors, suite, refusals);
            }
            Err(cause) => refusals.push(Refusal {
                subject: subject.into(),
                scenario: None,
                cause,
            }),
        }
    }
}

/// Every aspect of a binding whose event `trigger` publishes: flow, mapping, delivery, on-failure,
/// and a bounded retry's final failure.
fn binding_aspects(
    ir: &EssIr,
    binding: &ResolvedBinding,
    trigger: &binding_condition::Trigger<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let subject = BindingRef::new(binding.name.clone());
    let event = EventRef::from(binding.cause.event().expect("event cause"));
    let binding_condition::Trigger {
        publisher,
        published_by,
        run: trigger,
    } = trigger;
    let invoked = ir.command(&binding.command);
    let source = binding_source(ir, binding, publisher, published_by, trigger);
    // The row the invoked command addresses, arranged where an accepting branch admits it before
    // the trigger (beyond10x/ess#267), or the gap that stops every aspect needing it.
    let prepared = binding_effects::prepare(ir, binding, invoked, published_by, trigger, actors);
    for aspect in BindingAspect::ALL.map(|(aspect, _)| aspect) {
        let id = ScenarioId::Binding {
            binding: subject.clone(),
            aspect,
        };
        let built = match aspect {
            BindingAspect::Flow => flow(ir, invoked, &prepared, &event),
            BindingAspect::Mapping => mapping(ir, binding, invoked, trigger, &prepared, &event),
            BindingAspect::Delivery => delivery(ir, binding, invoked, &prepared, &event),
            BindingAspect::OnFailure if binding.retry.is_some() => {
                bounded_retry::exhausted(ir, binding, invoked, trigger, &event, actors)
            }
            BindingAspect::OnFailure => on_failure(ir, binding, invoked, &prepared, &event),
            // Not in `ALL`; produced below for the bindings that make the claim.
            BindingAspect::FinalFailure
            | BindingAspect::ConditionFalse
            | BindingAspect::ConditionAbsent => continue,
        };
        let (steps, purpose, extra) = match built {
            Ok(built) => built,
            Err(gap) => {
                refusals.push(Refusal::about(
                    &id,
                    RefusalCause::BindingUnobservable {
                        binding: subject.clone(),
                        gap,
                    },
                ));
                continue;
            }
        };
        // `drop` and a bounded retry build their own failure scenario, without the row's rest.
        let dropping =
            matches!(binding.on_failure(), ResolvedFailure::Drop) || binding.retry.is_some();
        binding_effects::name_unsettled(&prepared, aspect, dropping, &id, refusals);
        let mut depends = source.clone();
        depends.extend(extra);
        insert(
            suite,
            id,
            ConformanceScenario::new(purpose, steps, depends),
            refusals,
        );
    }
    bounded_retry::final_failure(
        ir, binding, invoked, trigger, &event, actors, &source, suite, refusals,
    );
}

/// A conditioned binding (ess/22, beyond10x/ess#268, beyond10x/ess#194): its positive aspects from
/// a trigger its condition holds for, refused aspect by aspect where no publishing branch gives
/// one, and its `condition-false` and `condition-absent` witnesses.
fn conditioned(
    ir: &EssIr,
    binding: &ResolvedBinding,
    condition: &ess_compiler::ir::ResolvedBindingCondition,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let subject = BindingRef::new(binding.name.clone());
    let witnesses = match binding_condition::witnesses(ir, binding, condition, actors) {
        Ok(witnesses) => witnesses,
        Err(cause) => {
            refusals.push(Refusal {
                subject: subject.into(),
                scenario: None,
                cause,
            });
            return;
        }
    };
    match &witnesses.holds {
        Ok(trigger) => binding_aspects(ir, binding, trigger, actors, suite, refusals),
        Err(gap) => {
            let finals = binding
                .retry
                .as_ref()
                .is_some_and(|bound| !bound.final_outcomes.is_empty());
            let aspects = BindingAspect::ALL
                .map(|(aspect, _)| aspect)
                .into_iter()
                .chain(finals.then_some(BindingAspect::FinalFailure));
            for aspect in aspects {
                let id = ScenarioId::Binding {
                    binding: subject.clone(),
                    aspect,
                };
                refusals.push(Refusal::about(
                    &id,
                    RefusalCause::BindingUnobservable {
                        binding: subject.clone(),
                        gap: gap.clone(),
                    },
                ));
            }
        }
    }
    let absent = witnesses.absent;
    for (aspect, witness) in [
        (
            BindingAspect::ConditionFalse,
            Some(witnesses.fails.map(|t| vec![t])),
        ),
        (BindingAspect::ConditionAbsent, absent),
    ] {
        let Some(witness) = witness else {
            continue;
        };
        let id = ScenarioId::Binding {
            binding: subject.clone(),
            aspect,
        };
        match witness.and_then(|triggers| never_invoked(ir, binding, &triggers)) {
            Ok((steps, purpose, depends)) => {
                let scenario = ConformanceScenario::new(purpose, steps, depends);
                insert(suite, id, scenario, refusals);
            }
            Err(gap) => refusals.push(Refusal::about(
                &id,
                RefusalCause::BindingUnobservable {
                    binding: subject.clone(),
                    gap,
                },
            )),
        }
    }
}

/// A conditioned binding's negative witness (ess/22, beyond10x/ess#268, beyond10x/ess#194): each
/// trigger publishes a payload the condition does not hold for and the occurrence is observed, and
/// the binding invokes its command no times for the whole eventual window. Not inferred from an
/// absent downstream event: the binding may invoke a command that publishes nothing.
///
/// Several triggers — one per proved Optional level — run in order in one scenario: the first with
/// its arrangement, each later one invoked again on that arrangement, which is refused where its
/// invocation binds an instance a second time.
fn never_invoked(
    ir: &EssIr,
    binding: &ResolvedBinding,
    triggers: &[binding_condition::Trigger<'_>],
) -> Built {
    let invoked = ir.command(&binding.command);
    let event = EventRef::from(binding.cause.event().expect("event cause"));
    let command = CommandRef::new(invoked.name.clone());
    let mut steps = Vec::new();
    let mut source: BTreeSet<EssSemanticRef> = [command.clone().into()].into_iter().collect();
    for (index, trigger) in triggers.iter().enumerate() {
        if index == 0 {
            steps.extend(trigger.run.steps());
        } else if trigger
            .run
            .invoke
            .iter()
            .any(|step| matches!(step, ScenarioStep::CaptureInstance { .. }))
        {
            return Err(BindingGap::ConditionUnarranged {
                why: "its proved levels need one occurrence each, and invoking its trigger again \
                      binds an instance a second time"
                    .into(),
            });
        } else {
            steps.extend(trigger.run.invoke.iter().cloned());
        }
        steps.push(ScenarioStep::ExpectEvent {
            event: event.clone(),
            payload: BTreeMap::new(),
            shape: payload_shape(ir, &event),
        });
        source.extend(binding_source(
            ir,
            binding,
            trigger.publisher,
            trigger.published_by,
            &trigger.run,
        ));
    }
    steps.push(ScenarioStep::ExpectNoInvocation {
        binding: BindingRef::new(binding.name.clone()),
        command,
    });
    let text = format!(
        "`{}` invokes `{}` no times for a `{event}` its condition does not hold for",
        binding.name, invoked.name
    );
    Ok((steps, clipped(&text), source))
}

/// What one binding aspect produces: its steps, its one-line purpose, and what else it depends on.
type Built = Result<(Vec<ScenarioStep>, ScenarioPurpose, BTreeSet<EssSemanticRef>), BindingGap>;

/// §16: the event happens, and the invoked command publishes what its branch declares.
///
/// Proved through the downstream **event**, never by observing the invocation — that is §16's rule,
/// and it is why a target with no command tracing can still be held to a binding's flow.
///
/// The observation is bounded rather than immediate ([`ScenarioStep::EventuallyEvent`]) because a
/// binding crosses a component boundary: nothing in the model says the consequence has happened by
/// the time the triggering command returns, and requiring that would be a transport assumption §41
/// refuses.
///
/// # What a flow scenario cannot tell apart, and what covers it
///
/// Where two bindings invoke one command, the event this waits for is one either of them could have
/// produced — and a binding whose trigger needs an arrangement may set the other one off while being
/// arranged. So a dropped binding is provable here only when its consequence is its own.
/// [`BindingAspect::Mapping`] is what closes that: [`ScenarioStep::ExpectInvocation`] names the
/// binding, which no event does.
///
/// # The row the invocation acts on
///
/// Where the invoked command acts on an existing row, that row is arranged — or made by the trigger
/// itself — in a state an accepting branch admits, before the trigger, and the scenario ends by
/// observing it eventually where the binding leaves it ([`binding_effects::prepare`],
/// beyond10x/ess#267). A `wrong_state:` branch beside the accepting one is what an ineligible row
/// reaches, not a second outcome a scenario must choose between.
fn flow(
    ir: &EssIr,
    invoked: &ResolvedCommand,
    prepared: &Result<binding_effects::Prepared<'_>, BindingGap>,
    event: &EventRef,
) -> Built {
    let prepared = prepared.as_ref().map_err(Clone::clone)?;
    let reached = prepared.reached.clone()?;
    let published = publishes(invoked, reached)?;

    let mut steps = prepared.steps();
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });
    for event in &published {
        steps.push(ScenarioStep::EventuallyEvent {
            event: event.clone(),
            payload: BTreeMap::new(),
            shape: payload_shape(ir, event),
        });
    }
    steps.extend(prepared.settled_row(ir));

    let text = format!(
        "`{event}` invokes `{}`, which publishes {}",
        invoked.name,
        listed(&published)
    );
    let mut source = downstream(ir, invoked, reached);
    source.extend(prepared.source.iter().cloned());
    Ok((steps, clipped(&text), source))
}

/// §16: each input receives the value the binding's mapping names for it.
///
/// The one clause a document can get *silently* wrong, and the reason
/// [`ScenarioStep::ExpectInvocation`] exists — a mapping's target is a command input, and the only
/// observation *attributed to the binding* is the invocation itself: a downstream event field,
/// even one the invoked outcome's `payload:` determines, says what some command was given and
/// never which binding filled it. Every value here is read straight off
/// the resolved mapping: a field of the triggering event becomes
/// [`ScenarioValue::Observed`], because no generator knows what the upstream implementation
/// published there, and a literal becomes the text the binding wrote.
///
/// Sent through the arranged trigger where the invoked command addresses a row
/// ([`binding_effects::prepare`]), so the invocation lands on a row that exists; through the plain
/// trigger where that row cannot be arranged, since what the invocation carries is a reading of the
/// document whatever it then does.
fn mapping(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    trigger: &Run,
    prepared: &Result<binding_effects::Prepared<'_>, BindingGap>,
    event: &EventRef,
) -> Built {
    let command = CommandRef::new(invoked.name.clone());
    if binding.mapping.is_empty() {
        return Err(BindingGap::NothingMapped { command });
    }
    let input = binding_effects::mapped_input(ir, binding, event)?;

    let mut steps = prepared
        .as_ref()
        .map_or_else(|_| trigger.steps(), binding_effects::Prepared::steps);
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });
    steps.push(ScenarioStep::ExpectInvocation {
        binding: BindingRef::new(binding.name.clone()),
        command: command.clone(),
        input,
        count: None,
    });

    let text = format!(
        "`{}` fills `{}` from `{event}` as it is mapped",
        binding.name, invoked.name
    );
    // The invoked command's input types, because that is what the mapping fills: widen
    // `Recipient` and this scenario's stored result is stale even though nothing it names moved.
    let mut source: BTreeSet<EssSemanticRef> = [command.into()].into_iter().collect();
    source.extend(
        input_types(ir, invoked)
            .into_iter()
            .map(EssSemanticRef::from),
    );
    if let Ok(prepared) = prepared {
        source.extend(prepared.source.iter().cloned());
    }
    Ok((steps, clipped(&text), source))
}

/// §17: the same event delivered twice still leaves the declared consequence observable.
///
/// What `delivery: at_least_once` actually says, and the only thing it says. A conformant
/// implementation **may** produce duplicates, so the assertion is deliberately not a count: §17
/// writes "exactly one `EmailSent` exists" out as the bad test, because it fails a target that is
/// doing exactly what the specification permits.
///
/// What is left is survivability, and it is worth a scenario of its own: an implementation that
/// treats a redelivery as an error, or stops delivering after one, breaks here and nowhere else.
///
/// `at_most_once` has no such scenario and gets [`BindingGap::DeliverySingleAttempt`] instead. The
/// refusal is taken **first**, before the branch and the published events are read: a single-attempt
/// binding is refused for the guarantee it declares, not for a shape its redelivery scenario would
/// have needed and never uses.
fn delivery(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    prepared: &Result<binding_effects::Prepared<'_>, BindingGap>,
    event: &EventRef,
) -> Built {
    // Read before anything else, for the reason `on_failure` reads its policy first.
    if matches!(binding.delivery, Delivery::AtMostOnce) {
        return Err(BindingGap::DeliverySingleAttempt);
    }
    let prepared = prepared.as_ref().map_err(Clone::clone)?;
    let reached = prepared.reached.clone()?;
    let published = publishes(invoked, reached)?;

    let mut steps = prepared.steps();
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });
    // The second delivery, which is the whole scenario. A total match rather than a wildcard: a
    // delivery guarantee that means something different here must not inherit this, which is why
    // `at_most_once` is refused above rather than falling through to a redelivery.
    match binding.delivery {
        Delivery::AtLeastOnce => steps.push(ScenarioStep::RedeliverEvent {
            event: event.clone(),
        }),
        Delivery::AtMostOnce => return Err(BindingGap::DeliverySingleAttempt),
    }
    for event in &published {
        steps.push(ScenarioStep::EventuallyEvent {
            event: event.clone(),
            payload: BTreeMap::new(),
            shape: payload_shape(ir, event),
        });
    }

    // Where the row the invocation acts on rests once both deliveries have run.
    steps.extend(prepared.settled_row(ir));

    let text = format!(
        "`{event}` delivered twice still leaves {} observable, and no count is required",
        listed(&published)
    );
    let mut source = downstream(ir, invoked, reached);
    source.extend(prepared.source.iter().cloned());
    Ok((steps, clipped(&text), source))
}

/// §18: the declared failure policy, with the failure forced.
///
/// Forced by injection and by nothing else. A binding fills the command's input from the event, so a
/// scenario cannot choose an input that fails; the only failure it can cause is one the
/// specification declares `external:` (§12), which is why a command with no such branch refuses here
/// rather than getting a scenario that hopes.
///
/// The control is armed **after** the arrangement and before the triggering command, because
/// `ConfigureExternalOutcome` says what the adapter must produce *next*: arming it first would spend
/// the injection on whatever the arrangement's own commands set off — in a system with several
/// bindings, that is not hypothetical.
///
/// | policy | what the scenario requires | why |
/// |---|---|---|
/// | `retry` | the consequence happens anyway | one injection forces one failure; a handler that retries reaches the branch that publishes |
/// | `escalate` | the declared escalation event | since gate G2 the model names it, so this is a reading rather than a hope |
/// | `drop` on a row read before the trigger | one attempt with the mapped input, no retry in the window, the row unchanged | [`binding_effects::dropped`], beyond10x/ess#267 |
/// | `drop` on nothing that can be read so | refused | "give up silently" leaves nothing else to observe |
fn on_failure(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    prepared: &Result<binding_effects::Prepared<'_>, BindingGap>,
    event: &EventRef,
) -> Built {
    // Read before the failure is forced, so a `drop` refuses for what it is rather than for a
    // missing external branch it would not have used.
    let policy = binding.on_failure();
    if matches!(policy, ResolvedFailure::Drop) {
        let prepared = prepared.as_ref().map_err(Clone::clone)?;
        return binding_effects::dropped(ir, binding, invoked, prepared, event);
    }
    let forced = invoked
        .outcomes
        .iter()
        .find(|outcome| outcome.test_strategy == TestStrategy::InjectFault)
        .ok_or_else(|| BindingGap::NoForcibleFailure {
            command: CommandRef::new(invoked.name.clone()),
        })?;
    if let Some(gap) = forced_eligibility(invoked, forced) {
        return Err(gap);
    }
    let prepared = prepared.as_ref().map_err(Clone::clone)?;
    let forced_ref = OutcomeRef::new(CommandRef::new(invoked.name.clone()), forced.name.clone());

    let mut steps = prepared.setup.clone();
    steps.push(ScenarioStep::ConfigureExternalOutcome {
        force: forced_ref.clone(),
        times: None,
    });
    steps.extend(prepared.invoke.iter().cloned());
    steps.extend(prepared.capture.iter().cloned());
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });

    let mut source: BTreeSet<EssSemanticRef> = [
        CommandRef::new(invoked.name.clone()).into(),
        forced_ref.clone().into(),
    ]
    .into_iter()
    .collect();
    if let Some(component) = accepting_component(ir, &invoked.name) {
        source.insert(component.into());
    }
    source.extend(prepared.source.iter().cloned());

    let text = match policy {
        ResolvedFailure::Retry => {
            let reached = prepared.reached.clone()?;
            let published = publishes(invoked, reached)?;
            for event in &published {
                steps.push(ScenarioStep::EventuallyEvent {
                    event: event.clone(),
                    payload: BTreeMap::new(),
                    shape: payload_shape(ir, event),
                });
            }
            steps.extend(prepared.settled_row(ir));
            source.extend(downstream(ir, invoked, reached));
            format!(
                "`{}` retries a failed `{}` until {} is published",
                binding.name,
                invoked.name,
                listed(&published)
            )
        }
        ResolvedFailure::Escalate { emits } => {
            let escalation = EventRef::from(emits);
            steps.push(ScenarioStep::EventuallyEvent {
                event: escalation.clone(),
                payload: BTreeMap::new(),
                shape: payload_shape(ir, &escalation),
            });
            source.insert(escalation.clone().into());
            format!(
                "a failed `{}` makes `{}` escalate into `{escalation}`",
                invoked.name, binding.name
            )
        }
        // Its own scenario above, `binding_effects::dropped`.
        ResolvedFailure::Drop => unreachable!("`drop` is synthesized before the failure is forced"),
        // Its own scenario, `bounded_retry::exhausted`, which `bindings` calls instead.
        ResolvedFailure::BoundedRetry { .. } => {
            unreachable!("a bounded retry is synthesized by `bounded_retry::exhausted`")
        }
    };
    Ok((steps, clipped(&text), source))
}

/// The branch a binding's invocation reaches, or why a scenario cannot say which one.
///
/// An input decides which branch a command takes, and a binding's input comes from the event — whose
/// values the upstream implementation chose. So the branch is knowable only when the input does not
/// choose it: exactly one branch that is not externally decided. Two of those and the answer depends
/// on a value nobody has, which is a refusal rather than a guess (§11).
fn reachable_branch(invoked: &ResolvedCommand) -> Result<&ResolvedOutcome, BindingGap> {
    let reachable: Vec<&ResolvedOutcome> = invoked
        .outcomes
        .iter()
        .filter(|outcome| outcome.test_strategy != TestStrategy::InjectFault)
        .collect();
    match reachable.as_slice() {
        // One branch, and no guard on it — so no value the event carries can send the invocation
        // anywhere else. A single *guarded* branch is not the same thing: it is a branch a
        // specification says may not be taken, and a scenario that required it anyway would be
        // claiming something about values only the upstream implementation knows.
        [only] if only.test_strategy == TestStrategy::DefaultBranch => Ok(only),
        branches => Err(BindingGap::BranchUndecided {
            command: CommandRef::new(invoked.name.clone()),
            branches: branches
                .iter()
                .map(|outcome| outcome.name.clone())
                .collect(),
        }),
    }
}

/// What a branch publishes, which is what a flow is proved through.
fn publishes(
    invoked: &ResolvedCommand,
    reached: &ResolvedOutcome,
) -> Result<Vec<EventRef>, BindingGap> {
    let published: Vec<EventRef> = reached.emits.iter().map(EventRef::from).collect();
    if published.is_empty() {
        return Err(BindingGap::NothingPublished {
            outcome: OutcomeRef::new(CommandRef::new(invoked.name.clone()), reached.name.clone()),
        });
    }
    Ok(published)
}

/// The command outcome that publishes an event, in name order, or nothing where none does.
///
/// The first in the model's own order, so the choice is a function of the specification and moves
/// only when the specification does (§37). Where two branches publish one event, either would make
/// the binding fire; taking the lower-named one keeps the suite stable when a third is added.
fn publisher<'ir>(
    ir: &'ir EssIr,
    event: &ess_compiler::ir::EventHandle,
) -> Option<(&'ir ResolvedCommand, &'ir ResolvedOutcome)> {
    ir.commands().values().find_map(|command| {
        command
            .outcomes
            .iter()
            .find(|outcome| outcome.emits.contains(event))
            .map(|outcome| (command, outcome))
    })
}

/// Everything a binding scenario's result rests on before its own aspect adds to it.
///
/// Including the two **components** it crosses, which is what a binding is: an event one component
/// publishes invoking a command another accepts. A semantic diff asking "did this change touch
/// anything this scenario rests on" has to be able to answer yes when a command moves between
/// components, and nothing else in a scenario names one.
fn binding_source(
    ir: &EssIr,
    binding: &ResolvedBinding,
    publisher: &ResolvedCommand,
    published_by: &ResolvedOutcome,
    trigger: &Run,
) -> BTreeSet<EssSemanticRef> {
    let mut source = trigger.source.clone();
    source.insert(BindingRef::new(binding.name.clone()).into());
    source.insert(EventRef::from(binding.cause.event().expect("event branch")).into());
    let command_ref = CommandRef::new(publisher.name.clone());
    source.insert(command_ref.clone().into());
    source.insert(OutcomeRef::new(command_ref, published_by.name.clone()).into());
    if let Some(subject) = &published_by.subject {
        source.insert(EntityRef::from(&subject.entity).into());
    }
    if let Some(actor) = trigger.actor.clone() {
        source.insert(actor.into());
    }
    if let Some(component) = accepting_component(ir, &publisher.name) {
        source.insert(component.into());
    }
    source.extend(
        input_types(ir, publisher)
            .into_iter()
            .map(EssSemanticRef::from),
    );
    source
}

/// What the branch a binding reaches depends on: the command, the branch, and what it publishes.
fn downstream(
    ir: &EssIr,
    invoked: &ResolvedCommand,
    reached: &ResolvedOutcome,
) -> BTreeSet<EssSemanticRef> {
    let command_ref = CommandRef::new(invoked.name.clone());
    let mut source: BTreeSet<EssSemanticRef> = [
        command_ref.clone().into(),
        OutcomeRef::new(command_ref, reached.name.clone()).into(),
    ]
    .into_iter()
    .collect();
    for event in &reached.emits {
        source.insert(EventRef::from(event).into());
        for field in &ir.event(event).fields {
            let mut types = BTreeSet::new();
            reachable_types(ir, &field.type_ref, &mut types);
            source.extend(types.into_iter().map(EssSemanticRef::from));
        }
    }
    if let Some(component) = accepting_component(ir, &invoked.name) {
        source.insert(component.into());
    }
    source
}

/// The component that accepts a command, where one does.
///
/// By name rather than by handle, because a command reached from a binding's own side is a
/// `ResolvedCommand`. Deterministic: the components are a [`BTreeMap`], and `ess-domain` refuses one
/// command accepted by two components.
fn accepting_component(ir: &EssIr, command: &QualifiedName) -> Option<ComponentRef> {
    ir.components()
        .values()
        .find(|component| {
            component
                .accepts
                .iter()
                .any(|accepted| accepted.name() == command)
        })
        .map(|component| ComponentRef::new(component.name.clone()))
}

/// `a`, `a and b`, `a, b and c` — for the one line a report prints beside a verdict.
fn listed(events: &[EventRef]) -> String {
    let written: Vec<String> = events.iter().map(|event| format!("`{event}`")).collect();
    match written.split_last() {
        None => "nothing".to_owned(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

/// The actor a scenario acts as, per command, where the specification grants one.
///
/// Read off [`EssIr::grants`] rather than by walking `may` again, and the lowest-named actor where a
/// command is granted to several: any of them is authorised, and choosing by name is the choice
/// that does not move when an unrelated actor is declared.
fn granted_actors(ir: &EssIr) -> BTreeMap<QualifiedName, ActorRef> {
    ir.grants()
        .iter()
        .filter_map(|(command, actors)| {
            actors
                .first()
                .map(|actor| (command.name().clone(), ActorRef::new(actor.name.clone())))
        })
        .collect()
}

/// Every view that holds one row per instance of its source, by the entity it projects.
///
/// [`EssIr::projections`] without the aggregate views: an aggregate row is a group, not an
/// instance, so a picker that finds an instance's row there — by its identity, its `state`, or a
/// field it carries — would be asserting about a row that does not exist. Aggregate views have a
/// family of their own (`aggregate`), and every row-level picker in this module reads this or
/// filters on [`ResolvedView::is_aggregate`].
fn row_projections(ir: &EssIr) -> BTreeMap<&EntityHandle, Vec<&ResolvedView>> {
    let mut out = ir.projections();
    for views in out.values_mut() {
        views.retain(|view| !view.is_aggregate());
    }
    out.retain(|_, views| !views.is_empty());
    out
}

/// The construct a scenario id is about.
fn subject_of(id: &ScenarioId) -> EssSemanticRef {
    match id {
        ScenarioId::Disclosure { cell } => cell.origin.clone().into(),
        ScenarioId::Outcome { outcome } => outcome.clone().into(),
        ScenarioId::Transition { transition, .. } => transition.clone().into(),
        ScenarioId::Refusal { entity, .. } | ScenarioId::Invariant { entity, .. } => {
            entity.clone().into()
        }
        ScenarioId::ValueInvariant { value, .. } => value.clone().into(),
        ScenarioId::Binding { binding, .. } => binding.clone().into(),
        ScenarioId::Aggregate { view } => view.clone().into(),
        ScenarioId::Grant { command } | ScenarioId::GrantAdmitted { command, .. } => {
            command.clone().into()
        }
        // Synthesis mints every id it refuses about and mints no authored one — an authored
        // scenario is a person's claim, compiled and refused by [`crate::authored`] in a vocabulary
        // of its own. The arm is here because the match is total, and it answers with the one
        // construct the id does carry rather than inventing a command the author never named.
        ScenarioId::Authored { domain, .. } => domain.clone().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_refusal_carries_a_distinct_code_in_one_family() {
        // A code is what a harness matches on, so two causes sharing one is two defects a reader
        // cannot tell apart. Built here rather than derived, so adding a variant without deciding
        // its number fails to compile.
        let causes = [
            RefusalCause::NoWitness(WitnessGap {
                path: "amount".to_owned(),
                type_ref: "A".to_owned(),
                reason: "refers to itself",
            }),
            RefusalCause::GuardUnevaluable(Unevaluable {
                predicate: "amount.vat > 0".to_owned(),
                command: "witness.orders.TaxOrder".to_owned(),
                causes: Vec::new(),
            }),
            RefusalCause::GuardUnsatisfiable {
                predicate: "never".to_owned(),
                tried: 4,
            },
            RefusalCause::InstanceRequired {
                entity: EntityRef::new(
                    QualifiedName::new("billing.invoice.Invoice").expect("valid"),
                ),
                need: InstanceNeed::Updates,
                reason: Unreachable::NothingCreates,
            },
            RefusalCause::ViewUndecidable {
                view: ViewRef::new(QualifiedName::new("billing.invoice.ById").expect("valid")),
                filter: "total.amount > 0".to_owned(),
                state: StateName::new("Draft").expect("valid"),
                unbound: Vec::new(),
            },
            RefusalCause::NotSynthesisedYet {
                construct: "a binding",
                sections: "§16",
            },
            RefusalCause::DuplicateScenario,
            RefusalCause::StrategyWithoutGuard {
                strategy: TestStrategy::ConstructInput,
            },
            RefusalCause::BindingUnobservable {
                binding: BindingRef::new(
                    ess_domain::binding::BindingName::new("notify-on-invoice-created")
                        .expect("valid"),
                ),
                gap: BindingGap::PolicySilent,
            },
            RefusalCause::InvariantUnobservable {
                entity: EntityRef::new(QualifiedName::new("oracle.order.Order").expect("valid")),
                invariant: "weight_grams >= 0".to_owned(),
                unpublished: vec![FactPath::new("weight_grams").expect("a fact path")],
                unassertable: Vec::new(),
                state: StateName::new("Placed").expect("valid"),
            },
            RefusalCause::AggregateUnscoped {
                view: ViewRef::new(QualifiedName::new("metrics.session.ByState").expect("valid")),
            },
            RefusalCause::AggregateUnwitnessed {
                view: ViewRef::new(QualifiedName::new("metrics.session.ByState").expect("valid")),
                reason: "a parameter is read inside a disjunction".to_owned(),
            },
            RefusalCause::CountUnwitnessed {
                path: FactPath::new("keys.count").expect("a fact path"),
                literal: "5000".to_owned(),
                bound: crate::witness::MAX_COUNT_WITNESS,
            },
        ];

        let codes: BTreeSet<String> = causes
            .iter()
            .map(|cause| cause.code().to_string())
            .collect();
        assert_eq!(
            codes.len(),
            causes.len(),
            "two causes share a code: {codes:?}"
        );
        for code in &codes {
            assert!(
                code.starts_with("ESS-SYNTH-"),
                "{code} is not in this module's family"
            );
        }
        for cause in &causes {
            assert!(
                !cause.hint().is_empty(),
                "`{cause}` says nothing about what would have to change"
            );
        }
    }

    #[test]
    fn a_refusal_names_the_construct_the_code_and_the_repair() {
        let refusal = Refusal {
            subject: EssSemanticRef::Entity {
                name: EntityRef::new(QualifiedName::new("billing.invoice.Invoice").expect("valid")),
            },
            scenario: None,
            cause: RefusalCause::InstanceRequired {
                entity: EntityRef::new(
                    QualifiedName::new("billing.invoice.Invoice").expect("valid"),
                ),
                need: InstanceNeed::InState {
                    state: StateName::new("Paid").expect("valid"),
                },
                reason: Unreachable::NoPath {
                    from: StateName::new("Draft").expect("valid"),
                },
            },
        };
        let rendered = refusal.to_string();

        assert!(rendered.contains("ESS-SYNTH-004"), "{rendered}");
        assert!(rendered.contains("billing.invoice.Invoice"), "{rendered}");
        assert!(rendered.contains("Paid"), "{rendered}");
        assert!(
            rendered.contains("help:"),
            "a refusal that does not say what to change is a refusal nobody can act on: {rendered}"
        );
    }
}
