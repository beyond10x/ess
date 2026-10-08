//! One row per literal identity in a run (beyond10x/ess#480).
//!
//! A target holds one row per identity: a creation sent an identity a row already holds is answered
//! by the existing-identity branch, or refused, and never makes a second row (`creates:` never
//! replaces an instance). Synthesis keeps the rows a scenario arranges apart by creating each at a
//! distinction of its own and sending that distinction's far witness as its identity
//! (`own_identity`). Where the guards on the identity leave one value for every distinction —
//! `item_id.count > 10` cuts each far witness to `item_id-13` — two rows of one scenario are still
//! sent one identity, and the scenario fails every target that honours the specification.
//!
//! [`withdraw_repeated_identities`] holds the finished suite to the rule, whatever family built a
//! scenario: one that requires a creation for a literal identity a row it created earlier still
//! holds is withdrawn and refused by name. A row the scenario deleted holds its identity no longer,
//! so a recreation after a deletion stands; a deletion naming a row this pass cannot trace to a
//! literal releases every identity of its entity, so only what the steps show is refused.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{EntityHandle, EssIr, ResolvedEffect, ResolvedInstance, ResolvedOutcome};
use ess_primitives::node::Node;

use crate::scenario::{InstanceName, OutcomeRef, ScenarioStep, ScenarioValue};
use crate::witness::WitnessGap;

use super::{RefusalCause, Synthesis};

/// Why a scenario that creates two rows under one identity has no witness.
pub(super) const CREATED_TWICE: &str =
    "names one row per value, and this scenario creates a second row \
                             under a value a row it created still holds: no value the guards \
                             admit keeps its rows apart";

/// Every scenario of `synthesis` that creates a second row under a literal identity a row it
/// created still holds, withdrawn and refused with the identity and its type.
pub(super) fn withdraw_repeated_identities(ir: &EssIr, synthesis: &mut Synthesis) {
    let withdrawn: Vec<_> = synthesis
        .suite
        .scenarios
        .iter()
        .filter_map(|(id, scenario)| repeated(ir, &scenario.steps).map(|cause| (id.clone(), cause)))
        .collect();
    for (id, cause) in withdrawn {
        super::withdraw(synthesis, &id, cause);
    }
}

/// The cause, where `steps` require a creation for a literal identity a row they created earlier
/// still holds; `None` otherwise.
fn repeated(ir: &EssIr, steps: &[ScenarioStep]) -> Option<RefusalCause> {
    let mut held: BTreeMap<&EntityHandle, BTreeSet<Node>> = BTreeMap::new();
    let mut captured: BTreeMap<&InstanceName, Node> = BTreeMap::new();
    let mut sent: Option<&BTreeMap<String, ScenarioValue>> = None;
    let mut created: Option<(&EntityHandle, Node)> = None;
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { input, .. } => {
                sent = Some(input);
                created = None;
            }
            ScenarioStep::ExecuteCommandWithoutInput { .. } => {
                sent = None;
                created = None;
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } => {
                if let Some((made, identity)) = created.take() {
                    if ir.entity(made).name.to_string() == entity.to_string() {
                        captured.insert(instance, identity);
                    }
                }
            }
            ScenarioStep::ExpectOutcome { outcome } => {
                let Some(declared) = declared(ir, outcome) else {
                    continue;
                };
                let Some(subject) = declared.subject.as_ref() else {
                    continue;
                };
                let entity = &subject.entity;
                match subject.effect {
                    ResolvedEffect::Creates => {
                        let Some(identity) = super::existence::identity_input(declared)
                            .and_then(|field| sent.and_then(|input| literal_at(input, field)))
                        else {
                            continue;
                        };
                        if !held.entry(entity).or_default().insert(identity.clone()) {
                            return Some(cause(ir, entity));
                        }
                        created = Some((entity, identity));
                    }
                    ResolvedEffect::Deletes => {
                        let rows = held.entry(entity).or_default();
                        match deleted(&subject.instance, sent, &captured) {
                            Some(identity) => {
                                rows.remove(&identity);
                            }
                            None => rows.clear(),
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    None
}

/// The literal identity of the row a deletion `sent` names, read from the input or from the
/// creation that captured the instance it names; `None` where the steps do not show it.
fn deleted(
    instance: &ResolvedInstance,
    sent: Option<&BTreeMap<String, ScenarioValue>>,
    captured: &BTreeMap<&InstanceName, Node>,
) -> Option<Node> {
    let ResolvedInstance::Supplied { field } = instance else {
        return None;
    };
    match sent?.get(&field.name)? {
        ScenarioValue::Literal { value } => Some(value.clone()),
        ScenarioValue::Instance { instance } => captured.get(instance).cloned(),
        _ => None,
    }
}

/// The declared outcome `outcome` names.
fn declared<'i>(ir: &'i EssIr, outcome: &OutcomeRef) -> Option<&'i ResolvedOutcome> {
    ir.commands()
        .get(outcome.command.name())?
        .outcomes
        .iter()
        .find(|declared| declared.name == outcome.outcome)
}

/// The literal `input` sends at `field`, a path (ess/22, A4) read inside its literal root.
fn literal_at(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<Node> {
    let literals: BTreeMap<String, Node> = input
        .iter()
        .filter_map(|(name, value)| value.as_literal().map(|node| (name.clone(), node.clone())))
        .collect();
    ess_compiler::ir::read_input(&literals, field).cloned()
}

/// Why `entity`'s rows of one scenario cannot be kept apart.
fn cause(ir: &EssIr, entity: &EntityHandle) -> RefusalCause {
    let identity = &ir.entity(entity).identity;
    RefusalCause::NoWitness(WitnessGap {
        path: identity.name.clone(),
        type_ref: identity.type_ref.to_string(),
        reason: CREATED_TWICE,
    })
}
