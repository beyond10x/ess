//! One command, executed from the IR: state in, command in, every outcome the model allows out.
//!
//! This is the library half of the interpreter, and it is kept free of a runner on purpose. The
//! [`Interpreted`](super::Interpreted) target calls [`execute`] once per `ExecuteCommand` step and
//! refuses when the answer is not exactly one step; a linearizability checker calls the same
//! function as its sequential model and explores every step it returns.
//!
//! # What is derived, and from where
//!
//! | fact | read from |
//! |---|---|
//! | which outcome the input selects | the precedence order: a missing related row's `exists: false` branch (`related_absent`), then the first declared input-guarded refusal whose `when:` holds, before anything else is read (`refused_by_input`); else the first accepting or external branch declared whose guard holds, over [`input::flatten`], then the one `Otherwise` branch |
//! | whether an external branch is taken | [`Externals`] — never the input, never this module |
//! | whether the subject may move | the transition's own `from` set against the state held in the [`Store`] |
//! | what a refused move answers | the command's `wrong_state:` branch, or its `unknown_instance:` branch for an identity nobody holds |
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
//! Where the model uses a construct this module does not execute yet — a guard over the subject's
//! stored fields or its held state, a retained replay, a typed response, a value expression other
//! than an input field or a literal — the answer is [`Undetermined::NotInterpreted`], never a
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
//! steps. A created identity is assigned once, the event field `instance:` names publishes that
//! value whatever payload source it declares, and an identity the store already holds is never
//! created again.

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
use ess_primitives::facts::FactValue;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate, Truth};
use ess_primitives::time::CurrentTime;

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
    instances: BTreeMap<QualifiedName, BTreeMap<String, Instance>>,
    minted: u64,
}

impl Store {
    /// The instance of `entity` with this identity, when one is held.
    pub fn instance(&self, entity: &QualifiedName, identity: &str) -> Option<&Instance> {
        self.instances.get(entity)?.get(identity)
    }

    /// Every held instance, by entity and then by identity.
    pub fn instances(&self) -> impl Iterator<Item = (&QualifiedName, &str, &Instance)> {
        self.instances.iter().flat_map(|(entity, held)| {
            held.iter()
                .map(move |(identity, instance)| (entity, identity.as_str(), instance))
        })
    }

    /// The next value of the counter.
    fn tick(&mut self) -> u64 {
        self.minted += 1;
        self.minted
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
    let spec = ir
        .commands()
        .get(command)
        .ok_or_else(|| Undetermined::UnknownCommand(command.to_string()))?;
    if let Some(steps) = related_absent(ir, spec, store, input, generated)? {
        return Ok(steps);
    }
    if let Some(steps) = refused_by_input(ir, spec, store, input)? {
        return Ok(steps);
    }
    interpretable(spec, matches!(generated, Generated::Recorded(_)))?;
    let facts = input::flatten(ir, spec, input)
        .map_err(|errors| Undetermined::Request(errors.to_string()))?;
    let selected = select(spec, &facts, command, externals)?;

    if selected.is_empty() {
        return Ok(vec![undeclared(store)]);
    }
    let mut steps: Vec<Step> = Vec::with_capacity(selected.len());
    let mut unmatched: Vec<String> = Vec::new();
    for outcome in selected {
        match take(ir, spec, outcome, store, input, generated)? {
            Ok(step) if !steps.contains(&step) => steps.push(step),
            Ok(_) => {}
            Err(why) => unmatched.push(format!("`{}`: {why}", branch(spec, outcome))),
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

/// The branches `facts` select under `externals`, in the declared precedence
/// (`docs/design/input-guard-overlap-precedence.md`), read in order and stopped at the first that
/// answers, so a guard after it is never read: an Unknown there leaves the answer as it is, and an
/// Unknown before it is Undecidable.
///
/// 1. The input-guarded refusals, before any other branch and before the provider is asked
///    (beyond10x/ess#178): the first declared whose guard holds answers (beyond10x/ess#227). Most
///    requests are answered earlier by [`refused_by_input`]; this step reads the refusals it leaves.
/// 2. The accepting `when:` branches and the external branches, in declaration order: the first
///    whose guard holds answers (beyond10x/ess#217), and an external one holds where its provider
///    takes it — forced, never while withheld, and either way while open, where it stays one
///    possible answer beside whatever the declarations after it select.
/// 3. The default, where no guard and no provider answered.
fn select<'s>(
    spec: &'s ResolvedCommand,
    facts: &input::InputFacts<'_>,
    command: &QualifiedName,
    externals: &Externals,
) -> Result<Vec<&'s ResolvedOutcome>, Undetermined> {
    let holds = |outcome: &ResolvedOutcome, guard: &Predicate| match guard.evaluate(facts) {
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
    if selected.is_empty() {
        let mut answered = false;
        for outcome in &spec.outcomes {
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

/// The answer of a command guarded by a related row (`when_related:`, ess/18) whose related row is
/// missing: its `exists: false` branch, before any input-guarded refusal — step 1 of the
/// precedence order (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence
/// order"). `None` on a command with no related guard, or where the row is stored.
///
/// `existing_instance:` answers before it on such a command, and this module does not decide the
/// command's own existence, so a command declaring one is declined here. A related row named
/// through a stored field of the subject, or through an input the request does not carry as text,
/// is declined too: nothing here reads it.
fn related_absent(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &Store,
    input: &BTreeMap<String, Node>,
    generated: &Generated,
) -> Result<Option<Vec<Step>>, Undetermined> {
    let related: Vec<&ResolvedOutcome> = spec
        .outcomes
        .iter()
        .filter(|outcome| matches!(outcome.condition, ResolvedCondition::Related { .. }))
        .collect();
    let Some(first) = related.first() else {
        return Ok(None);
    };
    let gap = |construct: String| Err(Undetermined::NotInterpreted { construct });
    if let Some(existing) = spec
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::ExistingInstance)
    {
        return gap(format!(
            "the existing-instance branch of `{}`, which answers before its related row is read",
            branch(spec, existing)
        ));
    }
    let ResolvedCondition::Related { via, entity, .. } = &first.condition else {
        unreachable!("filtered to related guards above")
    };
    let ResolvedRelatedVia::Input { field, .. } = via else {
        return gap(format!(
            "the guard over a row named by a stored field of `{}`",
            branch(spec, first)
        ));
    };
    let Some(identity) = input.get(field).and_then(Node::as_text) else {
        return gap(format!(
            "the guard over a related row of `{}` with no text identity in `{field}`",
            branch(spec, first)
        ));
    };
    let entity = &ir.entity(entity).name;
    if store.instance(entity, identity).is_some() {
        return Ok(None);
    }
    let Some(absent) = related.iter().find(|outcome| {
        matches!(
            &outcome.condition,
            ResolvedCondition::Related {
                test: ResolvedRelatedTest::Absent,
                ..
            }
        )
    }) else {
        return Ok(None);
    };
    if absent.subject.is_none() && absent.error.is_some() {
        return Ok(Some(vec![refusal(ir, spec, absent, store, input, None)?]));
    }
    match take(ir, spec, absent, store, input, generated)? {
        Ok(step) => Ok(Some(vec![step])),
        Err(why) => Err(Undetermined::Request(format!(
            "no branch the model allows is described by the given values — `{}`: {why}",
            branch(spec, absent)
        ))),
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
/// Where a refusal's guard reads the current time, nothing is answered early and [`interpretable`]
/// names that guard. A request that does not decode is left to the ordinary path, which reports it
/// in the order it always has.
fn refused_by_input(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &Store,
    input: &BTreeMap<String, Node>,
) -> Result<Option<Vec<Step>>, Undetermined> {
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
    if refusals.is_empty() || refusals.iter().any(|(_, guard)| reads_now(guard)) {
        return Ok(None);
    }
    let Ok(facts) = input::flatten(ir, spec, input) else {
        return Ok(None);
    };
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
    if !recorded && !spec.response.is_empty() {
        return gap(format!("the typed response of `{}`", spec.name));
    }
    for outcome in &spec.outcomes {
        let at = branch(spec, outcome);
        if let ResolvedCondition::When { predicate }
        | ResolvedCondition::ExternalWhen { predicate, .. } = &outcome.condition
        {
            if reads_now(predicate) {
                return gap(format!("the current-time guard of `{at}`"));
            }
        }
        match &outcome.condition {
            ResolvedCondition::When { .. }
            | ResolvedCondition::Otherwise
            | ResolvedCondition::External { .. }
            | ResolvedCondition::ExternalWhen { .. }
            | ResolvedCondition::WrongState
            | ResolvedCondition::UnknownInstance => {}
            ResolvedCondition::SubjectField { .. } | ResolvedCondition::SubjectPredicate { .. } => {
                return gap(format!(
                    "the guard over the subject's stored fields of `{at}`"
                ));
            }
            ResolvedCondition::SubjectState { .. } | ResolvedCondition::StateChange { .. } => {
                return gap(format!("the guard over the subject's held state of `{at}`"));
            }
            ResolvedCondition::Related { .. } => {
                return gap(format!("the guard over a related row of `{at}`"));
            }
            ResolvedCondition::InputAbsent => {
                return gap(format!("the absent-input branch of `{at}`"));
            }
            ResolvedCondition::ExistingInstance => {
                return gap(format!("the existing-instance branch of `{at}`"));
            }
        }
        if !recorded && (outcome.replays.is_some() || outcome.retains_result) {
            return gap(format!("the retained result of `{at}`"));
        }
    }
    Ok(())
}

/// Whether a guard compares a fact with the current time (`now`, `now - 60s`; ess/16). The model
/// holds no clock, so such a guard is not interpreted: comparing the operand as text would take a
/// branch the specification does not.
fn reads_now(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => children.iter().any(reads_now),
        Predicate::Not(inner) => reads_now(inner),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            reads_now(&quantified.body)
        }
        Predicate::Compare { left, right, .. } => [left, right].into_iter().any(|operand| {
            matches!(operand, Operand::Literal(FactValue::Text(text))
                if CurrentTime::parse(text).is_some())
        }),
        _ => false,
    }
}

/// The step for a request no declared branch covers.
fn undeclared(store: &Store) -> Step {
    Step {
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
/// carried either, as before `ess/19`. A source this module does not read — the caller, a related
/// row — is a gap, as it is on an event's payload.
fn declared_error(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    input: &BTreeMap<String, Node>,
    held: Option<&Instance>,
) -> Result<Option<DeclaredErrorValue>, Undetermined> {
    let Some(handle) = outcome.error.as_ref().filter(|_| outcome.refuses) else {
        return Ok(None);
    };
    let mut error = DeclaredErrorValue::new(ErrorRef::from(handle));
    for field in &outcome.error_payload {
        let value = match &field.value {
            ResolvedPayloadValue::Generated => None,
            ResolvedPayloadValue::SubjectField { field: read, .. } => {
                held.and_then(|row| row.fields.get(read)).cloned()
            }
            ResolvedPayloadValue::InputField { field: read, .. } => input
                .get(read)
                .filter(|value| **value != Node::Null)
                .cloned(),
            ResolvedPayloadValue::Literal { value } => {
                Some(literal(ir, &field.target_type, value)?)
            }
            other => {
                return Err(Undetermined::NotInterpreted {
                    construct: format!(
                        "the value source `{}` of error `{handle}`",
                        other.describe()
                    ),
                })
            }
        };
        if let Some(value) = value {
            error = error.with(field.target.clone(), value);
        }
    }
    Ok(Some(error))
}

/// The store a branch is building, and where its assigned values come from.
struct Work<'g> {
    next: Store,
    supply: Supply<'g>,
}

/// Brings the instance `subject` creates into existence in `work`, and returns its identity.
///
/// The identity is assigned once, from the published slot `instance:` names. One the store already
/// holds is not a new instance, so the branch yields no step rather than replacing it.
fn create(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    subject: &ResolvedSubject,
    input: &BTreeMap<String, Node>,
    work: &mut Work<'_>,
) -> Result<Result<Node, Unmatched>, Undetermined> {
    let entity = ir.entity(&subject.entity);
    let ResolvedInstance::Observed { event, field } = &subject.instance else {
        unreachable!("`creates:` publishes its identity; `take` routes nothing else here")
    };
    let identity = or_no_step!(assign(ir, event.name(), &field.name, &field.type_ref, work))
        .ok_or_else(|| Undetermined::NoValue {
            what: format!("the identity of a new `{}`", entity.name),
        })?;
    let key = identity_key(&identity, &entity.name)?;
    if work.next.instance(&entity.name, &key).is_some() {
        return Ok(Err(format!(
            "the identity `{key}` is already held by a `{}`, and `creates:` never replaces an \
             instance",
            entity.name
        )));
    }
    let state = subject
        .into
        .clone()
        .unwrap_or_else(|| entity.lifecycle.initial.clone());
    let mut fields = BTreeMap::new();
    write(ir, &outcome.sets, input, None, &mut fields, work)?;
    work.next
        .instances
        .entry(entity.name.clone())
        .or_default()
        .insert(key, Instance { state, fields });
    Ok(Ok(identity))
}

/// Takes one selected outcome: resolves its subject, writes, emits, and checks what now rests.
///
/// `Ok(Err(_))` is a branch the given values do not describe, which yields no step.
fn take(
    ir: &EssIr,
    spec: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    store: &Store,
    input: &BTreeMap<String, Node>,
    generated: &Generated,
) -> Result<Result<Step, Unmatched>, Undetermined> {
    let mut work = Work {
        next: store.clone(),
        supply: Supply::of(generated),
    };
    let mut created: Option<(String, Node)> = None;
    let mut touched: Option<(QualifiedName, String)> = None;
    // The row an existing subject held before this branch, which an error field may read.
    let mut before: Option<Instance> = None;

    if let Some(subject) = &outcome.subject {
        let entity = ir.entity(&subject.entity);
        match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, ResolvedInstance::Observed { field, .. }) => {
                let identity = or_no_step!(create(ir, outcome, subject, input, &mut work));
                let key = identity_key(&identity, &entity.name)?;
                created = Some((field.name.clone(), identity));
                touched = Some((entity.name.clone(), key));
            }
            (ResolvedEffect::Creates, ResolvedInstance::Supplied { .. })
            | (_, ResolvedInstance::Observed { .. }) => {
                return Err(Undetermined::NotInterpreted {
                    construct: format!(
                        "a `{}` whose identity is {}, in `{}`",
                        subject.effect.verb(),
                        match subject.instance {
                            ResolvedInstance::Supplied { .. } => "supplied",
                            ResolvedInstance::Observed { .. } => "observed",
                        },
                        branch(spec, outcome)
                    ),
                })
            }
            (effect, ResolvedInstance::Supplied { field }) => {
                let identity = input.get(&field.name).ok_or_else(|| {
                    Undetermined::Request(format!(
                        "`{}` names its subject in `{}`, and the input has none",
                        spec.name, field.name
                    ))
                })?;
                let key = identity_key(identity, &entity.name)?;
                let Some(held) = store.instance(&entity.name, &key) else {
                    return Ok(Ok(unknown_instance(ir, spec, store, input)?));
                };
                before = Some(held.clone());
                let after = match act(ir, &outcome.sets, effect, held, input, &mut work)? {
                    Acted::NotFromHere => {
                        return Ok(Ok(wrong_state(ir, spec, store, input, Some(held))?))
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
    } else if !outcome.sets.is_empty() {
        return Err(Undetermined::NotInterpreted {
            construct: format!("`sets:` without a subject, in `{}`", branch(spec, outcome)),
        });
    }

    let events = or_no_step!(emit(ir, spec, outcome, input, created.as_ref(), &mut work));
    if let Some((entity, key)) = &touched {
        at_rest(ir, &work.next, entity, key)?;
    }
    Ok(Ok(Step {
        outcome: Some(reference(spec, outcome)),
        error: declared_error(ir, outcome, input, before.as_ref())?,
        events,
        next: work.next,
    }))
}

/// What `effect` does to an instance the store holds. Never `creates:`, which [`take`] answers.
fn act(
    ir: &EssIr,
    sets: &[ResolvedPayloadField],
    effect: &ResolvedEffect,
    held: &Instance,
    input: &BTreeMap<String, Node>,
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
    write(ir, sets, input, Some(held), &mut after.fields, work)?;
    Ok(Acted::Rests(after))
}

/// What one effect did to a held instance.
enum Acted {
    /// The move does not start from the state held: the command's wrong-state question, not this
    /// branch's.
    NotFromHere,
    /// The instance rests on, holding this.
    Rests(Instance),
    /// The instance is removed.
    Removed,
}

/// The identity of an instance as the store keys it: the text an identity type is written as.
fn identity_key(identity: &Node, entity: &QualifiedName) -> Result<String, Undetermined> {
    identity
        .as_text()
        .map(ToOwned::to_owned)
        .ok_or_else(|| Undetermined::NotInterpreted {
            construct: format!("an identity of `{entity}` that is not text"),
        })
}

/// The command's `unknown_instance:` branch, else its `wrong_state:` branch, else no declared one.
///
/// The order is the model's (`ResolvedCondition::UnknownInstance`): an identity nobody holds is
/// answered by the marker that exists for it, and before it existed by the wrong-state branch — an
/// instance that does not exist rests in no state any move starts from.
fn unknown_instance(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &Store,
    input: &BTreeMap<String, Node>,
) -> Result<Step, Undetermined> {
    match spec
        .outcomes
        .iter()
        .find(|outcome| matches!(outcome.condition, ResolvedCondition::UnknownInstance))
    {
        Some(outcome) => refusal(ir, spec, outcome, store, input, None),
        None => wrong_state(ir, spec, store, input, None),
    }
}

/// The command's `wrong_state:` branch, else no declared one. `held` is the row it is answered
/// for, where one is held.
fn wrong_state(
    ir: &EssIr,
    spec: &ResolvedCommand,
    store: &Store,
    input: &BTreeMap<String, Node>,
    held: Option<&Instance>,
) -> Result<Step, Undetermined> {
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
    store: &Store,
    input: &BTreeMap<String, Node>,
    held: Option<&Instance>,
) -> Result<Step, Undetermined> {
    Ok(Step {
        outcome: Some(reference(spec, outcome)),
        error: declared_error(ir, outcome, input, held)?,
        events: Vec::new(),
        next: store.clone(),
    })
}

/// Writes `sets:` into `fields`.
fn write(
    ir: &EssIr,
    sets: &[ResolvedPayloadField],
    input: &BTreeMap<String, Node>,
    before: Option<&Instance>,
    fields: &mut BTreeMap<String, Node>,
    work: &mut Work<'_>,
) -> Result<(), Undetermined> {
    for set in sets {
        match &set.value {
            ResolvedPayloadValue::Cleared => {
                fields.remove(&set.target);
            }
            ResolvedPayloadValue::SubjectField { field, .. } => {
                match before.and_then(|held| held.fields.get(field)) {
                    Some(value) => {
                        fields.insert(set.target.clone(), value.clone());
                    }
                    None => {
                        fields.remove(&set.target);
                    }
                }
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
    input: &BTreeMap<String, Node>,
    work: &mut Work<'_>,
) -> Result<Option<Node>, Undetermined> {
    match &field.value {
        ResolvedPayloadValue::InputField { field: source, .. } => Ok(input
            .get(source)
            .filter(|value| **value != Node::Null)
            .cloned()),
        ResolvedPayloadValue::Literal { value } => literal(ir, &field.target_type, value).map(Some),
        ResolvedPayloadValue::Generated => mint(ir, &field.target_type, work),
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
    match representation(ir, target) {
        Representation::Primitive(Primitive::Integer) => text
            .parse::<i64>()
            .map(|value| Node::Number(Number::from(value)))
            .map_err(|_| gap()),
        Representation::Primitive(Primitive::Decimal) => Number::decimal_literal(text)
            .map(Node::Number)
            .ok_or_else(gap),
        Representation::Primitive(Primitive::Boolean) => match text {
            "true" => Ok(Node::Bool(true)),
            "false" => Ok(Node::Bool(false)),
            _ => Err(gap()),
        },
        Representation::Primitive(
            Primitive::String | Primitive::Uuid | Primitive::Timestamp | Primitive::Duration,
        )
        | Representation::Enum => Ok(Node::Text(text.to_owned())),
        Representation::Primitive(_) | Representation::Other => Err(gap()),
    }
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
) -> Result<Result<Option<Node>, Unmatched>, Undetermined> {
    let Some(given) = work.supply.given else {
        return mint(ir, target, work).map(Ok);
    };
    let slot = GeneratedSlot::new(event.clone(), field);
    match given.get(&slot) {
        None | Some(Node::Null) if target.is_optional() => Ok(Ok(None)),
        None if work.supply.mint_absent => mint(ir, target, work).map(Ok),
        None => Ok(Err(format!("no value was given for `{event}.{field}`"))),
        Some(value) => Ok(match input::validate_typed_value(ir, target, value) {
            Ok(()) => Ok(Some(value.clone())),
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
) -> Result<Option<Node>, Undetermined> {
    if target.is_optional() {
        return Ok(None);
    }
    let no_value = || Undetermined::NoValue {
        what: format!("a value of `{target}`"),
    };
    match representation(ir, target) {
        Representation::Primitive(Primitive::Uuid) => Ok(Some(Node::Text(format!(
            "00000000-0000-4000-8000-{:012}",
            work.next.tick()
        )))),
        _ => Err(no_value()),
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
    input: &BTreeMap<String, Node>,
    created: Option<&(String, Node)>,
    work: &mut Work<'_>,
) -> Result<Result<Vec<ObservedEvent>, Unmatched>, Undetermined> {
    let mut events = Vec::with_capacity(outcome.emits.len());
    for handle in &outcome.emits {
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
        let mut event = ObservedEvent::new(EventRef::from(handle));
        for field in &declared.fields {
            let source = determined
                .and_then(|payload| payload.fields.iter().find(|it| it.target == field.name));
            // The field `instance:` names publishes the created identity, whatever source the
            // payload declares for it: `{generated: true}` there says only that the implementation
            // assigns it, and it assigned it once, when the instance was created.
            let value = match (identity == Some(field.name.as_str()), source) {
                (true, _) => created.map(|(_, identity)| identity.clone()),
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
    store: &Store,
    entity: &QualifiedName,
    key: &str,
) -> Result<(), Undetermined> {
    let Some(declared) = ir.entities().get(entity) else {
        return Ok(());
    };
    let Some(instance) = store.instance(entity, key) else {
        return Ok(());
    };
    let facts = input::bind(
        ir,
        &declared.fields,
        &instance.fields,
        Completeness::Partial,
    )
    .map_err(|errors| Undetermined::Request(errors.to_string()))?;
    let facts = TypedFacts::new(ir, &declared.fields, facts);
    for invariant in &declared.invariants {
        if invariant.predicate.evaluate(&facts) == Truth::False {
            return Err(Undetermined::BrokenInvariant {
                instance: format!("`{entity}` `{key}`"),
                invariant: invariant.statement.clone(),
            });
        }
    }
    Ok(())
}
