//! `each:` entries that move or remove the row each element names (ess/27, beyond10x/ess#596,
//! `docs/design/set-effects-over-filtered-instances.md`, "Moves and deletes per element").
//!
//! The entries apply in the order written, each to the elements its `where:` selects, and each acts
//! on a held row only: an element naming a row resting outside its move's `from` states is skipped.
//! The witness is one segment appended to the branch's own scenario:
//!
//! * per entry, a row resting where its move starts (anywhere, for a deletion), named by an element
//!   its `where:` selects and, where the witness offers one, no other entry's: read back as that
//!   entry leaves it, so a target taking every named row by one entry fails;
//! * for the first moving entry, a row resting outside its `from` states, named by an element it
//!   selects: read back unmoved, so a target ignoring `from` fails. Where no arrangement leaves a row
//!   there, a first call of the command puts one there through another entry; where none does, the
//!   row is left out;
//! * a decoy resting where the first entry's move starts, which no element names: read unchanged.
//!
//! Every row is read from every view holding every row of the entity with its identity, the state
//! where an entry moves and every field an entry writes. This cut witnesses such entries as the
//! branch's only entries, over one list and one entity other than the subject's; any other shape is
//! refused by name.

use super::super::selects_branch;
use super::{
    after, gap, has_subject_guards, input_value, invocation, next, observing, prepare_in, reach,
    reach_in_state, reaches_other_rows, require, require_absent, row_fields, send,
    subject_accepted, subject_reads, supply, with_sets, written_in, ActorRef, Arrangement,
    BTreeMap, BTreeSet, Determined, Distinction, EntityRef, EssSemanticRef, FactPath, FactValue,
    InstanceName, InvocationModels, InvocationPhase, Node, QualifiedName, RefusalCause,
    ResolvedAffect, ResolvedCommand, ResolvedOutcome, ResolvedView, ScenarioStep, ScenarioValue,
    StateName, Truth, ViewExpectation, ViewRef, SUBJECT_REFUSING,
};
use ess_compiler::ir::{ResolvedEach, ResolvedTypeRef};

/// How many witness elements are tried for each one the segment needs.
const TRIES: usize = 32;

/// Whether `affect` names its rows by the elements of a list and moves or removes them.
pub(super) fn names_rows(affect: &ResolvedAffect) -> bool {
    affect.each.is_some() && (affect.moves.is_some() || affect.deletes)
}

/// One element the segment sends: the row it names, and its other members.
struct Element {
    row: InstanceName,
    members: BTreeMap<String, Node>,
}

/// One row the segment reads back: as arranged, and as the entries leave it (`None`: removed).
struct Witnessed {
    row: Arrangement,
    left: Option<(StateName, BTreeMap<String, Determined>)>,
}

/// Whether `each`'s filter selects `element` under `input`: `True` where it has none.
fn selects(
    each: &ResolvedEach,
    element: &BTreeMap<String, Node>,
    input: &BTreeMap<String, Node>,
) -> Truth {
    let Some(filter) = &each.filter else {
        return Truth::True;
    };
    let read = |path: &FactPath| -> Option<FactValue> {
        let (root, rest) = path.segments().split_first()?;
        if *root != each.binder {
            return input_value(path, input);
        }
        let (first, rest) = rest.split_first()?;
        let mut node = element.get(first)?;
        for segment in rest {
            let Node::Map(entries) = node else {
                return None;
            };
            node = entries.get(segment)?;
        }
        super::fact_value(node)
    };
    written_in(filter, &read).evaluate(&ess_primitives::facts::FactStore::default())
}

/// What `entries`, in the order written, leave in a row resting in `state` holding `settled`,
/// named by `element` under `supplied`: each entry selecting the element moves the row where it
/// rests in the move's `from` states, writes its `sets:` and the element's members it reads, or
/// removes it; `None` once removed.
fn left_by(
    ir: &super::EssIr,
    outcome: &ResolvedOutcome,
    entries: &[&ResolvedAffect],
    (state, settled): (&StateName, &BTreeMap<String, Determined>),
    element: &BTreeMap<String, Node>,
    (input, supplied): (&BTreeMap<String, Node>, &BTreeMap<String, ScenarioValue>),
) -> Option<(StateName, BTreeMap<String, Determined>)> {
    let mut state = state.clone();
    let mut settled = settled.clone();
    for affect in entries {
        let Some(each) = &affect.each else {
            continue;
        };
        if selects(each, element, input) != Truth::True {
            continue;
        }
        if affect.deletes {
            return None;
        }
        if let Some(transition) = &affect.moves {
            if !transition.from.contains(&state) {
                continue;
            }
            state = transition.to.clone();
        }
        settled = after(ir, &with_sets(outcome, affect), supplied, &settled);
        for read in &each.reads {
            if let Some(value) = element.get(&read.member) {
                settled.insert(
                    read.target.clone(),
                    Determined {
                        value: ScenarioValue::literal(value.clone()),
                        type_ref: read.target_type.clone(),
                    },
                );
            }
        }
    }
    Some((state, settled))
}

/// The list sent: each element's members, its identity member the captured row.
fn sent(member: &str, elements: &[&Element]) -> ScenarioValue {
    ScenarioValue::List {
        items: elements
            .iter()
            .map(|element| {
                let mut members: BTreeMap<String, ScenarioValue> = element
                    .members
                    .iter()
                    .map(|(name, value)| (name.clone(), ScenarioValue::literal(value.clone())))
                    .collect();
                members.insert(
                    member.to_owned(),
                    ScenarioValue::instance(element.row.clone()),
                );
                ScenarioValue::Members { members }
            })
            .collect(),
    }
}

/// The same list with a distinct literal identity in place of each captured row: what the branch
/// selection is decided on before any row exists.
fn placeholders(member: &str, elements: &[&Element], ids: &[Node]) -> Node {
    Node::Seq(
        elements
            .iter()
            .zip(ids)
            .map(|(element, id)| {
                let mut members = element.members.clone();
                members.insert(member.to_owned(), id.clone());
                Node::Map(members)
            })
            .collect(),
    )
}

/// The `affects:` segment of a branch whose entries name their rows by the elements of one list
/// and move or remove them (ess/27, beyond10x/ess#596).
#[allow(clippy::too_many_lines)] // One complete witness: arrangement, first call, command, reads.
pub(super) fn affects_segment(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ir = models.arrangement;
    let at = |index: usize| format!("{}.affects[{index}]", outcome.name);
    let first = &outcome.affects[0];
    let Some(each) = &first.each else {
        return Err(gap(at(0), first.entity.to_string(), SHAPE));
    };
    let entity = &first.entity;
    let subject = outcome.subject.as_ref().ok_or_else(|| {
        gap(
            at(0),
            command.name.to_string(),
            "is declared beside no subject",
        )
    })?;
    for (index, affect) in outcome.affects.iter().enumerate() {
        let same = affect.each.as_ref().is_some_and(|other| {
            other.list == each.list && other.member == each.member && other.binder == each.binder
        });
        if !names_rows(affect) || !same || affect.entity != *entity || *entity == subject.entity {
            return Err(gap(at(index), affect.entity.to_string(), SHAPE));
        }
    }
    let entries: Vec<&ResolvedAffect> = outcome.affects.iter().collect();
    let any_move = entries.iter().any(|affect| affect.moves.is_some());
    let views: Vec<&ResolvedView> = observing(ir, entity)
        .into_iter()
        .filter(|view| {
            (!any_move
                || super::lifecycle_state(ir, entity, view, &ir.entity(entity).lifecycle.initial)
                    .is_some())
                && entries.iter().all(|affect| {
                    let written = affect
                        .sets
                        .iter()
                        .map(|set| (&set.target, &set.target_type));
                    let reads = affect.each.iter().flat_map(|each| {
                        each.reads
                            .iter()
                            .map(|read| (&read.target, &read.target_type))
                    });
                    written.chain(reads).all(|(target, type_ref)| {
                        view.field(target)
                            .is_some_and(|field| field.type_ref == *type_ref)
                    })
                })
        })
        .collect();
    if views.is_empty() {
        return Err(gap(
            at(0),
            entity.to_string(),
            "is moved or removed by an `each:` entry and published by no immediate, unfiltered, \
             unparameterised, unpaged view carrying its identity, its state and every field the \
             entries write, so a row an entry took and one it skipped cannot be told apart",
        ));
    }
    let members = command
        .input
        .iter()
        .find(|field| field.name == each.list)
        .and_then(|field| match &field.type_ref {
            ResolvedTypeRef::List { of } => of.declared(),
            _ => None,
        })
        .and_then(|element| match &ir.named_type(element).body {
            ess_compiler::ir::ResolvedBody::Struct { fields, .. } => Some(fields),
            _ => None,
        })
        .ok_or_else(|| {
            gap(
                at(0),
                each.list.clone(),
                "is not a list of structs the witness can build elements of",
            )
        })?;
    let witness = |nth: usize| {
        crate::witness::fields(ir, members, Distinction::further(11 + nth))
            .ok()
            .map(|mut element| {
                element.remove(&each.member);
                element
            })
    };

    let distinction = next(ir, &subject.entity, taken);
    let setup = prepare_in(ir, outcome, actors, None, distinction)?;
    let instance = setup.instance.clone().ok_or_else(|| {
        gap(
            at(0),
            subject.entity.to_string(),
            "has no subject row an arrangement names",
        )
    })?;
    let mut input = match (&setup.before, has_subject_guards(command)) {
        (Some(held), true) => reach_in_state(ir, command, outcome, held, Distinction::PLAIN)?,
        _ => reach(ir, command, outcome, Distinction::PLAIN)?,
    };
    // A `when_subject:` refusal answered before the branch that takes this input on the subject
    // just arranged would answer in its place: the input is chosen again on that subject.
    if let Some(held) = &setup.before {
        input = subject_accepted(
            ir,
            (command, outcome),
            (&subject.entity, &instance, held),
            (&setup.steps, &setup.settled),
            input,
        )
        .map_err(|refusing| {
            gap(
                at(0),
                format!(
                    "{}` answered first by `{}",
                    subject.entity,
                    refusing.join("`, `")
                ),
                SUBJECT_REFUSING,
            )
        })?;
    }
    // An element selected by entry `index` and, where the witness offers one, by no other entry.
    let element_for = |index: usize| -> Option<BTreeMap<String, Node>> {
        let candidates: Vec<BTreeMap<String, Node>> = (0..TRIES).filter_map(witness).collect();
        let picks = |element: &&BTreeMap<String, Node>| {
            entries[index]
                .each
                .as_ref()
                .is_some_and(|own| selects(own, element, &input) == Truth::True)
        };
        let alone = |element: &&BTreeMap<String, Node>| {
            entries.iter().enumerate().all(|(other, affect)| {
                other == index
                    || affect
                        .each
                        .as_ref()
                        .is_some_and(|each| selects(each, element, &input) == Truth::False)
            })
        };
        candidates
            .iter()
            .find(|element| picks(element) && alone(element))
            .or_else(|| candidates.iter().find(picks))
            .cloned()
    };
    // A row arranged in the first state `rests` admits that a plain arrangement reaches.
    let arrange_in = |rests: &dyn Fn(&StateName) -> bool,
                      taken: &mut BTreeSet<InstanceName>|
     -> Option<Arrangement> {
        let states = &ir.entity(entity).lifecycle.states;
        states
            .iter()
            .filter(|state| rests(state))
            .find_map(|state| {
                super::super::arrange(ir, entity, state, actors, next(ir, entity, taken), &[])
                    .ok()
                    .filter(|row| !reaches_other_rows(ir, &row.steps))
            })
    };
    let starts = |affect: &ResolvedAffect, state: &StateName| match &affect.moves {
        Some(transition) => transition.from.contains(state),
        None => true,
    };
    let leaves = |affect: &ResolvedAffect, state: &StateName| match &affect.moves {
        Some(transition) => transition.from.contains(state) && *state != transition.to,
        None => true,
    };

    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    let mut named: Vec<(Arrangement, BTreeMap<String, Node>)> = Vec::new();
    for (index, affect) in entries.iter().enumerate() {
        let element = element_for(index).ok_or_else(|| {
            gap(
                at(index),
                each.list.clone(),
                "offers no element the entry's `where:` selects",
            )
        })?;
        let row = arrange_in(&|state| leaves(affect, state), taken)
            .or_else(|| arrange_in(&|state| starts(affect, state), taken))
            .ok_or_else(|| {
                gap(
                    at(index),
                    entity.to_string(),
                    "has no row resting where the entry's move starts that the arranging \
                     commands produce",
                )
            })?;
        named.push((row, element));
    }
    let decoy = arrange_in(&|state| leaves(entries[0], state), taken)
        .or_else(|| arrange_in(&|state| starts(entries[0], state), taken))
        .ok_or_else(|| {
            gap(
                at(0),
                entity.to_string(),
                "has no decoy row the arranging commands produce",
            )
        })?;
    // The row the first moving entry skips, and the element naming it.
    let one = identities(ir, members, &each.member, 1).unwrap_or_default();
    let skipped = match entries.iter().position(|affect| affect.moves.is_some()) {
        Some(mover) if !one.is_empty() => skipped_row(
            models,
            (command, outcome),
            actors,
            taken,
            (&entries, mover),
            (&input, &one, &element_for, &arrange_in),
            &mut source,
        )?,
        _ => None,
    };
    let mut first_call = Vec::new();
    if let Some((row, element, call)) = skipped {
        named.push((row, element));
        first_call = call;
    }
    for (row, _) in &named {
        steps.extend(row.steps.iter().cloned());
        source.extend(row.source.iter().cloned());
    }
    // The first call puts the skipped row where it rests, after the row itself exists.
    steps.extend(first_call);
    steps.extend(decoy.steps.iter().cloned());
    source.extend(decoy.source.iter().cloned());

    let elements: Vec<Element> = named
        .iter()
        .map(|(row, members)| Element {
            row: row.instance.clone(),
            members: members.clone(),
        })
        .collect();
    let listed: Vec<&Element> = elements.iter().collect();
    let ids = identities(ir, members, &each.member, listed.len()).ok_or_else(|| {
        gap(
            at(0),
            each.list.clone(),
            "offers no distinct identities to decide the branch on",
        )
    })?;
    let mut decided = input.clone();
    decided.insert(each.list.clone(), placeholders(&each.member, &listed, &ids));
    if !selects_branch(ir, command, outcome, setup.before.as_ref(), &decided)
        .is_ok_and(|reached| reached)
    {
        return Err(gap(
            at(0),
            each.list.clone(),
            "offers no list naming one row per entry that reaches the branch",
        ));
    }
    let mut supplied = supply(
        ir,
        command,
        &decided,
        Some(subject),
        Some(&instance),
        &setup.bound,
    );
    supplied.insert(each.list.clone(), sent(&each.member, &listed));

    steps.extend(setup.steps.iter().cloned());
    source.extend(setup.source.iter().cloned());
    let subject_left = after(ir, outcome, &supplied, &setup.settled);
    let reads = subject_reads(
        ir,
        &subject.entity,
        &instance,
        setup.after.as_ref(),
        &subject_left,
        &mut source,
    );
    models.mark(InvocationPhase::Arrange, &mut steps);
    invocation(
        models,
        command,
        outcome,
        actors,
        supplied.clone(),
        &mut steps,
        &mut source,
    );
    steps.extend(reads);

    let mut witnessed = Vec::new();
    for (row, element) in named {
        let left = left_by(
            ir,
            outcome,
            &entries,
            (&row.state, &row.settled),
            &element,
            (&decided, &supplied),
        );
        witnessed.push(Witnessed { row, left });
    }
    let kept = Some((decoy.state.clone(), decoy.settled.clone()));
    witnessed.push(Witnessed {
        row: decoy,
        left: kept,
    });
    for view in views {
        let name = ViewRef::new(view.name.clone());
        let mut reads = Vec::new();
        for witness in &witnessed {
            match &witness.left {
                None => require_absent(ir, entity, &name, &witness.row.instance, &mut reads),
                Some((state, settled)) => {
                    let fields =
                        row_fields(ir, entity, view, &witness.row.instance, state, settled);
                    require(
                        view,
                        &name,
                        BTreeMap::new(),
                        ViewExpectation::Contains { fields },
                        &mut reads,
                    );
                }
            }
        }
        steps.extend(reads);
        source.insert(name.into());
    }
    source.insert(EntityRef::from(entity).into());
    Ok((steps, source))
}

/// Why a branch's entries have no witness in this cut.
const SHAPE: &str = "moves or removes the row each element names beside another kind of entry, \
     another list or another entity, or over the subject's own entity, and this cut witnesses such \
     entries over one list and one entity other than the subject's as the branch's only entries";

/// `count` distinct identities of the element's identity member, from the witness.
fn identities(
    ir: &super::EssIr,
    members: &[ess_compiler::ir::ResolvedField],
    member: &str,
    count: usize,
) -> Option<Vec<Node>> {
    let mut out: Vec<Node> = Vec::with_capacity(count);
    for nth in 0..TRIES * 2 {
        if out.len() == count {
            break;
        }
        let element = crate::witness::fields(ir, members, Distinction::further(41 + nth)).ok()?;
        let id = element.get(member)?.clone();
        if !out.contains(&id) {
            out.push(id);
        }
    }
    (out.len() == count).then_some(out)
}

/// The row entry `mover` skips, the element naming it, and the steps of a first call that put it
/// where it rests where a plain arrangement does not; `None` where no row is reached outside the
/// move's `from` states.
#[allow(clippy::type_complexity)] // The closures the segment shares with this one row.
fn skipped_row(
    models: &InvocationModels<'_>,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    (entries, mover): (&[&ResolvedAffect], usize),
    (input, ids, element_for, arrange_in): (
        &BTreeMap<String, Node>,
        &[Node],
        &dyn Fn(usize) -> Option<BTreeMap<String, Node>>,
        &dyn Fn(&dyn Fn(&StateName) -> bool, &mut BTreeSet<InstanceName>) -> Option<Arrangement>,
    ),
    source: &mut BTreeSet<EssSemanticRef>,
) -> Result<Option<(Arrangement, BTreeMap<String, Node>, Vec<ScenarioStep>)>, RefusalCause> {
    let ir = models.arrangement;
    let Some(transition) = &entries[mover].moves else {
        return Ok(None);
    };
    let Some(element) = element_for(mover) else {
        return Ok(None);
    };
    let outside = |state: &StateName| !transition.from.contains(state);
    // A row resting where the move arrives reads the same whether or not it was taken, so another
    // state is tried first.
    if let Some(row) = arrange_in(&|state| outside(state) && *state != transition.to, taken)
        .or_else(|| arrange_in(&outside, taken))
    {
        return Ok(Some((row, element, Vec::new())));
    }
    // Another entry's move arrives outside this one's `from`: a first call of the command takes a
    // row there, for a further subject.
    for (other, affect) in entries.iter().enumerate() {
        let Some(through) = &affect.moves else {
            continue;
        };
        if other == mover || !outside(&through.to) {
            continue;
        }
        let Some(moving) = element_for(other) else {
            continue;
        };
        let Some(row) = arrange_in(&|state| through.from.contains(state), taken) else {
            continue;
        };
        let Some(subject) = outcome.subject.as_ref() else {
            return Ok(None);
        };
        let each = affect.each.as_ref().expect("checked by the segment");
        let distinction = next(ir, &subject.entity, taken);
        let setup = prepare_in(ir, outcome, actors, None, distinction)?;
        let Some(instance) = setup.instance.clone() else {
            continue;
        };
        let carried = Element {
            row: row.instance.clone(),
            members: moving.clone(),
        };
        let mut sent_input = input.clone();
        sent_input.insert(
            each.list.clone(),
            placeholders(&each.member, &[&carried], ids),
        );
        if !selects_branch(ir, command, outcome, setup.before.as_ref(), &sent_input)
            .is_ok_and(|reached| reached)
        {
            continue;
        }
        let mut supplied = supply(
            ir,
            command,
            &sent_input,
            Some(subject),
            Some(&instance),
            &setup.bound,
        );
        supplied.insert(each.list.clone(), sent(&each.member, &[&carried]));
        let Some((state, settled)) = left_by(
            ir,
            outcome,
            entries,
            (&row.state, &row.settled),
            &moving,
            (&sent_input, &supplied),
        ) else {
            continue;
        };
        if !outside(&state) {
            continue;
        }
        let mut first = setup.steps.clone();
        source.extend(setup.source.iter().cloned());
        send((command, outcome), actors, supplied, &mut first, source);
        let rested = Arrangement {
            state,
            settled,
            ..row
        };
        return Ok(Some((rested, element, first)));
    }
    Ok(None)
}
