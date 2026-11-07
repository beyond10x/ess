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
pub(super) const ROWS: usize = 27_720 * 25;

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
    // A branch selected by what the list holds is sent the list its witness holds, each element a
    // row created under it (beyond10x/ess#761), and never the empty list again after it.
    let guarded = !refuses(outcome) && guards_on_list(outcome, field);
    let (mut steps, source, items) = if guarded {
        let Some(Node::Seq(elements)) = input.get(field) else {
            return Err(unarranged());
        };
        named_rows(ir, entity, actors, elements, (ROWS + 16, &setup.steps), &[])?
    } else {
        two_rows(ir, command, outcome, entity, actors, field)?
    };
    steps.append(&mut setup.steps);
    setup.steps = steps;
    setup.source.extend(source);
    setup
        .source
        .insert(crate::scenario::EntityRef::from(entity).into());
    Ok(Listed {
        field: field.to_owned(),
        value: ScenarioValue::List { items },
        after: if refuses(outcome) || guarded {
            Vec::new()
        } else {
            empty_after(command, outcome, actors, input)
        },
    })
}

/// The steps, sources and list items of a list naming created rows, and for the refusal an
/// identity no row carries.
pub(super) type Rows = (
    Vec<super::ScenarioStep>,
    std::collections::BTreeSet<crate::scenario::EssSemanticRef>,
    Vec<ScenarioValue>,
);

/// One created row then, for the refusal, an identity no row carries, or for any other branch a
/// second created row.
fn two_rows(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    field: &str,
) -> Result<Rows, RefusalCause> {
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
    Ok((steps, source, items))
}

/// Whether `outcome`'s own `when:` guard reads the list `field` (beyond10x/ess#761): a branch
/// selected by what the list holds, whose scenario sends the list its witness holds.
fn guards_on_list(outcome: &ResolvedOutcome, field: &str) -> bool {
    crate::decision::when(outcome).is_some_and(|guard| {
        guard
            .fact_paths()
            .iter()
            .any(|path| path.segments().first().is_some_and(|root| root == field))
    })
}

/// Whether an empty list refutes `outcome`'s own `when:` guard over `input` (beyond10x/ess#761):
/// a branch selected by what the list holds, which no run sending the empty list every other run
/// of the command sends ([`empty`]) reaches.
pub(super) fn refuted_by_empty(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: &BTreeMap<String, Node>,
) -> bool {
    let (Some((field, _)), Some(guard)) = (reads(command), crate::decision::when(outcome)) else {
        return false;
    };
    let mut sent = input.clone();
    sent.insert(field.to_owned(), Node::Seq(Vec::new()));
    crate::input::flatten(ir, command, &sent)
        .is_ok_and(|facts| matches!(facts.decide(guard), crate::decision::Decision::Refuted(_)))
}

/// Whether a driver [`refuted_by_empty`] gives way to a sibling: another branch of its command
/// doing the same to the same entity that the empty list does not refute, which an arrangement
/// reaches with the empty list every driver is sent (beyond10x/ess#761).
pub(super) fn gives_way(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: &BTreeMap<String, Node>,
) -> bool {
    let Some(subject) = &outcome.subject else {
        return false;
    };
    command.outcomes.iter().any(|other| {
        other.name != outcome.name
            && other.error.is_none()
            && other.replays.is_none()
            && other.subject.as_ref().is_some_and(|theirs| {
                theirs.entity == subject.entity && theirs.effect == subject.effect
            })
            && !refuted_by_empty(ir, command, other, input)
    })
}

/// The list a driver [`refuted_by_empty`] with no sibling to give way to is sent: its witness
/// list, each element a row created under it ahead of the run, or the row `known` steps already
/// created under it (beyond10x/ess#761). `at` keeps the rows apart from those of another
/// arrangement of the same scenario.
pub(super) fn driven(
    ir: &EssIr,
    command: &ResolvedCommand,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    input: &BTreeMap<String, Node>,
    (at, known): (Distinction, &[super::ScenarioStep]),
    arranging: &[&EntityHandle],
) -> Result<(String, Rows), RefusalCause> {
    let (field, entity) = reads(command).ok_or_else(unarranged)?;
    let Some(Node::Seq(elements)) = input.get(field) else {
        return Err(unarranged());
    };
    let base = ROWS + 32 + 8 * at.get();
    let rows = named_rows(ir, entity, actors, elements, (base, known), arranging)?;
    Ok((field.to_owned(), rows))
}

/// The rows a further run of `outcome` is sent where its `row` moves the list a related guard
/// reads off the scenario's own: each element a row created under it, or the row `known` steps
/// already created under it, so the run is decided by its own guard rather than answered by the
/// refusal over a missing row (beyond10x/ess#761). `None` for the refusal, which sends a missing
/// element on purpose, and for a row sending the empty list, which names no row.
pub(super) fn backed(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    row: &BTreeMap<String, Node>,
    (at, known): (Distinction, &[super::ScenarioStep]),
) -> Option<Result<(String, Rows), RefusalCause>> {
    let (field, _) = reads(command)?;
    let named = matches!(row.get(field), Some(Node::Seq(items)) if !items.is_empty());
    (named && !refuses(outcome)).then(|| driven(ir, command, actors, row, (at, known), &[]))
}

/// The rows a branch [`guards_on_list`] is sent: one per distinct element of its witness list,
/// each created under that element as its identity at a distinction from `base` on, or the row
/// `known` steps already created under it, so the list holds the values its own guard reads
/// (beyond10x/ess#761). Refused where no creation of the entity taking its identity from its
/// input keeps its branch under that identity.
fn named_rows(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    elements: &[Node],
    (base, known): (usize, &[super::ScenarioStep]),
    arranging: &[&EntityHandle],
) -> Result<Rows, RefusalCause> {
    let initial = &ir.entity(entity).lifecycle.initial;
    let all = ir.drivers();
    let creators: Vec<_> = all
        .get(entity)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .filter(|driver| super::born(ir, driver) == initial)
        .collect();
    let entity_ref = crate::scenario::EntityRef::from(entity);
    let held = |element: &Node| {
        known.iter().find_map(|step| match step {
            super::ScenarioStep::CaptureInstance {
                instance,
                entity: captured,
                ..
            } if *captured == entity_ref
                && super::related_guard::created_identity(ir, known, instance).as_ref()
                    == Some(element) =>
            {
                Some(instance.clone())
            }
            _ => None,
        })
    };
    let mut steps = Vec::new();
    let mut source = std::collections::BTreeSet::new();
    let mut named: Vec<(&Node, crate::scenario::InstanceName)> = Vec::new();
    for element in elements {
        if named.iter().any(|(held, _)| *held == element) {
            continue;
        }
        if let Some(instance) = held(element) {
            named.push((element, instance));
            continue;
        }
        let distinction = Distinction::further(base + named.len());
        let row = creators
            .iter()
            .find_map(|creator| {
                let field = super::existence::identity_input(creator.outcome)?;
                let mut sent =
                    super::reach(ir, creator.command, creator.outcome, distinction).ok()?;
                if !super::keeps_branch(ir, creator.command, &sent, field, element) {
                    return None;
                }
                super::set_at(&mut sent, field, Some(element.clone()));
                super::created(
                    ir,
                    entity,
                    creator,
                    actors,
                    distinction,
                    arranging,
                    Some(&sent),
                )
                .ok()
            })
            .ok_or_else(unarranged)?;
        steps.extend(row.steps);
        source.extend(row.source);
        named.push((element, row.instance));
    }
    let items = elements
        .iter()
        .filter_map(|element| {
            named
                .iter()
                .find(|(held, _)| *held == element)
                .map(|(_, instance)| ScenarioValue::Instance {
                    instance: instance.clone(),
                })
        })
        .collect();
    Ok((steps, source, items))
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
