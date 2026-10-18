//! Rows arranged by callers chosen for them (downstream report, 2026-10-18).
//!
//! A command that writes a stored field from its caller (`sets: {owner: {caller: user_id}}`) gives
//! every row it creates under one assignment the same value: the assigned caller's. A branch that
//! selects rows by that field — a set effect's `where: owner == input.user_id`, an `affects:` entry,
//! a `when_related:` row set — needs rows on both sides of its filter, and one caller leaves them
//! all on one side, so the branch was refused as having no rows the arranging commands produce.
//!
//! The caller is the arrangement's to choose, as an input is: a target is sent each act as the
//! caller the step names. So where every reading of the model (`caller::synthesize`) leaves a
//! scenario about a command refused, that command's scenarios are synthesized once more from a
//! model in which every *other* command reads such an attribute from an input of its own
//! ([`lift`]); the command under test keeps the assigned caller. Each arranging act of the result
//! is then sent as a caller carrying the value the search chose for that input, and the input
//! itself is taken out ([`unlift`]). The mixed-caller planner (`cross_single_command`) is not
//! given these rows: a creator whose chosen attribute equals the acting caller's would pass for a
//! second principal and cross nothing.
//!
//! Only a command that reads the attribute in values alone is lifted: one comparing it in a guard
//! keeps the assigned caller, because the guard and the value must read one caller. Only scenarios
//! every ordinary reading refused are taken, so a suite the ordinary readings already witness keeps
//! its bytes.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EssIr, ResolvedCommand, ResolvedField, ResolvedPayloadField, ResolvedPayloadValue,
};
use ess_domain::name::{Naming, QualifiedName};
use ess_primitives::node::Node;

use super::{about, reads, under_test, written, Assignment, Callers};
use crate::scenario::{ConformanceScenario, ScenarioId, ScenarioStep, ScenarioValue};
use crate::synthesize::{Note, Synthesis};

/// The prefix of the input a lifted caller attribute is read from in the lifted model. It never
/// reaches a suite: [`unlift`] takes it out of every step, and a scenario still naming it is not
/// taken.
const LIFTED: &str = "caller_attribute_";

/// The inputs standing for caller attributes, by command, each with the attribute it stands for.
pub(super) type Lifted = BTreeMap<String, BTreeMap<String, String>>;

/// The model `who` reads, with every command but `except` reading each caller attribute it reads in
/// values alone from an input of its own; `None` where no command is lifted.
pub(super) fn lifted_model(
    ir: &EssIr,
    callers: &Callers,
    who: Assignment<'_>,
    except: &QualifiedName,
) -> Option<(EssIr, Lifted)> {
    let mut raised: BTreeMap<QualifiedName, ResolvedCommand> = BTreeMap::new();
    let mut lifted = Lifted::new();
    for command in ir.commands().values() {
        if command.name == *except {
            continue;
        }
        let (command, names) = lift(ir, command);
        if names.is_empty() {
            continue;
        }
        lifted.insert(command.name.to_string(), names);
        raised.insert(command.name.clone(), command);
    }
    if lifted.is_empty() {
        return None;
    }
    let model = ir.with_commands_rewritten(|command| {
        let values = callers.for_command(ir, who(&command.name), &command.name);
        written(ir, raised.get(&command.name).unwrap_or(command), &values)
    });
    Some((model, lifted))
}

/// `command` with every value read of each caller attribute it compares in no guard made a read
/// of an input standing for it, and those inputs with the attribute each stands for.
fn lift(ir: &EssIr, command: &ResolvedCommand) -> (ResolvedCommand, BTreeMap<String, String>) {
    let mut out = command.clone();
    let mut names = BTreeMap::new();
    for attribute in reads(ir, &command.name) {
        let name = format!("{LIFTED}{attribute}");
        if command.input.iter().any(|input| input.name == name) {
            continue;
        }
        let mut candidate = out.clone();
        let mut type_ref = None;
        for outcome in &mut candidate.outcomes {
            for field in outcome
                .payload
                .iter_mut()
                .flat_map(|payload| payload.fields.iter_mut())
                .chain(outcome.sets.iter_mut())
                .chain(outcome.error_payload.iter_mut())
            {
                lift_value(field, &attribute, &name, &mut type_ref);
            }
        }
        let Some(type_ref) = type_ref else {
            continue;
        };
        // Still read once every value read is lifted: a guard compares it, and the guard and the
        // value must read one caller.
        let probe = BTreeMap::from([(attribute.clone(), Node::Text(String::new()))]);
        if written(ir, &candidate, &probe) != candidate {
            continue;
        }
        candidate.input.push(ResolvedField {
            name: name.clone(),
            type_ref,
            naming: Naming::default(),
        });
        out = candidate;
        names.insert(name, attribute);
    }
    (out, names)
}

/// One determined field, with a read of the caller's `attribute` made a read of the input `name`.
fn lift_value(
    field: &mut ResolvedPayloadField,
    attribute: &str,
    name: &str,
    type_ref: &mut Option<ess_compiler::ir::ResolvedTypeRef>,
) {
    match &mut field.value {
        ResolvedPayloadValue::CallerAttribute {
            attribute: read,
            type_ref: of,
        } if read == attribute => {
            type_ref.get_or_insert_with(|| of.clone());
            field.value = ResolvedPayloadValue::InputField {
                field: name.to_owned(),
                type_ref: of.clone(),
            };
        }
        ResolvedPayloadValue::Struct { fields } => {
            for leaf in fields {
                lift_value(leaf, attribute, name, type_ref);
            }
        }
        _ => {}
    }
}

/// `step` sent as the caller whose lifted attributes carry the values its lifted inputs were
/// given, with those inputs taken out; `false` where one was not sent as a literal, which no caller
/// can carry.
fn unlift(step: &mut ScenarioStep, lifted: &Lifted) -> bool {
    let ScenarioStep::ExecuteCommand {
        command,
        input,
        caller,
        ..
    } = step
    else {
        return true;
    };
    let Some(names) = lifted.get(&command.to_string()) else {
        return true;
    };
    for (name, attribute) in names {
        match input.remove(name) {
            Some(ScenarioValue::Literal { value }) if !matches!(value, Node::Null) => {
                caller.insert(attribute.clone(), value);
            }
            _ => return false,
        }
    }
    true
}

/// Whether anything in the serialized scenario still names a lifted input.
fn names_lifted(scenario: &ConformanceScenario) -> bool {
    serde_json::to_string(scenario).map_or(true, |text| text.contains(LIFTED))
}

/// `scenario` with each step stamped by `stamp` and sent as the caller its lifted inputs name
/// ([`unlift`]); `None` where a step cannot be, or anything still names a lifted input.
fn finished(
    mut scenario: ConformanceScenario,
    lifted: &Lifted,
    stamp: &dyn Fn(&mut ScenarioStep),
) -> Option<ConformanceScenario> {
    let mut sendable = true;
    for step in &mut scenario.steps {
        stamp(step);
        sendable &= unlift(step, lifted);
    }
    (sendable && !names_lifted(&scenario)).then_some(scenario)
}

/// Every scenario of a synthesis of the lifted model [`finished`], and the others dropped.
pub(super) fn finish(
    synthesis: &mut Synthesis,
    lifted: &Lifted,
    stamp: &dyn Fn(&mut ScenarioStep),
) {
    let scenarios = std::mem::take(&mut synthesis.suite.scenarios);
    for (id, scenario) in scenarios {
        if let Some(scenario) = finished(scenario, lifted, stamp) {
            synthesis.suite.scenarios.insert(id, scenario);
        }
    }
}

/// The scenarios about `command` that `synthesis` refused and `again`, synthesized from the lifted
/// model, holds, moved into `synthesis`: [`finished`] by `stamp`, the
/// refusal withdrawn, and the notes about it carried. Whether any was.
pub(super) fn adopt(
    synthesis: &mut Synthesis,
    mut again: Synthesis,
    command: &QualifiedName,
    lifted: &Lifted,
    stamp: &dyn Fn(&mut ScenarioStep),
) -> bool {
    let mut any = false;
    let refused: BTreeSet<ScenarioId> = synthesis
        .refusals
        .iter()
        .filter(|refusal| !refusal.stands)
        .filter_map(|refusal| refusal.scenario.clone())
        .filter(|id| about(id, command) && !synthesis.suite.scenarios.contains_key(id))
        .collect();
    for id in refused {
        let Some(scenario) = again
            .suite
            .scenarios
            .remove(&id)
            .and_then(|scenario| finished(scenario, lifted, stamp))
        else {
            continue;
        };
        synthesis
            .refusals
            .retain(|refusal| refusal.scenario.as_ref() != Some(&id));
        synthesis.refusals.extend(
            again
                .refusals
                .iter()
                .filter(|refusal| refusal.stands && refusal.scenario.as_ref() == Some(&id))
                .cloned(),
        );
        synthesis.notes.extend(
            again
                .notes
                .iter()
                .filter(|note| scenario_of(note) == Some(&id))
                .cloned(),
        );
        synthesis.suite.scenarios.insert(id, scenario);
        any = true;
    }
    any
}

/// The commands `synthesis` refused a scenario about, by their declared names.
pub(super) fn refused_commands(ir: &EssIr, synthesis: &Synthesis) -> BTreeSet<QualifiedName> {
    let named: BTreeSet<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| !refusal.stands)
        .filter_map(|refusal| refusal.scenario.as_ref())
        .filter(|id| !synthesis.suite.scenarios.contains_key(*id))
        .filter_map(under_test)
        .collect();
    ir.commands()
        .keys()
        .filter(|command| named.contains(&command.to_string()))
        .cloned()
        .collect()
}

/// The scenario a note that travels with one names.
fn scenario_of(note: &Note) -> Option<&ScenarioId> {
    match note {
        Note::PartialObservation { scenario, .. }
        | Note::UnseparatedSources { scenario, .. }
        | Note::UnwitnessedOverlap { scenario, .. }
        | Note::UnaccompaniedRelatedCopy { scenario, .. } => Some(scenario),
        _ => None,
    }
}
