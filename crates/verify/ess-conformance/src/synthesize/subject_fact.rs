//! Bounded arrangement of a row to a goal over its stored fields.
//!
//! One strategy for both `when_subject:` shapes — `{field, equals}` (ess/6) runs here as the
//! one-leaf predicate `field == equals` — rewritten in the four parts
//! `docs/design/cross-record-and-stored-field-guards.md` ("Conformance: arranging a row to a goal")
//! names:
//!
//! 1. **Goal-directed input choice through `sets:` mappings.** Every guarded branch's stored-field
//!    predicate is translated through a driver's `sets:` input mappings onto that driver's input,
//!    and the witness search of [`candidates`] is asked for inputs over the translation — so the
//!    creating command is sent `weight_kg: 21` because the guard compares with `20`. A driver's own
//!    guards still have to select its outcome. A literal `sets:` is a fixed point; a field written
//!    through a conversion, or from nothing the search can set, is not arrangeable and the branch
//!    is refused with `ESS-SYNTH-001` naming it.
//! 2. **Goal values from the guard's own literals**, by the rule [`candidates`] already applies to
//!    input: a number at `n`, `n + 1`, `n - 1`, `0` and `-1`, an enum at each variant, a
//!    `Timestamp` at the instant written and a second either side.
//! 3. **A typed fact source over the arranged row**: the determined values are bound against the
//!    entity's declared fields and evaluated by the one evaluator validation and the runners use,
//!    so an `Integer` compares as a number and a `Timestamp` by its instant.
//! 4. **A bounded search** whose node is the lifecycle state crossed with how every guarded
//!    branch's predicate — and each of its leaves — decides over the row. Two arrangements that
//!    decide everything alike are one node; the cap of 64 nodes stays.
//!
//! Where several arranged rows reach the branch at one depth, the one whose values satisfy the
//! most leaves of the guards, and then sit on the most of their literals, is taken: a success
//! beside `service == Express and weight_kg > 20` is witnessed with an Express parcel of 20 kg, the
//! boundary, so an implementation that refuses every Express parcel fails it.
use super::{
    absorb, candidates, created, decides, flatten, has_subject_guards, invoke, invoke_with,
    map_paths, not_emitted, require, shows, state_default, supply, when, ActorRef, Arrangement,
    AssertionStyle, BTreeMap, BTreeSet, CommandRef, Distinction, Driver, EntityHandle, EntityRef,
    EntitySpec, EssIr, EssSemanticRef, FactPath, InstanceNeed, OutcomeRef, Predicate,
    QualifiedName, RefusalCause, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedInstance, ResolvedOutcome, ResolvedPayloadValue, ResolvedSubject, ScenarioStep,
    ScenarioValue, Setup, Truth, Unreachable, ViewExpectation, ViewRef, WitnessGap,
};
use ess_domain::entity::Cardinality;
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand};

/// The most search nodes one arrangement visits before it refuses.
const MAX_NODES: usize = 64;

/// Whether any branch of this command reads the existing subject's stored fields.
pub(super) fn uses(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectField { .. } | ResolvedCondition::SubjectPredicate { .. }
        )
    })
}

/// Whether any branch of this command uses the ess/9 predicate form.
pub(super) fn uses_predicate(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectPredicate { .. }
        )
    })
}

/// What a branch requires of the stored fields, as one predicate over them.
fn stored(condition: &ResolvedCondition) -> Option<Predicate> {
    match condition {
        ResolvedCondition::SubjectPredicate { predicate, .. } => Some(predicate.clone()),
        ResolvedCondition::SubjectField { field, equals, .. } => Some(Predicate::Compare {
            left: Operand::Fact(FactPath::new(field).ok()?),
            op: CompareOp::Eq,
            right: Operand::Literal(ess_primitives::facts::FactValue::text(equals.clone())),
        }),
        ResolvedCondition::When { .. }
        | ResolvedCondition::SubjectState { .. }
        | ResolvedCondition::StateChange { .. }
        | ResolvedCondition::Otherwise
        | ResolvedCondition::ExternalWhen { .. }
        | ResolvedCondition::External { .. }
        | ResolvedCondition::Related { .. }
        | ResolvedCondition::WrongState
        | ResolvedCondition::UnknownInstance
        | ResolvedCondition::InputAbsent
        | ResolvedCondition::ExistingInstance => None,
    }
}

/// What a branch requires of the input.
fn input_guard(condition: &ResolvedCondition) -> Option<&Predicate> {
    match condition {
        ResolvedCondition::When { predicate } => Some(predicate),
        ResolvedCondition::SubjectField { predicate, .. } => predicate.as_ref(),
        ResolvedCondition::SubjectPredicate { input, .. } => input.as_ref(),
        _ => None,
    }
}

/// The existing subject a subject-fact command reads: the one its subject-bearing branches name.
pub(super) fn common(command: &ResolvedCommand) -> Option<&ResolvedSubject> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .find(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }))
}

/// Whether this strategy arranges the scenario for `outcome`.
///
/// Every branch of a command reading stored fields that an arranged row decides: the guarded ones,
/// the default and any input-guarded sibling — including a refusal that names no subject of its
/// own and reads the command's. Faults, wrong-state refusals and replays keep their own families.
pub(super) fn routes(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    uses(command)
        && outcome.replays.is_none()
        && !matches!(
            outcome.condition,
            ResolvedCondition::External { .. }
                | ResolvedCondition::ExternalWhen { .. }
                | ResolvedCondition::WrongState
        )
        && (outcome.subject.is_some() || common(command).is_some())
        && !reads_identity(command, outcome)
}

/// Whether `outcome` is an input-guarded refusal whose guard reads the identity field that names
/// the subject (beyond10x/ess#178): `id-required: ticket_id == ""`.
///
/// No arranged row can be sent for it, because sending the row replaces the value its own guard
/// admits with the row's identity. It is sent as a plain invocation, with the input its guard
/// admits, and is taken before any row would be read.
pub(super) fn reads_identity(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    let Some(guard) = super::is_input_guarded_refusal(outcome)
        .then(|| when(outcome))
        .flatten()
    else {
        return false;
    };
    let Some(ResolvedInstance::Supplied { field }) =
        common(command).map(|subject| &subject.instance)
    else {
        return false;
    };
    guard
        .fact_paths()
        .iter()
        .any(|path| path.namespace() == field.name)
}

/// The subject this outcome's scenario arranges: its own, or the one its siblings name.
pub(super) fn reading<'a>(
    command: &'a ResolvedCommand,
    outcome: &'a ResolvedOutcome,
) -> Option<&'a ResolvedSubject> {
    outcome
        .subject
        .as_ref()
        .filter(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }))
        .or_else(|| common(command))
}

/// The branches that compete for selection: everything but the default and the families decided
/// elsewhere.
///
/// An `unknown_instance:` answer is one of those families: it is taken for an identity no row
/// carries, never for a row the arrangement built, and counting it beside the guarded branches
/// made every row select two of them.
fn guarded(command: &ResolvedCommand) -> impl Iterator<Item = &ResolvedOutcome> {
    command.outcomes.iter().filter(|branch| {
        !state_default(branch)
            && !matches!(
                branch.condition,
                ResolvedCondition::External { .. }
                    | ResolvedCondition::ExternalWhen { .. }
                    | ResolvedCondition::WrongState
                    | ResolvedCondition::UnknownInstance
                    | ResolvedCondition::InputAbsent
                    | ResolvedCondition::ExistingInstance
            )
    })
}

/// Every guarded branch's stored-field predicate, in declaration order.
fn hints(command: &ResolvedCommand) -> Vec<Predicate> {
    guarded(command)
        .filter_map(|branch| stored(&branch.condition))
        .collect()
}

/// The stored fields any guarded branch of this command reads, in name order.
pub(super) fn guarded_fields(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
) -> BTreeSet<String> {
    read_fields(ir, entity, &hints(command))
}

/// The stored fields one predicate reads.
pub(super) fn read_by(
    ir: &EssIr,
    entity: &EntityHandle,
    predicate: &Predicate,
) -> BTreeSet<String> {
    read_fields(ir, entity, std::slice::from_ref(predicate))
}

/// The stored fields these predicates read, in name order.
fn read_fields(ir: &EssIr, entity: &EntityHandle, predicates: &[Predicate]) -> BTreeSet<String> {
    let declared = ir.entity(entity);
    predicates
        .iter()
        .flat_map(Predicate::fact_paths)
        .map(|path| path.namespace().to_owned())
        .filter(|root| declared.fields.iter().any(|field| &field.name == root))
        .collect()
}

fn missing(entity: &EntityHandle, field: &str, reason: &'static str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path: format!("{entity}.{field}"),
        type_ref: entity.to_string(),
        reason,
    })
}

/// How one stored-field predicate decides over the arranged row.
///
/// Only the values the arrangement determined as literals are bound, and bound against the
/// entity's declared fields, so each is read at its declared type. A predicate reading a field the
/// arrangement did not determine is `Unknown` — never decided against an absent binding, because
/// the row does hold *something* there and the specification does not say what.
///
/// `held` is the lifecycle state the row rests in, which a predicate reading `state` decides over
/// (ess/18, beyond10x/ess#204); without it such a predicate is `Unknown`, for the same reason.
pub(super) fn row_truth(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    held: Option<&super::StateName>,
    predicate: &Predicate,
) -> Truth {
    row_truth_with(ir, entity, settled, held, predicate, None)
}

/// Whether a stored-field predicate reads the held lifecycle state as `state` (ess/18,
/// beyond10x/ess#204): the bare path, which no declared field can shadow.
pub(super) fn reads_held_state(ir: &EssIr, entity: &EntityHandle, predicate: &Predicate) -> bool {
    !ir.entity(entity)
        .fields
        .iter()
        .any(|field| field.name == EntitySpec::STATE)
        && ess_domain::command::subject_fact::reads_state(predicate)
}

/// The rows a wrong state `held` of `command` is answered on by guarded branches reading `state`
/// (ess/18, beyond10x/ess#204): for each such branch whose stored predicate is not false with
/// `state` bound to `held` alone, in declaration order, a row of `entity` arranged in `held` under a
/// distinction of its own, observed there, and sent an input the branch is selected by on it.
///
/// Guarded branches select before `wrong_state:` applies (the #192 ruling), so these rows belong in
/// the state's `<entity>/state/<S>/refuses/<command>` scenario beside the plain wrong-state row,
/// which is witnessed only where some input still reaches it. Every step asserts the branch taken
/// and its error or its absence; the branch's effects are its own outcome scenario's.
///
/// `Ok` with no steps where no guarded branch reads `state`. A branch that may be selected in `held`
/// and that no bounded arrangement reaches refuses the whole with that cause: the state is never
/// left silently unwitnessed.
pub(super) fn state_answered_rows(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    held: &super::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    let Ok(path) = FactPath::new(EntitySpec::STATE) else {
        return Ok((steps, source));
    };
    let mut facts = ess_primitives::facts::FactStore::new();
    facts.set_if_absent(
        path,
        ess_primitives::facts::FactValue::text(held.to_string()),
    );
    let hints = hints(command);
    let command_ref = CommandRef::new(command.name.clone());
    let mut rows = 0;
    for branch in guarded(command) {
        let Some(predicate) = stored(&branch.condition) else {
            continue;
        };
        if !reads_held_state(ir, entity, &predicate)
            || predicate.evaluate(&facts) == Truth::False
            || reading(command, branch).is_none_or(|subject| &subject.entity != entity)
        {
            continue;
        }
        rows += 1;
        let (arrangement, input) = search(
            ir,
            entity,
            actors,
            &hints,
            Distinction::further(rows),
            "state",
            |node| {
                if &node.state != held {
                    return Ok(None);
                }
                reach_at(ir, command, branch, entity, node)
            },
        )?;
        let (observed, view) =
            observe_fields(ir, entity, &read_by(ir, entity, &predicate), &arrangement)?;
        steps.extend(arrangement.steps);
        steps.extend(observed);
        source.extend(arrangement.source);
        source.insert(view.into());
        let outcome_ref = OutcomeRef::new(command_ref.clone(), branch.name.clone());
        steps.push(ScenarioStep::ExecuteCommand {
            caller: std::collections::BTreeMap::new(),
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
            input: supply(
                command,
                &input,
                reading(command, branch),
                Some(&arrangement.instance),
                &BTreeMap::new(),
            ),
        });
        steps.push(ScenarioStep::ExpectOutcome {
            outcome: outcome_ref.clone(),
        });
        match &branch.error {
            Some(error) => {
                steps.push(ScenarioStep::ExpectError {
                    error: super::ErrorRef::from(error),
                    fields: BTreeMap::new(),
                });
                steps.push(ScenarioStep::ExpectNoEvents);
            }
            None => steps.push(ScenarioStep::ExpectNoError),
        }
        source.insert(outcome_ref.into());
    }
    Ok((steps, source))
}

/// [`row_truth`], with the command's input bound under `input.` for a predicate that compares the
/// row with it (beyond10x/ess#157). Without an input such a comparison is `Unknown`, which is what
/// the search and the boundary goals see: the row alone does not decide it.
pub(super) fn row_truth_with(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    held: Option<&super::StateName>,
    predicate: &Predicate,
    input: Option<(&ResolvedCommand, &BTreeMap<String, Node>)>,
) -> Truth {
    let declared = ir.entity(entity);
    let mut fields = declared.fields.clone();
    let mut values: BTreeMap<String, Node> = settled
        .iter()
        .filter_map(|(name, determined)| {
            determined
                .value
                .as_literal()
                .map(|value| (name.clone(), value.clone()))
        })
        .collect();
    // A link to the owner compared with an input naming an arranged owner is bound as that
    // owner's token, which the input carries too (beyond10x/ess#193).
    if let Some((command, sent)) = input {
        values.extend(link_facts(ir, command, entity, settled, predicate, sent));
    }
    for path in predicate.fact_paths() {
        let root = path.namespace();
        if declared.fields.iter().any(|field| field.name == root) && !values.contains_key(root) {
            return Truth::Unknown;
        }
    }
    // The held state is bound as `state` at the lifecycle's own type, beside the stored fields.
    if reads_held_state(ir, entity, predicate) {
        let Some(held) = held else {
            return Truth::Unknown;
        };
        fields.push(declared.state_field());
        values.insert(EntitySpec::STATE.to_owned(), Node::Text(held.to_string()));
    }
    let Ok(store) = crate::input::bind(ir, &fields, &values, crate::input::Completeness::Partial)
    else {
        return Truth::Unknown;
    };
    let row = crate::input::TypedFacts::new(ir, &fields, store);
    let input = match input {
        Some((command, values)) if reads_input(ir, entity, predicate) => {
            match flatten(ir, command, values) {
                Ok(facts) => Some(facts),
                Err(_) => return Truth::Unknown,
            }
        }
        _ => None,
    };
    predicate.evaluate(&RowAndInput { row, input })
}

/// The distinction the second owner a link comparison names is arranged under: past every further
/// instance an arrangement or a view's companion rows number (those stop at
/// [`MAX_CANDIDATES`](super::MAX_CANDIDATES)), so its name is never one they capture as well.
const OTHER_OWNER: Distinction = Distinction::further(super::MAX_CANDIDATES + 1);

/// Every `==` or `!=` a guarded branch's stored predicate writes between the row's link to its
/// owner and an input field of the owner's identity type (beyond10x/ess#193): the input field, and
/// the link field it is compared with.
///
/// `account_id != input.account_id` asks whether the caller names the owner the row was filed
/// under. Neither side is a value the specification spells — the owner is the instance the
/// arrangement created, and the caller names one — so the input is sent as an arranged instance:
/// the row's own owner, or a second one arranged beside it ([`linked_inputs`]).
fn links(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let Some(owned) = ir.owner_of(entity) else {
        return out;
    };
    let Some(link) = ir
        .entity(entity)
        .fields
        .iter()
        .find(|field| field.name == owned.via)
    else {
        return out;
    };
    let mut found = Vec::new();
    for hint in hints(command) {
        leaves(&hint, &mut found);
    }
    for leaf in found.iter().filter(|leaf| reads_input(ir, entity, leaf)) {
        let Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq | CompareOp::Ne,
            right: Operand::Fact(right),
        } = leaf
        else {
            continue;
        };
        for (row, other) in [(left, right), (right, left)] {
            let Some(sent) = input_path(other).filter(|sent| sent.segments().len() == 1) else {
                continue;
            };
            let typed = command
                .input
                .iter()
                .find(|field| field.name == sent.namespace())
                .is_some_and(|field| field.type_ref.required() == link.type_ref.required());
            if typed && row.segments().len() == 1 && row.namespace() == link.name {
                out.insert((sent.namespace().to_owned(), link.name.clone()));
            }
        }
    }
    out
}

/// Whether any of `predicates` holds a link comparison of `command` ([`links`]): an `==` or `!=`
/// between the row's link to its owner and the input naming an owner. Such a predicate is decided
/// only with the input bound, so every search toward it runs over [`linked_inputs`] (beyond10x/ess
/// #193). A goal over it past [`MAX_BOUNDARIES`] is refused, and so is one the bounded search did not
/// reach where an input naming an arranged owner left it undecided ([`goal_input`]); one every
/// candidate decided and no bounded row meets adds no row, as for every other guard.
fn compares_link(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    predicates: &[Predicate],
) -> bool {
    let pairs = links(ir, command, entity);
    if pairs.is_empty() {
        return false;
    }
    let mut found = Vec::new();
    for predicate in predicates {
        leaves(predicate, &mut found);
    }
    let pair = |row: &FactPath, other: &FactPath| {
        pairs.iter().any(|(sent, field)| {
            row.segments().len() == 1
                && row.namespace() == field
                && input_path(other).is_some_and(|rest| rest.segments() == [sent.clone()])
        })
    };
    found.iter().any(|leaf| {
        matches!(
            leaf,
            Predicate::Compare {
                left: Operand::Fact(left),
                op: CompareOp::Eq | CompareOp::Ne,
                right: Operand::Fact(right),
            } if pair(left, right) || pair(right, left)
        )
    })
}

/// For each of `inputs`, whether an undecided goal on it counts as one the search could not decide:
/// an input naming an arranged owner ([`linked`]), or any input where none of them names one. A
/// plain candidate beside inputs that name an owner leaves a link comparison undecided by
/// construction, and says nothing about whether the goal's row exists.
fn naming_owner(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    inputs: &[BTreeMap<String, Node>],
) -> Vec<bool> {
    let names: Vec<bool> = inputs
        .iter()
        .map(|input| !linked(ir, command, entity, &arrangement.settled, input).is_empty())
        .collect();
    let any = names.contains(&true);
    names.into_iter().map(|named| named || !any).collect()
}

/// The value an input field naming `instance` is chosen at while the search decides a link
/// comparison: a token of the instance at the field's type, which no two instances share. It is
/// never sent — [`prepare`] sends the instance itself in its place, through `Setup::bound`.
fn token(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    instance: &super::InstanceName,
) -> Option<Node> {
    let path = FactPath::new(field).ok()?;
    let declared = |primitive| crate::input::declared_as(ir, &command.input, &path, primitive);
    let text = format!("instance:{instance}");
    if declared(ess_domain::types::Primitive::Uuid) {
        Some(Node::Text(crate::witness::uuid_of(&text)))
    } else if declared(ess_domain::types::Primitive::String) {
        Some(Node::Text(text))
    } else {
        None
    }
}

/// The owner a link comparison's other side names, and what it is called: an instance of the
/// entity that owns `entity`, arranged under [`OTHER_OWNER`].
fn other_owner(ir: &EssIr, entity: &EntityHandle) -> Option<(EntityHandle, super::InstanceName)> {
    let owned = ir.owner_of(entity)?;
    let name = super::instance_name(&ir.entity(&owned.owner).name, OTHER_OWNER);
    Some((owned.owner, name))
}

/// The arranged owner each link input of `input` names, by input field: the row's own owner, or
/// the [`other_owner`], where the input carries its [`token`]. A field carrying anything else — a
/// literal nobody assigned — names no owner, and the comparison stays undecided.
fn linked(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    input: &BTreeMap<String, Node>,
) -> BTreeMap<String, super::InstanceName> {
    let other = other_owner(ir, entity).map(|(_, name)| name);
    let mut out = BTreeMap::new();
    for (sent, field) in links(ir, command, entity) {
        let Some(ScenarioValue::Instance { instance: own }) =
            settled.get(&field).map(|held| &held.value)
        else {
            continue;
        };
        let Some(value) = input.get(&sent) else {
            continue;
        };
        let known: Vec<(&super::InstanceName, Option<Node>)> = std::iter::once(own)
            .chain(other.iter().filter(|other| *other != own))
            .map(|name| (name, token(ir, command, &sent, name)))
            .collect();
        // Two owners whose tokens coincide cannot be told apart, so neither is named.
        if let [(_, first), (_, second)] = known.as_slice() {
            if first == second {
                continue;
            }
        }
        if let Some((name, _)) = known
            .iter()
            .find(|(_, token)| token.as_ref() == Some(value))
        {
            out.insert(sent, (*name).clone());
        }
    }
    out
}

/// The link fields [`row_truth_with`] binds for `predicate` and this input: each link whose input
/// names an arranged owner, at the row's own owner's [`token`] — where `predicate` reads the link
/// and that input only in `==` or `!=` between the two, or the link in `defined()`. Anything else
/// asked of either — an ordering, a literal, a text test — is a question about the identity's
/// value, which no token answers, and leaves the predicate `Unknown` as before.
fn link_facts(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    predicate: &Predicate,
    input: &BTreeMap<String, Node>,
) -> BTreeMap<String, Node> {
    let named = linked(ir, command, entity, settled, input);
    let mut out = BTreeMap::new();
    for (sent, field) in links(ir, command, entity) {
        let Some(ScenarioValue::Instance { instance: own }) =
            settled.get(&field).map(|held| &held.value)
        else {
            continue;
        };
        if !named.contains_key(&sent) || !only_compared(ir, entity, predicate, &field, &sent) {
            continue;
        }
        if let Some(token) = token(ir, command, &sent, own) {
            out.insert(field, token);
        }
    }
    out
}

/// Whether every leaf of `predicate` reading the row's `field` or `input.<sent>` is an `==` or `!=`
/// between exactly those two, or `defined(field)`.
fn only_compared(
    ir: &EssIr,
    entity: &EntityHandle,
    predicate: &Predicate,
    field: &str,
    sent: &str,
) -> bool {
    let mut found = Vec::new();
    leaves(predicate, &mut found);
    let row = |path: &FactPath| path.segments().len() == 1 && path.namespace() == field;
    let input =
        |path: &FactPath| input_path(path).is_some_and(|rest| rest.segments() == [sent.to_owned()]);
    let touches = |path: &FactPath| {
        path.namespace() == field
            || (reads_input(ir, entity, predicate)
                && input_path(path).is_some_and(|rest| rest.namespace() == sent))
    };
    found.iter().all(|leaf| match leaf {
        Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq | CompareOp::Ne,
            right: Operand::Fact(right),
        } if (row(left) && input(right)) || (input(left) && row(right)) => true,
        Predicate::Defined(path) if row(path) => true,
        other => !other.fact_paths().into_iter().any(&touches),
    })
}

/// Whether a stored-field predicate compares the row with the command's input: a path rooted at
/// `input.`, where the entity declares no field of that name (beyond10x/ess#157).
fn reads_input(ir: &EssIr, entity: &EntityHandle, predicate: &Predicate) -> bool {
    let namespace = ess_domain::command::subject_fact::INPUT_NAMESPACE;
    !ir.entity(entity)
        .fields
        .iter()
        .any(|field| field.name == namespace)
        && predicate
            .fact_paths()
            .iter()
            .any(|path| path.namespace() == namespace && path.segments().len() > 1)
}

/// The path under `input.` a stored-field predicate reads, as the input path it names.
fn input_path(path: &FactPath) -> Option<FactPath> {
    let (root, rest) = path.segments().split_first()?;
    (root == ess_domain::command::subject_fact::INPUT_NAMESPACE && !rest.is_empty())
        .then(|| FactPath::from_segments(rest))
}

/// The arranged row, and the input a scenario sends read under `input.`: the one fact source a
/// stored-field predicate over both is evaluated against, each half at its declared types.
struct RowAndInput<'a> {
    row: crate::input::TypedFacts<'a>,
    input: Option<crate::InputFacts<'a>>,
}

impl RowAndInput<'_> {
    fn split(&self, path: &FactPath) -> Option<(&crate::InputFacts<'_>, FactPath)> {
        let input = self.input.as_ref()?;
        input_path(path).map(|rest| (input, rest))
    }
}

impl ess_primitives::facts::FactSource for RowAndInput<'_> {
    fn fact(&self, path: &FactPath) -> Option<ess_primitives::facts::FactValue> {
        match self.split(path) {
            Some((input, rest)) => input.fact(&rest),
            None => self.row.fact(path),
        }
    }

    /// `defined()` over an `Optional` struct, list or map reads the presence the row or the input
    /// recorded for it, which no fact carries (beyond10x/ess#176).
    fn present(&self, path: &FactPath) -> bool {
        match self.split(path) {
            Some((input, rest)) => input.present(&rest),
            None => self.row.present(path),
        }
    }

    fn scales(&self) -> &ess_primitives::facts::Scales {
        self.row.scales()
    }

    fn orders_as_instant(&self, path: &FactPath) -> bool {
        match self.split(path) {
            Some((input, rest)) => input.orders_as_instant(&rest),
            None => self.row.orders_as_instant(path),
        }
    }

    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        match self.split(path) {
            Some((input, rest)) => input.orders_text_by_bytes(&rest),
            None => self.row.orders_text_by_bytes(path),
        }
    }
}

/// `predicate` with every comparison between the row and the input replaced by `Always`: what the
/// arranging branches' input search is handed, because an `input.` path names nothing of theirs.
fn without_input(ir: &EssIr, entity: &EntityHandle, predicate: &Predicate) -> Predicate {
    match predicate {
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .map(|child| without_input(ir, entity, child))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .iter()
                .map(|child| without_input(ir, entity, child))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(without_input(ir, entity, inner))),
        leaf if reads_input(ir, entity, leaf) => Predicate::Always,
        other => other.clone(),
    }
}

/// Every comparison between the row and the input, grounded on this arrangement: the stored side
/// replaced by the value the row holds, the input side by the input path it names. Handed to the
/// candidate search beside the command's own guards, its literals are the values the input is
/// tried at — the row's value, and by rule 3 its neighbours — so one candidate names the stored
/// value and another does not, and the guard is witnessed both ways (beyond10x/ess#157). A
/// comparison inside a quantifier over a stored list or map is grounded once per element the row
/// holds ([`ground_leaf`], beyond10x/ess#240).
pub(super) fn grounded(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    predicates: &[Predicate],
) -> Vec<Predicate> {
    let mut found = Vec::new();
    for predicate in predicates {
        leaves(predicate, &mut found);
    }
    let mut out = Vec::new();
    for leaf in found.iter().filter(|leaf| reads_input(ir, entity, leaf)) {
        ground_leaf(settled, &[], leaf, &mut out);
    }
    out
}

/// The value the row holds at `path`, reading a quantifier's binder as the element it is bound to
/// (innermost first): a struct member by name, a list element by its ordinal.
fn held_node(
    settled: &BTreeMap<String, super::Determined>,
    bound: &[(&str, &Node)],
    path: &FactPath,
) -> Option<Node> {
    let (root, rest) = path.segments().split_first()?;
    let mut node = match bound.iter().rev().find(|(name, _)| name == root) {
        Some((_, element)) => *element,
        None => settled.get(root)?.value.as_literal()?,
    };
    for segment in rest {
        node = match node {
            Node::Map(entries) => entries.get(segment)?,
            Node::Seq(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(node.clone())
}

/// One leaf of a row/input comparison, grounded on the row: a comparison with its stored side
/// replaced by the value held there; a quantifier over a stored collection once per element the row
/// holds, its binder read as that element (beyond10x/ess#240). A map's elements are its values in
/// key order — what the quantifier binds — and a list's its elements, so `exists r in
/// redirect_uris: r == input.application` over a row holding `{k: v}` grounds `v == application`,
/// and the input is tried at `v`, which satisfies it, and at its neighbours, which do not. An empty
/// collection grounds nothing: the input cannot move a quantifier over it.
fn ground_leaf(
    settled: &BTreeMap<String, super::Determined>,
    bound: &[(&str, &Node)],
    leaf: &Predicate,
    out: &mut Vec<Predicate>,
) {
    let side = |operand: &Operand| -> Option<Operand> {
        match operand {
            Operand::Fact(path) if !bound.iter().any(|(name, _)| *name == path.namespace()) => {
                match input_path(path) {
                    Some(rest) => Some(Operand::Fact(rest)),
                    None => held_node(settled, bound, path)
                        .as_ref()
                        .and_then(super::fact_value)
                        .map(Operand::Literal),
                }
            }
            Operand::Fact(path) => held_node(settled, bound, path)
                .as_ref()
                .and_then(super::fact_value)
                .map(Operand::Literal),
            Operand::Literal(value) => Some(Operand::Literal(value.clone())),
        }
    };
    match leaf {
        Predicate::Compare { left, op, right } => {
            if let (Some(left), Some(right)) = (side(left), side(right)) {
                // Only a comparison the input takes part in steers the input.
                if matches!(left, Operand::Fact(_)) || matches!(right, Operand::Fact(_)) {
                    out.push(Predicate::Compare {
                        left,
                        op: *op,
                        right,
                    });
                }
            }
        }
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let elements = match held_node(settled, bound, &quantified.over) {
                Some(Node::Map(entries)) => entries.into_values().collect(),
                Some(Node::Seq(items)) => items,
                _ => Vec::new(),
            };
            let mut body = Vec::new();
            leaves(&quantified.body, &mut body);
            for element in &elements {
                let mut inner = bound.to_vec();
                inner.push((quantified.bind.as_str(), element));
                for leaf in &body {
                    ground_leaf(settled, &inner, leaf, out);
                }
            }
        }
        _ => {}
    }
}

/// The distinction the second and third entries of a spread collection are built at, past every
/// further instance an arrangement numbers ([`MAX_CANDIDATES`](super::MAX_CANDIDATES)) and the
/// second owner ([`OTHER_OWNER`]), so an entry never repeats a value another row of the scenario
/// holds. A row at distinction `d` takes `SPREAD + 2d` and `SPREAD + 2d + 1`.
const SPREAD: usize = 2 * (super::MAX_CANDIDATES + 2);

/// The quantifiers of `predicates` whose collection is stored on the row and whose body compares
/// an element with the command's input (beyond10x/ess#240): the leaves [`spread`] arranges several
/// entries for and [`witnesses_elements`] holds a scenario to.
fn elementwise(ir: &EssIr, entity: &EntityHandle, predicates: &[Predicate]) -> Vec<Predicate> {
    let declared = ir.entity(entity);
    let mut found = Vec::new();
    for predicate in predicates {
        leaves(predicate, &mut found);
    }
    found.retain(|leaf| match leaf {
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            input_path(&quantified.over).is_none()
                && declared
                    .fields
                    .iter()
                    .any(|field| field.name == quantified.over.namespace())
                && reads_input(ir, entity, leaf)
        }
        _ => false,
    });
    found
}

/// Whether any of `predicates` is an [`elementwise`] quantifier.
pub(super) fn has_elementwise(ir: &EssIr, entity: &EntityHandle, predicates: &[Predicate]) -> bool {
    !elementwise(ir, entity, predicates).is_empty()
}

/// Whether the row `node` and `input` witness every [`elementwise`] quantifier of `predicates`
/// element by element (beyond10x/ess#240).
///
/// A quantifier one element decides — an `exists` that holds, a `forall` that fails — must read
/// otherwise over the collection's first element alone, over its last alone, and as the other
/// quantifier, so a target reading only the first value, only the last, or `forall` for `exists`
/// (and the reverse) answers this scenario wrongly. One the whole collection decides — an `exists`
/// that fails, a `forall` that holds — must hold two or more elements, so it is not decided by one
/// value standing in for all of them. A quantifier the row does not decide is not held to either.
pub(super) fn witnesses_elements(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    predicates: &[Predicate],
    node: &Arrangement,
    input: &BTreeMap<String, Node>,
) -> bool {
    let truth = |settled: &BTreeMap<String, super::Determined>, leaf: &Predicate| {
        row_truth_with(
            ir,
            entity,
            settled,
            Some(&node.state),
            leaf,
            Some((command, input)),
        )
    };
    elementwise(ir, entity, predicates).iter().all(|leaf| {
        let (Predicate::Forall(quantified) | Predicate::Exists(quantified)) = leaf else {
            return true;
        };
        let exists = matches!(leaf, Predicate::Exists(_));
        let held = truth(&node.settled, leaf);
        let count = match held_node(&node.settled, &[], &quantified.over) {
            Some(Node::Map(entries)) => entries.len(),
            Some(Node::Seq(items)) => items.len(),
            _ => return true,
        };
        let one_decides = match held {
            Truth::True => exists,
            Truth::False => !exists,
            Truth::Unknown => return true,
        };
        if !one_decides {
            return count >= 2;
        }
        let other = if exists {
            Predicate::Forall(quantified.clone())
        } else {
            Predicate::Exists(quantified.clone())
        };
        let differs = |truth: Truth| truth != held && truth != Truth::Unknown;
        differs(truth(&node.settled, &other))
            && [false, true].into_iter().all(|last| {
                one_element(&node.settled, &quantified.over, last)
                    .is_some_and(|settled| differs(truth(&settled, leaf)))
            })
    })
}

/// `settled` with the stored collection at `path` cut to its first element, or its `last` — a map
/// to its first or last entry in key order, the order a quantifier walks it in.
fn one_element(
    settled: &BTreeMap<String, super::Determined>,
    path: &FactPath,
    last: bool,
) -> Option<BTreeMap<String, super::Determined>> {
    let (root, rest) = path.segments().split_first()?;
    let mut out = settled.clone();
    let held = out.get_mut(root)?;
    let mut value = held.value.as_literal()?.clone();
    let mut at = &mut value;
    for segment in rest {
        at = match at {
            Node::Map(members) => members.get_mut(segment)?,
            _ => return None,
        };
    }
    match at {
        Node::Map(entries) => {
            let kept = if last {
                entries.pop_last()?
            } else {
                entries.pop_first()?
            };
            *entries = BTreeMap::from([kept]);
        }
        Node::Seq(items) => {
            let kept = if last {
                items.pop()?
            } else {
                items.first()?.clone()
            };
            *items = vec![kept];
        }
        _ => return None,
    }
    held.value = ScenarioValue::Literal { value };
    Some(out)
}

/// Inputs for `driver` that write each stored collection an [`elementwise`] quantifier of `hints`
/// reads with several entries rather than the witness's one (beyond10x/ess#240): the witness input
/// at `distinction`, with the collection's input set to three entries in three shapes — every value
/// distinct, every value the first, and the first value either side of another. Over those an
/// `exists` finds a row where the one matching value is neither first nor last, a `forall` one where
/// several values all satisfy it and one where the one failing value sits in the middle, which is
/// what [`witnesses_elements`] asks of the row a scenario is arranged on.
///
/// The further entries are the witnesses at [`SPREAD`], so no value repeats one another row holds.
/// Empty where no such quantifier reads a field `driver` writes from its input, or where the input
/// cannot hold two distinct entries (a map keyed by a `Boolean` holds two at most).
fn spread(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    hints: &[Predicate],
    distinction: Distinction,
) -> Vec<BTreeMap<String, Node>> {
    let quantifiers = elementwise(ir, entity, hints);
    if quantifiers.is_empty() {
        return Vec::new();
    }
    let mapping = mapped(driver.outcome);
    let guards: Vec<&Predicate> = driver
        .command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .collect();
    let at = |distinction: Distinction| {
        candidates(ir, driver.command, &guards, distinction)
            .ok()?
            .into_iter()
            .next()
    };
    let Some(base) = at(distinction) else {
        return Vec::new();
    };
    let further = [
        at(Distinction::further(SPREAD + 2 * distinction.get())),
        at(Distinction::further(SPREAD + 2 * distinction.get() + 1)),
    ];
    let mut shapes = vec![base.clone(), base.clone(), base.clone()];
    let mut written = BTreeSet::new();
    for leaf in &quantifiers {
        let (Predicate::Forall(quantified) | Predicate::Exists(quantified)) = leaf else {
            continue;
        };
        let Some(sent) = mapping.get(quantified.over.namespace()) else {
            continue;
        };
        let mut path = vec![(*sent).to_owned()];
        path.extend(quantified.over.segments()[1..].iter().cloned());
        if !written.insert(path.clone()) {
            continue;
        }
        let witnesses = std::iter::once(Some(&base))
            .chain(further.iter().map(Option::as_ref))
            .flatten()
            .filter_map(|input| node_at(input, &path));
        let Some(collections) = spread_collection(witnesses) else {
            continue;
        };
        for (shape, collection) in shapes.iter_mut().zip(collections) {
            set_at(shape, &path, collection);
        }
    }
    if written.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<BTreeMap<String, Node>> = Vec::new();
    for shape in shapes {
        if shape != base && !out.contains(&shape) {
            out.push(shape);
        }
    }
    out
}

/// The three collections [`spread`] writes, from the first entry of each witness collection: every
/// value distinct, every value the first, and the first value either side of the second. `None`
/// where the witnesses give fewer than two distinct entries — a map's keys collide, or every value
/// is one value.
fn spread_collection<'n>(witnesses: impl Iterator<Item = &'n Node>) -> Option<[Node; 3]> {
    let mut keys = BTreeSet::new();
    let mut values = Vec::new();
    let mut map = false;
    for collection in witnesses {
        match collection {
            Node::Map(entries) => {
                map = true;
                if let Some((key, value)) = entries.first_key_value() {
                    if keys.insert(key.clone()) {
                        values.push(value.clone());
                    }
                }
            }
            Node::Seq(items) => values.extend(items.first().cloned()),
            _ => {}
        }
    }
    let (first, middle) = match values.as_slice() {
        [first, middle, ..] if values.iter().any(|value| value != first) => {
            (first.clone(), middle.clone())
        }
        _ => return None,
    };
    let odd = (0..values.len())
        .map(|index| {
            if index == 1 {
                middle.clone()
            } else {
                first.clone()
            }
        })
        .collect();
    let layouts = [values.clone(), vec![first; values.len()], odd];
    Some(layouts.map(|layout| {
        if map {
            Node::Map(keys.iter().cloned().zip(layout).collect())
        } else {
            Node::Seq(layout)
        }
    }))
}

/// The node at `path` in a command input, through struct members.
fn node_at<'a>(input: &'a BTreeMap<String, Node>, path: &[String]) -> Option<&'a Node> {
    let (root, rest) = path.split_first()?;
    let mut node = input.get(root)?;
    for segment in rest {
        node = match node {
            Node::Map(members) => members.get(segment)?,
            _ => return None,
        };
    }
    Some(node)
}

/// `input` with the node at `path` replaced, where every step to it exists.
fn set_at(input: &mut BTreeMap<String, Node>, path: &[String], value: Node) {
    let Some((root, rest)) = path.split_first() else {
        return;
    };
    let Some(mut node) = input.get_mut(root) else {
        return;
    };
    for segment in rest {
        node = match node {
            Node::Map(members) => match members.get_mut(segment) {
                Some(member) => member,
                None => return,
            },
            _ => return,
        };
    }
    *node = value;
}

/// The inputs tried against `arrangement` for one command reading stored fields: the witness
/// search over the command's own guards, and over every row/input comparison grounded on the row.
fn inputs_for(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Vec<BTreeMap<String, Node>>, RefusalCause> {
    let grounded = grounded(ir, entity, &arrangement.settled, &hints(command));
    let mut guards: Vec<&Predicate> = command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .collect();
    guards.extend(grounded.iter());
    let mut inputs =
        candidates(ir, command, &guards, Distinction::PLAIN).map_err(RefusalCause::NoWitness)?;
    // The row was arranged from plain witnesses, so a plain input may carry the very value the row
    // holds on every candidate — a text has no neighbour rule 3 could offer. A further witness
    // starts every leaf from another base value, which is the input a comparison with the row
    // needs for its other side. Only where such a comparison exists: every other command keeps the
    // candidates, and so the suites, it had.
    if !grounded.is_empty() {
        if let Ok(further) = candidates(ir, command, &guards, Distinction::further(1)) {
            inputs.extend(further);
        }
    }
    Ok(inputs)
}

/// [`inputs_for`], and each of them once more for every link comparison ([`links`]) on a row whose
/// link holds an arranged owner (beyond10x/ess#193): sent naming that owner, and — for each link
/// in turn — naming the [`other_owner`] instead. So an `!=` and an `==` between the link and the
/// input are each decided both ways, and a branch on either side is reached.
///
/// Only [`prepare`] offers these, because only it sends the owner named in their place: every other
/// arrangement keeps the candidates, and so the suites, it had.
fn linked_inputs(
    ir: &EssIr,
    command: &ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Vec<BTreeMap<String, Node>>, RefusalCause> {
    let mut inputs = inputs_for(ir, command, entity, arrangement)?;
    let owned: Vec<(String, &super::InstanceName)> = links(ir, command, entity)
        .into_iter()
        .filter_map(
            |(sent, field)| match &arrangement.settled.get(&field)?.value {
                ScenarioValue::Instance { instance } => Some((sent, instance)),
                _ => None,
            },
        )
        .collect();
    if owned.is_empty() {
        return Ok(inputs);
    }
    let other = other_owner(ir, entity).map(|(_, name)| name);
    let mut more = Vec::new();
    for input in &inputs {
        let mut same = input.clone();
        for (sent, own) in &owned {
            if let Some(token) = token(ir, command, sent, own) {
                same.insert(sent.clone(), token);
            }
        }
        more.push(same.clone());
        for (sent, _) in &owned {
            if let Some(token) = other
                .as_ref()
                .and_then(|other| token(ir, command, sent, other))
            {
                let mut apart = same.clone();
                apart.insert(sent.clone(), token);
                more.push(apart);
            }
        }
    }
    // A token that is not a value of the field's type (a text newtype narrower than the token) is
    // not a candidate: the comparison then stays undecided and is refused as it was.
    inputs.extend(
        more.into_iter()
            .filter(|input| flatten(ir, command, input).is_ok()),
    );
    Ok(inputs)
}

/// [`reach_at`] over [`linked_inputs`]. Only an input `accept` takes of the row is offered.
fn reach_linked(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    accept: &dyn Fn(&Arrangement, &BTreeMap<String, Node>) -> bool,
) -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
    for input in linked_inputs(ir, command, entity, arrangement)? {
        if selects(ir, command, entity, arrangement, &input)?
            .is_some_and(|branch| branch.name == outcome.name)
            && accept(arrangement, &input)
        {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// The owners the chosen `input` names through a link comparison, as `Setup::bound` sends them, and
/// the second owner arranged where it names that one and `present` says this scenario has not
/// arranged it yet (beyond10x/ess#193). An owner that cannot be arranged refuses the branch, naming
/// it: its side of the guard would otherwise go unwitnessed.
///
/// The second owner is given a row of `entity` of its own ([`row_under`]), so the owner the refused
/// side names holds rows too, just not this one: a target asking whether the named owner holds
/// *any* row, rather than this row, fails. Not under a `cardinality: one` owner relation where
/// `outcome`, the branch sent, writes the link from an input naming the second owner
/// ([`files_under`]): the branch then files this row under that owner, which would hold two, a
/// state the relation says no owner reaches.
#[allow(clippy::too_many_arguments)]
fn bind_links(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    arrangement: &mut Arrangement,
    input: &BTreeMap<String, Node>,
    present: &mut bool,
) -> Result<BTreeMap<String, super::InstanceName>, RefusalCause> {
    let bound = linked(ir, command, entity, &arrangement.settled, input);
    let crowded = ir.owner_of(entity).is_some_and(|owned| {
        owned.relation.cardinality == Cardinality::One
            && other_owner(ir, entity)
                .is_some_and(|(_, other)| files_under(outcome, owned.via, &bound, &other))
    });
    if let Some((owner, other)) = other_owner(ir, entity)
        .filter(|(_, other)| !*present && bound.values().any(|named| named == other))
    {
        let initial = ir.entity(&owner).lifecycle.initial.clone();
        let arranged = super::arrange_first(
            ir,
            &owner,
            std::slice::from_ref(&initial),
            actors,
            OTHER_OWNER,
            &[entity],
        )
        .map_err(|reason| RefusalCause::InstanceRequired {
            entity: EntityRef::from(&owner),
            need: InstanceNeed::InState { state: initial },
            reason,
        })?;
        assert_eq!(
            arranged.instance, other,
            "an owner is named after its entity and distinction"
        );
        arrangement.steps.extend(arranged.steps);
        arrangement.source.extend(arranged.source);
        if let Some(row) = (!crowded)
            .then(|| row_under(ir, entity, actors, &arranged.instance, &arranged.state))
            .flatten()
        {
            arrangement.steps.extend(row.steps);
            arrangement.source.extend(row.source);
        }
        *present = true;
    }
    Ok(bound)
}

/// Whether `outcome` writes the link field `via` from an input that `bound` sends as `owner`: the
/// branch files the row it names under that owner.
fn files_under(
    outcome: &ResolvedOutcome,
    via: &str,
    bound: &BTreeMap<String, super::InstanceName>,
    owner: &super::InstanceName,
) -> bool {
    outcome.sets.iter().any(|set| {
        set.target == via
            && matches!(
                &set.value,
                ResolvedPayloadValue::InputField { field, .. }
                    if bound.get(field) == Some(owner)
            )
    })
}

/// One row of `entity` created under the arranged `owner`, named under [`OTHER_OWNER`], by the
/// first creating branch that names the owner from its input and can be run — `None` where none
/// can. The owner holds no row before it, so a `cardinality: one` relation admits this one; the
/// caller ([`bind_links`]) does not ask for it where the branch it sends would then file a second
/// row under the same owner.
fn row_under(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    owner: &super::InstanceName,
    owner_state: &super::StateName,
) -> Option<Arrangement> {
    let via = ir.owner_of(entity)?.via;
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .find_map(|creator| {
            let field = creator
                .outcome
                .sets
                .iter()
                .find_map(|set| match &set.value {
                    ResolvedPayloadValue::InputField { field, .. }
                        if set.target == via && set.conversion.is_none() =>
                    {
                        Some(field.clone())
                    }
                    _ => None,
                })?;
            let under = (
                field,
                Arrangement {
                    instance: owner.clone(),
                    state: owner_state.clone(),
                    steps: Vec::new(),
                    source: BTreeSet::new(),
                    settled: BTreeMap::new(),
                },
            );
            super::created_owned(
                ir,
                entity,
                creator,
                actors,
                OTHER_OWNER,
                &[],
                Some(&under),
                None,
            )
            .ok()
        })
}

/// Which branch this command selects for the row `arrangement` holds and this input, if exactly
/// one does.
///
/// A row whose state no move of the command starts from is the wrong-state family's
/// ([`refusal_witness`]) and is not answered here, although its stored fields do select a guarded
/// branch there. An `Unknown` stored fact selects no branch, and never the default.
fn selects<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    input: &BTreeMap<String, Node>,
) -> Result<Option<&'a ResolvedOutcome>, RefusalCause> {
    let wrong = ir
        .wrong_states(command)
        .get(&entity)
        .is_some_and(|states| states.contains(&arrangement.state));
    // A guarded branch is selected before `wrong_state` applies (beyond10x/ess#192), and one
    // whose predicate reads `state` (ess/18, #204) may name a state no move starts from: such a
    // row is answered here when that branch is the one it selects. Every other wrong-state row is
    // the wrong-state family's, as before.
    let reads_state = |branch: &ResolvedOutcome| {
        stored(&branch.condition).is_some_and(|predicate| reads_held_state(ir, entity, &predicate))
    };
    if wrong && !guarded(command).any(reads_state) {
        return Ok(None);
    }
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    let mut selected = Vec::new();
    for branch in guarded(command) {
        if let Some(predicate) = stored(&branch.condition) {
            match row_truth_with(
                ir,
                entity,
                &arrangement.settled,
                Some(&arrangement.state),
                &predicate,
                Some((command, input)),
            ) {
                Truth::True => {}
                Truth::False => continue,
                Truth::Unknown => return Ok(None),
            }
        }
        if let Some(guard) = input_guard(&branch.condition) {
            if !decides(&facts, &[guard], true)? {
                continue;
            }
        }
        selected.push(branch);
    }
    // An input-guarded refusal is taken before any accepting branch it overlaps (beyond10x/ess
    // #178), so where one is selected the accepting branches beside it are not.
    if selected
        .iter()
        .any(|branch| super::is_input_guarded_refusal(branch))
    {
        selected.retain(|branch| super::is_input_guarded_refusal(branch));
    }
    let pick = match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    };
    if wrong && !pick.is_some_and(reads_state) {
        return Ok(None);
    }
    Ok(pick.filter(|branch| {
        branch
            .subject
            .as_ref()
            .and_then(|subject| subject.effect.transition())
            .is_none_or(|transition| transition.from.contains(&arrangement.state))
    }))
}

/// Whether an input alone selects `outcome` of a command that reads no stored field.
pub(super) fn input_selects(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: &BTreeMap<String, Node>,
) -> Result<bool, RefusalCause> {
    let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
    let mut selected = Vec::new();
    for branch in guarded(command) {
        let Some(guard) = when(branch) else {
            continue;
        };
        if decides(&facts, &[guard], true)? {
            selected.push(branch);
        }
    }
    if selected
        .iter()
        .any(|branch| super::is_input_guarded_refusal(branch))
    {
        selected.retain(|branch| super::is_input_guarded_refusal(branch));
    }
    let pick = match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    };
    Ok(pick.is_some_and(|branch| branch.name == outcome.name))
}

/// The row and input a wrong-state scenario sends a command reading stored fields
/// (beyond10x/ess#173, #192): an input the moving branch `outcome`'s own input guard admits, where
/// it declares one, sent to a row in `state` on which no sibling branch is selected.
///
/// A guarded branch is selected in any state before `wrong_state` applies — the stored fields do
/// select there (`docs/design/cross-record-and-stored-field-guards.md`, *Wrong state*; Entity
/// Runtime orders guarded branches before the wrong-state one). So every sibling is missed:
///
/// * a branch guarded by its input alone — a plain `when:`, an input-guarded refusal (#178) — only
///   through its input;
/// * a branch guarded by the stored row alone — `held: when_subject: history == Pending` — only
///   through the row, which must decide its guard false;
/// * a branch that needs both — a `when:` beside a `when_subject:` — through either: its input half
///   refuted, or, where no candidate refutes every input half, its subject half decided false by
///   the row. `already-confirmed: history == Confirmed, token != ""` beside `token-required: token
///   == ""` is missed by `token != ""` on a row whose `history` is not `Confirmed`.
///
/// A sibling that moves the subject along a transition not starting from `state` needs no
/// refuting: selected or not, the move is the wrong-state answer.
///
/// A candidate refuting every input half is preferred, so a witness that already did stays the one
/// it was. The row is `arrangement` where it serves, and otherwise the first row in `state` the
/// declared drivers leave that does — found by the search [`prepare`] arranges subject-fact
/// branches with, steered by the command's own stored guards. Where no row serves, the scenario is
/// refused (ESS-SYNTH-003, naming the guards) rather than written to depend on evaluation order.
/// Every stored guard the witness relies on the row for is observed before the command, as
/// [`prepare`] observes it: the row is a fact the scenario is about, not one it assumes.
#[allow(clippy::too_many_arguments)]
pub(super) fn refusal_witness(
    ir: &EssIr,
    entity: &EntityHandle,
    state: &super::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    arrangement: Arrangement,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<(Arrangement, BTreeMap<String, Node>), RefusalCause> {
    let (mut arrangement, input, relied) =
        match refusal_input(ir, entity, &arrangement, command, outcome, distinction) {
            Ok((input, relied)) => (arrangement, input, relied),
            Err(cause) => {
                let hints = hints(command);
                let found = search(
                    ir,
                    entity,
                    actors,
                    &hints,
                    Distinction::PLAIN,
                    "state",
                    |node| {
                        if &node.state != state {
                            return Ok(None);
                        }
                        Ok(refusal_input(ir, entity, node, command, outcome, distinction).ok())
                    },
                );
                let Ok((row, (input, relied))) = found else {
                    return Err(cause);
                };
                (row, input, relied)
            }
        };
    if !relied.is_empty() {
        let fields = read_fields(ir, entity, &relied);
        let (steps, view) = observe_fields(ir, entity, &fields, &arrangement)?;
        arrangement.steps.extend(steps);
        arrangement.source.insert(view.into());
    }
    Ok((arrangement, input))
}

/// The input [`refusal_witness`] sends to `arrangement`'s row, and the stored guards of the
/// siblings it misses through that row rather than through its input: every row-only sibling's,
/// and a mixed sibling's where its input half holds.
fn refusal_input(
    ir: &EssIr,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<(BTreeMap<String, Node>, Vec<Predicate>), RefusalCause> {
    let guards: Vec<&Predicate> = command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .collect();
    let own: Vec<&Predicate> = input_guard(&outcome.condition).into_iter().collect();
    let branches: Vec<&ResolvedOutcome> = command
        .outcomes
        .iter()
        .filter(|other| other.name != outcome.name && !state_default(other))
        .filter(|other| input_guard(&other.condition).is_some())
        .collect();
    let siblings: Vec<&Predicate> = branches
        .iter()
        .filter_map(|other| input_guard(&other.condition))
        .collect();
    // A sibling selected by the stored row alone — `held: when_subject: history == Pending` — is
    // taken in any state before `wrong_state` applies, and no input refutes it: only the row can.
    // A sibling moving the subject along a transition that does not start from the row's state is
    // answered by `wrong_state` whether or not its guard selects it — Entity Runtime lowers no
    // held-state guard onto a stored-field branch, and a move from a state it does not leave is the
    // wrong-state answer — so the row need not refute it (`rushed: urgent and not fast` on the row
    // `rush` itself left in `Rushed`).
    let answered_by_state = |other: &ResolvedOutcome| {
        other
            .subject
            .as_ref()
            .and_then(|own| own.effect.transition())
            .is_some_and(|transition| !transition.from.contains(&arrangement.state))
    };
    let row_only: Vec<&ResolvedOutcome> = guarded(command)
        .filter(|other| other.name != outcome.name && input_guard(&other.condition).is_none())
        .filter(|other| stored(&other.condition).is_some() && !answered_by_state(other))
        .collect();
    // What the row must refute, where the input leaves it to the row: every row-only sibling's
    // stored guard, and a mixed sibling's where its input half holds.
    let halves: Vec<Predicate> = row_only
        .iter()
        .chain(&branches)
        .filter_map(|other| stored(&other.condition))
        .collect();
    // The row the stored guards read is the command's subject; a row of another entity decides
    // none of them.
    let on_row = common(command).is_some_and(|subject| &subject.entity == entity);
    let falsified = |branch: &ResolvedOutcome, input: &BTreeMap<String, Node>| {
        stored(&branch.condition).filter(|predicate| {
            on_row
                && row_truth_with(
                    ir,
                    entity,
                    &arrangement.settled,
                    Some(&arrangement.state),
                    predicate,
                    Some((command, input)),
                ) == Truth::False
        })
    };
    let inputs = candidates(ir, command, &guards, distinction).map_err(RefusalCause::NoWitness)?;
    let mut through_row = None;
    let mut lost_on_row = false;
    'candidates: for input in &inputs {
        let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
        if !decides(&facts, &own, true)? {
            continue;
        }
        let mut relied = Vec::new();
        for branch in &row_only {
            let Some(predicate) = falsified(branch, input) else {
                lost_on_row = true;
                continue 'candidates;
            };
            relied.push(predicate);
        }
        if decides(&facts, &siblings, false)? {
            return Ok((input.clone(), relied));
        }
        if through_row.is_some() {
            continue;
        }
        for branch in &branches {
            let Some(guard) = input_guard(&branch.condition) else {
                continue;
            };
            if decides(&facts, &[guard], false)? {
                continue;
            }
            if stored(&branch.condition).is_some() && answered_by_state(branch) {
                continue;
            }
            let Some(predicate) = falsified(branch, input) else {
                lost_on_row |= stored(&branch.condition).is_some();
                continue 'candidates;
            };
            relied.push(predicate);
        }
        through_row = Some((input.clone(), relied));
    }
    if let Some(found) = through_row {
        return Ok(found);
    }
    Err(refusal_unsatisfied(
        entity,
        &own,
        &siblings,
        lost_on_row.then_some(halves.as_slice()),
        inputs.len().min(super::MAX_CANDIDATES),
    ))
}

/// ESS-SYNTH-003 for [`refusal_input`]: the input guards no candidate satisfied, and — where some
/// candidate was lost only to the row — the stored guards the row had to refute.
fn refusal_unsatisfied(
    entity: &EntityHandle,
    own: &[&Predicate],
    siblings: &[&Predicate],
    halves: Option<&[Predicate]>,
    tried: usize,
) -> RefusalCause {
    let mut named = own.to_vec();
    named.extend(siblings.iter().copied());
    let mut rendered = match (own.is_empty(), siblings.is_empty()) {
        (true, true) => String::new(),
        (false, true) => super::rendered(own, true),
        (true, false) => super::rendered(siblings, false),
        (false, false) => format!(
            "{} and {}",
            super::rendered(own, true),
            super::rendered(siblings, false)
        ),
    };
    if let Some(halves) = halves {
        named.extend(halves.iter());
        let row = format!(
            "a `{entity}` row in this state refuting every one of: {}",
            halves
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        rendered = if rendered.is_empty() {
            row
        } else {
            format!("{rendered}, on {row}")
        };
    }
    super::unsatisfied(&named, rendered, tried)
}

/// The stored fields `outcome`'s `sets:` fills from an input field, and the input field each reads.
///
/// A conversion says two types may meet and not what it computes, so a field crossing one is not a
/// field an input can be chosen for; a literal is a fixed point the search takes as it is.
fn mapped(outcome: &ResolvedOutcome) -> BTreeMap<&str, &str> {
    outcome
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
        .filter_map(|set| match &set.value {
            ResolvedPayloadValue::InputField { field, .. } => {
                Some((set.target.as_str(), field.as_str()))
            }
            _ => None,
        })
        .collect()
}

/// The candidate inputs for one arranging branch, varied toward the stored-field goal.
///
/// Every hint is translated through the branch's `sets:` mappings onto its own input, and the
/// translations are handed to the witness search beside the command's own guards: the literals the
/// guard writes become the values the mapped input fields are tried at. `None` when the branch
/// maps none of the fields the hints read, because then no choice of its input moves the row.
fn hinted(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    hints: &[Predicate],
) -> Result<Option<Vec<BTreeMap<String, Node>>>, RefusalCause> {
    let mapping = mapped(driver.outcome);
    // A comparison with the command's input (ess/15) is decided by the input the branch under
    // test sends, not by the row an arranging branch leaves, so it steers nothing here.
    let hints: Vec<Predicate> = hints
        .iter()
        .map(|hint| without_input(ir, entity, hint))
        .collect();
    let translated: Vec<Predicate> = hints
        .iter()
        .filter(|hint| {
            hint.fact_paths()
                .iter()
                .any(|path| mapping.contains_key(path.namespace()))
        })
        .map(|hint| {
            map_paths(
                hint,
                &|path: &FactPath| match mapping.get(path.namespace()) {
                    Some(input) => {
                        let mut segments = vec![(*input).to_owned()];
                        segments.extend(path.segments()[1..].iter().cloned());
                        FactPath::from_segments(segments)
                    }
                    None => path.clone(),
                },
            )
        })
        .collect();
    if translated.is_empty() {
        return Ok(None);
    }
    let mut guards: Vec<&Predicate> = driver
        .command
        .outcomes
        .iter()
        .filter_map(|branch| input_guard(&branch.condition))
        .collect();
    guards.extend(translated.iter());
    candidates(ir, driver.command, &guards, Distinction::PLAIN)
        .map(Some)
        .map_err(RefusalCause::NoWitness)
}

/// Every leaf of a predicate: the comparisons, tests and quantifiers its connectives join.
fn leaves(predicate: &Predicate, out: &mut Vec<Predicate>) {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                leaves(child, out);
            }
        }
        Predicate::Not(inner) => leaves(inner, out),
        Predicate::Always | Predicate::Never => {}
        other => out.push(other.clone()),
    }
}

/// A comparison leaf rewritten as equality with its own literal: true where the row sits on the
/// boundary the guard writes.
fn on_literal(leaf: &Predicate) -> Option<Predicate> {
    let Predicate::Compare { left, right, .. } = leaf else {
        return None;
    };
    match (left, right) {
        (Operand::Fact(_), Operand::Literal(_)) | (Operand::Literal(_), Operand::Fact(_)) => {
            Some(Predicate::Compare {
                left: left.clone(),
                op: CompareOp::Eq,
                right: right.clone(),
            })
        }
        _ => None,
    }
}

/// How the row decides every hint and every leaf: the search node, and the ranking of goals.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Profile {
    state: String,
    hints: Vec<Decided>,
    leaves: Vec<Decided>,
    on_literal: Vec<bool>,
    /// For each [`elementwise`] quantifier, which of the collection's values repeat an earlier one:
    /// rows [`spread`] writes decide every hint alike and differ only here (beyond10x/ess#240).
    shapes: Vec<Vec<usize>>,
}

/// One three-valued answer, ordered so a search node can be keyed on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Decided {
    Unknown,
    False,
    True,
}

fn code(truth: Truth) -> Decided {
    match truth {
        Truth::True => Decided::True,
        Truth::False => Decided::False,
        Truth::Unknown => Decided::Unknown,
    }
}

fn profile(
    ir: &EssIr,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    hints: &[Predicate],
) -> Profile {
    let mut all = Vec::new();
    for hint in hints {
        leaves(hint, &mut all);
    }
    let truth = |predicate: &Predicate| {
        row_truth(
            ir,
            entity,
            &arrangement.settled,
            Some(&arrangement.state),
            predicate,
        )
    };
    Profile {
        state: arrangement.state.to_string(),
        hints: hints.iter().map(|hint| code(truth(hint))).collect(),
        leaves: all.iter().map(|leaf| code(truth(leaf))).collect(),
        on_literal: all
            .iter()
            .map(|leaf| on_literal(leaf).is_some_and(|eq| truth(&eq) == Truth::True))
            .collect(),
        shapes: elementwise(ir, entity, hints)
            .iter()
            .filter_map(|leaf| match leaf {
                Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                    held_node(&arrangement.settled, &[], &quantified.over)
                }
                _ => None,
            })
            .map(|collection| {
                let values: Vec<Node> = match collection {
                    Node::Map(entries) => entries.into_values().collect(),
                    Node::Seq(items) => items,
                    _ => Vec::new(),
                };
                values
                    .iter()
                    .map(|value| values.iter().position(|other| other == value).unwrap_or(0))
                    .collect()
            })
            .collect(),
    }
}

/// How close a row sits to the guards: satisfied leaves first, then leaves on their own literal.
fn score(profile: &Profile) -> (usize, usize) {
    (
        profile
            .leaves
            .iter()
            .filter(|truth| **truth == Decided::True)
            .count(),
        profile.on_literal.iter().filter(|on| **on).count(),
    )
}

/// The rows the creating branch can leave: its plain witness first, then one per input chosen
/// toward the hints.
fn creations(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    arranging: &[&EntityHandle],
) -> Result<Vec<Arrangement>, RefusalCause> {
    let required = |reason| RefusalCause::InstanceRequired {
        entity: EntityRef::from(entity),
        need: InstanceNeed::Updates,
        reason,
    };
    let mut out =
        vec![created(ir, entity, creator, actors, distinction, arranging, None).map_err(required)?];
    if has_subject_guards(creator.command)
        || creator.outcome.test_strategy == ess_domain::command::TestStrategy::InjectFault
    {
        return Ok(out);
    }
    // And the rows whose stored collections hold several entries, where a quantifier over one
    // compares its elements with the input (beyond10x/ess#240).
    let mut inputs = hinted(ir, entity, creator, hints)?.unwrap_or_default();
    inputs.extend(spread(ir, entity, creator, hints, distinction));
    for input in inputs {
        if input_selects(ir, creator.command, creator.outcome, &input)? {
            if let Ok(arrangement) = created(
                ir,
                entity,
                creator,
                actors,
                distinction,
                arranging,
                Some(&input),
            ) {
                out.push(arrangement);
            }
        }
    }
    Ok(out)
}

/// The row after `driver` runs on `arrangement` with this input.
fn advanced(
    ir: &EssIr,
    driver: &Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    input: &BTreeMap<String, Node>,
    before: Vec<ScenarioStep>,
    no_error: bool,
) -> Arrangement {
    let mut next = arrangement.clone();
    next.steps.extend(before);
    let invoked = invoke_with(
        ir,
        driver,
        Some(&arrangement.instance),
        actors,
        &BTreeMap::new(),
        input,
    );
    next.steps.extend(invoked.steps);
    if no_error {
        next.steps.push(ScenarioStep::ExpectNoError);
    }
    next.source.extend(invoked.source);
    absorb(&mut next.settled, driver.outcome, invoked.settled);
    if let Some(transition) = driver.effect.transition() {
        next.state = transition.to.clone();
    }
    next
}

/// Every row one arranging branch can leave from `arrangement`, toward the hints.
///
/// A branch of a command that itself reads stored fields is taken only with an input the row
/// selects it for, and the facts it reads are observed first. Any other branch is run with its
/// plain witness, and — where its `sets:` maps a field the hints read — with each input chosen
/// toward them that its own guards still select it for.
fn successors(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    arranging: &[&EntityHandle],
) -> Vec<Arrangement> {
    let mut out = Vec::new();
    if uses(driver.command) {
        let own = self::hints(driver.command);
        let fields = read_fields(ir, entity, &own);
        let Ok((observed, _)) = observe_fields(ir, entity, &fields, arrangement) else {
            return out;
        };
        let mut inputs = inputs_for(ir, driver.command, entity, arrangement).unwrap_or_default();
        if let Ok(Some(more)) = hinted(ir, entity, driver, hints) {
            inputs.extend(more);
        }
        for input in inputs {
            if selects(ir, driver.command, entity, arrangement, &input)
                .ok()
                .flatten()
                .is_some_and(|branch| branch.name == driver.outcome.name)
            {
                out.push(advanced(
                    ir,
                    driver,
                    arrangement,
                    actors,
                    &input,
                    observed.clone(),
                    true,
                ));
            }
        }
        return out;
    }
    let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([entity]).collect();
    if let Ok(invoked) = invoke(
        ir,
        driver,
        Some(&arrangement.instance),
        Some(&arrangement.state),
        actors,
        Distinction::PLAIN,
        &BTreeMap::new(),
        &chain,
    ) {
        let mut next = arrangement.clone();
        next.steps.extend(invoked.steps);
        next.source.extend(invoked.source);
        absorb(&mut next.settled, driver.outcome, invoked.settled);
        if let Some(transition) = driver.effect.transition() {
            next.state = transition.to.clone();
        }
        out.push(next);
    }
    if has_subject_guards(driver.command)
        || driver.outcome.test_strategy == ess_domain::command::TestStrategy::InjectFault
        // A related-row command (ess/18) is run only with the row it reads arranged, above.
        || super::related_guard::uses(driver.command)
    {
        return out;
    }
    // A move writing a stored collection a quantifier compares with the input is also offered
    // with several entries (beyond10x/ess#240), as a creation is.
    let mut inputs = hinted(ir, entity, driver, hints)
        .ok()
        .flatten()
        .unwrap_or_default();
    inputs.extend(spread(ir, entity, driver, hints, Distinction::PLAIN));
    for input in inputs {
        if input_selects(ir, driver.command, driver.outcome, &input).unwrap_or(false) {
            out.push(advanced(
                ir,
                driver,
                arrangement,
                actors,
                &input,
                Vec::new(),
                false,
            ));
        }
    }
    out
}

/// Search the rows the declared drivers can leave for one the goal accepts.
///
/// Every creating branch is a place to start (beyond10x/ess#198): each is searched in the order
/// [`EssIr::drivers`] yields them — command name, then the command's branches as declared, the
/// IR keeping commands by name — within its own budget of [`MAX_NODES`], and the first whose rows
/// reach the goal wins. So a branch only a later creation's row selects is witnessed through that
/// creation, and a model whose first creation already reaches the goal is arranged exactly as
/// before. A creation that leaves no row, or none the goal accepts, gives way to the next, and the
/// refusal is the first creation's cause where every one fails.
fn search<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    field: &str,
    goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    search_within(ir, entity, actors, hints, distinction, field, &[], goal)
}

/// [`search`] for a further row of a scenario, under the first further distinction from `first`
/// whose arrangement binds no instance name in `taken` — the names the scenario's earlier steps
/// already bind (beyond10x/ess#193). The row the scenario's own subject was arranged on may already
/// sit on a further distinction ([`prepare`] asks `1..=FRESH_WITNESSES` for a row leaving fewer
/// writes unchanged), and a further row captured under that name again would rebind it, so the
/// closing observation of the scenario's own row would read another one.
///
/// The ordinal is searched rather than counted from a known offset, as `arrange_unbound` in the
/// parent module does, because the numbers taken are chosen by code that does not know about this
/// search. The inner result is [`search`]'s own, which the caller answers as it answers any search;
/// the outer error is a row for which every name up to [`MAX_CANDIDATES`](super::MAX_CANDIDATES)
/// is taken, which is refused whatever the goal, never skipped.
#[allow(clippy::too_many_arguments)]
fn search_unbound<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    first: usize,
    field: &str,
    taken: &BTreeSet<super::InstanceName>,
    mut goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<Result<(Arrangement, T), RefusalCause>, RefusalCause> {
    let name = &ir.entity(entity).name;
    for nth in first..=super::MAX_CANDIDATES {
        let distinction = Distinction::further(nth);
        if taken.contains(&super::instance_name(name, distinction)) {
            continue;
        }
        match search(ir, entity, actors, hints, distinction, field, &mut goal) {
            Ok(found) if !super::bound_instances(&found.0.steps).is_disjoint(taken) => {}
            found => return Ok(found),
        }
    }
    Err(RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{entity}` {field} row under an instance name no earlier step of the scenario binds"
        ),
        tried: 0,
    })
}

/// [`search`], inside an arrangement of the entities `arranging` names: a creator or a driver that
/// would need one of them again (a related row of its own entity, ess/18) stops at the cycle, and
/// the next creator is tried.
#[allow(clippy::too_many_arguments)]
pub(super) fn search_within<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    field: &str,
    arranging: &[&EntityHandle],
    mut goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    let mut first: Option<RefusalCause> = None;
    for creator in drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
    {
        match search_from(
            ir,
            entity,
            drivers,
            creator,
            actors,
            hints,
            distinction,
            field,
            arranging,
            &mut goal,
        ) {
            Ok(found) => return Ok(found),
            Err(cause) => {
                first.get_or_insert(cause);
            }
        }
    }
    Err(first.unwrap_or(RefusalCause::InstanceRequired {
        entity: EntityRef::from(entity),
        need: InstanceNeed::Updates,
        reason: Unreachable::NothingCreates,
    }))
}

/// [`search`] from the rows one creating branch can leave.
///
/// Breadth-first. At each depth every new node is offered to the goal, and of those it accepts the
/// one closest to the guards wins — ties to the earlier, so the choice is a function of the model
/// (§37).
#[allow(clippy::too_many_arguments)]
fn search_from<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    drivers: &[Driver<'_>],
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    field: &str,
    arranging: &[&EntityHandle],
    goal: &mut impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    let mut level = creations(ir, entity, creator, actors, hints, distinction, arranging)?;
    let mut seen = BTreeSet::new();
    let mut first: Option<RefusalCause> = None;
    loop {
        let mut fresh = Vec::new();
        for node in level {
            if seen.insert(profile(ir, entity, &node, hints)) {
                fresh.push(node);
            }
            if seen.len() > MAX_NODES {
                return Err(missing(
                    entity,
                    field,
                    "subject fact arrangement exceeds 64 lifecycle/fact combinations",
                ));
            }
        }
        if fresh.is_empty() {
            break;
        }
        let mut best: Option<((usize, usize), usize, T)> = None;
        for (index, node) in fresh.iter().enumerate() {
            match goal(node) {
                Ok(Some(found)) => {
                    let rank = score(&profile(ir, entity, node, hints));
                    if best.as_ref().is_none_or(|(held, ..)| rank > *held) {
                        best = Some((rank, index, found));
                    }
                }
                Ok(None) => {}
                Err(cause) => {
                    first.get_or_insert(cause);
                }
            }
        }
        if let Some((_, index, found)) = best {
            return Ok((fresh.swap_remove(index), found));
        }
        let mut next = Vec::new();
        for node in &fresh {
            for driver in drivers {
                if matches!(
                    driver.effect,
                    ResolvedEffect::Creates | ResolvedEffect::Preserves
                ) || driver
                    .effect
                    .transition()
                    .is_some_and(|transition| !transition.from.contains(&node.state))
                {
                    continue;
                }
                next.extend(successors(
                    ir, entity, driver, node, actors, hints, arranging,
                ));
            }
        }
        level = next;
    }
    // Every field the guards read can be set (`unarrangeable` checked that first), so what failed
    // is the search for values that decide the guard: a guard the candidate values cannot
    // satisfy, not a type without a value — `ESS-SYNTH-003`, with its repair, or `ESS-SYNTH-018`
    // where the guard's `.count` boundary lies past what a witness is built with.
    Err(first.unwrap_or_else(|| {
        super::unsatisfied(
            &hints.iter().collect::<Vec<_>>(),
            format!(
                "`{entity}` stored {field} selecting this branch, over the rows {} bounded \
                 arrangements left",
                seen.len()
            ),
            seen.len(),
        )
    }))
}

/// The first input that selects `outcome` for the row `arrangement` holds.
fn reach_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    arrangement: &Arrangement,
) -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
    for input in inputs_for(ir, command, entity, arrangement)? {
        if selects(ir, command, entity, arrangement, &input)?
            .is_some_and(|branch| branch.name == outcome.name)
        {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// The stored fields the goal reads that the arrangement cannot set, and why — or `None`.
///
/// Refused rather than searched for: a guarded field that no arranging branch writes from an input
/// and no literal fixes is a field the search has no way to move.
fn unarrangeable(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
) -> Option<RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    for field in fields {
        let written = drivers.iter().any(|driver| {
            driver.outcome.sets.iter().any(|set| {
                &set.target == field
                    && set.conversion.is_none()
                    && !matches!(set.value, ResolvedPayloadValue::Generated)
            })
        });
        if !written {
            return Some(missing(
                entity,
                field,
                "no arranging branch sets this stored field from an input or a literal without a \
                 conversion, so no row can be arranged to the guard",
            ));
        }
    }
    None
}

/// Records, for a row no input selects `outcome` on, every input whose own stored and input guards
/// hold of that row, and which sibling input-guarded refusal claimed it (beyond10x/ess#178). A row
/// whose state no move of the command starts from is the wrong-state family's and is skipped.
fn shadowed_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    shadow: &mut super::Shadow,
) -> Result<(), RefusalCause> {
    if outcome.error.is_some()
        || ir
            .wrong_states(command)
            .get(&entity)
            .is_some_and(|states| states.contains(&arrangement.state))
    {
        return Ok(());
    }
    let row_guard = stored(&outcome.condition);
    let own_input: Vec<&Predicate> = input_guard(&outcome.condition).into_iter().collect();
    for input in inputs_for(ir, command, entity, arrangement)? {
        if let Some(predicate) = &row_guard {
            if row_truth_with(
                ir,
                entity,
                &arrangement.settled,
                Some(&arrangement.state),
                predicate,
                Some((command, &input)),
            ) != Truth::True
            {
                continue;
            }
        }
        let facts = flatten(ir, command, &input).map_err(RefusalCause::WitnessRejected)?;
        if decides(&facts, &own_input, true)? {
            shadow.record(command, outcome, &facts)?;
        }
    }
    Ok(())
}

/// The stored-row search's refusal, restated as the shadow it is where every input the branch's
/// own guards admit on the rows searched was claimed by a sibling input-guarded refusal.
fn shadowed(
    outcome: &ResolvedOutcome,
    shadow: &super::Shadow,
    cause: RefusalCause,
) -> RefusalCause {
    let RefusalCause::GuardUnsatisfiable { tried, .. } = &cause else {
        return cause;
    };
    let row_guard = stored(&outcome.condition);
    let mut guards: Vec<&Predicate> = row_guard.iter().collect();
    guards.extend(input_guard(&outcome.condition));
    match shadow.rendered(&guards) {
        Some(predicate) => RefusalCause::GuardUnsatisfiable {
            predicate,
            tried: *tried,
        },
        None => cause,
    }
}

/// The row [`prepare`] starts from and the input that selects the branch on it, and whether the
/// two witness every [`elementwise`] quantifier of `hints` element by element
/// ([`witnesses_elements`], beyond10x/ess#240).
///
/// Where the command has such a quantifier, the search first offers only inputs that do, and the
/// plain search runs only where no bounded arrangement holds one — a quantifier whose collection no
/// input writes, or whose values the witnesses cannot spread. Every other command searches once,
/// as it always did.
fn first_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    label: &str,
) -> Result<(Arrangement, BTreeMap<String, Node>, bool), RefusalCause> {
    if has_elementwise(ir, entity, hints) {
        let strict = |node: &Arrangement, input: &BTreeMap<String, Node>| {
            witnesses_elements(ir, command, entity, hints, node, input)
        };
        if let Ok((row, input)) = search(
            ir,
            entity,
            actors,
            hints,
            Distinction::PLAIN,
            label,
            |node| reach_linked(ir, command, outcome, entity, node, &strict),
        ) {
            return Ok((row, input, true));
        }
    }
    let mut shadow = super::Shadow::default();
    let (row, input) = search(
        ir,
        entity,
        actors,
        hints,
        Distinction::PLAIN,
        label,
        |node| {
            let found = reach_linked(ir, command, outcome, entity, node, &|_, _| true)?;
            if found.is_none() {
                shadowed_at(ir, command, outcome, entity, node, &mut shadow)?;
            }
            Ok(found)
        },
    )
    .map_err(|cause| shadowed(outcome, &shadow, cause))?;
    Ok((row, input, false))
}

/// Arrange the row the branch under test is selected for, and the input that selects it.
pub(super) fn prepare(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let subject = reading(command, outcome).ok_or(RefusalCause::StrategyWithoutGuard {
        strategy: outcome.test_strategy,
    })?;
    let entity = &subject.entity;
    let hints = hints(command);
    let fields = read_fields(ir, entity, &hints);
    let label = fields.iter().cloned().collect::<Vec<_>>().join(",");
    if let Some(refusal) = unarrangeable(ir, entity, &fields) {
        return Err(refusal);
    }
    // Where a quantifier over a stored collection compares its elements with the input, the row
    // and input witness it element by element wherever some arrangement does ([`first_row`]), and
    // every refinement below keeps that.
    let (arrangement, input, elementwise) =
        first_row(ir, command, outcome, entity, actors, &hints, &label)?;
    let strict = |node: &Arrangement, input: &BTreeMap<String, Node>| {
        witnesses_elements(ir, command, entity, &hints, node, input)
    };
    let any = |_: &Arrangement, _: &BTreeMap<String, Node>| true;
    let accept: &dyn Fn(&Arrangement, &BTreeMap<String, Node>) -> bool =
        if elementwise { &strict } else { &any };
    // A row the branch's writes would leave unchanged proves nothing about them (beyond10x/ess#161).
    // So where the plain row leaves some write unchanged, the search is asked again — under the
    // plain witness and then further ones — for a row that leaves fewer unchanged, and keeps the
    // fewest: the criterion [`arranged`](super::arranged) applies on the plain path. A write no row
    // can change (a literal every arrangement already holds) does not stop the others being changed.
    let mut unchanged = super::unchanged_writes(ir, outcome, &input, &arrangement.settled);
    let (mut arrangement, input) = {
        let mut best = (arrangement, input);
        for distinction in std::iter::once(Distinction::PLAIN)
            .chain((1..=super::FRESH_WITNESSES).map(Distinction::further))
        {
            if unchanged == 0 {
                break;
            }
            let bound = unchanged;
            let Ok((row, input)) =
                search(ir, entity, actors, &hints, distinction, &label, |node| {
                    Ok(
                        reach_linked(ir, command, outcome, entity, node, accept)?.filter(|input| {
                            super::unchanged_writes(ir, outcome, input, &node.settled) < bound
                        }),
                    )
                })
            else {
                continue;
            };
            unchanged = super::unchanged_writes(ir, outcome, &input, &row.settled);
            best = (row, input);
        }
        best
    };
    // The pair a `sets-retarget` mutant joins is sent apart on the chosen row too, where the row
    // still selects the branch and no more of its writes are left unchanged (beyond10x/ess#202).
    let keeps = |next: &BTreeMap<String, Node>| {
        selects(ir, command, entity, &arrangement, next)
            .ok()
            .flatten()
            .is_some_and(|branch| branch.name == outcome.name)
            && super::unchanged_writes(ir, outcome, next, &arrangement.settled) <= unchanged
            && accept(&arrangement, next)
    };
    let (input, _) = super::sources_apart(ir, command, outcome, input, &keeps);
    let bound = bind_links(
        ir,
        command,
        outcome,
        entity,
        actors,
        &mut arrangement,
        &input,
        &mut false,
    )?;
    let (steps, view) = observe_fields(ir, entity, &fields, &arrangement)?;
    arrangement.steps.extend(steps);
    arrangement.source.insert(view.into());
    let after = outcome
        .subject
        .as_ref()
        .and_then(|own| own.effect.transition())
        .map_or_else(
            || arrangement.state.clone(),
            |transition| transition.to.clone(),
        );
    Ok((
        Setup {
            steps: arrangement.steps,
            instance: Some(arrangement.instance),
            bound,
            source: arrangement.source,
            after: Some(after),
            before: Some(arrangement.state),
            settled: arrangement.settled,
        },
        input,
    ))
}

/// The row after one route step through a branch of a command reading stored fields, where the
/// row the route built selects that branch.
///
/// Used by the ordinary lifecycle arrangement, which takes the plain witness at every step: this
/// checks the claim that witness makes rather than assuming it, and adds nothing to the steps a
/// route that was already right produced.
pub(super) fn step(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<Arrangement> {
    let inputs = inputs_for(ir, driver.command, entity, arrangement).ok()?;
    inputs.into_iter().find_map(|input| {
        selects(ir, driver.command, entity, arrangement, &input)
            .ok()
            .flatten()
            .filter(|branch| branch.name == driver.outcome.name)
            .map(|_| advanced(ir, driver, arrangement, actors, &input, Vec::new(), false))
    })
}

/// A row resting in `target`, reached through branches every row on the way selects.
///
/// Arranged under `distinction`, so a further instance searched for here keeps the name its caller
/// chose it apart by (beyond10x/ess#199).
pub(super) fn reach_state(
    ir: &EssIr,
    entity: &EntityHandle,
    target: &super::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
) -> Result<Arrangement, RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    let mut hints = Vec::new();
    for driver in drivers.iter().filter(|driver| uses(driver.command)) {
        for hint in self::hints(driver.command) {
            if !hints.contains(&hint) {
                hints.push(hint);
            }
        }
    }
    search(ir, entity, actors, &hints, distinction, "state", |node| {
        Ok((&node.state == target).then_some(()))
    })
    .map(|(arrangement, ())| arrangement)
}

/// Which assertion styles a stored-field observation may be made through.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reading {
    /// Only an `expect` view: the observation must see the row at the moment it is made.
    Immediate,
    /// An `expect` view where one qualifies, and an `eventually` view where none does.
    Settling,
}

/// The unfiltered, parameterless view a stored-field arrangement is observed through: it projects
/// the identity, `state` and every field named, each at the entity's declared type.
///
/// An immediate view is preferred. Where `reading` allows it and no immediate view qualifies, an
/// `eventual` one does (beyond10x/ess#172). [`require`] puts that requirement in an `eventually`
/// block, which waits until the projection shows the identity, the state and every fact named at
/// the values the arrangement's last step left, instead of racing it. What that proves is that the
/// implementation applied those values; no step of the scenario writes the row between that
/// observation and the command, so they are what the command reads. It proves nothing about a
/// row that did not move: a projection that is behind shows the old row too, which is why a
/// refusal's unchanged row is read with [`Reading::Immediate`] only ([`observe_unchanged`]).
fn observer<'ir>(
    ir: &'ir EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    reading: Reading,
) -> Option<&'ir ess_compiler::ir::ResolvedView> {
    let declared = ir.entity(entity);
    let projects = |view: &&ess_compiler::ir::ResolvedView| {
        !view.is_aggregate()
            && view.source == *entity
            && super::paging::read_whole(view)
            && view.filter.is_none()
            && view
                .field(&declared.identity.name)
                .is_some_and(|f| f.type_ref == declared.identity.type_ref)
            && view
                .field(EntitySpec::STATE)
                .is_some_and(|f| f.type_ref == declared.state_field().type_ref)
            && fields.iter().all(|name| {
                declared
                    .fields
                    .iter()
                    .find(|field| &field.name == name)
                    .is_some_and(|field| {
                        view.field(name)
                            .is_some_and(|shown| shown.type_ref == field.type_ref)
                    })
            })
    };
    let styled = |style: AssertionStyle| {
        ir.views()
            .values()
            .filter(projects)
            .find(|view| view.assertion_style == style)
    };
    styled(AssertionStyle::Expect).or_else(|| {
        (reading == Reading::Settling)
            .then(|| styled(AssertionStyle::Eventually))
            .flatten()
    })
}

/// Observe one stored fact on the arranged row. Kept for the replay family, which observes the
/// facts one at a time, and keeps its immediate witness: the eventual fallback of
/// [`observe_fields`] belongs to subject-fact selection (beyond10x/ess#172), and a replay's
/// observation has its own profile.
pub(super) fn observe(
    ir: &EssIr,
    entity: &EntityHandle,
    field: &str,
    arrangement: &Arrangement,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let fields = BTreeSet::from([field.to_owned()]);
    let row = expected_row(ir, entity, &fields, arrangement)?;
    let view = observer(ir, entity, &fields, Reading::Immediate).ok_or_else(|| {
        missing(
            entity,
            field,
            "subject fact selection requires an immediate unfiltered identity/state/fact view",
        )
    })?;
    Ok(required(view, row))
}

/// Require the arranged row, with every guarded field at the value the arrangement determined,
/// before the command runs: the facts the scenario is about are observed, not assumed.
///
/// Also the row a moving or updating branch leaves: it arrives at a state or values it did not
/// hold, so an `eventually` block waiting for them does not pass on a projection that has not
/// caught up.
pub(super) fn observe_fields(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    arrangement: &Arrangement,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let label = fields.iter().cloned().collect::<Vec<_>>().join(",");
    let row = expected_row(ir, entity, fields, arrangement)?;
    let view = observer(ir, entity, fields, Reading::Settling).ok_or_else(|| {
        missing(
            entity,
            &label,
            "subject fact selection requires an immediate unfiltered identity/state/fact view, \
             or an `eventual` one it waits for, and no view projects them",
        )
    })?;
    Ok(required(view, row))
}

/// Require the row a refusal left where it was, through an immediate view only — or `None`.
///
/// Nothing moved, so there is nothing for an `eventually` block to wait for: a projection that
/// has not caught up with a wrong change still shows the arranged row, and the check would pass on
/// exactly the implementation it exists to catch. Where no immediate view projects the identity,
/// the state and the fields named, the unchanged-row check is omitted rather than asserted
/// through an `eventual` one; the scenario still asserts the refusal's outcome, its error and
/// that no event was published. The same reason keeps [`absent`] on immediate views.
fn observe_unchanged(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    arrangement: &Arrangement,
) -> Result<Option<(Vec<ScenarioStep>, ViewRef)>, RefusalCause> {
    let row = expected_row(ir, entity, fields, arrangement)?;
    Ok(observer(ir, entity, fields, Reading::Immediate).map(|view| required(view, row)))
}

/// The row an observation of the arrangement requires: its identity, its state and every field
/// named at the value the arrangement determined.
fn expected_row(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    arrangement: &Arrangement,
) -> Result<BTreeMap<String, ScenarioValue>, RefusalCause> {
    let declared = ir.entity(entity);
    let mut row = BTreeMap::from([
        (
            declared.identity.name.clone(),
            ScenarioValue::instance(arrangement.instance.clone()),
        ),
        (
            EntitySpec::STATE.to_owned(),
            ScenarioValue::literal(Node::Text(arrangement.state.to_string())),
        ),
    ]);
    for field in fields {
        let Some(value) = arrangement.settled.get(field) else {
            return Err(missing(
                entity,
                field,
                "subject fact has no determined arrangement value",
            ));
        };
        row.insert(field.clone(), value.value.clone());
    }
    Ok(row)
}

/// The steps requiring `row` of `view`, in the block its consistency decides.
fn required(
    view: &ess_compiler::ir::ResolvedView,
    row: BTreeMap<String, ScenarioValue>,
) -> (Vec<ScenarioStep>, ViewRef) {
    let name = ViewRef::new(view.name.clone());
    let mut steps = Vec::new();
    require(
        view,
        &name,
        BTreeMap::new(),
        ViewExpectation::Contains { fields: row },
        &mut steps,
    );
    (steps, name)
}

/// The absent-subject witness: the command sent once for an identity no row carries.
///
/// The specification declares no outcome for it, so nothing is asserted about the error. What it
/// does say is that a subject-fact branch reads a row that exists: so no declared event is
/// published and no row appears under that identity.
pub(super) fn absent(
    ir: &EssIr,
    command: &ResolvedCommand,
    subject: &ResolvedSubject,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let ResolvedInstance::Supplied { field } = &subject.instance else {
        return Err(RefusalCause::StrategyWithoutGuard {
            strategy: ess_domain::command::TestStrategy::ObserveSubjectFact,
        });
    };
    let input = candidates(ir, command, &[], Distinction::PLAIN)
        .map_err(RefusalCause::NoWitness)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            missing(
                &subject.entity,
                &field.name,
                "the command has no witness input",
            )
        })?;
    let identity = input.get(&field.name).cloned().ok_or_else(|| {
        missing(
            &subject.entity,
            &field.name,
            "the witness names no identity",
        )
    })?;
    let view =
        observer(ir, &subject.entity, &BTreeSet::new(), Reading::Immediate).ok_or_else(|| {
            missing(
                &subject.entity,
                &field.name,
                "subject fact selection requires an immediate unfiltered identity/state/fact view",
            )
        })?;
    let command_ref = CommandRef::new(command.name.clone());
    let mut steps = vec![ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supply(command, &input, None, None, &BTreeMap::new()),
    }];
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    let name = ViewRef::new(view.name.clone());
    require(
        view,
        &name,
        BTreeMap::new(),
        ViewExpectation::Excludes {
            fields: BTreeMap::from([(
                ir.entity(&subject.entity).identity.name.clone(),
                ScenarioValue::literal(identity),
            )]),
        },
        &mut steps,
    );
    let mut source: BTreeSet<EssSemanticRef> = forbidden.into_iter().map(Into::into).collect();
    source.insert(command_ref.into());
    source.insert(name.into());
    Ok((steps, source))
}

/// What a routed branch's scenario observes around the command, beyond the arrangement.
///
/// A branch naming no subject of its own is opened by the [`absent`] witness, and after it the
/// arranged row is required again exactly as it was observed: a refusal changes nothing. A branch
/// that moves or updates the row it read is observed afterwards in the state it arrives at, with
/// the stored fields as the branch left them. Returns the steps that belong after the branch's own
/// assertions; the absent-subject steps are spliced into the arrangement. The [`boundaries`] a
/// default is further witnessed against come last.
pub(super) fn around(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &mut Setup,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> Result<Vec<ScenarioStep>, RefusalCause> {
    // Whether the scenario already arranged the second owner a link comparison names (#193).
    let mut present = reading(command, outcome)
        .and_then(|subject| other_owner(ir, &subject.entity))
        .is_some_and(|(_, other)| setup.bound.values().any(|named| named == &other));
    // Every instance name the arrangement already binds, which no further row may bind again.
    let mut taken = super::bound_instances(&setup.steps);
    taken.extend(setup.instance.iter().cloned());
    let (further, source) = boundaries(
        ir,
        command,
        outcome,
        actors,
        (&setup.settled, setup.before.as_ref()),
        (&mut present, &mut taken),
    )?;
    let (overlapping, overlap_source) =
        overlaps(ir, command, outcome, actors, (&mut present, &mut taken))?;
    let mut steps = around_row(ir, command, outcome, actors, setup, supplied)?;
    steps.extend(further);
    steps.extend(overlapping);
    setup.source.extend(source);
    setup.source.extend(overlap_source);
    Ok(steps)
}

/// Whether `outcome` moves or updates the row it names.
fn moves_row(outcome: &ResolvedOutcome) -> bool {
    outcome.subject.as_ref().is_some_and(|own| {
        matches!(
            own.effect,
            ResolvedEffect::Moves { .. } | ResolvedEffect::Updates
        )
    })
}

/// Whether the row after the command differs from the arranged one in its state or in a field
/// named: only then does an `eventually` block after the command wait for something a projection
/// that is behind does not show. A move back to the state it left, or an update writing the values
/// the row already held, is read as a refusal's row is, through [`observe_unchanged`]; the generic
/// view assertion (`view_expectations`) drops its `eventually` block for the same row.
fn leaves_changed(
    before_state: Option<&super::StateName>,
    before: &BTreeMap<String, super::Determined>,
    after: &Arrangement,
    fields: &BTreeSet<String>,
) -> bool {
    before_state.is_some_and(|state| state != &after.state)
        || fields.iter().any(|field| {
            before.get(field).map(|held| &held.value)
                != after.settled.get(field).map(|held| &held.value)
        })
}

fn around_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &mut Setup,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> Result<Vec<ScenarioStep>, RefusalCause> {
    let Some(subject) = reading(command, outcome) else {
        return Ok(Vec::new());
    };
    // Only for the ess/9 predicate form: an ess/6 `{field, equals}` command keeps the suite it
    // generated before this construct, and its moving branch is observed as it always was.
    let changes = uses_predicate(command) && moves_row(outcome);
    if outcome.subject.is_none() {
        let (steps, source) = absent(ir, command, subject, actors)?;
        setup.steps.splice(0..0, steps);
        setup.source.extend(source);
    } else if !changes {
        return Ok(Vec::new());
    }
    let (Some(instance), Some(state)) = (&setup.instance, &setup.after) else {
        return Ok(Vec::new());
    };
    let mut left = setup.settled.clone();
    if changes {
        absorb(
            &mut left,
            outcome,
            super::settled(ir, outcome, supplied, &setup.settled),
        );
    }
    let fields = guarded_fields(ir, command, &subject.entity)
        .into_iter()
        .filter(|field| left.contains_key(field))
        .collect();
    let arrangement = Arrangement {
        instance: instance.clone(),
        state: state.clone(),
        steps: Vec::new(),
        source: BTreeSet::new(),
        settled: left,
    };
    // A refusal left the row where it was, and only an immediate read can say so.
    let observation = if changes
        && leaves_changed(setup.before.as_ref(), &setup.settled, &arrangement, &fields)
    {
        Some(observe_fields(ir, &subject.entity, &fields, &arrangement)?)
    } else {
        observe_unchanged(ir, &subject.entity, &fields, &arrangement)?
    };
    let Some((observed, view)) = observation else {
        return Ok(Vec::new());
    };
    setup.source.insert(view.into());
    Ok(observed)
}

/// For every conjunctive stored-field guard, one goal per conjunct: that conjunct refuted and every
/// other one satisfied — skipping a goal the ordinary witness row already meets.
fn conjunct_goals(
    ir: &EssIr,
    entity: &EntityHandle,
    hints: &[Predicate],
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    let mut goals = Vec::new();
    for hint in hints {
        if let Predicate::All(conjuncts) = hint {
            goals.extend(isolating(
                ir,
                entity,
                conjuncts,
                false,
                witnessed,
                witnessed_state,
            ));
        }
    }
    goals
}

/// The further rows one branch is witnessed on: the default's, one per conjunct of a guarded
/// sibling; a guarded branch's own, one per disjunct of its predicate (beyond10x/ess#155).
fn goals_for(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    hints: &[Predicate],
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    if state_default(outcome) {
        if outcome.subject.is_none() {
            return Vec::new();
        }
        return conjunct_goals(ir, entity, hints, witnessed, witnessed_state);
    }
    stored(&outcome.condition)
        .map(|own| disjunct_goals(ir, entity, &own, witnessed, witnessed_state))
        .unwrap_or_default()
}

/// One row a further witness is arranged on: every predicate of the first list false on it, every
/// one of the second true.
type Goal = (Vec<Predicate>, Vec<Predicate>);

/// For a disjunctive stored-field guard of the branch itself — its predicate an `any`, or an `any`
/// among its top-level conjuncts — one goal per disjunct: that disjunct satisfied and every other one
/// refuted, with the guard's remaining conjuncts satisfied (beyond10x/ess#155). A row satisfying
/// every disjunct at once is also a row of the `all` a connective mutant writes.
fn disjunct_goals(
    ir: &EssIr,
    entity: &EntityHandle,
    own: &Predicate,
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    let conjuncts: Vec<&Predicate> = match own {
        Predicate::All(children) => children.iter().collect(),
        other => vec![other],
    };
    let mut goals = Vec::new();
    for (at, conjunct) in conjuncts.iter().enumerate() {
        let Predicate::Any(disjuncts) = conjunct else {
            continue;
        };
        let rest: Vec<Predicate> = conjuncts
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != at)
            .map(|(_, other)| (*other).clone())
            .collect();
        for (refuted, mut held) in
            isolating(ir, entity, disjuncts, true, witnessed, witnessed_state)
        {
            held.extend(rest.iter().cloned());
            goals.push((refuted, held));
        }
    }
    goals
}

/// One goal per child of a connective with two or more children: that child `alone` (true for a
/// disjunct, false for a conjunct) and every other the opposite — skipping a goal the ordinary
/// witness row already meets, because a second copy proves nothing more.
fn isolating(
    ir: &EssIr,
    entity: &EntityHandle,
    children: &[Predicate],
    alone: bool,
    witnessed: &BTreeMap<String, super::Determined>,
    witnessed_state: Option<&super::StateName>,
) -> Vec<Goal> {
    if children.len() < 2 {
        return Vec::new();
    }
    let wanted = |value: bool| if value { Truth::True } else { Truth::False };
    let mut goals = Vec::new();
    for index in 0..children.len() {
        if children.iter().enumerate().all(|(other, child)| {
            row_truth(ir, entity, witnessed, witnessed_state, child)
                == wanted((other == index) == alone)
        }) {
            continue;
        }
        let (mut falses, mut trues) = (Vec::new(), Vec::new());
        for (other, child) in children.iter().enumerate() {
            if (other == index) == alone {
                trues.push(child.clone());
            } else {
                falses.push(child.clone());
            }
        }
        goals.push((falses, trues));
    }
    goals
}

/// The most further rows the default of one command is witnessed against.
const MAX_BOUNDARIES: usize = 8;

/// Further rows the default branch is witnessed against, one per conjunct of a guarded sibling.
///
/// One row refuting a conjunctive guard shows only that *some* conjunct is read: `Express` at
/// `20` refutes `service == Express and weight_kg > 20` through the weight, and an implementation
/// refusing every parcel over 20 kg, whatever its service, passes it. So for each sibling whose
/// stored-field predicate is a conjunction of two or more conjuncts, the default is witnessed once
/// more per conjunct on a row that refutes **exactly that one** and satisfies the rest — `Standard`
/// at `21`, `Express` at `20` — each on its own instance. Bounded by [`MAX_BOUNDARIES`]; a conjunct
/// no bounded arrangement refutes alone adds no row, because the ordinary witness still stands.
///
/// The ess/9 predicate form only: an ess/6 `{field, equals}` guard is one leaf, and its suites keep
/// their bytes.
///
/// A goal holding a link comparison ([`compares_link`]) is decided with the input bound, over
/// [`linked_inputs`], and sent the owners that input names ([`bind_links`]) (beyond10x/ess#193).
/// The branch is refused with `ESS-SYNTH-003` naming the goal where the goal lies past
/// [`MAX_BOUNDARIES`], or where the bounded search did not reach it and an input naming an arranged
/// owner left it undecided: the search could not say whether a row meets it. A goal every
/// candidate decided and no bounded row meets adds no row, as for every other guard.
pub(super) fn boundaries(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (witnessed, witnessed_state): (
        &BTreeMap<String, super::Determined>,
        Option<&super::StateName>,
    ),
    (present, taken): (&mut bool, &mut BTreeSet<super::InstanceName>),
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    // A branch naming no subject of its own — a refusal — reads the row its siblings name.
    let Some(read) = reading(command, outcome) else {
        return Ok((steps, source));
    };
    if !uses_predicate(command) {
        return Ok((steps, source));
    }
    let entity = &read.entity;
    let hints = hints(command);
    let fields = read_fields(ir, entity, &hints);
    let goals = goals_for(ir, outcome, entity, &hints, witnessed, witnessed_state);
    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    // A command comparing a link with an input decides every guard with the input bound: `selects`
    // reads every branch, so a goal of its own reads the link through a sibling too.
    let decided_with_input = !links(ir, command, entity).is_empty();
    let mut rows = 0;
    for (refuted, held) in goals {
        let linked = compares_link(
            ir,
            command,
            entity,
            &refuted.iter().chain(&held).cloned().collect::<Vec<_>>(),
        );
        if rows >= MAX_BOUNDARIES {
            // Past the bound a goal adds no row; one isolating a link comparison was not searched,
            // so whether a row meets it is unknown, and it is refused rather than left unwitnessed.
            if linked {
                return Err(unreached(entity, command, outcome, (&refuted, &held), None));
            }
            continue;
        }
        let mut undecided = false;
        let found = search_unbound(
            ir,
            entity,
            actors,
            &hints,
            rows + 1,
            "boundary",
            taken,
            |node| {
                if !decided_with_input {
                    let truth = |predicate: &Predicate| {
                        row_truth(ir, entity, &node.settled, Some(&node.state), predicate)
                    };
                    if refuted.iter().any(|child| truth(child) != Truth::False)
                        || held.iter().any(|child| truth(child) != Truth::True)
                    {
                        return Ok(None);
                    }
                    return reach_at(ir, command, outcome, entity, node);
                }
                goal_input(
                    ir,
                    (command, outcome),
                    (entity, node),
                    (linked_inputs(ir, command, entity, node)?, &|_| Ok(true)),
                    (&refuted, &held),
                    &mut undecided,
                )
            },
        )?;
        let (arrangement, input) = match found {
            Ok(found) => found,
            // A goal no candidate left undecided is one no row meets, which adds no row as for
            // every other guard; one some candidate left undecided is refused.
            Err(cause) if linked && undecided => {
                return Err(unreached(
                    entity,
                    command,
                    outcome,
                    (&refuted, &held),
                    Some(&cause),
                ));
            }
            Err(_) => continue,
        };
        rows += 1;
        send_for_row(
            ir,
            command,
            outcome,
            actors,
            (read, &fields),
            arrangement,
            (&input, (&mut *present, &mut *taken)),
            (&mut steps, &mut source),
        )?;
    }
    if rows > 0 {
        source.insert(command_ref.into());
        source.insert(outcome_ref.into());
    }
    Ok((steps, source))
}

/// Which candidate inputs a further row's search may send at all, before its row is decided.
type Admits<'a> = &'a dyn Fn(&BTreeMap<String, Node>) -> Result<bool, RefusalCause>;

/// The first of `inputs` that `admits` and that sends `outcome` on `node`'s row with every predicate
/// of `refuted` false and every one of `held` true, each decided with that input bound
/// (beyond10x/ess#193). `undecided` is set where an input naming an arranged owner
/// ([`naming_owner`]) left a goal predicate `Unknown` and none decided wrong: the row may be the
/// goal's, and the search could not say.
fn goal_input(
    ir: &EssIr,
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    (entity, node): (&EntityHandle, &Arrangement),
    (inputs, admits): (Vec<BTreeMap<String, Node>>, Admits<'_>),
    (refuted, held): (&[Predicate], &[Predicate]),
    undecided: &mut bool,
) -> Result<Option<BTreeMap<String, Node>>, RefusalCause> {
    let naming = naming_owner(ir, command, entity, node, &inputs);
    for (input, names) in inputs.into_iter().zip(naming) {
        if !admits(&input)? {
            continue;
        }
        let truth = |predicate: &Predicate| {
            row_truth_with(
                ir,
                entity,
                &node.settled,
                Some(&node.state),
                predicate,
                Some((command, &input)),
            )
        };
        let falses: Vec<Truth> = refuted.iter().map(truth).collect();
        let trues: Vec<Truth> = held.iter().map(truth).collect();
        if falses.contains(&Truth::True) || trues.contains(&Truth::False) {
            continue;
        }
        if falses.contains(&Truth::Unknown) || trues.contains(&Truth::Unknown) {
            *undecided |= names;
            continue;
        }
        if selects(ir, command, entity, node, &input)?
            .is_some_and(|branch| branch.name == outcome.name)
        {
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// `ESS-SYNTH-003` for a further row a guard comparing a link with an input needs and no bounded
/// arrangement gives (beyond10x/ess#193): the goal, as the predicates that must be false and true
/// on it, and the branch it is for. `cause` is the search's own refusal; `None` where the goal lies
/// past [`MAX_BOUNDARIES`] and was not searched.
fn unreached(
    entity: &EntityHandle,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    (refuted, held): (&[Predicate], &[Predicate]),
    cause: Option<&RefusalCause>,
) -> RefusalCause {
    let (why, tried) = match cause {
        None => ("past the most further rows one branch is witnessed on", 0),
        Some(RefusalCause::GuardUnsatisfiable { tried, .. }) => {
            ("over the rows bounded arrangements left", *tried)
        }
        Some(_) => ("over the rows bounded arrangements left", 0),
    };
    let render = |predicates: &[Predicate]| {
        predicates
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{entity}` row for `{}/{}` with [{}] false and [{}] true, a guard comparing the \
             owner link with an input, {why}",
            command.name,
            outcome.name,
            render(refuted),
            render(held),
        ),
        tried,
    }
}

/// The branch sent once more for a further arranged row, with what it requires, and the row
/// observed again afterwards: as the branch left it, or unchanged. The owners a link comparison's
/// input names are arranged and sent first ([`bind_links`], with `present`), and every instance
/// name the row binds is added to `taken`, so no later further row of the scenario binds it again.
#[allow(clippy::too_many_arguments)]
fn send_for_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (read, fields): (&ResolvedSubject, &BTreeSet<String>),
    mut arrangement: Arrangement,
    (input, (present, taken)): (
        &BTreeMap<String, Node>,
        (&mut bool, &mut BTreeSet<super::InstanceName>),
    ),
    (steps, source): (&mut Vec<ScenarioStep>, &mut BTreeSet<EssSemanticRef>),
) -> Result<(), RefusalCause> {
    let entity = &read.entity;
    let bound = &bind_links(
        ir,
        command,
        outcome,
        entity,
        actors,
        &mut arrangement,
        input,
        present,
    )?;
    taken.extend(super::bound_instances(&arrangement.steps));
    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    let (observed, view) = observe_fields(ir, entity, fields, &arrangement)?;
    arrangement.steps.extend(observed);
    steps.append(&mut arrangement.steps);
    source.append(&mut arrangement.source);
    source.insert(view.into());
    let supplied = supply(
        command,
        input,
        Some(read),
        Some(&arrangement.instance),
        bound,
    );
    steps.push(ScenarioStep::ExecuteCommand {
        caller: std::collections::BTreeMap::new(),
        command: command_ref,
        actor: actors.get(&command.name).cloned(),
        input: supplied.clone(),
    });
    steps.push(ScenarioStep::ExpectOutcome {
        outcome: outcome_ref,
    });
    match &outcome.error {
        Some(error) => steps.push(ScenarioStep::ExpectError {
            error: super::ErrorRef::from(error),
            fields: BTreeMap::new(),
        }),
        None => steps.push(ScenarioStep::ExpectNoError),
    }
    let held = (arrangement.state.clone(), arrangement.settled.clone());
    let mut left = arrangement.settled.clone();
    absorb(
        &mut left,
        outcome,
        super::settled(ir, outcome, &supplied, &arrangement.settled),
    );
    if let Some(transition) = outcome
        .subject
        .as_ref()
        .and_then(|own| own.effect.transition())
    {
        arrangement.state = transition.to.clone();
    }
    arrangement.settled = left;
    let kept = fields
        .iter()
        .filter(|field| arrangement.settled.contains_key(*field))
        .cloned()
        .collect();
    if leaves_changed(Some(&held.0), &held.1, &arrangement, &kept) {
        let (after, _) = observe_fields(ir, entity, &kept, &arrangement)?;
        steps.extend(after);
    } else if let Some((after, _)) = observe_unchanged(ir, entity, &kept, &arrangement)? {
        steps.extend(after);
    }
    Ok(())
}

/// Further rows an input-guarded refusal sent for an arranged row is witnessed on where it
/// overlaps an accepting branch (beyond10x/ess#178): one per accepting sibling with an input half,
/// on a row that sibling's stored guard admits, with an input both input guards admit and every
/// other input-guarded refusal refutes. The refusal is required there, and the row is observed
/// unchanged, so a target that reads the accepting branch's guards first fails.
///
/// The row-free half of the rule — a refusal over the identity, and every command reading no
/// stored field — is `overlap_inputs` in the parent module.
fn overlaps(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (present, taken): (&mut bool, &mut BTreeSet<super::InstanceName>),
) -> Result<(Vec<ScenarioStep>, BTreeSet<EssSemanticRef>), RefusalCause> {
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    let Some(own) = super::is_input_guarded_refusal(outcome)
        .then(|| when(outcome))
        .flatten()
    else {
        return Ok((steps, source));
    };
    let Some(read) = reading(command, outcome) else {
        return Ok((steps, source));
    };
    let entity = &read.entity;
    let hints = hints(command);
    let fields = read_fields(ir, entity, &hints);
    let refusals: Vec<&Predicate> = super::sibling_refusals(command, outcome)
        .filter_map(when)
        .collect();
    // As in `boundaries`: a command comparing a link with an input is decided with the input bound
    // (beyond10x/ess#193). A row of an accepting guard holding that comparison is refused past the
    // bound, or where the bounded search missed it with an input naming an arranged owner leaving
    // it undecided; one every candidate decided and no bounded row meets adds no row.
    let decided_with_input = !links(ir, command, entity).is_empty();
    let mut rows = 0;
    for accepting in &command.outcomes {
        let Some(guard) = super::accepting_input_half(accepting) else {
            continue;
        };
        let row_guard = stored(&accepting.condition);
        let linked = row_guard.as_ref().is_some_and(|predicate| {
            compares_link(ir, command, entity, std::slice::from_ref(predicate))
        });
        if rows >= MAX_BOUNDARIES {
            if linked {
                return Err(unreached(
                    entity,
                    command,
                    outcome,
                    (&[], row_guard.as_slice()),
                    None,
                ));
            }
            continue;
        }
        // Numbered from past every row `boundaries` can arrange, and past every name the scenario
        // already binds, so no instance name is bound twice.
        let mut undecided = false;
        let admits = |input: &BTreeMap<String, Node>| {
            let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
            Ok(decides(&facts, &[own, guard], true)? && decides(&facts, &refusals, false)?)
        };
        let found = search_unbound(
            ir,
            entity,
            actors,
            &hints,
            MAX_BOUNDARIES + rows + 1,
            "overlap",
            taken,
            |node| {
                let inputs = if decided_with_input {
                    linked_inputs(ir, command, entity, node)?
                } else {
                    inputs_for(ir, command, entity, node)?
                };
                goal_input(
                    ir,
                    (command, outcome),
                    (entity, node),
                    (inputs, &admits),
                    (&[], row_guard.as_slice()),
                    &mut undecided,
                )
            },
        )?;
        let (arrangement, input) = match found {
            Ok(found) => found,
            Err(cause) if linked && undecided => {
                return Err(unreached(
                    entity,
                    command,
                    outcome,
                    (&[], row_guard.as_slice()),
                    Some(&cause),
                ));
            }
            Err(_) => continue,
        };
        rows += 1;
        send_for_row(
            ir,
            command,
            outcome,
            actors,
            (read, &fields),
            arrangement,
            (&input, (&mut *present, &mut *taken)),
            (&mut steps, &mut source),
        )?;
    }
    if rows > 0 {
        let command_ref = CommandRef::new(command.name.clone());
        source.insert(OutcomeRef::new(command_ref.clone(), outcome.name.clone()).into());
        source.insert(command_ref.into());
    }
    Ok((steps, source))
}

pub(super) struct Preservation {
    pub before: Vec<ScenarioStep>,
    pub after: Vec<ScenarioStep>,
    pub source: BTreeSet<EssSemanticRef>,
    /// The subject fields no view let the observation cover, in name order; empty for a complete
    /// one (beyond10x/ess#132).
    pub unobserved: Vec<String>,
}

/// Every declared field must be observed. Unknown generated values are captured from
/// the implementation before the command, never filled from the expected outcome.
pub(super) fn preservation(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    let subject = outcome
        .subject
        .as_ref()
        .expect("preservation has a subject");
    preserve_subject(ir, subject, setup)
}

pub(super) fn preserve_subject(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    preserve(ir, subject, setup, Observation::Legacy)
}

pub(super) fn preserve_complete_subject(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    preserve(ir, subject, setup, Observation::Complete)
}

/// [`preserve_complete_subject`] for a wrong-state refusal, which falls back to what the declared
/// views publish where no immediate views cover every subject field (beyond10x/ess#132), and
/// names the rest in [`Preservation::unobserved`].
pub(super) fn preserve_refused_subject(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    preserve(ir, subject, setup, Observation::CompleteOrPublished)
}

/// How much of the subject a preservation has to observe.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Observation {
    /// Snapshot and comparison over immediate views covering every field (before `ess/7`).
    Legacy,
    /// The complete typed snapshot over immediate views covering every field.
    Complete,
    /// [`Self::Complete`], or else whatever the declared views publish.
    CompleteOrPublished,
}

fn preserve(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
    observation: Observation,
) -> Result<Preservation, RefusalCause> {
    let complete = observation != Observation::Legacy;
    let entity = ir.entity(&subject.entity);
    let instance = setup
        .instance
        .as_ref()
        .expect("preservation arranged an existing subject");
    let mut required: BTreeSet<String> = entity
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect();
    required.insert(entity.identity.name.clone());
    required.insert(EntitySpec::STATE.into());
    let mut before = Vec::new();
    let mut after = Vec::new();
    let mut source = BTreeSet::new();
    for view in ir.views().values().filter(|view| {
        !view.is_aggregate()
            && view.source == subject.entity
            && super::paging::read_whole(view)
            && setup.after.as_ref().is_some_and(|state| {
                shows(ir, view, state, &setup.settled, &BTreeMap::new()) == Ok(true)
            })
            && view.assertion_style == AssertionStyle::Expect
            && view
                .field(&entity.identity.name)
                .is_some_and(|field| field.type_ref == entity.identity.type_ref)
    }) {
        let name = ViewRef::new(view.name.clone());
        for field in &view.fields {
            let exact = if field.name == entity.identity.name {
                field.type_ref == entity.identity.type_ref
            } else if field.name == EntitySpec::STATE {
                field.type_ref == entity.state_field().type_ref
            } else {
                entity.fields.iter().any(|declared| {
                    declared.name == field.name && declared.type_ref == field.type_ref
                })
            };
            if exact {
                required.remove(&field.name);
            }
        }
        before.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        let selected = [(
            entity.identity.name.clone(),
            ScenarioValue::instance(instance.clone()),
        )]
        .into_iter()
        .collect();
        before.push(if complete {
            let shape = crate::subject::SubjectShape::of(ir, view, &entity.identity.name).map_err(
                |reason| {
                    RefusalCause::NoWitness(WitnessGap {
                        path: format!("{name}: {reason}"),
                        type_ref: "complete subject observation".into(),
                        reason: "complete subject requires a finite exact typed observer",
                    })
                },
            )?;
            ScenarioStep::SnapshotCompleteSubject {
                view: name.clone(),
                subject: selected,
                shape,
            }
        } else {
            ScenarioStep::SnapshotSubject {
                view: name.clone(),
                subject: selected,
            }
        });
        after.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        after.push(if complete {
            ScenarioStep::ExpectCompleteSubjectUnchanged { view: name.clone() }
        } else {
            ScenarioStep::ExpectSubjectUnchanged { view: name.clone() }
        });
        source.insert(name.into());
    }
    let observed = Preservation {
        before,
        after,
        source,
        unobserved: Vec::new(),
    };
    covered(ir, subject, setup, observation, observed, required)
}

/// The observation, where the immediate views covered every subject field. Where they fall short,
/// a refusal's observation keeps what they and the `eventual` views observe, with the rest named as
/// unobserved (beyond10x/ess#132); anything else — or nothing observing the row at all — refuses.
fn covered(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
    observation: Observation,
    mut observed: Preservation,
    mut required: BTreeSet<String>,
) -> Result<Preservation, RefusalCause> {
    if required.is_empty() {
        return Ok(observed);
    }
    if observation != Observation::CompleteOrPublished {
        return Err(missing(
            &subject.entity,
            &required.into_iter().collect::<Vec<_>>().join(","),
            "preservation requires immediate identity views covering every subject field",
        ));
    }
    let instance = setup
        .instance
        .as_ref()
        .expect("preservation arranged an existing subject");
    let partial = eventual_observation(ir, subject, setup, instance, &mut required);
    if observed.before.is_empty() && partial.after.is_empty() {
        return Err(missing(
            &subject.entity,
            &required.into_iter().collect::<Vec<_>>().join(","),
            "a refused subject is observed through a view of the entity projecting its \
             identity, and none shows this row",
        ));
    }
    observed.after.extend(partial.after);
    observed.source.extend(partial.source);
    observed.unobserved = required.into_iter().collect();
    Ok(observed)
}

/// A complete refusal's observation where no set of immediate views covers every subject field
/// (beyond10x/ess#132): the fields the declared views do publish, each required after the refusal
/// to hold what the arrangement left there.
///
/// An `eventual` identity/state view is the honest declaration for an entity whose other fields the
/// implementation cannot read back, and requiring a view that claims a consistent store of every
/// field would make the specification false. So each `eventual` view that shows the arranged row is
/// required, in its own block, to hold that row with its identity, its state and every field the
/// arrangement settled and the view projects at the entity's type — and nothing more, so the
/// scenario's own steps record exactly which part of the subject was observed. The immediate views
/// that did qualify keep their complete snapshot and comparison beside it. Empty where no eventual
/// view shows the row; with no immediate view either there is nothing to observe, and the caller's
/// refusal stands.
///
/// What an `eventual` read proves is weaker, and [`Note::PartialObservation`](super::Note) says so:
/// a refusal leaves the row where it was, so there is no later state to poll for, and a projection
/// that has not yet caught up with a wrong change still shows the arranged row on its first read.
/// The check catches a wrong change the projection has already applied, and nothing sooner.
fn eventual_observation(
    ir: &EssIr,
    subject: &ResolvedSubject,
    setup: &Setup,
    instance: &super::InstanceName,
    unobserved: &mut BTreeSet<String>,
) -> Preservation {
    let entity = ir.entity(&subject.entity);
    let mut after = Vec::new();
    let mut source = BTreeSet::new();
    let Some(state) = setup.after.as_ref() else {
        return Preservation {
            before: Vec::new(),
            after,
            source,
            unobserved: Vec::new(),
        };
    };
    for view in ir.views().values().filter(|view| {
        !view.is_aggregate()
            && view.source == subject.entity
            && super::paging::read_whole(view)
            && view.assertion_style == AssertionStyle::Eventually
            && shows(ir, view, state, &setup.settled, &BTreeMap::new()) == Ok(true)
            && view
                .field(&entity.identity.name)
                .is_some_and(|field| field.type_ref == entity.identity.type_ref)
    }) {
        let mut row = BTreeMap::from([(
            entity.identity.name.clone(),
            ScenarioValue::instance(instance.clone()),
        )]);
        if view
            .field(EntitySpec::STATE)
            .is_some_and(|field| field.type_ref == entity.state_field().type_ref)
        {
            row.insert(
                EntitySpec::STATE.to_owned(),
                ScenarioValue::literal(Node::Text(state.to_string())),
            );
        }
        for field in &view.fields {
            let Some(determined) = setup.settled.get(&field.name) else {
                continue;
            };
            if determined.type_ref == field.type_ref
                && entity
                    .fields
                    .iter()
                    .any(|declared| declared.name == field.name)
            {
                row.insert(field.name.clone(), determined.value.clone());
            }
        }
        for observed in row.keys() {
            unobserved.remove(observed);
        }
        let name = ViewRef::new(view.name.clone());
        require(
            view,
            &name,
            BTreeMap::new(),
            ViewExpectation::Contains { fields: row },
            &mut after,
        );
        source.insert(name.into());
    }
    Preservation {
        before: Vec::new(),
        after,
        source,
        unobserved: Vec::new(),
    }
}
