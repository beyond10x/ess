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
    ResolvedEffect, ResolvedEntity, ResolvedPayloadValue, ResolvedRelatedHop, ResolvedRelatedVia,
    ResolvedTypeRef, ResolvedView,
};
use ess_domain::entity::{EntitySpec, StateName};
use ess_domain::name::QualifiedName;
use ess_domain::types::{Primitive, MAX_TYPE_DEPTH};
use ess_domain::view::{AggregateFunction, AssertionStyle, ViewSpec};
use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use super::{
    advance, arrange_first, arrange_owner, clipped, created_owned, has_subject_guards,
    identity_inputs, insert, literal_value, reach, reachable_types, related, related_guard,
    route_from, shares_owner, shows, subject_fact, Arrangement, CommandRef, Determined, Refusal,
    RefusalCause,
};
use crate::aggregate::{evaluate, evaluate_skipping_absent, ValueKind};
use crate::scenario::{
    ActorRef, ConformanceScenario, ConformanceSuite, EntityRef, EssSemanticRef, ScenarioId,
    ScenarioInitialState, ScenarioStep, ScenarioValue, ViewExpectation, ViewRef,
};
use crate::witness::{uuid_of, Distinction};

mod contrast;

/// The most inputs the page's pattern keeps apart: seven, with a group of nine rows.
const MAX_INPUTS: usize = 7;

/// The step between the blocks of distinctions the rows a related group key reads are arranged at:
/// past every row the pattern numbers, so their instances and witness values are their own.
const RELATED_ROWS: usize = 1_000;

/// Every aggregate view's scenario, or the refusal that says why it has none.
pub(super) fn aggregates(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let literals = model_literals(ir);
    // The suite's own authority, established before this family runs: only a scenario that starts
    // from an empty logical namespace makes an exact count over every row it reads a claim about
    // this scenario alone (`docs/design/aggregate-group-selection.md`). Never inferred from an
    // absent parameter or a version number.
    let isolated = suite.provenance.scenario_initial_state == Some(ScenarioInitialState::Empty);
    for view in ir.views().values() {
        let Some(aggregation) = &view.aggregation else {
            continue;
        };
        let id = ScenarioId::Aggregate {
            view: ViewRef::new(view.name.clone()),
        };
        match scenario(ir, view, aggregation, actors, &literals, isolated) {
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
    Text {
        prefix: String,
    },
    Timestamp,
    Enum(Vec<String>),
    Boolean,
    Uuid {
        prefix: String,
    },
    /// The values of another ladder its type admits, in ladder order (`admitted`): a key whose
    /// newtype bounds, sizes or patterns its values walks only those.
    Listed(Vec<Node>),
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
            Self::Listed(values) => values[ordinal % values.len()].clone(),
        }
    }

    /// How many distinct values the ladder has, where that is finite.
    fn len(&self) -> Option<usize> {
        match self {
            Self::Enum(variants) => Some(variants.len()),
            Self::Listed(values) => Some(values.len()),
            Self::Boolean => Some(2),
            _ => None,
        }
    }
}

/// How many of a ladder's values are checked against a key's type before the ladder is kept as it
/// is, and how many admitted values a listed ladder keeps.
const ADMITTED: usize = 64;

/// How far a ladder is walked for values its type admits.
const ADMITTED_SCAN: usize = 4_096;

/// `ladder`, kept to the values `declared` admits (beyond10x/ess#361): a newtype's invariants —
/// a range, a length, a pattern — bound a walked key as much as its primitive does, and a row
/// holding a value outside them is one a correct target refuses to create. A ladder whose first
/// [`ADMITTED`] values all pass is kept as it is, so every key no invariant bounds walks exactly as
/// before; otherwise it is the admitted values met walking it, in order, and `None` where it meets
/// none.
fn admitted(ir: &EssIr, declared: &ResolvedTypeRef, ladder: Ladder) -> Option<Ladder> {
    let valid = |value: &Node| crate::input::validate_typed_value(ir, declared, value).is_ok();
    let bound = ladder.len().unwrap_or(ADMITTED_SCAN);
    if (0..bound.min(ADMITTED)).all(|ordinal| valid(&ladder.at(ordinal))) {
        return Some(ladder);
    }
    let values: Vec<Node> = (0..bound)
        .map(|ordinal| ladder.at(ordinal))
        .filter(|value| valid(value))
        .take(ADMITTED)
        .collect();
    (!values.is_empty()).then_some(Ladder::Listed(values))
}

/// Why a group tuple holds a value its key's type does not admit, where one does: a scoped or
/// walked value outside a newtype's invariants is a row a correct target refuses, so the view is
/// refused by name rather than arranged (beyond10x/ess#361).
fn inadmissible(plan: &Plan<'_>) -> Option<String> {
    for (label, tuple) in &plan.tuples {
        for ((name, _), value) in plan.keys.iter().zip(tuple) {
            if *value == Node::Null {
                continue;
            }
            let Some(field) = plan.entity.observable_field(name) else {
                continue;
            };
            if let Err(why) = crate::input::validate_typed_value(plan.ir, &field.type_ref, value) {
                return Some(format!(
                    "group `{label}` would hold `{name}` = {}, which its type does not admit \
                     ({why}), and no arrangement here chooses another",
                    serde_json::to_string(value).unwrap_or_default()
                ));
            }
        }
    }
    None
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

/// A field the creating command copies from the row another row references (`{related: …}` in
/// `sets:`, beyond10x/ess#257). The scenario creates that row holding the value it wants, and
/// points the creating command's input at it.
#[derive(Debug, Clone, Copy)]
struct RelatedKey<'ir> {
    /// The creating command's input that names the referenced row.
    via: &'ir str,
    /// The further references a chained read follows from that row (ess/22, beyond10x/ess#285):
    /// empty for a one-hop read.
    through: &'ir [ResolvedRelatedHop],
    /// Whether the creating command may leave `via` out, leaving the reference — and so the value
    /// — absent (ess/22, beyond10x/ess#285).
    optional_via: bool,
    /// The referenced entity: the one the last reference names.
    entity: &'ir EntityHandle,
    /// The referenced row's field the value is copied from.
    field: &'ir str,
}

/// The input a `creates:` branch names another row by: `input.<field>`, or a subject field the
/// branch fills from its input unchanged.
fn via_input<'a>(
    via: &'a ResolvedRelatedVia,
    mapped: &BTreeMap<&'a str, &'a str>,
) -> Option<&'a str> {
    match via {
        ResolvedRelatedVia::Input { field, .. } => Some(field),
        ResolvedRelatedVia::Subject { field, .. } => mapped.get(field.as_str()).copied(),
    }
}

/// The branches that can create a row of `entity` from an input the scenario chooses, in the
/// order [`EssIr::drivers`] yields them.
fn related_creators<'ir>(ir: &'ir EssIr, entity: &EntityHandle) -> Vec<Driver<'ir>> {
    let all = ir.drivers();
    all.get(entity)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .filter(|driver| {
            !has_subject_guards(driver.command)
                && driver.outcome.test_strategy != ess_domain::command::TestStrategy::InjectFault
                && !related_guard::routes(driver.command, driver.outcome)
        })
        .copied()
        .collect()
}

/// The input `creator` fills the entity field `field` from, unchanged.
fn filled_from<'ir>(creator: &Driver<'ir>, field: &str) -> Option<&'ir str> {
    creator
        .outcome
        .sets
        .iter()
        .find_map(|set| match &set.value {
            // A path (ess/22, A4) reads inside a struct input, which no arrangement here writes.
            ResolvedPayloadValue::InputField { field: input, .. }
                if set.target == field
                    && set.conversion.is_none()
                    && !ess_domain::command::input_path::is_path(input) =>
            {
                Some(input.as_str())
            }
            _ => None,
        })
}

/// Whether `creator` may leave `field` absent: it fills it from an `Optional` input.
fn leaves_absent(creator: &Driver<'_>, field: &str) -> bool {
    filled_from(creator, field).is_some_and(|read| {
        creator
            .command
            .input
            .iter()
            .any(|input| input.name == read && input.type_ref.is_optional())
    })
}

/// How the scenario gives the field `{related: {via, field}}` fills a value of its choosing, or
/// why it cannot. Which branch creates the related row is chosen per row, once for every field it
/// must hold ([`arrange_related`]).
#[allow(clippy::too_many_arguments)]
fn related_key<'ir>(
    ir: &'ir EssIr,
    handle: &EntityHandle,
    creator: &Driver<'ir>,
    mapped: &BTreeMap<&'ir str, &'ir str>,
    via: &'ir ResolvedRelatedVia,
    through: &'ir [ResolvedRelatedHop],
    entity: &'ir EntityHandle,
    field: &'ir str,
) -> Result<RelatedKey<'ir>, String> {
    let name = &ir.entity(entity).name;
    let via_input = via_input(via, mapped)
        .ok_or_else(|| format!("`{via}` is not an input the creating command reads unchanged"))?;
    if entity == handle {
        return Err(format!(
            "the row it reads is a `{name}`, which the view would then count"
        ));
    }
    // The row a related guard on the creating command reads is the one arranged here where the key
    // is read through the same input from the same entity: the scenario creates it holding the
    // value it wants, and the guard is decided on it ([`related_guard::drive_on`],
    // beyond10x/ess#272). Any other row of the guard's would be a second one the input names.
    if related_guard::routes(creator.command, creator.outcome)
        && related_guard::reads(creator.command) != Some((via_input, entity))
    {
        return Err(format!(
            "`{}` is chosen by a related row its own arrangement supplies",
            creator.command.name
        ));
    }
    if !related_creators(ir, entity)
        .iter()
        .any(|driver| filled_from(driver, field).is_some())
    {
        return Err(format!(
            "nothing creates a `{name}` that sets `{field}` from its input"
        ));
    }
    // A chained read (ess/22, beyond10x/ess#285) is arranged through a row of each entity it
    // names, each reference stored from its creating branch's input; a related guard on the
    // creating command reads one row, never a chain.
    for hop in through {
        let named = &ir.entity(&hop.entity).name;
        if hop.entity == *handle {
            return Err(format!(
                "a row it reads through is a `{named}`, which the view would then count"
            ));
        }
        if related_guard::routes(creator.command, creator.outcome) {
            return Err(format!(
                "`{}` is chosen by a related row its own arrangement supplies",
                creator.command.name
            ));
        }
        if !related_creators(ir, &hop.entity)
            .iter()
            .any(|driver| filled_from(driver, &hop.field).is_some())
        {
            return Err(format!(
                "nothing creates a `{named}` that sets `{}` from its input",
                hop.field
            ));
        }
    }
    let optional_via = via.type_ref().is_optional()
        && creator
            .command
            .input
            .iter()
            .any(|input| input.name == via_input && input.type_ref.is_optional());
    Ok(RelatedKey {
        via: via_input,
        through,
        optional_via,
        entity,
        field,
    })
}

/// Why no arrangement here chooses the value the creating branch writes into `field`: "does not
/// set" only where its `sets:` writes nothing there. `role` names the field in the sentence, and
/// may end in the comma that closes a clause about it.
fn unchosen(
    creator: &Driver<'_>,
    field: &str,
    unrelated: &BTreeMap<&str, String>,
    role: &str,
) -> String {
    let Some(set) = creator.outcome.sets.iter().find(|set| set.target == field) else {
        return format!(
            "the creating command does not set {}",
            role.trim_end_matches(',')
        );
    };
    if set.conversion.is_some() {
        return format!(
            "the creating command sets {role} through a conversion, whose result no arrangement \
             here chooses"
        );
    }
    let source = match &set.value {
        ResolvedPayloadValue::RelatedField {
            via, field: read, ..
        } => {
            let why = unrelated.get(field).map_or(
                "a value read from another row is chosen only for a group key or a scoping field",
                String::as_str,
            );
            return format!(
                "the creating command copies {role} from `{{related: {{via: {via}, field: \
                 {read}}}}}`, but {why}"
            );
        }
        ResolvedPayloadValue::Literal { .. } => "a literal it cannot read as its type",
        ResolvedPayloadValue::InputField { .. } => "its input",
        ResolvedPayloadValue::InputOrGenerated { .. } => "an input with a fallback",
        ResolvedPayloadValue::Generated => "`{generated: true}`",
        ResolvedPayloadValue::Cleared => "`{cleared: true}`",
        ResolvedPayloadValue::SubjectField { .. } => "a `{subject: …}` source",
        ResolvedPayloadValue::Increment { .. } => "an `{increment: …}` source",
        ResolvedPayloadValue::Struct { .. } => "a nested mapping",
        ResolvedPayloadValue::ResponseField { .. } => "an external response",
        ResolvedPayloadValue::CallerAttribute { .. } => "a `{caller: …}` source",
        ResolvedPayloadValue::ChangedCount => "`{count: changed}`",
    };
    format!(
        "the creating command sets {role} from {source}, whose value no arrangement here chooses"
    )
}

/// One declared parameter bound to a scoped value: the filter reads it as `field == param.name`.
#[derive(Debug, Clone)]
struct Scope {
    param: String,
    field: String,
    kind: Scoped,
}

/// One declared parameter the filter compares with a group key, `key == param.name`
/// (beyond10x/ess#361): each read binds it to an actual arranged group's key, and it never
/// overwrites a row's tuple (`docs/design/aggregate-group-selection.md`).
#[derive(Debug, Clone)]
struct Selector {
    param: String,
    field: String,
    /// The key's position in [`Plan::keys`].
    key: usize,
}

/// The most reads one selecting scenario makes: every arranged selection first, then the valid
/// unmatched ones, cut here in that order.
const MAX_READS: usize = 16;

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
    /// The lifecycle state a measure's condition has this row rest in, where the contrast search
    /// chose one (beyond10x/ess#363, [`contrast::arrange`]).
    state: Option<StateName>,
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
    /// The positions in [`Self::keys`] of the keys that may be absent: each gets a group of its own
    /// where it is (`N<k>`).
    absent_keys: Vec<usize>,
    /// The inputs a skipping aggregate reads, left out of the absent rows.
    skipping: BTreeSet<String>,
    scopes: Vec<Scope>,
    /// The entity fields the creating branch copies from a related row whose value the scenario
    /// chooses, by name.
    related: BTreeMap<&'ir str, RelatedKey<'ir>>,
    /// The input the creating branch's related guard compares with the related row's link to that
    /// row's owner, where it has one (beyond10x/ess#272): see [`GuardLink`].
    guard_link: Option<GuardLink>,
    /// The owners the related rows a key is read from are filed under, where the field read is
    /// that row's link to its owner: one per related entity and value the pattern gives it, in the
    /// order first given ([`related_owners`]).
    related_owners: Vec<(EntityHandle, Node, Arrangement)>,
    /// The group key filled from the input the creating branch's related guard reads its row
    /// through ([`via_key`]).
    via_key: Option<String>,
    /// The owners [`GuardLink`] inputs name, one per slot ([`guard_owners`]).
    guard_owners: Vec<(Node, Arrangement)>,
    /// The group keys read from the owner the view also groups by (its link, beyond10x/ess#193),
    /// with the position of that link in [`Self::keys`]: rows sharing an owner share their values.
    follows_owner: BTreeMap<String, usize>,
    /// The group tuples in assignment order, labelled `A`, `B`, `C`, `B2`, …, `N1`, …
    tuples: Vec<(String, Vec<Node>)>,
    /// Whether the view is ungrouped and nothing scopes it, so it is asserted as the change its
    /// rows make ([`observe_change`]).
    delta: bool,
    /// The parameters that select a group key.
    selectors: Vec<Selector>,
    /// Whether the scenario asserts every group and the number of rows exactly, under the suite's
    /// `Empty` authority ([`observe_exact`]): a view with a group selector, or one nothing scopes.
    /// Every other view keeps the observation it had.
    exact: bool,
}

/// An input of the creating command that a `when_related:` predicate compares with the related
/// row's link to its owner (`study_id != input.study_id`, beyond10x/ess#272).
///
/// The branch creating the rows is selected only where the input names the owner the related row
/// was filed under, and that owner is an instance the arrangement creates, never a value the
/// pattern can choose. So the input always names an arranged owner, and the related row is filed
/// under it:
///
/// * where the related row is one the plan arranges itself (a key copied from it, #257), the
///   input names the owner that row was filed under ([`create_row`]);
/// * where a group key is filled from the input, each value the pattern gives the key stands for
///   one owner, arranged once and shared by every row given that value, and the row holds the
///   owner's identity, which this scenario alone created and so scopes its group as the planned
///   value would have;
/// * otherwise the view does not read the input, and the rows are spread over two owners, the
///   first row (or the rows sharing the first related row) under one and every other under the
///   second, so a target counting one owner's rows alone counts fewer ([`guard_slots`]).
///
/// Outside the first case the related row is arranged under the owner named
/// ([`related_guard::drive`] pins it, beyond10x/ess#271); where the row's own owner link is filled
/// from one of the inputs, the owner it is created under is the one named. Several inputs compared
/// with the same link all name the one owner.
#[derive(Debug, Clone)]
struct GuardLink {
    /// The creating command's inputs the predicates compare, in name order.
    inputs: Vec<String>,
    /// The entity that owns the related row, which the inputs name.
    owner: EntityHandle,
    /// The group key filled from one of the inputs, where there is one.
    key: Option<String>,
    /// Whether the row's own owner link is filled from one of the inputs: the owner it is created
    /// under is then the one they name.
    own: bool,
    /// The fields copied from the related row the guard reads ([`RelatedKey`]): a row given a value
    /// of one has that row arranged by the plan itself ([`arrange_related`]).
    read_by: Vec<String>,
}

/// The distinction the owners [`GuardLink`] inputs name are arranged at, one apart per value: past
/// every block the related rows a key is read from are numbered in.
const GUARD_OWNERS: usize = 100 * RELATED_ROWS;

/// What `row`'s [`GuardLink`] inputs name, and the related row it shares with an earlier row given
/// the same value of the guard's input ([`Plan::via_key`]).
#[derive(Default)]
struct RowGuard {
    owner: Option<super::InstanceName>,
    related: Option<Arrangement>,
}

/// Whether a function changes, when rows are added, by an amount those rows alone decide.
fn additive(function: AggregateFunction) -> bool {
    matches!(function, AggregateFunction::Count | AggregateFunction::Sum)
}

impl Plan<'_> {
    /// Whether only this scenario's rows can land in the group with these key values: the view is
    /// scoped by a parameter, or a scoped key holds one of its values. A tuple whose scoped key is
    /// absent (`N<k>` of a view scoped only by that key) is shared with every row another scenario
    /// creates without the key, so it is asserted to exist and nothing more — except where the
    /// scenario is [`exact`](Self::exact): its rows are all the view holds.
    fn scoped_tuple(&self, tuple: &[Node]) -> bool {
        self.exact
            || !self.scopes.is_empty()
            || self
                .keys
                .iter()
                .zip(tuple)
                .any(|((_, key), value)| matches!(key, Key::Scoped(_)) && *value != Node::Null)
    }

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

    /// `params`, with every group selector bound to the key `reached` actually holds: the
    /// selector conjunct then holds of the row, and the rest of the filter decides it. A key the
    /// row holds as absent is left unbound — no equality selects it.
    fn selecting(
        &self,
        params: &BTreeMap<String, ScenarioValue>,
        reached: &Arrangement,
    ) -> BTreeMap<String, ScenarioValue> {
        let mut bound = params.clone();
        for selector in &self.selectors {
            let value = if selector.field == EntitySpec::STATE {
                Some(ScenarioValue::literal(Node::Text(
                    reached.state.to_string(),
                )))
            } else {
                reached
                    .settled
                    .get(&selector.field)
                    .map(|held| held.value.clone())
            };
            match value {
                Some(value) if value != ScenarioValue::literal(Node::Null) => {
                    bound.insert(selector.param.clone(), value);
                }
                _ => {}
            }
        }
        bound
    }

    /// Whether the filter reads nothing but group selectors and their parameters: bound to a row's
    /// own key it holds of every row, so a refuted row is witnessed by a distinct group and is
    /// never forced through a state.
    fn selectors_only(&self) -> bool {
        !self.selectors.is_empty()
            && self.view.filter.as_ref().is_some_and(|filter| {
                filter.fact_paths().into_iter().all(|path| {
                    self.selectors.iter().any(|selector| match path.segments() {
                        [field] => *field == selector.field,
                        [namespace, name] => {
                            namespace == ViewSpec::PARAM && *name == selector.param
                        }
                        _ => false,
                    })
                })
            })
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
                ResolvedCondition::Related { test, input, .. } => {
                    if let ess_compiler::ir::ResolvedRelatedTest::Holds { predicate } = test {
                        texts(predicate, &mut out);
                    }
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
                | ResolvedCondition::WrongState
                | ResolvedCondition::UnknownInstance
                | ResolvedCondition::InputAbsent
                | ResolvedCondition::ExistingInstance => {}
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
        ..
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
/// and 5, so a mean over it can repeat forever at the sixth decimal — and `m + k` too for every
/// `k` in `beyond`: an input averaged beside skipping inputs is read over A's `m` pattern rows and
/// every absent row that holds it (one per skipping input, each lacking only its own).
fn group_size(inputs: usize, beyond: &[usize]) -> usize {
    let repeats = |size: usize| {
        let mut rest = size;
        for factor in [2, 5] {
            while rest % factor == 0 {
                rest /= factor;
            }
        }
        rest > 1
    };
    let mut size = (inputs + 2).max(3);
    while !(repeats(size) && beyond.iter().all(|k| repeats(size + k))) {
        size += 1;
    }
    size
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
    isolated: bool,
) -> Result<ConformanceScenario, RefusalCause> {
    let handle = &view.source;
    let entity = ir.entity(handle);
    let all = ir.drivers();
    let drivers: &[Driver<'_>] = all.get(handle).map_or(&[], Vec::as_slice);
    // A creation into `initial` first: every declared state is reachable from there, where one
    // `into:` (ess/15) a later state may never lead back to an earlier one.
    let creator = drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .min_by_key(|driver| {
            driver
                .outcome
                .subject
                .as_ref()
                .is_some_and(|subject| subject.into.is_some())
        })
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
            ResolvedPayloadValue::InputField { field, .. }
                if !ess_domain::command::input_path::is_path(field) =>
            {
                Some((set.target.as_str(), field.as_str()))
            }
            _ => None,
        })
        .collect();
    // The entity fields it copies from a related row, and for each either how the scenario
    // chooses its value there or why it cannot (beyond10x/ess#257).
    let mut related: BTreeMap<&str, RelatedKey<'_>> = BTreeMap::new();
    let mut unrelated: BTreeMap<&str, String> = BTreeMap::new();
    for set in creator
        .outcome
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
    {
        if let ResolvedPayloadValue::RelatedField {
            via,
            through,
            entity: other,
            field,
            ..
        } = &set.value
        {
            match related_key(ir, handle, creator, &mapped, via, through, other, field) {
                Ok(key) => {
                    related.insert(set.target.as_str(), key);
                }
                Err(reason) => {
                    unrelated.insert(set.target.as_str(), reason);
                }
            }
        }
    }
    let chosen_by_scenario = |name: &str| mapped.contains_key(name) || related.contains_key(name);
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
    // A group key is scoped by its present values whether or not it may be absent; its absent
    // value is a group of its own, scoped only by another key or a parameter (see `Plan::scoped_tuple`).
    let scopable_as = |name: &str, optional_admitted: bool| -> Option<Scoped> {
        if name == entity.identity.name || name == EntitySpec::STATE || !chosen_by_scenario(name) {
            return None;
        }
        match field_type(name)? {
            (optional, _) if optional && !optional_admitted => None,
            (_, Leaf::Primitive(Primitive::String)) => Some(Scoped::Text),
            (_, Leaf::Primitive(Primitive::Uuid)) => Some(Scoped::Uuid),
            _ => None,
        }
    };
    let scopable = |name: &str| scopable_as(name, false);
    // Whether a created row can lack this entity field: the creating branch fills it from an
    // `Optional` input, which an invocation may leave out, and the field is itself `Optional`, so
    // the row then holds it as absent (`synthesize.rs`, `settled`).
    // A field copied from a related row is absent where that row's field is: some branch creating
    // the row fills it from an `Optional` input, and the field is itself `Optional`.
    // From ess/22 (beyond10x/ess#285) it is absent too where a reference it is read through is:
    // the creating command leaves its Optional `via` input out, or a chained read's Optional next
    // reference is left absent by a branch creating the row that holds it.
    let related_absent = |name: &str| {
        related.get(name).is_some_and(|key| {
            (ir.entity(key.entity)
                .observable_field(key.field)
                .is_some_and(|field| field.type_ref.is_optional())
                && related_creators(ir, key.entity)
                    .iter()
                    .any(|driver| leaves_absent(driver, key.field)))
                || key.optional_via
                || key.through.iter().any(|hop| {
                    hop.type_ref.is_optional()
                        && related_creators(ir, &hop.entity)
                            .iter()
                            .any(|driver| leaves_absent(driver, &hop.field))
                })
        })
    };
    let absent_able = |name: &str| {
        (mapped.get(name).is_some_and(|read| {
            creator
                .command
                .input
                .iter()
                .any(|input| input.name == *read && input.type_ref.is_optional())
        }) || related_absent(name))
            && entity
                .observable_field(name)
                .is_some_and(|field| field.type_ref.is_optional())
    };
    let cannot_lack = |name: &str, role: &str| {
        unwitnessed(
            view,
            format!(
                "no row `{}/{}` creates can leave `{name}` absent, which {role}, so what an absent \
                 value does there cannot be witnessed",
                creator.command.name, creator.outcome.name
            ),
        )
    };

    // Scoping by parameter: every declared parameter is read by exactly one top-level
    // `field == param.name` conjunct over a scopable field that is not a group key — or, under the
    // suite's `Empty` authority, over a group key it then selects (beyond10x/ess#361).
    let mut scopes = Vec::new();
    let mut selectors: Vec<Selector> = Vec::new();
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
            Some((field, _)) if reads == 1 && isolated => {
                if let Some(other) = selectors.iter().find(|selector| selector.field == field) {
                    return Err(unwitnessed(
                        view,
                        format!(
                            "the parameters `{}` and `{}` both select the group key `{field}`",
                            other.param, param.name
                        ),
                    ));
                }
                let key = aggregation
                    .group_by
                    .iter()
                    .position(|key| *key == field)
                    .unwrap_or_else(|| unreachable!("`{field}` is a group key"));
                selectors.push(Selector {
                    param: param.name.clone(),
                    field,
                    key,
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
    let mut absent_keys = Vec::new();
    for (index, key) in aggregation.group_by.iter().enumerate() {
        if field_type(key).is_some_and(|(optional, _)| optional) {
            if !absent_able(key) {
                return Err(cannot_lack(key, "the view groups by"));
            }
            absent_keys.push(index);
        }
        let chosen = if let Some(kind) = scopable_as(key, true) {
            Key::Scoped(kind)
        } else if key == EntitySpec::STATE {
            Key::State(entity.lifecycle.states.iter().cloned().collect())
        } else if key == &entity.identity.name {
            Key::Fixed(Node::Null)
        } else if chosen_by_scenario(key) {
            let (_, found) = field_type(key).unwrap_or((false, Leaf::Other));
            let declared = entity.observable_field(key).map(|field| field.type_ref);
            match Ladder::of(&view.name, key, &found)
                .zip(declared)
                .map(|(ladder, declared)| admitted(ir, &declared, ladder))
            {
                Some(Some(ladder)) => Key::Walked(ladder),
                Some(None) => {
                    return Err(unwitnessed(
                        view,
                        format!(
                            "no value the ladder walks for the group key `{key}` is one its type \
                             admits"
                        ),
                    ))
                }
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
                unchosen(creator, key, &unrelated, &format!("the group key `{key}`")),
            ));
        };
        keys.push((key.clone(), chosen));
    }
    let guard_link = guard_link(
        ir,
        handle,
        creator,
        &mapped,
        &aggregation.group_by,
        &related,
    )
    .map_err(|reason| unwitnessed(view, reason))?;
    // An ungrouped view with no parameter is over every row of its source, so only the change its
    // own rows make is the scenario's to assert — and only a `count` or a `sum` changes by an
    // amount those rows alone decide (`docs/design/aggregate-views.md`, "Scoping").
    let delta = aggregation.is_ungrouped()
        && scopes.is_empty()
        && aggregation
            .functions
            .values()
            .any(|aggregate| additive(aggregate.function));
    // Under the suite's `Empty` authority a scenario's rows are all the view holds, so a view a
    // parameter selects a group of, and every view nothing scopes — grouped by the lifecycle
    // state, an enum or a `Boolean` alone, or ungrouped with no `count` or `sum` — is asserted
    // exactly (beyond10x/ess#361, beyond10x/ess#362). Every other view keeps the observation it
    // had, the change of an ungrouped `count` or `sum` included.
    let unscoped =
        !keys.iter().any(|(_, key)| matches!(key, Key::Scoped(_))) && scopes.is_empty() && !delta;
    let exact = isolated && (!selectors.is_empty() || unscoped);
    if unscoped && !exact {
        return Err(RefusalCause::AggregateUnscoped {
            view: ViewRef::new(view.name.clone()),
        });
    }
    if exact {
        if let Some(reason) = unsettled(ir, handle) {
            return Err(unwitnessed(view, reason));
        }
        // Every row the creating command names must exist (`arrange_related`).
        if let Some((field, reason)) = unrelated.iter().next() {
            return Err(unwitnessed(
                view,
                format!(
                    "the creating command copies `{field}` from a related row no arrangement here \
                     supplies ({reason}), so the rows it creates would name a row that does not \
                     exist"
                ),
            ));
        }
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
                unchosen(
                    creator,
                    name,
                    &unrelated,
                    &format!("`{name}`, which `{}` aggregates,", field.name),
                ),
            ));
        }
    }
    // The inputs a skipping aggregate reads (`docs/design/aggregate-views.md`, "Absent values"):
    // each is left out of the absent rows. One no created row can lack is refused, rather than
    // asserted over rows that all hold it — which would read as a check of skipping and be none.
    let mut skipping: BTreeSet<String> = BTreeSet::new();
    for (field, aggregate) in &aggregation.functions {
        let (true, Some(input)) = (aggregate.skip_absent, &aggregate.input) else {
            continue;
        };
        if aggregation.group_by.contains(&input.name) {
            continue;
        }
        if !absent_able(&input.name) {
            return Err(cannot_lack(&input.name, &format!("`{field}` skips")));
        }
        skipping.insert(input.name.clone());
    }
    // Skipping inputs are numbered first. With two or more, each gains one distinct value outside
    // its pattern in the absent rows, so its distinct count is `m − i` against a required input's
    // `m − 1 − i`: numbering them first keeps every input's distinct count different from every
    // other's (`m … m − s + 1`, then `m − 1 − s …`).
    inputs.sort_by_key(|(name, _)| !skipping.contains(name));
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
    // With `s` skipping inputs A gains `s` absent rows, each lacking only its own input and
    // holding every other at its lowest A value. A required input's mean is then over `m + s` rows
    // and a skipping input's over `m + s − 1`, and each such count must separate rounding too.
    let absent_rows = inputs
        .iter()
        .filter(|(name, _)| skipping.contains(name))
        .count();
    let beyond: Vec<usize> = aggregation
        .functions
        .values()
        .filter(|aggregate| aggregate.function == AggregateFunction::Avg && absent_rows > 0)
        .filter_map(|aggregate| aggregate.input.as_ref())
        .filter(|input| inputs.iter().any(|(name, _)| *name == input.name))
        .map(|input| {
            if skipping.contains(&input.name) {
                absent_rows - 1
            } else {
                absent_rows
            }
        })
        .collect();
    // A conditioned `avg` (beyond10x/ess#363) is over a proper subset of A whose mean must tell
    // rounding from truncation, which three of at least six rows can and three of three cannot:
    // the group is sized as for two inputs at least.
    let conditioned_mean = aggregation.functions.values().any(|aggregate| {
        aggregate.function == AggregateFunction::Avg && aggregate.r#where.is_some()
    });
    let m = group_size(
        if conditioned_mean {
            inputs.len().max(2)
        } else {
            inputs.len()
        },
        &beyond,
    );
    // The pattern keeps every A value below `100·i + 85` up to nine rows (`t ≤ 7`); a group the
    // absent rows' counts made larger would let an A value reach the values b, x, c and bₖ hold.
    if m > group_size(MAX_INPUTS, &[]) {
        return Err(unwitnessed(
            view,
            format!(
                "{} inputs, averaged beside {absent_rows} skipping ones, need a group of {m} \
                 rows, more than the pattern keeps apart",
                inputs.len()
            ),
        ));
    }

    // Rows the view groups by a shared owner are created under one owner per link value
    // (`arrange_and_observe`); a key read through the input that names that owner is its value.
    let mut follows_owner = BTreeMap::new();
    if let Some(owned) = ir.owner_of(handle).filter(|_| shares_owner(ir, handle)) {
        let linked = mapped.get(owned.via).copied();
        if let Some(link) = keys.iter().position(|(name, _)| name == owned.via) {
            for (name, key) in &related {
                if Some(key.via) == linked {
                    follows_owner.insert((*name).to_owned(), link);
                }
            }
        }
    }
    // An input the guard compares with the related row's owner link names an owner, so a field
    // filled from it holds no value the pattern chose: not a scope's, nor an aggregate input's.
    if let Some(link) = &guard_link {
        let chosen = |field: &str| {
            scopes.iter().any(|scope| scope.field == field)
                || inputs.iter().any(|(name, _)| name == field)
        };
        if let Some((field, read)) = mapped
            .iter()
            .find(|(field, read)| link.inputs.iter().any(|held| held == *read) && chosen(field))
        {
            return Err(unwitnessed(
                view,
                format!(
                    "`{field}` is filled from `input.{read}`, which `{}/{}` is selected on only \
                     where it names the `{}` its related row is filed under, so no value the \
                     pattern chooses for `{field}` reaches it",
                    creator.command.name,
                    creator.outcome.name,
                    ir.entity(&link.owner).name
                ),
            ));
        }
    }
    let mut plan = Plan {
        ir,
        view,
        aggregation,
        handle,
        entity,
        literals,
        keys,
        absent_keys,
        skipping,
        scopes,
        related,
        guard_link,
        related_owners: Vec::new(),
        via_key: via_key(creator, &mapped, &aggregation.group_by),
        guard_owners: Vec::new(),
        follows_owner,
        tuples: Vec::new(),
        delta,
        selectors,
        exact,
    };
    assign_tuples(&mut plan, &mapped);
    if let Some(reason) = inadmissible(&plan) {
        return Err(unwitnessed(view, reason));
    }
    let mut rows = rows(&plan, &inputs, m);
    // A measure that reads only the rows its condition admits (beyond10x/ess#363) needs a group
    // holding rows on both sides of it, arranged so that its value decides the condition. Rows
    // something outside the arrangement changes could move between its sides unseen.
    if !contrast::conditioned(&plan).is_empty() {
        if let Some(reason) = unsettled(ir, handle) {
            return Err(unwitnessed(view, reason));
        }
        let keyed: BTreeSet<&str> = aggregation
            .group_by
            .iter()
            .map(String::as_str)
            .chain(plan.scopes.iter().map(|scope| scope.field.as_str()))
            .collect();
        let measured: BTreeSet<&str> = inputs.iter().map(|(name, _)| name.as_str()).collect();
        let dimensions = contrast::dimensions(&plan, &mapped, &keyed, &measured);
        contrast::arrange(&plan, &mut rows, &dimensions, &plan.params("in"))?;
    }
    plan.related_owners = related_owners(&plan, &rows, actors)?;
    plan.guard_owners = guard_owners(&plan, &rows, actors)?;
    arrange_and_observe(&plan, creator, &mapped, rows, actors)
}

/// Group tuples in the order A, B, C, B₂, …, each differing from every tuple before it.
fn assign_tuples(plan: &mut Plan<'_>, mapped: &BTreeMap<&str, &str>) {
    if plan.aggregation.is_ungrouped() {
        plan.tuples.push(("A".to_owned(), Vec::new()));
        return;
    }
    // Two keys filled by the same creating input cannot vary independently (#309). Keep the
    // first key as their representative, including its ladder and its absent-value witness.
    let representatives: Vec<usize> = plan
        .keys
        .iter()
        .enumerate()
        .map(|(index, (name, _))| {
            mapped.get(name.as_str()).map_or(index, |input| {
                plan.keys[..index]
                    .iter()
                    .position(|(other, _)| mapped.get(other.as_str()) == Some(input))
                    .unwrap_or(index)
            })
        })
        .collect();
    let set = |tuple: &mut [Node], index: usize, value: Node| {
        for (position, representative) in representatives.iter().enumerate() {
            if *representative == representatives[index] {
                tuple[position] = value.clone();
            }
        }
    };
    let start = |plan: &Plan<'_>, label: &str, n: usize| -> Vec<Node> {
        representatives
            .iter()
            .map(|index| match &plan.keys[*index].1 {
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
    // C already differ in — unless every other key is scoped as well: then A, B and C differ in
    // all of them at once and no value of a later key repeats, so a target grouping by the later
    // keys alone forms the same groups. B₁ repeats B's later keys under another first key
    // (beyond10x/ess#193, `group_by: [account_id, memo]`).
    let all_scoped = representatives.iter().any(|index| *index != 0)
        && plan
            .keys
            .iter()
            .all(|(_, key)| matches!(key, Key::Scoped(_)));
    for (index, representative) in representatives.iter().enumerate() {
        if *representative != index {
            continue;
        }
        let label = format!("B{}", index + 1);
        // A key read from the owner the link key names holds one value per owner, so no tuple
        // keeps B's owner under another value of it. Bₖ keeps B's value under an owner of its
        // own instead: two owners then share the value, and a target that groups by it without
        // the link merges them.
        if let Some(link) = plan.follows_owner.get(&plan.keys[index].0).copied() {
            if let Key::Scoped(kind) = plan.keys[link].1 {
                let mut tuple = b.clone();
                set(&mut tuple, link, plan.scoped(kind, &label));
                if !plan.tuples.iter().any(|(_, held)| *held == tuple) {
                    plan.tuples.push((label, tuple));
                }
            }
            continue;
        }
        let candidates: Vec<Node> = match &plan.keys[index].1 {
            Key::Scoped(_) if index == 0 && !all_scoped => continue,
            Key::Scoped(kind) => vec![plan.scoped(*kind, &label)],
            key => {
                let bound = key.len().unwrap_or(plan.tuples.len() + 1);
                (0..bound).filter_map(|n| key.sequence(n)).collect()
            }
        };
        let found = candidates.into_iter().find_map(|value| {
            let mut tuple = b.clone();
            set(&mut tuple, index, value);
            (!plan.tuples.iter().any(|(_, held)| *held == tuple)).then_some(tuple)
        });
        if let Some(tuple) = found {
            plan.tuples.push((label, tuple));
        }
    }
    // One `N<k>` per key that may be absent: B's tuple with that key absent, a group no other tuple
    // is, because no other tuple holds an absent value.
    // A key read from the owner the link key names is absent under an owner of its own.
    for index in plan.absent_keys.clone() {
        if representatives[index] != index {
            continue;
        }
        // No equality selects an absent key, so a group a selector's key leaves absent is one no
        // read could ask for (`docs/design/aggregate-group-selection.md`).
        if plan
            .selectors
            .iter()
            .any(|selector| representatives[selector.key] == index)
        {
            continue;
        }
        let label = format!("N{}", index + 1);
        let mut tuple = b.clone();
        set(&mut tuple, index, Node::Null);
        if let Some(link) = plan.follows_owner.get(&plan.keys[index].0).copied() {
            let Key::Scoped(kind) = plan.keys[link].1 else {
                continue;
            };
            set(&mut tuple, link, plan.scoped(kind, &label));
        }
        plan.tuples.push((label, tuple));
    }
}

/// The rows, in creation order: a₁ … aₘ, one absent row aₘ₊₁, … per skipping input, b,
/// x, c, then one bₖ per further tuple and one nₖ per key that may be absent.
///
/// An ordinal of `None` is an absent value: the row's creating input leaves it out.
/// The absent rows of A, as ladder ordinals per input: one per skipping input, lacking that input
/// only.
///
/// SQL skips column by column, so a target that drops a row lacking *any* skipping input must
/// report another value: one row lacking them all would not tell it apart (correction round 1),
/// and neither would a row holding the other skipping inputs at a value that moves no extreme
/// (round 2). So every other skipping input holds a value outside its pattern — below the column's
/// minimum (ordinal `100·i`) where the view takes its `min`, above its maximum (`100·i + 80`; the
/// pattern with its δ stays at or below `100·i + 72`) otherwise — which moves that extreme, its sum
/// and mean, and adds exactly one distinct value. A required input holds its lowest A value, so its
/// distinct count does not move.
fn absent_rows(plan: &Plan<'_>, inputs: &[(String, Ladder)]) -> Vec<Vec<Option<usize>>> {
    let skips = |i: usize| plan.skipping.contains(&inputs[i].0);
    let has_min = |i: usize| {
        plan.aggregation.functions.values().any(|aggregate| {
            aggregate.function == AggregateFunction::Min
                && aggregate
                    .input
                    .as_ref()
                    .is_some_and(|input| input.name == inputs[i].0)
        })
    };
    let outside = |i: usize| {
        if has_min(i) {
            100 * i
        } else {
            100 * i + 80
        }
    };
    (0..inputs.len())
        .filter(|s| skips(*s))
        .map(|s| {
            (0..inputs.len())
                .map(|i| match (i == s, skips(i)) {
                    (true, _) => None,
                    (false, true) => Some(outside(i)),
                    (false, false) => Some(a_ordinal(i, 0)),
                })
                .collect()
        })
        .collect()
}

fn rows(plan: &Plan<'_>, inputs: &[(String, Ladder)], m: usize) -> Vec<Row> {
    let row = |tuple: usize, admitted: bool, label: String, ordinals: Vec<Option<usize>>| {
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
            values.insert(name.clone(), ordinal.map_or(Node::Null, |at| ladder.at(at)));
        }
        Row {
            tuple,
            admitted,
            values,
            label,
            state: None,
        }
    };
    let skips = |i: usize| plan.skipping.contains(&inputs[i].0);
    let ladder = |offset: usize| {
        (0..inputs.len())
            .map(|i| Some(100 * i + offset))
            .collect::<Vec<_>>()
    };
    let absent_rows = absent_rows(plan, inputs);

    let mut a: Vec<Vec<usize>> = (0..m)
        .map(|j| (0..inputs.len()).map(|i| a_ordinal(i, j)).collect())
        .collect();
    // The A mean of every `avg` input must separate rounding from truncation: its seventh decimal
    // is 5 or more (with no tie), so `avg` rounded and `avg` truncated are different numbers. The
    // last A value is raised by the smallest δ below the count the mean divides by that makes it so
    // — it stays the largest value in its column, and below `100·i + 85`
    // (`docs/design/aggregate-views.md`, "Non-terminating mean"). Each mean is over the `m`
    // pattern rows and every absent row that holds its input. Where no δ does, the check in
    // `observe` refuses the view.
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
        let mut sum: usize = a.iter().map(|ordinals| ordinals[i]).sum();
        let mut count = m;
        for extra in absent_rows.iter().filter_map(|ordinals| ordinals[i]) {
            sum += extra;
            count += 1;
        }
        if let Some(delta) = (0..count).find(|delta| separates(sum + delta, count)) {
            a[m - 1][i] += delta;
        }
    }

    let mut out: Vec<Row> = a
        .into_iter()
        .enumerate()
        .map(|(j, ordinals)| {
            row(
                0,
                true,
                format!("a{}", j + 1),
                ordinals.into_iter().map(Some).collect(),
            )
        })
        .collect();
    for (k, ordinals) in absent_rows.into_iter().enumerate() {
        out.push(row(0, true, format!("a{}", m + 1 + k), ordinals));
    }
    let filtered = plan.view.filter.is_some();
    if !plan.aggregation.is_ungrouped() {
        // B holds no value of a skipping input: its `sum`, `avg`, `min` and `max` are absent.
        let b = (0..inputs.len())
            .map(|i| (!skips(i)).then_some(85 + 100 * i))
            .collect();
        out.push(row(1, true, "b".to_owned(), b));
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
            let label = &plan.tuples[tuple].0;
            let (row_label, offset) = match label.strip_prefix('N') {
                Some(rest) => (format!("n{rest}"), 89),
                None => (format!("b{}", &label[1..]), 87),
            };
            out.push(row(tuple, true, row_label, ladder(offset)));
        }
    }
    out
}

/// The input the rows start from, before the pattern sets their values.
///
/// A creating command guarded by a related row (`when_related:` on a sibling branch,
/// beyond10x/ess#272) is reached by no input alone: [`reach`] refuses it, because the row decides
/// the branch. Each row's related row is arranged where the row is created ([`create_row`], through
/// [`related_guard::drive`]), and the input it is sent with is checked there against that row; here
/// it is only the plain witness the pattern's values are written over.
fn base_input(
    ir: &EssIr,
    creator: &Driver<'_>,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    if related_guard::routes(creator.command, creator.outcome) {
        related_guard::plain_input(ir, creator.command, distinction)
    } else {
        reach(ir, creator.command, creator.outcome, distinction)
    }
}

/// The inputs `creator`'s related guard compares with the related row's link to its owner
/// ([`GuardLink`]): `None` for every creator without a related guard, and for one whose guard
/// compares no input with that link. Refused, with the reason, where two group keys are filled
/// from them: the pattern gives two keys values of their own, and the inputs must name one owner.
fn guard_link(
    ir: &EssIr,
    handle: &EntityHandle,
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    group_by: &[String],
    related: &BTreeMap<&str, RelatedKey<'_>>,
) -> Result<Option<GuardLink>, String> {
    if !related_guard::routes(creator.command, creator.outcome) {
        return Ok(None);
    }
    let Some((via, read)) = related_guard::reads(creator.command) else {
        return Ok(None);
    };
    let Some(belongs) = ir.owner_of(read) else {
        return Ok(None);
    };
    let inputs: Vec<String> = subject_fact::links(ir, creator.command, read)
        .into_iter()
        .map(|(sent, _)| sent)
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect();
    if inputs.is_empty() {
        return Ok(None);
    }
    let keys: Vec<&String> = group_by
        .iter()
        .filter(|key| {
            mapped
                .get(key.as_str())
                .is_some_and(|input| inputs.iter().any(|held| held == input))
        })
        .collect();
    if let [first, second, ..] = keys.as_slice() {
        return Err(format!(
            "the group keys `{first}` and `{second}` are filled from inputs `{}` compares with \
             the link of the `{}` it reads to its owner: the pattern gives each key values of its \
             own, and every such input must name the one owner that row is filed under",
            creator.command.name,
            ir.entity(read).name
        ));
    }
    let own = !owner_is_guarded(ir, creator)
        && ir
            .owner_of(handle)
            .and_then(|owned| mapped.get(owned.via))
            .is_some_and(|input| inputs.iter().any(|held| held == input));
    Ok(Some(GuardLink {
        read_by: related
            .iter()
            .filter(|(_, key)| key.via == via)
            .map(|(field, _)| (*field).to_owned())
            .collect(),
        inputs,
        owner: belongs.owner.clone(),
        key: keys.first().map(|key| (*key).clone()),
        own,
    }))
}

/// The group key filled, unchanged, from the input `creator`'s related guard reads the row
/// through, where there is one (beyond10x/ess#272): rows given one value of it share one related
/// row, so a group of several rows reads one row, as a group sharing an owner does.
fn via_key(
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    group_by: &[String],
) -> Option<String> {
    if !related_guard::routes(creator.command, creator.outcome) {
        return None;
    }
    let (via, _) = related_guard::reads(creator.command)?;
    group_by
        .iter()
        .find(|key| mapped.get(key.as_str()) == Some(&via))
        .cloned()
}

/// Which owner each row's [`GuardLink`] inputs name, by row: a key's value where one is filled
/// from them, else one of two slots — the first row, or the rows sharing the first related row
/// ([`via_key`]), in the first and every other row in the second. `None` where the plan has no
/// link, where the row's own owner is the one named, and where the row reads a related row the
/// plan arranges itself. A key left absent is refused: the inputs must name an owner.
fn guard_slots(plan: &Plan<'_>, rows: &[Row]) -> Result<Vec<Option<Node>>, RefusalCause> {
    let Some(link) = plan.guard_link.as_ref().filter(|link| !link.own) else {
        return Ok(vec![None; rows.len()]);
    };
    let owner_name = &plan.ir.entity(&link.owner).name;
    let first_via = plan
        .via_key
        .as_ref()
        .and_then(|key| rows.first().and_then(|row| row.values.get(key)));
    let mut out = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if link
            .read_by
            .iter()
            .any(|field| row.values.contains_key(field))
        {
            out.push(None);
            continue;
        }
        let slot = if let Some(key) = &link.key {
            match row.values.get(key) {
                Some(value) if *value != Node::Null => value.clone(),
                _ => {
                    return Err(plan.unwitnessed(format!(
                        "row `{}` leaves the group key `{key}` absent, but the creating branch is \
                         selected only where the input filling it names the `{owner_name}` its \
                         related row is filed under",
                        row.label
                    )))
                }
            }
        } else {
            let first = match (&plan.via_key, first_via) {
                (Some(key), Some(value)) => row.values.get(key) == Some(value),
                _ => index == 0,
            };
            Node::Text(if first { "first" } else { "second" }.to_owned())
        };
        out.push(Some(slot));
    }
    Ok(out)
}

/// One owner per slot [`guard_slots`] gives a row, arranged where its lifecycle starts, in the
/// order first given. They are created before any row ([`observe`]).
fn guard_owners(
    plan: &Plan<'_>,
    rows: &[Row],
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Vec<(Node, Arrangement)>, RefusalCause> {
    let mut out: Vec<(Node, Arrangement)> = Vec::new();
    let Some(link) = &plan.guard_link else {
        return Ok(out);
    };
    let ir = plan.ir;
    let initial = ir.entity(&link.owner).lifecycle.initial.clone();
    for (slot, row) in guard_slots(plan, rows)?.into_iter().zip(rows) {
        let Some(slot) = slot else { continue };
        if out.iter().any(|(held, _)| *held == slot) {
            continue;
        }
        let owner = super::arrange_first(
            ir,
            &link.owner,
            std::slice::from_ref(&initial),
            actors,
            Distinction::further(GUARD_OWNERS + out.len()),
            &[plan.handle],
        )
        .map_err(|cause| {
            plan.unwitnessed(format!(
                "no `{}` can be arranged for `input.{}` of row `{}` to name: {cause}",
                ir.entity(&link.owner).name,
                link.inputs.join("`, `input."),
                row.label
            ))
        })?;
        out.push((slot, owner));
    }
    Ok(out)
}

/// Whether the row a related guard on `creator` reads is also the owner a row is created under
/// ([`related_guard::owner_is_related`]): it is then arranged once, as the related row, and not
/// first as an owner, which would bind the input the guard is pointed through.
fn owner_is_guarded(ir: &EssIr, creator: &Driver<'_>) -> bool {
    related_guard::routes(creator.command, creator.outcome)
        && related_guard::owner_is_related(ir, creator.command, creator.outcome)
}

/// One arranged row: what the steps left, and what the filter says of it.
struct Arranged {
    row: Row,
    arrangement: Arrangement,
    admitted: bool,
    /// The steps that create the related rows the row reads, run before any row is created
    /// ([`observe`]).
    prelude: Vec<ScenarioStep>,
}

fn arrange_and_observe(
    plan: &Plan<'_>,
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    rows: Vec<Row>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let ir = plan.ir;
    let base = base_input(ir, creator, Distinction::PLAIN).map_err(|_| {
        plan.unwitnessed(format!(
            "no input reaches `{}/{}`, which creates the rows",
            creator.command.name, creator.outcome.name
        ))
    })?;
    let params = plan.params("in");
    // A link to the entity's owner that the view groups by or aggregates is realised as owners
    // (beyond10x/ess#193): the first row the pattern gives a value arranges an owner, and every
    // other row given the same value is created under the same instance, so the pattern's groups
    // and duplicate counts hold of the owners. The value the pattern chose is never sent; the
    // owner's identity is.
    // Only where an owner may hold many rows: under `cardinality: one` every row has an owner of
    // its own, and a pattern value that repeats names no shared owner.
    let link = ir
        .owner_of(plan.handle)
        .filter(|_| shares_owner(ir, plan.handle))
        .map(|owned| owned.via.to_owned())
        .filter(|via| {
            plan.aggregation.group_by.contains(via)
                || plan.aggregation.functions.values().any(|aggregate| {
                    aggregate
                        .input
                        .as_ref()
                        .is_some_and(|input| input.name == *via)
                })
        });
    let mut owners: Vec<(Node, (String, Arrangement))> = Vec::new();
    // A refuted row is not in the view, so where a key is read from its owner it is created under
    // an owner of its own: the values that refute it need not be the shared owner's, and its owner
    // is never shared with a later row.
    let reads_through = |field: &str| plan.related.values().any(|key| key.via == field);
    let owner_for = |row: &Row, distinction, owners: &[(Node, (String, Arrangement))]| {
        let shared = link
            .as_ref()
            .and_then(|field| row.values.get(field))
            .and_then(|key| owners.iter().find(|(held, _)| held == key))
            .filter(|(_, (field, _))| row.admitted || !reads_through(field));
        match shared {
            Some((_, (field, owner))) => Some(Owner {
                row: (
                    field.clone(),
                    Arrangement {
                        steps: Vec::new(),
                        ..owner.clone()
                    },
                ),
                shared: true,
            }),
            None => arrange_owner(ir, creator.outcome, plan.handle, actors, distinction, &[])
                .map(|row| Owner { row, shared: false }),
        }
    };
    let slots = guard_slots(plan, &rows)?;
    let mut shared: Shared = Vec::new();
    let mut arranged = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        let distinction = Distinction::further(index + 1);
        let mut attempt = row.clone();
        let owner =
            owner_for(&attempt, distinction, &owners).filter(|_| !owner_is_guarded(ir, creator));
        let slot = slots[index].as_ref();
        let guard = row_guard(plan, &attempt, owner.as_ref(), slot, &shared);
        let arrange = |attempt: &Row| {
            arrange_row(
                plan,
                creator,
                mapped,
                &base,
                attempt,
                distinction,
                (owner.as_ref(), &guard),
                actors,
                &params,
            )
        };
        let result = match arrange(&attempt) {
            Ok(done) => done,
            // A refuted row of a parameter-scoped filter that no state refutes is moved out of the
            // scope the query binds.
            Err(_) if !attempt.admitted && !plan.scopes.is_empty() => {
                for scope in &plan.scopes {
                    attempt
                        .values
                        .insert(scope.field.clone(), plan.scoped(scope.kind, "out"));
                }
                arrange(&attempt)?
            }
            // A filter that reads nothing but group selectors holds of every row under its own
            // key: the selection is refuted by the distinct groups, and no state is forced.
            Err(_) if !attempt.admitted && plan.selectors_only() => continue,
            Err(error) => return Err(error),
        };
        if let (Some(key), Some(owner)) = (
            link.as_ref().and_then(|field| attempt.values.get(field)),
            result.owner,
        ) {
            let own = !attempt.admitted && reads_through(&owner.0);
            if !own && !owners.iter().any(|(held, _)| held == key) {
                owners.push((key.clone(), owner));
            }
        }
        share_related(plan, &attempt, &guard, result.guard_row, &mut shared);
        arranged.push(Arranged {
            admitted: attempt.admitted,
            row: attempt,
            arrangement: result.reached,
            prelude: result.prelude,
        });
    }
    observe(plan, &arranged, &params)
}

/// What `row`'s [`GuardLink`] inputs name and the related row it shares ([`RowGuard`]): the owner
/// the row is created under, where its own owner link is filled from one of the inputs; else the
/// owner its slot stands for ([`guard_slots`]). The related row is the one an earlier row given
/// the same value of [`Plan::via_key`] was started on.
fn row_guard(
    plan: &Plan<'_>,
    row: &Row,
    found: Option<&Owner>,
    slot: Option<&Node>,
    shared: &Shared,
) -> RowGuard {
    let owner = match &plan.guard_link {
        Some(link) if link.own => found
            .filter(|owner| link.inputs.contains(&owner.row.0))
            .map(|owner| owner.row.1.instance.clone()),
        Some(_) => slot.and_then(|slot| {
            plan.guard_owners
                .iter()
                .find(|(held, _)| held == slot)
                .map(|(_, owner)| owner.instance.clone())
        }),
        None => None,
    };
    let related = plan
        .via_key
        .as_ref()
        .and_then(|key| row.values.get(key))
        .and_then(|value| {
            shared
                .iter()
                .find(|(held, named, _)| held == value && *named == owner)
        })
        .map(|(_, _, related)| related.clone());
    RowGuard { owner, related }
}

/// The related rows rows are started on, each with the value of [`Plan::via_key`] it was given and
/// the owner its [`GuardLink`] inputs named.
type Shared = Vec<(Node, Option<super::InstanceName>, Arrangement)>;

/// Keeps the related row `row` was started on for the later rows given the same value of
/// [`Plan::via_key`] whose inputs name the same owner. A row naming another owner is started on a
/// row of its own: the one shared is filed under the owner the first named, and would select
/// another branch.
fn share_related(
    plan: &Plan<'_>,
    row: &Row,
    guard: &RowGuard,
    related: Option<Arrangement>,
    shared: &mut Shared,
) {
    let (Some(value), Some(related)) = (
        plan.via_key.as_ref().and_then(|key| row.values.get(key)),
        related,
    ) else {
        return;
    };
    if !shared
        .iter()
        .any(|(held, named, _)| held == value && *named == guard.owner)
    {
        shared.push((value.clone(), guard.owner.clone(), related));
    }
}

/// Points every [`GuardLink`] input at the owner `named`: the owner's token is what the guard is
/// decided on, and the owner itself is what is sent (`Setup::bound`, [`create_from`]).
fn name_guard_owner(
    plan: &Plan<'_>,
    creator: &Driver<'_>,
    row: &Row,
    named: Option<&super::InstanceName>,
    input: &mut BTreeMap<String, Node>,
) -> Result<(), RefusalCause> {
    let (Some(link), Some(named)) = (&plan.guard_link, named) else {
        return Ok(());
    };
    for sent in &link.inputs {
        let token =
            subject_fact::token(plan.ir, creator.command, sent, named).ok_or_else(|| {
                plan.unwitnessed(format!(
                "`input.{sent}` cannot name the `{}` row `{}` is arranged under: its type holds \
                 no token of an instance",
                plan.ir.entity(&link.owner).name,
                row.label
            ))
            })?;
        input.insert(sent.clone(), token);
    }
    Ok(())
}

/// `bound`, with every [`GuardLink`] input naming `named`.
fn naming_guard_owner(
    plan: &Plan<'_>,
    bound: &BTreeMap<String, super::InstanceName>,
    named: Option<&super::InstanceName>,
) -> BTreeMap<String, super::InstanceName> {
    let mut bound = bound.clone();
    if let (Some(link), Some(named)) = (&plan.guard_link, named) {
        for sent in &link.inputs {
            bound.insert(sent.clone(), named.clone());
        }
    }
    bound
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
    (owner, guard): (Option<&Owner>, &RowGuard),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<Created, RefusalCause> {
    let ir = plan.ir;
    let mut input = base.clone();
    // An identity the scenario supplies (`instance:` published from `input.<field>`) is its own in
    // every row: the witness at this row's distinction, never the base row's again.
    if let Ok(distinct) = base_input(ir, creator, distinction) {
        for field in identity_inputs(creator.command) {
            if let Some(value) = distinct.get(&field) {
                input.insert(field, value.clone());
            }
        }
    }
    for (field, value) in &row.values {
        if plan.related.contains_key(field.as_str()) {
            continue;
        }
        let Some(read) = mapped.get(field.as_str()) else {
            return Err(plan.unwitnessed(format!(
                "the creating command does not set `{field}` for row `{}`",
                row.label
            )));
        };
        // An absent value is an `Optional` input left out, never sent as `null`: the row then
        // holds the field it fills as absent.
        if *value == Node::Null {
            input.remove(*read);
        } else {
            input.insert((*read).to_owned(), value.clone());
        }
    }
    name_guard_owner(plan, creator, row, guard.owner.as_ref(), &mut input)?;
    if !subject_fact::input_selects(ir, creator.command, creator.outcome, &input).unwrap_or(false) {
        return Err(plan.unwitnessed(format!(
            "`{}/{}` is not selected by the input row `{}` needs",
            creator.command.name, creator.outcome.name, row.label
        )));
    }
    let drive = |input: &BTreeMap<String, Node>| {
        drive_row(
            plan,
            creator,
            mapped,
            row,
            distinction,
            (owner, guard),
            actors,
            params,
            input,
        )
    };
    let first = drive(&input);
    if first.is_ok() {
        return first;
    }
    // A move guarded by the row's stored fields that the plain witness does not select would send
    // the route searching for another row, with values of its own in the fields the pattern chose
    // (beyond10x/ess#279). So the row is created again with every field the pattern did not choose
    // taken toward those guards, and the pattern's own values kept.
    let mut planned: BTreeSet<String> = identity_inputs(creator.command).into_iter().collect();
    planned.extend(
        row.values
            .keys()
            .filter_map(|field| mapped.get(field.as_str()))
            .map(|read| (*read).to_owned()),
    );
    planned.extend(
        input
            .iter()
            .filter(|(field, value)| base.get(*field) != Some(*value))
            .map(|(field, _)| field.clone()),
    );
    for toward in subject_fact::toward_moves(ir, plan.handle, creator) {
        let mut retried = input.clone();
        for (field, value) in toward {
            if !planned.contains(&field) {
                retried.insert(field, value);
            }
        }
        if retried == input
            || !subject_fact::input_selects(ir, creator.command, creator.outcome, &retried)
                .unwrap_or(false)
        {
            continue;
        }
        if let Ok(created) = drive(&retried) {
            return Ok(created);
        }
    }
    first
}

/// [`arrange_row`] from the creating command's `input`: the row created, driven to a state its
/// filter truth needs, and refused where a move on the way rewrote what the pattern chose.
#[allow(clippy::too_many_arguments)]
fn drive_row(
    plan: &Plan<'_>,
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    row: &Row,
    distinction: Distinction,
    (owner, guard): (Option<&Owner>, &RowGuard),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    params: &BTreeMap<String, ScenarioValue>,
    input: &BTreeMap<String, Node>,
) -> Result<Created, RefusalCause> {
    let ir = plan.ir;
    let created = create_row(
        plan,
        creator,
        mapped,
        row,
        distinction,
        (owner, guard),
        actors,
        input,
    )?;
    let start = &created.reached;

    let wanted_state = plan
        .keys
        .iter()
        .zip(&plan.tuples[row.tuple].1)
        .find_map(|((_, key), value)| match (key, value) {
            (Key::State(_), Node::Text(state)) => StateName::new(state).ok(),
            _ => None,
        })
        .or_else(|| row.state.clone());
    let all = ir.drivers();
    let drivers: &[Driver<'_>] = all.get(plan.handle).map_or(&[], Vec::as_slice);
    let targets: Vec<StateName> = match wanted_state {
        Some(state) => vec![state],
        None => plan.entity.lifecycle.states.iter().cloned().collect(),
    };
    let mut best: Option<Arrangement> = None;
    for target in targets {
        // From where the row was created: `into:` (ess/15) or the lifecycle's initial state.
        let Some(path) = route_from(ir, plan.handle, drivers, &start.state, &target) else {
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
        let selecting = plan.selecting(params, &reached);
        if shows(ir, plan.view, &reached.state, &reached.settled, &selecting) != Ok(row.admitted) {
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
    kept_as_planned(plan, row, start, &reached)?;
    Ok(Created { reached, ..created })
}

/// Create `row` from `input`, after the related rows it copies keys from: the row as created, and
/// the steps that create those related rows.
#[allow(clippy::too_many_arguments)]
fn create_row(
    plan: &Plan<'_>,
    creator: &Driver<'_>,
    mapped: &BTreeMap<&str, &str>,
    row: &Row,
    distinction: Distinction,
    (owner, guard): (Option<&Owner>, &RowGuard),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    input: &BTreeMap<String, Node>,
) -> Result<Created, RefusalCause> {
    let shared = owner.filter(|owner| owner.shared).map(|owner| &owner.row);
    let Referenced {
        named: referenced,
        read_from,
        omitted,
        others,
    } = arrange_related(plan, (creator, mapped), row, distinction, actors, shared)?;
    let prelude: Vec<ScenarioStep> = referenced
        .iter()
        .flat_map(|(_, row)| row.steps.iter().cloned())
        .collect();
    // An owner the row is created under that is the row a key is read from is that row, and is
    // what a later row with the same owner link is created under.
    let kept = owner.map(|owner| {
        let (field, arrangement) = &owner.row;
        match referenced.iter().find(|(via, _)| *via == field.as_str()) {
            Some((_, row)) => (field.clone(), row.clone()),
            None => (field.clone(), arrangement.clone()),
        }
    });
    let owner = kept.as_ref().map(|(field, owner)| {
        (
            field.clone(),
            Arrangement {
                steps: if referenced.iter().any(|(via, _)| via == field) {
                    Vec::new()
                } else {
                    owner.steps.clone()
                },
                ..owner.clone()
            },
        )
    });
    let (mut start, guard_row) = create_from(
        plan,
        creator,
        row,
        distinction,
        (owner.as_ref(), guard),
        actors,
        input,
        &referenced,
    )?;
    for (via, row) in &referenced {
        let from = (read_from.get(via), others.get(via));
        point_at(creator, &mut start, via, row, from, mapped).ok_or_else(|| {
            plan.unwitnessed(format!(
                "the creating command's `{via}` cannot be pointed at the row it reads"
            ))
        })?;
        start.source.extend(row.source.iter().cloned());
    }
    for via in &omitted {
        leave_out(creator, &mut start, via, mapped).ok_or_else(|| {
            plan.unwitnessed(format!(
                "the creating command's `{via}` cannot be left out for row `{}`",
                row.label
            ))
        })?;
    }
    Ok(Created {
        reached: start,
        prelude,
        owner: kept,
        guard_row,
    })
}

/// The creating command run for `row` with `input`, under `owner`, and the related row it was
/// started on where later rows share it ([`Plan::via_key`]).
///
/// A command with a related guard (beyond10x/ess#272) is sent with the row the guard reads
/// arranged first, and its [`GuardLink`] inputs naming the owner that row is filed under: the one
/// among `referenced` the guard's input names, where the plan arranged it to copy a key from — the
/// inputs then name the owner that row was filed under —, else the row an earlier row given the
/// same value of the guard's input was started on, and otherwise one [`related_guard::drive`]
/// arranges under the owner `guard` names. Where none selects the creating branch, the refusal
/// says why.
#[allow(clippy::too_many_arguments)]
fn create_from(
    plan: &Plan<'_>,
    creator: &Driver<'_>,
    row: &Row,
    distinction: Distinction,
    (owner, guard): (Option<&(String, Arrangement)>, &RowGuard),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    input: &BTreeMap<String, Node>,
    referenced: &[(&str, Arrangement)],
) -> Result<(Arrangement, Option<Arrangement>), RefusalCause> {
    let ir = plan.ir;
    let cannot = |why: &dyn std::fmt::Display| {
        plan.unwitnessed(format!("row `{}` cannot be created: {why}", row.label))
    };
    let Some((via, read)) = related_guard::reads(creator.command)
        .filter(|_| related_guard::routes(creator.command, creator.outcome))
    else {
        // This aggregate owns the related-row prelude and points the invocation at those
        // captured rows below. The general creator arrangement would create them again,
        // interleaving unused sources with the aggregate's rows and changing its witness.
        return super::created_by(ir, plan.handle, creator, distinction, owner, |bound, _| {
            Ok::<_, super::Unreachable>(super::invoke_with(ir, creator, None, actors, bound, input))
        })
        .map(|created| (created, None))
        .map_err(|_| plan.unwitnessed(format!("row `{}` cannot be created", row.label)));
    };
    let mut input = input.clone();
    let mut named = guard.owner.clone();
    let related = match referenced.iter().find(|(named, _)| *named == via) {
        Some((_, related)) => {
            if let Some(link) = &plan.guard_link {
                let filed = ir
                    .owner_of(read)
                    .and_then(|owned| related.settled.get(owned.via))
                    .and_then(|held| match &held.value {
                        ScenarioValue::Instance { instance } => Some(instance.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        cannot(&format!(
                            "the `{}` it reads is filed under no arranged `{}` for `input.{}` to \
                             name",
                            ir.entity(read).name,
                            ir.entity(&link.owner).name,
                            link.inputs.join("`, `input.")
                        ))
                    })?;
                name_guard_owner(plan, creator, row, Some(&filed), &mut input)?;
                named = Some(filed);
            }
            Some(related)
        }
        None => guard.related.as_ref(),
    };
    if let Some(related) = related {
        let created =
            super::created_by(ir, plan.handle, creator, distinction, owner, |bound, _| {
                let bound = naming_guard_owner(plan, bound, named.as_ref());
                related_guard::drive_on(ir, creator, actors, &bound, related, &input)
            })
            .map_err(|cause| cannot(&cause))?;
        return Ok((created, None));
    }
    let sharing = plan
        .via_key
        .as_ref()
        .is_some_and(|key| row.values.contains_key(key));
    let mut kept = None;
    let created = super::created_by(
        ir,
        plan.handle,
        creator,
        distinction,
        owner,
        |bound, steps| {
            let bound = naming_guard_owner(plan, bound, named.as_ref());
            if sharing {
                let (invocation, related) = related_guard::drive_sharing(
                    ir,
                    creator,
                    actors,
                    distinction,
                    (&bound, steps),
                    &input,
                    &[plan.handle],
                )?;
                kept = Some(related);
                return Ok(invocation);
            }
            related_guard::drive(
                ir,
                creator,
                None,
                actors,
                distinction,
                (&bound, steps),
                Some(&input),
                &[plan.handle],
            )
        },
    )
    .map_err(|cause| cannot(&cause))?;
    Ok((created, kept))
}

/// The owner a row is created under, and whether an earlier row with the same owner link
/// arranged it (its steps are then already the scenario's).
struct Owner {
    row: (String, Arrangement),
    shared: bool,
}

/// One row as created, or as it reached its state.
struct Created {
    reached: Arrangement,
    /// The steps that create the related rows it reads, run before any row ([`observe`]).
    prelude: Vec<ScenarioStep>,
    /// The owner it was created under, as a later row with the same owner link reuses it.
    owner: Option<(String, Arrangement)>,
    /// The related row it was started on, for later rows sharing it ([`Plan::via_key`]).
    guard_row: Option<Arrangement>,
}

/// The values one related row must hold, by field.
type Wanted<'a> = BTreeMap<&'a str, Node>;

/// Whether `row` holds every wanted value as chosen: a value it does not keep as sent — an owner
/// link, say — is not the scenario's choice.
fn holds(row: &Arrangement, values: &Wanted<'_>) -> bool {
    values.iter().all(|(field, value)| {
        row.settled.get(*field).map(|held| &held.value)
            == Some(&ScenarioValue::literal(value.clone()))
    })
}

/// The related rows `row` reads its keys from, each holding the values the row copies: one per
/// input that names one, by that input.
///
/// `shared` is the owner an earlier row with the same owner link was created under
/// (beyond10x/ess#193): where its link is an input a key is read through, that owner is the row
/// read, so rows that share an owner share its values and its group.
fn arrange_related<'p>(
    plan: &Plan<'p>,
    (creator, mapped): (&Driver<'p>, &BTreeMap<&str, &str>),
    row: &Row,
    distinction: Distinction,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    shared: Option<&(String, Arrangement)>,
) -> Result<Referenced<'p>, RefusalCause> {
    let mut wanted: BTreeMap<&str, (RelatedKey<'_>, Wanted<'_>)> = BTreeMap::new();
    for (field, value) in &row.values {
        let Some(key) = plan.related.get(field.as_str()) else {
            continue;
        };
        let (held, values) = wanted.entry(key.via).or_insert((*key, BTreeMap::new()));
        // One input read along two paths — once in one hop, once chained — names one row, which
        // cannot be both (ess/22, beyond10x/ess#285).
        if held.through != key.through {
            return Err(plan.unwitnessed(format!(
                "row `{}` reads `{}` along two paths of references",
                row.label, key.via
            )));
        }
        values.insert(key.field, value.clone());
    }
    // Under an exact observation every row the creating command reads through is arranged, also
    // where the view reads nothing copied from it: an input naming no row is a command a correct
    // target refuses, never a row it counts.
    if plan.exact {
        for key in plan.related.values() {
            wanted.entry(key.via).or_insert((*key, BTreeMap::new()));
        }
    }
    let mut out = Referenced::default();
    for (nth, (via, (key, values))) in wanted.into_iter().enumerate() {
        if let Some((_, owner)) = shared.filter(|(field, _)| field == via) {
            if !holds(owner, &values) {
                return Err(plan.unwitnessed(format!(
                    "row `{}` shares its `{via}` with an earlier row but not the values it copies \
                     from it",
                    row.label
                )));
            }
            out.named.push((
                via,
                Arrangement {
                    steps: Vec::new(),
                    ..owner.clone()
                },
            ));
            out.read_from.insert(via, ReadFrom::Named);
            continue;
        }
        let at = Distinction::further(RELATED_ROWS * (nth + 1) + distinction.get());
        match arrange_key_rows(plan, &key, &values, at, actors, &row.label)? {
            (Some(mut named), from) => {
                let chains = other_chains(creator, mapped, &key);
                let pointed =
                    point_others(plan, &mut named, &chains, at, actors, (&row.label, via))?;
                out.named.push((via, named));
                out.read_from.insert(via, from);
                out.others.insert(via, pointed);
            }
            (None, _) => out.omitted.push(via),
        }
    }
    Ok(out)
}

/// The chained reads of `creator`'s `sets:` through `key`'s input other than `key`'s own: each a
/// further reference on the row that input names, and the entity it names (ess/22,
/// beyond10x/ess#285).
fn other_chains<'ir>(
    creator: &Driver<'ir>,
    mapped: &BTreeMap<&str, &str>,
    key: &RelatedKey<'_>,
) -> Vec<(&'ir ResolvedRelatedHop, &'ir EntityHandle)> {
    let mut out: Vec<(&ResolvedRelatedHop, &EntityHandle)> = Vec::new();
    for set in &creator.outcome.sets {
        let ResolvedPayloadValue::RelatedField {
            via,
            through,
            entity,
            ..
        } = &set.value
        else {
            continue;
        };
        let [hop] = through.as_slice() else {
            continue;
        };
        let other = via_input(via, mapped) == Some(key.via) && through.as_slice() != key.through;
        if other && !out.iter().any(|(held, _)| held.field == hop.field) {
            out.push((hop, entity));
        }
    }
    out
}

/// Points every further reference `chains` names on `named` — the row a key's input is pointed at,
/// which other chained reads of the creating branch follow on along references of their own — at a
/// row of the entity each names, arranged first; or, where none can be arranged (the view's own
/// entity, which it would count) and the reference may be absent, leaves it absent. A required
/// reference no row can be arranged for is refused by name (ess/22, beyond10x/ess#285).
fn point_others(
    plan: &Plan<'_>,
    named: &mut Arrangement,
    chains: &[(&ResolvedRelatedHop, &EntityHandle)],
    at: Distinction,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (label, via): (&str, &str),
) -> Result<BTreeMap<String, Option<Arrangement>>, RefusalCause> {
    let ir = plan.ir;
    let mut out = BTreeMap::new();
    let mut prelude = Vec::new();
    for (nth, (hop, entity)) in chains.iter().enumerate() {
        let last = (*entity != plan.handle)
            .then(|| {
                arrange_first(
                    ir,
                    entity,
                    std::slice::from_ref(&ir.entity(entity).lifecycle.initial),
                    actors,
                    Distinction::further(at.get() + RELATED_ROWS / 2 + nth),
                    &[plan.handle],
                )
                .ok()
            })
            .flatten()
            .filter(|last| related::rewrite_reference(ir, named, &hop.field, Some(&last.instance)));
        match last {
            Some(last) => {
                prelude.extend(last.steps.iter().cloned());
                named.source.extend(last.source.iter().cloned());
                out.insert(hop.field.clone(), Some(last));
            }
            None if hop.type_ref.is_optional()
                && related::rewrite_reference(ir, named, &hop.field, None) =>
            {
                out.insert(hop.field.clone(), None);
            }
            None => {
                return Err(plan.unwitnessed(format!(
                    "row `{label}` reads `{via}` along two chains of references, and `{hop}` can \
                     be pointed at no row the arrangement makes"
                )))
            }
        }
    }
    prelude.append(&mut named.steps);
    named.steps = prelude;
    Ok(out)
}

/// Where the values a row copies through one input are read (ess/22, beyond10x/ess#285).
enum ReadFrom {
    /// The row the input is pointed at holds them.
    Named,
    /// A chained read: the last row it reaches holds them.
    Last(Arrangement),
    /// A reference on the way was left absent, so every value read is absent.
    Absent,
}

/// The related rows a created row reads its keys from ([`arrange_related`]).
#[derive(Default)]
struct Referenced<'p> {
    /// The row each input is pointed at, by input. A chained read's carries the steps of the rows
    /// it reaches through, first.
    named: Vec<(&'p str, Arrangement)>,
    /// Where each pointed input's values are read.
    read_from: BTreeMap<&'p str, ReadFrom>,
    /// The rows the further references of other chained reads through each input name, by the
    /// reference's field — `None` where it is left absent ([`point_others`]).
    others: BTreeMap<&'p str, BTreeMap<String, Option<Arrangement>>>,
    /// The Optional inputs left out, leaving the reference and every value read through it absent
    /// (ess/22, beyond10x/ess#285).
    omitted: Vec<&'p str>,
}

/// The rows one input a row's keys are read through is arranged as: the row the input is pointed
/// at, and where the values are read — or no row, where the input is left out.
///
/// A one-hop read is arranged as [`arrange_referenced`] arranges it, and only where that cannot
/// leave every value absent is its Optional input left out instead (ess/22, beyond10x/ess#285).
/// A chained read gets a row of the last entity holding the values and a row of the entity `via`
/// names whose next reference is pointed at it; where every value is absent, that next reference
/// is left absent instead, where it may be, else the input is left out, else the last row holds
/// the values absent.
fn arrange_key_rows(
    plan: &Plan<'_>,
    key: &RelatedKey<'_>,
    values: &Wanted<'_>,
    distinction: Distinction,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    label: &str,
) -> Result<(Option<Arrangement>, ReadFrom), RefusalCause> {
    let all_absent = !values.is_empty() && values.values().all(|value| *value == Node::Null);
    let [hop] = key.through else {
        if !key.through.is_empty() {
            return Err(plan.unwitnessed(format!(
                "row `{label}` reads through `{}` across more than two references",
                key.via
            )));
        }
        return match arrange_referenced(plan, key, values, distinction, actors, label) {
            Ok(row) => Ok((Some(row), ReadFrom::Named)),
            Err(_) if all_absent && key.optional_via => Ok((None, ReadFrom::Absent)),
            Err(error) => Err(error),
        };
    };
    let ir = plan.ir;
    let name = &ir.entity(&hop.entity).name;
    let middle = |to: Option<&crate::scenario::InstanceName>| {
        let mut row = arrange_first(
            ir,
            &hop.entity,
            std::slice::from_ref(&ir.entity(&hop.entity).lifecycle.initial),
            actors,
            distinction,
            &[plan.handle],
        )
        .ok()?;
        related::rewrite_reference(ir, &mut row, &hop.field, to).then_some(row)
    };
    if all_absent && hop.type_ref.is_optional() {
        if let Some(named) = middle(None) {
            return Ok((Some(named), ReadFrom::Absent));
        }
    }
    if all_absent && key.optional_via {
        return Ok((None, ReadFrom::Absent));
    }
    let last = arrange_referenced(plan, key, values, distinction, actors, label)?;
    let mut named = middle(Some(&last.instance)).ok_or_else(|| {
        plan.unwitnessed(format!(
            "no `{name}` can be created naming the row that row `{label}` reads through `{}`",
            key.via
        ))
    })?;
    let mut steps = last.steps.clone();
    steps.append(&mut named.steps);
    named.steps = steps;
    named.source.extend(last.source.iter().cloned());
    Ok((Some(named), ReadFrom::Last(last)))
}

/// The distinction the owners of [`related_owners`] are arranged at, one apart per owner.
const RELATED_OWNERS: usize = 200 * RELATED_ROWS;

/// The link to its owner of the entity a key is read from, where that link is the field read: the
/// value is then an owner's identity, which the arrangement creates and the pattern cannot choose.
fn owner_link<'ir>(ir: &'ir EssIr, key: &RelatedKey<'_>) -> Option<&'ir str> {
    ir.owner_of(key.entity)
        .map(|owned| owned.via)
        .filter(|via| *via == key.field)
}

/// One owner per related entity and value the pattern gives a key read from that entity's link to
/// its owner (beyond10x/ess#272): each value stands for the owner, arranged once where its
/// lifecycle starts, and every related row given that value is filed under it — so rows given one
/// value hold one owner and fall in one group, and rows given another hold another. The owners are
/// created first ([`observe`]), before any related row.
fn related_owners(
    plan: &Plan<'_>,
    rows: &[Row],
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Vec<(EntityHandle, Node, Arrangement)>, RefusalCause> {
    let ir = plan.ir;
    let mut out: Vec<(EntityHandle, Node, Arrangement)> = Vec::new();
    for row in rows {
        for (field, value) in &row.values {
            let Some(key) = plan.related.get(field.as_str()) else {
                continue;
            };
            if *value == Node::Null || owner_link(ir, key).is_none() {
                continue;
            }
            if out
                .iter()
                .any(|(entity, held, _)| entity == key.entity && held == value)
            {
                continue;
            }
            let Some(belongs) = ir.owner_of(key.entity) else {
                continue;
            };
            let initial = ir.entity(&belongs.owner).lifecycle.initial.clone();
            let owner = super::arrange_first(
                ir,
                &belongs.owner,
                std::slice::from_ref(&initial),
                actors,
                Distinction::further(RELATED_OWNERS + out.len()),
                &[plan.handle, key.entity],
            )
            .map_err(|_| {
                plan.unwitnessed(format!(
                    "no `{}` can be arranged for the `{}` row `{}` reads its `{field}` from to be \
                     filed under",
                    ir.entity(&belongs.owner).name,
                    ir.entity(key.entity).name,
                    row.label
                ))
            })?;
            out.push((key.entity.clone(), value.clone(), owner));
        }
    }
    Ok(out)
}

/// A row of the entity `key` reads, created holding `values` through one branch that fills every
/// one of them from its input — leaving out the input of a value that is absent.
fn arrange_referenced(
    plan: &Plan<'_>,
    key: &RelatedKey<'_>,
    values: &Wanted<'_>,
    distinction: Distinction,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    label: &str,
) -> Result<Arrangement, RefusalCause> {
    let ir = plan.ir;
    let name = &ir.entity(key.entity).name;
    let refuse = || {
        plan.unwitnessed(format!(
            "no `{name}` can be created holding the values row `{label}` reads through `{}`",
            key.via
        ))
    };
    let creator = related_creators(ir, key.entity)
        .into_iter()
        .find(|driver| {
            values.iter().all(|(field, value)| {
                filled_from(driver, field).is_some()
                    && (*value != Node::Null || leaves_absent(driver, field))
            })
        })
        .ok_or_else(|| {
            let fields: Vec<String> = values.keys().map(|field| format!("`{field}`")).collect();
            plan.unwitnessed(format!(
                "no one branch creates a `{name}` that sets {} from its input, as row `{label}` \
                 needs",
                fields.join(", ")
            ))
        })?;
    let mut input =
        reach(ir, creator.command, creator.outcome, distinction).map_err(|_| refuse())?;
    for (field, value) in values {
        let Some(read) = filled_from(&creator, field) else {
            return Err(refuse());
        };
        // An absent value is an `Optional` input left out, never sent as `null`.
        if *value == Node::Null {
            input.remove(read);
        } else {
            input.insert(read.to_owned(), value.clone());
        }
    }
    if !subject_fact::input_selects(ir, creator.command, creator.outcome, &input).unwrap_or(false) {
        return Err(refuse());
    }
    let chain = [plan.handle];
    // Where the value read is the row's link to its owner, the row is filed under the owner that
    // value stands for ([`related_owners`]), which an earlier step created.
    let mut values = values.clone();
    let filed = match owner_link(ir, key).and_then(|via| Some((via, values.remove(via)?))) {
        Some((via, value)) if value != Node::Null => {
            let (_, _, owner) = plan
                .related_owners
                .iter()
                .find(|(entity, held, _)| entity == key.entity && *held == value)
                .ok_or_else(refuse)?;
            let read = filled_from(&creator, via).ok_or_else(refuse)?;
            Some((
                read.to_owned(),
                Arrangement {
                    steps: Vec::new(),
                    ..owner.clone()
                },
            ))
        }
        Some((via, value)) => {
            values.insert(via, value);
            None
        }
        None => None,
    };
    let owner = match &filed {
        Some(_) => filed.clone(),
        None => arrange_owner(ir, creator.outcome, key.entity, actors, distinction, &chain),
    };
    let row = created_owned(
        ir,
        key.entity,
        &creator,
        actors,
        distinction,
        &chain,
        owner.as_ref(),
        Some(&input),
    )
    .map_err(|_| refuse())?;
    let under = filed.as_ref().is_none_or(|(_, owner)| {
        owner_link(ir, key).is_some_and(|via| {
            row.settled.get(via).map(|held| &held.value)
                == Some(&ScenarioValue::instance(owner.instance.clone()))
        })
    });
    if !under || !holds(&row, &values) {
        return Err(refuse());
    }
    Ok(row)
}

/// Points the creating command's input `via` at the referenced row `row`, in the step that runs it
/// and in what it settled: the fields it fills from `via`, and those it copies from `row`.
///
/// A chained read's values are read from the last row it reaches, and a read through a reference
/// left absent copies absent values (ess/22, beyond10x/ess#285).
fn point_at(
    creator: &Driver<'_>,
    start: &mut Arrangement,
    via: &str,
    row: &Arrangement,
    (from, others): (
        Option<&ReadFrom>,
        Option<&BTreeMap<String, Option<Arrangement>>>,
    ),
    mapped: &BTreeMap<&str, &str>,
) -> Option<()> {
    let absent = Determined {
        value: ScenarioValue::literal(Node::Null),
        type_ref: ResolvedTypeRef::Primitive {
            name: Primitive::String,
        },
    };
    let command = CommandRef::new(creator.command.name.clone());
    let pointed = ScenarioValue::instance(row.instance.clone());
    let step = start.steps.iter_mut().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand {
            command: run,
            input,
            ..
        } if *run == command => Some(input),
        _ => None,
    })?;
    step.insert(via.to_owned(), pointed.clone());
    for set in creator
        .outcome
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
    {
        let value = match &set.value {
            ResolvedPayloadValue::InputField { field, .. } if field == via => Some(pointed.clone()),
            ResolvedPayloadValue::RelatedField {
                via: read,
                through,
                field,
                ..
            } if via_input(read, mapped) == Some(via) => {
                // A one-hop read reads the row the input names; a chained one the row its next
                // reference names there — another chain's ([`point_others`]) or the key's own —
                // and nothing where that reference was left absent (ess/22, beyond10x/ess#285).
                let other = through
                    .first()
                    .and_then(|hop| others.and_then(|others| others.get(&hop.field)));
                match (through.is_empty(), other, from) {
                    (true, _, _) | (false, None, None | Some(ReadFrom::Named)) => {
                        row.settled.get(field)
                    }
                    (false, Some(Some(last)), _) | (false, None, Some(ReadFrom::Last(last))) => {
                        last.settled.get(field)
                    }
                    (false, Some(None), _) | (false, None, Some(ReadFrom::Absent)) => Some(&absent),
                }
                .map(|held| held.value.clone())
            }
            _ => continue,
        };
        match value {
            Some(value) => {
                start.settled.insert(
                    set.target.clone(),
                    Determined {
                        value,
                        type_ref: set.target_type.clone(),
                    },
                );
            }
            None => {
                start.settled.remove(&set.target);
            }
        }
    }
    Some(())
}

/// Leaves the creating command's Optional input `via` out (ess/22, beyond10x/ess#285), in the step
/// that runs it and in what it settled: the fields it fills from `via`, and those it copies through
/// it, are absent.
fn leave_out(
    creator: &Driver<'_>,
    start: &mut Arrangement,
    via: &str,
    mapped: &BTreeMap<&str, &str>,
) -> Option<()> {
    let command = CommandRef::new(creator.command.name.clone());
    let step = start.steps.iter_mut().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand {
            command: run,
            input,
            ..
        } if *run == command => Some(input),
        _ => None,
    })?;
    step.remove(via);
    for set in creator
        .outcome
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
    {
        let reads = match &set.value {
            ResolvedPayloadValue::InputField { field, .. } => field == via,
            ResolvedPayloadValue::RelatedField { via: read, .. } => {
                via_input(read, mapped) == Some(via)
            }
            _ => false,
        };
        if !reads {
            continue;
        }
        if !set.target_type.is_optional() {
            return None;
        }
        start.settled.insert(
            set.target.clone(),
            Determined {
                value: ScenarioValue::literal(Node::Null),
                type_ref: set.target_type.clone(),
            },
        );
    }
    Some(())
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
        // The link to the owner holds the owner the row was created under, not the value that
        // named its group (see `arrange_and_observe`).
        let linked = created
            .settled
            .get(field)
            .is_some_and(|held| matches!(held.value, ScenarioValue::Instance { .. }));
        let planned = match row.values.get(field) {
            Some(value) if !linked => Some(ScenarioValue::literal(value.clone())),
            _ => literal(created, field),
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
        // An owner the implementation named: one token per instance, which is all a group key or a
        // `count_distinct` reads of it (beyond10x/ess#193).
        ScenarioValue::Instance { instance } => Some(Node::Text(format!("instance:{instance}"))),
        _ => None,
    }
}

/// The value a group key is asserted as: what its rows were given, which for a link to an owner is
/// the owner's identity and never the token [`held`] groups by.
fn key_value(arranged: &[Arranged], members: &[usize], key: &str, held: &Node) -> ScenarioValue {
    members
        .first()
        .and_then(|index| arranged[*index].arrangement.settled.get(key))
        .map(|determined| &determined.value)
        .filter(|value| matches!(value, ScenarioValue::Instance { .. }))
        .cloned()
        .unwrap_or_else(|| ScenarioValue::literal(held.clone()))
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

/// The steps run before any row: the owners [`GuardLink`] inputs name and the owners related rows
/// are filed under, then every related row a key is read from.
fn first_steps(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    steps: &mut Vec<ScenarioStep>,
    source: &mut BTreeSet<EssSemanticRef>,
) {
    let owners = plan.guard_owners.iter().map(|(_, owner)| owner);
    for owner in owners.chain(plan.related_owners.iter().map(|(_, _, owner)| owner)) {
        steps.extend(owner.steps.iter().cloned());
        source.extend(owner.source.iter().cloned());
    }
    for row in arranged {
        steps.extend(row.prelude.iter().cloned());
    }
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
    // Every related row a key is read from exists before the first row reads one, so a target
    // that copies from the row registered last or first, not the one named, copies another value.
    // The owners those rows are filed under exist before any of them.
    first_steps(plan, arranged, &mut steps, &mut source);
    for row in arranged {
        steps.extend(row.arrangement.steps.iter().cloned());
        source.extend(row.arrangement.source.iter().cloned());
    }

    if plan.delta {
        return observe_change(plan, arranged, &name, steps, source);
    }

    let groups = groups(plan, arranged)?;
    if plan.exact {
        return observe_exact(plan, arranged, &groups, params, (&name, steps, source));
    }

    let mut expectations = Vec::new();
    // The conditioned measures some asserted group decides, and whether one asserted group selects
    // nothing for every one of them (beyond10x/ess#363).
    let (mut decided, mut nothing) = (BTreeSet::new(), false);
    for (tuple, members) in &groups {
        let keys: BTreeMap<String, ScenarioValue> = plan
            .aggregation
            .group_by
            .iter()
            .zip(tuple)
            .map(|(key, value)| (key.clone(), key_value(arranged, members, key, value)))
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
        if !plan.scoped_tuple(tuple) {
            // Other scenarios' rows that lack the key land here too: the group exists, and that is
            // all this scenario can say of it.
            expectations.push(ViewExpectation::Contains { fields: keys });
            continue;
        }
        if members.contains(&0) {
            rounding_is_observable(plan, arranged, &admitted, params)?;
        }
        nothing |= contrast::decisive(plan, arranged, &admitted, params, &mut decided)?;
        let mut fields = keys;
        for (field, value) in aggregates_over(plan, arranged, &admitted, params)? {
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
            for (field, value) in aggregates_over(plan, arranged, &[], params)? {
                fields.insert(field, ScenarioValue::literal(value));
            }
            expectations.push(ViewExpectation::Contains { fields });
        }
        expectations.push(one.clone());
    }
    contrast::all_decisive(plan, &decided, nothing)?;
    read(view, &name, params, expectations, &mut steps);

    if plan.aggregation.is_ungrouped() {
        // Decision 4: the one row exists when no row passes the filter.
        let mut fields = BTreeMap::new();
        for (field, value) in aggregates_over(plan, arranged, &[], params)? {
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

/// The observation of an [`exact`](Plan::exact) scenario (`docs/design/aggregate-group-selection.md`):
/// one read per selection, each evaluating the whole filter afresh over every row the scenario
/// reached with that read's own parameters. A read holds a `Contains` with every aggregate for each
/// group it admits a row of, an `Excludes` for each arranged group it admits none of, and the exact
/// number of rows — so a group nobody arranged, or one merged or dropped, is caught.
fn observe_exact(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    groups: &[Group],
    params: &BTreeMap<String, ScenarioValue>,
    (name, mut steps, source): (&ViewRef, Vec<ScenarioStep>, BTreeSet<EssSemanticRef>),
) -> Result<ConformanceScenario, RefusalCause> {
    if let Some((_, members)) = groups.iter().find(|(_, members)| members.contains(&0)) {
        let admitted: Vec<usize> = members
            .iter()
            .copied()
            .filter(|index| arranged[*index].admitted)
            .collect();
        if !admitted.is_empty() {
            rounding_is_observable(plan, arranged, &admitted, params)?;
        }
    }
    // The conditioned measures some asserted group decides, and whether one asserted group selects
    // nothing for every one of them (beyond10x/ess#363).
    let mut decided = BTreeSet::new();
    let mut nothing = false;
    for read_with in selections(plan, arranged, groups, params) {
        let mut expectations = Vec::new();
        let mut answered = 0;
        for (tuple, members) in groups {
            let keys: BTreeMap<String, ScenarioValue> = plan
                .aggregation
                .group_by
                .iter()
                .zip(tuple)
                .map(|(key, value)| (key.clone(), key_value(arranged, members, key, value)))
                .collect();
            let mut admitted = Vec::new();
            for index in members {
                let row = &arranged[*index].arrangement;
                match shows(plan.ir, plan.view, &row.state, &row.settled, &read_with) {
                    Ok(true) => admitted.push(*index),
                    Ok(false) => {}
                    Err(unknown) => {
                        let unknown: Vec<String> =
                            unknown.iter().map(|path| format!("`{path}`")).collect();
                        return Err(plan.unwitnessed(format!(
                            "the filter's truth for row `{}` under one of its reads is unknown: \
                             nothing answers {}",
                            arranged[*index].row.label,
                            unknown.join(", ")
                        )));
                    }
                }
            }
            // An ungrouped view is one row whatever it admits, its aggregates over no row included.
            if admitted.is_empty() && !plan.aggregation.is_ungrouped() {
                expectations.push(ViewExpectation::Excludes { fields: keys });
                continue;
            }
            nothing |= contrast::decisive(plan, arranged, &admitted, &read_with, &mut decided)?;
            let mut fields = keys;
            for (field, value) in aggregates_over(plan, arranged, &admitted, &read_with)? {
                fields.insert(field, ScenarioValue::literal(value));
            }
            expectations.push(ViewExpectation::Contains { fields });
            answered += 1;
        }
        expectations.push(ViewExpectation::Counts {
            at_least: Some(answered),
            at_most: Some(answered),
        });
        read(plan.view, name, &read_with, expectations, &mut steps);
    }
    contrast::all_decisive(plan, &decided, nothing)?;
    let purpose = if plan.aggregation.is_ungrouped() {
        format!(
            "`{}` reports its one row's exact aggregates over the rows this scenario made",
            plan.view.name
        )
    } else if plan.selectors.is_empty() {
        format!(
            "`{}` reports every group the rows this scenario made reach, each with its exact \
             aggregates, and no other group",
            plan.view.name
        )
    } else {
        format!(
            "`{}` answers each selection with exactly the groups it admits over the rows this \
             scenario made, each with its exact aggregates",
            plan.view.name
        )
    };
    Ok(ConformanceScenario::new(clipped(&purpose), steps, source))
}

/// The parameters of each read of an [`exact`](Plan::exact) scenario, in order: the scoped
/// parameters alone where nothing selects a group; otherwise every distinct selection the arranged
/// groups hold, in group order, then each selector moved to a valid value no arranged group holds,
/// then, with several selectors, the first combination of arranged values no group holds — at most
/// [`MAX_READS`]. A selection that needs an absent key is no selection: no equality asks for it.
fn selections(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    groups: &[Group],
    params: &BTreeMap<String, ScenarioValue>,
) -> Vec<BTreeMap<String, ScenarioValue>> {
    if plan.selectors.is_empty() {
        return vec![params.clone()];
    }
    let absent = ScenarioValue::literal(Node::Null);
    let held = |selector: &Selector, (tuple, members): &Group| {
        key_value(arranged, members, &selector.field, &tuple[selector.key])
    };
    let mut vectors: Vec<Vec<ScenarioValue>> = Vec::new();
    for group in groups {
        let vector: Vec<ScenarioValue> = plan
            .selectors
            .iter()
            .map(|selector| held(selector, group))
            .collect();
        if !vector.contains(&absent) && !vectors.contains(&vector) {
            vectors.push(vector);
        }
    }
    let arranged_vectors = vectors.clone();
    if let Some(first) = arranged_vectors.first() {
        for (position, selector) in plan.selectors.iter().enumerate() {
            let values: Vec<ScenarioValue> =
                groups.iter().map(|group| held(selector, group)).collect();
            if let Some(value) = unheld(plan, selector, &values) {
                let mut vector = first.clone();
                vector[position] = ScenarioValue::literal(value);
                if !vectors.contains(&vector) {
                    vectors.push(vector);
                }
            }
        }
    }
    if plan.selectors.len() > 1 {
        let mut columns: Vec<Vec<ScenarioValue>> = vec![Vec::new(); plan.selectors.len()];
        for vector in &arranged_vectors {
            for (column, value) in columns.iter_mut().zip(vector) {
                if !column.contains(value) {
                    column.push(value.clone());
                }
            }
        }
        let mut combination = vec![0_usize; columns.len()];
        'combinations: loop {
            let vector: Vec<ScenarioValue> = combination
                .iter()
                .zip(&columns)
                .map(|(at, column)| column[*at].clone())
                .collect();
            if !arranged_vectors.contains(&vector) {
                if !vectors.contains(&vector) {
                    vectors.push(vector);
                }
                break;
            }
            for (at, column) in combination.iter_mut().zip(&columns).rev() {
                *at += 1;
                if *at < column.len() {
                    continue 'combinations;
                }
                *at = 0;
            }
            break;
        }
    }
    vectors.truncate(MAX_READS);
    vectors
        .into_iter()
        .map(|vector| {
            let mut bound = params.clone();
            for (selector, value) in plan.selectors.iter().zip(vector) {
                bound.insert(selector.param.clone(), value);
            }
            bound
        })
        .collect()
}

/// A value of `selector`'s key that is valid for its type and unequal to every value in `held`,
/// where one exists: a scoped value no scenario writes, the next step of an unbounded ladder, or a
/// variant, `Boolean` or state no arranged group holds. None for a key whose values are instances
/// the target generates, and none where the domain is used up — no invalid value is invented.
fn unheld(plan: &Plan<'_>, selector: &Selector, held: &[ScenarioValue]) -> Option<Node> {
    if held
        .iter()
        .any(|value| matches!(value, ScenarioValue::Instance { .. }))
    {
        return None;
    }
    let ladder = |ladder: &Ladder| -> Vec<Node> {
        let bound = ladder.len().unwrap_or(held.len() + 1);
        (0..bound).map(|ordinal| ladder.at(ordinal)).collect()
    };
    let candidates = match &plan.keys[selector.key].1 {
        Key::Scoped(kind) => vec![plan.scoped(*kind, "unselected")],
        Key::State(states) => states
            .iter()
            .map(|state| Node::Text(state.to_string()))
            .collect(),
        Key::Walked(walked) => ladder(walked),
        Key::Fixed(_) => plan
            .entity
            .observable_field(&selector.field)
            .and_then(|field| {
                Ladder::of(
                    &plan.view.name,
                    &selector.field,
                    &leaf(plan.ir, &field.type_ref).1,
                )
            })
            .map(|walked| ladder(&walked))
            .unwrap_or_default(),
    };
    // Only a value the parameter's own declared type admits is ever sent: a read the target must
    // refuse as a request asks nothing of the view. With none, the read is not made.
    let declared = plan
        .view
        .params
        .iter()
        .find(|param| param.name == selector.param)?;
    candidates.into_iter().find(|candidate| {
        !held.contains(&ScenarioValue::literal(candidate.clone()))
            && crate::input::validate_typed_value(plan.ir, &declared.type_ref, candidate).is_ok()
    })
}

/// Why an exact observation of `handle`'s rows is not this scenario's to make, where something
/// outside the arrangement changes them: a binding whose command changes them after the command
/// that triggers it returns — an exact count needs a causal cut after that binding's effects, which
/// no conformance step provides yet — or a declared precondition that makes them before every
/// scenario (`docs/design/aggregate-group-selection.md`, "Bindings and preconditions").
fn unsettled(ir: &EssIr, handle: &EntityHandle) -> Option<String> {
    let entity = &ir.entity(handle).name;
    let touches = |outcome: &ess_compiler::ir::ResolvedOutcome| {
        outcome
            .subject
            .as_ref()
            .is_some_and(|subject| subject.entity == *handle)
            || outcome
                .instances
                .as_ref()
                .is_some_and(|set| set.entity == *handle)
            || outcome
                .affects
                .iter()
                .any(|affect| affect.entity == *handle)
    };
    for binding in ir.bindings().values() {
        let command = ir.command(&binding.command);
        if command.outcomes.iter().any(touches) {
            return Some(format!(
                "the binding `{}` runs `{}`, which changes `{entity}` rows after the command that \
                 triggers it returns; an exact aggregate over every row the scenario holds needs \
                 a causal cut after that binding's effects, and no conformance step provides that \
                 binding cut yet",
                binding.name, command.name
            ));
        }
    }
    for precondition in ir.preconditions() {
        let command = ir.command(&precondition.command);
        if let Some(outcome) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name == precondition.outcome)
            .filter(|outcome| touches(outcome))
        {
            return Some(format!(
                "the declared precondition `{}/{}` runs before every scenario and makes `{entity}` \
                 rows this arrangement does not determine, so no exact total is this scenario's \
                 to assert",
                command.name, outcome.name
            ));
        }
    }
    None
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
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<(), RefusalCause> {
    for (field, aggregate) in &plan.aggregation.functions {
        let (AggregateFunction::Avg, Some(input)) = (aggregate.function, &aggregate.input) else {
            continue;
        };
        // A conditioned mean (beyond10x/ess#363) is over the rows its condition admits, and those
        // are the rows whose mean must tell rounding from truncation.
        let mut selected = Vec::new();
        for index in members {
            if let Some(condition) = &aggregate.r#where {
                if !contrast::holds(plan, arranged, *index, field, condition, params)? {
                    continue;
                }
            }
            selected.push(*index);
        }
        let members = &selected;
        let values: Option<Vec<Node>> = members
            .iter()
            .map(|index| held(plan, &arranged[*index], *index, &input.name))
            .collect();
        let values = values.map(|values| {
            if aggregate.skip_absent {
                crate::aggregate::present(&values)
            } else {
                values
            }
        });
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
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<Vec<(String, Node)>, RefusalCause> {
    let mut out = Vec::new();
    for (field, aggregate) in &plan.aggregation.functions {
        out.push((
            field.clone(),
            aggregate_over(plan, arranged, members, field, aggregate, params)?,
        ));
    }
    Ok(out)
}

/// One aggregate field's expected value over the admitted rows `members` — of those, where the
/// measure declares `where:` (beyond10x/ess#363), the ones its condition holds for under a read
/// sending `params`.
fn aggregate_over(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    members: &[usize],
    field: &str,
    aggregate: &ess_compiler::ir::ResolvedAggregate,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<Node, RefusalCause> {
    let selected: Vec<usize>;
    let members = match &aggregate.r#where {
        None => members,
        Some(condition) => {
            let mut kept = Vec::new();
            for index in members {
                if contrast::holds(plan, arranged, *index, field, condition, params)? {
                    kept.push(*index);
                }
            }
            selected = kept;
            &selected
        }
    };
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
    let value = if aggregate.skip_absent {
        evaluate_skipping_absent(aggregate.function, &values, kind)
    } else {
        evaluate(aggregate.function, &values, kind)
    }
    .ok_or_else(|| {
        plan.unwitnessed(format!(
            "`{field}` has no exact value over the arranged rows"
        ))
    })?;
    Ok(value)
}

/// An ungrouped view nothing scopes: read and snapshotted before the first row is created, and
/// read again after the last with the change in every `count` and `sum` and its one row.
///
/// The change is what the admitted rows add: a `count` by how many there are, a `sum` by the
/// total of their values (`0` where none holds one, as a skipping `sum` over no present value
/// changes by nothing). Every other function's change depends on rows the scenario did not make,
/// so it is named in the purpose as not asserted rather than asserted as a number the target's
/// other users decide.
fn observe_change(
    plan: &Plan<'_>,
    arranged: &[Arranged],
    name: &ViewRef,
    arranging: Vec<ScenarioStep>,
    source: BTreeSet<EssSemanticRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let admitted: Vec<usize> = (0..arranged.len())
        .filter(|index| arranged[*index].admitted)
        .collect();
    // A conditioned measure (beyond10x/ess#363) is decided by the change only where its value
    // over the admitted rows tells its condition from none and from the inverted one.
    let mut decided = BTreeSet::new();
    let nothing = contrast::decisive(plan, arranged, &admitted, &BTreeMap::new(), &mut decided)?;
    contrast::all_decisive(plan, &decided, nothing)?;
    let mut changes = BTreeMap::new();
    let mut absent_is_zero = BTreeSet::new();
    let mut unasserted = Vec::new();
    for (field, aggregate) in &plan.aggregation.functions {
        if !additive(aggregate.function) {
            unasserted.push(format!("`{field}`"));
            continue;
        }
        let value = match aggregate_over(
            plan,
            arranged,
            &admitted,
            field,
            aggregate,
            &BTreeMap::new(),
        )? {
            Node::Null => Node::Number(Number::from(0_usize)),
            value => value,
        };
        // Only a skipping `sum` is absent over no present value; a `count` and a required `sum`
        // are `0` over no row, so their absence on either read is a failure, not a zero.
        if aggregate.skip_absent && aggregate.function == AggregateFunction::Sum {
            absent_is_zero.insert(field.clone());
        }
        changes.insert(field.clone(), value);
    }
    let mut steps = vec![
        ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        },
        ScenarioStep::SnapshotView { view: name.clone() },
    ];
    steps.extend(arranging);
    read(
        plan.view,
        name,
        &BTreeMap::new(),
        vec![
            ViewExpectation::ChangedBy {
                fields: changes,
                absent_is_zero,
            },
            ViewExpectation::Counts {
                at_least: Some(1),
                at_most: Some(1),
            },
        ],
        &mut steps,
    );
    let purpose = if unasserted.is_empty() {
        format!(
            "`{}` changes by exactly what the rows this scenario made add; its absolute value \
             also counts rows other users made, and is not asserted",
            plan.view.name
        )
    } else {
        format!(
            "`{}` changes by exactly what the rows this scenario made add; {} not asserted, \
             since {} change depends on rows other users made",
            plan.view.name,
            unasserted.join(", ") + if unasserted.len() == 1 { " is" } else { " are" },
            if unasserted.len() == 1 {
                "its"
            } else {
                "their"
            }
        )
    };
    Ok(ConformanceScenario::new(clipped(&purpose), steps, source))
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
        let sizes: Vec<usize> = (0..=7).map(|inputs| group_size(inputs, &[])).collect();
        assert_eq!(sizes, [3, 3, 6, 6, 6, 7, 9, 9]);
        // Beside the absent row, `m + 1` must repeat too: not 3 (4), 7 (8) or 9 (10); 6 and 11 do.
        let beside: Vec<usize> = (0..=7).map(|inputs| group_size(inputs, &[1])).collect();
        assert_eq!(beside, [6, 6, 6, 6, 6, 11, 11, 11]);
    }

    /// A planner run without `scenario_initial_state: empty` keeps the shared-target contract: a
    /// parameter on a group key and a grouping nothing scopes are refused exactly as before
    /// (`docs/design/aggregate-group-selection.md`, "Problem and authority").
    #[test]
    fn without_empty_authority_the_old_refusals_stand() {
        use ess_compiler::{resolve::compile, source::SourceMap};
        use ess_domain::{spec::RawSpecFile, system::Source, Specification};
        let cases = [
            (
                include_str!("../../tests/fixtures/aggregate-group-parameter.yaml"),
                "ESS-SYNTH-017",
                "the parameter `team` is read other than by one top-level `field == param.team` \
                 conjunct over a field that is not a group key",
            ),
            (
                include_str!("../../tests/fixtures/aggregate-copied-group-parameter.yaml"),
                "ESS-SYNTH-017",
                "the parameter `team` is read other than by one top-level `field == param.team` \
                 conjunct over a field that is not a group key",
            ),
            (
                include_str!("../../tests/fixtures/aggregate-state-groups.yaml"),
                "ESS-SYNTH-016",
                "",
            ),
        ];
        for (text, code, reason) in cases {
            let raw = RawSpecFile::parse(text).unwrap();
            let spec = Specification::assemble([(Source::new("work.yaml"), raw)]).unwrap();
            let ir = compile(&spec, &SourceMap::new()).unwrap();
            let mut suite = ConformanceSuite::new(crate::scenario::SuiteProvenance::of(&ir));
            assert_eq!(suite.provenance.scenario_initial_state, None);
            let mut refusals = Vec::new();
            aggregates(
                &ir,
                &super::super::granted_actors(&ir),
                &mut suite,
                &mut refusals,
            );
            assert_eq!(suite.len(), 0, "{text}");
            assert_eq!(refusals.len(), 1, "{refusals:?}");
            assert_eq!(refusals[0].code().to_string(), code);
            match &refusals[0].cause {
                RefusalCause::AggregateUnwitnessed { reason: held, .. } => {
                    assert_eq!(held, reason);
                }
                RefusalCause::AggregateUnscoped { .. } => assert_eq!(reason, ""),
                other => panic!("{other:?}"),
            }
        }
        // The wider fixture: every view a fresh suite observes exactly — selected, grouped by the
        // state, an enum or a `Boolean` alone, or ungrouped with no `count` or `sum` — keeps its
        // old refusal here.
        let text = include_str!("../../tests/fixtures/aggregate-group-selection.yaml");
        let raw = RawSpecFile::parse(text).unwrap();
        let spec = Specification::assemble([(Source::new("work.yaml"), raw)]).unwrap();
        let ir = compile(&spec, &SourceMap::new()).unwrap();
        let mut suite = ConformanceSuite::new(crate::scenario::SuiteProvenance::of(&ir));
        let mut refusals = Vec::new();
        aggregates(
            &ir,
            &super::super::granted_actors(&ir),
            &mut suite,
            &mut refusals,
        );
        assert_eq!(suite.len(), 0);
        let codes: Vec<(String, String)> = refusals
            .iter()
            .map(|refusal| {
                (
                    refusal
                        .scenario
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    refusal.code().to_string(),
                )
            })
            .collect();
        let expected: Vec<(String, String)> = [
            ("ByBucket", "017"),
            ("ByKind", "016"),
            ("ByState", "016"),
            ("ByTeam", "017"),
            ("ByTeamChannel", "017"),
            ("ByTeamState", "017"),
            ("ByTeamUrgency", "017"),
            ("ByUrgency", "017"),
            ("ByUrgent", "016"),
            ("OpenByTeam", "017"),
            ("TeamsInLane", "017"),
            ("TopOpen", "016"),
        ]
        .into_iter()
        .map(|(view, code)| {
            (
                format!("demo.work.{view}/aggregate"),
                format!("ESS-SYNTH-{code}"),
            )
        })
        .collect();
        assert_eq!(codes, expected);
    }

    #[test]
    fn each_input_has_its_own_duplicate_pattern() {
        let column = |i| (0..6).map(|j| a_ordinal(i, j)).collect::<Vec<_>>();
        assert_eq!(column(0), [1, 1, 3, 7, 13, 21]);
        assert_eq!(column(1), [101, 101, 101, 103, 107, 113]);
        assert_eq!(column(2), [201, 201, 201, 201, 203, 207]);
    }
}
