//! The mutation audit: specification mutants, each replayed against an unchanged target.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, Part 1. A green suite means something only
//! once a broken implementation has turned it red. This module derives the broken cases from the
//! **specification**: each mutant is the parsed documents with one altering edit, compiled again
//! through the same `assemble` and `compile` an author meets, synthesized into a fresh ordinary
//! suite, and run against a target that implements the **unchanged** specification.
//!
//! # What a verdict means
//!
//! A mutant is *killed* when its suite fails against that target: synthesis asked a question whose
//! answer differs between the two specifications, and the target gave the original answer. A
//! *survivor* is a finding about what synthesis asks of the model — either the mutant is
//! equivalent, or no generated scenario observes the rule. It is **not** answered by authoring a
//! scenario: an authored scenario's expectations are its author's, not the model's, so it runs
//! identically in every mutant's suite and could never kill one. That is why no authored scenario
//! is run here at all.
//!
//! No class *weakens* the specification. On a branch that creates its row, dropping a `sets` entry
//! says the value is the implementation's to choose, and a correct target satisfies a weaker
//! specification by definition, so such a mutant could never be killed and would be noise shaped
//! like signal. On a branch that acts on an existing row the same drop *alters* it: the field keeps
//! what the row held instead of taking the input (beyond10x/ess#212), so `sets-drop` is offered
//! there and only there.
//!
//! # Why the parsed documents, and not the IR
//!
//! [`EssIr`]'s fields are private and it is built only from crate-private parts. An IR mutation API
//! could build IR no source produces, and it would give up validation. A mutant of the raw
//! documents that the model does not admit is refused by the check an author would meet, and is
//! recorded as [`Verdict::Stillborn`] with that check's own code — so no admissibility rule of
//! this module's own sits beside the validator.
//!
//! # Determinism
//!
//! Mutants are enumerated and run in byte order of id, every run is on a fresh target, and the
//! runner is deterministic by construction, so two audits of one tree produce identical bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;

use ess_compiler::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::command::{PayloadSource, RawCommandSpec, RawOutcome};
use ess_domain::entity::{RawEntitySpec, StateName, Transition};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::view::{Direction, RawViewSpec};
use ess_primitives::predicate::{CompareOp, Predicate};

use crate::report::Status;
use crate::runner::Runner;
use crate::target::ConformanceTarget;
use crate::AdmittedSuite;

/// One parsed source document, as `ess-cli`'s loader produces it.
pub type Document = (Source, RawSpecFile);

/// The family every code of this module carries.
pub const FAMILY: &str = "MUTATE";

/// The document family the audit writes.
pub const REPORT_FORMAT: &str = "ess-mutation-report/3";
/// The report [`collect`] writes for an emission scoped to one component: `/3` with the component
/// named and the mutants it leaves out listed (beyond10x/ess#236).
pub const REPORT_FORMAT_4: &str = "ess-mutation-report/4";

// ---- the classes --------------------------------------------------------------------------------

/// The eleven altering classes, a closed set with an [`ALL`](Self::ALL) constant as `Fault` has.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum MutantClass {
    /// Remove one state from a transition's `from`, where a command moving along it declares a
    /// `wrong_state` outcome and `from` holds at least two states.
    FromDrop,
    /// Send a transition to the first other state, in an entity with at least two states.
    TransitionTo,
    /// Move a guard boundary in an outcome's `when`: swap the strictness of an ordering comparison
    /// (`>=`↔`>`, `<=`↔`<`); move the integral literal of a `>=` or `<=` one step outward, the
    /// direction the swap does not take; or flip `==`↔`!=` on a leaf that is not the whole guard.
    GuardBoundary,
    /// Take a `sets` value from the first other input field of the identical written type.
    SetsRetarget,
    /// Negate an outcome's `when`.
    GuardNegate,
    /// Swap `all` and `any` in an outcome's `when`.
    GuardConnective,
    /// Refuse with the first other error declared in the command's domain.
    ErrorSwap,
    /// Stop emitting one event, and drop its `payload:` entry.
    EmitDrop,
    /// Flip one ranking key's direction.
    OrderFlip,
    /// Drop one `sets` entry reading an input field, on a branch that updates or moves an existing
    /// row: the field keeps what the row held.
    SetsDrop,
    /// Swap two adjacent branches guarded by the input alone, both accepting or both refusing, so
    /// the second answers where both guards hold.
    PrecedenceSwap,
}

impl MutantClass {
    /// Every class, in the order the design lists them.
    pub const ALL: &'static [Self] = &[
        Self::FromDrop,
        Self::TransitionTo,
        Self::GuardBoundary,
        Self::SetsRetarget,
        Self::GuardNegate,
        Self::GuardConnective,
        Self::ErrorSwap,
        Self::EmitDrop,
        Self::OrderFlip,
        Self::SetsDrop,
        Self::PrecedenceSwap,
    ];

    /// The oldest manifest format whose readers know this class: [`MANIFEST_FORMAT_4`] for the two
    /// classes beyond10x/ess#212 added, [`MANIFEST_FORMAT_1`] for the rest.
    pub fn manifest_format(self) -> &'static str {
        match self {
            Self::SetsDrop | Self::PrecedenceSwap => MANIFEST_FORMAT_4,
            _ => MANIFEST_FORMAT_1,
        }
    }

    /// The kebab name `--class` takes, and the first segment of a mutant's id.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FromDrop => "from-drop",
            Self::TransitionTo => "transition-to",
            Self::GuardBoundary => "guard-boundary",
            Self::SetsRetarget => "sets-retarget",
            Self::GuardNegate => "guard-negate",
            Self::GuardConnective => "guard-connective",
            Self::ErrorSwap => "error-swap",
            Self::EmitDrop => "emit-drop",
            Self::OrderFlip => "order-flip",
            Self::SetsDrop => "sets-drop",
            Self::PrecedenceSwap => "precedence-swap",
        }
    }
}

impl fmt::Display for MutantClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---- the codes ----------------------------------------------------------------------------------

/// The codes of the `MUTATE` family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MutateCode {
    /// A baseline scenario failed or ended `error` against the target, so no failure under a
    /// mutant can be shown to be *because of* the mutant. A scenario the target reported
    /// `unsupported` or `skipped` is not scored rather than red.
    BaselineFailed,
    /// A mutant was refused by `assemble` or `compile`.
    Stillborn,
    /// The selected classes found no site: an audit that ran nothing and exited 0 would be a green
    /// exit that checked nothing.
    NoSite,
    /// A mutant's suite gained synthesis refusals the baseline's does not have, and no scored
    /// scenario failed: what the mutant changed has no scenario in its suite, so passing what is
    /// left says nothing about it. Also a mutant on an outcome whose scenario the baseline's suite
    /// already refused, or on a transition only such outcomes perform: neither suite can witness
    /// what it changed.
    Unwitnessed,
    /// A guard mutant left its outcome's guard satisfied by no input, and no scored scenario
    /// failed and none it changed went unscored: the rule it wrote is dead, not missed.
    Equivalent,
}

impl MutateCode {
    /// Every code.
    pub const ALL: &'static [Self] = &[
        Self::BaselineFailed,
        Self::Stillborn,
        Self::NoSite,
        Self::Unwitnessed,
        Self::Equivalent,
    ];

    /// Its stable code, derived from the variant as `RefusalCause::code` derives its own: the
    /// number is the variant's entry in [`MutateCode::CATALOGUE`].
    pub fn code(self) -> Code {
        Code::new(FAMILY, self.catalogue_entry().key)
    }
}

crate::authored::diagnostic_catalogue! {
    impl MutateCode => u16 {
        Self::BaselineFailed => 1,
            "A baseline scenario failed or ended `error` against the target, so no failure under \
             a mutant can be shown to be because of the mutant.",
            "make the unmutated suite pass against the target first; a scenario reported \
             `unsupported` or `skipped` is not scored and does not cause this";
        Self::Stillborn => 2,
            "A mutant was refused by `assemble` or `compile`, so there was nothing to run.",
            "nothing to repair in the specification; the mutant is reported and not scored";
        Self::NoSite => 3,
            "The selected mutant classes found no site in this specification, so the audit would \
             run nothing.",
            "select classes that apply to the specification, or leave `--class` out to run \
             every class";
        Self::Unwitnessed => 4,
            "A mutant's suite gained synthesis refusals the baseline's does not have and no \
             scored scenario failed, so passing what is left says nothing about the mutant.",
            "repair the synthesis refusal the verdict names, so the suite has a scenario for \
             what the mutant changed";
        Self::Equivalent => 5,
            "A guard mutant left its outcome's guard satisfied by no input: the rule it wrote is \
             dead, not missed.",
            "nothing to repair in the implementation; read the named guard, which no input can \
             satisfy under the mutation";
    }
}

// ---- mutations ----------------------------------------------------------------------------------

/// One altering edit to the parsed documents, addressed by the names it changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutation {
    /// Remove `state` from `transition`'s `from`.
    FromDrop {
        /// The entity.
        entity: String,
        /// The transition's local name.
        transition: String,
        /// The state removed.
        state: String,
    },
    /// Send `transition` to `to`.
    TransitionTo {
        /// The entity.
        entity: String,
        /// The transition's local name.
        transition: String,
        /// The new arrival state.
        to: String,
    },
    /// Swap the strictness of the `leaf`-th ordering comparison of `outcome`'s `when`, in
    /// pre-order.
    GuardBoundary {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// Which ordering comparison, counted in pre-order from zero.
        leaf: usize,
    },
    /// Take the `sets` value for `target` from the input field `field`.
    SetsRetarget {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// The entity field set.
        target: String,
        /// The sibling input field it is now read from.
        field: String,
    },
    /// Negate `outcome`'s `when`.
    GuardNegate {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
    },
    /// Swap `all` and `any` at the `node`-th connective of `outcome`'s `when`, in pre-order.
    GuardConnective {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// Which connective with at least two children, counted in pre-order from zero.
        node: usize,
    },
    /// Refuse with `error` instead.
    ErrorSwap {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// The error it now names.
        error: String,
    },
    /// Stop emitting `event`, and drop its `payload:` entry.
    EmitDrop {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// The event.
        event: String,
    },
    /// Flip the direction of the ranking key on `field`.
    OrderFlip {
        /// The view.
        view: String,
        /// The ranked field.
        field: String,
    },
    /// Move the integral literal of the `leaf`-th ordering comparison of `outcome`'s `when`, a
    /// `>=` or `<=`, one step outward: `x >= 10` becomes `x >= 9`. A class `guard-boundary` site.
    GuardOutward {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// Which ordering comparison, counted in pre-order from zero as [`Self::GuardBoundary`]
        /// counts it.
        leaf: usize,
    },
    /// Flip the `leaf`-th `==` or `!=` comparison of `outcome`'s `when`, in pre-order, where it is
    /// not the whole guard. A class `guard-boundary` site.
    GuardEquality {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// Which equality comparison, counted in pre-order from zero.
        leaf: usize,
    },
    /// Drop the `sets` entry for `target` of a branch that acts on an existing row.
    SetsDrop {
        /// The command.
        command: String,
        /// The outcome.
        outcome: String,
        /// The entity field no longer set.
        target: String,
    },
    /// Swap `first` with `second`, the outcome declared right after it.
    PrecedenceSwap {
        /// The command.
        command: String,
        /// The outcome declared first.
        first: String,
        /// The outcome declared right after it.
        second: String,
    },
}

impl Mutation {
    /// Its class.
    pub fn class(&self) -> MutantClass {
        match self {
            Self::FromDrop { .. } => MutantClass::FromDrop,
            Self::TransitionTo { .. } => MutantClass::TransitionTo,
            Self::GuardBoundary { .. } | Self::GuardOutward { .. } | Self::GuardEquality { .. } => {
                MutantClass::GuardBoundary
            }
            Self::SetsDrop { .. } => MutantClass::SetsDrop,
            Self::PrecedenceSwap { .. } => MutantClass::PrecedenceSwap,
            Self::SetsRetarget { .. } => MutantClass::SetsRetarget,
            Self::GuardNegate { .. } => MutantClass::GuardNegate,
            Self::GuardConnective { .. } => MutantClass::GuardConnective,
            Self::ErrorSwap { .. } => MutantClass::ErrorSwap,
            Self::EmitDrop { .. } => MutantClass::EmitDrop,
            Self::OrderFlip { .. } => MutantClass::OrderFlip,
        }
    }

    /// The site: the id without its class.
    pub fn site(&self) -> String {
        match self {
            Self::FromDrop {
                entity,
                transition,
                state,
            } => format!("{entity}.{transition}/{state}"),
            Self::TransitionTo {
                entity, transition, ..
            } => format!("{entity}.{transition}"),
            Self::GuardBoundary {
                command,
                outcome,
                leaf,
            } => format!("{command}/{outcome}/{leaf}"),
            Self::GuardOutward {
                command,
                outcome,
                leaf,
            } => format!("{command}/{outcome}/{leaf}-outward"),
            Self::GuardEquality {
                command,
                outcome,
                leaf,
            } => format!("{command}/{outcome}/equality-{leaf}"),
            Self::SetsRetarget {
                command,
                outcome,
                target,
                ..
            }
            | Self::SetsDrop {
                command,
                outcome,
                target,
            } => format!("{command}/{outcome}/{target}"),
            Self::PrecedenceSwap {
                command,
                first,
                second,
            } => format!("{command}/{first}/{second}"),
            Self::GuardNegate { command, outcome }
            | Self::ErrorSwap {
                command, outcome, ..
            } => {
                format!("{command}/{outcome}")
            }
            Self::GuardConnective {
                command,
                outcome,
                node,
            } => format!("{command}/{outcome}/{node}"),
            Self::EmitDrop {
                command,
                outcome,
                event,
            } => format!("{command}/{outcome}/{event}"),
            Self::OrderFlip { view, field } => format!("{view}/{field}"),
        }
    }
}

/// One mutant: an id, its class, a sentence saying what changed, and the edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mutant {
    /// `<class>/<site>`.
    pub id: String,
    /// Its class.
    pub class: MutantClass,
    /// What changed, in the document's own words.
    pub change: String,
    /// The edit.
    pub mutation: Mutation,
}

impl Mutant {
    /// Describes `mutation` against `documents`, or `None` when its site is not there.
    pub fn new(documents: &[Document], mutation: Mutation) -> Option<Self> {
        let change = describe(documents, &mutation)?;
        Some(Self {
            id: format!("{}/{}", mutation.class(), mutation.site()),
            class: mutation.class(),
            change,
            mutation,
        })
    }
}

// ---- finding things in the documents ------------------------------------------------------------

fn entities(documents: &[Document]) -> impl Iterator<Item = &RawEntitySpec> {
    documents.iter().flat_map(|(_, file)| &file.entities)
}

fn commands(documents: &[Document]) -> impl Iterator<Item = &RawCommandSpec> {
    documents.iter().flat_map(|(_, file)| &file.commands)
}

fn views(documents: &[Document]) -> impl Iterator<Item = &RawViewSpec> {
    documents.iter().flat_map(|(_, file)| &file.views)
}

fn transition<'a>(documents: &'a [Document], entity: &str, name: &str) -> Option<&'a Transition> {
    entities(documents)
        .find(|it| it.name.to_string() == entity)?
        .states
        .transitions
        .iter()
        .find(|it| it.name == name)
}

fn outcome<'a>(documents: &'a [Document], command: &str, name: &str) -> Option<&'a RawOutcome> {
    commands(documents)
        .find(|it| it.name.to_string() == command)?
        .outcomes
        .iter()
        .find(|it| it.name.to_string() == name)
}

fn transition_mut<'a>(
    documents: &'a mut [Document],
    entity: &str,
    name: &str,
) -> Option<&'a mut Transition> {
    documents
        .iter_mut()
        .flat_map(|(_, file)| &mut file.entities)
        .find(|it| it.name.to_string() == entity)?
        .states
        .transitions
        .iter_mut()
        .find(|it| it.name == name)
}

fn outcome_mut<'a>(
    documents: &'a mut [Document],
    command: &str,
    name: &str,
) -> Option<&'a mut RawOutcome> {
    documents
        .iter_mut()
        .flat_map(|(_, file)| &mut file.commands)
        .find(|it| it.name.to_string() == command)?
        .outcomes
        .iter_mut()
        .find(|it| it.name.to_string() == name)
}

/// The qualified name's namespace: everything before its last segment.
fn namespace(name: &str) -> &str {
    name.rsplit_once('.').map_or("", |(head, _)| head)
}

fn states(set: &BTreeSet<StateName>) -> String {
    set.iter()
        .map(StateName::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Every ordering comparison of a predicate, in pre-order.
fn ordering_leaves(predicate: &Predicate) -> Vec<&Predicate> {
    let mut found = Vec::new();
    preorder(predicate, &mut |node| {
        if matches!(
            node,
            Predicate::Compare {
                op: CompareOp::Lt | CompareOp::Le | CompareOp::Gt | CompareOp::Ge,
                ..
            }
        ) {
            found.push(node);
        }
    });
    found
}

/// Every `all`/`any` of at least two children, in pre-order.
fn connectives(predicate: &Predicate) -> Vec<&Predicate> {
    let mut found = Vec::new();
    preorder(predicate, &mut |node| {
        if matches!(node, Predicate::All(children) | Predicate::Any(children) if children.len() >= 2)
        {
            found.push(node);
        }
    });
    found
}

fn preorder<'a>(predicate: &'a Predicate, visit: &mut dyn FnMut(&'a Predicate)) {
    visit(predicate);
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                preorder(child, visit);
            }
        }
        Predicate::Not(inner) => preorder(inner, visit),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            preorder(&quantified.body, visit);
        }
        _ => {}
    }
}

/// Applies `edit` to the `index`-th node, in pre-order, that `select` picks.
fn edit_nth(
    predicate: &mut Predicate,
    select: &dyn Fn(&Predicate) -> bool,
    index: usize,
    edit: &dyn Fn(&mut Predicate),
) -> bool {
    fn walk(
        node: &mut Predicate,
        select: &dyn Fn(&Predicate) -> bool,
        seen: &mut usize,
        index: usize,
        edit: &dyn Fn(&mut Predicate),
    ) -> bool {
        if select(node) {
            if *seen == index {
                edit(node);
                return true;
            }
            *seen += 1;
        }
        match node {
            Predicate::All(children) | Predicate::Any(children) => children
                .iter_mut()
                .any(|child| walk(child, select, seen, index, edit)),
            Predicate::Not(inner) => walk(inner, select, seen, index, edit),
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                walk(&mut quantified.body, select, seen, index, edit)
            }
            _ => false,
        }
    }
    let mut seen = 0;
    walk(predicate, select, &mut seen, index, edit)
}

fn is_ordering(node: &Predicate) -> bool {
    matches!(
        node,
        Predicate::Compare {
            op: CompareOp::Lt | CompareOp::Le | CompareOp::Gt | CompareOp::Ge,
            ..
        }
    )
}

fn is_connective(node: &Predicate) -> bool {
    matches!(node, Predicate::All(children) | Predicate::Any(children) if children.len() >= 2)
}

fn swap_strictness(node: &mut Predicate) {
    if let Predicate::Compare { op, .. } = node {
        *op = match *op {
            CompareOp::Ge => CompareOp::Gt,
            CompareOp::Gt => CompareOp::Ge,
            CompareOp::Le => CompareOp::Lt,
            CompareOp::Lt => CompareOp::Le,
            other => other,
        };
    }
}

fn is_equality(node: &Predicate) -> bool {
    matches!(
        node,
        Predicate::Compare {
            op: CompareOp::Eq | CompareOp::Ne,
            ..
        }
    )
}

/// Every `==` and `!=` comparison of a guard, in pre-order, or none where one of them is the whole
/// guard, or the whole guard negated: flipping it there is what `guard-negate` already does.
fn equality_leaves(guard: &Predicate) -> Vec<&Predicate> {
    let whole = match guard {
        Predicate::Not(inner) => inner,
        other => other,
    };
    if is_equality(whole) {
        return Vec::new();
    }
    let mut found = Vec::new();
    preorder(guard, &mut |node| {
        if is_equality(node) {
            found.push(node);
        }
    });
    found
}

fn flip_equality(node: &mut Predicate) {
    if let Predicate::Compare { op, .. } = node {
        *op = match *op {
            CompareOp::Eq => CompareOp::Ne,
            CompareOp::Ne => CompareOp::Eq,
            other => other,
        };
    }
}

/// An ordering comparison of an input with an integral literal under `>=` or `<=`, the literal
/// moved one step outward — away from the accepting side, so the accepting side gains the value
/// just past the old boundary. `None` for a strict comparison, whose strictness swap already moves
/// it outward, for a literal that is not an integer, and where the step leaves `i64`.
fn outward(node: &Predicate) -> Option<Predicate> {
    use ess_primitives::facts::{FactValue, Number};
    use ess_primitives::predicate::Operand;
    let Predicate::Compare { left, op, right } = node else {
        return None;
    };
    let step = |value: &FactValue, by: i64| -> Option<Operand> {
        let FactValue::Number(number) = value else {
            return None;
        };
        let moved = number.as_i64()?.checked_add(by)?;
        Some(Operand::Literal(FactValue::Number(Number::from(moved))))
    };
    let (left, right) = match (left, op, right) {
        (Operand::Fact(_), CompareOp::Ge, Operand::Literal(value)) => {
            (left.clone(), step(value, -1)?)
        }
        (Operand::Fact(_), CompareOp::Le, Operand::Literal(value)) => {
            (left.clone(), step(value, 1)?)
        }
        (Operand::Literal(value), CompareOp::Le, Operand::Fact(_)) => {
            (step(value, -1)?, right.clone())
        }
        (Operand::Literal(value), CompareOp::Ge, Operand::Fact(_)) => {
            (step(value, 1)?, right.clone())
        }
        _ => return None,
    };
    Some(Predicate::Compare {
        left,
        op: *op,
        right,
    })
}

fn move_outward(node: &mut Predicate) {
    if let Some(moved) = outward(node) {
        *node = moved;
    }
}

/// Whether an outcome is guarded by its input alone: a `when:` that is not `Always`, and nothing
/// the held row, a related row, a provider or a replay decides. Only such branches are ordered by
/// declaration among themselves (`docs/design/input-guard-overlap-precedence.md`).
fn guarded_by_input_alone(outcome: &RawOutcome) -> bool {
    outcome
        .when
        .as_ref()
        .is_some_and(|when| *when != Predicate::Always)
        && outcome.when_subject.is_none()
        && outcome.when_related.is_none()
        && outcome.when_subject_state.is_none()
        && outcome.when_state_changes.is_none()
        && outcome.external.is_none()
        && outcome.replays.is_none()
        && !outcome.wrong_state
        && !outcome.unknown_instance
        && !outcome.input_absent
        && !outcome.existing_instance
}

/// Whether an outcome acts on a row that already exists: it updates or moves one, and creates none.
fn acts_on_existing_row(outcome: &RawOutcome) -> bool {
    outcome.creates.is_none() && (outcome.updates.is_some() || outcome.moves.is_some())
}

/// Whether `target`, a field of the row `outcome` updates or moves, is declared `Optional<…>`.
///
/// Such a field nothing wrote is held absent, and no synthesized row expectation states an
/// absence, so a `sets-drop` there leaves a suite that asks nothing about the field: unkillable by
/// construction, the noise the audit does not generate. It is left out of `sets-drop` by name.
fn optional_field(documents: &[Document], outcome: &RawOutcome, target: &str) -> bool {
    let entity = match (&outcome.updates, &outcome.moves) {
        (Some(entity), _) => entity.to_string(),
        (None, Some(moves)) => namespace(&moves.to_string()).to_owned(),
        (None, None) => return false,
    };
    entities(documents)
        .find(|it| it.name.to_string() == entity)
        .and_then(|it| it.fields.iter().find(|field| field.name == target))
        .is_some_and(|field| field.type_ref.is_optional())
}

fn swap_connective(node: &mut Predicate) {
    *node = match std::mem::take(node) {
        Predicate::All(children) => Predicate::Any(children),
        Predicate::Any(children) => Predicate::All(children),
        other => other,
    };
}

fn flipped(direction: Direction) -> Direction {
    match direction {
        Direction::Ascending => Direction::Descending,
        Direction::Descending => Direction::Ascending,
    }
}

// ---- enumeration --------------------------------------------------------------------------------

/// Every mutant of the selected classes, in byte order of id.
pub fn mutants(documents: &[Document], classes: &[MutantClass]) -> Vec<Mutant> {
    let selected: BTreeSet<MutantClass> = classes.iter().copied().collect();
    let mut found: Vec<Mutant> = sites(documents)
        .into_iter()
        .filter(|mutation| selected.contains(&mutation.class()))
        .filter_map(|mutation| Mutant::new(documents, mutation))
        .collect();
    found.sort_by(|left, right| left.id.cmp(&right.id));
    found.dedup_by(|left, right| left.id == right.id);
    found
}

/// Every site of every class.
fn sites(documents: &[Document]) -> Vec<Mutation> {
    let mut found = Vec::new();
    transition_sites(documents, &mut found);
    for command in commands(documents) {
        outcome_sites(documents, command, &mut found);
    }
    for view in views(documents) {
        for key in &view.order_by {
            found.push(Mutation::OrderFlip {
                view: view.name.to_string(),
                field: key.field.clone(),
            });
        }
    }
    found
}

/// The `transition-to` and `from-drop` sites.
fn transition_sites(documents: &[Document], found: &mut Vec<Mutation>) {
    for entity in entities(documents) {
        let name = entity.name.to_string();
        let machine = &entity.states;
        for transition in &machine.transitions {
            let qualified = format!("{name}.{}", transition.name);
            if machine.states.len() >= 2 {
                if let Some(to) = machine.states.iter().find(|state| **state != transition.to) {
                    found.push(Mutation::TransitionTo {
                        entity: name.clone(),
                        transition: transition.name.clone(),
                        to: to.as_str().to_owned(),
                    });
                }
            }
            let refused_elsewhere = commands(documents).any(|command| {
                command.outcomes.iter().any(|outcome| {
                    outcome
                        .moves
                        .as_ref()
                        .is_some_and(|moves| moves.to_string() == qualified)
                }) && command.outcomes.iter().any(|outcome| outcome.wrong_state)
            });
            if transition.from.len() >= 2 && refused_elsewhere {
                for state in &transition.from {
                    found.push(Mutation::FromDrop {
                        entity: name.clone(),
                        transition: transition.name.clone(),
                        state: state.as_str().to_owned(),
                    });
                }
            }
        }
    }
}

/// The sites one command's outcomes offer: guards, `sets`, errors and events.
fn outcome_sites(documents: &[Document], command: &RawCommandSpec, found: &mut Vec<Mutation>) {
    let command_name = command.name.to_string();
    let domain = namespace(&command_name);
    let mut errors: Vec<String> = documents
        .iter()
        .flat_map(|(_, file)| &file.errors)
        .map(|error| error.name.to_string())
        .filter(|error| namespace(error) == domain)
        .collect();
    errors.sort();
    for outcome in &command.outcomes {
        let outcome_name = outcome.name.to_string();
        let at = |mutation: fn(String, String) -> Mutation| {
            mutation(command_name.clone(), outcome_name.clone())
        };
        if let Some(when) = &outcome.when {
            boundary_sites(&command_name, &outcome_name, when, found);
            if *when != Predicate::Always {
                found.push(at(|command, outcome| Mutation::GuardNegate {
                    command,
                    outcome,
                }));
            }
            for node in 0..connectives(when).len() {
                found.push(Mutation::GuardConnective {
                    command: command_name.clone(),
                    outcome: outcome_name.clone(),
                    node,
                });
            }
        }
        for set in &outcome.sets.0 {
            let PayloadSource::InputField { field } = &set.source else {
                continue;
            };
            if acts_on_existing_row(outcome) && !optional_field(documents, outcome, &set.target) {
                found.push(Mutation::SetsDrop {
                    command: command_name.clone(),
                    outcome: outcome_name.clone(),
                    target: set.target.clone(),
                });
            }
            let Some(written) = command.input.iter().find(|input| input.name == *field) else {
                continue;
            };
            if let Some(sibling) = command
                .input
                .iter()
                .find(|input| input.name != *field && input.type_ref == written.type_ref)
            {
                found.push(Mutation::SetsRetarget {
                    command: command_name.clone(),
                    outcome: outcome_name.clone(),
                    target: set.target.clone(),
                    field: sibling.name.clone(),
                });
            }
        }
        if let Some(current) = &outcome.error {
            let current = current.to_string();
            if let Some(other) = errors.iter().find(|error| **error != current) {
                found.push(Mutation::ErrorSwap {
                    command: command_name.clone(),
                    outcome: outcome_name.clone(),
                    error: other.clone(),
                });
            }
        }
        for event in &outcome.emits {
            found.push(Mutation::EmitDrop {
                command: command_name.clone(),
                outcome: outcome_name.clone(),
                event: event.to_string(),
            });
        }
    }
    precedence_sites(command, found);
}

/// The `guard-boundary` sites of one outcome's `when`: each ordering comparison's strictness swap
/// and, where it has one, its outward literal; then each equality that is not the whole guard.
fn boundary_sites(command: &str, outcome: &str, when: &Predicate, found: &mut Vec<Mutation>) {
    for (leaf, node) in ordering_leaves(when).into_iter().enumerate() {
        found.push(Mutation::GuardBoundary {
            command: command.to_owned(),
            outcome: outcome.to_owned(),
            leaf,
        });
        if outward(node).is_some() {
            found.push(Mutation::GuardOutward {
                command: command.to_owned(),
                outcome: outcome.to_owned(),
                leaf,
            });
        }
    }
    for leaf in 0..equality_leaves(when).len() {
        found.push(Mutation::GuardEquality {
            command: command.to_owned(),
            outcome: outcome.to_owned(),
            leaf,
        });
    }
}

/// The `precedence-swap` sites of one command: two adjacent branches the input alone guards, both
/// accepting or both refusing, where the first declared whose guard holds answers. An input-guarded
/// refusal comes before every accepting branch whatever the order, so a refusal and an accepting
/// branch are never a pair.
fn precedence_sites(command: &RawCommandSpec, found: &mut Vec<Mutation>) {
    for pair in command.outcomes.windows(2) {
        let [first, second] = pair else { continue };
        if guarded_by_input_alone(first)
            && guarded_by_input_alone(second)
            && first.error.is_some() == second.error.is_some()
        {
            found.push(Mutation::PrecedenceSwap {
                command: command.name.to_string(),
                first: first.name.to_string(),
                second: second.name.to_string(),
            });
        }
    }
}

/// What `mutation` changes, in the words the document uses, or `None` when its site is absent.
fn describe(documents: &[Document], mutation: &Mutation) -> Option<String> {
    Some(match mutation {
        Mutation::FromDrop {
            entity,
            transition,
            state,
        } => {
            let from = &self::transition(documents, entity, transition)?.from;
            let kept: BTreeSet<StateName> = from
                .iter()
                .filter(|it| it.as_str() != state)
                .cloned()
                .collect();
            if kept.len() == from.len() {
                return None;
            }
            format!(
                "`from: [{}]` becomes `from: [{}]`",
                states(from),
                states(&kept)
            )
        }
        Mutation::TransitionTo {
            entity,
            transition,
            to,
        } => {
            let current = &self::transition(documents, entity, transition)?.to;
            format!("`to: {current}` becomes `to: {to}`")
        }
        Mutation::GuardBoundary { .. }
        | Mutation::GuardOutward { .. }
        | Mutation::GuardEquality { .. }
        | Mutation::GuardNegate { .. }
        | Mutation::GuardConnective { .. } => return describe_guard(documents, mutation),
        Mutation::SetsDrop { .. } | Mutation::PrecedenceSwap { .. } => {
            return describe_drop_or_swap(documents, mutation)
        }
        Mutation::SetsRetarget {
            command,
            outcome,
            target,
            field,
        } => {
            let set = self::outcome(documents, command, outcome)?
                .sets
                .0
                .iter()
                .find(|set| set.target == *target)?;
            format!(
                "`{target}: {}` becomes `{target}: input.{field}`",
                set.source
            )
        }
        Mutation::ErrorSwap {
            command,
            outcome,
            error,
        } => {
            let current = self::outcome(documents, command, outcome)?.error.as_ref()?;
            format!("`error: {current}` becomes `error: {error}`")
        }
        Mutation::EmitDrop {
            command,
            outcome,
            event,
        } => {
            let emits = &self::outcome(documents, command, outcome)?.emits;
            if !emits.iter().any(|it| it.to_string() == *event) {
                return None;
            }
            format!("`{event}` is no longer emitted")
        }
        Mutation::OrderFlip { view, field } => {
            let key = views(documents)
                .find(|it| it.name.to_string() == *view)?
                .order_by
                .iter()
                .find(|key| key.field == *field)?;
            format!(
                "`{field} {}` becomes `{field} {}`",
                key.direction.as_str(),
                flipped(key.direction).as_str()
            )
        }
    })
}

/// What a `sets-drop` or `precedence-swap` changes, or `None` when its site is absent.
fn describe_drop_or_swap(documents: &[Document], mutation: &Mutation) -> Option<String> {
    Some(match mutation {
        Mutation::SetsDrop {
            command,
            outcome,
            target,
        } => {
            let set = self::outcome(documents, command, outcome)?
                .sets
                .0
                .iter()
                .find(|set| set.target == *target)?;
            format!("`{target}: {}` is no longer set", set.source)
        }
        Mutation::PrecedenceSwap {
            command,
            first,
            second,
        } => {
            let guard = |name: &str| self::outcome(documents, command, name)?.when.as_ref();
            format!(
                "`{first}` (`{}`) and `{second}` (`{}`) change places, so `{second}` answers \
                 where both hold",
                guard(first)?,
                guard(second)?
            )
        }
        _ => return None,
    })
}

/// What a guard mutation changes: the node before and after, or `None` when its site is absent.
fn describe_guard(documents: &[Document], mutation: &Mutation) -> Option<String> {
    Some(match mutation {
        Mutation::GuardBoundary {
            command,
            outcome,
            leaf,
        } => {
            let when = self::outcome(documents, command, outcome)?.when.as_ref()?;
            let mut changed = when.clone();
            let before = ordering_leaves(when).get(*leaf)?.to_string();
            let mut after = String::new();
            edit_nth(&mut changed, &is_ordering, *leaf, &swap_strictness);
            if let Some(node) = ordering_leaves(&changed).get(*leaf) {
                after = node.to_string();
            }
            format!("`{before}` becomes `{after}`")
        }
        Mutation::GuardOutward {
            command,
            outcome,
            leaf,
        } => {
            let when = self::outcome(documents, command, outcome)?.when.as_ref()?;
            let before = ordering_leaves(when).get(*leaf).copied()?;
            format!("`{before}` becomes `{}`", outward(before)?)
        }
        Mutation::GuardEquality {
            command,
            outcome,
            leaf,
        } => {
            let when = self::outcome(documents, command, outcome)?.when.as_ref()?;
            let before = equality_leaves(when).get(*leaf).copied()?;
            let mut after = before.clone();
            flip_equality(&mut after);
            format!("`{before}` becomes `{after}`")
        }
        Mutation::GuardNegate { command, outcome } => {
            let when = self::outcome(documents, command, outcome)?.when.as_ref()?;
            format!(
                "`when: {when}` becomes `when: {}`",
                Predicate::not(when.clone())
            )
        }
        Mutation::GuardConnective {
            command,
            outcome,
            node,
        } => {
            let when = self::outcome(documents, command, outcome)?.when.as_ref()?;
            let before = connectives(when).get(*node)?.to_string();
            let mut changed = when.clone();
            edit_nth(&mut changed, &is_connective, *node, &swap_connective);
            let after = connectives(&changed)
                .get(*node)
                .map(ToString::to_string)
                .unwrap_or_default();
            format!("`{before}` becomes `{after}`")
        }
        _ => return None,
    })
}

// ---- applying one -------------------------------------------------------------------------------

/// The documents with `mutation` applied: a clone with one edit, or why the site is not there.
pub fn apply(documents: &[Document], mutation: &Mutation) -> Result<Vec<Document>, String> {
    let mut mutated = documents.to_vec();
    let absent = || format!("the specification has no site `{}`", mutation.site());
    match mutation {
        Mutation::FromDrop {
            entity,
            transition,
            state,
        } => {
            let found = transition_mut(&mut mutated, entity, transition).ok_or_else(absent)?;
            let before = found.from.len();
            found.from.retain(|it| it.as_str() != state);
            if found.from.len() == before {
                return Err(absent());
            }
        }
        Mutation::TransitionTo {
            entity,
            transition,
            to,
        } => {
            let to = StateName::new(to).map_err(|error| error.to_string())?;
            transition_mut(&mut mutated, entity, transition)
                .ok_or_else(absent)?
                .to = to;
        }
        Mutation::GuardBoundary { .. }
        | Mutation::GuardOutward { .. }
        | Mutation::GuardEquality { .. }
        | Mutation::GuardNegate { .. }
        | Mutation::GuardConnective { .. } => apply_guard(&mut mutated, mutation)?,
        Mutation::SetsDrop { .. } | Mutation::PrecedenceSwap { .. } => {
            apply_drop_or_swap(&mut mutated, mutation)?;
        }
        Mutation::SetsRetarget {
            command,
            outcome,
            target,
            field,
        } => {
            let set = outcome_mut(&mut mutated, command, outcome)
                .and_then(|it| it.sets.0.iter_mut().find(|set| set.target == *target))
                .ok_or_else(absent)?;
            set.source = PayloadSource::InputField {
                field: field.clone(),
            };
        }
        Mutation::ErrorSwap {
            command,
            outcome,
            error,
        } => {
            let error = ess_domain::name::QualifiedName::new(error).map_err(|e| e.to_string())?;
            let found = outcome_mut(&mut mutated, command, outcome).ok_or_else(absent)?;
            if found.error.is_none() {
                return Err(absent());
            }
            found.error = Some(error);
        }
        Mutation::EmitDrop {
            command,
            outcome,
            event,
        } => {
            let found = outcome_mut(&mut mutated, command, outcome).ok_or_else(absent)?;
            let before = found.emits.len();
            found.emits.retain(|it| it.to_string() != *event);
            if found.emits.len() == before {
                return Err(absent());
            }
            found.payload.0.retain(|(it, _)| it.to_string() != *event);
        }
        Mutation::OrderFlip { view, field } => {
            let key = mutated
                .iter_mut()
                .flat_map(|(_, file)| &mut file.views)
                .find(|it| it.name.to_string() == *view)
                .and_then(|it| it.order_by.iter_mut().find(|key| key.field == *field))
                .ok_or_else(absent)?;
            key.direction = flipped(key.direction);
        }
    }
    Ok(mutated)
}

/// Applies a `sets-drop` or `precedence-swap` in place, or says why its site is not there.
fn apply_drop_or_swap(mutated: &mut [Document], mutation: &Mutation) -> Result<(), String> {
    let absent = || format!("the specification has no site `{}`", mutation.site());
    match mutation {
        Mutation::SetsDrop {
            command,
            outcome,
            target,
        } => {
            let found = outcome_mut(mutated, command, outcome).ok_or_else(absent)?;
            let before = found.sets.0.len();
            found.sets.0.retain(|set| set.target != *target);
            if found.sets.0.len() == before {
                return Err(absent());
            }
        }
        Mutation::PrecedenceSwap {
            command,
            first,
            second,
        } => {
            let outcomes = &mut mutated
                .iter_mut()
                .flat_map(|(_, file)| &mut file.commands)
                .find(|it| it.name.to_string() == *command)
                .ok_or_else(absent)?
                .outcomes;
            let at = outcomes
                .iter()
                .position(|it| it.name.to_string() == *first)
                .ok_or_else(absent)?;
            if outcomes
                .get(at + 1)
                .is_none_or(|next| next.name.to_string() != *second)
            {
                return Err(absent());
            }
            outcomes.swap(at, at + 1);
        }
        _ => return Err(absent()),
    }
    Ok(())
}

/// Applies a guard mutation in place, or says why its site is not there.
fn apply_guard(mutated: &mut [Document], mutation: &Mutation) -> Result<(), String> {
    let absent = || format!("the specification has no site `{}`", mutation.site());
    match mutation {
        Mutation::GuardBoundary {
            command,
            outcome,
            leaf,
        } => {
            let when = outcome_mut(mutated, command, outcome)
                .and_then(|it| it.when.as_mut())
                .ok_or_else(absent)?;
            if !edit_nth(when, &is_ordering, *leaf, &swap_strictness) {
                return Err(absent());
            }
        }
        Mutation::GuardOutward {
            command,
            outcome,
            leaf,
        } => {
            let when = outcome_mut(mutated, command, outcome)
                .and_then(|it| it.when.as_mut())
                .ok_or_else(absent)?;
            if ordering_leaves(when)
                .get(*leaf)
                .is_none_or(|node| outward(node).is_none())
            {
                return Err(absent());
            }
            edit_nth(when, &is_ordering, *leaf, &move_outward);
        }
        Mutation::GuardEquality {
            command,
            outcome,
            leaf,
        } => {
            let when = outcome_mut(mutated, command, outcome)
                .and_then(|it| it.when.as_mut())
                .ok_or_else(absent)?;
            if *leaf >= equality_leaves(when).len() {
                return Err(absent());
            }
            edit_nth(when, &is_equality, *leaf, &flip_equality);
        }
        Mutation::GuardNegate { command, outcome } => {
            let when = outcome_mut(mutated, command, outcome)
                .and_then(|it| it.when.as_mut())
                .ok_or_else(absent)?;
            *when = Predicate::not(std::mem::take(when));
        }
        Mutation::GuardConnective {
            command,
            outcome,
            node,
        } => {
            let when = outcome_mut(mutated, command, outcome)
                .and_then(|it| it.when.as_mut())
                .ok_or_else(absent)?;
            if !edit_nth(when, &is_connective, *node, &swap_connective) {
                return Err(absent());
            }
        }
        _ => return Err(absent()),
    }
    Ok(())
}

// ---- compiling one ------------------------------------------------------------------------------

/// Why a mutant never ran: the refusal the model's own validation gave it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stillborn {
    /// The refusal's own code, such as `ESS-ENTITY-003`.
    pub cause: String,
    /// `ESS-MUTATE-002`.
    pub code: String,
    /// The refusal's own message.
    pub message: String,
}

impl Stillborn {
    fn from_diagnostics(diagnostics: &Diagnostics) -> Self {
        let first: Option<&Diagnostic> = diagnostics
            .as_slice()
            .iter()
            .find(|it| it.severity == Severity::Error)
            .or_else(|| diagnostics.as_slice().first());
        Self {
            cause: first.map_or_else(String::new, |it| it.code.to_string()),
            code: MutateCode::Stillborn.code().to_string(),
            message: first.map_or_else(String::new, |it| it.message.clone()),
        }
    }
}

/// Assembles and compiles documents exactly as the loader does, or the first refusal.
pub fn compile(documents: Vec<Document>, texts: &SourceMap) -> Result<EssIr, Stillborn> {
    let labels: Vec<String> = documents
        .iter()
        .map(|(source, _)| source.to_string())
        .collect();
    let specification = Specification::assemble(documents).map_err(|errors| {
        Stillborn::from_diagnostics(&ess_compiler::resolve::diagnose_locating(
            &errors, texts, &labels,
        ))
    })?;
    ess_compiler::compile(&specification, texts)
        .map_err(|diagnostics| Stillborn::from_diagnostics(&diagnostics))
}

// ---- dead guards and unwitnessed outcomes -------------------------------------------------------

/// The command and outcome mutant `id` of `class` is on, for the classes that edit one outcome.
///
/// The id is `<class>/<command>/<outcome>[/…]`: a qualified command name holds no `/`, and
/// neither does an outcome name. A `precedence-swap` is on the outcome declared first.
fn outcome_site(class: MutantClass, id: &str) -> Option<(&str, &str)> {
    match class {
        MutantClass::GuardBoundary
        | MutantClass::SetsRetarget
        | MutantClass::GuardNegate
        | MutantClass::GuardConnective
        | MutantClass::ErrorSwap
        | MutantClass::EmitDrop
        | MutantClass::SetsDrop
        | MutantClass::PrecedenceSwap => {}
        MutantClass::FromDrop | MutantClass::TransitionTo | MutantClass::OrderFlip => return None,
    }
    let mut segments = id
        .strip_prefix(class.as_str())?
        .strip_prefix('/')?
        .split('/');
    Some((segments.next()?, segments.next()?))
}

/// The guard a guard mutant left its outcome, rendered, where no input satisfies it and the
/// baseline's guard at the same outcome is satisfied by one: the mutant made the outcome dead.
///
/// `None` for any other mutation, where either guard is not a plain input guard, where the
/// baseline's guard is not satisfied either (the mutant did not make it dead), and wherever
/// [`satisfiable`] cannot decide.
///
/// For a `precedence-swap`, the overlap of the two guards in the baseline, where no input satisfies
/// it: the two branches never compete, so their order decides nothing.
fn unsatisfiable_guard(baseline: &EssIr, mutant: &EssIr, mutation: &Mutation) -> Option<String> {
    if let Mutation::PrecedenceSwap {
        command,
        first,
        second,
    } = mutation
    {
        let (found, one) = guard_of(baseline, command, first)?;
        let (_, other) = guard_of(baseline, command, second)?;
        let overlap = Predicate::All(vec![one.clone(), other.clone()]);
        return (!satisfiable(baseline, found, &overlap)?).then(|| overlap.to_string());
    }
    let (Mutation::GuardBoundary {
        command, outcome, ..
    }
    | Mutation::GuardOutward {
        command, outcome, ..
    }
    | Mutation::GuardEquality {
        command, outcome, ..
    }
    | Mutation::GuardNegate { command, outcome }
    | Mutation::GuardConnective {
        command, outcome, ..
    }) = mutation
    else {
        return None;
    };
    let (before, was) = guard_of(baseline, command, outcome)?;
    if !satisfiable(baseline, before, was)? {
        return None;
    }
    let (after, now) = guard_of(mutant, command, outcome)?;
    (!satisfiable(mutant, after, now)?).then(|| now.to_string())
}

/// The command of `ir` named `command`, and the input guard of its outcome `outcome`.
fn guard_of<'ir>(
    ir: &'ir EssIr,
    command: &str,
    outcome: &str,
) -> Option<(&'ir ess_compiler::ir::ResolvedCommand, &'ir Predicate)> {
    let found = ir
        .commands()
        .values()
        .find(|it| it.name.to_string() == command)?;
    let guard = found
        .outcomes
        .iter()
        .find(|it| it.name.to_string() == outcome)
        .and_then(crate::when)?;
    Some((found, guard))
}

/// Whether some input satisfies `guard`, or `None` where that is not decided here.
///
/// Decided only where every test in the guard compares one scalar input leaf, never optional,
/// never a count and never a `Timestamp`, with literals by equality, membership or truth. Over such
/// a guard, which literal each leaf equals — or none — is all that decides it, so each leaf's
/// domain is finite: a boolean's two values, an enum's variants, and for any other text or number
/// its literals and one value none of them equals. Every combination of those domains is
/// evaluated, and only where their product is at most
/// [`MAX_CANDIDATES`](crate::witness::MAX_CANDIDATES): completeness is that product, counted before
/// any type rule or invariant is applied, never how
/// many admitted inputs a bounded walk returned (beyond10x/ess#218). A combination no admitted
/// input takes — a literal or an other value an invariant refuses — is still tried, so the answer
/// can only err towards satisfiable, which leaves a mutant scored as before; it never calls a
/// satisfiable guard dead. An ordering, a text match, a quantifier or a comparison of two facts is
/// not decided: its satisfying values may lie between any finite set of representatives.
fn satisfiable(
    ir: &EssIr,
    command: &ess_compiler::ir::ResolvedCommand,
    guard: &Predicate,
) -> Option<bool> {
    use ess_domain::expression::ScalarKind;
    use ess_primitives::facts::{FactPath, FactValue};
    let mut leaves: std::collections::BTreeMap<FactPath, Vec<FactValue>> =
        std::collections::BTreeMap::new();
    if !equality_tests(guard, &mut leaves) {
        return None;
    }
    let environment = ess_compiler::expression::Environment::new(ir, &command.input);
    let mut domains: Vec<(FactPath, Vec<FactValue>)> = Vec::new();
    let mut combinations: usize = 1;
    for (path, literals) in &leaves {
        let resolved =
            ess_domain::expression::resolve_path(&environment, path, "mutation audit").ok()?;
        if resolved.optional || resolved.access.collection || resolved.access.text_length {
            return None;
        }
        let scalar = resolved.scalar?;
        // Truth of a text or a number is not a question of which literal it equals.
        if scalar != ScalarKind::Bool && literals.iter().any(|it| matches!(it, FactValue::Bool(_)))
        {
            return None;
        }
        // A declared `Timestamp` compares by the instant it names, so two spellings of one instant
        // are equal and one representative per literal does not stand for every value.
        if crate::input::declared_as(
            ir,
            &command.input,
            path,
            ess_domain::types::Primitive::Timestamp,
        ) {
            return None;
        }
        let mut domain: Vec<FactValue> = Vec::new();
        let mut add = |value: FactValue| {
            if !domain.contains(&value) {
                domain.push(value);
            }
        };
        match (scalar, &resolved.variants) {
            (ScalarKind::Bool, _) => {
                add(FactValue::Bool(false));
                add(FactValue::Bool(true));
            }
            (ScalarKind::Text, Some(variants)) => {
                for variant in variants {
                    add(FactValue::Text(variant.clone()));
                }
                literals
                    .iter()
                    .filter(|it| matches!(it, FactValue::Text(_)))
                    .for_each(|it| add(it.clone()));
            }
            (ScalarKind::Text | ScalarKind::Number, _) => {
                let text = scalar == ScalarKind::Text;
                literals
                    .iter()
                    .filter(|it| matches!(it, FactValue::Text(_)) == text)
                    .for_each(|it| add(it.clone()));
                // One value none of the literals equals stands for every such value: the guard
                // asks of this leaf only which literal it equals, or none.
                let other = (0..=literals.len())
                    .map(|index| {
                        if text {
                            FactValue::Text(format!("~{index}"))
                        } else {
                            FactValue::count(index)
                        }
                    })
                    .find(|candidate| !literals.contains(candidate))?;
                add(other);
            }
        }
        combinations = combinations.checked_mul(domain.len())?;
        if combinations > crate::witness::MAX_CANDIDATES {
            return None;
        }
        domains.push((path.clone(), domain));
    }
    for index in 0..combinations {
        let mut rest = index;
        let facts: ess_primitives::facts::FactStore = domains
            .iter()
            .map(|(path, domain)| {
                let value = domain[rest % domain.len()].clone();
                rest /= domain.len();
                (path.clone(), value)
            })
            .collect();
        match guard.evaluate(&facts) {
            ess_primitives::predicate::Truth::True => return Some(true),
            ess_primitives::predicate::Truth::False => {}
            ess_primitives::predicate::Truth::Unknown => return None,
        }
    }
    Some(false)
}

/// Whether `predicate` tests only input leaves against literals by `==`, `!=`, membership or
/// truth, under `all`, `any` and `not`; each leaf read is recorded with the literals it meets.
fn equality_tests(
    predicate: &Predicate,
    leaves: &mut std::collections::BTreeMap<
        ess_primitives::facts::FactPath,
        Vec<ess_primitives::facts::FactValue>,
    >,
) -> bool {
    use ess_primitives::predicate::Operand;
    match predicate {
        Predicate::Always | Predicate::Never => true,
        Predicate::All(children) | Predicate::Any(children) => {
            children.iter().all(|child| equality_tests(child, leaves))
        }
        Predicate::Not(inner) => equality_tests(inner, leaves),
        Predicate::Compare {
            left,
            op: CompareOp::Eq | CompareOp::Ne,
            right,
        } => match (left, right) {
            (Operand::Fact(path), Operand::Literal(value))
            | (Operand::Literal(value), Operand::Fact(path)) => {
                leaves.entry(path.clone()).or_default().push(value.clone());
                true
            }
            _ => false,
        },
        Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
            leaves
                .entry(path.clone())
                .or_default()
                .extend(values.iter().cloned());
            true
        }
        Predicate::Truthy(path) => {
            leaves
                .entry(path.clone())
                .or_default()
                .push(ess_primitives::facts::FactValue::Bool(true));
            true
        }
        _ => false,
    }
}

// ---- verdicts -----------------------------------------------------------------------------------

/// What one mutant came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// At least one scored scenario failed.
    Killed,
    /// Every scored scenario passed, and no scenario it changed went unscored.
    Survived,
    /// No scored scenario failed, and at least one ended `error`, `unsupported` or `skipped`, or
    /// a scenario it changed was excluded because the baseline did not execute it: nothing
    /// contradicted the mutant, and nobody found out.
    Inconclusive,
    /// `ESS-MUTATE-004`: no scored scenario failed, and the mutant's suite gained synthesis
    /// refusals the baseline's does not have, or the mutant is on an outcome whose scenario the
    /// baseline's suite refused, or on a transition only such outcomes perform. What the mutant
    /// changed has no scenario, so what is left passing is not a finding that the suite misses it.
    Unwitnessed,
    /// `ESS-MUTATE-005`: no scored scenario failed, and the mutant left its outcome's guard
    /// satisfied by no input, and no scenario it changed went unscored.
    /// The rule it wrote is dead by construction, so no scenario of its suite can take it.
    Equivalent,
    /// `assemble` or `compile` refused the mutant.
    Stillborn,
}

impl Verdict {
    /// The verdict a set of scenario statuses comes to. `Unsupported` is never `Passed`.
    pub fn classify(statuses: &[Status]) -> Self {
        if statuses.contains(&Status::Failed) {
            Self::Killed
        } else if statuses
            .iter()
            .any(|status| matches!(status, Status::Error | Status::Unsupported))
        {
            Self::Inconclusive
        } else {
            Self::Survived
        }
    }

    /// The verdict of a mutant whose scored statuses are `statuses`, whose suite `gained`
    /// synthesis refusals over the baseline's, and which `hidden` a scenario: one it changed that
    /// was excluded because the baseline did not execute it. A failure kills it whatever else
    /// holds; otherwise a gained refusal makes it [`Unwitnessed`](Self::Unwitnessed), and a hidden
    /// scenario makes it [`Inconclusive`](Self::Inconclusive). An excluded scenario the mutant
    /// holds unchanged asks the target what the baseline asked, so it cannot hide a kill.
    pub fn judge(statuses: &[Status], gained: bool, hidden: bool) -> Self {
        match Self::classify(statuses) {
            Self::Killed => Self::Killed,
            _ if gained => Self::Unwitnessed,
            Self::Survived if hidden => Self::Inconclusive,
            verdict => verdict,
        }
    }

    /// How it is written.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Killed => "killed",
            Self::Survived => "survived",
            Self::Inconclusive => "inconclusive",
            Self::Unwitnessed => "unwitnessed",
            Self::Equivalent => "equivalent",
            Self::Stillborn => "stillborn",
        }
    }

    /// This verdict for a mutant that left its guard satisfied by no input: an otherwise
    /// `survived` or `unwitnessed` mutant is [`Equivalent`](Self::Equivalent). A failure still
    /// kills it, and `inconclusive` stands: a changed scenario nothing scored could have killed it
    /// (the fallback scenario a dead region is probed with does).
    #[must_use]
    pub fn with_dead_guard(self) -> Self {
        match self {
            Self::Survived | Self::Unwitnessed | Self::Equivalent => Self::Equivalent,
            Self::Killed | Self::Inconclusive | Self::Stillborn => self,
        }
    }
}

/// One synthesis refusal, as the audit tells two apart: its code, the scenario that would have
/// existed where there is one, and its subject. Two refusals may still share a key, so suites are
/// compared key by key with counts.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct RefusalKey {
    /// The refusal's code, `ESS-SYNTH-…`.
    pub code: String,
    /// The scenario that would have existed; absent when the refusal is about none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    /// The ESS element refused, and for an unobservable invariant the invariant; absent only in a
    /// key a hand-written manifest gives no subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

impl RefusalKey {
    /// The key of one refusal: its code, its scenario where it has one, and its subject.
    ///
    /// The subject is the element refused, except where one scenario, or one element with no
    /// scenario, is refused once per something smaller, which then is the subject: the invariant
    /// for `ESS-SYNTH-011`, the view for `ESS-SYNTH-005` and `ESS-SYNTH-014`, and the input path
    /// for `ESS-SYNTH-001`, which is refused once per `Binary64` use of one element.
    pub fn of(refusal: &crate::Refusal) -> Self {
        use crate::RefusalCause;
        let subject = match &refusal.cause {
            RefusalCause::InvariantUnobservable {
                entity, invariant, ..
            } => format!("{entity} invariant `{invariant}`"),
            RefusalCause::ViewUndecidable { view, .. }
            | RefusalCause::OrderUnwitnessed { view, .. } => format!("view {view}"),
            RefusalCause::NoWitness(gap) => format!("{} at `{}`", refusal.subject, gap.path),
            _ => refusal.subject.to_string(),
        };
        Self {
            code: refusal.code().to_string(),
            scenario: refusal.scenario.as_ref().map(ToString::to_string),
            subject: Some(subject),
        }
    }

    /// At least one of `scenario` and `subject`, or why not.
    fn check(&self) -> Result<(), String> {
        if self.scenario.is_none() && self.subject.is_none() {
            return Err(format!(
                "a refusal `{}` names neither a `scenario` nor a `subject`",
                self.code
            ));
        }
        Ok(())
    }
}

impl fmt::Display for RefusalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.scenario, &self.subject) {
            (Some(scenario), Some(subject)) => {
                write!(f, "{} `{scenario}` ({subject})", self.code)
            }
            (Some(about), None) | (None, Some(about)) => write!(f, "{} `{about}`", self.code),
            (None, None) => write!(f, "{}", self.code),
        }
    }
}

/// Why a baseline scenario is not scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotScoredStatus {
    /// The target cannot expose what the scenario observes.
    Unsupported,
    /// The runner skipped it.
    Skipped,
}

/// A baseline scenario the target did not execute, so no mutant is scored on it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NotScored {
    /// The scenario.
    pub scenario: String,
    /// What the baseline's run reported for it.
    pub status: NotScoredStatus,
}

// ---- the report ---------------------------------------------------------------------------------

/// One mutant's entry in the report. Fields are declared in key order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MutantEntry {
    /// The synthesis refusals the mutant's suite has and the baseline's does not, sorted; absent
    /// when there are none, or when an `ess-mutation-manifest/1` emission names none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_refusals: Option<Vec<RefusalKey>>,
    /// The baseline's synthesis refusals of the scenario of the outcome the mutant is on, sorted,
    /// where no baseline scenario expects that outcome — or, for a `from-drop` or `transition-to`
    /// mutant, of every outcome that performs its transition, where no baseline scenario expects
    /// any of them: neither suite witnesses what the mutant changed. Absent otherwise, and when the
    /// baseline's refusals are known only by count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_refusals: Option<Vec<RefusalKey>>,
    /// What changed.
    pub change: String,
    /// Its class.
    pub class: MutantClass,
    /// Scenarios of the mutant's suite not scored because the baseline's run reported the same
    /// scenario `unsupported` or `skipped`, sorted; absent when there are none. A scenario new to
    /// the mutant's suite is scored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded: Option<Vec<String>>,
    /// `<class>/<site>`.
    pub id: String,
    /// The scenarios that failed, sorted; only on `killed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub killers: Option<Vec<String>>,
    /// Synthesis refusals of the mutant's suite; absent on `stillborn`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusals: Option<usize>,
    /// Scenarios of the mutant's suite; absent on `stillborn`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenarios: Option<usize>,
    /// Why it never ran; only on `stillborn`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stillborn: Option<Stillborn>,
    /// The guard the mutant left its outcome, as written, where no input satisfies it while some
    /// input satisfies the baseline's guard there; absent where that is not so or cannot be
    /// decided, and in a report collected from an `ess-mutation-manifest/1` or `/2` emission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsatisfiable_guard: Option<String>,
    /// Why its report was not scored; only on an `inconclusive` mutant of a collected audit
    /// ([`collect`]) whose report is missing, unreadable or of another suite or implementation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unscored: Option<String>,
    /// What it came to.
    pub verdict: Verdict,
}

/// The baseline suite's size.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SuiteSize {
    /// The scenarios the target did not execute, in suite order: no mutant is scored on them.
    pub not_scored: Vec<NotScored>,
    /// Synthesis refusals.
    pub refusals: usize,
    /// Scenarios.
    pub scenarios: usize,
}

/// How many mutants came to each verdict.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Counts {
    /// Equivalent.
    pub equivalent: usize,
    /// Inconclusive.
    pub inconclusive: usize,
    /// Killed.
    pub killed: usize,
    /// Every mutant.
    pub mutants: usize,
    /// Stillborn.
    pub stillborn: usize,
    /// Survived.
    pub survived: usize,
    /// Unwitnessed.
    pub unwitnessed: usize,
}

impl Counts {
    fn count(&mut self, verdict: Verdict) {
        match verdict {
            Verdict::Killed => self.killed += 1,
            Verdict::Survived => self.survived += 1,
            Verdict::Inconclusive => self.inconclusive += 1,
            Verdict::Unwitnessed => self.unwitnessed += 1,
            Verdict::Equivalent => self.equivalent += 1,
            Verdict::Stillborn => self.stillborn += 1,
        }
    }
}

/// A mutant of a component-scoped emission whose site belongs to another component, which is
/// that component's to answer: listed, not scored.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OutOfScope {
    /// What changed.
    pub change: String,
    /// Its class.
    pub class: MutantClass,
    /// `<class>/<site>`.
    pub id: String,
}

/// The `ess-mutation-report/3` document, or `/4` where it is scored for one component: keys sorted,
/// no timestamp, so its bytes are a function of the tree and the target.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MutationReport {
    /// The unmutated suite.
    pub baseline: SuiteSize,
    /// The component the suites were scoped to; only in [`REPORT_FORMAT_4`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// How many mutants came to each verdict. Out-of-scope mutants are not counted.
    pub counts: Counts,
    /// [`REPORT_FORMAT`], or [`REPORT_FORMAT_4`] where a component is named.
    pub format: String,
    /// The implementation that answered.
    pub implementation: String,
    /// Every scored or stillborn mutant, in byte order of id.
    pub mutants: Vec<MutantEntry>,
    /// The mutants that change no scenario of the component's suite, in byte order of id; only in
    /// [`REPORT_FORMAT_4`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_of_scope: Option<Vec<OutOfScope>>,
    /// The original specification's digest.
    pub spec_digest: String,
    /// `<system> <version>`.
    pub specification: String,
}

impl MutationReport {
    /// Two-space JSON with sorted keys and one trailing LF.
    pub fn to_canonical_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("the report serializes");
        json.push('\n');
        json
    }

    /// Why nothing could kill `entry`, as its text line says after the change: the code of an
    /// `unwitnessed` or `equivalent` verdict, the dead guard, and the refusals of either suite.
    fn write_reasons(&self, tail: &mut String, entry: &MutantEntry) {
        if entry.verdict == Verdict::Unwitnessed {
            let _ = write!(tail, " — {}", MutateCode::Unwitnessed.code());
            if entry.added_refusals.is_none() && entry.baseline_refusals.is_none() {
                let _ = write!(
                    tail,
                    ": {} refusal(s) against the baseline's {}",
                    entry.refusals.unwrap_or_default(),
                    self.baseline.refusals
                );
            }
        }
        if let Some(guard) = &entry.unsatisfiable_guard {
            if entry.verdict == Verdict::Equivalent {
                let _ = write!(tail, " — {}:", MutateCode::Equivalent.code());
            } else {
                tail.push(';');
            }
            let _ = write!(tail, " no input satisfies `{guard}`");
        }
        if let Some(refused) = &entry.baseline_refusals {
            let named: Vec<String> = refused.iter().map(ToString::to_string).collect();
            let _ = write!(tail, "; the baseline refuses {}", named.join(", "));
        }
        if let Some(added) = &entry.added_refusals {
            let named: Vec<String> = added.iter().map(ToString::to_string).collect();
            let _ = write!(tail, "; adds {}", named.join(", "));
        }
    }

    /// One summary line, then the baseline scenarios not scored, then survivors, unwitnessed,
    /// inconclusive, equivalent, stillborn and killed, one line each.
    pub fn render_text(&self) -> String {
        let counts = &self.counts;
        let not_scored = &self.baseline.not_scored;
        let out_of_scope = self.out_of_scope.as_deref().unwrap_or_default();
        let mut out = format!(
            "mutation audit of {}{} against {}: {} mutant(s), {} killed, {} survived, {} \
             inconclusive, {} stillborn, {} unwitnessed, {} equivalent (baseline: {} \
             scenario(s), {} refusal(s){}){}\n",
            self.specification,
            self.component
                .as_ref()
                .map(|component| format!(" for component `{component}`"))
                .unwrap_or_default(),
            self.implementation,
            counts.mutants,
            counts.killed,
            counts.survived,
            counts.inconclusive,
            counts.stillborn,
            counts.unwitnessed,
            counts.equivalent,
            self.baseline.scenarios,
            self.baseline.refusals,
            if not_scored.is_empty() {
                String::new()
            } else {
                format!(", {} not scored", not_scored.len())
            },
            if self.component.is_some() {
                format!("; {} out of scope", out_of_scope.len())
            } else {
                String::new()
            },
        );
        for skipped in not_scored {
            let status = match skipped.status {
                NotScoredStatus::Unsupported => "unsupported",
                NotScoredStatus::Skipped => "skipped",
            };
            let _ = writeln!(
                out,
                "not scored {}: the baseline's run reported it {status}",
                skipped.scenario
            );
        }
        if let Some(component) = &self.component {
            for left in out_of_scope {
                let _ = writeln!(
                    out,
                    "out of scope {}: {} — its site belongs to another component than \
                     `{component}`",
                    left.id, left.change
                );
            }
        }
        for verdict in [
            Verdict::Survived,
            Verdict::Unwitnessed,
            Verdict::Inconclusive,
            Verdict::Equivalent,
            Verdict::Stillborn,
            Verdict::Killed,
        ] {
            for entry in self.mutants.iter().filter(|entry| entry.verdict == verdict) {
                let mut tail = match (&entry.killers, &entry.stillborn) {
                    (Some(killers), _) => {
                        let named = killers.iter().take(3).cloned().collect::<Vec<_>>();
                        format!(
                            " — by {}{} ({} in total)",
                            named.join(", "),
                            if killers.len() > 3 { ", …" } else { "" },
                            killers.len()
                        )
                    }
                    (None, Some(stillborn)) => format!(
                        " — {} {}: {}",
                        stillborn.code, stillborn.cause, stillborn.message
                    ),
                    (None, None) => String::new(),
                };
                self.write_reasons(&mut tail, entry);
                if let Some(excluded) = &entry.excluded {
                    let named = excluded.iter().take(3).cloned().collect::<Vec<_>>();
                    let _ = write!(
                        tail,
                        "; not scored, as the baseline did not execute them: {}{} ({} in total)",
                        named.join(", "),
                        if excluded.len() > 3 { ", …" } else { "" },
                        excluded.len()
                    );
                }
                let _ = writeln!(
                    out,
                    "{} {}: {}{tail}",
                    verdict.as_str(),
                    entry.id,
                    entry.change
                );
            }
        }
        out
    }
}

// ---- refusals -----------------------------------------------------------------------------------

/// Why an audit ran no mutant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditRefusal {
    /// `ESS-MUTATE-001`: a baseline scenario failed or ended `error` against the target.
    BaselineFailed {
        /// The implementation.
        implementation: String,
        /// Every scenario that failed or ended `error`, in suite order.
        not_passed: Vec<String>,
    },
    /// `ESS-MUTATE-003`: the selected classes found no site.
    NoSite {
        /// The classes asked for.
        classes: Vec<MutantClass>,
    },
    /// The baseline executed no scenario — every one was `unsupported` or `skipped`, or the suite
    /// holds none — so no mutant can be scored on anything.
    NothingScored {
        /// The implementation.
        implementation: String,
        /// Every baseline scenario, with what the run reported for it.
        not_scored: Vec<NotScored>,
    },
    /// The unmutated specification does not compile.
    Unloadable(Stillborn),
    /// A suite was refused by admission.
    Admission(String),
    /// A mutation names a site the specification does not have.
    NoSuchSite(String),
    /// A collected audit cannot be scored at all: its manifest or its baseline report is missing
    /// or unreadable, the manifest names a directory outside the emission, or it was emitted for
    /// another component than the one asked for.
    Uncollectable(String),
    /// The specification declares no component of the name asked for.
    UnknownComponent(String),
}

impl AuditRefusal {
    /// The `MUTATE` code, where the refusal has one.
    pub fn code(&self) -> Option<Code> {
        match self {
            Self::BaselineFailed { .. } => Some(MutateCode::BaselineFailed.code()),
            Self::NoSite { .. } => Some(MutateCode::NoSite.code()),
            Self::NothingScored { .. }
            | Self::Unloadable(_)
            | Self::Admission(_)
            | Self::NoSuchSite(_)
            | Self::Uncollectable(_)
            | Self::UnknownComponent(_) => None,
        }
    }

    /// `true` when the audit ran and could conclude nothing — a refusal with a code, or a
    /// baseline that executed nothing — rather than failing to load or read its input.
    pub fn is_inconclusive(&self) -> bool {
        self.code().is_some() || matches!(self, Self::NothingScored { .. })
    }
}

impl fmt::Display for AuditRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaselineFailed {
                implementation,
                not_passed,
            } => {
                write!(
                    f,
                    "refusal[{}]: the unmutated suite did not pass against `{implementation}`, so \
                     no scenario failing under a mutant could be shown to fail because of it; {} \
                     scenario(s) failed or ended `error`:",
                    MutateCode::BaselineFailed.code(),
                    not_passed.len()
                )?;
                for id in not_passed {
                    write!(f, "\n  {id}")?;
                }
                Ok(())
            }
            Self::NoSite { classes } => write!(
                f,
                "refusal[{}]: {} find(s) no site in this specification, so the audit would run \
                 nothing",
                MutateCode::NoSite.code(),
                classes
                    .iter()
                    .map(|class| format!("`{class}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::NothingScored {
                implementation,
                not_scored,
            } => {
                write!(
                    f,
                    "nothing scored: the unmutated suite executed no scenario against \
                     `{implementation}`, so no mutant can be scored; {} scenario(s) not scored:",
                    not_scored.len()
                )?;
                for skipped in not_scored {
                    let status = match skipped.status {
                        NotScoredStatus::Unsupported => "unsupported",
                        NotScoredStatus::Skipped => "skipped",
                    };
                    write!(f, "\n  {} ({status})", skipped.scenario)?;
                }
                Ok(())
            }
            Self::Unloadable(stillborn) => write!(
                f,
                "the specification does not compile: {} {}",
                stillborn.cause, stillborn.message
            ),
            Self::Admission(message) => {
                write!(f, "the synthesized suite is not admitted: {message}")
            }
            Self::NoSuchSite(message) => f.write_str(message),
            Self::Uncollectable(message) => {
                write!(f, "the emitted audit cannot be collected: {message}")
            }
            Self::UnknownComponent(message) => {
                write!(f, "the audit cannot be scoped: {message}")
            }
        }
    }
}

impl std::error::Error for AuditRefusal {}

// ---- scoring ------------------------------------------------------------------------------------

/// What a run reported for a scenario that did not pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seen {
    Failed,
    Error,
    Unsupported,
    Skipped,
}

impl Seen {
    fn from_status(status: Status) -> Option<Self> {
        match status {
            Status::Passed => None,
            Status::Failed => Some(Self::Failed),
            Status::Error => Some(Self::Error),
            Status::Unsupported => Some(Self::Unsupported),
        }
    }

    /// A status as an `ess-conformance-report/1` lists it; an unknown one could not answer.
    fn listed(status: &str) -> Self {
        match status {
            "failed" => Self::Failed,
            "unsupported" => Self::Unsupported,
            "skipped" => Self::Skipped,
            _ => Self::Error,
        }
    }

    /// As a mutant's scenario status: a skip contradicted nothing, and nobody found out.
    fn status(self) -> Status {
        match self {
            Self::Failed => Status::Failed,
            Self::Error | Self::Skipped => Status::Error,
            Self::Unsupported => Status::Unsupported,
        }
    }
}

/// One suite's run: every scenario in suite order, and each that did not pass.
struct Observed {
    scenarios: Vec<String>,
    not_passed: Vec<(String, Seen)>,
    /// Each scenario as the suite holds it, by id.
    bodies: Bodies,
}

/// A suite's scenarios by id.
type Bodies = std::collections::BTreeMap<String, crate::scenario::ConformanceScenario>;

fn bodies_of(suite: &crate::ConformanceSuite) -> Bodies {
    suite
        .scenarios
        .iter()
        .map(|(id, scenario)| (id.to_string(), scenario.clone()))
        .collect()
}

/// The synthesis refusals of one suite: always the count, and the keys where they are known.
struct Refused {
    count: usize,
    keys: Option<Vec<RefusalKey>>,
}

/// The baseline, read as the ruler every mutant is measured against.
struct Ruler {
    /// The baseline scenarios the target reported `unsupported` or `skipped`. A mutant scenario
    /// with one of these ids is not scored; one new to the mutant's suite still is, because a
    /// scenario the mutant alone obliges is how a mutant that drops or adds a rule is killed.
    unexecuted: BTreeSet<String>,
    /// The baseline's own copy of each scenario in `unexecuted`.
    unexecuted_bodies: Bodies,
    not_scored: Vec<NotScored>,
    refused: Refused,
    /// Every outcome, as `<command>/<outcome>`, some baseline scenario requires a command to take.
    witnessed: BTreeSet<String>,
    /// The baseline model's transitions ([`performers`]); `None` where the model was not read.
    performers: Option<Performers>,
}

/// Every transition, as `<entity>.<transition>`, with the outcomes, as `(command, outcome)`, that
/// perform it.
type Performers = BTreeMap<String, Vec<(String, String)>>;

/// The transitions of the compiled model `model` holds in its compact JSON (`ir.json`), and the
/// outcomes that perform each; `None` where it is not such a model.
///
/// Read from the JSON rather than the model so that [`audit`] and [`collect`], which has only the
/// emitted `ir.json`, read one thing.
fn performers(model: &str) -> Option<Performers> {
    let model: serde_json::Value = serde_json::from_str(model).ok()?;
    let mut found = Performers::new();
    for (command, body) in model.get("commands")?.as_object()? {
        for outcome in body.get("outcomes")?.as_array()? {
            // A transition is performed by the outcome's own subject or by its set subject
            // (`instances:`, ess/16), which carries its effect the same way; `affects:` only sets.
            for key in ["subject", "instances"] {
                let Some(subject) = outcome.get(key).filter(|it| !it.is_null()) else {
                    continue;
                };
                if subject.get("effect")?.as_str()? != "moves" {
                    continue;
                }
                let entity = subject.get("entity")?.as_str()?;
                let transition = subject.get("transition")?.get("name")?.as_str()?;
                found
                    .entry(format!("{entity}.{transition}"))
                    .or_default()
                    .push((command.clone(), outcome.get("name")?.as_str()?.to_owned()));
            }
        }
    }
    Some(found)
}

impl Ruler {
    /// Refuses a baseline with a failed or `error` scenario, and one that executed nothing.
    fn new(
        implementation: &str,
        observed: &Observed,
        refused: Refused,
        performers: Option<Performers>,
    ) -> Result<Self, AuditRefusal> {
        let red: Vec<String> = observed
            .not_passed
            .iter()
            .filter(|(_, seen)| matches!(seen, Seen::Failed | Seen::Error))
            .map(|(id, _)| id.clone())
            .collect();
        if !red.is_empty() {
            return Err(AuditRefusal::BaselineFailed {
                implementation: implementation.to_owned(),
                not_passed: red,
            });
        }
        let not_scored: Vec<NotScored> = observed
            .not_passed
            .iter()
            .map(|(id, seen)| NotScored {
                scenario: id.clone(),
                status: if *seen == Seen::Unsupported {
                    NotScoredStatus::Unsupported
                } else {
                    NotScoredStatus::Skipped
                },
            })
            .collect();
        let unexecuted: BTreeSet<String> =
            not_scored.iter().map(|it| it.scenario.clone()).collect();
        if observed.scenarios.iter().all(|id| unexecuted.contains(id)) {
            return Err(AuditRefusal::NothingScored {
                implementation: implementation.to_owned(),
                not_scored,
            });
        }
        let unexecuted_bodies = observed
            .bodies
            .iter()
            .filter(|(id, _)| unexecuted.contains(*id))
            .map(|(id, scenario)| (id.clone(), scenario.clone()))
            .collect();
        let witnessed = observed
            .bodies
            .values()
            .flat_map(|scenario| &scenario.steps)
            .filter_map(|step| match step {
                crate::scenario::ScenarioStep::ExpectOutcome { outcome } => {
                    Some(outcome.to_string())
                }
                _ => None,
            })
            .collect();
        Ok(Self {
            unexecuted,
            unexecuted_bodies,
            not_scored,
            refused,
            witnessed,
            performers,
        })
    }

    /// The baseline's refusals of the scenarios of the outcomes mutant `id` of `class` changes,
    /// where no baseline scenario requires any of them and the baseline refused each: the outcome
    /// an outcome-class mutant is on, or every outcome that performs the transition a `from-drop`
    /// or `transition-to` mutant is on. `None` for any other mutant, where one of those outcomes
    /// is witnessed or not refused, where no outcome performs the transition, where the model's
    /// transitions were not read, and where the baseline's refusals are known only by count.
    fn unwitnessed_outcome(&self, class: MutantClass, id: &str) -> Option<Vec<RefusalKey>> {
        let outcomes: Vec<(&str, &str)> = match class {
            MutantClass::FromDrop | MutantClass::TransitionTo => {
                let transition = id
                    .strip_prefix(class.as_str())?
                    .strip_prefix('/')?
                    .split('/')
                    .next()?;
                self.performers
                    .as_ref()?
                    .get(transition)?
                    .iter()
                    .map(|(command, outcome)| (command.as_str(), outcome.as_str()))
                    .collect()
            }
            _ => vec![outcome_site(class, id)?],
        };
        let keys = self.refused.keys.as_ref()?;
        let mut refused: Vec<RefusalKey> = Vec::new();
        for (command, outcome) in &outcomes {
            if self.witnessed.contains(&format!("{command}/{outcome}")) {
                return None;
            }
            let scenario = format!("{command}/outcome/{outcome}");
            let before = refused.len();
            refused.extend(
                keys.iter()
                    .filter(|key| key.scenario.as_deref() == Some(scenario.as_str()))
                    .cloned(),
            );
            if refused.len() == before {
                return None;
            }
        }
        refused.sort();
        refused.dedup();
        (!refused.is_empty()).then_some(refused)
    }

    /// Scores one mutant's run into its entry: a scenario the baseline did not execute is excluded,
    /// and a refusal the baseline does not have makes a mutant nothing killed `unwitnessed`. An
    /// excluded scenario makes it `inconclusive` only where the mutant's copy differs from the
    /// baseline's: an unchanged one asks the target what the baseline asked, so it cannot kill it.
    /// A mutant on an outcome the baseline does not witness, or on a transition only such outcomes
    /// perform, is `unwitnessed` as a gained refusal makes it; one that left its guard `dead` is
    /// `equivalent` unless killed or hiding a changed scenario nothing scored.
    fn judge(
        &self,
        entry: &mut MutantEntry,
        observed: &Observed,
        refused: &Refused,
        dead: Option<String>,
    ) {
        let statuses: Vec<Status> = observed
            .not_passed
            .iter()
            .filter(|(id, _)| !self.unexecuted.contains(id))
            .map(|(_, seen)| seen.status())
            .collect();
        let mut excluded: Vec<String> = observed
            .scenarios
            .iter()
            .filter(|id| self.unexecuted.contains(*id))
            .cloned()
            .collect();
        excluded.sort();
        let (gained, added) = match (&self.refused.keys, &refused.keys) {
            (Some(baseline), Some(mutant)) => {
                // Per-key counts: a second refusal with a key the baseline holds once is gained.
                let mut held: std::collections::BTreeMap<&RefusalKey, usize> =
                    std::collections::BTreeMap::new();
                for key in baseline {
                    *held.entry(key).or_default() += 1;
                }
                let mut added: Vec<RefusalKey> = Vec::new();
                for key in mutant {
                    match held.get_mut(key) {
                        Some(count) if *count > 0 => *count -= 1,
                        _ => added.push(key.clone()),
                    }
                }
                added.sort();
                (!added.is_empty(), (!added.is_empty()).then_some(added))
            }
            _ => (refused.count > self.refused.count, None),
        };
        let hidden = excluded
            .iter()
            .any(|id| observed.bodies.get(id) != self.unexecuted_bodies.get(id));
        let at_baseline = self.unwitnessed_outcome(entry.class, &entry.id);
        // A mutant on an outcome the baseline does not witness is unwitnessed as a gained refusal
        // makes it; a dead guard outranks both, but not a changed scenario nothing scored, which
        // could have killed it (a gained refusal outranks `inconclusive` in `Verdict::judge`, so
        // `hidden` is asked here too).
        entry.verdict = Verdict::judge(&statuses, gained || at_baseline.is_some(), hidden);
        if dead.is_some() && !hidden {
            entry.verdict = entry.verdict.with_dead_guard();
        }
        entry.baseline_refusals = at_baseline;
        entry.unsatisfiable_guard = dead;
        if entry.verdict == Verdict::Killed {
            let mut killers: Vec<String> = observed
                .not_passed
                .iter()
                .filter(|(id, seen)| *seen == Seen::Failed && !self.unexecuted.contains(id))
                .map(|(id, _)| id.clone())
                .collect();
            killers.sort();
            entry.killers = Some(killers);
        }
        entry.added_refusals = added;
        entry.excluded = (!excluded.is_empty()).then_some(excluded);
        entry.refusals = Some(refused.count);
        entry.scenarios = Some(observed.scenarios.len());
    }

    fn size(&self, scenarios: usize) -> SuiteSize {
        SuiteSize {
            not_scored: self.not_scored.clone(),
            refusals: self.refused.count,
            scenarios,
        }
    }
}

// ---- running ------------------------------------------------------------------------------------

/// A suite synthesized from `ir` and run once on a fresh target.
struct Ran {
    observed: Observed,
    refused: Refused,
    implementation: String,
    spec_digest: String,
}

/// The write a `sets-drop` mutation removes, as synthesis of its model is told it
/// ([`with_dropped_write`](crate::synthesize::with_dropped_write)); `None` for any other mutation,
/// and where the entry does not read an input field.
fn dropped_write(
    documents: &[Document],
    mutation: &Mutation,
) -> Option<crate::synthesize::DroppedWrite> {
    let Mutation::SetsDrop {
        command,
        outcome,
        target,
    } = mutation
    else {
        return None;
    };
    let set = self::outcome(documents, command, outcome)?
        .sets
        .0
        .iter()
        .find(|set| set.target == *target)?;
    let PayloadSource::InputField { field } = &set.source else {
        return None;
    };
    Some(crate::synthesize::DroppedWrite {
        command: command.clone(),
        outcome: outcome.clone(),
        target: target.clone(),
        source: field.clone(),
    })
}

/// The suite `ir` obliges, run once on a fresh target; `dropped` is the write a `sets-drop`
/// mutant's model no longer makes.
fn run<T: ConformanceTarget>(
    ir: &EssIr,
    dropped: Option<crate::synthesize::DroppedWrite>,
    new_target: &impl Fn() -> T,
) -> Result<Ran, AuditRefusal> {
    let (suite, refused) = crate::synthesize::with_dropped_write(dropped, || synthesized(ir));
    let admitted = AdmittedSuite::from_suite(&suite)
        .map_err(|error| AuditRefusal::Admission(error.to_string()))?;
    let executed = Runner::for_suite(&suite).run_admitted(&admitted, &new_target());
    Ok(Ran {
        observed: Observed {
            scenarios: executed
                .scenarios
                .iter()
                .map(|result| result.scenario.to_string())
                .collect(),
            not_passed: executed
                .scenarios
                .iter()
                .filter_map(|result| {
                    Seen::from_status(result.status).map(|seen| (result.scenario.to_string(), seen))
                })
                .collect(),
            bodies: bodies_of(&suite),
        },
        refused: Refused {
            count: refused.len(),
            keys: Some(refused),
        },
        implementation: executed.implementation.name.clone(),
        spec_digest: suite.provenance.spec_digest.to_string(),
    })
}

fn blank_entry(mutant: &Mutant) -> MutantEntry {
    MutantEntry {
        added_refusals: None,
        baseline_refusals: None,
        change: mutant.change.clone(),
        class: mutant.class,
        excluded: None,
        id: mutant.id.clone(),
        killers: None,
        refusals: None,
        scenarios: None,
        stillborn: None,
        unsatisfiable_guard: None,
        unscored: None,
        verdict: Verdict::Stillborn,
    }
}

/// The baseline of `documents` run on a fresh target, as the ruler [`evaluate`] measures with.
struct BaselineRun {
    ruler: Ruler,
    implementation: String,
    spec_digest: String,
    scenarios: usize,
}

fn baseline<T: ConformanceTarget>(
    ir: &EssIr,
    new_target: &impl Fn() -> T,
) -> Result<BaselineRun, AuditRefusal> {
    let ran = run(ir, None, new_target)?;
    let scenarios = ran.observed.scenarios.len();
    Ok(BaselineRun {
        ruler: Ruler::new(
            &ran.implementation,
            &ran.observed,
            ran.refused,
            performers(&ir.to_compact_json()),
        )?,
        implementation: ran.implementation,
        spec_digest: ran.spec_digest,
        scenarios,
    })
}

/// Runs one mutant on a fresh target and says what it came to, measured against the baseline of
/// `documents` run on a fresh target of its own.
///
/// Refuses as [`audit`] refuses when that baseline cannot be scored against.
pub fn evaluate<T: ConformanceTarget>(
    documents: &[Document],
    texts: &SourceMap,
    mutant: &Mutant,
    new_target: impl Fn() -> T,
) -> Result<MutantEntry, AuditRefusal> {
    let baseline_ir = compile(documents.to_vec(), texts).map_err(AuditRefusal::Unloadable)?;
    let ruler = baseline(&baseline_ir, &new_target)?.ruler;
    measure(
        documents,
        texts,
        mutant,
        (&baseline_ir, &ruler),
        &new_target,
    )
}

fn measure<T: ConformanceTarget>(
    documents: &[Document],
    texts: &SourceMap,
    mutant: &Mutant,
    (baseline_ir, ruler): (&EssIr, &Ruler),
    new_target: &impl Fn() -> T,
) -> Result<MutantEntry, AuditRefusal> {
    let mutated = apply(documents, &mutant.mutation).map_err(AuditRefusal::NoSuchSite)?;
    let mut entry = blank_entry(mutant);
    let ir = match compile(mutated, texts) {
        Ok(ir) => ir,
        Err(stillborn) => {
            entry.stillborn = Some(stillborn);
            return Ok(entry);
        }
    };
    let ran = run(&ir, dropped_write(documents, &mutant.mutation), new_target)?;
    let dead = unsatisfiable_guard(baseline_ir, &ir, &mutant.mutation);
    ruler.judge(&mut entry, &ran.observed, &ran.refused, dead);
    Ok(entry)
}

/// The audit: a baseline with no failed or `error` scenario, then every mutant of `classes`, each
/// on a fresh target and scored on the scenarios the baseline executed.
pub fn audit<T: ConformanceTarget>(
    documents: &[Document],
    texts: &SourceMap,
    classes: &[MutantClass],
    new_target: impl Fn() -> T,
) -> Result<MutationReport, AuditRefusal> {
    let baseline_ir = compile(documents.to_vec(), texts).map_err(AuditRefusal::Unloadable)?;
    crate::admission::model(&baseline_ir)
        .map_err(|error| AuditRefusal::Admission(error.to_string()))?;
    let selected = mutants(documents, classes);
    if selected.is_empty() {
        let mut classes = classes.to_vec();
        classes.sort();
        classes.dedup();
        return Err(AuditRefusal::NoSite { classes });
    }
    let baseline = baseline(&baseline_ir, &new_target)?;

    let mut entries = Vec::with_capacity(selected.len());
    let mut counts = Counts {
        mutants: selected.len(),
        ..Counts::default()
    };
    for mutant in &selected {
        let entry = measure(
            documents,
            texts,
            mutant,
            (&baseline_ir, &baseline.ruler),
            &new_target,
        )?;
        counts.count(entry.verdict);
        entries.push(entry);
    }
    Ok(MutationReport {
        baseline: baseline.ruler.size(baseline.scenarios),
        component: None,
        counts,
        format: REPORT_FORMAT.to_owned(),
        implementation: baseline.implementation,
        mutants: entries,
        out_of_scope: None,
        spec_digest: baseline.spec_digest,
        specification: format!("{} {}", baseline_ir.system(), baseline_ir.version()),
    })
}

// ---- an external target: emit, then collect -----------------------------------------------------

/// The document family [`emit`] writes beside the suites where nothing in it needs
/// [`MANIFEST_FORMAT_4`], so a reader of `/3` still collects it.
pub const MANIFEST_FORMAT: &str = "ess-mutation-manifest/3";
/// The manifest [`emit_for`] writes where it names a component, or holds a mutant of a class only
/// this format's readers know ([`MutantClass::manifest_format`]): `/3` with the component, and each
/// mutant it leaves out marked `out_of_scope` (beyond10x/ess#212, beyond10x/ess#236).
pub const MANIFEST_FORMAT_4: &str = "ess-mutation-manifest/4";
/// The manifest 0.41.0 wrote, which [`collect`] still reads: it names each suite's refusals and no
/// mutant's `unsatisfiable_guard`, so no mutant it names is scored `equivalent`.
pub const MANIFEST_FORMAT_2: &str = "ess-mutation-manifest/2";
/// The earlier manifest [`collect`] still reads: it names each suite's refusal count and not the
/// refusals, so a mutant's gained refusals are judged by count alone.
pub const MANIFEST_FORMAT_1: &str = "ess-mutation-manifest/1";
/// The manifest's file name, at the top of the emitted directory.
pub const MANIFEST_FILE: &str = "manifest.json";
/// The directory the unmutated suite is written to.
pub const BASELINE_DIR: &str = "baseline";
/// A suite's file name inside its directory: the canonical suite document.
pub const SUITE_FILE: &str = "suite.json";
/// The compiled model beside a suite, in the compact bytes a generated package's `ir.json` holds.
pub const MODEL_FILE: &str = "ir.json";
/// A mutant's identity inside its directory: its [`EmittedMutant`] entry.
pub const MUTANT_FILE: &str = "mutant.json";
/// Where the project's runner writes the conformance report of the suite beside it.
pub const REPORT_FILE: &str = "report.json";

/// One emitted suite: where it is and what it holds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmittedSuite {
    /// Its directory, relative to the emission.
    pub dir: String,
    /// Synthesis refusals.
    pub refusals: usize,
    /// Each synthesis refusal, sorted; always in `ess-mutation-manifest/2` and `/3`, never in `/1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refused: Option<Vec<RefusalKey>>,
    /// Scenarios.
    pub scenarios: usize,
    /// The digest of the model the suite was synthesized from.
    pub spec_digest: String,
}

impl EmittedSuite {
    fn refused(&self) -> Refused {
        Refused {
            count: self.refusals,
            keys: self.refused.clone(),
        }
    }
}

/// One mutant of an emission. Fields are declared in key order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmittedMutant {
    /// What changed.
    pub change: String,
    /// Its class.
    pub class: MutantClass,
    /// Its suite's directory; absent on a stillborn mutant, which has no suite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    /// `<class>/<site>`.
    pub id: String,
    /// `true` where the emission is scoped to a component and the mutant's site belongs to another
    /// component: it then has no suite, and is listed rather than scored. Only in
    /// `ess-mutation-manifest/4`, and only beside its `component`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub out_of_scope: bool,
    /// Synthesis refusals of its suite; absent on a stillborn mutant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusals: Option<usize>,
    /// Each synthesis refusal of its suite, sorted; absent on a stillborn mutant and in
    /// `ess-mutation-manifest/1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refused: Option<Vec<RefusalKey>>,
    /// Scenarios of its suite; absent on a stillborn mutant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenarios: Option<usize>,
    /// The site: the id without its class.
    pub site: String,
    /// The digest of the mutated model; absent on a stillborn mutant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_digest: Option<String>,
    /// Why it has no suite; only on a stillborn mutant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stillborn: Option<Stillborn>,
    /// The guard it left its outcome, where no input satisfies it while some input satisfies the
    /// baseline's guard there; never on a stillborn mutant, and only in
    /// `ess-mutation-manifest/3`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsatisfiable_guard: Option<String>,
}

impl EmittedMutant {
    /// Its suite, or `None` for a stillborn or out-of-scope mutant; an entry that is none of these
    /// is refused.
    fn suite(&self) -> Result<Option<EmittedSuite>, String> {
        match (
            &self.dir,
            self.refusals,
            self.scenarios,
            &self.spec_digest,
            &self.stillborn,
        ) {
            (None, None, None, None, Some(_))
                if !self.out_of_scope
                    && self.refused.is_none()
                    && self.unsatisfiable_guard.is_none() =>
            {
                Ok(None)
            }
            (None, None, None, None, None)
                if self.out_of_scope
                    && self.refused.is_none()
                    && self.unsatisfiable_guard.is_none() =>
            {
                Ok(None)
            }
            (Some(dir), Some(refusals), Some(scenarios), Some(spec_digest), None)
                if !self.out_of_scope =>
            {
                Ok(Some(EmittedSuite {
                    dir: dir.clone(),
                    refusals,
                    refused: self.refused.clone(),
                    scenarios,
                    spec_digest: spec_digest.clone(),
                }))
            }
            _ => Err(format!(
                "`{}` is neither a stillborn mutant, an out-of-scope one nor one with a suite",
                self.id
            )),
        }
    }
}

/// The `ess-mutation-manifest/3` or `/4` document: what [`emit`] wrote, and what [`collect`]
/// scores.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The unmutated suite.
    pub baseline: EmittedSuite,
    /// The component every suite was scoped to, as `synthesize --component` scopes one; only in
    /// [`MANIFEST_FORMAT_4`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// [`MANIFEST_FORMAT_4`] or [`MANIFEST_FORMAT`], or [`MANIFEST_FORMAT_2`] or
    /// [`MANIFEST_FORMAT_1`] for a manifest an earlier release wrote.
    pub format: String,
    /// Every mutant, in byte order of id.
    pub mutants: Vec<EmittedMutant>,
    /// The original specification's digest.
    pub spec_digest: String,
    /// `<system> <version>`.
    pub specification: String,
}

impl Manifest {
    /// Two-space JSON with sorted keys and one trailing LF.
    pub fn to_canonical_json(&self) -> String {
        canonical(self)
    }

    /// Reads a manifest, refusing another format, an incoherent mutant entry, a suite's refusals
    /// that `/2` and later omit, `/1` carries or disagree with its count, an `unsatisfiable_guard`
    /// before `/3`, a `component`, an `out_of_scope` mutant or a class only `/4` knows before
    /// `/4`, an `out_of_scope` mutant without a `component`, or a directory that leaves the
    /// emission.
    pub fn from_json(text: &str) -> Result<Self, String> {
        // The format first, so a manifest of a version this reader does not know is refused by
        // that name rather than by whichever of its fields this reader does not know.
        let declared = serde_json::from_str::<serde_json::Value>(text)
            .ok()
            .and_then(|value| value.get("format")?.as_str().map(str::to_owned));
        // Whether suites name their refusals (`/2` on), whether a mutant may name a dead guard
        // (`/3` on), and whether the emission may be scoped to a component or hold a class only
        // `/4` knows.
        let version_of = |format: &str| match format {
            MANIFEST_FORMAT_4 => Some(4),
            MANIFEST_FORMAT => Some(3),
            MANIFEST_FORMAT_2 => Some(2),
            MANIFEST_FORMAT_1 => Some(1),
            _ => None,
        };
        if let Some(other) = declared.as_deref().filter(|it| version_of(it).is_none()) {
            return Err(format!(
                "{MANIFEST_FILE} is `{other}`, not `{MANIFEST_FORMAT_4}`, `{MANIFEST_FORMAT}`, \
                 `{MANIFEST_FORMAT_2}` or `{MANIFEST_FORMAT_1}`"
            ));
        }
        let manifest: Self =
            serde_json::from_str(text).map_err(|error| format!("{MANIFEST_FILE}: {error}"))?;
        let Some(version) = version_of(&manifest.format) else {
            return Err(format!(
                "{MANIFEST_FILE} is `{}`, not `{MANIFEST_FORMAT_4}`, `{MANIFEST_FORMAT}`, \
                 `{MANIFEST_FORMAT_2}` or `{MANIFEST_FORMAT_1}`",
                manifest.format
            ));
        };
        let (keyed, guarded, scoped) = (version >= 2, version >= 3, version >= 4);
        if !scoped {
            if manifest.component.is_some() {
                return Err(format!(
                    "{MANIFEST_FILE} names a `component`, which `{}` does not have; it is \
                     `{MANIFEST_FORMAT_4}`",
                    manifest.format
                ));
            }
            if let Some(mutant) = manifest.mutants.iter().find(|mutant| {
                version_of(mutant.class.manifest_format()).is_some_and(|needs| needs > version)
            }) {
                return Err(format!(
                    "{MANIFEST_FILE}: `{}` is a `{}` mutant, a class `{}` does not have; it is \
                     `{}`",
                    mutant.id,
                    mutant.class,
                    manifest.format,
                    mutant.class.manifest_format()
                ));
            }
        }
        if let Some(mutant) = manifest.mutants.iter().find(|mutant| mutant.out_of_scope) {
            if manifest.component.is_none() {
                return Err(format!(
                    "{MANIFEST_FILE}: `{}` is `out_of_scope`, which only an emission naming a \
                     `component` ({MANIFEST_FORMAT_4}) has",
                    mutant.id
                ));
            }
        }
        let refused = |suite: &EmittedSuite| -> Result<(), String> {
            match (&suite.refused, keyed) {
                (None, false) => Ok(()),
                (Some(keys), true) => {
                    if keys.len() != suite.refusals {
                        return Err(format!(
                            "{MANIFEST_FILE}: `{}` counts {} refusal(s) and lists {} as `refused`",
                            suite.dir,
                            suite.refusals,
                            keys.len()
                        ));
                    }
                    keys.iter().try_for_each(RefusalKey::check)
                }
                (None, true) => Err(format!(
                    "{MANIFEST_FILE}: `{}` has no `refused`, which `{}` requires",
                    suite.dir, manifest.format
                )),
                (Some(_), false) => Err(format!(
                    "{MANIFEST_FILE}: `{}` carries `refused`, which `{MANIFEST_FORMAT_1}` does not \
                     have",
                    suite.dir
                )),
            }
        };
        contained(&manifest.baseline.dir)?;
        refused(&manifest.baseline)?;
        for mutant in &manifest.mutants {
            if let Some(suite) = mutant.suite()? {
                contained(&suite.dir)?;
                refused(&suite)?;
            }
            if !guarded && mutant.unsatisfiable_guard.is_some() {
                return Err(format!(
                    "{MANIFEST_FILE}: `{}` carries `unsatisfiable_guard`, which `{}` does not have",
                    mutant.id, manifest.format
                ));
            }
        }
        Ok(manifest)
    }
}

/// What [`emit`] produced: the manifest, and every file to write, by path relative to the
/// emission's directory. The manifest is among the files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emission {
    /// The manifest.
    pub manifest: Manifest,
    /// Every file, by relative path.
    pub files: std::collections::BTreeMap<String, String>,
}

fn canonical(value: &impl serde::Serialize) -> String {
    let mut json = serde_json::to_string_pretty(value).expect("the document serializes");
    json.push('\n');
    json
}

/// A relative directory of plain segments, so a manifest cannot point a read outside the
/// emission.
fn contained(dir: &str) -> Result<(), String> {
    let plain = |segment: &str| {
        !segment.is_empty()
            && segment != "."
            && segment != ".."
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    };
    if dir.split('/').all(plain) {
        Ok(())
    } else {
        Err(format!(
            "`{dir}` is not a directory inside the emission: every segment must be a plain name"
        ))
    }
}

/// The suite `ir` obliges, synthesized as the built-in audit synthesizes it, and its refusals,
/// sorted.
///
/// Without its `…/grant/denied` scenarios (beyond10x/ess#265), for the reason no authored scenario
/// is run here: no class alters a grant, so each asks every mutant's suite exactly the question it
/// asks the baseline's and could never kill one. Leaving them in would move a suite that needs
/// nothing newer onto the suite major that carries them, which the audit's report/1 evidence does
/// not read.
fn synthesized(ir: &EssIr) -> (crate::ConformanceSuite, Vec<RefusalKey>) {
    synthesized_from(crate::synthesize(ir), ir)
}

/// [`synthesized`], scoped to `component` as `synthesize --component` scopes a suite
/// ([`synthesize_for`](crate::synthesize::synthesize_for)); the whole system's where it is `None`.
fn synthesized_for(
    ir: &EssIr,
    component: Option<&str>,
) -> Result<(crate::ConformanceSuite, Vec<RefusalKey>), AuditRefusal> {
    match component {
        None => Ok(synthesized(ir)),
        Some(component) => crate::synthesize::synthesize_for(ir, component)
            .map(|synthesis| synthesized_from(synthesis, ir))
            .map_err(|unknown| AuditRefusal::UnknownComponent(unknown.to_string())),
    }
}

fn synthesized_from(
    synthesis: crate::Synthesis,
    ir: &EssIr,
) -> (crate::ConformanceSuite, Vec<RefusalKey>) {
    let mut suite = synthesis.suite;
    suite
        .scenarios
        .retain(|id, _| !matches!(id, crate::ScenarioId::Grant { .. }));
    suite.select_fresh_format_for(ir);
    let mut refused: Vec<RefusalKey> = synthesis.refusals.iter().map(RefusalKey::of).collect();
    refused.sort();
    (suite, refused)
}

/// Writes `suite`, synthesized from `ir` with the synthesis refusals `refused`, and the model
/// beside it, into `dir`.
fn emit_suite(
    ir: &EssIr,
    (suite, refused): (crate::ConformanceSuite, Vec<RefusalKey>),
    dir: &str,
    files: &mut std::collections::BTreeMap<String, String>,
) -> Result<EmittedSuite, AuditRefusal> {
    contained(dir).map_err(AuditRefusal::NoSuchSite)?;
    let json = suite
        .to_canonical_json()
        .map_err(|error| AuditRefusal::Admission(error.to_string()))?;
    AdmittedSuite::from_json(&json).map_err(|error| AuditRefusal::Admission(error.to_string()))?;
    files.insert(format!("{dir}/{SUITE_FILE}"), json);
    files.insert(
        format!("{dir}/{MODEL_FILE}"),
        format!("{}\n", ir.to_compact_json()),
    );
    Ok(EmittedSuite {
        dir: dir.to_owned(),
        refusals: refused.len(),
        refused: Some(refused),
        scenarios: suite.len(),
        spec_digest: suite.provenance.spec_digest.to_string(),
    })
}

/// The audit's first half: the baseline suite and every mutant's suite, and nothing run.
///
/// Refuses as [`audit`] refuses before it runs anything: a specification that does not compile or
/// is not admitted, and `ESS-MUTATE-003` when the classes find no site. A stillborn mutant is
/// recorded in the manifest with its refusal, and has no suite.
pub fn emit(
    documents: &[Document],
    texts: &SourceMap,
    classes: &[MutantClass],
) -> Result<Emission, AuditRefusal> {
    emit_for(documents, texts, classes, None)
}

/// [`emit`], with every suite scoped to `component` where one is named, as `synthesize
/// --component` scopes it (beyond10x/ess#236).
///
/// A mutant whose site belongs to another component (`in_component`) is that component's to
/// answer, so it is marked `out_of_scope` with no suite, and listed rather than scored; a mutant on
/// the component's own site is scored, a survivor included. The manifest names the component and is `ess-mutation-manifest/4`, as is
/// one holding a mutant of a class only `/4` knows; any other is `/3`. Refuses a component the
/// specification does not declare, naming those it does.
pub fn emit_for(
    documents: &[Document],
    texts: &SourceMap,
    classes: &[MutantClass],
    component: Option<&str>,
) -> Result<Emission, AuditRefusal> {
    let baseline_ir = compile(documents.to_vec(), texts).map_err(AuditRefusal::Unloadable)?;
    crate::admission::model(&baseline_ir)
        .map_err(|error| AuditRefusal::Admission(error.to_string()))?;
    let baseline_suite = synthesized_for(&baseline_ir, component)?;
    let selected = mutants(documents, classes);
    if selected.is_empty() {
        let mut classes = classes.to_vec();
        classes.sort();
        classes.dedup();
        return Err(AuditRefusal::NoSite { classes });
    }
    let mut files = std::collections::BTreeMap::new();
    let baseline = emit_suite(&baseline_ir, baseline_suite, BASELINE_DIR, &mut files)?;
    let scope = component.and_then(|name| {
        baseline_ir
            .components()
            .values()
            .find(|declared| declared.name.as_str() == name)
    });
    let transitions = performers(&baseline_ir.to_compact_json()).unwrap_or_default();
    let mut entries = Vec::with_capacity(selected.len());
    for mutant in &selected {
        contained(&mutant.id).map_err(AuditRefusal::NoSuchSite)?;
        let mutated = apply(documents, &mutant.mutation).map_err(AuditRefusal::NoSuchSite)?;
        let out_of_scope = scope.is_some_and(|component| {
            !in_component(&baseline_ir, component, &transitions, &mutant.mutation)
        });
        let mut entry = EmittedMutant {
            change: mutant.change.clone(),
            class: mutant.class,
            dir: None,
            id: mutant.id.clone(),
            out_of_scope,
            refusals: None,
            refused: None,
            scenarios: None,
            site: mutant.mutation.site(),
            spec_digest: None,
            stillborn: None,
            unsatisfiable_guard: None,
        };
        if out_of_scope {
            entries.push(entry);
            continue;
        }
        match compile(mutated, texts) {
            Ok(ir) => {
                let synthesized = crate::synthesize::with_dropped_write(
                    dropped_write(documents, &mutant.mutation),
                    || synthesized_for(&ir, component),
                )?;
                let suite = emit_suite(&ir, synthesized, &mutant.id, &mut files)?;
                entry.dir = Some(suite.dir);
                entry.refusals = Some(suite.refusals);
                entry.refused = suite.refused;
                entry.scenarios = Some(suite.scenarios);
                entry.spec_digest = Some(suite.spec_digest);
                entry.unsatisfiable_guard =
                    unsatisfiable_guard(&baseline_ir, &ir, &mutant.mutation);
            }
            Err(stillborn) => entry.stillborn = Some(stillborn),
        }
        files.insert(format!("{}/{MUTANT_FILE}", mutant.id), canonical(&entry));
        entries.push(entry);
    }
    let format = if component.is_some()
        || entries
            .iter()
            .any(|entry| entry.class.manifest_format() == MANIFEST_FORMAT_4)
    {
        MANIFEST_FORMAT_4
    } else {
        MANIFEST_FORMAT
    };
    let manifest = Manifest {
        spec_digest: baseline.spec_digest.clone(),
        baseline,
        component: component.map(str::to_owned),
        format: format.to_owned(),
        mutants: entries,
        specification: format!("{} {}", baseline_ir.system(), baseline_ir.version()),
    };
    files.insert(MANIFEST_FILE.to_owned(), manifest.to_canonical_json());
    Ok(Emission { manifest, files })
}

/// Refuses `manifest` where `component` is named and the emission was scoped to another component,
/// or to none.
fn emitted_for(manifest: &Manifest, component: Option<&str>) -> Result<(), AuditRefusal> {
    let Some(asked) = component else {
        return Ok(());
    };
    match &manifest.component {
        Some(emitted) if emitted == asked => Ok(()),
        Some(emitted) => Err(AuditRefusal::Uncollectable(format!(
            "it was emitted for component `{emitted}`, not `{asked}`; collect it with \
             `--component {emitted}` or without `--component`"
        ))),
        None => Err(AuditRefusal::Uncollectable(format!(
            "it was emitted for the whole system, not for component `{asked}`; collect it \
             without `--component`"
        ))),
    }
}

/// Whether the site `mutation` edits belongs to `component`, by the membership `synthesize
/// --component` scopes a suite with (beyond10x/ess#236): an outcome's guard, `sets`, error or events,
/// and two outcomes' order, belong to the component that handles their command; a view's ranking
/// to the component that owns the view; a transition to a component that handles a command
/// performing it, as `performers` lists them from the baseline model.
fn in_component(
    ir: &EssIr,
    component: &ess_compiler::ir::ResolvedComponent,
    performers: &Performers,
    mutation: &Mutation,
) -> bool {
    let handles = |command: &str| {
        ess_domain::name::QualifiedName::new(command)
            .is_ok_and(|name| crate::synthesize::handles(ir, component, &name))
    };
    match mutation {
        Mutation::GuardBoundary { command, .. }
        | Mutation::GuardOutward { command, .. }
        | Mutation::GuardEquality { command, .. }
        | Mutation::SetsRetarget { command, .. }
        | Mutation::GuardNegate { command, .. }
        | Mutation::GuardConnective { command, .. }
        | Mutation::ErrorSwap { command, .. }
        | Mutation::EmitDrop { command, .. }
        | Mutation::SetsDrop { command, .. }
        | Mutation::PrecedenceSwap { command, .. } => handles(command),
        Mutation::OrderFlip { view, .. } => ess_domain::name::QualifiedName::new(view)
            .is_ok_and(|name| crate::synthesize::owns_view(ir, component, &name)),
        Mutation::FromDrop {
            entity, transition, ..
        }
        | Mutation::TransitionTo {
            entity, transition, ..
        } => performers
            .get(&format!("{entity}.{transition}"))
            .is_some_and(|by| by.iter().any(|(command, _)| handles(command))),
    }
}

/// One report, read against the suite it claims to be a run of.
struct Scored {
    implementation: String,
    observed: Observed,
}

/// Reads the report beside `suite`, or says why it cannot be scored.
fn score(read: &impl Fn(&str) -> Option<String>, suite: &EmittedSuite) -> Result<Scored, String> {
    let dir = &suite.dir;
    let suite_path = format!("{dir}/{SUITE_FILE}");
    let text = read(&suite_path).ok_or_else(|| format!("no suite at {suite_path}"))?;
    let admitted =
        AdmittedSuite::from_json(&text).map_err(|error| format!("{suite_path}: {error}"))?;
    let provenance = &admitted.suite().provenance;
    if provenance.spec_digest.to_string() != suite.spec_digest
        || admitted.suite().len() != suite.scenarios
    {
        return Err(format!(
            "{suite_path} is not the suite the manifest emitted"
        ));
    }
    let bodies = bodies_of(admitted.suite());
    let order: Vec<String> = admitted
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();

    let report_path = format!("{dir}/{REPORT_FILE}");
    let text = read(&report_path).ok_or_else(|| format!("no report at {report_path}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("{report_path}: {error}"))?;
    let format = value
        .get("format")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let mut seen: std::collections::BTreeMap<String, Seen> = std::collections::BTreeMap::new();
    let implementation = match format {
        crate::evidence::STANDALONE_REPORT_FORMAT => {
            let report = crate::evidence::StandaloneConformanceReport::from_json(&text)
                .map_err(|error| format!("{report_path}: {error}"))?;
            if report.spec_digest != provenance.spec_digest
                || report.suite_version != provenance.suite_version.to_string()
                || report.scenarios_total != order.len()
                || report.specification
                    != format!("{}/{}", provenance.system, provenance.specification_version)
            {
                return Err(format!("{report_path} is a report of another suite"));
            }
            for entry in &report.failed_scenarios {
                let (status, id) = entry.split_once(' ').unwrap_or_default();
                if seen.insert(id.to_owned(), Seen::listed(status)).is_some() {
                    return Err(format!("{report_path} lists `{id}` twice"));
                }
            }
            report.implementation
        }
        crate::counts::COUNT_REPORT_FORMAT => {
            crate::CountReport::from_json(&text, &admitted)
                .map_err(|error| format!("{report_path}: {error}"))?;
            for (category, status) in [
                ("failed", Seen::Failed),
                ("unsupported", Seen::Unsupported),
                ("error", Seen::Error),
                ("skipped", Seen::Skipped),
            ] {
                let ids = value["outcomes"][category]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(serde_json::Value::as_str);
                for id in ids {
                    seen.insert(id.to_owned(), status);
                }
            }
            value["implementation"]
                .as_str()
                .unwrap_or_default()
                .to_owned()
        }
        other => {
            return Err(format!(
                "{report_path} is `{other}`, not `{}` or `{}`",
                crate::evidence::STANDALONE_REPORT_FORMAT,
                crate::counts::COUNT_REPORT_FORMAT
            ))
        }
    };
    if let Some(id) = seen.keys().find(|id| !order.contains(id)) {
        return Err(format!(
            "{report_path} names `{id}`, which {suite_path} does not hold"
        ));
    }
    Ok(Scored {
        implementation,
        observed: Observed {
            not_passed: order
                .iter()
                .filter_map(|id| seen.get(id).map(|status| (id.clone(), *status)))
                .collect(),
            scenarios: order,
            bodies,
        },
    })
}

/// The audit's second half: every report the project's runner wrote beside an emitted suite,
/// scored into the `ess-mutation-report/3` [`audit`] writes.
///
/// `read` answers a path relative to the emission with that file's text, or `None` when it is
/// not there. A baseline report with a failed or `error` scenario is refused with
/// `ESS-MUTATE-001`, as [`audit`] refuses; its `unsupported` and `skipped` scenarios are listed as
/// not scored, and each mutant is scored on the scenarios the baseline executed. A mutant whose
/// report is missing, unreadable, of another suite, or answered by another implementation than
/// the baseline's is `inconclusive`, with the reason in `unscored`: nothing contradicted it, and
/// nobody found out.
pub fn collect(read: impl Fn(&str) -> Option<String>) -> Result<MutationReport, AuditRefusal> {
    collect_for(read, None)
}

/// [`collect`], refusing an emission scoped to another component than `component`, or to none,
/// where one is named; `None` collects the emission for whatever component its manifest names.
///
/// An emission scoped to a component is scored into an `ess-mutation-report/4` naming it, with each
/// mutant marked `out_of_scope` listed and not scored or counted.
pub fn collect_for(
    read: impl Fn(&str) -> Option<String>,
    component: Option<&str>,
) -> Result<MutationReport, AuditRefusal> {
    let text = read(MANIFEST_FILE)
        .ok_or_else(|| AuditRefusal::Uncollectable(format!("no {MANIFEST_FILE}")))?;
    let manifest = Manifest::from_json(&text).map_err(AuditRefusal::Uncollectable)?;
    emitted_for(&manifest, component)?;
    let baseline = score(&read, &manifest.baseline)
        .map_err(|why| AuditRefusal::Uncollectable(format!("the baseline: {why}")))?;
    // The emitted baseline model says which outcomes perform each transition; where it is gone, a
    // transition mutant is scored without that, as before.
    let performers = read(&format!("{}/{MODEL_FILE}", manifest.baseline.dir))
        .as_deref()
        .and_then(performers);
    let ruler = Ruler::new(
        &baseline.implementation,
        &baseline.observed,
        manifest.baseline.refused(),
        performers,
    )?;

    let mut entries = Vec::with_capacity(manifest.mutants.len());
    let mut out_of_scope = Vec::new();
    let mut counts = Counts {
        mutants: manifest
            .mutants
            .iter()
            .filter(|mutant| !mutant.out_of_scope)
            .count(),
        ..Counts::default()
    };
    for mutant in &manifest.mutants {
        if mutant.out_of_scope {
            out_of_scope.push(OutOfScope {
                change: mutant.change.clone(),
                class: mutant.class,
                id: mutant.id.clone(),
            });
            continue;
        }
        let mut entry = MutantEntry {
            added_refusals: None,
            baseline_refusals: None,
            change: mutant.change.clone(),
            class: mutant.class,
            excluded: None,
            id: mutant.id.clone(),
            killers: None,
            refusals: None,
            scenarios: None,
            stillborn: mutant.stillborn.clone(),
            // Named whatever the report scores it; only a scored run can make it `equivalent`.
            unsatisfiable_guard: mutant.unsatisfiable_guard.clone(),
            unscored: None,
            verdict: Verdict::Stillborn,
        };
        if let Some(suite) = mutant.suite().map_err(AuditRefusal::Uncollectable)? {
            match score(&read, &suite) {
                Ok(scored) if scored.implementation != baseline.implementation => {
                    entry.refusals = Some(suite.refusals);
                    entry.scenarios = Some(suite.scenarios);
                    entry.verdict = Verdict::Inconclusive;
                    entry.unscored = Some(format!(
                        "{}/{REPORT_FILE} was answered by `{}`, not the baseline's `{}`",
                        suite.dir, scored.implementation, baseline.implementation
                    ));
                }
                Ok(scored) => ruler.judge(
                    &mut entry,
                    &scored.observed,
                    &suite.refused(),
                    mutant.unsatisfiable_guard.clone(),
                ),
                Err(why) => {
                    entry.refusals = Some(suite.refusals);
                    entry.scenarios = Some(suite.scenarios);
                    entry.verdict = Verdict::Inconclusive;
                    entry.unscored = Some(why);
                }
            }
        }
        counts.count(entry.verdict);
        entries.push(entry);
    }
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    out_of_scope.sort_by(|left, right| left.id.cmp(&right.id));
    let scoped = manifest.component.is_some();
    Ok(MutationReport {
        baseline: ruler.size(manifest.baseline.scenarios),
        component: manifest.component,
        counts,
        format: if scoped {
            REPORT_FORMAT_4
        } else {
            REPORT_FORMAT
        }
        .to_owned(),
        implementation: baseline.implementation,
        mutants: entries,
        out_of_scope: scoped.then_some(out_of_scope),
        spec_digest: manifest.spec_digest,
        specification: manifest.specification,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every code this module can emit is named on the formats page, so none ships unnamed.
    #[test]
    fn every_mutate_code_is_named_in_the_formats_reference() {
        let page = include_str!("../../../../website/docs/reference/formats.md");
        for code in MutateCode::ALL {
            let code = code.code().to_string();
            assert!(page.contains(&code), "`{code}` is not named in formats.md");
        }
        assert!(page.contains(REPORT_FORMAT));
    }
}
