//! A refusal guarded by a related row the addressed record's stored reference names reads that
//! record's stored fields (`ess/27`, `story:expression-roots-a-branch-establishes-are-readable`,
//! beyond10x/ess#555): what its scenario asserts of them.
//!
//! The record is the one the refusal's input addresses, arranged by earlier steps of the scenario,
//! and what it holds is what those steps wrote: a field a creating or updating branch set from its
//! input holds what that step sent. Such a value is often an identity of another row the
//! arrangement created, a reference to the instance a step captured, which an `expect_error` step
//! cannot carry: it compares literals. Where the step that created that instance named it by a
//! literal its own event publishes unchanged — a creation whose identity is a supplied input, read
//! back from the event's `input.<field>` — the instance is that literal, and the error is required
//! to carry it.

use std::collections::BTreeMap;

use ess_compiler::ir::{
    EssIr, ResolvedCommand, ResolvedCondition, ResolvedInstance, ResolvedOutcome,
    ResolvedPayloadValue,
};
use ess_primitives::node::Node;

use crate::{InstanceName, ScenarioStep, ScenarioValue};

/// Requires of `expected`, the refusal's `expect_error` step, each `{subject: <field>}` payload
/// field the steps before it determine as a literal, where `outcome` is a refusal guarded through
/// the subject's stored reference under `ess/27`. Every other refusal keeps the step it had.
pub(super) fn name_instances(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    steps: &[ScenarioStep],
    expected: &mut ScenarioStep,
) {
    let through_reference = matches!(
        &outcome.condition,
        ResolvedCondition::Related {
            via: ess_compiler::ir::ResolvedRelatedVia::Subject { .. },
            ..
        }
    );
    if !through_reference || ir.format().major() < ess_domain::system::FormatVersion::V27.major() {
        return;
    }
    let ScenarioStep::ExpectError { fields, .. } = expected else {
        return;
    };
    let Some(ScenarioValue::Instance {
        instance: addressed,
    }) = addressed_identity(command).and_then(|field| super::supplied_at(supplied, field))
    else {
        return;
    };
    let arranged = Arranged::of(ir, steps);
    let Some(row) = arranged.rows.get(&addressed) else {
        return;
    };
    for field in &outcome.error_payload {
        if fields.contains_key(&field.target) || field.conversion.is_some() {
            continue;
        }
        let ResolvedPayloadValue::SubjectField { field: read, .. } = &field.value else {
            continue;
        };
        if let Some(value) = row.get(read).and_then(|held| arranged.literal(held)) {
            fields.insert(field.target.clone(), value);
        }
    }
}

/// The input field a branch of `command` addressing a supplied identity takes it from.
fn addressed_identity(command: &ResolvedCommand) -> Option<&str> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .find_map(|subject| match &subject.instance {
            ResolvedInstance::Supplied { field } => Some(field.name.as_str()),
            ResolvedInstance::Observed { .. } => None,
        })
}

/// What the steps of a scenario left: the literal each captured instance is, where it is known,
/// and the fields each addressed row was written from its input.
#[derive(Default)]
struct Arranged {
    literals: BTreeMap<InstanceName, Node>,
    rows: BTreeMap<InstanceName, BTreeMap<String, ScenarioValue>>,
}

impl Arranged {
    fn of(ir: &EssIr, steps: &[ScenarioStep]) -> Self {
        let mut arranged = Self::default();
        let mut sent: Option<(&ResolvedCommand, &BTreeMap<String, ScenarioValue>)> = None;
        let mut taken: Option<&ResolvedOutcome> = None;
        for step in steps {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. } => {
                    sent = ir
                        .commands()
                        .get(command.name())
                        .map(|declared| (declared, input));
                    taken = None;
                }
                ScenarioStep::ExpectOutcome { outcome } => {
                    taken = sent.and_then(|(declared, input)| {
                        let branch = declared
                            .outcomes
                            .iter()
                            .find(|branch| branch.name == outcome.outcome)?;
                        arranged.wrote(branch, input, None);
                        Some(branch)
                    });
                }
                ScenarioStep::CaptureInstance {
                    instance,
                    event,
                    field,
                    ..
                } => {
                    let (Some((_, input)), Some(branch)) = (sent, taken) else {
                        continue;
                    };
                    let read = branch
                        .payload
                        .iter()
                        .filter(|payload| payload.event.name() == event.name())
                        .flat_map(|payload| &payload.fields)
                        .find(|filled| filled.target == *field && filled.conversion.is_none())
                        .and_then(|filled| match &filled.value {
                            ResolvedPayloadValue::InputField { field, .. } => Some(field),
                            _ => None,
                        });
                    if let Some(value) = read
                        .and_then(|read| super::supplied_at(input, read))
                        .and_then(|value| arranged.literal(&value))
                    {
                        arranged.literals.insert(instance.clone(), value);
                    }
                    arranged.wrote(branch, input, Some(instance));
                }
                _ => {}
            }
        }
        arranged
    }

    /// Records what `branch`, sent `input`, wrote to the row it addresses, or to the row it created
    /// and `captured` names: each field set from its input holds what was sent, and every other
    /// field it sets is no longer known.
    fn wrote(
        &mut self,
        branch: &ResolvedOutcome,
        input: &BTreeMap<String, ScenarioValue>,
        captured: Option<&InstanceName>,
    ) {
        let Some(subject) = &branch.subject else {
            return;
        };
        let row = match (&subject.instance, captured) {
            (_, Some(instance)) => instance.clone(),
            (ResolvedInstance::Supplied { field }, None) => {
                match super::supplied_at(input, &field.name) {
                    Some(ScenarioValue::Instance { instance }) => instance,
                    _ => return,
                }
            }
            (ResolvedInstance::Observed { .. }, None) => return,
        };
        let held = self.rows.entry(row).or_default();
        for set in &branch.sets {
            match &set.value {
                ResolvedPayloadValue::InputField { field, .. } if set.conversion.is_none() => {
                    match super::supplied_at(input, field) {
                        Some(value) => {
                            held.insert(set.target.clone(), value);
                        }
                        None => {
                            held.remove(&set.target);
                        }
                    }
                }
                _ => {
                    held.remove(&set.target);
                }
            }
        }
    }

    /// `value` as a literal, an instance resolved through what is known of it.
    fn literal(&self, value: &ScenarioValue) -> Option<Node> {
        match value {
            ScenarioValue::Literal { value } => Some(value.clone()),
            ScenarioValue::Instance { instance } => self.literals.get(instance).cloned(),
            _ => None,
        }
    }
}
