//! Synthesis for a model whose commands depend on who sends them (source format `ess/16`,
//! beyond10x/ess#168, `docs/design/caller-values.md`).
//!
//! An actor declares `attributes:` its credential carries; a command reads one as a value
//! (`{caller: account_id}`) or compares one in a guard (`caller.agent_id`). Nothing else in
//! synthesis knows what a caller is, and nothing has to: under a fixed **assignment** — which of two
//! callers sends each command — every caller read is a constant, and the model it leaves is one the
//! rest of this module already synthesizes. So:
//!
//! 1. Two callers are chosen, `first` and `second`: one value per attribute each, from the
//!    attribute's type by the witness builder, far from every witness an input is given.
//! 2. The model is read under an assignment by writing the assigned caller's values in place of
//!    every read — a value source becomes the literal, a guard's `caller.<attribute>` operand the
//!    literal — and synthesized as any other.
//! 3. Every command step of the result is marked with the caller its assignment sends it as, so the
//!    target sends it authenticated as that caller (suite/26, [`crate::caller_values`]).
//!
//! The suite is the one synthesized with **every** command sent by `first`. A branch that assignment
//! cannot reach — the refusal for a caller who is not the record's agent, where the record was made
//! by `first` — is taken from the assignment that sends the command under test as `second` and
//! everything that arranges it as `first`, so the refusal is witnessed with two callers.
//!
//! Then every scenario that sends a command reading the caller runs a second time, in the same
//! scenario, with the two callers' roles swapped (its instances renamed): created by `second` and
//! carrying `second`'s account; refused to `first` on `second`'s note. An implementation that
//! answers one caller by name — records one fixed account, refuses everyone but one caller — fails
//! the half in which that caller's role is the other one. A scenario whose steps cannot be run twice
//! in one scenario (a fixture prelude, a retained replay, a periodic check) keeps its first run only.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EssIr, ResolvedCommand, ResolvedCondition, ResolvedField, ResolvedInstance,
    ResolvedPayloadField, ResolvedPayloadValue,
};
use ess_domain::command::caller_value::CALLER_NAMESPACE;
use ess_domain::name::QualifiedName;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate};

use super::{grant, synthesize_plain, Focus, Note, Synthesis};
use crate::scenario::{
    ConformanceScenario, ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation,
};
use crate::witness::{candidates, Distinction, MAX_CANDIDATES};

/// How far the two callers' values sit from the plain witness: past every further instance an
/// arrangement numbers, and short of [`Distinction::UNKNOWN`], so no input witness shares one.
const FIRST: usize = 1 << 18;

/// The suffix a swapped run's instances are renamed with, so they bind apart from the first run's.
const SWAPPED: &str = "swapped";

/// The first witness a swapped run's caller-supplied identities are drawn from: between the
/// callers' own values and [`Distinction::UNKNOWN`], past every further instance an arrangement
/// numbers (beyond10x/ess#275).
const SWAPPED_IDENTITY: usize = 1 << 19;

/// How many further witnesses one swapped identity may try before the scenario keeps its first run
/// only: a type with fewer values than the suite sends identities runs out.
const SWAPPED_TRIES: usize = 64;

/// Whether any actor of the model declares an attribute its credential carries.
pub(super) fn uses(ir: &EssIr) -> bool {
    ir.actors()
        .values()
        .any(|actor| !actor.attributes.is_empty())
}

/// Which of the two callers sends a command.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Who {
    First,
    Second,
}

impl Who {
    fn other(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}

/// The interpretation that supplied an invocation's guards, inputs and effects.
#[derive(Clone, Copy)]
pub(super) enum InvocationPhase {
    Arrange,
    Act,
}

/// Arrangements and acting commands may read the same source under different credentials.
/// Keep both whole commands: a creator must satisfy its own competing guards.
pub(super) struct InvocationModels<'a> {
    pub arrangement: &'a EssIr,
    pub acting: &'a EssIr,
    credentials: Option<(&'a EssIr, &'a Callers, Who)>,
    /// The explicitly admitted synthesis seeds (beyond10x/ess#413): empty but for a seeded request.
    pub seeds: &'a super::AdmittedSeeds,
}

impl<'a> InvocationModels<'a> {
    pub fn plain(ir: &'a EssIr) -> Self {
        Self::seeded(ir, &super::seeds::EMPTY)
    }

    /// [`plain`](Self::plain), offering `seeds` where ordinary arrangement leaves a row unmet.
    pub fn seeded(ir: &'a EssIr, seeds: &'a super::AdmittedSeeds) -> Self {
        Self {
            arrangement: ir,
            acting: ir,
            credentials: None,
            seeds,
        }
    }

    /// Stamp a block while the planner still knows its role. Previously stamped nested blocks
    /// keep their role; in particular a stored-field boundary contains both phases.
    pub fn mark(&self, phase: InvocationPhase, steps: &mut [ScenarioStep]) {
        let Some((ir, callers, first)) = self.credentials else {
            return;
        };
        let who = match phase {
            InvocationPhase::Arrange => first,
            InvocationPhase::Act => first.other(),
        };
        for step in steps {
            if matches!(step, ScenarioStep::ExecuteCommand { caller, .. } | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } if !caller.is_empty())
            {
                continue;
            }
            mark(ir, callers, &|_| who, step);
        }
    }

    pub fn mark_run(&self, run: &mut super::Run) {
        self.mark(InvocationPhase::Arrange, &mut run.setup);
        self.mark(InvocationPhase::Act, &mut run.invoke);
    }
}

/// The two callers: for each actor that declares attributes, one value per attribute, each.
///
/// Held per actor, because two actors may declare one attribute name at different types for
/// commands of their own (the domain refuses that only where one command's actors disagree); each
/// is sent a value of its own type.
struct Callers {
    first: BTreeMap<String, BTreeMap<String, Node>>,
    second: BTreeMap<String, BTreeMap<String, Node>>,
}

impl Callers {
    /// Values of every attribute each actor declares, at that actor's type, far from every input
    /// witness and different between the two callers; `None` where the witness builder has no value
    /// for one.
    fn of(ir: &EssIr) -> Option<Self> {
        // The witness builder values a command's input; each actor's attributes are given one to
        // value. A value is a function of the attribute's name and type, so two actors declaring
        // one name at one type hold one value.
        let mut probe = ir.commands().values().next()?.clone();
        probe.outcomes.clear();
        probe.examples.clear();
        probe.fixture_inputs.clear();
        probe.response.clear();
        let mut callers = Self {
            first: BTreeMap::new(),
            second: BTreeMap::new(),
        };
        for actor in ir.actors().values() {
            if actor.attributes.is_empty() {
                continue;
            }
            probe.input.clone_from(&actor.attributes);
            let value = |distance: usize| {
                candidates(ir, &probe, &[], Distinction::further(distance))
                    .ok()?
                    .into_iter()
                    .next()
            };
            let name = actor.name.to_string();
            callers.first.insert(name.clone(), value(FIRST)?);
            callers.second.insert(name, value(FIRST + 1)?);
        }
        Some(callers)
    }

    /// The values `actor` carries as `who`.
    fn of_actor(&self, who: Who, actor: &str) -> Option<&BTreeMap<String, Node>> {
        match who {
            Who::First => self.first.get(actor),
            Who::Second => self.second.get(actor),
        }
    }

    /// The values a caller of `command` carries as `who`: those of the actors that may invoke it,
    /// which agree on the type of every attribute the command reads.
    fn for_command(&self, ir: &EssIr, who: Who, command: &QualifiedName) -> BTreeMap<String, Node> {
        let mut values = BTreeMap::new();
        for actor in ir.actors().values() {
            if !actor.may.iter().any(|granted| granted.name() == command) {
                continue;
            }
            for (name, value) in self
                .of_actor(who, &actor.name.to_string())
                .into_iter()
                .flatten()
            {
                values.entry(name.clone()).or_insert_with(|| value.clone());
            }
        }
        values
    }
}

/// The suite for a model whose actors carry attributes (see the module documentation).
pub(super) fn synthesize(ir: &EssIr) -> Synthesis {
    let Some(callers) = Callers::of(ir) else {
        return synthesize_plain(ir, Focus::Whole);
    };
    let reading = reading(ir, &callers);
    let rotated = grant::rotated(ir);
    let mut whole = run(ir, &callers, &|_| Who::First, Focus::Whole);
    let swapped = run(ir, &callers, &|_| Who::Second, Focus::Whole);
    let mut identities = Identities::of(ir, whole.suite.scenarios.values());
    // The model read with one command sent as `second` and every other as `first`, by command:
    // read only for a scenario acting on the one row of a singleton entity.
    let mut acted_on_as_second: BTreeMap<String, Synthesis> = BTreeMap::new();
    for (id, scenario) in &mut whole.suite.scenarios {
        if !sends_any(scenario, &reading) {
            continue;
        }
        let Some(again) = swapped.suite.scenarios.get(id) else {
            continue;
        };
        let Err(exhausted) = append(ir, scenario, again, &mut identities) else {
            continue;
        };
        match one_row_acted_on(
            ir,
            &callers,
            id,
            &exhausted,
            &identities,
            &mut acted_on_as_second,
        ) {
            Some(steps) => scenario.steps = steps,
            None => whole.notes.push(unswapped(id, exhausted)),
        }
    }
    // A command need not read a credential to depend on shared state. In particular, a
    // related-row guard must observe the row arranged by the other principal (#312).
    let mixed_commands: BTreeSet<_> = ir
        .commands()
        .keys()
        .filter(|command| {
            callers.for_command(ir, Who::First, command)
                != callers.for_command(ir, Who::Second, command)
        })
        .cloned()
        .collect();
    for command in &mixed_commands {
        let sent = |name: &QualifiedName| {
            if name == command {
                Who::Second
            } else {
                Who::First
            }
        };
        let back = |name: &QualifiedName| sent(name).other();
        let (other, back) = about_command(ir, &callers, (&sent, &back), command, &rotated);
        for (id, mut scenario) in other.suite.scenarios {
            if !about(&id, command) {
                continue;
            }
            // Preserve an existing single-caller witness unless the mixed plan actually sends
            // another command as a different caller. Merely renaming a principal proves nothing.
            if whole.suite.scenarios.contains_key(&id) && !crosses_callers(&scenario) {
                continue;
            }
            if let Some(again) = back.suite.scenarios.get(&id) {
                identities.take(&scenario.steps);
                if let Err(exhausted) = append(ir, &mut scenario, again, &mut identities) {
                    whole.notes.push(unswapped(&id, exhausted));
                }
            }
            whole
                .refusals
                .retain(|refusal| refusal.scenario.as_ref() != Some(&id));
            whole.suite.scenarios.insert(id, scenario);
        }
    }
    cross_single_command(ir, &callers, &mut whole);
    // Every reading above is synthesized from a rewritten copy of the model, whose digests are not
    // the model's; the suite names the model it came from, the one a target and an adapter digest
    // (beyond10x/ess#216).
    let model = crate::SuiteProvenance::of(ir);
    whole.suite.provenance.spec_digest = model.spec_digest;
    whole.suite.provenance.contract_digest = model.contract_digest;
    whole.suite.select_fresh_format();
    whole
}

/// Upserts and duplicate-identity controls arrange and act with the same command name.
fn cross_single_command(ir: &EssIr, callers: &Callers, whole: &mut Synthesis) {
    let first = ir.with_commands_rewritten(|command| {
        written(
            ir,
            command,
            &callers.for_command(ir, Who::First, &command.name),
        )
    });
    let second = ir.with_commands_rewritten(|command| {
        written(
            ir,
            command,
            &callers.for_command(ir, Who::Second, &command.name),
        )
    });
    let forward = InvocationModels {
        arrangement: &first,
        acting: &second,
        credentials: Some((ir, callers, Who::First)),
        seeds: &super::seeds::EMPTY,
    };
    let backward = InvocationModels {
        arrangement: &second,
        acting: &first,
        credentials: Some((ir, callers, Who::Second)),
        seeds: &super::seeds::EMPTY,
    };
    for command in ir.commands().keys() {
        if callers.for_command(ir, Who::First, command)
            == callers.for_command(ir, Who::Second, command)
        {
            continue;
        }
        let mixed = super::synthesize_invocations(&forward, Focus::About(command));
        let reversed = super::synthesize_invocations(&backward, Focus::About(command));
        for refusal in &mixed.refusals {
            let Some(id) = &refusal.scenario else {
                continue;
            };
            if about(id, command)
                && whole.suite.scenarios.contains_key(id)
                && !mixed.suite.scenarios.get(id).is_some_and(crosses_callers)
            {
                whole.notes.push(Note::CrossCallerUnwitnessed {
                    scenario: id.clone(),
                    reason: format!(
                        "the mixed invocation planner retained the ordinary outcome: {}",
                        refusal.cause
                    ),
                });
            }
        }
        for (id, mut scenario) in mixed.suite.scenarios {
            if !about(&id, command) || !crosses_callers(&scenario) {
                continue;
            }
            whole.notes.retain(|note| {
                !matches!(note,
                    Note::UnswappedCallers { scenario, .. }
                    | Note::CrossCallerUnswapped { scenario, .. }
                    | Note::CrossCallerUnwitnessed { scenario, .. } if scenario == &id
                )
            });
            if let Some(again) = reversed.suite.scenarios.get(&id) {
                if let Err(reason) = append_independent(ir, &mut scenario, again) {
                    whole.notes.push(Note::CrossCallerUnswapped {
                        scenario: id.clone(),
                        reason,
                    });
                }
            } else {
                whole.notes.push(Note::CrossCallerUnswapped { scenario: id.clone(), reason: "the reversed caller interpretation cannot arrange this source-selected outcome" });
            }
            whole
                .refusals
                .retain(|refusal| refusal.scenario.as_ref() != Some(&id));
            whole.refusals.extend(
                mixed
                    .refusals
                    .iter()
                    .filter(|refusal| refusal.scenario.as_ref() == Some(&id))
                    .cloned(),
            );
            whole.suite.scenarios.insert(id, scenario);
        }
    }
}

/// Whether a scenario actually invokes commands under two distinct declared credentials.
fn crosses_callers(scenario: &ConformanceScenario) -> bool {
    let mut first = None;
    for step in &scenario.steps {
        let (ScenarioStep::ExecuteCommand { caller, .. }
        | ScenarioStep::ExecuteCommandWithoutInput { caller, .. }) = step
        else {
            continue;
        };
        if caller.is_empty() {
            continue;
        }
        if let Some(first) = first {
            if first != caller {
                return true;
            }
        } else {
            first = Some(caller);
        }
    }
    false
}

/// Which caller each command is sent as.
type Assignment<'a> = &'a dyn Fn(&QualifiedName) -> Who;

/// The two runs one caller-reading command is read under — sent as the second caller with
/// everything else sent as the first, and the reverse — of which only the scenarios [`about`] the
/// command are kept.
///
/// Each is synthesized for that command's scenarios alone (beyond10x/ess#301): a whole synthesis
/// per caller-reading command made a model with a hundred of them a hundred times slower than one
/// with none. Every family files a command's scenarios from its own loop over that command, so
/// those are the scenarios the whole run writes, but for one exception: a command sent in turn as
/// each actor granted it ([`grant::rotated`]) is sent by turn across every scenario sending it. A
/// kept scenario sending one is read from the whole runs, as before.
fn about_command(
    ir: &EssIr,
    callers: &Callers,
    (sent, back): (Assignment<'_>, Assignment<'_>),
    command: &QualifiedName,
    rotated: &BTreeSet<QualifiedName>,
) -> (Synthesis, Synthesis) {
    #[cfg(test)]
    if tests::UNFOCUSED.get() {
        return (
            run(ir, callers, sent, Focus::Whole),
            run(ir, callers, back, Focus::Whole),
        );
    }
    let focus = Focus::About(command);
    let other = run(ir, callers, sent, focus);
    let back_run = run(ir, callers, back, focus);
    let turned = [&other, &back_run].iter().any(|synthesis| {
        synthesis
            .suite
            .scenarios
            .iter()
            .any(|(id, scenario)| about(id, command) && sends_any(scenario, rotated))
    });
    if turned {
        return (
            run(ir, callers, sent, Focus::Whole),
            run(ir, callers, back, Focus::Whole),
        );
    }
    (other, back_run)
}

/// The model read under one assignment, synthesized, with every command step marked with the
/// caller it is sent as.
fn run(
    ir: &EssIr,
    callers: &Callers,
    who: &dyn Fn(&QualifiedName) -> Who,
    focus: Focus<'_>,
) -> Synthesis {
    #[cfg(test)]
    if focus.is_whole() {
        tests::WHOLE.set(tests::WHOLE.get() + 1);
    }
    let written = ir.with_commands_rewritten(|command| {
        written(
            ir,
            command,
            &callers.for_command(ir, who(&command.name), &command.name),
        )
    });
    let mut synthesis = synthesize_plain(&written, focus);
    for scenario in synthesis.suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            mark(ir, callers, who, step);
        }
    }
    synthesis
}

/// The caller a command step is sent as: the assigned caller's value of every attribute the
/// actor that sends it declares, and nothing where it declares none.
fn mark(
    ir: &EssIr,
    callers: &Callers,
    who: &dyn Fn(&QualifiedName) -> Who,
    step: &mut ScenarioStep,
) {
    let (ScenarioStep::ExecuteCommand {
        command,
        actor,
        caller,
        ..
    }
    | ScenarioStep::ExecuteCommandWithoutInput {
        command,
        actor,
        caller,
    }) = step
    else {
        return;
    };
    let name = QualifiedName::new(command.to_string()).expect("a command reference is a name");
    let declared = actor
        .as_ref()
        .and_then(|actor| {
            ir.actors()
                .values()
                .find(|declared| declared.name.to_string() == actor.to_string())
        })
        .or_else(|| {
            ir.actors()
                .values()
                .find(|declared| declared.may.iter().any(|granted| granted.name() == &name))
        });
    let Some(declared) = declared else {
        return;
    };
    let Some(values) = callers.of_actor(who(&name), &declared.name.to_string()) else {
        return;
    };
    *caller = declared
        .attributes
        .iter()
        .filter_map(|field| {
            values
                .get(&field.name)
                .map(|value| (field.name.clone(), value.clone()))
        })
        .collect();
}

/// Every command that reads the caller, in a value or in a guard: the ones writing a caller's values
/// in changes.
fn reading(ir: &EssIr, callers: &Callers) -> BTreeSet<QualifiedName> {
    ir.commands()
        .values()
        .filter(|command| {
            written(
                ir,
                command,
                &callers.for_command(ir, Who::First, &command.name),
            ) != **command
        })
        .map(|command| command.name.clone())
        .collect()
}

/// `command` with every caller read replaced by `values`: a value source by the literal it holds,
/// and a guard operand by the literal fact. A read `values` has nothing for is left as it is.
fn written(
    ir: &EssIr,
    command: &ResolvedCommand,
    values: &BTreeMap<String, Node>,
) -> ResolvedCommand {
    let mut command = command.clone();
    let stored_roots: Vec<Vec<ResolvedField>> = command
        .outcomes
        .iter()
        .map(|outcome| {
            command
                .selection_subject(outcome)
                .map(|subject| ir.entity(&subject.entity).fields.clone())
                .unwrap_or_default()
        })
        .collect();
    let input = command.input.clone();
    let fact = |roots: &[ResolvedField]| {
        let roots = roots.to_vec();
        move |path: &FactPath| {
            (path.namespace() == CALLER_NAMESPACE
                && path.segments().len() == 2
                && !roots.iter().any(|field| field.name == CALLER_NAMESPACE))
            .then(|| values.get(&path.segments()[1]))
            .flatten()
            .and_then(caller_fact)
        }
    };
    for (outcome, stored) in command.outcomes.iter_mut().zip(&stored_roots) {
        for field in outcome
            .payload
            .iter_mut()
            .flat_map(|payload| payload.fields.iter_mut())
            .chain(outcome.sets.iter_mut())
            .chain(outcome.error_payload.iter_mut())
        {
            write_value(field, values);
        }
        let on_input = fact(&input);
        let on_stored = fact(stored);
        match &mut outcome.condition {
            ResolvedCondition::When { predicate }
            | ResolvedCondition::ExternalWhen { predicate, .. } => {
                *predicate = write_predicate(predicate, &on_input);
            }
            ResolvedCondition::SubjectField { predicate, .. }
            | ResolvedCondition::SubjectState { predicate, .. }
            | ResolvedCondition::StateChange { predicate, .. } => {
                if let Some(predicate) = predicate {
                    *predicate = write_predicate(predicate, &on_input);
                }
            }
            ResolvedCondition::SubjectPredicate { predicate, input } => {
                *predicate = write_predicate(predicate, &on_stored);
                if let Some(input) = input {
                    *input = write_predicate(input, &on_input);
                }
            }
            // The related row is another entity's, which reads no caller; its input guard may.
            ResolvedCondition::Related { input, .. }
            | ResolvedCondition::RelatedSet { input, .. } => {
                if let Some(input) = input {
                    *input = write_predicate(input, &on_input);
                }
            }
            ResolvedCondition::Otherwise
            | ResolvedCondition::External { .. }
            | ResolvedCondition::WrongState
            | ResolvedCondition::UnknownInstance
            | ResolvedCondition::InputAbsent
            | ResolvedCondition::ExistingInstance => {}
        }
    }
    command
}

/// A caller's value as the fact a guard compares, at the attribute's own type: a `Boolean` is a
/// Boolean fact and a number a number, as the stored field or input it is compared with is. The
/// witness builder values each attribute at its declared type, so the node's shape is that type's.
fn caller_fact(node: &Node) -> Option<FactValue> {
    match node {
        Node::Text(text) => Some(FactValue::text(text.clone())),
        Node::Bool(flag) => Some(FactValue::bool(*flag)),
        Node::Number(number) => Some(FactValue::Number(*number)),
        Node::Null | Node::Seq(_) | Node::Map(_) => None,
    }
}

/// One determined field, with a caller read replaced by the literal the caller holds.
fn write_value(field: &mut ResolvedPayloadField, values: &BTreeMap<String, Node>) {
    match &mut field.value {
        ResolvedPayloadValue::CallerAttribute { attribute, .. } => {
            let literal = match values.get(attribute.as_str()) {
                Some(Node::Text(text)) => text.clone(),
                Some(Node::Number(number)) => number.to_string(),
                Some(Node::Bool(flag)) => flag.to_string(),
                _ => return,
            };
            field.value = ResolvedPayloadValue::Literal { value: literal };
        }
        ResolvedPayloadValue::Struct { fields } => {
            for leaf in fields {
                write_value(leaf, values);
            }
        }
        _ => {}
    }
}

/// `predicate` with every operand `fact` answers replaced by that literal.
fn write_predicate(
    predicate: &Predicate,
    fact: &dyn Fn(&FactPath) -> Option<FactValue>,
) -> Predicate {
    let operand = |it: &Operand| match it {
        Operand::Fact(path) => fact(path).map_or_else(|| it.clone(), Operand::Literal),
        Operand::Offset(offset) => fact(&offset.base)
            .and_then(|base| offset.value_at(&base))
            .map_or_else(|| it.clone(), Operand::Literal),
        Operand::Derived(derived) => derived
            .value_with(fact)
            .map_or_else(|| it.clone(), Operand::Literal),
        Operand::Literal(_) => it.clone(),
    };
    match predicate {
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .map(|child| write_predicate(child, fact))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .iter()
                .map(|child| write_predicate(child, fact))
                .collect(),
        ),
        Predicate::Not(child) => Predicate::Not(Box::new(write_predicate(child, fact))),
        Predicate::Compare {
            left,
            op,
            right,
            kind,
        } => Predicate::Compare {
            kind: *kind,
            left: operand(left),
            op: *op,
            right: operand(right),
        },
        other => other.clone(),
    }
}

/// Whether the scenario sends any of `commands`.
fn sends_any(scenario: &ConformanceScenario, commands: &BTreeSet<QualifiedName>) -> bool {
    scenario.steps.iter().any(|step| match step {
        ScenarioStep::ExecuteCommand { command, .. }
        | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => commands
            .iter()
            .any(|name| name.to_string() == command.to_string()),
        _ => false,
    })
}

/// Whether the scenario `id` names is about a branch of `command`.
fn about(id: &ScenarioId, command: &QualifiedName) -> bool {
    let of = match id {
        ScenarioId::Outcome { outcome } => &outcome.command,
        ScenarioId::Transition { by, .. } => &by.command,
        ScenarioId::Refusal { command, .. } => command,
        ScenarioId::Invariant { after, .. } => &after.command,
        _ => return false,
    };
    of.to_string() == command.to_string()
}

/// Compose an independent public arrangement, preserving the first run's observations and
/// identities. Unlike the optional caller variant, a required disclosure cell must name any
/// arrangement that cannot be appended, including globally incompatible view assertions.
pub(super) fn append_independent(
    ir: &EssIr,
    scenario: &mut ConformanceScenario,
    again: &ConformanceScenario,
) -> Result<(), &'static str> {
    // A row established by setup carries its literal identity, which is never renamed or drawn
    // afresh (beyond10x/ess#413): a second run would establish the same row twice.
    if established(&scenario.steps) || established(&again.steps) {
        return Err(
            "a row established by setup cannot be established again for the reversed callers",
        );
    }
    let mut identities = Identities::of(ir, [&*scenario, again]);
    let before = scenario.steps.len();
    append(ir, scenario, again, &mut identities)
        .map_err(|_| "the follow-up arrangement exhausted its fresh identity witnesses")?;
    if scenario.steps.len() == before {
        return Err("the follow-up assertions cannot compose with the retained origin state");
    }
    scenario.source.extend(again.source.iter().cloned());
    Ok(())
}

/// `again`'s steps after `scenario`'s own, with every instance and instant it binds renamed apart
/// from the first run's and every caller-supplied identity it sends drawn afresh, where both runs
/// can share one scenario. A swapped run left out because no fresh identity could be drawn for it
/// is answered as [`Exhausted`], which the caller names in a note or answers otherwise.
fn append(
    ir: &EssIr,
    scenario: &mut ConformanceScenario,
    again: &ConformanceScenario,
    identities: &mut Identities<'_>,
) -> Result<(), Exhausted> {
    let once = |steps: &[ScenarioStep]| {
        established(steps)
            || steps.iter().any(|step| {
                matches!(
                    step,
                    ScenarioStep::ResolveFixtures { .. }
                        | ScenarioStep::CaptureCommandResult { .. }
                        | ScenarioStep::ExpectReplayResult { .. }
                        | ScenarioStep::CheckPeriodic { .. }
                )
            })
    };
    if once(&scenario.steps) || once(&again.steps) || reads_every_row(ir, &again.steps) {
        return Ok(());
    }
    let drawn = identities.drawn(&scenario.steps, &again.steps)?;
    let Ok(mut value) = serde_json::to_value(&again.steps) else {
        return Ok(());
    };
    rename(&mut value);
    redraw(&mut value, &identities.keys, &drawn);
    let Some(value) = recaptured(ir, value) else {
        return Ok(());
    };
    if let Ok(steps) = serde_json::from_value::<Vec<ScenarioStep>>(value) {
        scenario.steps.extend(steps);
    }
    Ok(())
}

/// Whether `steps` establish a row by setup: its literal identity can be established only once in a
/// scenario, so a second run of them cannot be appended.
fn established(steps: &[ScenarioStep]) -> bool {
    steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::EstablishEntity { .. }))
}

/// The note for a swapped run left out of the scenario `id`.
fn unswapped(
    id: &ScenarioId,
    Exhausted {
        input, type_ref, ..
    }: Exhausted,
) -> Note {
    Note::UnswappedCallers {
        scenario: id.clone(),
        input,
        type_ref,
    }
}

/// The run that stands for both caller orders of a scenario acting on the one row of a singleton
/// entity it did not create (beyond10x/ess#287): every step sent as `first` but the command under
/// test's, sent as `second` — the row arranged by one caller and acted on by the other — read from
/// the model under that assignment, `acted_on_as_second` caching it by command.
///
/// A swapped run would arrange the row a second time, which the one row cannot be, and draws no
/// other identity; with it left out, a target keeping one row per caller passed every scenario.
/// Two runs in one scenario cannot both start from no row, so this one replaces the first, and
/// still sends both callers. `None` — the note — where the identity is the one the command under
/// test creates, which has no row to act on, or where that assignment has no such scenario.
fn one_row_acted_on(
    ir: &EssIr,
    callers: &Callers,
    id: &ScenarioId,
    exhausted: &Exhausted,
    identities: &Identities<'_>,
    acted_on_as_second: &mut BTreeMap<String, Synthesis>,
) -> Option<Vec<ScenarioStep>> {
    if !exhausted.one_value {
        return None;
    }
    let command = under_test(id)?;
    let creates_it = identities
        .creating
        .get(&command)
        .is_some_and(|(_, fields)| fields.contains(&exhausted.input));
    if creates_it {
        return None;
    }
    let mixed = acted_on_as_second
        .entry(command.clone())
        .or_insert_with(|| {
            run(
                ir,
                callers,
                &|name| {
                    if name.to_string() == command {
                        Who::Second
                    } else {
                        Who::First
                    }
                },
                Focus::Whole,
            )
        });
    mixed
        .suite
        .scenarios
        .get(id)
        .map(|scenario| scenario.steps.clone())
}

/// The command the scenario `id` names is about.
pub(super) fn under_test(id: &ScenarioId) -> Option<String> {
    match id {
        ScenarioId::Outcome { outcome } => Some(outcome.command.to_string()),
        ScenarioId::Transition { by, .. } => Some(by.command.to_string()),
        ScenarioId::Refusal { command, .. } => Some(command.to_string()),
        ScenarioId::Invariant { after, .. } => Some(after.command.to_string()),
        _ => None,
    }
}

/// Whether a run observes all matching rows: a set effect, an aggregate view (a count or sum per
/// group), or a view expectation that counts, ranks or positions rows. Appended after the first run, such an expectation would be
/// read over both runs' rows and its absolute figures would be the first run's plus its own.
fn reads_every_row(ir: &EssIr, steps: &[ScenarioStep]) -> bool {
    steps.iter().any(|step| {
        if let ScenarioStep::ExpectOutcome { outcome } = step {
            return ir.commands().values().any(|command| {
                command.name.to_string() == outcome.command.to_string()
                    && command.outcomes.iter().any(|branch| {
                        branch.name == outcome.outcome
                            && (branch.instances.is_some() || !branch.affects.is_empty())
                    })
            });
        }
        let (ScenarioStep::ExpectView {
            view, expectation, ..
        }
        | ScenarioStep::EventuallyView {
            view, expectation, ..
        }) = step
        else {
            return false;
        };
        let Some(declared) = ir
            .views()
            .iter()
            .find(|(name, _)| name.to_string() == view.to_string())
            .map(|(_, declared)| declared)
        else {
            return false;
        };
        let identity = &ir.entity(&declared.source).identity.name;
        declared.is_aggregate()
            || match expectation {
                ViewExpectation::Counts { .. }
                | ViewExpectation::Ranked { .. }
                | ViewExpectation::At { .. } => true,
                ViewExpectation::Excludes { fields } => !names_a_row(fields, identity),
                _ => false,
            }
    })
}

/// Whether an `excludes` names one row: by the identity field, or by a value a step bound to an
/// instance. Without either — "no row with this text" — it is read over every row the view holds,
/// and the first run's rows answer it.
fn names_a_row(fields: &BTreeMap<String, ScenarioValue>, identity: &str) -> bool {
    fields.iter().any(|(name, value)| {
        name == identity
            || matches!(
                value,
                ScenarioValue::Instance { .. } | ScenarioValue::Observed { .. }
            )
    })
}

/// A serialized run with every `observed` reference turned into an instance it captures itself.
///
/// The runner reads `{kind: observed, event, field}` from the **first** occurrence of the event in
/// the scenario, which in a second run is the first run's. So the second run captures the field
/// right after the step that first expects the event of its own command, and reads that instead.
/// `None` where a reference comes before any such step, or the field names no entity to capture.
fn recaptured(ir: &EssIr, value: serde_json::Value) -> Option<serde_json::Value> {
    let serde_json::Value::Array(steps) = value else {
        return None;
    };
    let mut wanted: BTreeSet<(String, String)> = BTreeSet::new();
    for step in &steps {
        observed(step, &mut wanted);
    }
    let mut captured: BTreeMap<(String, String), String> = BTreeMap::new();
    let mut out = Vec::with_capacity(steps.len());
    for mut step in steps {
        let mut used = BTreeSet::new();
        observed(&step, &mut used);
        if used.iter().any(|pair| !captured.contains_key(pair)) {
            return None;
        }
        replace_observed(&mut step, &captured);
        // Either expectation form: an event carrying a captured identity is expected through
        // `expect_event_values` (beyond10x/ess#273).
        let expected = matches!(
            step.get("step").and_then(serde_json::Value::as_str),
            Some("expect_event" | "expect_event_values")
        )
        .then(|| {
            step.get("event")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .flatten();
        out.push(step);
        let Some(event) = expected else {
            continue;
        };
        for (seen, field) in &wanted {
            if *seen != event || captured.contains_key(&(seen.clone(), field.clone())) {
                continue;
            }
            let entity = ir
                .entities()
                .values()
                .find(|entity| entity.identity.name == *field)
                .or_else(|| {
                    ir.entities()
                        .values()
                        .find(|entity| entity.fields.iter().any(|held| held.name == *field))
                })?;
            let instance = format!("observed-{}-{SWAPPED}", captured.len() + 1);
            out.push(serde_json::json!({
                "step": "capture_instance",
                "instance": instance,
                "entity": entity.name.to_string(),
                "event": seen,
                "field": field,
            }));
            captured.insert((seen.clone(), field.clone()), instance);
        }
    }
    Some(serde_json::Value::Array(out))
}

/// Every `(event, field)` an `observed` reference in `value` reads.
fn observed(value: &serde_json::Value, found: &mut BTreeSet<(String, String)>) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(serde_json::Value::as_str) == Some("observed") {
                if let (Some(event), Some(field)) = (
                    map.get("event").and_then(serde_json::Value::as_str),
                    map.get("field").and_then(serde_json::Value::as_str),
                ) {
                    found.insert((event.to_owned(), field.to_owned()));
                }
            }
            map.values().for_each(|inner| observed(inner, found));
        }
        serde_json::Value::Array(items) => items.iter().for_each(|inner| observed(inner, found)),
        _ => {}
    }
}

/// `value` with every `observed` reference `captured` holds read from its captured instance.
fn replace_observed(value: &mut serde_json::Value, captured: &BTreeMap<(String, String), String>) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(serde_json::Value::as_str) == Some("observed") {
                let pair = (
                    map.get("event").and_then(serde_json::Value::as_str),
                    map.get("field").and_then(serde_json::Value::as_str),
                );
                if let (Some(event), Some(field)) = pair {
                    if let Some(instance) = captured.get(&(event.to_owned(), field.to_owned())) {
                        *value = serde_json::json!({"kind": "instance", "instance": instance});
                        return;
                    }
                }
            }
            map.values_mut()
                .for_each(|inner| replace_observed(inner, captured));
        }
        serde_json::Value::Array(items) => items
            .iter_mut()
            .for_each(|inner| replace_observed(inner, captured)),
        _ => {}
    }
}

/// The caller-supplied identities of the suite, and the fresh ones swapped runs are given
/// (beyond10x/ess#275).
///
/// A swapped run is synthesized from the same witnesses as the first run, so it sends the same
/// literal for an input that becomes a created identity; appended to the first run, it would send
/// that identity a second time and still expect the creation, which an `existing_instance:`
/// refusal answers instead. Each such literal is replaced, throughout the swapped run, by a value
/// no scenario sends and no other swapped run was given, so the run stays consistent with itself —
/// a refusal that sends its own run's stored identity again still sends it, an event field or a
/// stored field that copies it expects the copy — and apart from every other row a target the
/// scenarios share may hold.
///
/// An identity a command addresses without creating it is drawn afresh the same way
/// (beyond10x/ess#465): a target may arrange the addressed record itself — under an external
/// control, whose means it chooses — and the first run's literal would name a record the first run
/// already had arranged.
struct Identities<'a> {
    ir: &'a EssIr,
    /// Every command creating an identity from input, by name, with the inputs that become it.
    creating: BTreeMap<String, (&'a ResolvedCommand, BTreeSet<String>)>,
    /// Every command taking an identity from input, by name, with those inputs: the ones that
    /// become a created identity ([`Self::creating`]) and the ones a branch names as the instance
    /// it acts on ([`supplied_instances`]).
    inputs: BTreeMap<String, (&'a ResolvedCommand, BTreeSet<String>)>,
    /// Every declared field, input, event field and view column of an identity input's type, and
    /// every name the model copies an identity under whatever its type ([`derived`]): where a
    /// serialized run may carry a copy of an identity that is not text (see [`redraw`]).
    keys: BTreeSet<String>,
    /// Every identity a scenario sends or a swapped run was given, serialized.
    taken: BTreeSet<String>,
    /// The next witness past [`SWAPPED_IDENTITY`] to try.
    next: usize,
}

/// Why a swapped run could not be given a fresh identity: the input and its type, and whether that
/// type has one value — the one row of a singleton entity (beyond10x/ess#287).
struct Exhausted {
    input: String,
    type_ref: String,
    one_value: bool,
}

impl<'a> Identities<'a> {
    fn of<'s>(ir: &'a EssIr, scenarios: impl IntoIterator<Item = &'s ConformanceScenario>) -> Self {
        let mut creating = BTreeMap::new();
        let mut inputs = BTreeMap::new();
        let mut types = BTreeSet::new();
        for command in ir.commands().values() {
            let created: BTreeSet<String> = command
                .outcomes
                .iter()
                .filter_map(super::existence::identity_input)
                .map(str::to_owned)
                .collect();
            let mut fields = created.clone();
            fields.extend(supplied_instances(command));
            if fields.is_empty() {
                continue;
            }
            types.extend(
                command
                    .input
                    .iter()
                    .filter(|input| fields.contains(&input.name))
                    .map(|input| input.type_ref.to_string()),
            );
            if !created.is_empty() {
                creating.insert(command.name.to_string(), (command, created));
            }
            inputs.insert(command.name.to_string(), (command, fields));
        }
        let declared = ir
            .commands()
            .values()
            .flat_map(|command| &command.input)
            .chain(ir.events().values().flat_map(|event| &event.fields))
            .chain(
                ir.entities()
                    .values()
                    .flat_map(|entity| std::iter::once(&entity.identity).chain(&entity.fields)),
            )
            .chain(
                ir.views()
                    .values()
                    .flat_map(|view| view.fields.iter().chain(&view.params)),
            );
        let mut keys: BTreeSet<String> = declared
            .filter(|field| types.contains(&field.type_ref.to_string()))
            .map(|field| field.name.clone())
            .collect();
        keys.extend(derived(ir, &inputs));
        keys.extend(member_parameters(ir, &types));
        let mut identities = Self {
            ir,
            creating,
            inputs,
            keys,
            taken: BTreeSet::new(),
            next: 0,
        };
        for scenario in scenarios {
            identities.take(&scenario.steps);
        }
        identities
    }

    /// Every caller-supplied identity `steps` send as a literal, to a command that creates it or
    /// to one that addresses it.
    fn sent<'s>(&self, steps: &'s [ScenarioStep]) -> Vec<Sent<'a, 's>> {
        let mut sent = Vec::new();
        for step in steps {
            let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
                continue;
            };
            let Some((sending, fields)) = self.inputs.get(&command.to_string()) else {
                continue;
            };
            let literals: BTreeMap<String, Node> = input
                .iter()
                .filter_map(|(name, value)| match value {
                    ScenarioValue::Literal { value } => Some((name.clone(), value.clone())),
                    _ => None,
                })
                .collect();
            for (name, value) in input {
                if let ScenarioValue::Literal { value } = value {
                    if fields.contains(name) && !matches!(value, Node::Null) {
                        sent.push(Sent {
                            command: sending,
                            field: name,
                            value,
                            input: literals.clone(),
                        });
                    }
                }
            }
        }
        sent
    }

    /// Records every caller-supplied identity `steps` send as taken.
    fn take(&mut self, steps: &[ScenarioStep]) {
        let keys: Vec<String> = self
            .sent(steps)
            .into_iter()
            .map(|sent| key(sent.value))
            .collect();
        self.taken.extend(keys);
    }

    /// A fresh value for every caller-supplied identity `again` sends, by the serialized value it
    /// replaces; [`Exhausted`] where the type has too few values to keep one apart from every
    /// identity taken, in which case the scenario keeps its first run only.
    fn drawn(
        &mut self,
        first: &[ScenarioStep],
        again: &[ScenarioStep],
    ) -> Result<BTreeMap<String, serde_json::Value>, Exhausted> {
        let mut drawn = BTreeMap::new();
        let sent = self.sent(again);
        let mut avoid: BTreeSet<String> = self
            .sent(first)
            .iter()
            .chain(&sent)
            .map(|sent| key(sent.value))
            .collect();
        for one in &sent {
            let old = key(one.value);
            if drawn.contains_key(&old) {
                continue;
            }
            let (command, field) = (one.command, one.field);
            let exhausted = || Exhausted {
                input: field.to_owned(),
                type_ref: command
                    .input
                    .iter()
                    .find(|input| input.name == field)
                    .map(|input| input.type_ref.to_string())
                    .unwrap_or_default(),
                one_value: super::singleton::names_the_one_row(self.ir, command, field),
            };
            // Every step of the run sending this identity, each of which must take the branch it
            // took with the old one.
            let sending: Vec<&Sent<'_, '_>> = sent
                .iter()
                .filter(|other| key(other.value) == old)
                .collect();
            let fresh = self
                .draw(command, field, one.value, &avoid, &sending)
                .ok_or_else(exhausted)?;
            let new = key(&fresh);
            avoid.insert(new.clone());
            self.taken.insert(new);
            // A struct identity is replaced whole, and each member leaf by its fresh member too,
            // wherever a member is copied on its own — a view parameter compared with
            // `slot.shelf`, an event field set from it (beyond10x/ess#430).
            for (from, to) in member_leaves(one.value, &fresh) {
                if let Ok(to) = serde_json::to_value(to) {
                    drawn.entry(key(from)).or_insert(to);
                }
            }
            drawn.insert(old, serde_json::to_value(&fresh).map_err(|_| exhausted())?);
        }
        Ok(drawn)
    }

    /// The first value of `field` that no identity taken or in `avoid` carries and that every step
    /// in `sending` takes its branch with ([`super::keeps_branch`]): far witnesses first, past every
    /// witness an arrangement numbers, then the near ones a bounded type's witnesses are spread
    /// over — the witness builder answers every far distance of a narrow range with one value —
    /// then the values inside the guards that read it ([`super::guided_values`]), where a far
    /// witness would take another branch.
    ///
    /// A struct is fresh only where every member differs from the member it replaces
    /// (beyond10x/ess#430): a member copied on its own must not name the first run's row.
    fn draw(
        &mut self,
        command: &ResolvedCommand,
        field: &str,
        old: &Node,
        avoid: &BTreeSet<String>,
        sending: &[&Sent<'_, '_>],
    ) -> Option<Node> {
        let far: Vec<Distinction> = (0..SWAPPED_TRIES)
            .map(|offset| Distinction::further(SWAPPED_IDENTITY + self.next + offset))
            .collect();
        self.next += SWAPPED_TRIES;
        let near = (0..=MAX_CANDIDATES).map(Distinction::further);
        let witnesses = far.into_iter().chain(near).filter_map(|at| {
            candidates(self.ir, command, &[], at)
                .ok()
                .and_then(|inputs| inputs.into_iter().next())
                .and_then(|mut input| input.remove(field))
        });
        let guided = super::guided_values(self.ir, command, field);
        for value in witnesses.chain(guided) {
            let at = key(&value);
            if self.taken.contains(&at) || avoid.contains(&at) || !fresh_everywhere(old, &value) {
                continue;
            }
            if sending.iter().all(|sent| {
                super::keeps_branch(self.ir, sent.command, &sent.input, sent.field, &value)
            }) {
                return Some(value);
            }
        }
        None
    }
}

/// One caller-supplied identity a step sends as a literal: the command, the input, the value, and
/// every literal input of the step, against which a fresh value must keep the branch.
struct Sent<'a, 's> {
    command: &'a ResolvedCommand,
    field: &'s str,
    value: &'s Node,
    input: BTreeMap<String, Node>,
}

/// Every name the model copies a caller-supplied identity under, by where its value comes from
/// rather than by the type it lands at (beyond10x/ess#275): an event field or a stored field a
/// branch of a command taking it from input sets from that input, created or addressed
/// (beyond10x/ess#465) — through a declared conversion into a plain `Integer`, or into an
/// `Optional` of the identity's type — and, transitively, an event field or a stored field any
/// branch sets from such a stored field. A view shows a stored field under its own name, so the
/// stored names cover the view rows too.
fn derived(
    ir: &EssIr,
    inputs: &BTreeMap<String, (&ResolvedCommand, BTreeSet<String>)>,
) -> BTreeSet<String> {
    fn copies(
        entry: &ResolvedPayloadField,
        from_input: &BTreeSet<String>,
        from_stored: &BTreeSet<String>,
        found: &mut BTreeSet<String>,
    ) {
        match &entry.value {
            ResolvedPayloadValue::InputField { field, .. }
            | ResolvedPayloadValue::InputOrGenerated { field, .. }
                if from_input.contains(field)
                    // A member of the identity input, read by an input path (ess/22, A4): the
                    // field copies that member (beyond10x/ess#430).
                    || field
                        .split_once('.')
                        .is_some_and(|(root, _)| from_input.contains(root)) =>
            {
                found.insert(entry.target.clone());
            }
            ResolvedPayloadValue::SubjectField { field, .. } if from_stored.contains(field) => {
                found.insert(entry.target.clone());
            }
            ResolvedPayloadValue::Struct { fields } => {
                for leaf in fields {
                    copies(leaf, from_input, from_stored, found);
                }
            }
            _ => {}
        }
    }
    let entries = |command: &ResolvedCommand| -> Vec<ResolvedPayloadField> {
        command
            .outcomes
            .iter()
            .flat_map(|outcome| {
                outcome
                    .payload
                    .iter()
                    .flat_map(|payload| payload.fields.iter())
                    .chain(&outcome.sets)
            })
            .cloned()
            .collect()
    };
    let mut found = BTreeSet::new();
    for (command, fields) in inputs.values() {
        for entry in entries(command) {
            copies(&entry, fields, &BTreeSet::new(), &mut found);
        }
    }
    let none = BTreeSet::new();
    loop {
        let before = found.len();
        let stored = found.clone();
        for command in ir.commands().values() {
            for entry in entries(command) {
                copies(&entry, &none, &stored, &mut found);
            }
        }
        if found.len() == before {
            return found;
        }
    }
}

/// Every input a branch of `command` names as the instance it acts on (`moves:`, `updates:`,
/// `deletes:` with `instance:` an input), whether or not another branch creates it: the identity
/// the command addresses (beyond10x/ess#465).
fn supplied_instances(command: &ResolvedCommand) -> impl Iterator<Item = String> + '_ {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.subject.as_ref()?.instance {
            ResolvedInstance::Supplied { field } => Some(field.name.clone()),
            ResolvedInstance::Observed { .. } => None,
        })
}

/// Every view parameter the filter compares with a member of its row's identity, where that
/// identity is of a type a caller supplies (`types`): a parameter sent one member of a struct
/// identity (beyond10x/ess#428), which a swapped run sends the fresh member in (beyond10x/ess#430).
fn member_parameters(ir: &EssIr, types: &BTreeSet<String>) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for view in ir.views().values() {
        let identity = &ir.entity(&view.source).identity;
        if !types.contains(&identity.type_ref.to_string()) {
            continue;
        }
        for param in &view.params {
            let member =
                super::identity::compared_with(ir, view, &param.name).is_some_and(|field| {
                    field
                        .split_once('.')
                        .is_some_and(|(root, _)| root == identity.name)
                });
            if member {
                names.insert(param.name.clone());
            }
        }
    }
    names
}

/// Whether `new` differs from `old` in every member leaf, where both are structs; any two values
/// otherwise (beyond10x/ess#430).
fn fresh_everywhere(old: &Node, new: &Node) -> bool {
    match (old, new) {
        (Node::Map(before), Node::Map(after)) => before.iter().all(|(name, held)| {
            after
                .get(name)
                .is_none_or(|replaced| fresh_everywhere(held, replaced))
        }),
        (Node::Map(_), _) | (_, Node::Map(_)) => true,
        (before, after) => before != after,
    }
}

/// Each scalar leaf of the struct `old` beside the leaf at the same path of `new`, where both are
/// structs; nothing otherwise (beyond10x/ess#430).
fn member_leaves<'n>(old: &'n Node, new: &'n Node) -> Vec<(&'n Node, &'n Node)> {
    let (Node::Map(before), Node::Map(after)) = (old, new) else {
        return Vec::new();
    };
    before
        .iter()
        .filter_map(|(name, held)| after.get(name).map(|replaced| (held, replaced)))
        .flat_map(|(held, replaced)| match (held, replaced) {
            (Node::Map(_), Node::Map(_)) => member_leaves(held, replaced),
            (Node::Text(_) | Node::Number(_) | Node::Bool(_), _) => vec![(held, replaced)],
            _ => Vec::new(),
        })
        .collect()
}

/// A value, serialized, as the set of taken identities holds it.
fn key(value: &Node) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// `value` with every identity `drawn` names replaced by its fresh value, wherever the serialized
/// run carries it: in an input, an event payload, a view row or an expectation.
///
/// Replace values only under names declared at an identity's type or derived from its source.
/// A String witness can equal a schema field name (`item_id`); rewriting that metadata would make
/// a capture read a nonexistent event field. Counts and unrelated scalar fields likewise stay put.
fn redraw(
    value: &mut serde_json::Value,
    keys: &BTreeSet<String>,
    drawn: &BTreeMap<String, serde_json::Value>,
) {
    if drawn.is_empty() {
        return;
    }
    redraw_under(value, false, keys, drawn);
}

/// [`redraw`] of `value`, which a name in `keys` holds where `typed`.
fn redraw_under(
    value: &mut serde_json::Value,
    typed: bool,
    keys: &BTreeSet<String>,
    drawn: &BTreeMap<String, serde_json::Value>,
) {
    // A drawn struct identity is replaced whole, before its members are read as anything else
    // (beyond10x/ess#430): `key` and `Value::to_string` serialize one struct identically
    // (`tests/caller_struct_identity.rs`).
    if typed && value.is_object() {
        if let Some(new) = drawn.get(&value.to_string()) {
            *value = new.clone();
            return;
        }
    }
    match value {
        serde_json::Value::Object(map) => {
            // A literal scenario value is the value it wraps, under the name that holds it.
            let literal = map.get("kind").and_then(serde_json::Value::as_str) == Some("literal");
            for (name, inner) in map.iter_mut() {
                let typed = keys.contains(name) || (literal && typed && name == "value");
                redraw_under(inner, typed, keys, drawn);
            }
        }
        serde_json::Value::Array(items) => items
            .iter_mut()
            .for_each(|inner| redraw_under(inner, typed, keys, drawn)),
        serde_json::Value::String(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::Bool(_) => {
            if typed {
                if let Some(new) = drawn.get(&value.to_string()) {
                    *value = new.clone();
                }
            }
        }
        serde_json::Value::Null => {}
    }
}

/// Every instance and instant name in a serialized run, suffixed.
fn rename(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, inner) in map.iter_mut() {
                match inner {
                    serde_json::Value::String(name) if key == "instance" || key == "instant" => {
                        *name = format!("{name}-{SWAPPED}");
                    }
                    _ => rename(inner),
                }
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(rename),
        _ => {}
    }
}

/// The caller-synthesis work guards (beyond10x/ess#301): how many whole syntheses a model with
/// caller-reading commands costs, and that reading a command's scenarios from a run written for
/// that command alone gives the suite the whole runs gave.
#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use ess_compiler::resolve::compile;
    use ess_compiler::source::SourceMap;
    use ess_domain::spec::{RawSpecFile, Specification};
    use ess_domain::system::Source;

    use super::*;

    thread_local! {
        /// How many whole syntheses of an assignment ran.
        pub(super) static WHOLE: Cell<usize> = const { Cell::new(0) };
        /// Whether every per-command run is a whole synthesis, as before beyond10x/ess#301.
        pub(super) static UNFOCUSED: Cell<bool> = const { Cell::new(false) };
    }

    /// Two caller-reading commands, one of them sent in its scenarios only after a command its two
    /// unattributed holders are sent in turn as.
    const NOTES: &str = "format: ess/16
system: demo
version: v1
summary: Caller values beside a command sent in turn.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}
  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
    lifecycle:
      initial: Open
      states: [Open, Tagged, Closed]
      terminal: [Closed]
      transitions:
        - {name: tag, from: [Open], to: Tagged}
        - {name: close, from: [Tagged], to: Closed}
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.CloseNote]
  - name: demo.notes.Clerk
    may: [demo.notes.AddTag]
  - name: demo.notes.Auditor
    may: [demo.notes.AddTag]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {note_id: {generated: true}, account_id: {caller: account_id}, text: input.text}
  - name: demo.notes.AddTag
    input:
      - {name: note_id, type: demo.notes.NoteId}
    outcomes:
      - name: tagged
        moves: demo.notes.Note.tag
        instance: note_id
        emits: [demo.notes.NoteTagged]
        payload:
          demo.notes.NoteTagged: {note_id: input.note_id}
  - name: demo.notes.CloseNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: closed
        moves: demo.notes.Note.close
        instance: note_id
        emits: [demo.notes.NoteClosed]
        payload:
          demo.notes.NoteClosed: {note_id: input.note_id}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: text, type: String}
  - name: demo.notes.NoteTagged
    fields:
      - {name: note_id, type: demo.notes.NoteId}
  - name: demo.notes.NoteClosed
    fields:
      - {name: note_id, type: demo.notes.NoteId}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
      - {name: state, type: demo.notes.Note.State}
";

    /// Serves the commands, so `AddTag` is sent in turn as `Clerk` and `Auditor`.
    const SERVED: &str = "components:
  - component: notes-service
    owns: {domains: [demo.notes]}
    accepts: {commands: [demo.notes.CreateNote, demo.notes.AddTag, demo.notes.CloseNote]}
    publishes: {events: [demo.notes.NoteCreated, demo.notes.NoteTagged, demo.notes.NoteClosed]}
    reached_by: network
";

    fn compiled(text: &str) -> EssIr {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
        let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("{errors}"));
        compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
    }

    /// The synthesis, and how many whole syntheses of an assignment it ran.
    fn counted(ir: &EssIr, unfocused: bool) -> (Synthesis, usize) {
        WHOLE.set(0);
        UNFOCUSED.set(unfocused);
        let synthesis = super::super::synthesize(ir);
        UNFOCUSED.set(false);
        (synthesis, WHOLE.get())
    }

    #[test]
    fn a_source_owned_outcome_keeps_its_mixed_arrangement_explanation() {
        let synthesis = super::super::synthesize(&compiled(NOTES));
        assert!(
            synthesis.notes.iter().any(|note| matches!(note,
                Note::CrossCallerUnwitnessed { scenario, reason }
                    if scenario.to_string() == "demo.notes.CloseNote/outcome/closed"
                        && reason.contains("mixed")
            )),
            "the source-owned accepting outcome must not silently claim a mixed witness"
        );
        assert!(
            synthesis
                .suite
                .scenarios
                .contains_key(&"demo.notes.CloseNote/outcome/forbidden".parse().unwrap()),
            "the other caller's source-selected refusal remains executable"
        );
    }

    #[test]
    fn a_caller_reading_command_costs_no_whole_synthesis_of_its_own() {
        let ir = compiled(NOTES);
        let callers = Callers::of(&ir).expect("the model reads callers");
        assert_eq!(
            reading(&ir, &callers).len(),
            2,
            "two commands read the caller"
        );
        assert!(grant::rotated(&ir).is_empty(), "nothing is served");

        let (synthesis, whole) = counted(&ir, false);
        assert!(!synthesis.suite.scenarios.is_empty());
        assert_eq!(
            whole, 2,
            "the model is synthesized whole under all-first and all-second only, not once more \
             per caller-reading command"
        );
    }

    #[test]
    fn a_command_sent_in_turn_reads_its_scenarios_from_the_whole_runs() {
        let ir = compiled(&format!("{NOTES}{SERVED}"));
        let rotated = grant::rotated(&ir);
        assert_eq!(
            rotated.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["demo.notes.AddTag"]
        );

        let (_, whole) = counted(&ir, false);
        // All-first, all-second, and the pair for `CloseNote`, whose scenarios send `AddTag`.
        assert_eq!(whole, 4);
    }

    #[test]
    fn a_run_written_for_one_command_gives_the_suite_the_whole_runs_gave() {
        for text in [NOTES.to_owned(), format!("{NOTES}{SERVED}")] {
            let ir = compiled(&text);
            let (focused, _) = counted(&ir, false);
            let (unfocused, whole) = counted(&ir, true);
            assert_eq!(
                whole,
                2 + 2 * 2,
                "the reference runs every assignment whole"
            );
            assert!(!focused.suite.scenarios.is_empty());
            assert_eq!(
                focused.suite.to_canonical_json(),
                unfocused.suite.to_canonical_json()
            );
            assert_eq!(focused, unfocused);
        }
    }
}
