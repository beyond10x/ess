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
        | ResolvedCondition::WrongState
        | ResolvedCondition::UnknownInstance => None,
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
fn guarded(command: &ResolvedCommand) -> impl Iterator<Item = &ResolvedOutcome> {
    command.outcomes.iter().filter(|branch| {
        !state_default(branch)
            && !matches!(
                branch.condition,
                ResolvedCondition::External { .. }
                    | ResolvedCondition::ExternalWhen { .. }
                    | ResolvedCondition::WrongState
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
pub(super) fn row_truth(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    predicate: &Predicate,
) -> Truth {
    row_truth_with(ir, entity, settled, predicate, None)
}

/// [`row_truth`], with the command's input bound under `input.` for a predicate that compares the
/// row with it (beyond10x/ess#157). Without an input such a comparison is `Unknown`, which is what
/// the search and the boundary goals see: the row alone does not decide it.
fn row_truth_with(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    predicate: &Predicate,
    input: Option<(&ResolvedCommand, &BTreeMap<String, Node>)>,
) -> Truth {
    let declared = ir.entity(entity);
    let values: BTreeMap<String, Node> = settled
        .iter()
        .filter_map(|(name, determined)| {
            determined
                .value
                .as_literal()
                .map(|value| (name.clone(), value.clone()))
        })
        .collect();
    for path in predicate.fact_paths() {
        let root = path.namespace();
        if declared.fields.iter().any(|field| field.name == root) && !values.contains_key(root) {
            return Truth::Unknown;
        }
    }
    let Ok(store) = crate::input::bind(
        ir,
        &declared.fields,
        &values,
        crate::input::Completeness::Partial,
    ) else {
        return Truth::Unknown;
    };
    let row = crate::input::TypedFacts::new(ir, &declared.fields, store);
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
/// value and another does not, and the guard is witnessed both ways (beyond10x/ess#157).
fn grounded(
    ir: &EssIr,
    entity: &EntityHandle,
    settled: &BTreeMap<String, super::Determined>,
    predicates: &[Predicate],
) -> Vec<Predicate> {
    let mut found = Vec::new();
    for predicate in predicates {
        leaves(predicate, &mut found);
    }
    let held = |path: &FactPath| -> Option<ess_primitives::facts::FactValue> {
        let (root, rest) = path.segments().split_first()?;
        let mut node = settled.get(root)?.value.as_literal()?;
        for segment in rest {
            let Node::Map(entries) = node else {
                return None;
            };
            node = entries.get(segment)?;
        }
        super::fact_value(node)
    };
    let side = |operand: &Operand| -> Option<Operand> {
        match operand {
            Operand::Fact(path) => match input_path(path) {
                Some(rest) => Some(Operand::Fact(rest)),
                None => held(path).map(Operand::Literal),
            },
            Operand::Literal(value) => Some(Operand::Literal(value.clone())),
        }
    };
    found
        .iter()
        .filter(|leaf| reads_input(ir, entity, leaf))
        .filter_map(|leaf| match leaf {
            Predicate::Compare { left, op, right } => Some(Predicate::Compare {
                left: side(left)?,
                op: *op,
                right: side(right)?,
            }),
            _ => None,
        })
        .collect()
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

/// Which branch this command selects for the row `arrangement` holds and this input, if exactly
/// one does.
///
/// A row whose state no move of the command starts from is the wrong-state family's, and selects
/// nothing here. An `Unknown` stored fact selects no branch, and never the default.
fn selects<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
    entity: &EntityHandle,
    arrangement: &Arrangement,
    input: &BTreeMap<String, Node>,
) -> Result<Option<&'a ResolvedOutcome>, RefusalCause> {
    if ir
        .wrong_states(command)
        .get(&entity)
        .is_some_and(|states| states.contains(&arrangement.state))
    {
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
    let pick = match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    };
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
    let pick = match selected.as_slice() {
        [] => command.outcomes.iter().find(|branch| state_default(branch)),
        [only] => Some(*only),
        _ => None,
    };
    Ok(pick.is_some_and(|branch| branch.name == outcome.name))
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
    let truth = |predicate: &Predicate| row_truth(ir, entity, &arrangement.settled, predicate);
    Profile {
        state: arrangement.state.to_string(),
        hints: hints.iter().map(|hint| code(truth(hint))).collect(),
        leaves: all.iter().map(|leaf| code(truth(leaf))).collect(),
        on_literal: all
            .iter()
            .map(|leaf| on_literal(leaf).is_some_and(|eq| truth(&eq) == Truth::True))
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
) -> Result<Vec<Arrangement>, RefusalCause> {
    let required = |reason| RefusalCause::InstanceRequired {
        entity: EntityRef::from(entity),
        need: InstanceNeed::Updates,
        reason,
    };
    let mut out =
        vec![created(ir, entity, creator, actors, distinction, &[], None).map_err(required)?];
    if has_subject_guards(creator.command)
        || creator.outcome.test_strategy == ess_domain::command::TestStrategy::InjectFault
    {
        return Ok(out);
    }
    for input in hinted(ir, entity, creator, hints)?.unwrap_or_default() {
        if input_selects(ir, creator.command, creator.outcome, &input)? {
            if let Ok(arrangement) =
                created(ir, entity, creator, actors, distinction, &[], Some(&input))
            {
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
    if let Ok(invoked) = invoke(
        ir,
        driver,
        Some(&arrangement.instance),
        Some(&arrangement.state),
        actors,
        Distinction::PLAIN,
        &BTreeMap::new(),
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
    {
        return out;
    }
    for input in hinted(ir, entity, driver, hints)
        .ok()
        .flatten()
        .unwrap_or_default()
    {
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
/// Breadth-first from the rows the creating branch can leave. At each depth every new node is
/// offered to the goal, and of those it accepts the one closest to the guards wins — ties to the
/// earlier, so the choice is a function of the model (§37).
fn search<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    hints: &[Predicate],
    distinction: Distinction,
    field: &str,
    mut goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    let creator = drivers
        .iter()
        .find(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .ok_or(RefusalCause::InstanceRequired {
            entity: EntityRef::from(entity),
            need: InstanceNeed::Updates,
            reason: Unreachable::NothingCreates,
        })?;
    let mut level = creations(ir, entity, creator, actors, hints, distinction)?;
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
                next.extend(successors(ir, entity, driver, node, actors, hints));
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
    let (arrangement, input) = search(
        ir,
        entity,
        actors,
        &hints,
        Distinction::PLAIN,
        &label,
        |node| reach_at(ir, command, outcome, entity, node),
    )?;
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
                        reach_at(ir, command, outcome, entity, node)?.filter(|input| {
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
            bound: BTreeMap::new(),
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
pub(super) fn reach_state(
    ir: &EssIr,
    entity: &EntityHandle,
    target: &super::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
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
    search(
        ir,
        entity,
        actors,
        &hints,
        Distinction::PLAIN,
        "state",
        |node| Ok((&node.state == target).then_some(())),
    )
    .map(|(arrangement, ())| arrangement)
}

/// The immediate, unfiltered, parameterless view a stored-field arrangement is observed through:
/// it projects the identity, `state` and every field named, each at the entity's declared type.
fn observer<'ir>(
    ir: &'ir EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
) -> Option<&'ir ess_compiler::ir::ResolvedView> {
    let declared = ir.entity(entity);
    ir.views().values().find(|view| {
        !view.is_aggregate()
            && view.source == *entity
            && view.params.is_empty()
            && view.filter.is_none()
            && view.assertion_style == AssertionStyle::Expect
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
    })
}

/// Observe one stored fact on the arranged row. Kept for the replay family, which observes the
/// facts one at a time.
pub(super) fn observe(
    ir: &EssIr,
    entity: &EntityHandle,
    field: &str,
    arrangement: &Arrangement,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    observe_fields(ir, entity, &BTreeSet::from([field.to_owned()]), arrangement)
}

/// Require the arranged row, with every guarded field at the value the arrangement determined,
/// before the command runs: the facts the scenario is about are observed, not assumed.
pub(super) fn observe_fields(
    ir: &EssIr,
    entity: &EntityHandle,
    fields: &BTreeSet<String>,
    arrangement: &Arrangement,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let label = fields.iter().cloned().collect::<Vec<_>>().join(",");
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
    let view = observer(ir, entity, fields).ok_or_else(|| {
        missing(
            entity,
            &label,
            "subject fact selection requires an immediate unfiltered identity/state/fact view",
        )
    })?;
    let name = ViewRef::new(view.name.clone());
    let mut steps = Vec::new();
    require(
        view,
        &name,
        BTreeMap::new(),
        ViewExpectation::Contains { fields: row },
        &mut steps,
    );
    Ok((steps, name))
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
    let view = observer(ir, &subject.entity, &BTreeSet::new()).ok_or_else(|| {
        missing(
            &subject.entity,
            &field.name,
            "subject fact selection requires an immediate unfiltered identity/state/fact view",
        )
    })?;
    let command_ref = CommandRef::new(command.name.clone());
    let mut steps = vec![ScenarioStep::ExecuteCommand {
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
    let (further, source) = boundaries(ir, command, outcome, actors, &setup.settled)?;
    let mut steps = around_row(ir, command, outcome, actors, setup, supplied)?;
    steps.extend(further);
    setup.source.extend(source);
    Ok(steps)
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
    let changes = uses_predicate(command)
        && outcome.subject.as_ref().is_some_and(|own| {
            matches!(
                own.effect,
                ResolvedEffect::Moves { .. } | ResolvedEffect::Updates
            )
        });
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
    let (observed, view) = observe_fields(ir, &subject.entity, &fields, &arrangement)?;
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
) -> Vec<Goal> {
    let mut goals = Vec::new();
    for hint in hints {
        if let Predicate::All(conjuncts) = hint {
            goals.extend(isolating(ir, entity, conjuncts, false, witnessed));
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
) -> Vec<Goal> {
    if state_default(outcome) {
        if outcome.subject.is_none() {
            return Vec::new();
        }
        return conjunct_goals(ir, entity, hints, witnessed);
    }
    stored(&outcome.condition)
        .map(|own| disjunct_goals(ir, entity, &own, witnessed))
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
        for (refuted, mut held) in isolating(ir, entity, disjuncts, true, witnessed) {
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
) -> Vec<Goal> {
    if children.len() < 2 {
        return Vec::new();
    }
    let wanted = |value: bool| if value { Truth::True } else { Truth::False };
    let mut goals = Vec::new();
    for index in 0..children.len() {
        if children.iter().enumerate().all(|(other, child)| {
            row_truth(ir, entity, witnessed, child) == wanted((other == index) == alone)
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
pub(super) fn boundaries(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    witnessed: &BTreeMap<String, super::Determined>,
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
    let goals = goals_for(ir, outcome, entity, &hints, witnessed);
    let command_ref = CommandRef::new(command.name.clone());
    let outcome_ref = OutcomeRef::new(command_ref.clone(), outcome.name.clone());
    let mut rows = 0;
    for (refuted, held) in goals {
        if rows >= MAX_BOUNDARIES {
            break;
        }
        let distinction = Distinction::further(rows + 1);
        let found = search(
            ir,
            entity,
            actors,
            &hints,
            distinction,
            "boundary",
            |node| {
                let truth = |predicate: &Predicate| row_truth(ir, entity, &node.settled, predicate);
                if refuted.iter().any(|child| truth(child) != Truth::False)
                    || held.iter().any(|child| truth(child) != Truth::True)
                {
                    return Ok(None);
                }
                reach_at(ir, command, outcome, entity, node)
            },
        );
        let Ok((mut arrangement, input)) = found else {
            continue;
        };
        rows += 1;
        let (observed, view) = observe_fields(ir, entity, &fields, &arrangement)?;
        arrangement.steps.extend(observed);
        steps.append(&mut arrangement.steps);
        source.append(&mut arrangement.source);
        source.insert(view.into());
        let supplied = supply(
            command,
            &input,
            Some(read),
            Some(&arrangement.instance),
            &BTreeMap::new(),
        );
        steps.push(ScenarioStep::ExecuteCommand {
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
            input: supplied.clone(),
        });
        steps.push(ScenarioStep::ExpectOutcome {
            outcome: outcome_ref.clone(),
        });
        match &outcome.error {
            Some(error) => steps.push(ScenarioStep::ExpectError {
                error: super::ErrorRef::from(error),
                fields: BTreeMap::new(),
            }),
            None => steps.push(ScenarioStep::ExpectNoError),
        }
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
        let (after, _) = observe_fields(ir, entity, &kept, &arrangement)?;
        steps.extend(after);
    }
    if rows > 0 {
        source.insert(command_ref.into());
        source.insert(outcome_ref.into());
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
            && view.params.is_empty()
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
            && view.params.is_empty()
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
