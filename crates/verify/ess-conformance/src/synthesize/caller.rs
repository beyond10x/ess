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

mod arranged;

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EssIr, ResolvedCommand, ResolvedCondition, ResolvedField, ResolvedInstance, ResolvedOutcome,
    ResolvedPayloadField, ResolvedPayloadValue, ResolvedRelatedVia, ResolvedRowSetTest,
    ResolvedSubject,
};
use ess_domain::command::caller_value::CALLER_NAMESPACE;
use ess_domain::command::subject_fact::INPUT_NAMESPACE;
use ess_domain::name::QualifiedName;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use super::{grant, synthesize_plain, Focus, Note, Synthesis};
use crate::scenario::{
    ConformanceScenario, EventRef, InstanceName, ScenarioId, ScenarioStep, ScenarioValue,
    ViewExpectation,
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

/// Marks every command step of a scenario built after the caller readings, from the model itself,
/// with the caller the suite sends every command as, `first`: a one-time outcome's disclosure
/// scenarios (beyond10x/ess#550). A step already sent as a caller keeps it, and a model no command
/// of which reads its caller is left alone, so its suite keeps its bytes.
pub(super) fn send_as_first(ir: &EssIr, scenario: &mut ConformanceScenario) {
    if !uses(ir) {
        return;
    }
    let Some(callers) = Callers::of(ir) else {
        return;
    };
    if reading(ir, &callers).is_empty() {
        return;
    }
    for step in &mut scenario.steps {
        if matches!(step, ScenarioStep::ExecuteCommand { caller, .. } | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } if !caller.is_empty())
        {
            continue;
        }
        mark(ir, &callers, &|_| Who::First, step);
    }
}

/// The model read under the two assignments a scenario built after the caller readings may send
/// a command under: a one-time outcome's follow-up cells. `None` for a model no command of which
/// reads its caller, whose cells keep being built from the model itself.
///
/// A follow-up branch selected by its caller (`owner != caller.user_id`) has no input the plain
/// model can choose for it: the caller is no value there. The ordinary suite reaches it under an
/// assignment ([`synthesize`]), and so does a cell, sending every command as `first`, and then
/// arranging as `first` and acting as `second`, the refusal for a caller who is not the row's.
pub(super) struct Readings {
    callers: Callers,
    first: EssIr,
    second: EssIr,
}

impl Readings {
    pub(super) fn of(ir: &EssIr) -> Option<Self> {
        if !uses(ir) {
            return None;
        }
        let callers = Callers::of(ir)?;
        if reading(ir, &callers).is_empty() {
            return None;
        }
        let read_as = |who: Who| {
            ir.with_commands_rewritten(|command| {
                written(ir, command, &callers.for_command(ir, who, &command.name))
            })
        };
        let first = read_as(Who::First);
        let second = read_as(Who::Second);
        Some(Self {
            callers,
            first,
            second,
        })
    }

    /// Every command sent as `first`, its steps left for [`send_as_first`] to mark; then the
    /// arrangement sent as `first` and the command under test as `second`, marked as they are
    /// planned.
    pub(super) fn models<'a>(&'a self, ir: &'a EssIr) -> [InvocationModels<'a>; 2] {
        [
            InvocationModels::plain(&self.first),
            InvocationModels {
                arrangement: &self.first,
                acting: &self.second,
                credentials: Some((ir, &self.callers, Who::First)),
                seeds: &super::seeds::EMPTY,
            },
        ]
    }
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
        let Err(exhausted) = append(ir, (Some(id), scenario), again, &mut identities) else {
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
                if let Err(exhausted) =
                    append(ir, (Some(&id), &mut scenario), again, &mut identities)
                {
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
    arranged_by_callers(ir, &callers, &reading, &mut identities, &mut whole);
    // Every reading above is synthesized from a rewritten copy of the model, whose digests are not
    // the model's; the suite names the model it came from, the one a target and an adapter digest
    // (beyond10x/ess#216).
    let model = crate::SuiteProvenance::of(ir);
    whole.suite.provenance.spec_digest = model.spec_digest;
    whole.suite.provenance.contract_digest = model.contract_digest;
    whole.suite.select_fresh_format_for(ir);
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
                if let Err(reason) = append_independent_about(ir, (Some(&id), &mut scenario), again)
                {
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

/// Every scenario the readings above left refused, synthesized from the model whose arranging
/// commands are sent as callers chosen for the rows they arrange ([`arranged`]), with its run under
/// the swapped callers appended as for any other scenario. Last, so a scenario another reading
/// witnesses keeps that witness and its bytes.
fn arranged_by_callers(
    ir: &EssIr,
    callers: &Callers,
    reading: &BTreeSet<QualifiedName>,
    identities: &mut Identities<'_>,
    whole: &mut Synthesis,
) {
    let mut any = false;
    for command in arranged::refused_commands(ir, whole) {
        let Some((model, lifted)) = arranged::lifted_model(ir, callers, &|_| Who::First, &command)
        else {
            continue;
        };
        let first = synthesize_plain(&model, Focus::About(&command));
        let held: BTreeSet<ScenarioId> = whole.suite.scenarios.keys().cloned().collect();
        any |= arranged::adopt(whole, first, &command, &lifted, &|step| {
            mark(ir, callers, &|_| Who::First, step);
        });
        let adopted: Vec<ScenarioId> = whole
            .suite
            .scenarios
            .keys()
            .filter(|id| !held.contains(*id))
            .cloned()
            .collect();
        if adopted.is_empty() {
            continue;
        }
        let swapped = arranged::lifted_model(ir, callers, &|_| Who::Second, &command).map(
            |(model, lifted)| {
                let mut again = synthesize_plain(&model, Focus::About(&command));
                arranged::finish(&mut again, &lifted, &|step| {
                    mark(ir, callers, &|_| Who::Second, step);
                });
                again
            },
        );
        for id in adopted {
            let scenario = whole.suite.scenarios.get_mut(&id).expect("adopted above");
            identities.take(&scenario.steps);
            if !sends_any(scenario, reading) {
                continue;
            }
            let Some(again) = swapped
                .as_ref()
                .and_then(|swapped| swapped.suite.scenarios.get(&id))
            else {
                continue;
            };
            if let Err(exhausted) = append(ir, (Some(&id), scenario), again, identities) {
                whole.notes.push(unswapped(&id, exhausted));
            }
        }
    }
    // A refusal of the grant withheld for want of an accepting send may have one now.
    if any {
        super::grant::denied_since(
            ir,
            &mut whole.suite,
            &mut whole.refusals,
            &mut whole.notes,
            &|step| mark(ir, callers, &|_| Who::First, step),
        );
    }
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

/// The attributes of its caller `command` reads, in a value or in a guard, by name: each one
/// [`written`] replaces when it is given a value for it alone. An authored act states these
/// (`caller:`, ess-scenario/5).
pub(crate) fn reads(ir: &EssIr, command: &QualifiedName) -> BTreeSet<String> {
    let Some(resolved) = ir.commands().get(command) else {
        return BTreeSet::new();
    };
    let declared: BTreeSet<&String> = ir
        .actors()
        .values()
        .flat_map(|actor| actor.attributes.iter().map(|attribute| &attribute.name))
        .collect();
    declared
        .into_iter()
        .filter(|attribute| {
            let probe = BTreeMap::from([((*attribute).clone(), Node::Text(String::new()))]);
            written(ir, resolved, &probe) != *resolved
        })
        .cloned()
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
            ResolvedCondition::SubjectPredicate { predicate, input }
            | ResolvedCondition::ExternalSubject {
                predicate, input, ..
            } => {
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
        ScenarioId::Outcome { outcome } | ScenarioId::HeldOutcome { outcome, .. } => {
            &outcome.command
        }
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
    append_independent_about(ir, (None, scenario), again)
}

/// [`append_independent`] for a disclosure cell: where a set effect of `again` would select a row
/// the origin's run left ([`shared_selection`]), the refusal names its filter, not the origin
/// state.
pub(super) fn append_followup(
    ir: &EssIr,
    scenario: &mut ConformanceScenario,
    again: &ConformanceScenario,
) -> Result<(), String> {
    let appended = append_independent(ir, scenario, again);
    appended.map_err(|reason| match reason {
        UNCOMPOSED => {
            shared_selection(ir, &scenario.steps, &again.steps).unwrap_or_else(|| reason.to_owned())
        }
        _ => reason.to_owned(),
    })
}

/// [`append_independent`] for the scenario `id` names, where given: the fresh values must keep its
/// command under test on the branch the first run took (beyond10x/ess#672).
fn append_independent_about(
    ir: &EssIr,
    (id, scenario): (Option<&ScenarioId>, &mut ConformanceScenario),
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
    append(ir, (id, scenario), again, &mut identities)
        .map_err(|_| "the follow-up arrangement exhausted its fresh identity witnesses")?;
    if scenario.steps.len() == before {
        return Err(UNCOMPOSED);
    }
    scenario.source.extend(again.source.iter().cloned());
    Ok(())
}

/// `again`'s steps after `scenario`'s own, with every instance and instant it binds renamed apart
/// from the first run's and every caller-supplied identity it sends drawn afresh, where both runs
/// can share one scenario. A swapped run left out because no fresh identity could be drawn for it
/// is answered as [`Exhausted`], which the caller names in a note or answers otherwise. `id`, where
/// given, names the scenario, whose command under test the fresh values must keep on its branch.
fn append(
    ir: &EssIr,
    (id, scenario): (Option<&ScenarioId>, &mut ConformanceScenario),
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
    // A set effect selecting only rows `again` arranges reads none of the first run's rows.
    if once(&scenario.steps)
        || once(&again.steps)
        || observes_every_row(ir, &again.steps)
        || shared_selection(ir, &scenario.steps, &again.steps).is_some()
    {
        return Ok(());
    }
    // The branches selection tries up to the one under test, which it reaches only past every
    // earlier one; a later branch's guard decides nothing where that one is selected.
    let tested = id.and_then(under_test).and_then(|name| {
        let command = ir
            .commands()
            .values()
            .find(|command| command.name.to_string() == name)?;
        let reached = match id {
            Some(ScenarioId::Outcome { outcome } | ScenarioId::HeldOutcome { outcome, .. }) => {
                command
                    .outcomes
                    .iter()
                    .position(|branch| branch.name == outcome.outcome)
                    .map_or(command.outcomes.len(), |at| at + 1)
            }
            _ => command.outcomes.len(),
        };
        Some((command, &command.outcomes[..reached]))
    });
    let drawn = identities.drawn(&scenario.steps, &again.steps, tested)?;
    let Ok(mut value) = serde_json::to_value(&again.steps) else {
        return Ok(());
    };
    rename(&mut value);
    redraw(&mut value, &identities.keys, &drawn);
    let Some(value) = recaptured(ir, value) else {
        return Ok(());
    };
    if let Ok(mut steps) = serde_json::from_value::<Vec<ScenarioStep>>(value) {
        cumulative(&scenario.steps, &mut steps);
        scenario.steps.extend(steps);
    }
    Ok(())
}

/// A run's exact counts, appended after the `first` run's steps, counted over both runs.
///
/// An invocation count (`expect_invocation` with `count:`) and a publication count are read over
/// every invocation or publication under the scenario's correlation, the first run's included: a
/// second run expecting its own one invocation observes two. Each count is raised by the last one
/// the first run expects of the same binding, command and input filter, or of the same event,
/// which that run observed for its whole window.
fn cumulative(first: &[ScenarioStep], steps: &mut [ScenarioStep]) {
    for step in steps {
        let earlier = match &*step {
            ScenarioStep::ExpectInvocation {
                binding,
                command,
                input,
                count: Some(_),
            } => first.iter().rev().find_map(|prior| match prior {
                ScenarioStep::ExpectInvocation {
                    binding: by,
                    command: invoked,
                    input: with,
                    count: Some(count),
                } if by == binding && invoked == command && with == input => Some(*count),
                _ => None,
            }),
            ScenarioStep::ExpectPublicationCount { event, .. } => {
                first.iter().rev().find_map(|prior| match prior {
                    ScenarioStep::ExpectPublicationCount {
                        event: published,
                        count,
                    } if published == event => Some(*count),
                    _ => None,
                })
            }
            _ => None,
        };
        let Some(earlier) = earlier else {
            continue;
        };
        if let ScenarioStep::ExpectInvocation {
            count: Some(count), ..
        }
        | ScenarioStep::ExpectPublicationCount { count, .. } = step
        {
            *count = count.saturating_add(earlier.get());
        }
    }
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
        ScenarioId::Outcome { outcome } | ScenarioId::HeldOutcome { outcome, .. } => {
            Some(outcome.command.to_string())
        }
        ScenarioId::Transition { by, .. } => Some(by.command.to_string()),
        ScenarioId::Refusal { command, .. } => Some(command.to_string()),
        ScenarioId::Invariant { after, .. } => Some(after.command.to_string()),
        _ => None,
    }
}

/// Whether a run observes all matching rows: a set effect, an aggregate view (a count or sum per
/// group), or a view expectation that counts, ranks or positions rows. Appended after the first run, such an expectation would be
/// read over both runs' rows and its absolute figures would be the first run's plus its own.
pub(super) fn reads_every_row(ir: &EssIr, steps: &[ScenarioStep]) -> bool {
    steps.iter().any(|step| set_effect(ir, step).is_some()) || observes_every_row(ir, steps)
}

/// The detail that refuses to compose a disclosure cell whose follow-up assertions were not
/// appended for a reason no other refusal names.
const UNCOMPOSED: &str = "the follow-up assertions cannot compose with the retained origin state";

/// The command and the branch with a set effect, `instances:` or `affects:`, that `step` expects.
fn set_effect<'a>(
    ir: &'a EssIr,
    step: &ScenarioStep,
) -> Option<(&'a ResolvedCommand, &'a ResolvedOutcome)> {
    let ScenarioStep::ExpectOutcome { outcome } = step else {
        return None;
    };
    ir.commands()
        .values()
        .filter(|command| command.name.to_string() == outcome.command.to_string())
        .find_map(|command| {
            command
                .outcomes
                .iter()
                .find(|branch| {
                    branch.name == outcome.outcome
                        && (branch.instances.is_some() || !branch.affects.is_empty())
                })
                .map(|branch| (command, branch))
        })
}

/// The set effect `again` takes whose filter may select a row `first` left, as a refusal names
/// it; `None` where every set effect `again` takes selects only rows `again` arranges.
///
/// An `instances:` filter selects only the rows of the run that sends it where it pins a stored
/// field to an input (`owner == input.owner`, alone or as one conjunct of an `and`) and every send
/// of its command in `again` carries, for one such input, a literal that `first` carries nowhere:
/// no row `first` left can hold that value. An `affects:` filter, and an `instances:` filter that
/// pins nothing so, reads every row of its entity.
fn shared_selection(ir: &EssIr, first: &[ScenarioStep], again: &[ScenarioStep]) -> Option<String> {
    let first = serde_json::to_value(first).ok();
    again.iter().find_map(|step| {
        let (command, branch) = set_effect(ir, step)?;
        let named = format!("{}/{}", command.name, branch.name);
        let Some(subject) = branch
            .instances
            .as_ref()
            .filter(|_| branch.affects.is_empty())
        else {
            return Some(format!(
                "the set effect of `{named}` selects rows by `affects:`, which reads every row \
                 of its entity, so it would also select the rows the origin's run left"
            ));
        };
        let pinned = pinned_inputs(&subject.filter);
        let own = !pinned.is_empty()
            && again.iter().all(|send| match send {
                ScenarioStep::ExecuteCommand {
                    command: sent,
                    input,
                    ..
                } if sent.to_string() == command.name.to_string() => pinned.iter().any(|name| {
                    matches!(
                        input.get(*name),
                        Some(ScenarioValue::Literal { value })
                            if serde_json::to_value(value)
                                .is_ok_and(|value| {
                                    first.as_ref().is_some_and(|first| !carries(first, &value))
                                })
                    )
                }),
                ScenarioStep::ExecuteCommandWithoutInput { command: sent, .. } => {
                    sent.to_string() != command.name.to_string()
                }
                _ => true,
            });
        if !own {
            return Some(format!(
                "the set effect of `{named}` selects `instances: {{where: {}}}`, which pins no \
                 stored field to a value only the follow-up's run sends, so it would also select \
                 the rows the origin's run left",
                subject.filter
            ));
        }
        (!changes_rows(ir, again, command, branch)).then(|| {
            format!(
                "the set effect of `{named}` selects `instances: {{where: {}}}`, and the \
                 follow-up's run arranges no row it selects, so the run would witness no row it \
                 changes",
                subject.filter
            )
        })
    })
}

/// Whether the run `steps` witnesses the set effect (`instances:`) of `branch`: a send of
/// `command` expecting `branch` finds a row its filter selects among the rows the steps before it
/// leave ([`super::row_set::changed_states`]), and every send after which an event it publishes
/// once per changed row is expected finds one resting where that event is published. A run that
/// selects no row publishes no per-row event, so a run expecting one would fail every target
/// honouring the specification.
fn changes_rows(
    ir: &EssIr,
    steps: &[ScenarioStep],
    command: &ResolvedCommand,
    branch: &ResolvedOutcome,
) -> bool {
    let Some(set) = &branch.instances else {
        return false;
    };
    let mut witnessed = false;
    for (at, step) in steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand {
            command: named,
            input,
            ..
        } = step
        else {
            continue;
        };
        let expected = matches!(
            steps.get(at + 1),
            Some(ScenarioStep::ExpectOutcome { outcome }) if outcome.outcome == branch.name
        );
        if named.to_string() != command.name.to_string() || !expected {
            continue;
        }
        // Rows an earlier step changed unfollowed, such as an earlier set effect, are none known.
        let states = super::row_set::changed_states(ir, &steps[..at], command, branch, input)
            .unwrap_or_default();
        witnessed |= !states.is_empty();
        let answered = steps[at + 1..].iter().take_while(|step| {
            !matches!(
                step,
                ScenarioStep::ExecuteCommand { .. }
                    | ScenarioStep::ExecuteCommandWithoutInput { .. }
            )
        });
        for step in answered {
            let (ScenarioStep::ExpectEvent { event, .. }
            | ScenarioStep::ExpectEventValues { event, .. }
            | ScenarioStep::EventuallyEvent { event, .. }) = step
            else {
                continue;
            };
            let Some(per_row) = set
                .events
                .iter()
                .find(|entry| EventRef::from(&entry.event) == *event)
            else {
                continue;
            };
            let published = states.iter().any(|state| {
                per_row.when_state.is_empty()
                    || per_row.when_state.iter().any(|held| held == state.as_str())
            });
            if !published {
                return false;
            }
        }
    }
    witnessed
}

/// `followup`, the run a disclosure cell appends for `branch` of `command`, where it witnesses the
/// branch's set effect ([`changes_rows`]); else the branch's own scenario in `suite`, which arranges
/// the rows its filter selects before it sends the command, where that one does. A follow-up that
/// selects no row it arranged witnesses nothing the effect does (a downstream adopter's report of
/// 2026-10-26). A branch with no set effect keeps its follow-up.
pub(super) fn witnessing_followup(
    ir: &EssIr,
    suite: &crate::ConformanceSuite,
    (command, branch): (&ResolvedCommand, &ResolvedOutcome),
    followup: ConformanceScenario,
) -> ConformanceScenario {
    if branch.instances.is_none() || changes_rows(ir, &followup.steps, command, branch) {
        return followup;
    }
    let id = ScenarioId::Outcome {
        outcome: crate::scenario::OutcomeRef::new(
            crate::scenario::CommandRef::new(command.name.clone()),
            branch.name.clone(),
        ),
    };
    suite
        .scenario(&id)
        .filter(|own| changes_rows(ir, &own.steps, command, branch))
        .cloned()
        .unwrap_or(followup)
}

/// The inputs `filter` pins a stored field to: each `<field> == input.<name>`, either way round,
/// that is the filter or one conjunct of it.
fn pinned_inputs(filter: &Predicate) -> Vec<&str> {
    fn input(operand: &Operand) -> Option<&str> {
        match operand {
            Operand::Fact(path) => match path.segments() {
                [root, name] if root == INPUT_NAMESPACE => Some(name.as_str()),
                _ => None,
            },
            _ => None,
        }
    }
    let conjuncts = match filter {
        Predicate::All(children) => children.as_slice(),
        other => std::slice::from_ref(other),
    };
    let stored =
        |operand: &Operand| matches!(operand, Operand::Fact(path) if path.segments().len() == 1);
    conjuncts
        .iter()
        .filter_map(|conjunct| match conjunct {
            Predicate::Compare {
                left,
                op: CompareOp::Eq,
                right,
                ..
            } if stored(left) => input(right),
            Predicate::Compare {
                left,
                op: CompareOp::Eq,
                right,
                ..
            } if stored(right) => input(left),
            _ => None,
        })
        .collect()
}

/// Whether `value` is `within` or any value nested in it.
fn carries(within: &serde_json::Value, value: &serde_json::Value) -> bool {
    within == value
        || match within {
            serde_json::Value::Array(items) => items.iter().any(|item| carries(item, value)),
            serde_json::Value::Object(fields) => fields.values().any(|field| carries(field, value)),
            _ => false,
        }
}

/// Whether a run's view expectations observe all matching rows: an aggregate view, or an
/// expectation that counts, ranks or positions rows ([`reads_every_row`] without set effects).
fn observes_every_row(ir: &EssIr, steps: &[ScenarioStep]) -> bool {
    steps.iter().any(|step| {
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
///
/// So is every input a guard compares with stored rows (beyond10x/ess#580): a row-set selector, a
/// `when_related:` row and its test read the rows the first run left, so a name unique across
/// callers, sent again, finds the first run's row and takes the refusal. One rule holds every
/// input a swapped run sends that a guard, a selector or an identity reads; the caller's own
/// attributes, which the swap already exchanges, are not inputs.
struct Identities<'a> {
    ir: &'a EssIr,
    /// Every command creating an identity from input, by name, with the inputs that become it.
    creating: BTreeMap<String, (&'a ResolvedCommand, BTreeSet<String>)>,
    /// Every command taking an identity from input, by name, with those inputs: the ones that
    /// become a created identity ([`Self::creating`]) and the ones a branch names as the instance
    /// it acts on ([`supplied_instances`]), and the ones a guard compares with stored rows
    /// ([`selected`]).
    inputs: BTreeMap<String, (&'a ResolvedCommand, BTreeSet<String>)>,
    /// The struct inputs of [`Self::inputs`] drawn afresh only in some members, by command and
    /// input, with those members' paths ([`selected`]); every other input is drawn whole.
    members: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
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
        let selected_fields = row_fields_read(ir);
        let mut guarded_names = BTreeSet::new();
        let mut members: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
        for command in ir.commands().values() {
            let created: BTreeSet<String> = command
                .outcomes
                .iter()
                .filter_map(super::existence::identity_input)
                .map(str::to_owned)
                .collect();
            let mut fields = created.clone();
            fields.extend(supplied_instances(command));
            types.extend(
                command
                    .input
                    .iter()
                    .filter(|input| fields.contains(&input.name))
                    .map(|input| input.type_ref.to_string()),
            );
            // Every input a guard compares with the rows the first run left, and every input
            // written where such a guard reads (beyond10x/ess#580): drawn afresh like an identity,
            // and redrawn under its own name, not under every name its type is declared at.
            let compared = selected(command, &selected_fields);
            guarded_names.extend(compared.keys().cloned());
            for (name, read) in &compared {
                if let (Some(paths), false) = (read, fields.contains(name)) {
                    members
                        .entry(command.name.to_string())
                        .or_default()
                        .insert(name.clone(), paths.clone());
                }
            }
            fields.extend(compared.into_keys());
            if fields.is_empty() {
                continue;
            }
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
        keys.extend(guarded_names);
        keys.extend(derived(ir, &inputs));
        keys.extend(member_parameters(ir, &types));
        let mut identities = Self {
            ir,
            creating,
            inputs,
            members,
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
    ///
    /// Where the scenario is about a branch of `tested`, a fresh value must also leave every guard
    /// of `tested` reading a row the run writes it into, and that `tested` addresses in the run,
    /// deciding as before ([`keeps_stored_branch`], beyond10x/ess#672).
    fn drawn(
        &mut self,
        first: &[ScenarioStep],
        again: &[ScenarioStep],
        tested: Option<(&ResolvedCommand, &[ResolvedOutcome])>,
    ) -> Result<BTreeMap<String, serde_json::Value>, Exhausted> {
        let tested =
            tested.map(|(command, branches)| Tested::of(self.ir, command, branches, again));
        let tested = tested.as_ref();
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
            let members = self
                .members
                .get(&command.name.to_string())
                .and_then(|inputs| inputs.get(field))
                .cloned();
            let fresh = self
                .draw(
                    (command, field, one.value),
                    members.as_ref(),
                    &avoid,
                    &sending,
                    tested,
                )
                .ok_or_else(exhausted)?;
            let new = key(&fresh);
            avoid.insert(new.clone());
            self.taken.insert(new);
            // A struct identity is replaced whole, and each member leaf by its fresh member too,
            // wherever a member is copied on its own — a view parameter compared with
            // `slot.shelf`, an event field set from it (beyond10x/ess#430). A member kept as it was
            // is no replacement (beyond10x/ess#672).
            for (from, to) in member_leaves(one.value, &fresh) {
                if from == to {
                    continue;
                }
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
    /// (beyond10x/ess#430): a member copied on its own must not name the first run's row. A struct
    /// drawn afresh only in `members` keeps every other member of `old`, and is fresh where those
    /// members are (beyond10x/ess#672).
    fn draw(
        &mut self,
        (command, field, old): (&ResolvedCommand, &str, &Node),
        members: Option<&BTreeSet<String>>,
        avoid: &BTreeSet<String>,
        sending: &[&Sent<'_, '_>],
        tested: Option<&Tested<'_>>,
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
            let (value, fresh) = if let Some(paths) = members {
                let Some(value) = only_members(old, &value, paths) else {
                    continue;
                };
                let fresh = paths.iter().all(|path| {
                    match (at_member(old, path), at_member(&value, path)) {
                        (Some(before), Some(after)) => fresh_everywhere(before, after),
                        _ => true,
                    }
                });
                (value, fresh)
            } else {
                let fresh = fresh_everywhere(old, &value);
                (value, fresh)
            };
            let at = key(&value);
            if self.taken.contains(&at) || avoid.contains(&at) || !fresh {
                continue;
            }
            if sending.iter().all(|sent| {
                super::keeps_branch(self.ir, sent.command, &sent.input, sent.field, &value)
                    && tested
                        .is_none_or(|tested| keeps_stored_branch(self.ir, tested, sent, &value))
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

/// Every stored field a row-set selector or its `forall:` test reads, by the selected entity's
/// name (beyond10x/ess#580): a value written there by the first run is one a swapped run's guard
/// may find.
fn row_fields_read(ir: &EssIr) -> BTreeMap<String, BTreeSet<String>> {
    let mut read: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for outcome in ir.commands().values().flat_map(|command| &command.outcomes) {
        let ResolvedCondition::RelatedSet {
            selection, test, ..
        } = &outcome.condition
        else {
            continue;
        };
        let forall = match test {
            ResolvedRowSetTest::Forall(predicate) => Some(predicate),
            _ => None,
        };
        let fields = std::iter::once(&selection.filter)
            .chain(forall)
            .flat_map(Predicate::fact_paths)
            .filter_map(|path| match path.segments() {
                [field] => Some(field.clone()),
                _ => None,
            });
        read.entry(selection.entity.name().to_string())
            .or_default()
            .extend(fields);
    }
    read
}

/// Every input of `command` a swapped run must draw afresh because a guard compares it with stored
/// rows that do not belong to the run (beyond10x/ess#580), whatever its type: the inputs a row-set
/// selector or its `forall:` test reads, the input a `when_related:` guard names its row by, and
/// every input a branch writes into a field a row-set selector reads (`selected`,
/// [`row_fields_read`]), so a row a run arranges is still the row its guard finds. A guard reading
/// the caller is not among them: its value is the swapped caller's already.
///
/// Each input maps to the members of it that are read, by path below the input: `None` where the
/// whole input is, and the member paths where only a member of a struct input is written into a
/// selected field — the rest of the struct is the arranged value and is kept (beyond10x/ess#672,
/// N24). The inputs a `when_related:` test reads are not among them (N25): the row they are
/// compared with is the one the guard names by its input, which the run draws or arranges itself,
/// so the value the first run tied to that row's stored field is still the tie. Unless that stored
/// field is one a row-set selector reads: a run writes it from a value drawn afresh, and the tied
/// input is drawn with it.
fn selected(
    command: &ResolvedCommand,
    selected: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, Option<BTreeSet<String>>> {
    fn whole(fields: &mut BTreeMap<String, Option<BTreeSet<String>>>, name: &str) {
        fields.insert(name.to_owned(), None);
    }
    fn member(fields: &mut BTreeMap<String, Option<BTreeSet<String>>>, root: &str, path: &str) {
        if let Some(paths) = fields
            .entry(root.to_owned())
            .or_insert_with(|| Some(BTreeSet::new()))
        {
            paths.insert(path.to_owned());
        }
    }
    let declared = |name: &str| command.input.iter().any(|input| input.name == name);
    let mut fields = BTreeMap::new();
    for outcome in &command.outcomes {
        match &outcome.condition {
            ResolvedCondition::RelatedSet {
                selection, test, ..
            } => {
                for read in super::row_set::input_reads(&selection.filter) {
                    whole(&mut fields, read);
                }
                if let ResolvedRowSetTest::Forall(predicate) = test {
                    for read in super::row_set::input_reads(predicate) {
                        whole(&mut fields, read);
                    }
                }
            }
            ResolvedCondition::Related { via, also, .. } => {
                // From ess/28 every row an `all:` branch lists.
                for via in std::iter::once(via).chain(also.iter().map(|row| &row.via)) {
                    if let ResolvedRelatedVia::Input { field, .. } = via {
                        whole(&mut fields, field);
                    }
                }
                // A test comparing an input with a stored field a row-set selector reads is the
                // exception to N25: a run writes that field from a value it draws afresh, so the
                // tied input is drawn with it, under the same old value.
                for (_, entity, predicate) in outcome.condition.related_predicates() {
                    let Some(read) = selected.get(&entity.name().to_string()) else {
                        continue;
                    };
                    let ties_selected = predicate
                        .fact_paths()
                        .iter()
                        .any(|path| matches!(path.segments(), [field] if read.contains(field)));
                    if ties_selected {
                        for read in super::row_set::input_reads(predicate) {
                            whole(&mut fields, read);
                        }
                    }
                }
            }
            _ => {}
        }
        let Some(subject) = &outcome.subject else {
            continue;
        };
        let Some(read) = selected.get(&subject.entity.name().to_string()) else {
            continue;
        };
        for entry in &outcome.sets {
            if !read.contains(&entry.target) {
                continue;
            }
            if let ResolvedPayloadValue::InputField { field, .. }
            | ResolvedPayloadValue::InputOrGenerated { field, .. } = &entry.value
            {
                match field.split_once('.') {
                    Some((root, path)) => member(&mut fields, root, path),
                    None => whole(&mut fields, field),
                }
            }
        }
    }
    fields.retain(|field, _| declared(field));
    fields
}

/// The value at the member `path` (dotted, below the struct) of `value`.
fn at_member<'n>(value: &'n Node, path: &str) -> Option<&'n Node> {
    path.split('.').try_fold(value, |node, name| match node {
        Node::Map(members) => members.get(name),
        _ => None,
    })
}

/// `old` with each member `paths` names, where `old` holds one, replaced by the member `drawn`
/// holds at the same path: a struct input redrawn only where a guard reads it, its other members
/// the arranged ones (beyond10x/ess#672). `None` where `drawn` lacks a member `old` holds.
fn only_members(old: &Node, drawn: &Node, paths: &BTreeSet<String>) -> Option<Node> {
    let mut merged = old.clone();
    for path in paths {
        if at_member(old, path).is_none() {
            continue;
        }
        let fresh = at_member(drawn, path)?.clone();
        let mut node = &mut merged;
        for name in path.split('.') {
            let Node::Map(members) = node else {
                return None;
            };
            node = members.get_mut(name)?;
        }
        *node = fresh;
    }
    Some(merged)
}

/// The command under test of the scenario a swapped run is appended to, the branches whose guards
/// decide the one under test, and the input of every step of that run sending it: what a
/// stored-row guard of it reads is the row those inputs address.
struct Tested<'t> {
    branches: &'t [ResolvedOutcome],
    /// Each step's input, every value the literal it is or names: an instance a step captured is
    /// the identity its creating step sent, where that step sent it as a literal. `None` where it
    /// is neither.
    inputs: Vec<BTreeMap<&'t str, Option<&'t Node>>>,
}

impl<'t> Tested<'t> {
    fn of(
        ir: &EssIr,
        command: &ResolvedCommand,
        branches: &'t [ResolvedOutcome],
        steps: &'t [ScenarioStep],
    ) -> Self {
        let mut captured: BTreeMap<&InstanceName, &Node> = BTreeMap::new();
        let mut last: Option<(&ResolvedCommand, &BTreeMap<String, ScenarioValue>)> = None;
        let mut inputs = Vec::new();
        for step in steps {
            match step {
                ScenarioStep::ExecuteCommand {
                    command: sent,
                    input,
                    ..
                } => {
                    last = ir
                        .commands()
                        .values()
                        .find(|known| known.name.to_string() == sent.to_string())
                        .map(|known| (known, input));
                    if sent.to_string() == command.name.to_string() {
                        inputs.push(
                            input
                                .iter()
                                .map(|(name, value)| {
                                    let value = match value {
                                        ScenarioValue::Literal { value } => Some(value),
                                        ScenarioValue::Instance { instance } => {
                                            captured.get(instance).copied()
                                        }
                                        _ => None,
                                    };
                                    (name.as_str(), value)
                                })
                                .collect(),
                        );
                    }
                }
                ScenarioStep::CaptureInstance {
                    instance,
                    event,
                    field,
                    ..
                } => {
                    let Some((creating, input)) = last else {
                        continue;
                    };
                    let from = creating.outcomes.iter().find_map(|outcome| {
                        let subject = outcome.subject.as_ref()?;
                        let ResolvedInstance::Observed {
                            event: published,
                            field: carried,
                        } = &subject.instance
                        else {
                            return None;
                        };
                        (EventRef::from(published) == *event && carried.name == *field)
                            .then(|| instance_input(outcome, subject))
                            .flatten()
                    });
                    if let Some(ScenarioValue::Literal { value }) =
                        from.and_then(|name| input.get(name))
                    {
                        captured.insert(instance, value);
                    }
                }
                _ => {}
            }
        }
        Tested { branches, inputs }
    }

    /// Whether a step of the run sending the command under test may address `row` in `by`: where
    /// either is unknown — the row's identity is not a literal, the guard reads a row through a
    /// stored field, or a step sends no literal there — it may.
    fn addresses(&self, row: Option<&Node>, by: Option<&str>) -> bool {
        let (Some(row), Some(by)) = (row, by) else {
            return true;
        };
        self.inputs.iter().any(|input| {
            input
                .get(by)
                .is_none_or(|value| value.is_none_or(|value| value == row))
        })
    }
}

/// The input field a branch takes the identity of the row it acts on from: named by `instance:`
/// directly, or carried by the created row's event field, which the payload fills from input.
fn instance_input<'o>(
    outcome: &'o ResolvedOutcome,
    subject: &'o ResolvedSubject,
) -> Option<&'o str> {
    match &subject.instance {
        ResolvedInstance::Supplied { field } => Some(&field.name),
        ResolvedInstance::Observed { event, field } => outcome
            .payload
            .iter()
            .filter(|payload| &payload.event == event)
            .flat_map(|payload| &payload.fields)
            .find(|filled| filled.target == field.name)
            .and_then(|filled| match &filled.value {
                ResolvedPayloadValue::InputField { field, .. }
                | ResolvedPayloadValue::InputOrGenerated { field, .. } => Some(field.as_str()),
                _ => None,
            }),
    }
}

/// Whether the redraw of `sent`'s value to `new` leaves every guard of the command under test
/// that reads a stored row deciding as it did (beyond10x/ess#672): a stored field `sent`'s command
/// writes from the value, or from a member of it, is read by a `when_related:` test or a
/// stored-subject guard of the command under test over the same entity, on the row a step sending
/// it addresses, and that guard decides the field's old value and its new one alike, read alone. A
/// guard the field alone leaves undecided both times is kept: it is the scenario's other values
/// that decide it. A row the command under test never addresses — another row of the same entity
/// the run arranges — decides nothing of its branch.
fn keeps_stored_branch(ir: &EssIr, tested: &Tested<'_>, sent: &Sent<'_, '_>, new: &Node) -> bool {
    let (field, old) = (sent.field, sent.value);
    for outcome in &sent.command.outcomes {
        let Some(subject) = &outcome.subject else {
            continue;
        };
        let written = subject.entity.name().to_string();
        let entity = ir.entity(&subject.entity);
        let row = instance_input(outcome, subject).and_then(|name| sent.input.get(name));
        for entry in &outcome.sets {
            let (ResolvedPayloadValue::InputField { field: from, .. }
            | ResolvedPayloadValue::InputOrGenerated { field: from, .. }) = &entry.value
            else {
                continue;
            };
            let (before, after) = match from.split_once('.') {
                Some((root, path)) if root == field => (at_member(old, path), at_member(new, path)),
                None if from == field => (Some(old), Some(new)),
                _ => continue,
            };
            if before == after {
                continue;
            }
            let decide = |predicate: &Predicate, value: Option<&Node>| {
                let mut row = BTreeMap::new();
                if let Some(value) = value {
                    row.insert(entry.target.clone(), value.clone());
                }
                let Ok(facts) = crate::input::bind(
                    ir,
                    &entity.fields,
                    &row,
                    crate::input::Completeness::Partial,
                ) else {
                    return None;
                };
                Some(predicate.evaluate(&crate::input::TypedFacts::new(ir, &entity.fields, facts)))
            };
            let guards = tested.branches.iter().flat_map(|guarded| {
                let related = guarded
                    .condition
                    .related_predicates()
                    .into_iter()
                    .filter(|(_, over, _)| over.name().to_string() == written)
                    .map(|(via, _, predicate)| {
                        let by = match via {
                            ResolvedRelatedVia::Input { field, .. } => Some(field.as_str()),
                            ResolvedRelatedVia::Subject { .. } => None,
                        };
                        (by, predicate)
                    });
                let stored = match (&guarded.condition, &guarded.subject) {
                    (ResolvedCondition::SubjectPredicate { predicate, .. }, Some(on))
                        if on.entity.name().to_string() == written =>
                    {
                        let by = match &on.instance {
                            ResolvedInstance::Supplied { field } => Some(field.name.as_str()),
                            ResolvedInstance::Observed { .. } => None,
                        };
                        Some((by, predicate))
                    }
                    _ => None,
                };
                related.chain(stored)
            });
            for (by, guard) in guards {
                let reads = guard
                    .fact_paths()
                    .iter()
                    .any(|path| path.segments().first() == Some(&entry.target));
                if reads
                    && tested.addresses(row, by)
                    && decide(guard, before) != decide(guard, after)
                {
                    return false;
                }
            }
        }
    }
    true
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
