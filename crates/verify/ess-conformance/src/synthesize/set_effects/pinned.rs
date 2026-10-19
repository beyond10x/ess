//! A set effect whose filter **pins** the row's identity (ess/27, beyond10x/ess#547,
//! `docs/design/set-effect-identity-filters.md`, "A pin" and "Synthesis").
//!
//! A filter pins the identity where it is `<identity> == subject.<field>` or `== input.<field>`
//! (either side), alone or as the one such conjunct of a top-level `all:`. It selects at most one
//! row, and that row's identity is minted by its own creation, so the arrangement runs the other
//! way round from [`super::affects_segment`]: the row is created and captured first, and the
//! subject — or the input — is then arranged to name it.
//!
//! Each send names one row, so each row the witness needs gets its own send:
//!
//! * the **target**, on which the filter's other conjuncts hold, resting where the effect visibly
//!   changes it: read back changed;
//! * a **decoy** of the same entity in the target's state, which no send names: read unchanged, so a
//!   target applying the effect to every row fails;
//! * per other conjunct, a row failing it alone, and for a move, a row resting outside its `from`
//!   states: each named by its own send, and read unchanged;
//! * a reference naming an identity no row carries: accepted, and that identity is read absent, so
//!   a target creating the row fails;
//! * an `Optional` reference holding nothing: accepted, and every row reads as before.
//!
//! A pair no declared command reaches is left out, as [`super::zero_match`] is. A pin the subject
//! cannot be arranged to name — no command writes the field from an input — and an identity read
//! other than by a pin are refused by name.

use ess_domain::command::set_effects::SUBJECT_NAMESPACE;
use ess_domain::command::subject_fact::INPUT_NAMESPACE;
use ess_domain::entity::Transition;

use super::super::{related, selects_branch, Setup};
use super::{
    after, answer, arrange_toward_bound, gap, has_subject_guards, input_value, invocation, misses,
    next, observed, observed_removal, prepare_in, reach, reach_in_state, reaches_other_rows,
    read_back, subject_fact, subject_reads, supply, with_sets, written_in, ActorRef, Arrangement,
    BTreeMap, BTreeSet, CompareOp, ConformanceScenario, Distinction, EntityHandle, EntityRef,
    EssIr, EssSemanticRef, FactPath, InstanceName, InvocationModels, InvocationPhase, Node,
    Operand, Predicate, QualifiedName, RefusalCause, ResolvedAffect, ResolvedCommand,
    ResolvedEffect, ResolvedOutcome, ResolvedSetSubject, ResolvedView, Rows, ScenarioStep,
    ScenarioValue, Selection, StateName, Truth, ViewRef,
};

/// Where a pin reads the identity it compares the row's with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Source {
    /// `subject.<field>`: a stored field of the subject, as it held before the outcome.
    Subject(String),
    /// `input.<field>`: a top-level input field.
    Input(String),
}

/// A filter pinning the row's identity: what it reads the identity from, and its other conjuncts.
#[derive(Debug, Clone)]
pub(super) struct Pin {
    pub(super) source: Source,
    /// The filter's other conjuncts, [`Predicate::Always`] where there are none.
    pub(super) rest: Predicate,
}

/// Why a filter reading the identity other than by a pin has no witness.
const NOT_A_PIN: &str = "reads the row's identity other than by a pin (`<identity> == \
                         subject.<field>` or `== input.<field>`, alone or as the one such conjunct \
                         of a top-level `all:`), so no arrangement can name the rows it selects";

/// The pin `filter` declares over a row of `entity`: `Ok(None)` where it does not read the row's
/// identity at all, and an error naming why where it reads it other than by a pin.
pub(super) fn pin_of(
    ir: &EssIr,
    entity: &EntityHandle,
    filter: &Predicate,
) -> Result<Option<Pin>, &'static str> {
    let identity = FactPath::from_segments([ir.entity(entity).identity.name.clone()]);
    if !reads(&identity, filter) {
        return Ok(None);
    }
    if let Some(source) = pinned(&identity, filter) {
        return Ok(Some(Pin {
            source,
            rest: Predicate::Always,
        }));
    }
    let Predicate::All(conjuncts) = filter else {
        return Err(NOT_A_PIN);
    };
    let mut found = None;
    let mut rest = Vec::new();
    for conjunct in conjuncts {
        match pinned(&identity, conjunct) {
            Some(source) if found.is_none() => found = Some(source),
            _ if reads(&identity, conjunct) => return Err(NOT_A_PIN),
            _ => rest.push(conjunct.clone()),
        }
    }
    let source = found.ok_or(NOT_A_PIN)?;
    let rest = match rest.len() {
        0 => Predicate::Always,
        1 => rest.remove(0),
        _ => Predicate::All(rest),
    };
    Ok(Some(Pin { source, rest }))
}

/// Whether `predicate` reads `identity` anywhere.
fn reads(identity: &FactPath, predicate: &Predicate) -> bool {
    predicate
        .fact_paths()
        .into_iter()
        .any(|path| path == identity)
}

/// The source `predicate` pins `identity` to, where it is `identity == <source>` either way round.
fn pinned(identity: &FactPath, predicate: &Predicate) -> Option<Source> {
    let Predicate::Compare {
        left: Operand::Fact(left),
        op: CompareOp::Eq,
        right: Operand::Fact(right),
        ..
    } = predicate
    else {
        return None;
    };
    let other = if left == identity {
        right
    } else if right == identity {
        left
    } else {
        return None;
    };
    let [root, field] = other.segments() else {
        return None;
    };
    if root == SUBJECT_NAMESPACE {
        Some(Source::Subject(field.clone()))
    } else if root == INPUT_NAMESPACE {
        Some(Source::Input(field.clone()))
    } else {
        None
    }
}

/// The rows a pinned filter is witnessed on.
struct Pinned {
    /// The row the first send names: the other conjuncts hold on it, and the effect changes it.
    target: Arrangement,
    /// Another row the other conjuncts select, in the target's state, which no send names.
    decoy: Arrangement,
    /// Rows each named by a send of their own and left as arranged: one per other conjunct,
    /// failing it alone, and for a move, one resting outside its `from` states where one is
    /// reached.
    named: Vec<Arrangement>,
}

impl Pinned {
    fn all(&self) -> impl Iterator<Item = &Arrangement> {
        [&self.target, &self.decoy].into_iter().chain(&self.named)
    }

    /// The rows read back after every send: the target changed, the others as arranged.
    fn rows(&self) -> Rows {
        Rows {
            changed: vec![self.target.clone()],
            kept: std::iter::once(&self.decoy)
                .chain(&self.named)
                .cloned()
                .collect(),
        }
    }
}

/// The target, the decoy and the named rows `selection` — the pin's other conjuncts — arranges,
/// for an effect taking `moves` where it moves its rows.
fn pinned_rows(
    selection: &Selection<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    moves: Option<&Transition>,
) -> Result<Pinned, &'static str> {
    let writes = !selection.sets.sets.is_empty();
    let movable = |state: &StateName| moves.is_none_or(|move_| move_.from.contains(state));
    // A row resting where the move arrives reads the same whether or not the move was taken.
    let leaves = |state: &StateName| {
        moves.is_none_or(|move_| move_.from.contains(state) && *state != move_.to)
    };
    let target = selection
        .row(actors, taken, &leaves, true)
        .or_else(|| moves.and_then(|_| selection.row(actors, taken, &leaves, false)))
        .or_else(|| {
            (moves.is_some() && writes)
                .then(|| selection.row(actors, taken, &movable, true))
                .flatten()
        })
        .ok_or(
            "has no row the pin's other conjuncts select, resting where the effect visibly \
             changes it, that the arranging commands can produce",
        )?;
    let beside = |state: &StateName| *state == target.state;
    let decoy = selection
        .row(actors, taken, &beside, true)
        .or_else(|| selection.row(actors, taken, &beside, false))
        .ok_or(
            "has no second row the pin's other conjuncts select in the target's state, so a \
             target changing a row the pin does not name reads the same",
        )?;
    let mut named = Vec::new();
    if selection.filter != Predicate::Always {
        for miss in misses(&selection.filter)
            .into_iter()
            .zip(misses(&selection.closed))
        {
            named.push(
                selection
                    .row_meeting(actors, taken, &miss, &movable, true)
                    .ok_or(
                        "has no row leaving out exactly one other conjunct of the pinned filter \
                         that the arranging commands can produce",
                    )?,
            );
        }
    }
    if let Some(move_) = moves {
        let elsewhere = |state: &StateName| !move_.from.contains(state) && *state != move_.to;
        let outside = |state: &StateName| !move_.from.contains(state);
        if let Some(skipped) = selection
            .row(actors, taken, &elsewhere, false)
            .or_else(|| selection.row(actors, taken, &outside, true))
        {
            named.push(skipped);
        }
    }
    Ok(Pinned {
        target,
        decoy,
        named,
    })
}

/// What one send names through the pin.
#[derive(Clone, Copy)]
enum Names<'r> {
    /// An arranged row.
    Row(&'r InstanceName),
    /// An identity no row carries.
    Unheld,
    /// Nothing: the `Optional` source left absent.
    Nothing,
}

/// One send of the command under test: the subject it acts on, what it supplies, and the identity
/// no row carries where it names one.
struct Send {
    setup: Option<Setup>,
    supplied: BTreeMap<String, ScenarioValue>,
    unheld: Option<Node>,
}

/// The subject of a send arranged so that its stored fields `bound` names the captured rows and
/// `holds` accepts it, resting where the branch is taken; `None` where no declared command does.
#[allow(clippy::too_many_arguments)] // The arrangement search's own inputs.
fn pinned_subject(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    steer: &Predicate,
    bound: &BTreeMap<String, InstanceName>,
    holds: &dyn Fn(&Arrangement) -> bool,
) -> Result<Option<Setup>, RefusalCause> {
    let subject = outcome
        .subject
        .as_ref()
        .expect("a subject pin is checked to have a subject");
    let initial = &ir.entity(&subject.entity).lifecycle.initial;
    let targets: Vec<&StateName> = match &subject.effect {
        ResolvedEffect::Moves { transition } => transition.from.iter().collect(),
        _ => vec![initial],
    };
    let distinction = next(ir, &subject.entity, taken);
    let accept = |row: &Arrangement| {
        targets.contains(&&row.state) && holds(row) && !reaches_other_rows(ir, &row.steps)
    };
    let Some(row) = arrange_toward_bound(
        ir,
        &subject.entity,
        steer,
        actors,
        distinction,
        None,
        bound,
        &accept,
    ) else {
        return Ok(None);
    };
    let after = match &subject.effect {
        ResolvedEffect::Moves { transition } => Some(transition.to.clone()),
        ResolvedEffect::Deletes => None,
        _ => Some(row.state.clone()),
    };
    let setup = Setup {
        steps: row.steps,
        instance: Some(row.instance),
        bound: BTreeMap::new(),
        source: row.source,
        after,
        before: Some(row.state),
        settled: row.settled,
        unobserved: row.unobserved,
    };
    related::arrange(ir, outcome, actors, distinction, setup).map(Some)
}

/// Whether the command's input field, or the subject's stored field, `source` reads is `Optional`.
fn optional(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    source: &Source,
) -> bool {
    match source {
        Source::Subject(field) => outcome.subject.as_ref().is_some_and(|subject| {
            ir.entity(&subject.entity)
                .field(field)
                .is_some_and(|field| field.type_ref.is_optional())
        }),
        Source::Input(field) => command
            .input
            .iter()
            .any(|input| input.name == *field && input.type_ref.is_optional()),
    }
}

/// The send naming `names` through `source` for an `affects:` branch, or `None` where no declared
/// command arranges it.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // One send: a subject, then its input.
fn affects_send(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    input: &BTreeMap<String, Node>,
    source: &Source,
    names: Names<'_>,
) -> Result<Option<Send>, RefusalCause> {
    let subject = outcome
        .subject
        .as_ref()
        .expect("a pinned `affects:` entry is checked to have a subject");
    let mut unheld = None;
    let setup = match source {
        Source::Subject(field) => {
            let path = FactPath::from_segments([field.clone()]);
            let absent = Predicate::Not(Box::new(Predicate::Defined(path.clone())));
            // A present identity: an `Optional` field holding nothing names no row either.
            let literal = |row: &Arrangement| {
                row.settled
                    .get(field)
                    .and_then(|held| held.value.as_literal())
                    .filter(|value| **value != Node::Null)
                    .cloned()
            };
            let found = match names {
                // Steered toward a present value: a bound input is substituted only where the
                // witness sends one, and an `Optional` input left absent would name no row.
                Names::Row(row) => pinned_subject(
                    ir,
                    outcome,
                    actors,
                    taken,
                    &Predicate::Defined(path.clone()),
                    &[(field.clone(), row.clone())].into_iter().collect(),
                    &|arranged| {
                        matches!(
                            arranged.settled.get(field).map(|held| &held.value),
                            Some(ScenarioValue::Instance { instance }) if instance == row
                        )
                    },
                )?,
                Names::Unheld => pinned_subject(
                    ir,
                    outcome,
                    actors,
                    taken,
                    &Predicate::Defined(path.clone()),
                    &BTreeMap::new(),
                    &|row| literal(row).is_some(),
                )?,
                Names::Nothing => pinned_subject(
                    ir,
                    outcome,
                    actors,
                    taken,
                    &absent,
                    &BTreeMap::new(),
                    &|row| {
                        subject_fact::row_truth_with(
                            ir,
                            &subject.entity,
                            &row.settled,
                            &row.unwritten,
                            Some(&row.state),
                            &absent,
                            None,
                        ) == Truth::True
                    },
                )?,
            };
            let Some(setup) = found else {
                return Ok(None);
            };
            if matches!(names, Names::Unheld) {
                unheld = setup
                    .settled
                    .get(field)
                    .and_then(|held| held.value.as_literal())
                    .filter(|value| **value != Node::Null)
                    .cloned();
            }
            setup
        }
        Source::Input(_) => {
            prepare_in(ir, outcome, actors, None, next(ir, &subject.entity, taken))?
        }
    };
    let Some(instance) = setup.instance.clone() else {
        return Ok(None);
    };
    let mut sent = match (&setup.before, has_subject_guards(command)) {
        (Some(held), true) => reach_in_state(ir, command, outcome, held, Distinction::PLAIN)?,
        _ => input.clone(),
    };
    let mut bound = setup.bound.clone();
    if let Source::Input(field) = source {
        match names {
            Names::Row(row) => {
                bound.insert(field.clone(), row.clone());
            }
            Names::Unheld => {
                let Some(literal) = sent
                    .get(field)
                    .filter(|value| **value != Node::Null)
                    .cloned()
                else {
                    return Ok(None);
                };
                unheld = Some(literal);
            }
            Names::Nothing => {
                sent.remove(field);
                if !selects_branch(ir, command, outcome, setup.before.as_ref(), &sent)
                    .is_ok_and(|reached| reached)
                {
                    return Ok(None);
                }
            }
        }
    }
    let supplied = supply(ir, command, &sent, Some(subject), Some(&instance), &bound);
    Ok(Some(Send {
        setup: Some(setup),
        supplied,
        unheld,
    }))
}

/// The reads requiring `identity` absent from every view in `views`: a send naming an identity no
/// row carries creates no row.
fn require_unheld(
    ir: &EssIr,
    entity: &EntityHandle,
    views: &[&ResolvedView],
    identity: &Node,
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) {
    for view in views {
        let name = ViewRef::new(view.name.clone());
        steps.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        steps.push(ScenarioStep::ExpectSubjectAbsent {
            view: name.clone(),
            subject: [(
                ir.entity(entity).identity.name.clone(),
                ScenarioValue::literal(identity.clone()),
            )]
            .into_iter()
            .collect(),
        });
        source.insert(name.into());
    }
}

/// The views a pinned effect's rows are read back from, or a refusal naming why there are none.
fn views_for<'i>(
    ir: &'i EssIr,
    entity: &EntityHandle,
    at: String,
    (moves, sets, removes): (bool, &[ess_compiler::ir::ResolvedPayloadField], bool),
) -> Result<Vec<&'i ResolvedView>, RefusalCause> {
    if removes {
        observed_removal(ir, entity, at)
    } else {
        observed(ir, entity, at, (moves, sets))
    }
}

/// The `affects:` segment for a branch whose one filtered entry pins the row's identity: every
/// row arranged, then every subject, then one send per row named, then the reads.
#[allow(clippy::too_many_lines)] // One complete pinned witness: rows, sends and reads.
pub(super) fn affects_segment(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    (index, affect, pin): (usize, &ResolvedAffect, &Pin),
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ir = models.arrangement;
    let at = || format!("{}.affects[{index}]", outcome.name);
    let subject = outcome.subject.as_ref().ok_or_else(|| {
        gap(
            at(),
            command.name.to_string(),
            "is declared beside no subject, so `subject.` reads nothing",
        )
    })?;
    if pin
        .rest
        .fact_paths()
        .into_iter()
        .any(|path| path.namespace() == SUBJECT_NAMESPACE)
    {
        return Err(gap(
            at(),
            affect.entity.to_string(),
            "pins the row's identity and reads the subject in another conjunct; the pinned \
             arrangement settles the subject after the row, so this cut does not witness it",
        ));
    }
    if matches!(pin.source, Source::Subject(_)) && subject.effect == ResolvedEffect::Creates {
        return Err(gap(
            at(),
            affect.entity.to_string(),
            "pins the row's identity to a field of a subject the branch creates, which holds \
             nothing before the outcome",
        ));
    }
    let views = views_for(
        ir,
        &affect.entity,
        at(),
        (affect.moves.is_some(), &affect.sets, affect.deletes),
    )?;
    let input = reach(ir, command, outcome, Distinction::PLAIN)?;
    let base = supply(ir, command, &input, Some(subject), None, &BTreeMap::new());
    let sets = with_sets(outcome, affect);
    let selection = Selection {
        ir,
        command,
        entity: &affect.entity,
        filter: pin.rest.clone(),
        closed: written_in(&pin.rest, &|path| input_value(path, &input)),
        input: &input,
        supplied: &base,
        sets: &sets,
        symbols: BTreeMap::new(),
        other: None,
    };
    let rows = pinned_rows(&selection, actors, taken, affect.moves.as_ref())
        .map_err(|reason| gap(at(), affect.entity.to_string(), reason))?;

    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    for row in rows.all() {
        steps.extend(row.steps.iter().cloned());
        source.extend(row.source.iter().cloned());
    }
    source.insert(EntityRef::from(&affect.entity).into());

    let mut sends = Vec::new();
    let first = affects_send(
        ir,
        command,
        outcome,
        actors,
        taken,
        &input,
        &pin.source,
        Names::Row(&rows.target.instance),
    )?
    .ok_or_else(|| {
        let field = match &pin.source {
            Source::Subject(field) => format!("subject.{field}"),
            Source::Input(field) => format!("input.{field}"),
        };
        gap(
            format!("{}: {field}", at()),
            subject.entity.to_string(),
            "is written from an input by no command that arranges the subject, so no subject \
             can be arranged to name the row the filter pins",
        )
    })?;
    sends.push(first);
    for row in &rows.named {
        if let Some(send) = affects_send(
            ir,
            command,
            outcome,
            actors,
            taken,
            &input,
            &pin.source,
            Names::Row(&row.instance),
        )? {
            sends.push(send);
        }
    }
    let mut optional_names = vec![Names::Unheld];
    if optional(ir, command, outcome, &pin.source) {
        optional_names.push(Names::Nothing);
    }
    for names in optional_names {
        if let Some(send) = affects_send(
            ir,
            command,
            outcome,
            actors,
            taken,
            &input,
            &pin.source,
            names,
        )? {
            sends.push(send);
        }
    }
    for send in &sends {
        if let Some(setup) = &send.setup {
            steps.extend(setup.steps.iter().cloned());
            source.extend(setup.source.iter().cloned());
        }
    }
    models.mark(InvocationPhase::Arrange, &mut steps);
    for send in &sends {
        invocation(
            models,
            command,
            outcome,
            actors,
            send.supplied.clone(),
            &mut steps,
            &mut source,
        );
    }
    for send in &sends {
        let Some(setup) = &send.setup else {
            continue;
        };
        let Some(instance) = &setup.instance else {
            continue;
        };
        let left = after(ir, outcome, &send.supplied, &setup.settled);
        steps.extend(subject_reads(
            ir,
            &subject.entity,
            instance,
            setup.after.as_ref(),
            &left,
            &mut source,
        ));
    }
    read_back(
        ir,
        &affect.entity,
        &views,
        &rows.rows(),
        (affect.moves.as_ref().map(|move_| &move_.to), affect.deletes),
        &sets,
        &sends[0].supplied,
        &mut steps,
        &mut source,
    );
    for identity in sends.iter().filter_map(|send| send.unheld.as_ref()) {
        require_unheld(
            ir,
            &affect.entity,
            &views,
            identity,
            &mut steps,
            &mut source,
        );
    }
    Ok((steps, source))
}

/// The `instances:` scenario for a filter pinning the row's identity to an input: one send per
/// row named, `{count: changed}` reading 1 for the target and 0 for every other send, then the
/// reads.
#[allow(clippy::too_many_lines)] // One complete pinned witness: rows, sends and reads.
pub(super) fn set_scenario(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    set: &ResolvedSetSubject,
    pin: &Pin,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let ir = models.arrangement;
    let entity = &set.entity;
    let at = || format!("{}.instances", outcome.name);
    let Source::Input(field) = &pin.source else {
        return Err(gap(
            at(),
            entity.to_string(),
            "pins the row's identity to a subject field, and an `instances:` branch has no \
             subject",
        ));
    };
    let moves = set.effect.transition();
    let removes = set.effect == ResolvedEffect::Deletes;
    let views = views_for(ir, entity, at(), (moves.is_some(), &outcome.sets, removes))?;
    let input = reach(ir, command, outcome, Distinction::PLAIN)?;
    let supplied = supply(ir, command, &input, None, None, &BTreeMap::new());
    let selection = Selection {
        ir,
        command,
        entity,
        filter: pin.rest.clone(),
        closed: written_in(&pin.rest, &|path| input_value(path, &input)),
        input: &input,
        supplied: &supplied,
        sets: outcome,
        symbols: BTreeMap::new(),
        other: None,
    };
    let rows = pinned_rows(&selection, actors, &mut BTreeSet::new(), moves)
        .map_err(|reason| gap(at(), entity.to_string(), reason))?;

    let mut steps = Vec::new();
    let mut source: BTreeSet<EssSemanticRef> = BTreeSet::new();
    for row in rows.all() {
        steps.extend(row.steps.iter().cloned());
        source.extend(row.source.iter().cloned());
    }
    source.insert(EntityRef::from(entity).into());
    models.mark(InvocationPhase::Arrange, &mut steps);

    let naming = |row: &InstanceName| {
        let mut request = supplied.clone();
        request.insert(field.clone(), ScenarioValue::instance(row.clone()));
        request
    };
    let mut sends = vec![(naming(&rows.target.instance), 1)];
    sends.extend(rows.named.iter().map(|row| (naming(&row.instance), 0)));
    let unheld = input
        .get(field)
        .filter(|value| **value != Node::Null)
        .cloned();
    if let Some(literal) = &unheld {
        let mut request = supplied.clone();
        request.insert(field.clone(), ScenarioValue::literal(literal.clone()));
        sends.push((request, 0));
    }
    if optional(ir, command, outcome, &pin.source) {
        let mut without = input.clone();
        without.remove(field);
        if selects_branch(ir, command, outcome, None, &without).is_ok_and(|reached| reached) {
            let mut request = supplied.clone();
            request.remove(field);
            sends.push((request, 0));
        }
    }
    for (request, changed) in &sends {
        invocation(
            models,
            command,
            outcome,
            actors,
            request.clone(),
            &mut steps,
            &mut source,
        );
        answer(
            ir,
            command,
            outcome,
            request,
            if *changed == 0 {
                &[]
            } else {
                std::slice::from_ref(&rows.target)
            },
            &mut steps,
            &mut source,
        )?;
    }
    read_back(
        ir,
        entity,
        &views,
        &rows.rows(),
        (moves.map(|move_| &move_.to), removes),
        outcome,
        &sends[0].0,
        &mut steps,
        &mut source,
    );
    if let Some(literal) = &unheld {
        require_unheld(ir, entity, &views, literal, &mut steps, &mut source);
    }
    let text = format!(
        "`{}` {} the `{}` its input names by identity and no other row, and reports how many",
        command.name,
        set.effect.verb(),
        ir.entity(entity).name
    );
    Ok(ConformanceScenario::new(
        super::super::clipped(&text),
        steps,
        source,
    ))
}
