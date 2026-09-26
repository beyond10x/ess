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
//! No class *weakens* the specification. Dropping a `sets` entry says the value is the
//! implementation's to choose, and a correct target satisfies a weaker specification by
//! definition, so such a mutant could never be killed and would be noise shaped like signal. Each
//! of the issue's classes is replaced by the altering version of the same fault.
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

use std::collections::BTreeSet;
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
pub const REPORT_FORMAT: &str = "ess-mutation-report/1";

// ---- the classes --------------------------------------------------------------------------------

/// The nine altering classes, a closed set with an [`ALL`](Self::ALL) constant as `Fault` has.
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
    /// Swap the strictness of an ordering comparison in an outcome's `when`: `>=`↔`>`, `<=`↔`<`.
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
    ];

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
    /// The baseline suite did not pass against the target, so no failure under a mutant can be
    /// shown to be *because of* the mutant.
    BaselineFailed,
    /// A mutant was refused by `assemble` or `compile`.
    Stillborn,
    /// The selected classes found no site: an audit that ran nothing and exited 0 would be a green
    /// exit that checked nothing.
    NoSite,
}

impl MutateCode {
    /// Every code.
    pub const ALL: &'static [Self] = &[Self::BaselineFailed, Self::Stillborn, Self::NoSite];

    /// Its stable code, derived from the variant as `RefusalCause::code` derives its own.
    pub fn code(self) -> Code {
        Code::new(
            FAMILY,
            match self {
                Self::BaselineFailed => 1,
                Self::Stillborn => 2,
                Self::NoSite => 3,
            },
        )
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
}

impl Mutation {
    /// Its class.
    pub fn class(&self) -> MutantClass {
        match self {
            Self::FromDrop { .. } => MutantClass::FromDrop,
            Self::TransitionTo { .. } => MutantClass::TransitionTo,
            Self::GuardBoundary { .. } => MutantClass::GuardBoundary,
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
            Self::SetsRetarget {
                command,
                outcome,
                target,
                ..
            } => format!("{command}/{outcome}/{target}"),
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
            for leaf in 0..ordering_leaves(when).len() {
                found.push(Mutation::GuardBoundary {
                    command: command_name.clone(),
                    outcome: outcome_name.clone(),
                    leaf,
                });
            }
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
        | Mutation::GuardNegate { .. }
        | Mutation::GuardConnective { .. } => return describe_guard(documents, mutation),
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
        | Mutation::GuardNegate { .. }
        | Mutation::GuardConnective { .. } => apply_guard(&mut mutated, mutation)?,
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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

// ---- verdicts -----------------------------------------------------------------------------------

/// What one mutant came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// At least one scenario failed.
    Killed,
    /// Every scenario passed.
    Survived,
    /// No scenario failed, and at least one ended `error` or `unsupported`: nothing contradicted
    /// the mutant, and nobody found out.
    Inconclusive,
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

    /// How it is written.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Killed => "killed",
            Self::Survived => "survived",
            Self::Inconclusive => "inconclusive",
            Self::Stillborn => "stillborn",
        }
    }
}

// ---- the report ---------------------------------------------------------------------------------

/// One mutant's entry in the report. Fields are declared in key order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MutantEntry {
    /// What changed.
    pub change: String,
    /// Its class.
    pub class: MutantClass,
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
    /// What it came to.
    pub verdict: Verdict,
}

/// The baseline suite's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct SuiteSize {
    /// Synthesis refusals.
    pub refusals: usize,
    /// Scenarios.
    pub scenarios: usize,
}

/// How many mutants came to each verdict.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Counts {
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
}

/// The `ess-mutation-report/1` document: keys sorted, no timestamp, so its bytes are a function of
/// the tree and the target.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MutationReport {
    /// The unmutated suite.
    pub baseline: SuiteSize,
    /// How many mutants came to each verdict.
    pub counts: Counts,
    /// [`REPORT_FORMAT`].
    pub format: String,
    /// The implementation that answered.
    pub implementation: String,
    /// Every mutant, in byte order of id.
    pub mutants: Vec<MutantEntry>,
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

    /// One summary line, then survivors, inconclusive, stillborn and killed, one line each.
    pub fn render_text(&self) -> String {
        let counts = &self.counts;
        let mut out = format!(
            "mutation audit of {} against {}: {} mutant(s), {} killed, {} survived, {} \
             inconclusive, {} stillborn (baseline: {} scenario(s), {} refusal(s))\n",
            self.specification,
            self.implementation,
            counts.mutants,
            counts.killed,
            counts.survived,
            counts.inconclusive,
            counts.stillborn,
            self.baseline.scenarios,
            self.baseline.refusals,
        );
        for verdict in [
            Verdict::Survived,
            Verdict::Inconclusive,
            Verdict::Stillborn,
            Verdict::Killed,
        ] {
            for entry in self.mutants.iter().filter(|entry| entry.verdict == verdict) {
                let tail = match (&entry.killers, &entry.stillborn) {
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
    /// `ESS-MUTATE-001`: the baseline suite did not pass against the target.
    BaselineFailed {
        /// The implementation.
        implementation: String,
        /// Every scenario that did not pass, in suite order.
        not_passed: Vec<String>,
    },
    /// `ESS-MUTATE-003`: the selected classes found no site.
    NoSite {
        /// The classes asked for.
        classes: Vec<MutantClass>,
    },
    /// The unmutated specification does not compile.
    Unloadable(Stillborn),
    /// A suite was refused by admission.
    Admission(String),
    /// A mutation names a site the specification does not have.
    NoSuchSite(String),
}

impl AuditRefusal {
    /// The `MUTATE` code, where the refusal has one.
    pub fn code(&self) -> Option<Code> {
        match self {
            Self::BaselineFailed { .. } => Some(MutateCode::BaselineFailed.code()),
            Self::NoSite { .. } => Some(MutateCode::NoSite.code()),
            Self::Unloadable(_) | Self::Admission(_) | Self::NoSuchSite(_) => None,
        }
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
                     scenario(s) did not pass:",
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
            Self::Unloadable(stillborn) => write!(
                f,
                "the specification does not compile: {} {}",
                stillborn.cause, stillborn.message
            ),
            Self::Admission(message) => {
                write!(f, "the synthesized suite is not admitted: {message}")
            }
            Self::NoSuchSite(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for AuditRefusal {}

// ---- running ------------------------------------------------------------------------------------

/// A suite synthesized from `ir` and run once on a fresh target.
struct Ran {
    statuses: Vec<(String, Status)>,
    scenarios: usize,
    refusals: usize,
    implementation: String,
    spec_digest: String,
}

fn run<T: ConformanceTarget>(ir: &EssIr, new_target: &impl Fn() -> T) -> Result<Ran, AuditRefusal> {
    let synthesis = crate::synthesize(ir);
    let mut suite = synthesis.suite;
    suite.select_fresh_format();
    let admitted = AdmittedSuite::from_suite(&suite)
        .map_err(|error| AuditRefusal::Admission(error.to_string()))?;
    let executed = Runner::for_suite(&suite).run_admitted(&admitted, &new_target());
    Ok(Ran {
        statuses: executed
            .scenarios
            .iter()
            .map(|result| (result.scenario.to_string(), result.status))
            .collect(),
        scenarios: suite.len(),
        refusals: synthesis.refusals.len(),
        implementation: executed.implementation.name.clone(),
        spec_digest: suite.provenance.spec_digest.to_string(),
    })
}

/// Runs one mutant on a fresh target and says what it came to.
pub fn evaluate<T: ConformanceTarget>(
    documents: &[Document],
    texts: &SourceMap,
    mutant: &Mutant,
    new_target: impl Fn() -> T,
) -> Result<MutantEntry, AuditRefusal> {
    let mutated = apply(documents, &mutant.mutation).map_err(AuditRefusal::NoSuchSite)?;
    let entry = |verdict| MutantEntry {
        change: mutant.change.clone(),
        class: mutant.class,
        id: mutant.id.clone(),
        killers: None,
        refusals: None,
        scenarios: None,
        stillborn: None,
        verdict,
    };
    let ir = match compile(mutated, texts) {
        Ok(ir) => ir,
        Err(stillborn) => {
            return Ok(MutantEntry {
                stillborn: Some(stillborn),
                ..entry(Verdict::Stillborn)
            })
        }
    };
    let ran = run(&ir, &new_target)?;
    let statuses: Vec<Status> = ran.statuses.iter().map(|(_, status)| *status).collect();
    let verdict = Verdict::classify(&statuses);
    let killers = (verdict == Verdict::Killed).then(|| {
        let mut killers: Vec<String> = ran
            .statuses
            .iter()
            .filter(|(_, status)| *status == Status::Failed)
            .map(|(id, _)| id.clone())
            .collect();
        killers.sort();
        killers
    });
    Ok(MutantEntry {
        killers,
        refusals: Some(ran.refusals),
        scenarios: Some(ran.scenarios),
        ..entry(verdict)
    })
}

/// The audit: a green baseline, then every mutant of `classes`, each on a fresh target.
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
    let baseline = run(&baseline_ir, &new_target)?;
    let not_passed: Vec<String> = baseline
        .statuses
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.clone())
        .collect();
    if !not_passed.is_empty() {
        return Err(AuditRefusal::BaselineFailed {
            implementation: baseline.implementation,
            not_passed,
        });
    }

    let mut entries = Vec::with_capacity(selected.len());
    let mut counts = Counts {
        mutants: selected.len(),
        ..Counts::default()
    };
    for mutant in &selected {
        let entry = evaluate(documents, texts, mutant, &new_target)?;
        match entry.verdict {
            Verdict::Killed => counts.killed += 1,
            Verdict::Survived => counts.survived += 1,
            Verdict::Inconclusive => counts.inconclusive += 1,
            Verdict::Stillborn => counts.stillborn += 1,
        }
        entries.push(entry);
    }
    Ok(MutationReport {
        baseline: SuiteSize {
            refusals: baseline.refusals,
            scenarios: baseline.scenarios,
        },
        counts,
        format: REPORT_FORMAT.to_owned(),
        implementation: baseline.implementation,
        mutants: entries,
        spec_digest: baseline.spec_digest,
        specification: format!("{} {}", baseline_ir.system(), baseline_ir.version()),
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
