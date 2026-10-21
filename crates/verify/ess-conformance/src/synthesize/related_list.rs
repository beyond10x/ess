//! The rows a `List` of references names (`when_related: {via: input.<list>, exists: false}`,
//! `ess/28`, beyond10x/ess#558, `docs/design/generic-language-features.md` § G2, quantifiers).
//!
//! A command guarding on a list of references is arranged here, apart from the one-row strategy
//! of [`super::related_guard`], which leaves it out:
//!
//! * its refusal is sent a list whose first element names a row the scenario creates and whose
//!   second names none, so a target that checks only the first element, or answers "some row
//!   exists", fails, and the refusal's `{missing: input.<list>}` is asserted to name the second;
//! * every other branch is sent a list naming two rows the scenario creates, so a target that
//!   refuses a list with more than one element, or checks only one of them, fails;
//! * every other run of the command — a driver arranging the row it creates — is sent an empty
//!   list, which names no missing row ([`empty`]): the empty list is witnessed by every such run.
use std::collections::BTreeMap;

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedInstance,
    ResolvedOutcome, ResolvedRelatedTest, ResolvedRelatedVia, ResolvedTypeRef,
};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use super::{arrange_first, ActorRef, RefusalCause, ScenarioValue, Setup};
use crate::witness::Distinction;

/// The distinctions the rows a list names are created at: apart from every block the one-row
/// strategy uses ([`super::related_guard`]), so no row is another arrangement's.
const ROWS: usize = 27_720 * 25;

/// The list input `command`'s related guard reads, and the entity whose rows its elements name.
pub(super) fn reads(command: &ResolvedCommand) -> Option<(&str, &EntityHandle)> {
    command
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                via:
                    ResolvedRelatedVia::Input {
                        field,
                        type_ref: ResolvedTypeRef::List { .. },
                    },
                entity,
                ..
            } => Some((field.as_str(), entity)),
            _ => None,
        })
}

/// Whether `command` guards on the rows a list of references names.
pub(super) fn uses(command: &ResolvedCommand) -> bool {
    reads(command).is_some()
}

/// Whether this strategy arranges `outcome`'s scenario: every branch of a command guarding on a
/// list, but an injected fault and a replay.
pub(super) fn routes(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    uses(command)
        && outcome.replays.is_none()
        && !matches!(
            outcome.condition,
            ResolvedCondition::External { .. }
                | ResolvedCondition::ExternalWhen { .. }
                | ResolvedCondition::ExternalSubject { .. }
        )
}

/// The value every run of `command` not arranged here sends in its list input: an empty list,
/// which names no missing row, so a driver reaches the branch it arranges rather than the refusal.
pub(super) fn empty(command: &ResolvedCommand) -> Option<(&str, ScenarioValue)> {
    reads(command).map(|(field, _)| (field, ScenarioValue::literal(Node::Seq(Vec::new()))))
}

fn unarranged() -> RefusalCause {
    super::related_guard::unarranged()
}

fn refuses(outcome: &ResolvedOutcome) -> bool {
    matches!(
        outcome.condition,
        ResolvedCondition::Related {
            test: ResolvedRelatedTest::Absent,
            ..
        }
    )
}

/// An identity of the list's element type no row carries: the first element of a list the
/// witness search tries at a distinction of its own, where the arranged rows' identities are
/// minted by the target or tried at others.
fn fresh_element(ir: &EssIr, command: &ResolvedCommand, field: &str) -> Result<Node, RefusalCause> {
    for nth in 1..=4 {
        let inputs =
            crate::witness::candidates(ir, command, &[], Distinction::further(ROWS + 9 + nth))
                .map_err(RefusalCause::NoWitness)?;
        let element = inputs.iter().find_map(|input| match input.get(field) {
            Some(Node::Seq(items)) => items.first().cloned(),
            _ => None,
        });
        if let Some(element) = element {
            return Ok(element);
        }
    }
    // A search trying only empty lists: an identity backed by a `Uuid` is minted from the command
    // and the field, which no arrangement of this scenario tries.
    let declared = command
        .input
        .iter()
        .find(|held| held.name == field)
        .map(|held| &held.type_ref);
    let Some(ResolvedTypeRef::List { of }) = declared else {
        return Err(unarranged());
    };
    uuid_backed(ir, of)
        .then(|| {
            Node::Text(crate::witness::uuid_of(&format!(
                "{}/{field}/missing",
                command.name
            )))
        })
        .ok_or_else(unarranged)
}

/// Whether `type_ref` is a `Uuid`, through newtypes.
fn uuid_backed(ir: &EssIr, type_ref: &ResolvedTypeRef) -> bool {
    let mut current = type_ref;
    for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Primitive { name } => {
                return *name == ess_domain::types::Primitive::Uuid;
            }
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ess_compiler::ir::ResolvedBody::Newtype { of, .. } => current = of,
                _ => return false,
            },
            _ => return false,
        }
    }
    false
}

/// Arranges the rows `outcome`'s list names ahead of `setup`'s own steps, and the list it is sent:
/// for the refusal one created row then an identity no row carries, for any other branch two
/// created rows. Returns the list input and its value.
pub(super) fn arrange(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &mut Setup,
    input: &BTreeMap<String, Node>,
) -> Result<Listed, RefusalCause> {
    let (field, entity) = reads(command).ok_or_else(unarranged)?;
    let initial = ir.entity(entity).lifecycle.initial.clone();
    let row = |at: usize| {
        arrange_first(
            ir,
            entity,
            std::slice::from_ref(&initial),
            actors,
            Distinction::further(ROWS + at),
            &[],
        )
        .map_err(|_| unarranged())
    };
    let first = row(0)?;
    let mut steps = first.steps;
    let mut source = first.source;
    let mut items = vec![ScenarioValue::Instance {
        instance: first.instance,
    }];
    if refuses(outcome) {
        items.push(ScenarioValue::literal(fresh_element(ir, command, field)?));
    } else {
        let second = row(4)?;
        steps.extend(second.steps);
        source.extend(second.source);
        items.push(ScenarioValue::Instance {
            instance: second.instance,
        });
    }
    steps.append(&mut setup.steps);
    setup.steps = steps;
    setup.source.extend(source);
    setup
        .source
        .insert(crate::scenario::EntityRef::from(entity).into());
    Ok(Listed {
        field: field.to_owned(),
        value: ScenarioValue::List { items },
        after: if refuses(outcome) {
            Vec::new()
        } else {
            empty_after(command, outcome, actors, input)
        },
    })
}

/// What [`arrange`] decided: the list input, the list it is sent, and the steps that follow the
/// branch's own assertions ([`empty_after`]).
pub(super) struct Listed {
    pub(super) field: String,
    pub(super) value: ScenarioValue,
    pub(super) after: Vec<super::ScenarioStep>,
}

/// The accepting branch sent an empty list again after its own assertions, where a second run of
/// it changes nothing the scenario already read: it acts on no row, or creates one under an identity the implementation
/// mints. An empty list names no missing row, so a target refusing it fails. Empty for a branch
/// addressing a row by its input, which a second run would address again.
fn empty_after(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    input: &BTreeMap<String, Node>,
) -> Vec<super::ScenarioStep> {
    let minted = outcome.subject.as_ref().is_none_or(|subject| {
        subject.effect == ResolvedEffect::Creates
            && matches!(subject.instance, ResolvedInstance::Observed { .. })
    });
    let Some((field, empty)) = empty(command).filter(|_| minted && outcome.error.is_none()) else {
        return Vec::new();
    };
    let mut sent: BTreeMap<String, ScenarioValue> = input
        .iter()
        .map(|(name, value)| (name.clone(), ScenarioValue::literal(value.clone())))
        .collect();
    sent.insert(field.to_owned(), empty);
    let command_ref = crate::scenario::CommandRef::new(command.name.clone());
    vec![
        super::ScenarioStep::ExecuteCommand {
            caller: BTreeMap::new(),
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
            input: sent,
        },
        super::ScenarioStep::ExpectOutcome {
            outcome: crate::scenario::OutcomeRef::new(command_ref, outcome.name.clone()),
        },
    ]
}

/// The first element of a list `supplied` sends that the scenario arranged no row for: a literal,
/// where every arranged row is named by its instance ([`arrange`]).
pub(super) fn first_missing(
    supplied: &BTreeMap<String, ScenarioValue>,
    field: &str,
) -> Option<Node> {
    let Some(ScenarioValue::List { items }) = supplied.get(field) else {
        return None;
    };
    items.iter().find_map(|item| match item {
        ScenarioValue::Literal { value } => Some(value.clone()),
        _ => None,
    })
}
