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
    EssIr, ResolvedCommand, ResolvedCondition, ResolvedField, ResolvedPayloadField,
    ResolvedPayloadValue,
};
use ess_domain::command::caller_value::CALLER_NAMESPACE;
use ess_domain::name::QualifiedName;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate};

use super::{synthesize_plain, Synthesis};
use crate::scenario::{
    ConformanceScenario, ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation,
};
use crate::witness::{candidates, Distinction};

/// How far the two callers' values sit from the plain witness: past every further instance an
/// arrangement numbers, and short of [`Distinction::UNKNOWN`], so no input witness shares one.
const FIRST: usize = 1 << 18;

/// The suffix a swapped run's instances are renamed with, so they bind apart from the first run's.
const SWAPPED: &str = "swapped";

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
        return synthesize_plain(ir);
    };
    let reading = reading(ir, &callers);
    let mut whole = run(ir, &callers, &|_| Who::First);
    let swapped = run(ir, &callers, &|_| Who::Second);
    for (id, scenario) in &mut whole.suite.scenarios {
        if sends_any(scenario, &reading) {
            if let Some(again) = swapped.suite.scenarios.get(id) {
                append(ir, scenario, again);
            }
        }
    }
    for command in &reading {
        let sent = |name: &QualifiedName| {
            if name == command {
                Who::Second
            } else {
                Who::First
            }
        };
        let other = run(ir, &callers, &sent);
        let back = run(ir, &callers, &|name| sent(name).other());
        for (id, mut scenario) in other.suite.scenarios {
            if whole.suite.scenarios.contains_key(&id) || !about(&id, command) {
                continue;
            }
            if let Some(again) = back.suite.scenarios.get(&id) {
                append(ir, &mut scenario, again);
            }
            whole
                .refusals
                .retain(|refusal| refusal.scenario.as_ref() != Some(&id));
            whole.suite.scenarios.insert(id, scenario);
        }
    }
    whole.suite.select_fresh_format();
    whole
}

/// The model read under one assignment, synthesized, with every command step marked with the
/// caller it is sent as.
fn run(ir: &EssIr, callers: &Callers, who: &dyn Fn(&QualifiedName) -> Who) -> Synthesis {
    let written = ir.with_commands_rewritten(|command| {
        written(
            ir,
            command,
            &callers.for_command(ir, who(&command.name), &command.name),
        )
    });
    let mut synthesis = synthesize_plain(&written);
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
            ResolvedCondition::Otherwise
            | ResolvedCondition::External { .. }
            | ResolvedCondition::WrongState
            | ResolvedCondition::UnknownInstance
            | ResolvedCondition::InputAbsent => {}
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
        Predicate::Compare { left, op, right } => Predicate::Compare {
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

/// `again`'s steps after `scenario`'s own, with every instance and instant it binds renamed apart
/// from the first run's, where both runs can share one scenario.
fn append(ir: &EssIr, scenario: &mut ConformanceScenario, again: &ConformanceScenario) {
    let once = |steps: &[ScenarioStep]| {
        steps.iter().any(|step| {
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
        return;
    }
    let Ok(mut value) = serde_json::to_value(&again.steps) else {
        return;
    };
    rename(&mut value);
    let Some(value) = recaptured(ir, value) else {
        return;
    };
    if let Ok(steps) = serde_json::from_value::<Vec<ScenarioStep>>(value) {
        scenario.steps.extend(steps);
    }
}

/// Whether a run asserts something that depends on every row a view holds rather than on the
/// rows it made: a view expectation over an aggregate view (a count or sum per group), or one that
/// counts, ranks or positions rows. Appended after the first run, such an expectation would be
/// read over both runs' rows and its absolute figures would be the first run's plus its own.
fn reads_every_row(ir: &EssIr, steps: &[ScenarioStep]) -> bool {
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
        let expected = (step.get("step").and_then(serde_json::Value::as_str)
            == Some("expect_event"))
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
