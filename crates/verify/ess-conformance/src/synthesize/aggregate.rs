//! One scenario per aggregate view: rows this scenario made, and every group's exact aggregates.
//!
//! `docs/design/aggregate-views.md`, "Conformance". An aggregate is an exact number — a ceiling and a
//! floor at once — so unlike every other view assertion in this module it cannot be a floor that a
//! shared target (§8) is allowed to exceed. The scenario therefore gives its groups key values no
//! other scenario of the suite produces (**scoping**), creates the rows through the declared creating
//! outcome with a fixed pattern of values (**arrangement**), and reads the view once with a
//! `Contains` per group that holds an admitted row and an `Excludes` per group whose rows are all
//! refuted (**observation**). Every expected number is computed by [`crate::aggregate::evaluate`]
//! over what the arrangement actually determined, never restated here.
//!
//! # What this search does not do
//!
//! The page lets the filter goal override an input's value where a filter reads one. This search
//! reaches a row's filter truth through the lifecycle state the declared moves reach and, for a
//! parameter-scoped filter, through the `<view>/out` value — nothing else. A filter whose truth
//! depends on an input the pattern fixes, and that no reachable state decides as the row needs, is
//! refused as `ESS-SYNTH-017` naming the row, which the page permits ("a filter truth it cannot
//! reach") and which never asserts a wrong number.
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    Driver, EntityHandle, EssIr, ResolvedAggregation, ResolvedBody, ResolvedCondition,
    ResolvedEffect, ResolvedEntity, ResolvedPayloadValue, ResolvedTypeRef, ResolvedView,
};
use ess_domain::entity::{EntitySpec, StateName};
use ess_domain::name::QualifiedName;
use ess_domain::types::{Primitive, MAX_TYPE_DEPTH};
use ess_domain::view::{AggregateFunction, AssertionStyle, ViewSpec};
use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use super::{
    advance, clipped, created, has_subject_guards, insert, literal_value, reach, reachable_types,
    route, shows, subject_fact, Arrangement, Refusal, RefusalCause,
};
use crate::aggregate::{evaluate, ValueKind};
use crate::scenario::{
    ActorRef, ConformanceScenario, ConformanceSuite, EntityRef, EssSemanticRef, ScenarioId,
    ScenarioStep, ScenarioValue, ViewExpectation, ViewRef,
};
use crate::witness::{uuid_of, Distinction};

/// The most inputs the page's pattern keeps apart: seven, with a group of nine rows.
const MAX_INPUTS: usize = 7;

/// Every aggregate view's scenario, or the refusal that says why it has none.
pub(super) fn aggregates(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let literals = model_literals(ir);
    for view in ir.views().values() {
        let Some(aggregation) = &view.aggregation else {
            continue;
        };
        let id = ScenarioId::Aggregate {
            view: ViewRef::new(view.name.clone()),
        };
        match scenario(ir, view, aggregation, actors, &literals) {
            Ok(scenario) => insert(suite, id, scenario, refusals),
            Err(cause) => refusals.push(Refusal::about(&id, cause)),
        }
    }
}

/// What an entity field's type unwraps to, through newtypes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Leaf {
    Primitive(Primitive),
    /// An enum, with its variants in declaration order.
    Enum(Vec<String>),
    Other,
}

fn leaf(ir: &EssIr, type_ref: &ResolvedTypeRef) -> (bool, Leaf) {
    let mut optional = false;
    let mut current = type_ref;
    for _ in 0..=MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Primitive { name } => return (optional, Leaf::Primitive(*name)),
            ResolvedTypeRef::Optional { of } => {
                optional = true;
                current = of;
            }
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => current = of,
                ResolvedBody::Enum { variants } => {
                    return (
                        optional,
                        Leaf::Enum(
                            variants
                                .iter()
                                .map(|variant| variant.name.clone())
                                .collect(),
                        ),
                    )
                }
                ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => break,
            },
            ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => break,
        }
    }
    (optional, Leaf::Other)
}

/// How one value of a field is chosen at a ladder ordinal (`docs/design/aggregate-views.md`,
/// "The ladder").
#[derive(Debug, Clone, PartialEq, Eq)]
enum Ladder {
    Number,
    Text { prefix: String },
    Timestamp,
    Enum(Vec<String>),
    Boolean,
    Uuid { prefix: String },
}

impl Ladder {
    fn of(view: &QualifiedName, path: &str, leaf: &Leaf) -> Option<Self> {
        Some(match leaf {
            Leaf::Primitive(Primitive::Integer | Primitive::Decimal) => Self::Number,
            Leaf::Primitive(Primitive::String) => Self::Text {
                prefix: format!("{view}/{path}"),
            },
            Leaf::Primitive(Primitive::Timestamp) => Self::Timestamp,
            Leaf::Primitive(Primitive::Boolean) => Self::Boolean,
            Leaf::Primitive(Primitive::Uuid) => Self::Uuid {
                prefix: format!("{view}#{path}"),
            },
            Leaf::Enum(variants) if !variants.is_empty() => Self::Enum(variants.clone()),
            _ => return None,
        })
    }

    fn at(&self, ordinal: usize) -> Node {
        match self {
            Self::Number => Node::Number(Number::from(ordinal)),
            Self::Text { prefix } => Node::Text(format!("{prefix}/{ordinal:03}")),
            Self::Timestamp => {
                let (days, minutes) = (ordinal / 1440, ordinal % 1440);
                Node::Text(format!(
                    "2026-01-{:02}T{:02}:{:02}:00Z",
                    days + 1,
                    minutes / 60,
                    minutes % 60
                ))
            }
            Self::Enum(variants) => Node::Text(variants[ordinal % variants.len()].clone()),
            Self::Boolean => Node::Bool(ordinal % 2 == 1),
            Self::Uuid { prefix } => Node::Text(uuid_of(&format!("{prefix}#{ordinal}"))),
        }
    }

    /// How many distinct values the ladder has, where that is finite.
    fn len(&self) -> Option<usize> {
        match self {
            Self::Enum(variants) => Some(variants.len()),
            Self::Boolean => Some(2),
            _ => None,
        }
    }
}

/// Where a scoped value is text and where it is a `Uuid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scoped {
    Text,
    Uuid,
}

/// How one group key's values are chosen.
#[derive(Debug, Clone)]
enum Key {
    /// A `String` or `Uuid` key the creating command sets from its input: `<view>/<group>`.
    Scoped(Scoped),
    /// Any other key the creating command sets from its input, walked along its value sequence.
    Walked(Ladder),
    /// The lifecycle state, reached by the declared moves.
    State(Vec<StateName>),
    /// A key the creating command sets to one literal: a sequence of one value.
    Fixed(Node),
}

impl Key {
    /// The `n`th value of this key's sequence, for a non-scoped key.
    fn sequence(&self, n: usize) -> Option<Node> {
        match self {
            Self::Scoped(_) => None,
            Self::Walked(ladder) => Some(ladder.at(match ladder.len() {
                Some(len) => n % len,
                None => n,
            })),
            Self::State(states) => {
                (!states.is_empty()).then(|| Node::Text(states[n % states.len()].to_string()))
            }
            Self::Fixed(value) => Some(value.clone()),
        }
    }

    /// How many distinct values the sequence has, where that is finite.
    fn len(&self) -> Option<usize> {
        match self {
            Self::Scoped(_) => None,
            Self::Walked(ladder) => ladder.len(),
            Self::State(states) => Some(states.len()),
            Self::Fixed(_) => Some(1),
        }
    }
}

/// One declared parameter bound to a scoped value: the filter reads it as `field == param.name`.
#[derive(Debug, Clone)]
struct Scope {
    param: String,
    field: String,
    kind: Scoped,
}

/// One row the scenario creates.
#[derive(Debug, Clone)]
struct Row {
    /// Which group tuple it is placed in, by position in [`Plan::tuples`].
    tuple: usize,
    /// Whether the filter must admit it.
    admitted: bool,
    /// The entity fields the creating command is to set, by name.
    values: BTreeMap<String, Node>,
    /// What the row is called in a refusal.
    label: String,
}

/// Everything the arrangement decided before any command is chosen.
struct Plan<'ir> {
    ir: &'ir EssIr,
    view: &'ir ResolvedView,
    aggregation: &'ir ResolvedAggregation,
    handle: &'ir EntityHandle,
    entity: &'ir ResolvedEntity,
    literals: &'ir BTreeSet<String>,
    keys: Vec<(String, Key)>,
    scopes: Vec<Scope>,
    /// The group tuples in assignment order, labelled `A`, `B`, `C`, `B2`, …
    tuples: Vec<(String, Vec<Node>)>,
}

impl Plan<'_> {
    fn unwitnessed(&self, reason: impl Into<String>) -> RefusalCause {
        unwitnessed(self.view, reason)
    }

    /// `<view>/<suffix>`, moved past every text literal the model writes (`#2`, `#3`, …).
    fn scoped_text(&self, suffix: &str) -> String {
        let base = format!("{}/{suffix}", self.view.name);
        if !self.literals.contains(&base) {
            return base;
        }
        // Of `len + 1` suffixes, at least one is no literal of the model.
        (2..=self.literals.len() + 2)
            .map(|n| format!("{base}#{n}"))
            .find(|candidate| !self.literals.contains(candidate))
            .unwrap_or_else(|| base.clone())
    }

    fn scoped(&self, kind: Scoped, suffix: &str) -> Node {
        match kind {
            Scoped::Text => Node::Text(self.scoped_text(suffix)),
            Scoped::Uuid => Node::Text(uuid_of(&format!("{}/{suffix}", self.view.name))),
        }
    }

    fn params(&self, suffix: &str) -> BTreeMap<String, ScenarioValue> {
        self.scopes
            .iter()
            .map(|scope| {
                (
                    scope.param.clone(),
                    ScenarioValue::literal(self.scoped(scope.kind, suffix)),
                )
            })
            .collect()
    }
}

fn unwitnessed(view: &ResolvedView, reason: impl Into<String>) -> RefusalCause {
    RefusalCause::AggregateUnwitnessed {
        view: ViewRef::new(view.name.clone()),
        reason: reason.into(),
    }
}

/// Every text literal the model writes where a value is compared or set: guards, invariants, view
/// filters, `sets:` and payload literals. A scoped value equal to one of them is moved past it.
fn model_literals(ir: &EssIr) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            match &outcome.condition {
                ResolvedCondition::SubjectField {
                    equals, predicate, ..
                } => {
                    out.insert(equals.clone());
                    if let Some(predicate) = predicate {
                        texts(predicate, &mut out);
                    }
                }
                ResolvedCondition::SubjectPredicate { predicate, input } => {
                    texts(predicate, &mut out);
                    if let Some(input) = input {
                        texts(input, &mut out);
                    }
                }
                ResolvedCondition::When { predicate }
                | ResolvedCondition::ExternalWhen { predicate, .. } => texts(predicate, &mut out),
                ResolvedCondition::SubjectState { predicate, .. }
                | ResolvedCondition::StateChange { predicate, .. } => {
                    if let Some(predicate) = predicate {
                        texts(predicate, &mut out);
                    }
                }
                ResolvedCondition::Otherwise
                | ResolvedCondition::External { .. }
                | ResolvedCondition::WrongState => {}
            }
            let written = outcome
                .sets
                .iter()
                .chain(outcome.payload.iter().flat_map(|payload| &payload.fields));
            for field in written {
                if let ResolvedPayloadValue::Literal { value } = &field.value {
                    out.insert(value.clone());
                }
            }
        }
    }
    for entity in ir.entities().values() {
        for invariant in &entity.invariants {
            texts(&invariant.predicate, &mut out);
        }
    }
    for view in ir.views().values() {
        if let Some(filter) = &view.filter {
            texts(filter, &mut out);
        }
    }
    out
}

fn texts(predicate: &Predicate, out: &mut BTreeSet<String>) {
    let value = |value: &FactValue, out: &mut BTreeSet<String>| {
        if let FactValue::Text(text) = value {
            out.insert(text.clone());
        }
    };
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                texts(child, out);
            }
        }
        Predicate::Not(inner) => texts(inner, out),
        Predicate::Compare { left, right, .. } => {
            for operand in [left, right] {
                if let Operand::Literal(literal) = operand {
                    value(literal, out);
                }
            }
        }
        Predicate::AnyOf { values, .. } | Predicate::NoneOf { values, .. } => {
            for literal in values {
                value(literal, out);
            }
        }
        Predicate::TextMatch { value: literal, .. } => value(literal, out),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            texts(&quantified.body, out);
        }
        _ => {}
    }
}

/// The top-level conjuncts of a filter.
fn conjuncts(filter: Option<&Predicate>) -> Vec<&Predicate> {
    match filter {
        None => Vec::new(),
        Some(Predicate::All(children)) => children.iter().collect(),
        Some(other) => vec![other],
    }
}

/// `field == param.name`, either way round, as `(field, name)`.
fn scoping_equality(predicate: &Predicate) -> Option<(String, String)> {
    let Predicate::Compare {
        left: Operand::Fact(left),
        op: CompareOp::Eq,
        right: Operand::Fact(right),
    } = predicate
    else {
        return None;
    };
    let param = |path: &FactPath| match path.segments() {
        [namespace, name] if namespace == ViewSpec::PARAM => Some(name.clone()),
        _ => None,
    };
    let field = |path: &FactPath| match path.segments() {
        [name] => Some(name.clone()),
        _ => None,
    };
    match (param(left), param(right)) {
        (None, Some(name)) => Some((field(left)?, name)),
        (Some(name), None) => Some((field(right)?, name)),
        _ => None,
    }
}

/// `m`: the smallest group size at least `max(3, inputs + 2)` with a prime factor other than 2
/// and 5, so a mean over it can repeat forever at the sixth decimal.
fn group_size(inputs: usize) -> usize {
    let mut size = (inputs + 2).max(3);
    loop {
        let mut rest = size;
        for factor in [2, 5] {
            while rest % factor == 0 {
                rest /= factor;
            }
        }
        if rest > 1 {
            return size;
        }
        size += 1;
    }
}

/// The ladder ordinal of input `i` at A-row `j`: `100·i + 1 + t(t+1)`, `t = max(0, j − i − 1)`.
fn a_ordinal(i: usize, j: usize) -> usize {
    let t = j.saturating_sub(i + 1);
    100 * i + 1 + t * (t + 1)
}

/// Whether the mean `sum / m` of integers, taken to six places, leaves a remainder of more than
/// half: its seventh decimal and beyond round it up, so a truncating `avg` reports another number.
fn separates(sum: usize, m: usize) -> bool {
    let remainder = (sum * 1_000_000) % m;
    2 * remainder > m
}

#[allow(clippy::too_many_lines)]
fn scenario(
    ir: &EssIr,
    view: &ResolvedView,
    aggregation: &ResolvedAggregation,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    literals: &BTreeSet<String>,
) -> Result<ConformanceScenario, RefusalCause> {
    let handle = &view.source;
    let entity = ir.entity(handle);
    let all = ir.drivers();
    let drivers: &[Driver<'_>] = all.get(handle).map_or(&[], Vec::as_slice);
    let creator = drivers
        .iter()
        .find(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .ok_or_else(|| unwitnessed(view, format!("nothing creates `{}`", entity.name)))?;
    if has_subject_guards(creator.command)
        || creator.outcome.test_strategy == ess_domain::command::TestStrategy::InjectFault
    {
        return Err(unwitnessed(
            view,
            format!(
                "`{}` creates `{}` on a branch its input alone does not select",
                creator.command.name, entity.name
            ),
        ));
    }
    // The entity fields the creating branch fills from its input, and the input field each reads.
    let mapped: BTreeMap<&str, &str> = creator
        .outcome
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
        .filter_map(|set| match &set.value {
            ResolvedPayloadValue::InputField { field, .. } => {
                Some((set.target.as_str(), field.as_str()))
            }
            _ => None,
        })
        .collect();
    let fixed = |field: &str| {
        creator
            .outcome
            .sets
            .iter()
            .find_map(|set| match &set.value {
                ResolvedPayloadValue::Literal { value } if set.target == field => {
                    literal_value(ir, &set.target_type, value, 0)
                }
                _ => None,
            })
    };
    let field_type = |name: &str| {
        entity
            .observable_field(name)
            .map(|field| leaf(ir, &field.type_ref))
    };
    let scopable = |name: &str| -> Option<Scoped> {
        if name == entity.identity.name || name == EntitySpec::STATE || !mapped.contains_key(name) {
            return None;
        }
        match field_type(name)? {
            (false, Leaf::Primitive(Primitive::String)) => Some(Scoped::Text),
            (false, Leaf::Primitive(Primitive::Uuid)) => Some(Scoped::Uuid),
            _ => None,
        }
    };

    // Scoping by parameter: every declared parameter is read by exactly one top-level
    // `field == param.name` conjunct over a scopable field that is not a group key.
    let mut scopes = Vec::new();
    for param in &view.params {
        let read = FactPath::from_segments([ViewSpec::PARAM, param.name.as_str()]);
        let reads = view.filter.as_ref().map_or(0, |filter| {
            filter
                .fact_paths()
                .into_iter()
                .filter(|path| **path == read)
                .count()
        });
        let scoping = conjuncts(view.filter.as_ref())
            .into_iter()
            .filter_map(scoping_equality)
            .find(|(_, name)| *name == param.name);
        match scoping {
            Some((field, _)) if reads == 1 && !aggregation.group_by.contains(&field) => {
                let Some(kind) = scopable(&field) else {
                    return Err(unwitnessed(
                        view,
                        format!(
                            "the parameter `{}` is compared with `{field}`, which is not a \
                             `String` or `Uuid` field the creating command sets from its input",
                            param.name
                        ),
                    ));
                };
                scopes.push(Scope {
                    param: param.name.clone(),
                    field,
                    kind,
                });
            }
            _ => {
                return Err(unwitnessed(
                    view,
                    format!(
                    "the parameter `{}` is read other than by one top-level `field == param.{}` \
                         conjunct over a field that is not a group key",
                    param.name, param.name
                ),
                ))
            }
        }
    }

    let mut keys = Vec::new();
    for key in &aggregation.group_by {
        let chosen = if let Some(kind) = scopable(key) {
            Key::Scoped(kind)
        } else if key == EntitySpec::STATE {
            Key::State(entity.lifecycle.states.iter().cloned().collect())
        } else if key == &entity.identity.name {
            Key::Fixed(Node::Null)
        } else if mapped.contains_key(key.as_str()) {
            let (_, found) = field_type(key).unwrap_or((false, Leaf::Other));
            match Ladder::of(&view.name, key, &found) {
                Some(ladder) => Key::Walked(ladder),
                None => {
                    return Err(unwitnessed(
                        view,
                        format!("no value ladder walks the group key `{key}`"),
                    ))
                }
            }
        } else if let Some(value) = fixed(key) {
            Key::Fixed(value)
        } else {
            return Err(unwitnessed(
                view,
                format!("the creating command does not set the group key `{key}`"),
            ));
        };
        keys.push((key.clone(), chosen));
    }
    if !keys.iter().any(|(_, key)| matches!(key, Key::Scoped(_))) && scopes.is_empty() {
        return Err(RefusalCause::AggregateUnscoped {
            view: ViewRef::new(view.name.clone()),
        });
    }
    // The identity is scopable by nothing and differs in every row, so no group of `m` rows
    // shares one; it is scoped out above only when nothing else scopes the view.
    if let Some((key, _)) = keys.iter().find(|(key, _)| *key == entity.identity.name) {
        return Err(unwitnessed(
            view,
            format!("the group key `{key}` is the identity, which no two rows share"),
        ));
    }

    // The inputs: the distinct aggregate arguments, in field order, that are neither a key nor a
    // scope field, and that the creating command sets from its input.
    let keyed: BTreeSet<&str> = aggregation
        .group_by
        .iter()
        .map(String::as_str)
        .chain(scopes.iter().map(|scope| scope.field.as_str()))
        .collect();
    let mut inputs: Vec<(String, Ladder)> = Vec::new();
    for field in &view.fields {
        let Some(aggregate) = aggregation.functions.get(&field.name) else {
            continue;
        };
        let Some(input) = &aggregate.input else {
            continue;
        };
        let name = input.name.as_str();
        if keyed.contains(name)
            || name == entity.identity.name
            || name == EntitySpec::STATE
            || inputs.iter().any(|(held, _)| held == name)
        {
            continue;
        }
        if mapped.contains_key(name) {
            let (_, found) = field_type(name).unwrap_or((false, Leaf::Other));
            let ladder = Ladder::of(&view.name, name, &found).ok_or_else(|| {
                unwitnessed(view, format!("no value ladder walks the input `{name}`"))
            })?;
            inputs.push((name.to_owned(), ladder));
        } else if fixed(name).is_none() {
            return Err(unwitnessed(
                view,
                format!(
                    "the creating command does not set `{name}`, which `{}` aggregates",
                    field.name
                ),
            ));
        }
    }
    if inputs.len() > MAX_INPUTS {
        return Err(unwitnessed(
            view,
            format!(
                "{} inputs are more than the {MAX_INPUTS} one group's pattern keeps apart",
                inputs.len()
            ),
        ));
    }
    // The identity is the one value a row holds that the target, not the scenario, chooses. Its
    // distinct count is the row count whatever it is; its sum, extremes or mean are a number no
    // expected value can name.
    if let Some((field, aggregate)) = aggregation.functions.iter().find(|(_, aggregate)| {
        !matches!(
            aggregate.function,
            AggregateFunction::Count | AggregateFunction::CountDistinct
        ) && aggregate
            .input
            .as_ref()
            .is_some_and(|input| input.name == entity.identity.name)
    }) {
        return Err(unwitnessed(
            view,
            format!(
                "`{field}` computes {aggregate} over the identity, which the target generates, so \
                 no expected value can be named"
            ),
        ));
    }
    let m = group_size(inputs.len());

    let mut plan = Plan {
        ir,
        view,
        aggregation,
        handle,
        entity,
        literals,
        keys,
        scopes,
        tuples: Vec::new(),
    };
    assign_tuples(&mut plan);
    let rows = rows(&plan, &inputs, m);
    arrange_and_observe(&plan, creator, &mapped, rows, actors)
}

/// Group tuples in the order A, B, C, B₂, …, each differing from every tuple before it.
fn assign_tuples(plan: &mut Plan<'_>) {
    if plan.aggregation.is_ungrouped() {
        plan.tuples.push(("A".to_owned(), Vec::new()));
        return;
    }
    let start = |plan: &Plan<'_>, label: &str, n: usize| -> Vec<Node> {
        plan.keys
            .iter()
            .map(|(_, key)| match key {
                Key::Scoped(kind) => plan.scoped(*kind, label),
                other => other.sequence(n).unwrap_or(Node::Null),
            })
            .collect()
    };
    let a = start(plan, "A", 0);
    let b = start(plan, "B", 1);
    let c = start(plan, "C", 2);
    plan.tuples.push(("A".to_owned(), a));
    plan.tuples.push(("B".to_owned(), b.clone()));
    if plan.view.filter.is_some() {
        plan.tuples.push(("C".to_owned(), c));
    }
    // One Bₖ per key that another key does not already tell apart from B: every non-scoped key,
    // the first included — a non-scoped first key with no Bₖ is a key an implementation may ignore
    // and pass — and every scoped key after the first. The first key, when scoped, is what A, B and
    // C already differ in.
    for index in 0..plan.keys.len() {
        let label = format!("B{}", index + 1);
        let candidates: Vec<Node> = match &plan.keys[index].1 {
            Key::Scoped(_) if index == 0 => continue,
            Key::Scoped(kind) => vec![plan.scoped(*kind, &label)],
            key => {
                let bound = key.len().unwrap_or(plan.tuples.len() + 1);
                (0..bound).filter_map(|n| key.sequence(n)).collect()
            }
        };
        let found = candidates.into_iter().find_map(|value| {
            let mut tuple = b.clone();
            tuple[index] = value;
            (!plan.tuples.iter().any(|(_, held)| *held == tuple)).then_some(tuple)
        });
        if let Some(tuple) = found {
            plan.tuples.push((label, tuple));
        }
    }
}

/// The rows, in creation order: a₁ … aₘ, b, x, c, then one bₖ per further tuple.
fn rows(plan: &Plan<'_>, inputs: &[(String, Ladder)], m: usize) -> Vec<Row> {
    let row = |tuple: usize, admitted: bool, label: String, ordinals: Vec<usize>| {
        let mut values = BTreeMap::new();
        for ((name, key), value) in plan.keys.iter().zip(&plan.tuples[tuple].1) {
            if !matches!(key, Key::State(_) | Key::Fixed(_)) {
                values.insert(name.clone(), value.clone());
            }
        }
        for scope in &plan.scopes {
            values.insert(scope.field.clone(), plan.scoped(scope.kind, "in"));
        }
        for ((name, ladder), ordinal) in inputs.iter().zip(ordinals) {
            values.insert(name.clone(), ladder.at(ordinal));
        }
        Row {
            tuple,
            admitted,
            values,
            label,
        }
    };
    let ladder = |offset: usize| {
        (0..inputs.len())
            .map(|i| 100 * i + offset)
            .collect::<Vec<_>>()
    };

    let mut a: Vec<Vec<usize>> = (0..m)
        .map(|j| (0..inputs.len()).map(|i| a_ordinal(i, j)).collect())
        .collect();
    // The A mean of every `avg` input must separate rounding from truncation: its seventh decimal
    // is 5 or more (with no tie), so `avg` rounded and `avg` truncated are different numbers. The
    // last A value is raised by the smallest δ < m that makes it so — it stays the largest value in
    // its column, and below `100·i + 85` (`docs/design/aggregate-views.md`, "Non-terminating
    // mean"). Where no δ does, the check in `observe` refuses the view.
    let averaged: BTreeSet<&str> = plan
        .aggregation
        .functions
        .values()
        .filter(|aggregate| aggregate.function == AggregateFunction::Avg)
        .filter_map(|aggregate| aggregate.input.as_ref().map(|input| input.name.as_str()))
        .collect();
    for (i, (name, ladder)) in inputs.iter().enumerate() {
        if *ladder != Ladder::Number || !averaged.contains(name.as_str()) {
            continue;
        }
        let sum: usize = a.iter().map(|ordinals| ordinals[i]).sum();
        if let Some(delta) = (0..m).find(|delta| separates(sum + delta, m)) {
            a[m - 1][i] += delta;
        }
    }

    let mut out: Vec<Row> = a
        .into_iter()
        .enumerate()
        .map(|(j, ordinals)| row(0, true, format!("a{}", j + 1), ordinals))
        .collect();
    let filtered = plan.view.filter.is_some();
    if !plan.aggregation.is_ungrouped() {
        out.push(row(1, true, "b".to_owned(), ladder(85)));
    }
    if filtered {
        out.push(row(0, false, "x".to_owned(), ladder(97)));
    }
    if !plan.aggregation.is_ungrouped() {
        if filtered {
            out.push(row(2, false, "c".to_owned(), ladder(93)));
        }
        let first_further = if filtered { 3 } else { 2 };
        for tuple in first_further..plan.tuples.len() {
            let label = format!("b{}", &plan.tuples[tuple].0[1..]);
            out.push(row(tuple, true, label, ladder(87)));
        }
    }
    out
}

/// One arranged row: what the steps left, and what the filter says of it.
struct Arranged {
    row: Row,
    arrangement: Arrangement,
    admitted: bool,
}

fn arrange_and_observe(
    plan: &Plan<'_>,
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    rows: Vec<Row>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let ir = plan.ir;
    let base = reach(ir, creator.command, creator.outcome, Distinction::PLAIN).map_err(|_| {
        plan.unwitnessed(format!(
            "no input reaches `{}/{}`, which creates the rows",
            creator.command.name, creator.outcome.name
        ))
    })?;
    let params = plan.params("in");
    let mut arranged = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        let distinction = Distinction::further(index + 1);
        let mut attempt = row.clone();
        let result = match arrange_row(
            plan,
            creator,
            mapped,
            &base,
            &attempt,
            distinction,
            actors,
            &params,
        ) {
            Ok(done) => Ok(done),
            // A refuted row of a parameter-scoped filter that no state refutes is moved out of the
            // scope the query binds.
            Err(_) if !attempt.admitted && !plan.scopes.is_empty() => {
                for scope in &plan.scopes {
                    attempt
                        .values
                        .insert(scope.field.clone(), plan.scoped(scope.kind, "out"));
                }
                arrange_row(
                    plan,
                    creator,
                    mapped,
                    &base,
                    &attempt,
                    distinction,
                    actors,
                    &params,
                )
            }
            Err(error) => Err(error),
        }?;
        arranged.push(Arranged {
            row: attempt,
            arrangement: result.0,
            admitted: result.1,
        });
    }
    observe(plan, &arranged, &params)
}

/// Create one row with its pattern values and drive it to a state its filter truth needs.
#[allow(clippy::too_many_arguments)]
fn arrange_row(
    plan: &Plan<'_>,
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    base: &BTreeMap<String, Node>,
    row: &Row,
    distinction: Distinction,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<(Arrangement, bool), RefusalCause> {
    let ir = plan.ir;
    let mut input = base.clone();
    for (field, value) in &row.values {
        let Some(read) = mapped.get(field.as_str()) else {
            return Err(plan.unwitnessed(format!(
                "the creating command does not set `{field}` for row `{}`",
                row.label
            )));
        };
        input.insert((*read).to_owned(), value.clone());
    }
    if !subject_fact::input_selects(ir, creator.command, creator.outcome, &input).unwrap_or(false) {
        return Err(plan.unwitnessed(format!(
            "`{}/{}` is not selected by the input row `{}` needs",
            creator.command.name, creator.outcome.name, row.label
        )));
    }
    let start = created(
        ir,
        plan.handle,
        creator,
        actors,
        distinction,
        &[],
        Some(&input),
    )
    .map_err(|_| plan.unwitnessed(format!("row `{}` cannot be created", row.label)))?;

    let wanted_state =
        plan.keys
            .iter()
            .zip(&plan.tuples[row.tuple].1)
            .find_map(|((_, key), value)| match (key, value) {
                (Key::State(_), Node::Text(state)) => StateName::new(state).ok(),
                _ => None,
            });
    let all = ir.drivers();
    let drivers: &[Driver<'_>] = all.get(plan.handle).map_or(&[], Vec::as_slice);
    let targets: Vec<StateName> = match wanted_state {
        Some(state) => vec![state],
        None => plan.entity.lifecycle.states.iter().cloned().collect(),
    };
    let mut best: Option<Arrangement> = None;
    for target in targets {
        let Some(path) = route(ir, plan.handle, drivers, &target) else {
            continue;
        };
        let Ok(reached) = advance(
            ir,
            plan.handle,
            start.clone(),
            path,
            &target,
            actors,
            distinction,
            &[],
        ) else {
            continue;
        };
        if shows(ir, plan.view, &reached.state, &reached.settled, params) != Ok(row.admitted) {
            continue;
        }
        if best
            .as_ref()
            .is_none_or(|held| reached.steps.len() < held.steps.len())
        {
            best = Some(reached);
        }
    }
    let reached = best.ok_or_else(|| {
        plan.unwitnessed(format!(
            "no reachable state leaves row `{}` {} by the filter",
            row.label,
            if row.admitted { "admitted" } else { "refuted" }
        ))
    })?;
    kept_as_planned(plan, row, &start, &reached)?;
    Ok((reached, row.admitted))
}

/// Refuse a row whose group keys or aggregate inputs the arranging moves rewrote.
///
/// The pattern — scoped keys, distinct tuples, each input's duplicate counts — is what makes the
/// asserted numbers exact and discriminating, and it holds only for the values the scenario chose.
/// A move whose `sets:` overwrites one leaves a group keyed by a value another scenario may also
/// produce, or an input pattern nobody checked, so the row is refused rather than asserted.
fn kept_as_planned(
    plan: &Plan<'_>,
    row: &Row,
    created: &Arrangement,
    reached: &Arrangement,
) -> Result<(), RefusalCause> {
    let literal = |arrangement: &Arrangement, field: &str| {
        arrangement
            .settled
            .get(field)
            .map(|determined| determined.value.clone())
    };
    let read =
        plan.aggregation
            .group_by
            .iter()
            .map(String::as_str)
            .chain(
                plan.aggregation.functions.values().filter_map(|aggregate| {
                    aggregate.input.as_ref().map(|input| input.name.as_str())
                }),
            )
            .chain(plan.scopes.iter().map(|scope| scope.field.as_str()));
    for field in read {
        if field == plan.entity.identity.name || field == EntitySpec::STATE {
            continue;
        }
        let planned = match row.values.get(field) {
            Some(value) => Some(ScenarioValue::literal(value.clone())),
            None => literal(created, field),
        };
        if literal(reached, field) != planned {
            return Err(plan.unwitnessed(format!(
                "a move on the way to row `{}`'s state rewrites `{field}`, which the pattern \
                 chose; its groups would no longer be the scenario's own",
                row.label
            )));
        }
    }
    Ok(())
}

/// The value a row holds for an aggregate's input or a group key, read off what was arranged.
fn held(plan: &Plan<'_>, arranged: &Arranged, index: usize, field: &str) -> Option<Node> {
    if field == EntitySpec::STATE {
        return Some(Node::Text(arranged.arrangement.state.to_string()));
    }
    if field == plan.entity.identity.name {
        // A generated identity nobody can spell; every row's is its own, which is all
        // `count_distinct` reads of it.
        return Some(Node::Text(format!("row-{index}")));
    }
    match &arranged.arrangement.settled.get(field)?.value {
        ScenarioValue::Literal { value } => Some(value.clone()),
        _ => None,
    }
}

fn kind(ir: &EssIr, entity: &ResolvedEntity, field: &str) -> ValueKind {
    if field == EntitySpec::STATE {
        return ValueKind::Other;
    }
    match entity
        .observable_field(field)
        .map(|found| leaf(ir, &found.type_ref).1)
    {
        Some(Leaf::Primitive(Primitive::Integer | Primitive::Decimal)) => ValueKind::Numeric,
        Some(Leaf::Primitive(Primitive::String)) => ValueKind::Text,
        Some(Leaf::Primitive(Primitive::Timestamp)) => ValueKind::Timestamp,
        _ => ValueKind::Other,
    }
}

/// One group: the key values its rows hold, and the indices of those rows.
type Group = (Vec<Node>, Vec<usize>);

/// The groups, keyed by what each row actually holds, in tuple order: each tuple and the indices
/// of the rows that hold it.
fn groups(plan: &Plan<'_>, arranged: &[Arranged]) -> Result<Vec<Group>, RefusalCause> {
    let mut groups: Vec<Group> = Vec::new();
    let mut order: Vec<(usize, usize)> = arranged
        .iter()
        .enumerate()
        .map(|(index, row)| (row.row.tuple, index))
        .collect();
    order.sort_by_key(|(tuple, index)| (*tuple, *index));
    for (_, index) in order {
        let mut tuple = Vec::new();
        for key in &plan.aggregation.group_by {
            tuple.push(held(plan, &arranged[index], index, key).ok_or_else(|| {
                plan.unwitnessed(format!(
                    "row `{}` holds a value of the group key `{key}` nothing determined",
                    arranged[index].row.label
                ))
            })?);
        }
        match groups.iter_mut().find(|(held, _)| *held == tuple) {
            Some((_, members)) => members.push(index),
            None => groups.push((tuple, vec![index])),
        }
    }
    Ok(groups)
}

fn observe(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<ConformanceScenario, RefusalCause> {
    let ir = plan.ir;
    let view = plan.view;
    let name = ViewRef::new(view.name.clone());
    let mut steps = Vec::new();
    let mut source: BTreeSet<EssSemanticRef> = BTreeSet::new();
    source.insert(name.clone().into());
    source.insert(EntityRef::from(plan.handle).into());
    let mut types = BTreeSet::new();
    for field in &view.fields {
        reachable_types(ir, &field.type_ref, &mut types);
    }
    source.extend(types.into_iter().map(EssSemanticRef::from));
    for row in arranged {
        steps.extend(row.arrangement.steps.iter().cloned());
        source.extend(row.arrangement.source.iter().cloned());
    }

    let groups = groups(plan, arranged)?;

    let mut expectations = Vec::new();
    for (tuple, members) in &groups {
        let keys: BTreeMap<String, ScenarioValue> = plan
            .aggregation
            .group_by
            .iter()
            .zip(tuple)
            .map(|(key, value)| (key.clone(), ScenarioValue::literal(value.clone())))
            .collect();
        let admitted: Vec<usize> = members
            .iter()
            .copied()
            .filter(|index| arranged[*index].admitted)
            .collect();
        if admitted.is_empty() {
            if !plan.aggregation.is_ungrouped() {
                expectations.push(ViewExpectation::Excludes { fields: keys });
            }
            continue;
        }
        if members.contains(&0) {
            rounding_is_observable(plan, arranged, &admitted)?;
        }
        let mut fields = keys;
        for (field, value) in aggregates_over(plan, arranged, &admitted)? {
            fields.insert(field, ScenarioValue::literal(value));
        }
        expectations.push(ViewExpectation::Contains { fields });
    }
    let one = ViewExpectation::Counts {
        at_least: Some(1),
        at_most: Some(1),
    };
    if plan.aggregation.is_ungrouped() {
        if !expectations
            .iter()
            .any(|expectation| matches!(expectation, ViewExpectation::Contains { .. }))
        {
            let mut fields = BTreeMap::new();
            for (field, value) in aggregates_over(plan, arranged, &[])? {
                fields.insert(field, ScenarioValue::literal(value));
            }
            expectations.push(ViewExpectation::Contains { fields });
        }
        expectations.push(one.clone());
    }
    read(view, &name, params, expectations, &mut steps);

    if plan.aggregation.is_ungrouped() {
        // Decision 4: the one row exists when no row passes the filter.
        let mut fields = BTreeMap::new();
        for (field, value) in aggregates_over(plan, arranged, &[])? {
            fields.insert(field, ScenarioValue::literal(value));
        }
        read(
            view,
            &name,
            &plan.params("empty"),
            vec![ViewExpectation::Contains { fields }, one],
            &mut steps,
        );
    }

    Ok(ConformanceScenario::new(
        clipped(&format!(
            "`{}` reports every group's exact aggregates over rows this scenario made, and no \
             empty group",
            view.name
        )),
        steps,
        source,
    ))
}

/// Every aggregate field's expected value over the admitted rows `members`.
/// Refuse a view whose A group would assert an `avg` a truncating implementation also reports.
///
/// The A group is the one whose values the pattern chose, and the one the page's mutant table
/// says catches "`avg` truncates instead of rounding". A mean whose seventh decimal is below 5 —
/// or one that ends, as the mean of a key every row shares does — is right and catches nothing;
/// asserting it would look like a check in every report that reads it. Groups of one row (B, bₖ)
/// hold an integer mean and are asserted as the exact values they are: they are not the check.
fn rounding_is_observable(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    members: &[usize],
) -> Result<(), RefusalCause> {
    for (field, aggregate) in &plan.aggregation.functions {
        let (AggregateFunction::Avg, Some(input)) = (aggregate.function, &aggregate.input) else {
            continue;
        };
        let values: Option<Vec<Node>> = members
            .iter()
            .map(|index| held(plan, &arranged[*index], *index, &input.name))
            .collect();
        let separates = values
            .as_deref()
            .and_then(crate::aggregate::avg_separates_rounding);
        if separates != Some(true) {
            return Err(plan.unwitnessed(format!(
                "`{field}` = {aggregate} over the group the pattern chose has a mean that a \
                 truncating `avg` also reports, so no arrangement here tells rounding from \
                 truncation"
            )));
        }
    }
    Ok(())
}

fn aggregates_over(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    members: &[usize],
) -> Result<Vec<(String, Node)>, RefusalCause> {
    let mut out = Vec::new();
    for (field, aggregate) in &plan.aggregation.functions {
        let (values, kind) = match &aggregate.input {
            None => (vec![Node::Null; members.len()], ValueKind::Other),
            Some(input) => {
                let mut values = Vec::new();
                for index in members {
                    values.push(
                        held(plan, &arranged[*index], *index, &input.name).ok_or_else(|| {
                            plan.unwitnessed(format!(
                                "row `{}` holds a value of `{}` nothing determined",
                                arranged[*index].row.label, input.name
                            ))
                        })?,
                    );
                }
                (values, kind(plan.ir, plan.entity, &input.name))
            }
        };
        let value = evaluate(aggregate.function, &values, kind).ok_or_else(|| {
            plan.unwitnessed(format!(
                "`{field}` has no exact value over the arranged rows"
            ))
        })?;
        out.push((field.clone(), value));
    }
    Ok(out)
}

/// One read of the view with these parameters, holding every expectation, in the block its
/// consistency decides.
fn read(
    view: &ResolvedView,
    name: &ViewRef,
    params: &BTreeMap<String, ScenarioValue>,
    expectations: Vec<ViewExpectation>,
    steps: &mut Vec<ScenarioStep>,
) {
    match view.assertion_style {
        AssertionStyle::Expect => {
            steps.push(ScenarioStep::QueryView {
                view: name.clone(),
                params: params.clone(),
            });
            for expectation in expectations {
                steps.push(ScenarioStep::ExpectView {
                    view: name.clone(),
                    expectation,
                });
            }
        }
        AssertionStyle::Eventually => {
            for expectation in expectations {
                steps.push(ScenarioStep::EventuallyView {
                    view: name.clone(),
                    params: params.clone(),
                    expectation,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_group_size_has_a_factor_other_than_two_and_five() {
        let sizes: Vec<usize> = (0..=7).map(group_size).collect();
        assert_eq!(sizes, [3, 3, 6, 6, 6, 7, 9, 9]);
    }

    #[test]
    fn each_input_has_its_own_duplicate_pattern() {
        let column = |i| (0..6).map(|j| a_ordinal(i, j)).collect::<Vec<_>>();
        assert_eq!(column(0), [1, 1, 3, 7, 13, 21]);
        assert_eq!(column(1), [101, 101, 101, 103, 107, 113]);
        assert_eq!(column(2), [201, 201, 201, 201, 203, 207]);
    }
}
