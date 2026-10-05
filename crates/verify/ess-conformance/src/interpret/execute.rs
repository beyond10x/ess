//! One command, executed from the IR: state in, command in, every outcome the model allows out.
//!
//! This is the library half of the interpreter, and it is kept free of a runner on purpose. The
//! [`Interpreted`](super::Interpreted) target adds the invocation's authenticated caller and
//! executes once per `ExecuteCommand` step, then
//! refuses when the answer is not exactly one step; a linearizability checker calls the same
//! function as its sequential model and explores every step it returns.
//!
//! # What is derived, and from where
//!
//! | fact | read from |
//! |---|---|
//! | which outcome the input selects | the precedence order: a missing related row's `exists: false` branch, the first declared of several rows read through the input (ess/22, `related_absent`; an absent Optional reference, ess/22, reads no row and selects no related branch), then the first declared input-guarded refusal whose `when:` holds (`refused_by_input`); addressed-row existence and held state; a related row named by a stored field of the addressed subject (ess/22, `stored_reference`): absent, none; missing, its `exists: false` branch; for the ess/22 `wrong_state` composition, several related rows and every stored reference, the first declared present-related predicate refusal whose predicate holds; then the first accepting or external branch declared whose guard holds, over [`input::flatten`], then the one `Otherwise` branch |
//! | whether an external branch is taken | [`Externals`] — never the input, never this module |
//! | whether the subject may move | the transition's own `from` set against the state held in the [`Store`] |
//! | what a refused move answers | the command's `wrong_state:` branch; for an identity nobody holds, its `unknown_instance:` branch, else its one declared not-found refusal, else `wrong_state:` |
//! | what a created instance holds | `creates:` lands at `into:` or the lifecycle's `initial`; `sets:` writes `input.<field>` or a typed literal |
//! | what each emitted event carries | `payload:` for a determined field, the new identity for the field `instance:` names, a minted value for an undetermined one |
//! | whether the instance may rest there | the entity's `invariants:`, evaluated over what it now holds |
//!
//! # Nothing is chosen here
//!
//! Where the model leaves more than one outcome open — two input-guarded refusals that both hold, or
//! an external branch declared before the one the input selects, under [`Externals::Open`] — every one
//! of them is returned. The branches are read in the declared precedence and stop at the first that
//! answers (`docs/design/input-guard-overlap-precedence.md`), so a guard after it is never read.
//! Where it leaves none, the single step carries no outcome, which is the target's
//! [`SemanticCommandResult::undeclared`](crate::target::SemanticCommandResult::undeclared): a
//! refusal the model does not declare is not available, so it is never answered as a declared one.
//!
//! Where the model uses a construct this module does not execute yet — a retained replay, a value
//! expression reading a response — the answer is [`Undetermined::NotInterpreted`], never a
//! guess. A guard that evaluates to `Unknown` is [`Undetermined::Undecidable`] for the same reason.
//!
//! # Minted values
//!
//! A value the model says the implementation assigns — a created instance's identity, a field
//! declared `{generated: true}`, a legacy payload field nothing determines — comes from
//! [`Generated`]. A value somebody can observe (an emitted event's field, which is where a created
//! identity is published) is taken from the caller's [`Generated::Given`] slots where it has them —
//! how a checker replays what a history recorded, under the rules that variant states — and a
//! `sets:` write nobody observes always draws from a counter the [`Store`] carries. The counter is
//! part of the state, so a step is a function: the same store, command and values give the same
//! steps. A created identity follows its declared payload source; only generated sources use
//! this supply. The event field `instance:` names publishes that same value, and an identity
//! the store already holds is never replaced by creation.

pub(super) mod caller;
mod existence;
pub(crate) mod history;
mod related;
mod set_effects;
mod subject;
mod values;
use caller::Invocation;
use history::{Context, Row, State, Transition, Value};

use std::collections::BTreeMap;
use std::fmt;

use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedInstance,
    ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue, ResolvedRelatedTest,
    ResolvedRelatedVia, ResolvedSubject, ResolvedTypeRef,
};
use ess_domain::command::OutcomeName;
use ess_domain::entity::StateName;
use ess_domain::name::QualifiedName;
use ess_domain::types::Primitive;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

use crate::input::{self, Completeness, TypedFacts};
use crate::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use crate::target::{DeclaredErrorValue, ObservedEvent};

// ---- the state a step reads and writes -----------------------------------------------------------

/// Every instance the model holds, and the counter every minted value comes from.
///
/// A value, not a handle: [`execute`] never mutates the store it is given, and every [`Step`] carries
/// the store after it. That is what lets a search keep two branches of one history side by side.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Store {
    instances: BTreeMap<QualifiedName, BTreeMap<Node, Instance>>,
    minted: u64,
}

impl Store {
    /// Establish already-validated upstream state without replacing an existing identity.
    pub(super) fn establish(
        &mut self,
        entity: QualifiedName,
        identity: Node,
        instance: Instance,
    ) -> bool {
        if let std::collections::btree_map::Entry::Vacant(slot) =
            self.instances.entry(entity).or_default().entry(identity)
        {
            slot.insert(instance);
            true
        } else {
            false
        }
    }

    /// Convenience lookup for a text identity. Other identity types use [`Self::instance_typed`].
    pub fn instance(&self, entity: &QualifiedName, identity: &str) -> Option<&Instance> {
        self.instance_typed(entity, &Node::Text(identity.into()))
    }

    /// The instance of `entity` with this exact typed identity, when one is held.
    pub fn instance_typed(&self, entity: &QualifiedName, identity: &Node) -> Option<&Instance> {
        self.instances.get(entity)?.get(identity)
    }

    /// Every held instance, by entity and then by identity.
    pub fn instances(&self) -> impl Iterator<Item = (&QualifiedName, &Node, &Instance)> {
        self.instances.iter().flat_map(|(entity, held)| {
            held.iter()
                .map(move |(identity, instance)| (entity, identity, instance))
        })
    }

    /// Only rows whose identities are text; use [`Self::instances`] to enumerate every row.
    pub fn text_instances(&self) -> impl Iterator<Item = (&QualifiedName, &str, &Instance)> {
        self.instances().filter_map(|(entity, identity, instance)| {
            identity.as_text().map(|text| (entity, text, instance))
        })
    }
}

/// One held instance: the state its lifecycle rests in, and the fields some outcome has written.
///
/// A field no outcome has written is absent, not defaulted. The model left it to the
/// implementation, and an interpreter that filled it would be claiming a value nobody declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    /// Where its lifecycle rests.
    pub state: StateName,
    /// What outcomes have written, by declared field name.
    pub fields: BTreeMap<String, Node>,
}

/// Who decides a branch declared `external:`.
///
/// Never this module. The three answers are the three a caller can have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Externals {
    /// Nothing outside the input has decided anything, so no external branch is taken. What a
    /// scenario that configured no outcome means.
    Withheld,
    /// A scenario forced this branch of the command, and it is taken where its input eligibility
    /// holds.
    Forced(OutcomeName),
    /// Anything outside may have happened: every eligible external branch is one more allowed
    /// outcome. What a checker searching recorded histories asks for.
    Open,
}

/// Where the observable values the model leaves to the implementation come from.
///
/// The model says an identity or a `{generated: true}` field is the implementation's to assign, and
/// nothing more. A target minting its own answers [`Counter`](Self::Counter); a checker replaying a
/// recorded history answers [`Given`](Self::Given) with the values the implementation published, so
/// the next command naming one of them names an instance the step holds.
///
/// Only a value somebody can observe is ever given: an emitted event's field, which is also where a
/// created instance's identity is published (`instance:` names that field). A `sets:` write the
/// model declares `{generated: true}` is not observable, and always draws from the counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Generated {
    /// Minted from the counter the [`Store`] carries, in the shape the declared type takes. An
    /// optional field stays absent.
    Counter,
    /// The published value of each slot. Per branch, for each slot the branch assigns:
    ///
    /// * a slot present is assigned its value, which must be a value of the field's declared type;
    /// * a required slot absent, or a value not of the declared type, means the branch yields no
    ///   step — it is not what the history recorded;
    /// * an optional slot absent stays absent;
    /// * a slot no branch assigns is ignored.
    ///
    /// A creating branch whose given identity the store already holds yields no step: `creates:`
    /// brings a new instance into existence and never replaces one. Where no branch yields a step,
    /// the answer is [`Undetermined::Request`].
    Given(BTreeMap<GeneratedSlot, Node>),
    /// What a recorded history knows: the published value of some slots and not of the rest.
    ///
    /// A slot present is used exactly as [`Given`](Self::Given) uses it — type-checked, and a
    /// creating branch whose identity the store already holds yields no step. A slot absent is
    /// minted from the counter, as under [`Counter`](Self::Counter), rather than making the branch
    /// yield no step: `ess-history/1` records no payload values, so its absence says nothing.
    Recorded(BTreeMap<GeneratedSlot, Node>),
}

/// One observable value the implementation assigns: `field` of the emitted `event`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedSlot {
    /// The emitted event.
    pub event: QualifiedName,
    /// Its field.
    pub field: String,
}

impl GeneratedSlot {
    /// `field` of `event`.
    pub fn new(event: QualifiedName, field: impl Into<String>) -> Self {
        Self {
            event,
            field: field.into(),
        }
    }
}

/// One branch's reading of [`Generated`].
struct Supply<'g> {
    given: Option<&'g BTreeMap<GeneratedSlot, Node>>,
    /// Whether a slot not given is minted rather than refused ([`Generated::Recorded`]).
    mint_absent: bool,
}

impl<'g> Supply<'g> {
    fn of(generated: &'g Generated) -> Self {
        Self {
            given: match generated {
                Generated::Counter => None,
                Generated::Given(values) | Generated::Recorded(values) => Some(values),
            },
            mint_absent: matches!(generated, Generated::Recorded(_)),
        }
    }
}

/// A branch the given values do not describe: it yields no step, and this says why.
type Unmatched = String;

/// Unwraps a branch-level answer, returning early with the branch's own "no step" reason.
macro_rules! or_no_step {
    ($answer:expr) => {
        match $answer? {
            Ok(value) => value,
            Err(why) => return Ok(Err(why)),
        }
    };
}

/// One outcome the model allows, and everything it determines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The declared branch taken. `None` where no declared branch covers the request, which is a
    /// refusal the model does not declare and is reported as exactly that.
    pub outcome: Option<OutcomeRef>,
    /// The declared error the branch reports. Carries no field the model does not determine.
    pub error: Option<DeclaredErrorValue>,
    /// The events the branch emits, in declared order, each with its determined payload.
    pub events: Vec<ObservedEvent>,
    /// The store after the step.
    pub next: Store,
}

/// Why the model, as this module reads it, does not determine an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Undetermined {
    /// The model declares no such command.
    UnknownCommand(String),
    /// The request is not a value of what the model declares — an input or a forced outcome.
    Request(String),
    /// A guard could not be decided over the input it was given.
    Undecidable {
        /// The branch whose guard it is.
        outcome: String,
        /// The guard, as the model holds it.
        guard: String,
    },
    /// The model uses a construct this interpreter does not execute yet.
    NotInterpreted {
        /// Which construct, where.
        construct: String,
    },
    /// The model leaves a value to the implementation and its type has no shape to mint.
    NoValue {
        /// Which value.
        what: String,
    },
    /// The model's own outcome leaves an instance violating one of its declared invariants.
    BrokenInvariant {
        /// Which instance.
        instance: String,
        /// The invariant, as the author wrote it.
        invariant: String,
    },
}

impl Undetermined {
    /// `true` for the cases that are a gap in this interpreter rather than a defect in the request
    /// or the model — the ones a target reports as `unsupported`.
    pub fn is_capability_gap(&self) -> bool {
        matches!(
            self,
            Self::Undecidable { .. } | Self::NotInterpreted { .. } | Self::NoValue { .. }
        )
    }
}

impl fmt::Display for Undetermined {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(command) => write!(f, "the model declares no command `{command}`"),
            Self::Request(why) => write!(f, "the request is not what the model declares: {why}"),
            Self::Undecidable { outcome, guard } => write!(
                f,
                "the guard of `{outcome}` is Unknown over this input: {guard}"
            ),
            Self::NotInterpreted { construct } => {
                write!(f, "{construct} is not interpreted yet")
            }
            Self::NoValue { what } => write!(
                f,
                "the model leaves {what} to the implementation, and its type has no shape the \
                 interpreter can mint"
            ),
            Self::BrokenInvariant {
                instance,
                invariant,
            } => write!(
                f,
                "the model's own outcome leaves {instance} violating its invariant `{invariant}`"
            ),
        }
    }
}

impl std::error::Error for Undetermined {}

// ---- the step ----------------------------------------------------------------------------------

/// [`execute_generating`] with every assigned value minted from the store's counter.
///
/// # Errors
///
/// [`Undetermined`] where the model, as this interpreter reads it, determines no answer.
pub fn execute(
    ir: &EssIr,
    store: &Store,
    command: &QualifiedName,
    input: &BTreeMap<String, Node>,
    externals: &Externals,
) -> Result<Vec<Step>, Undetermined> {
    execute_generating(ir, store, command, input, externals, &Generated::Counter)
}

/// A missing document is its own request, never an empty map sent through input selection.
pub(super) fn without_input(
    ir: &EssIr,
    store: &Store,
    command: &QualifiedName,
    caller: Option<&caller::Caller<'_>>,
) -> Result<Step, Undetermined> {
    let store = State::import(store);
    let spec = ir
        .commands()
        .get(command)
        .ok_or_else(|| Undetermined::UnknownCommand(command.to_string()))?;
    let Some(outcome) = spec
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::InputAbsent)
    else {
        return undeclared(&store).publish();
    };
    // Admission restricts this marker to a refusal with no effects or input-dependent payload.
    take(
        ir,
        spec,
        outcome,
        &store,
        &Context {
            input: &BTreeMap::new(),
            caller,
            operation: None,
            now: None,
            unresolved: std::cell::RefCell::default(),
            domains: history::Domains::default(),
        },
        &Generated::Counter,
        &mut super::response::Authority::default(),
    )?
    .map_err(Undetermined::Request)?
    .publish()
}

/// Executes `command` with `input` against `store`, returning every distinct step the model allows.
///
/// Never empty: a request no declared branch covers is one step with no outcome. More than one
/// step is the model leaving the answer open, and the caller decides what to do with that — the
/// target refuses, a checker explores. Two branches that come to the same step are one step: the
/// model left the branch open and determined the answer.
///
/// # Errors
///
/// [`Undetermined`] where the model, as this interpreter reads it, determines no answer.
pub fn execute_generating(
    ir: &EssIr,
    store: &Store,
    command: &QualifiedName,
    input: &BTreeMap<String, Node>,
    externals: &Externals,
    generated: &Generated,
) -> Result<Vec<Step>, Undetermined> {
    execute_at(ir, store, command, input, externals, generated, None)
}

/// [`execute_generating`], decided at `decision_time`: the one instant this command decision
/// observed, which every guard of the decision reading the current time reads
/// ([`crate::occurrence_clock`], beyond10x/ess#244). The clock-free entrypoints pass `None`, and
/// with `None` a guard that needs the instant is Unknown — [`Undetermined::Undecidable`] — while
/// every answer decided before such a guard is reached stands.
///
/// # Errors
///
/// [`Undetermined`] where the model, as this interpreter reads it, determines no answer.
pub fn execute_at(
    ir: &EssIr,
    store: &Store,
    command: &QualifiedName,
    input: &BTreeMap<String, Node>,
    externals: &Externals,
    generated: &Generated,
    decision_time: Option<crate::occurrence_clock::DecisionInstant>,
) -> Result<Vec<Step>, Undetermined> {
    responding(
        ir,
        store,
        command,
        &Invocation {
            input,
            caller: None,
        },
        externals,
        generated,
        &mut super::response::Authority::default(),
        decision_time,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "one command decision: its request, its authorities and its one instant"
)]
pub(super) fn responding(
    ir: &EssIr,
    store: &Store,
    command: &QualifiedName,
    input: &Invocation<'_>,
    externals: &Externals,
    generated: &Generated,
    responses: &mut super::response::Authority,
    decision_time: Option<crate::occurrence_clock::DecisionInstant>,
) -> Result<Vec<Step>, Undetermined> {
    responding_core(
        ir,
        &State::import(store),
        command,
        &Context {
            input: input.input,
            caller: input.caller,
            operation: None,
            now: decision_time,
            unresolved: std::cell::RefCell::default(),
            domains: history::Domains::default(),
        },
        externals,
        generated,
        responses,
    )?
    .into_iter()
    .map(Transition::publish)
    .collect()
}

#[allow(
    clippy::too_many_lines,
    reason = "command execution keeps its ordered selection and response authorities together"
)]
fn responding_core(
    ir: &EssIr,
    store: &State,
    command: &QualifiedName,
    input: &Context<'_>,
    externals: &Externals,
    generated: &Generated,
    responses: &mut super::response::Authority,
) -> Result<Vec<Transition>, Undetermined> {
    let spec = ir
        .commands()
        .get(command)
        .ok_or_else(|| Undetermined::UnknownCommand(command.to_string()))?;
    if spec
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, ResolvedCondition::Related { .. }))
    {
        if let Some(step) = existence::existing(ir, spec, store, input, externals)? {
            return Ok(vec![step]);
        }
    }
    // A related row named by a stored field of the addressed subject (ess/22, beyond10x/ess#304)
    // is read after that row's existence and held state, below — not before the input refusals.
    let stored = related::stored(spec);
    if stored.is_none() {
        if let Some(steps) = related_absent(ir, spec, store, input, generated, responses)? {
            return Ok(steps);
        }
    }
    if let Some(steps) = refused_by_input(ir, spec, store, input)? {
        return Ok(steps);
    }
    if let Some(step) = existence::existing(ir, spec, store, input, externals)? {
        return Ok(vec![step]);
    }
    interpretable(spec, matches!(generated, Generated::Recorded(_)))?;
    let facts = input::flatten(ir, spec, input)
        .map_err(|errors| Undetermined::Request(errors.to_string()))?;
    let mut held_subjects = BTreeMap::new();
    if let Some((field, entity)) = stored {
        let read = stored_reference(
            ir,
            spec,
            store,
            input,
            (&facts, command, externals),
            generated,
            responses,
            (field, entity),
        )?;
        match read {
            Stored::Answered(steps) => return Ok(steps),
            Stored::Absent => {}
            Stored::Row(row) => {
                for outcome in &spec.outcomes {
                    if matches!(outcome.condition, ResolvedCondition::Related { .. }) {
                        held_subjects.insert(
                            outcome.name.clone(),
                            subject::Held::new(ir, ir.entity(entity), row)?,
                        );
                    }
                }
            }
        }
    }
    for outcome in &spec.outcomes {
        if stored.is_some() && matches!(outcome.condition, ResolvedCondition::Related { .. }) {
            continue;
        }
        if let Some(held) = related::held(ir, store, input, &outcome.condition)? {
            held_subjects.insert(outcome.name.clone(), held);
            continue;
        }
        if !matches!(
            outcome.condition,
            ResolvedCondition::SubjectState { .. }
                | ResolvedCondition::StateChange { .. }
                | ResolvedCondition::SubjectField { .. }
                | ResolvedCondition::SubjectPredicate { .. }
        ) {
            continue;
        }
        let subject =
            spec.selection_subject(outcome)
                .ok_or_else(|| Undetermined::NotInterpreted {
                    construct: "a state guard with no resolved subject".into(),
                })?;
        let ResolvedInstance::Supplied { field } = &subject.instance else {
            return Err(Undetermined::NotInterpreted {
                construct: "a state guard without a supplied subject".into(),
            });
        };
        let entity = &ir.entity(&subject.entity).name;
        let identity = input.get(&field.name).ok_or_else(|| {
            Undetermined::Request("the guarded subject identity is absent".into())
        })?;
        let key = identity.clone();
        let Some(held) = store.instance_typed(entity, &key) else {
            return Ok(vec![unknown_instance(
                ir, spec, store, input, generated, responses,
            )?]);
        };
        held_subjects.insert(
            outcome.name.clone(),
            subject::Held::new(ir, ir.entity(&subject.entity), held)?,
        );
    }
    let orders_present_related_refusal = orders_present_related_refusal(ir, spec);
    // A stored reference was read after the addressed row's existence and held state answered
    // ([`stored_reference`]); its present-related refusal answers before every accepting branch.
    let orders_present_related_refusal = orders_present_related_refusal || stored.is_some();
    if orders_present_related_refusal && stored.is_none() {
        if let Some(steps) = subject_refusals_before_present_related(
            ir,
            spec,
            store,
            input,
            &facts,
            command,
            externals,
            &held_subjects,
            generated,
            responses,
        )? {
            return Ok(steps);
        }
    }
    let selected = select(
        spec,
        &facts,
        command,
        externals,
        &held_subjects,
        input.caller,
        input,
        true,
        orders_present_related_refusal,
    )?;

    if selected.is_empty() {
        if input.unresolved.borrow().is_some() {
            return Ok(Vec::new());
        }
        return Ok(vec![undeclared(store)]);
    }
    let mut steps: Vec<Transition> = Vec::with_capacity(selected.len());
    let mut unmatched: Vec<String> = Vec::new();
    for outcome in selected {
        match take(ir, spec, outcome, store, input, generated, responses) {
            Ok(Ok(step)) if !steps.contains(&step) => steps.push(step),
            Ok(Ok(_)) => {}
            Ok(Err(why)) => unmatched.push(format!("`{}`: {why}", branch(spec, outcome))),
            Err(Undetermined::Request(why)) if input.operation.is_some() => unmatched.push(why),
            Err(why) => input.defer(why)?,
        }
    }
    if steps.is_empty() {
        return Err(Undetermined::Request(format!(
            "no branch the model allows is described by the given values — {}",
            unmatched.join("; ")
        )));
    }
    Ok(steps)
}

/// Whether a present-related predicate refusal answers before every accepting branch, after the
/// addressed row's existence and held state: from ess/22 beside `wrong_state` (beyond10x/ess#282),
/// and on a command reading several related rows, where the first declared refusal whose
/// predicate holds answers across them (beyond10x/ess#283).
fn orders_present_related_refusal(ir: &EssIr, spec: &ResolvedCommand) -> bool {
    ir.format().major() >= ess_domain::system::FormatVersion::V22.major()
        && spec.outcomes.iter().any(is_present_related_refusal)
        && (related::several(spec)
            || spec
                .outcomes
                .iter()
                .any(|outcome| matches!(outcome.condition, ResolvedCondition::WrongState)))
}

#[allow(
    clippy::too_many_arguments,
    reason = "the preflight shares the command execution's existing authorities"
)]
fn subject_refusals_before_present_related(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    facts: &input::InputFacts<'_>,
    command: &QualifiedName,
    externals: &Externals,
    held_subjects: &BTreeMap<OutcomeName, subject::Held<'_>>,
    generated: &Generated,
    responses: &mut super::response::Authority,
) -> Result<Option<Vec<Transition>>, Undetermined> {
    let after_related = select(
        spec,
        facts,
        command,
        externals,
        held_subjects,
        input.caller,
        input,
        false,
        false,
    )?;
    selected_subject_refusals(ir, spec, store, input, generated, responses, &after_related)
}

/// The branches `facts` select under `externals`, in the declared precedence
/// (`docs/design/input-guard-overlap-precedence.md`), read in order and stopped at the first that
/// answers, so a guard after it is never read: an Unknown there leaves the answer as it is, and an
/// Unknown before it is Undecidable.
///
/// 1. The input-guarded refusals, before any other branch and before the provider is asked
///    (beyond10x/ess#178): the first declared whose guard holds answers (beyond10x/ess#227). Most
///    requests are answered earlier by [`refused_by_input`]; this step reads the refusals it leaves.
/// 2. For the ess/22 `wrong_state` composition, the present-related predicate refusal whose guard
///    holds, after addressed-row existence and held state have been checked (beyond10x/ess#282).
/// 3. The accepting `when:` branches and the external branches, in declaration order: the first
///    whose guard holds answers (beyond10x/ess#217), and an external one holds where its provider
///    takes it — forced, never while withheld, and either way while open, where it stays one
///    possible answer beside whatever the declarations after it select.
/// 4. The default, where no guard and no provider answered.
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "selection preserves source-declared guard precedence in one ordered routine"
)]
fn select<'s>(
    spec: &'s ResolvedCommand,
    facts: &input::InputFacts<'_>,
    command: &QualifiedName,
    externals: &Externals,
    held_subjects: &BTreeMap<OutcomeName, subject::Held<'_>>,
    caller: Option<&caller::Caller<'_>>,
    context: &Context<'_>,
    include_present_related_refusals: bool,
    prioritize_present_related_refusals: bool,
) -> Result<Vec<&'s ResolvedOutcome>, Undetermined> {
    let invocation = caller::Facts::new(facts, caller, &spec.input, context.now);
    let holds = |outcome: &ResolvedOutcome, guard: &Predicate| match guard.evaluate(&invocation) {
        Truth::True => Ok(true),
        Truth::False => Ok(false),
        Truth::Unknown => Err(Undetermined::Undecidable {
            outcome: branch(spec, outcome),
            guard: format!("{guard:?}"),
        }),
    };
    let eligible_external = |outcome: &ResolvedOutcome| match &outcome.condition {
        ResolvedCondition::External { .. } => Ok(true),
        ResolvedCondition::ExternalWhen { predicate, .. } => holds(outcome, predicate),
        _ => Ok(false),
    };

    let is_external = |outcome: &ResolvedOutcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
        )
    };
    let forced = match externals {
        Externals::Forced(name) => Some(
            spec.outcomes
                .iter()
                .find(|outcome| &outcome.name == name)
                .filter(|outcome| is_external(outcome))
                .ok_or_else(|| {
                    Undetermined::Request(format!(
                        "`{command}/{name}` is not an outcome the model declares external"
                    ))
                })?,
        ),
        Externals::Withheld | Externals::Open => None,
    };

    let mut selected: Vec<&ResolvedOutcome> = Vec::new();
    for outcome in &spec.outcomes {
        if let (ResolvedCondition::When { predicate }, Some(_)) =
            (&outcome.condition, &outcome.error)
        {
            if holds(outcome, predicate)? {
                selected.push(outcome);
                break;
            }
        }
    }
    if selected.is_empty() && prioritize_present_related_refusals {
        for outcome in spec
            .outcomes
            .iter()
            .filter(|outcome| is_present_related_refusal(outcome))
        {
            let held = held_subjects
                .get(&outcome.name)
                .map(|held| {
                    held.selects(
                        &outcome.condition,
                        facts,
                        caller,
                        branch(spec, outcome),
                        context.now,
                    )
                })
                .transpose();
            match held {
                Ok(Some(Some(true))) => {
                    selected.push(outcome);
                    break;
                }
                Ok(_) => {}
                Err(why) => {
                    context.defer(why)?;
                    return Ok(Vec::new());
                }
            }
        }
    }
    if selected.is_empty() {
        let mut answered = false;
        for outcome in &spec.outcomes {
            let held = (!is_present_related_refusal(outcome) || include_present_related_refusals)
                .then(|| held_subjects.get(&outcome.name))
                .flatten()
                .map(|held| {
                    held.selects(
                        &outcome.condition,
                        facts,
                        caller,
                        branch(spec, outcome),
                        context.now,
                    )
                })
                .transpose();
            let takes = match held {
                Ok(takes) => takes.flatten(),
                Err(why) => {
                    context.defer(why)?;
                    // Only the provider choices already collected are proved. Neither this
                    // unresolved guard nor its fallback is a selected branch.
                    answered = true;
                    break;
                }
            };
            if let Some(takes) = takes {
                if takes {
                    selected.push(outcome);
                    answered = true;
                    break;
                }
                continue;
            }
            match &outcome.condition {
                ResolvedCondition::When { predicate } if outcome.error.is_none() => {
                    if holds(outcome, predicate)? {
                        selected.push(outcome);
                        answered = true;
                        break;
                    }
                }
                ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. } => {
                    match externals {
                        Externals::Withheld => {}
                        Externals::Forced(_) => {
                            if forced.is_some_and(|forced| forced.name == outcome.name)
                                && eligible_external(outcome)?
                            {
                                selected.push(outcome);
                                answered = true;
                                break;
                            }
                        }
                        Externals::Open => {
                            if eligible_external(outcome)? {
                                selected.push(outcome);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if !answered {
            selected.extend(
                spec.outcomes
                    .iter()
                    .filter(|outcome| matches!(outcome.condition, ResolvedCondition::Otherwise)),
            );
        }
    }
    Ok(selected)
}

fn is_present_related_refusal(outcome: &ResolvedOutcome) -> bool {
    outcome.error.is_some()
        && matches!(
            outcome.condition,
            ResolvedCondition::Related {
                test: ResolvedRelatedTest::Holds { .. },
                ..
            }
        )
}

/// The addressed-row existence or held-state answers selected before a present-related predicate
/// refusal (ess/22, beyond10x/ess#282). `selected` is the complete alternative set the later
/// accepting/external step would take with those refusals omitted. Each alternative keeps its own
/// subject authority: an invalid moving branch becomes the lifecycle refusal while an independent
/// nonmoving or external branch remains possible beside it. If every selected alternative may act,
/// selection continues to the present-related refusal phase without preparing any response.
fn selected_subject_refusals(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    generated: &Generated,
    responses: &mut super::response::Authority,
    selected: &[&ResolvedOutcome],
) -> Result<Option<Vec<Transition>>, Undetermined> {
    let mut refusals = Vec::with_capacity(selected.len());
    for outcome in selected {
        refusals.push(selected_subject_refusal(
            ir, spec, store, input, generated, responses, outcome,
        )?);
    }
    if refusals.iter().all(Option::is_none) {
        return Ok(None);
    }

    let mut steps = Vec::with_capacity(selected.len());
    let mut unmatched = Vec::new();
    for (outcome, refusal) in selected.iter().copied().zip(refusals) {
        if let Some(step) = refusal {
            if !steps.contains(&step) {
                steps.push(step);
            }
            continue;
        }
        match take(ir, spec, outcome, store, input, generated, responses) {
            Ok(Ok(step)) if !steps.contains(&step) => steps.push(step),
            Ok(Ok(_)) => {}
            Ok(Err(why)) => unmatched.push(format!("`{}`: {why}", branch(spec, outcome))),
            Err(Undetermined::Request(why)) if input.operation.is_some() => unmatched.push(why),
            Err(why) => input.defer(why)?,
        }
    }
    if steps.is_empty() {
        return Err(Undetermined::Request(format!(
            "no branch the model allows is described by the given values — {}",
            unmatched.join("; ")
        )));
    }
    Ok(Some(steps))
}

fn selected_subject_refusal(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    generated: &Generated,
    responses: &mut super::response::Authority,
    outcome: &ResolvedOutcome,
) -> Result<Option<Transition>, Undetermined> {
    let Some(subject) = &outcome.subject else {
        return Ok(None);
    };
    if subject.effect == ResolvedEffect::Creates {
        return Ok(None);
    }
    let ResolvedInstance::Supplied { field } = &subject.instance else {
        return Ok(None);
    };
    let identity = input.get(&field.name).ok_or_else(|| {
        Undetermined::Request(format!(
            "`{}` names its subject in `{}`, and the input has none",
            spec.name, field.name
        ))
    })?;
    let entity = ir.entity(&subject.entity);
    let Some(held) = store.instance_typed(&entity.name, identity) else {
        return unknown_instance(ir, spec, store, input, generated, responses).map(Some);
    };
    if let ResolvedEffect::Moves { transition } = &subject.effect {
        if !transition.from.contains(&held.state) {
            return wrong_state(ir, spec, store, input, Some(held)).map(Some);
        }
    }
    Ok(None)
}

/// The answer of a command guarded by a related row (`when_related:`, ess/18) whose related row is
/// missing: its `exists: false` branch, before any input-guarded refusal — step 1 of the
/// precedence order (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence
/// order"). `None` on a command with no related guard, or where the row is stored.
///
/// On a command reading several rows through its input (ess/22, beyond10x/ess#283) the rows are
/// read in the declaration order of their `exists: false` branches, and the first missing one
/// answers: a missing row is answered before any present row's predicate, whatever the order.
///
/// `existing_instance:` is resolved before this helper on such a command. An absent Optional
/// reference (ess/22, beyond10x/ess#304) answers `None` without a lookup. A related row named
/// through a required input the request does not carry is declined: nothing here reads it. One
/// named through a stored field of the addressed subject is never asked here; it is read at its
/// own later step ([`stored_reference`]).
fn related_absent(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    generated: &Generated,
    responses: &mut super::response::Authority,
) -> Result<Option<Vec<Transition>>, Undetermined> {
    let gap = |construct: String| Err(Undetermined::NotInterpreted { construct });
    for read in related::reads_in_order(spec) {
        let ResolvedCondition::Related { via, entity, .. } = &read.condition else {
            unreachable!("ordered among related guards")
        };
        let ResolvedRelatedVia::Input { field, type_ref } = via else {
            return gap(format!(
                "the guard over a row named by a stored field of `{}`",
                branch(spec, read)
            ));
        };
        // An absent Optional reference (ess/22, beyond10x/ess#304) reads no row and selects no
        // related branch: it is not a missing row, and selection carries on without it.
        if related::absent_optional(type_ref, input.get(field)) {
            continue;
        }
        let Some(identity) = input.get(field) else {
            return gap(format!(
                "the guard over a related row of `{}` with no identity in `{field}`",
                branch(spec, read)
            ));
        };
        let entity = &ir.entity(entity).name;
        if store.instance_typed(entity, identity).is_some() {
            continue;
        }
        return missing_row(ir, spec, store, input, generated, responses, Some(field));
    }
    Ok(None)
}

/// The `exists: false` branch over the row `field` names taken — the command's only one where
/// `field` is `None` — for a related row no row carries the identity of; `None` where it declares
/// none.
fn missing_row(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    generated: &Generated,
    responses: &mut super::response::Authority,
    field: Option<&str>,
) -> Result<Option<Vec<Transition>>, Undetermined> {
    let Some(absent) = spec.outcomes.iter().find(|outcome| {
        matches!(
            &outcome.condition,
            ResolvedCondition::Related {
                via,
                test: ResolvedRelatedTest::Absent,
                ..
            } if field.is_none_or(|field| via.field() == field)
        )
    }) else {
        return Ok(None);
    };
    if absent.subject.is_none() && absent.error.is_some() {
        return Ok(Some(vec![refusal(ir, spec, absent, store, input, None)?]));
    }
    match take(ir, spec, absent, store, input, generated, responses)? {
        Ok(step) => Ok(Some(vec![step])),
        Err(why) => Err(Undetermined::Request(format!(
            "no branch the model allows is described by the given values — `{}`: {why}",
            branch(spec, absent)
        ))),
    }
}

/// What reading a stored reference left selection with ([`stored_reference`]).
enum Stored<'a> {
    /// The addressed row's existence, its held state, or a missing related row answered.
    Answered(Vec<Transition>),
    /// The reference is absent: no row is read and no related branch is selected.
    Absent,
    /// The related row the reference names, for the related branches to read.
    Row(&'a Row),
}

/// The related row named by a stored field of the addressed subject (`when_related: {via:
/// <field>}`, ess/22, beyond10x/ess#304), read at its step of the precedence order
/// (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order"): after the
/// input-guarded refusals, the addressed row's existence — an identity no row holds takes the
/// command's not-found answer — and its held state — a selected branch moving from a state the
/// row does not hold takes `wrong_state` — and before every accepting branch.
///
/// The field is read from the subject as it was before the branch. Absent, it names no row; an
/// identity no row carries is answered by the `exists: false` branch; otherwise the row it names is
/// the one the related branches read.
#[allow(
    clippy::too_many_arguments,
    reason = "the stored read shares the command execution's existing authorities"
)]
fn stored_reference<'s>(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &'s State,
    input: &Context<'_>,
    (facts, command, externals): (&input::InputFacts<'_>, &QualifiedName, &Externals),
    generated: &Generated,
    responses: &mut super::response::Authority,
    (field, entity): (&str, &ess_compiler::ir::EntityHandle),
) -> Result<Stored<'s>, Undetermined> {
    let subject = spec
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .find(|subject| subject.effect != ResolvedEffect::Creates)
        .ok_or_else(|| Undetermined::NotInterpreted {
            construct: "a stored-field related guard with no addressed subject".into(),
        })?;
    let ResolvedInstance::Supplied { field: named } = &subject.instance else {
        return Err(Undetermined::NotInterpreted {
            construct: "a stored-field related guard without a supplied subject".into(),
        });
    };
    let identity = input.get(&named.name).ok_or_else(|| {
        Undetermined::Request(format!(
            "`{}` names its subject in `{}`, and the input has none",
            spec.name, named.name
        ))
    })?;
    // 3. The addressed row's existence.
    let Some(addressed) = store.instance_typed(&ir.entity(&subject.entity).name, identity) else {
        return Ok(Stored::Answered(vec![unknown_instance(
            ir, spec, store, input, generated, responses,
        )?]));
    };
    // 4. Its held state, for the branch the request selects with no related row read.
    let selected = select(
        spec,
        facts,
        command,
        externals,
        &BTreeMap::new(),
        input.caller,
        input,
        false,
        false,
    )?;
    if let Some(steps) =
        selected_subject_refusals(ir, spec, store, input, generated, responses, &selected)?
    {
        return Ok(Stored::Answered(steps));
    }
    // 5. The stored reference, as the subject held it before the branch.
    match related::reference(ir, store, addressed, field, entity)? {
        related::Reference::Absent => Ok(Stored::Absent),
        related::Reference::Row(row) => Ok(Stored::Row(row)),
        related::Reference::Missing => {
            match missing_row(ir, spec, store, input, generated, responses, None)? {
                Some(steps) => Ok(Stored::Answered(steps)),
                None => Err(Undetermined::Undecidable {
                    outcome: "related-row selection".into(),
                    guard: format!(
                        "no `{}` row holds the identity `{field}` stores",
                        ir.entity(entity).name
                    ),
                }),
            }
        }
    }
}

/// The input-guarded refusals (`when:` with an `error:`, naming no subject) the input selects,
/// answered before anything else is read, or `None` where it selects none.
///
/// Such a refusal is taken before existence, the held state, the stored row, a provider and every
/// accepting branch it overlaps (`docs/design/input-guard-overlap-precedence.md`,
/// `docs/design/outcome-shapes.md` "Precedence", beyond10x/ess#178, #209, #227). So a request it
/// claims is answered here even on a command whose other branches read what this module does not
/// execute — a held state, a stored field — since none of them is consulted; a request it does not
/// claim goes on to [`interpretable`] as before. Of two refusals the input selects together, the
/// first declared answers, as Entity Runtime orders them (`ess-entity-runtime` sorts input-guarded
/// refusals first, then by declaration, and takes the first whose guard holds).
///
/// On a command guarded by a related row (`when_related:`) it runs after [`related_absent`]: a
/// missing related row is answered by its `exists: false` branch first
/// (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order").
///
/// A refusal's guard reading the current time reads the one instant this decision observed
/// ([`Context::now`]); with none, the first such guard the order reaches is Unknown, and every
/// refusal declared before it still answers. A request that does not decode is left to the
/// ordinary path, which reports it in the order it always has.
fn refused_by_input(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
) -> Result<Option<Vec<Transition>>, Undetermined> {
    let refusals: Vec<(&ResolvedOutcome, &Predicate)> = spec
        .outcomes
        .iter()
        .filter(|outcome| {
            outcome.error.is_some() && outcome.subject.is_none() && outcome.replays.is_none()
        })
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::When { predicate } if !predicate.is_trivially_true() => {
                Some((outcome, predicate))
            }
            _ => None,
        })
        .collect();
    if refusals.is_empty() {
        return Ok(None);
    }
    let Ok(facts) = input::flatten(ir, spec, input) else {
        return Ok(None);
    };
    let facts = caller::Facts::new(&facts, input.caller, &spec.input, input.now);
    for (outcome, guard) in refusals {
        match guard.evaluate(&facts) {
            Truth::True => return Ok(Some(vec![refusal(ir, spec, outcome, store, input, None)?])),
            Truth::False => {}
            Truth::Unknown => {
                return Err(Undetermined::Undecidable {
                    outcome: branch(spec, outcome),
                    guard: format!("{guard:?}"),
                })
            }
        }
    }
    Ok(None)
}

/// Refuses a command using any construct this module does not execute, before anything is read.
///
/// `recorded` is a caller replaying what a history recorded ([`Generated::Recorded`]), which
/// records neither a typed response nor a retained result's contents: for it, a command that
/// declares a response is stepped without one, and a `replays:` branch is the step it declares —
/// the retained answer again, with no event and no change to any instance. Whether anything was
/// retained to replay is the request's, which a step does not see: the checker takes a replay only
/// once its request's origin branch has been taken in the order it tries ([`crate::linearize`]). A target,
/// which owes the response and the retained result themselves, is still refused both.
fn interpretable(spec: &ResolvedCommand, recorded: bool) -> Result<(), Undetermined> {
    let gap = |construct: String| Err(Undetermined::NotInterpreted { construct });
    // Every guard reading the current time — an input guard, and a `when_subject:` or
    // `when_related:` predicate over a stored row (ess/22, family F A3) — reads the decision's one
    // instant ([`Context::now`]) over the pre-outcome snapshot; with none, the first such guard the
    // precedence order reaches is Unknown.
    for outcome in &spec.outcomes {
        if !recorded && (outcome.replays.is_some() || outcome.retains_result) {
            return gap(format!(
                "the retained result of `{}`",
                branch(spec, outcome)
            ));
        }
    }
    Ok(())
}

/// The step for a request no declared branch covers.
fn undeclared(store: &State) -> Transition {
    Transition {
        outcome: None,
        error: None,
        events: Vec::new(),
        next: store.clone(),
    }
}

/// `command/outcome`, as a diagnostic names a branch.
fn branch(spec: &ResolvedCommand, outcome: &ResolvedOutcome) -> String {
    format!("{}/{}", spec.name, outcome.name)
}

/// The reference a step reports for `outcome` of `spec`.
fn reference(spec: &ResolvedCommand, outcome: &ResolvedOutcome) -> OutcomeRef {
    OutcomeRef::new(CommandRef::new(spec.name.clone()), outcome.name.clone())
}

/// The declared error `outcome` reports, carrying the fields the specification gives a source
/// (ess/19, `story:error-payload-sources`) and no other.
///
/// An input field carries the input's value, a literal its value at the field's type, and a
/// `{subject: …}` the row the refusal is answered for (`held`). A `{generated: true}` field is
/// the implementation's to choose, so it is not carried, and a field with no source is not
/// carried either, as before `ess/19`. Caller attributes come from this invocation's validated
/// authentication facts, including nested leaves. Related values read the original store, through
/// the same typed address and presence rules as event payloads and assignments.
fn declared_error(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    input: &Context<'_>,
    held: Option<&Row>,
    store: &State,
) -> Result<Option<history::Error>, Undetermined> {
    let Some(handle) = outcome.error.as_ref().filter(|_| outcome.refuses) else {
        return Ok(None);
    };
    let mut error = history::Error::new(ErrorRef::from(handle));
    let reads = values::Reads {
        original: store,
        before: held,
        outcome,
    };
    let location = vec!["error".into(), error.error.to_string()];
    for field in &outcome.error_payload {
        if let Some(value) = error_value(ir, field, input, reads, &location)? {
            if input.operation.is_some() {
                history::validate(ir, &field.target_type, &value)?;
            }
            error = error.with(field.target.clone(), value);
        }
    }
    if let Some(operation) = input.operation {
        for field in &ir.error(handle).fields {
            if outcome
                .error_payload
                .iter()
                .any(|source| source.target == field.name)
            {
                continue;
            }
            let mut path = location.clone();
            path.push(field.name.clone());
            let value = history::generated(
                ir,
                &field.type_ref,
                history::Origin {
                    operation: operation.into(),
                    branch: outcome.name.to_string(),
                    location: path,
                },
                &input.domains,
            )?;
            error = error.with(field.name.clone(), value);
        }
    }
    Ok(Some(error))
}

fn error_value(
    ir: &EssIr,
    field: &ResolvedPayloadField,
    input: &Context<'_>,
    reads: values::Reads<'_>,
    location: &[String],
) -> Result<Option<Value>, Undetermined> {
    let mut path = location.to_vec();
    path.push(field.target.clone());
    Ok(match &field.value {
        ResolvedPayloadValue::Generated => match input.operation {
            None => None,
            Some(operation) => Some(history::generated(
                ir,
                &field.target_type,
                history::Origin {
                    operation: operation.into(),
                    branch: reads.outcome.name.to_string(),
                    location: path,
                },
                &input.domains,
            )?),
        },
        ResolvedPayloadValue::CallerAttribute {
            attribute,
            type_ref,
        } => input
            .caller_value(ir, attribute, type_ref)?
            .map(Value::Known),
        ResolvedPayloadValue::Struct { fields } => {
            let mut values = BTreeMap::new();
            for member in fields {
                if let Some(value) = error_value(ir, member, input, reads, &path)? {
                    values.insert(member.target.clone(), value);
                }
            }
            Some(Value::Object(values))
        }
        ResolvedPayloadValue::SubjectField {
            field: read,
            type_ref,
        } => values::subject(
            ir,
            read,
            type_ref,
            &field.target_type,
            field.conversion.as_deref(),
            reads.before,
        )?,
        ResolvedPayloadValue::RelatedField { .. } => reads.related(ir, field, input)?,
        ResolvedPayloadValue::InputField { field: read, .. } => input
            .get(read)
            .filter(|value| **value != Node::Null)
            .cloned()
            .map(Value::Known),
        ResolvedPayloadValue::Literal { value } => {
            Some(Value::Known(literal(ir, &field.target_type, value)?))
        }
        other => {
            return Err(Undetermined::NotInterpreted {
                construct: format!("the value source `{}` of an error", other.describe()),
            })
        }
    })
}

/// The store a branch is building, and where its assigned values come from.
struct Work<'g> {
    domains: &'g history::Domains,
    original: &'g State,
    outcome: &'g ResolvedOutcome,
    next: State,
    supply: Supply<'g>,
    /// Immutable pre-outcome fields, including the exact identity held outside `Row.fields`.
    before: Option<Row>,
    response: Option<super::response::Value>,
    operation: Option<&'g str>,
    location: Vec<String>,
    /// The field location being written, independent of history-generation provenance.
    target_location: Vec<String>,
}

impl Work<'_> {
    fn origin(&self) -> history::Origin {
        history::Origin {
            operation: self.operation.expect("history-only generation").into(),
            branch: self.outcome.name.to_string(),
            location: self.location.clone(),
        }
    }
    fn reads(&self) -> values::Reads<'_> {
        values::Reads {
            original: self.original,
            before: self.before.as_ref(),
            outcome: self.outcome,
        }
    }
}

/// Brings the instance `subject` creates into existence in `work`, and returns its identity.
///
/// A supplied identity already held yields no step. An interpreter-minted identity skips held
/// candidates before any other field is generated; setup may have reserved the next counter value.
fn create(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    subject: &ResolvedSubject,
    input: &Context<'_>,
    work: &mut Work<'_>,
) -> Result<Result<Node, Unmatched>, Undetermined> {
    let entity = ir.entity(&subject.entity);
    let ResolvedInstance::Observed { event, field } = &subject.instance else {
        unreachable!("`creates:` publishes its identity; `take` routes nothing else here")
    };
    let source = existence::identity_source(outcome);
    let (identity, key) = if existence::generated(source, input) {
        or_no_step!(creation_identity(
            ir,
            &entity.name,
            event.name(),
            field,
            work
        ))
    } else {
        let identity = existence::identity(ir, source.expect("determined identity"), input, work)?;
        if work.next.instance_typed(&entity.name, &identity).is_some() {
            return Ok(Err(format!(
                "a supplied identity is already held by `{}`, and creation never replaces it",
                entity.name
            )));
        }
        (identity.clone(), identity)
    };
    let state = subject
        .into
        .clone()
        .unwrap_or_else(|| entity.lifecycle.initial.clone());
    let mut fields = BTreeMap::new();
    work.location = vec![
        "row".into(),
        entity.name.to_string(),
        format!("{key:?}"),
        "sets".into(),
    ];
    write(ir, &outcome.sets, input, &mut fields, work)?;
    work.next
        .instances
        .entry(entity.name.clone())
        .or_default()
        .insert(key, Row { state, fields });
    Ok(Ok(identity))
}

fn creation_identity(
    ir: &EssIr,
    entity: &QualifiedName,
    event: &QualifiedName,
    field: &ess_compiler::ir::ResolvedField,
    work: &mut Work<'_>,
) -> Result<Result<(Node, Node), Unmatched>, Undetermined> {
    let supplied = work
        .supply
        .given
        .is_some_and(|given| given.contains_key(&GeneratedSlot::new(event.clone(), &field.name)));
    let occupied = work.next.instances.get(entity).map_or(0, BTreeMap::len);
    // At most one candidate beyond the held population: sufficient for the counter's distinct
    // UUIDs, and a bounded refusal when a constrained witness repeats or is exhausted.
    for _ in 0..=occupied {
        let identity = or_no_step!(assign(ir, event, &field.name, &field.type_ref, work))
            .ok_or_else(|| Undetermined::NoValue {
                what: format!("the identity of a new `{entity}`"),
            })?
            .require("the actual new row identity")?;
        let key = identity.clone();
        if work.next.instance_typed(entity, &key).is_none() {
            return Ok(Ok((identity, key)));
        }
        if supplied {
            return Ok(Err(format!(
                "the identity `{key:?}` is already held by a `{entity}`, and `creates:` never \
                 replaces an instance"
            )));
        }
    }
    Err(Undetermined::NoValue {
        what: format!("a fresh identity of `{entity}` within the bounded witness search"),
    })
}

/// Takes one selected outcome: resolves its subject, writes, emits, and checks what now rests.
///
/// `Ok(Err(_))` is a branch the given values do not describe, which yields no step.
#[allow(
    clippy::too_many_lines,
    reason = "one transaction routine keeps outcome effects atomic across native and history modes"
)]
fn take(
    ir: &EssIr,
    spec: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    store: &State,
    input: &Context<'_>,
    generated: &Generated,
    responses: &mut super::response::Authority,
) -> Result<Result<Transition, Unmatched>, Undetermined> {
    let mut prepared = None;
    let mut work = Work {
        domains: &input.domains,
        original: store,
        outcome,
        next: store.clone(),
        supply: Supply::of(generated),
        before: values::selected_subject(ir, spec, outcome, store, input),
        response: None,
        operation: input.operation,
        location: Vec::new(),
        target_location: Vec::new(),
    };
    let mut created: Option<(String, Node)> = None;
    let mut touched: Option<(QualifiedName, Node)> = None;

    if let Some(subject) = &outcome.subject {
        let entity = ir.entity(&subject.entity);
        match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, ResolvedInstance::Observed { field, .. }) => {
                // The declared creation identity can itself read the actual response.
                prepared = responses.prepare(ir, spec, outcome)?;
                work.response = prepared.as_ref().and_then(|value| value.value.clone());
                let identity = or_no_step!(create(ir, outcome, subject, input, &mut work));
                let key = identity.clone();
                created = Some((field.name.clone(), identity));
                touched = Some((entity.name.clone(), key));
            }
            (ResolvedEffect::Creates, ResolvedInstance::Supplied { .. })
            | (_, ResolvedInstance::Observed { .. }) => {
                return Err(unsupported_instance(spec, outcome, subject));
            }
            (effect, ResolvedInstance::Supplied { field }) => {
                let identity = input.get(&field.name).ok_or_else(|| {
                    Undetermined::Request(format!(
                        "`{}` names its subject in `{}`, and the input has none",
                        spec.name, field.name
                    ))
                })?;
                let key = identity.clone();
                let Some(held) = store.instance_typed(&entity.name, &key) else {
                    return Ok(Ok(unknown_instance(
                        ir, spec, store, input, generated, responses,
                    )?));
                };
                work.location = vec![
                    "row".into(),
                    entity.name.to_string(),
                    format!("{key:?}"),
                    "sets".into(),
                ];
                let after = match act(ir, &outcome.sets, effect, held, input, &mut work)? {
                    Acted::NotFromHere => {
                        return Ok(Ok(wrong_state(
                            ir,
                            spec,
                            store,
                            input,
                            work.before.as_ref(),
                        )?))
                    }
                    Acted::Rests(after) => Some(after),
                    Acted::Removed => None,
                };
                let held = work.next.instances.entry(entity.name.clone()).or_default();
                match after {
                    Some(after) => {
                        held.insert(key.clone(), after);
                        touched = Some((entity.name.clone(), key));
                    }
                    None => {
                        held.remove(&key);
                    }
                }
            }
        }
    } else if outcome.instances.is_none() && !outcome.sets.is_empty() {
        return Err(Undetermined::NotInterpreted {
            construct: format!("`sets:` without a subject, in `{}`", branch(spec, outcome)),
        });
    }

    let changed = set_effects::apply(ir, spec, outcome, store, input, &mut work)?;
    if prepared.is_none() {
        prepared = responses.prepare(ir, spec, outcome)?;
        work.response = prepared.as_ref().and_then(|value| value.value.clone());
    }
    let events = or_no_step!(emit(
        ir,
        spec,
        outcome,
        input,
        created.as_ref(),
        changed,
        &mut work
    ));

    if let Some((entity, key)) = &touched {
        at_rest(ir, &work.next, entity, key)?;
    }
    let step = Transition {
        outcome: Some(reference(spec, outcome)),
        error: declared_error(ir, outcome, input, work.before.as_ref(), store)?,
        events,
        next: work.next,
    };
    responses.completed(reference(spec, outcome), prepared);
    Ok(Ok(step))
}

fn unsupported_instance(
    spec: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    subject: &ResolvedSubject,
) -> Undetermined {
    Undetermined::NotInterpreted {
        construct: format!(
            "a `{}` whose identity is {}, in `{}`",
            subject.effect.verb(),
            match subject.instance {
                ResolvedInstance::Supplied { .. } => "supplied",
                ResolvedInstance::Observed { .. } => "observed",
            },
            branch(spec, outcome)
        ),
    }
}

/// What `effect` does to an instance the store holds. Never `creates:`, which [`take`] answers.
fn act(
    ir: &EssIr,
    sets: &[ResolvedPayloadField],
    effect: &ResolvedEffect,
    held: &Row,
    input: &Context<'_>,
    work: &mut Work<'_>,
) -> Result<Acted, Undetermined> {
    let mut after = held.clone();
    match effect {
        ResolvedEffect::Moves { transition } => {
            if !transition.from.contains(&held.state) {
                return Ok(Acted::NotFromHere);
            }
            after.state = transition.to.clone();
        }
        ResolvedEffect::Updates | ResolvedEffect::Preserves | ResolvedEffect::Creates => {}
        ResolvedEffect::Deletes => return Ok(Acted::Removed),
    }
    write(ir, sets, input, &mut after.fields, work)?;
    Ok(Acted::Rests(after))
}

/// What one effect did to a held instance.
enum Acted {
    /// The move does not start from the state held: the command's wrong-state question, not this
    /// branch's.
    NotFromHere,
    /// The instance rests on, holding this.
    Rests(Row),
    /// The instance is removed.
    Removed,
}

/// The command's `unknown_instance:` branch, else its one declared not-found refusal, else its
/// `wrong_state:` branch, else no declared one.
///
/// The order is the model's (`ResolvedCondition::UnknownInstance`, and existence before held state
/// in `docs/design/cross-record-and-stored-field-guards.md#the-precedence-order`): an identity
/// nobody holds is answered by the marker that exists for it; then by an `external:` refusal whose
/// error carries the identity's type, which synthesis reads as the not-found answer
/// (`synthesize::declared_not_found`, beyond10x/ess#291); and only then by the wrong-state branch —
/// an instance that does not exist rests in no state any move starts from.
fn unknown_instance(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    generated: &Generated,
    responses: &mut super::response::Authority,
) -> Result<Transition, Undetermined> {
    match spec
        .outcomes
        .iter()
        .find(|outcome| matches!(outcome.condition, ResolvedCondition::UnknownInstance))
        .or_else(|| crate::synthesize::declared_not_found(ir, spec))
    {
        Some(outcome)
            if outcome
                .subject
                .as_ref()
                .is_some_and(|subject| subject.effect == ResolvedEffect::Creates) =>
        {
            take(ir, spec, outcome, store, input, generated, responses)?
                .map_err(Undetermined::Request)
        }
        Some(outcome) => refusal(ir, spec, outcome, store, input, None),
        None => wrong_state(ir, spec, store, input, None),
    }
}

/// The command's `wrong_state:` branch, else no declared one. `held` is the row it is answered
/// for, where one is held.
fn wrong_state(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &State,
    input: &Context<'_>,
    held: Option<&Row>,
) -> Result<Transition, Undetermined> {
    match spec
        .outcomes
        .iter()
        .find(|outcome| matches!(outcome.condition, ResolvedCondition::WrongState))
    {
        Some(outcome) => refusal(ir, spec, outcome, store, input, held),
        None => Ok(undeclared(store)),
    }
}

/// A branch answered for the subject rather than the input: it reports its error where it refuses,
/// and in either case moves, writes and emits nothing (`refusal_mutated_state`).
fn refusal(
    ir: &EssIr,
    spec: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    store: &State,
    input: &Context<'_>,
    held: Option<&Row>,
) -> Result<Transition, Undetermined> {
    Ok(Transition {
        outcome: Some(reference(spec, outcome)),
        error: declared_error(ir, outcome, input, held, store)?,
        events: Vec::new(),
        next: store.clone(),
    })
}

/// Writes `sets:` into `fields`.
fn write(
    ir: &EssIr,
    sets: &[ResolvedPayloadField],
    input: &Context<'_>,
    fields: &mut BTreeMap<String, Value>,
    work: &mut Work<'_>,
) -> Result<(), Undetermined> {
    for set in sets {
        match &set.value {
            ResolvedPayloadValue::Cleared => {
                fields.remove(&set.target);
            }
            _ => match value(ir, set, input, work)? {
                Some(value) => {
                    fields.insert(set.target.clone(), value);
                }
                None => {
                    fields.remove(&set.target);
                }
            },
        }
    }
    Ok(())
}

/// The value one determined field takes. `None` is absent: an optional input the caller did not
/// send, or an optional generated value.
fn value(
    ir: &EssIr,
    field: &ResolvedPayloadField,
    input: &Context<'_>,
    work: &mut Work<'_>,
) -> Result<Option<Value>, Undetermined> {
    work.location.push(field.target.clone());
    work.target_location.push(field.target.clone());
    let result = value_at(ir, field, input, work);
    work.target_location.pop();
    work.location.pop();
    let value = result?;
    if input.operation.is_some() {
        history::validate(
            ir,
            &field.target_type,
            value.as_ref().unwrap_or(&Value::Absent),
        )?;
    }
    Ok(value)
}

fn value_at(
    ir: &EssIr,
    field: &ResolvedPayloadField,
    input: &Context<'_>,
    work: &mut Work<'_>,
) -> Result<Option<Value>, Undetermined> {
    match &field.value {
        ResolvedPayloadValue::ResponseField { .. } => {
            values::response(ir, field, work.response.as_ref()).map(|value| value.map(Value::Known))
        }
        ResolvedPayloadValue::RelatedField { .. } => work.reads().related(ir, field, input),
        ResolvedPayloadValue::SubjectField {
            field: read,
            type_ref,
        } => values::subject(
            ir,
            read,
            type_ref,
            &field.target_type,
            field.conversion.as_deref(),
            work.before.as_ref(),
        ),
        ResolvedPayloadValue::Increment { by } => values::increment(
            ir,
            field,
            by,
            work.before.as_ref(),
            &work.target_location,
            work.operation.is_some(),
        )
        .map(Some),
        ResolvedPayloadValue::CallerAttribute {
            attribute,
            type_ref,
        } => input
            .caller_value(ir, attribute, type_ref)
            .map(|value| value.map(Value::Known)),
        ResolvedPayloadValue::InputField { field: source, .. } => Ok(input
            .get(source)
            .filter(|value| **value != Node::Null)
            .cloned()
            .map(Value::Known)),
        ResolvedPayloadValue::Literal { value } => {
            literal(ir, &field.target_type, value).map(|value| Some(Value::Known(value)))
        }
        ResolvedPayloadValue::Generated => mint(ir, &field.target_type, work),
        ResolvedPayloadValue::Struct { fields } => {
            let mut members = BTreeMap::new();
            for member in fields {
                if let Some(value) = value(ir, member, input, work)? {
                    members.insert(member.target.clone(), value);
                }
            }
            Ok(Some(Value::Object(members)))
        }
        ResolvedPayloadValue::InputOrGenerated {
            field: source,
            otherwise,
            ..
        } => match input.get(source).filter(|value| **value != Node::Null) {
            Some(value) => Ok(Some(Value::Known(value.clone()))),
            None => match otherwise {
                Some(text) => {
                    literal(ir, &field.target_type, text).map(|value| Some(Value::Known(value)))
                }
                None => mint(ir, &field.target_type, work),
            },
        },

        other => Err(Undetermined::NotInterpreted {
            construct: format!("the value source `{}`", other.describe()),
        }),
    }
}

/// The typed value a literal written in the model denotes, read against the field it fills.
fn literal(ir: &EssIr, target: &ResolvedTypeRef, text: &str) -> Result<Node, Undetermined> {
    let gap = || Undetermined::NotInterpreted {
        construct: format!("the literal `{text}` over `{target}`"),
    };
    let mut current = target.required();
    for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
        let value = match current {
            ResolvedTypeRef::Primitive { name } => {
                input::primitive_literal(*name, text).ok_or_else(gap)?
            }
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => {
                    current = of.required();
                    continue;
                }
                ResolvedBody::Enum { .. } => Node::Text(text.into()),
                _ => return Err(gap()),
            },
            _ => return Err(gap()),
        };
        input::validate_typed_value(ir, target, &value).map_err(Undetermined::Request)?;
        return Ok(value);
    }
    Err(gap())
}

/// The observable value the implementation assigns to `field` of the emitted `event`.
///
/// From [`Generated::Given`] or [`Generated::Recorded`] where the caller has values, by the rules
/// those variants state; otherwise [`mint`]ed. `Ok(Err(_))` is a branch the given values do not describe.
fn assign(
    ir: &EssIr,
    event: &QualifiedName,
    field: &str,
    target: &ResolvedTypeRef,
    work: &mut Work<'_>,
) -> Result<Result<Option<Value>, Unmatched>, Undetermined> {
    let Some(given) = work.supply.given else {
        return mint(ir, target, work).map(Ok);
    };
    let slot = GeneratedSlot::new(event.clone(), field);
    match given.get(&slot) {
        None if work.operation.is_some() && work.supply.mint_absent => {
            mint(ir, target, work).map(Ok)
        }
        None | Some(Node::Null) if target.is_optional() => Ok(Ok(None)),
        None if work.supply.mint_absent => mint(ir, target, work).map(Ok),
        None => Ok(Err(format!("no value was given for `{event}.{field}`"))),
        Some(value) => Ok(match input::validate_typed_value(ir, target, value) {
            Ok(()) => Ok(Some(Value::Known(value.clone()))),
            Err(why) => Err(format!(
                "the value given for `{event}.{field}` is not a `{target}`: {why}"
            )),
        }),
    }
}

/// A value the implementation assigns, minted from the store's counter in the shape its declared
/// type takes.
///
/// `None` for an optional type: absent is a value the implementation may choose, and the one that
/// claims least.
fn mint(
    ir: &EssIr,
    target: &ResolvedTypeRef,
    work: &mut Work<'_>,
) -> Result<Option<Value>, Undetermined> {
    if work.operation.is_some() {
        return history::generated(ir, target, work.origin(), work.domains).map(
            |value| match value {
                Value::Absent => None,
                value => Some(value),
            },
        );
    }
    if target.is_optional() {
        return Ok(None);
    }
    let no_value = || Undetermined::NoValue {
        what: format!("a value of `{target}`"),
    };
    if let Representation::Primitive(Primitive::Uuid) = representation(ir, target) {
        Ok(Some(Value::Known(Node::Text(format!(
            "00000000-0000-4000-8000-{:012}",
            work.next.tick()
        )))))
    } else {
        let field = ess_compiler::ir::ResolvedField {
            name: "generated".into(),
            type_ref: target.clone(),
            naming: ess_domain::name::Naming::default(),
        };
        let distinction = usize::try_from(work.next.tick()).map_err(|_| no_value())?;
        crate::witness::fields(
            ir,
            &[field],
            crate::witness::Distinction::further(distinction),
        )
        .map(|mut values| values.remove("generated").map(Value::Known))
        .map_err(|_| no_value())
    }
}

/// What a field's type is underneath every newtype, where no newtype on the way constrains it.
enum Representation {
    Primitive(Primitive),
    Enum,
    Other,
}

fn representation(ir: &EssIr, target: &ResolvedTypeRef) -> Representation {
    let mut current = target.required();
    // Bounded by the IR's own type depth: a newtype chain is a chain of declared handles.
    for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Primitive { name } => return Representation::Primitive(*name),
            ResolvedTypeRef::Declared { name } => {
                let declared = ir.named_type(name);
                match &declared.body {
                    ResolvedBody::Newtype { of, .. } if !declared.body.is_constrained() => {
                        current = of.required();
                    }
                    ResolvedBody::Enum { .. } => return Representation::Enum,
                    _ => return Representation::Other,
                }
            }
            _ => return Representation::Other,
        }
    }
    Representation::Other
}

/// The events `outcome` emits, each carrying what the model determines of it.
fn emit(
    ir: &EssIr,
    spec: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: &Context<'_>,
    created: Option<&(String, Node)>,
    changed: Option<usize>,
    work: &mut Work<'_>,
) -> Result<Result<Vec<history::Event>, Unmatched>, Undetermined> {
    let mut events = Vec::with_capacity(outcome.emits.len());
    for (index, handle) in outcome.emits.iter().enumerate() {
        let declared = ir.event(handle);
        let determined = outcome
            .payload
            .iter()
            .find(|payload| &payload.event == handle);
        let identity = outcome
            .subject
            .as_ref()
            .and_then(|subject| match &subject.instance {
                ResolvedInstance::Observed { event, field } if event == handle => {
                    Some(field.name.as_str())
                }
                _ => None,
            });
        let mut event = history::Event::new(EventRef::from(handle));
        for field in &declared.fields {
            work.location = vec![
                "event".into(),
                index.to_string(),
                declared.name.to_string(),
                field.name.clone(),
            ];
            let source = determined
                .and_then(|payload| payload.fields.iter().find(|it| it.target == field.name));
            // The field `instance:` names publishes the created identity, whatever source the
            // payload declares for it: `{generated: true}` there says only that the implementation
            // assigns it, and it assigned it once, when the instance was created.
            let value = match (identity == Some(field.name.as_str()), source) {
                (true, _) => created.map(|(_, identity)| Value::Known(identity.clone())),
                (
                    false,
                    Some(ResolvedPayloadField {
                        value: ResolvedPayloadValue::Generated,
                        ..
                    }),
                ) => or_no_step!(assign(
                    ir,
                    &declared.name,
                    &field.name,
                    &field.type_ref,
                    work
                )),
                (
                    false,
                    Some(ResolvedPayloadField {
                        value: ResolvedPayloadValue::ChangedCount,
                        ..
                    }),
                ) => {
                    let count = changed.ok_or_else(|| {
                        Undetermined::Request("changed count without a set subject".into())
                    })?;
                    Some(Value::Known(Node::Number(Number::from(count))))
                }
                (false, Some(source)) => value(ir, source, input, work)?,

                (false, None) => {
                    or_no_step!(
                        assign(ir, &declared.name, &field.name, &field.type_ref, work).map_err(
                            |error| match error {
                                Undetermined::NoValue { .. } => Undetermined::NoValue {
                                    what: format!(
                                        "`{}.{}`, emitted by `{}`",
                                        declared.name,
                                        field.name,
                                        branch(spec, outcome)
                                    ),
                                },
                                other => other,
                            }
                        )
                    )
                }
            };
            if let Some(value) = value {
                event = event.with(field.name.clone(), value);
            }
        }
        events.push(event);
    }
    Ok(Ok(events))
}

/// Refuses a step that leaves an instance violating an invariant its entity declares.
///
/// An invariant reading a field no outcome has written evaluates to `Unknown`, and concludes
/// nothing: the value is the implementation's, and the model has not said it breaks anything.
fn at_rest(
    ir: &EssIr,
    store: &State,
    entity: &QualifiedName,
    key: &Node,
) -> Result<(), Undetermined> {
    let Some(declared) = ir.entities().get(entity) else {
        return Ok(());
    };
    let Some(instance) = store.instance_typed(entity, key) else {
        return Ok(());
    };
    let mut fields = declared.fields.clone();
    fields.push(declared.identity.clone());
    let mut row = instance.clone();
    row.fields
        .insert(declared.identity.name.clone(), Value::Known(key.clone()));
    let facts = history::Facts::row(ir, &fields, &row)?;
    for invariant in &declared.invariants {
        let universal = facts.evaluate_with(|candidate| invariant.predicate.evaluate(candidate));
        if universal == Truth::True {
            continue;
        }
        if universal == Truth::False {
            return Err(Undetermined::BrokenInvariant {
                instance: format!("`{entity}` `{key:?}`"),
                invariant: invariant.statement.clone(),
            });
        }
        let result = invariant.predicate.outcome(&facts);
        if result.truth == Truth::False {
            return Err(Undetermined::BrokenInvariant {
                instance: format!("`{entity}` `{key:?}`"),
                invariant: invariant.statement.clone(),
            });
        }
        if result.truth == Truth::Unknown
            && result
                .causes
                .iter()
                .filter(|cause| cause.truth == Truth::Unknown)
                .flat_map(|cause| &cause.missing)
                .any(|path| facts.unobserved(path))
        {
            return Err(Undetermined::Undecidable {
                outcome: format!("invariant of `{entity}` at `{key:?}`"),
                guard: invariant.statement.clone(),
            });
        }
    }
    Ok(())
}
