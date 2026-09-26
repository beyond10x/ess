//! Views: what a system promises can be observed about it, and how soon.
//!
//! # Why consistency is part of the model
//!
//! §4.6 calls a view a "stable observable" and §18's conformance scenario asserts one immediately
//! after the command that caused it. If the view is a projection — the normal case — that assertion
//! passes on a laptop and flakes in CI, and the fix everyone reaches for is a sleep, which makes the
//! suite a test of the machine it runs on rather than of the system.
//!
//! So a view declares its [`Consistency`], and [`ViewSpec::assertion_style`] turns it into the block
//! a generated scenario must use:
//!
//! | [`Consistency`] | [`AssertionStyle`] | scenario |
//! |---|---|---|
//! | [`ReadYourWrites`](Consistency::ReadYourWrites) | [`Expect`](AssertionStyle::Expect) | `expect:` — the view is current the moment the command returns |
//! | [`Eventual`](Consistency::Eventual) | [`Eventually`](AssertionStyle::Eventually) | `eventually:` — the runner retries until the projection catches up |
//!
//! The default is [`Eventual`](Consistency::Eventual), and the asymmetry is the reason: declaring
//! read-your-writes and being wrong produces a suite that fails at random, which costs a person a
//! day and eventually costs the suite its credibility; declaring eventual and being wrong produces a
//! suite that is slower and still correct. The cheap mistake is the default.
//!
//! # What each rejection is called
//!
//! [`ValidationCode`] belongs to `aep-domain` and is closed to this crate, so ESS reuses the nearest
//! protocol code rather than opening a parallel vocabulary:
//!
//! | rule | code |
//! |---|---|
//! | a view declares neither `shape` nor `fields` | [`EmptyDeclaration`](ValidationCode::EmptyDeclaration) |
//! | a view declares both `shape` and `fields` | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | its source entity, shape, a projected field, or a projected field's type is not declared | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | its shape is not a struct | [`TypeMismatch`](ValidationCode::TypeMismatch) |
//! | a filter compares a field to a value the field's type does not have | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | a projected field's type disagrees with the entity's | [`TypeMismatch`](ValidationCode::TypeMismatch) |
//! | a filter reads something the source does not have | [`UnobservableFact`](ValidationCode::UnobservableFact) |
//! | a field is projected twice | [`DuplicateDeclaration`](ValidationCode::DuplicateDeclaration) |
//!
//! An **aggregate view** — `group_by:` and a field-level `aggregate:` — adds fifteen rules, V1–V15 of
//! `docs/design/aggregate-views.md`; [`Aggregation`] says which run where.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_primitives::error::{ParseError, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::Predicate;

use crate::name::{Naming, QualifiedName};
use crate::types::{Field, Primitive, TypeBody, TypeRef, TypeRegistry, MAX_TYPE_DEPTH};

/// How soon a view reflects a command that has already returned.
///
/// This decides whether a generated scenario asserts the view with `expect` or with `eventually` —
/// which is the entire reason the field exists. Getting it wrong in the model is how a generated
/// suite acquires a sleep.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Consistency {
    /// The view is current as soon as the command that changed it returns.
    ///
    /// Only for an implementation that reads the same store it wrote, or one that carries a
    /// consistency token from the write to the read.
    ReadYourWrites,
    /// The view catches up some time after the command returns.
    ///
    /// The default, because assuming read-your-writes and being wrong produces a flaky suite,
    /// while assuming eventual and being wrong produces a slower but correct one.
    #[default]
    Eventual,
}

impl Consistency {
    /// The consistency as written in a document.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadYourWrites => "read_your_writes",
            Self::Eventual => "eventual",
        }
    }

    /// How a generated scenario must assert a view with this consistency.
    pub fn assertion_style(self) -> AssertionStyle {
        match self {
            Self::ReadYourWrites => AssertionStyle::Expect,
            Self::Eventual => AssertionStyle::Eventually,
        }
    }
}

impl fmt::Display for Consistency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The block a generated conformance scenario puts a view assertion in (§18).
///
/// Derived from the model rather than chosen per assertion by whoever writes the scenario, because a
/// choice made per assertion is a choice made wrong eventually.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AssertionStyle {
    /// `expect:` — assert once, immediately after the command.
    Expect,
    /// `eventually:` — retry until the projection catches up, with no sleep and no fixed delay.
    Eventually,
}

impl AssertionStyle {
    /// The scenario key this style writes, such as `eventually`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Expect => "expect",
            Self::Eventually => "eventually",
        }
    }

    /// `true` when the runner must retry rather than assert once.
    pub fn is_retried(self) -> bool {
        self == Self::Eventually
    }
}

impl fmt::Display for AssertionStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What [`ViewSpec::validate`] needs to know about the entities a view may project.
///
/// A trait rather than a reference to the entity model, so that this module does not depend on the
/// shape of a type it does not own, and so that a caller can supply whatever it already has.
///
/// The field list an implementation returns is the entity's **observable surface**, not only its
/// declared fields: it must include the identity, and the state where the entity has a state
/// machine. A view projects and filters all three — `InvoiceById` names `invoice_id`, and
/// `OutstandingInvoices` filters on `state` — and a view is the one place where the difference
/// between them does not matter.
///
/// The state must carry the entity's own synthesised state type
/// ([`EntitySpec::state_type`](crate::entity::EntitySpec::state_type)) rather than a bare string:
/// that is what lets [`ViewSpec::validate`] check `state == Issued` against the lifecycle's own
/// names, so a filter that can never match is refused instead of generated.
pub trait EntityFields {
    /// The observable fields of the entity with this name, or `None` when no such entity exists.
    fn entity_fields(&self, name: &QualifiedName) -> Option<&[Field]>;

    /// Every declared entity name, for a diagnostic that says what was available instead.
    fn entity_names(&self) -> Vec<String>;
}

impl EntityFields for BTreeMap<QualifiedName, Vec<Field>> {
    fn entity_fields(&self, name: &QualifiedName) -> Option<&[Field]> {
        self.get(name).map(Vec::as_slice)
    }

    fn entity_names(&self) -> Vec<String> {
        self.keys().map(ToString::to_string).collect()
    }
}

/// Which way a ranking key runs.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Smallest first. The default, because it is the one a reader assumes when nothing says.
    #[default]
    Ascending,
    /// Largest first.
    Descending,
}

impl Direction {
    /// The word as written in a document.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }

    /// Reads `asc`, `ascending`, `desc` or `descending`.
    fn parse(word: &str) -> Option<Self> {
        match word {
            "asc" | "ascending" => Some(Self::Ascending),
            "desc" | "descending" => Some(Self::Descending),
            _ => None,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ranking key of a view: a field it projects, and which way that field runs.
///
/// Written `priority desc`, or `queued_at` for the ascending default.
#[derive(Debug, Clone, PartialEq, Eq, schemars::JsonSchema)]
#[schemars(with = "String")]
pub struct Ranking {
    /// The projected field the rows are ordered by.
    pub field: String,
    /// Which way it runs.
    pub direction: Direction,
}

impl Ranking {
    /// Parses one key, written `<field>` or `<field> asc|desc`.
    fn parse(value: &str) -> Result<Self, ParseError> {
        let mut words = value.split_whitespace();
        let Some(field) = words.next() else {
            return Err(ParseError::shape(
                "order_by",
                "`<field> asc|desc`",
                "nothing",
            ));
        };
        let direction = match words.next() {
            None => Direction::Ascending,
            Some(word) => Direction::parse(word).ok_or_else(|| {
                ParseError::identifier(
                    "order_by direction",
                    word,
                    "must be `asc`, `ascending`, `desc` or `descending`".to_owned(),
                )
            })?,
        };
        if let Some(extra) = words.next() {
            return Err(ParseError::shape(
                "order_by",
                "`<field> asc|desc`",
                format!("a third word, `{extra}`"),
            ));
        }
        Ok(Self {
            field: field.to_owned(),
            direction,
        })
    }
}

impl fmt::Display for Ranking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.field, self.direction)
    }
}

impl serde::Serialize for Ranking {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Ranking {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = String::deserialize(deserializer)?;
        Self::parse(&written).map_err(serde::de::Error::custom)
    }
}

/// One aggregate function, as written (`docs/design/aggregate-views.md`, "Result types").
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AggregateFunction {
    /// How many admitted rows the group holds. Takes no argument.
    Count,
    /// How many distinct values of one field the group's rows hold.
    CountDistinct,
    /// The exact sum of one numeric field.
    Sum,
    /// The least value of one ordered field; absent over no rows.
    Min,
    /// The greatest value of one ordered field; absent over no rows.
    Max,
    /// The mean of one numeric field, rounded to 6 fractional digits, ties to even; absent over no
    /// rows.
    Avg,
}

impl AggregateFunction {
    /// Every function, in the order the page's table lists them.
    pub const ALL: [Self; 6] = [
        Self::Count,
        Self::CountDistinct,
        Self::Sum,
        Self::Min,
        Self::Max,
        Self::Avg,
    ];

    /// The function as written in a document.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::CountDistinct => "count_distinct",
            Self::Sum => "sum",
            Self::Min => "min",
            Self::Max => "max",
            Self::Avg => "avg",
        }
    }
}

impl fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One aggregate field's computation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Aggregate {
    /// The function.
    pub function: AggregateFunction,
    /// The source field it reads: a top-level observable field of the view's source. `None`
    /// exactly for [`AggregateFunction::Count`].
    pub input: Option<String>,
}

impl fmt::Display for Aggregate {
    /// `sum(talk_seconds)`, or `count()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.function,
            self.input.as_deref().unwrap_or_default()
        )
    }
}

impl From<&RawAggregate> for Aggregate {
    fn from(raw: &RawAggregate) -> Self {
        let (function, input) = match raw {
            RawAggregate::Count(_) => (AggregateFunction::Count, None),
            RawAggregate::CountDistinct(input) => {
                (AggregateFunction::CountDistinct, Some(input.clone()))
            }
            RawAggregate::Sum(input) => (AggregateFunction::Sum, Some(input.clone())),
            RawAggregate::Min(input) => (AggregateFunction::Min, Some(input.clone())),
            RawAggregate::Max(input) => (AggregateFunction::Max, Some(input.clone())),
            RawAggregate::Avg(input) => (AggregateFunction::Avg, Some(input.clone())),
        };
        Self { function, input }
    }
}

impl From<&Aggregate> for RawAggregate {
    fn from(aggregate: &Aggregate) -> Self {
        let input = aggregate.input.clone().unwrap_or_default();
        match aggregate.function {
            AggregateFunction::Count => Self::Count(Empty {}),
            AggregateFunction::CountDistinct => Self::CountDistinct(input),
            AggregateFunction::Sum => Self::Sum(input),
            AggregateFunction::Min => Self::Min(input),
            AggregateFunction::Max => Self::Max(input),
            AggregateFunction::Avg => Self::Avg(input),
        }
    }
}

/// What makes a view an aggregate view. Present iff at least one field declares `aggregate:`.
///
/// # Where each rule runs
///
/// V3, V5 and V13 of the design page run where the view is read (`TryFrom<RawViewSpec>`), because
/// their subject is the written `group_by` list, and a `group_by` with no aggregate has no
/// [`Aggregation`] to be checked later. Every other rule runs in [`ViewSpec::validate`], and V15 —
/// the source format — beside the other format gates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Aggregation {
    /// Group-key view field names, in declaration order. Empty: the view returns one row.
    pub group_by: Vec<String>,
    /// Aggregate view field name → its computation.
    pub functions: BTreeMap<String, Aggregate>,
}

impl Aggregation {
    /// The computation of the aggregate field `name`, where it is one.
    pub fn function(&self, name: &str) -> Option<&Aggregate> {
        self.functions.get(name)
    }

    /// Whether the view returns exactly one row: no group key.
    pub fn is_ungrouped(&self) -> bool {
        self.group_by.is_empty()
    }
}

/// An aggregate as written: a map with exactly one key, the function.
///
/// The shape is the reader's to refuse — an unknown function, two keys, or an argument to `count`
/// fails the parse with serde's message naming the six functions, as an unknown key does anywhere a
/// [`Field`] is read.
///
/// Read and written by hand as a one-entry map rather than derived: `serde_yaml` reads and writes a
/// derived externally tagged enum as a YAML tag (`!sum talk_seconds`), which is not the syntax the
/// design page fixes and not what a JSON document can carry. The schema is still the derived one —
/// one object per function with exactly that key.
#[derive(Debug, Clone, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RawAggregate {
    /// `{count: {}}`.
    Count(Empty),
    /// `{count_distinct: field}`.
    CountDistinct(String),
    /// `{sum: field}`.
    Sum(String),
    /// `{min: field}`.
    Min(String),
    /// `{max: field}`.
    Max(String),
    /// `{avg: field}`.
    Avg(String),
}

impl RawAggregate {
    /// Every function name, as the reader's refusal lists them.
    const NAMES: &'static [&'static str] = &["count", "count_distinct", "sum", "min", "max", "avg"];
}

impl serde::Serialize for RawAggregate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::Count(empty) => map.serialize_entry("count", empty)?,
            Self::CountDistinct(input) => map.serialize_entry("count_distinct", input)?,
            Self::Sum(input) => map.serialize_entry("sum", input)?,
            Self::Min(input) => map.serialize_entry("min", input)?,
            Self::Max(input) => map.serialize_entry("max", input)?,
            Self::Avg(input) => map.serialize_entry("avg", input)?,
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for RawAggregate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = RawAggregate;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a map with exactly one aggregate function as its key")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                use serde::de::Error;
                let Some(function) = map.next_key::<String>()? else {
                    return Err(A::Error::custom(format!(
                        "an aggregate names one function; expected one of {}",
                        RawAggregate::NAMES.join(", ")
                    )));
                };
                let aggregate = match function.as_str() {
                    "count" => RawAggregate::Count(map.next_value()?),
                    "count_distinct" => RawAggregate::CountDistinct(map.next_value()?),
                    "sum" => RawAggregate::Sum(map.next_value()?),
                    "min" => RawAggregate::Min(map.next_value()?),
                    "max" => RawAggregate::Max(map.next_value()?),
                    "avg" => RawAggregate::Avg(map.next_value()?),
                    other => return Err(A::Error::unknown_variant(other, RawAggregate::NAMES)),
                };
                if let Some(second) = map.next_key::<String>()? {
                    return Err(A::Error::custom(format!(
                        "an aggregate names exactly one function, and this one also names \
                         `{second}`; expected one of {}",
                        RawAggregate::NAMES.join(", ")
                    )));
                }
                Ok(aggregate)
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}

/// The empty map `count` takes: no other spelling of "no argument" is accepted.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Empty {}

/// One view field as written: a [`Field`] that may declare `aggregate:`.
///
/// A sibling of [`Field`] rather than [`Field`] flattened, so `Field`'s `deny_unknown_fields` keeps
/// refusing `aggregate:` at every other position a field is written.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawViewField {
    /// Its name.
    #[serde(deserialize_with = "crate::types::deserialize_field_name")]
    #[schemars(regex(pattern = "^[A-Za-z][A-Za-z0-9_]*$"))]
    pub name: String,
    /// Its type. For an aggregate field, exactly the function's result type.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    /// What it is on the wire, and what a person is shown.
    #[serde(default, flatten, skip_serializing_if = "Naming::is_empty")]
    pub naming: Naming,
    /// What it computes over the group's rows, where it is an aggregate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<RawAggregate>,
}

impl RawViewField {
    fn split(self) -> (Field, Option<RawAggregate>) {
        (
            Field {
                name: self.name,
                type_ref: self.type_ref,
                naming: self.naming,
            },
            self.aggregate,
        )
    }
}

/// A type reference with every newtype followed, and whether an `Optional` was met on the way.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Unwrapped {
    /// An `Optional<…>` appears somewhere in the chain.
    optional: bool,
    /// What the chain ends in: a primitive, an enum, a struct, a union, a list or a map.
    leaf: Leaf,
}

/// The end of a newtype chain, classified for the aggregate tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Leaf {
    /// A primitive.
    Primitive(Primitive),
    /// A declared enum (including an entity's `state`).
    Enum,
    /// A struct, a union, a list, a map, or a type that does not resolve.
    Other,
}

fn unwrap_chain(type_ref: &TypeRef, types: &TypeRegistry) -> Unwrapped {
    let mut optional = false;
    let mut current = type_ref;
    for _ in 0..=MAX_TYPE_DEPTH {
        match current {
            TypeRef::Primitive(primitive) => {
                return Unwrapped {
                    optional,
                    leaf: Leaf::Primitive(*primitive),
                }
            }
            TypeRef::Optional(inner) => {
                optional = true;
                current = inner;
            }
            TypeRef::Named(name) => match types.get(name).map(|declared| &declared.body) {
                Some(TypeBody::Newtype { of, .. }) => current = of,
                Some(TypeBody::Enum { .. }) => {
                    return Unwrapped {
                        optional,
                        leaf: Leaf::Enum,
                    }
                }
                _ => break,
            },
            TypeRef::List(_) | TypeRef::Map(..) => break,
        }
    }
    Unwrapped {
        optional,
        leaf: Leaf::Other,
    }
}

impl Leaf {
    fn is(self, primitive: Primitive) -> bool {
        self == Self::Primitive(primitive)
    }
}

/// The result type `function` gives over a source field of type `source` unwrapping to `leaf`, or
/// `None` where the function does not admit it (V8).
fn result_type(function: AggregateFunction, leaf: Leaf, source: &TypeRef) -> Option<TypeRef> {
    use Primitive as P;
    let integer = TypeRef::Primitive(P::Integer);
    match function {
        AggregateFunction::Count => Some(integer),
        AggregateFunction::CountDistinct => (leaf == Leaf::Enum
            || [
                P::String,
                P::Integer,
                P::Decimal,
                P::Boolean,
                P::Uuid,
                P::Timestamp,
            ]
            .iter()
            .any(|admitted| leaf.is(*admitted)))
        .then_some(integer),
        // A sum drops a newtype: it does not satisfy the invariants its inputs do.
        AggregateFunction::Sum => match leaf {
            Leaf::Primitive(P::Integer) => Some(integer),
            Leaf::Primitive(P::Decimal) => Some(TypeRef::Primitive(P::Decimal)),
            _ => None,
        },
        // A minimum is one of the inputs, so it keeps the source's declared type.
        AggregateFunction::Min | AggregateFunction::Max => {
            [P::Integer, P::Decimal, P::String, P::Timestamp]
                .iter()
                .any(|admitted| leaf.is(*admitted))
                .then(|| TypeRef::Optional(Box::new(source.clone())))
        }
        AggregateFunction::Avg => (leaf.is(P::Integer) || leaf.is(P::Decimal))
            .then(|| TypeRef::Optional(Box::new(TypeRef::Primitive(P::Decimal)))),
    }
}

/// A declared projection of an entity: the part of it the outside world is promised.
///
/// A view does not require the implementation to use CQRS (§4.6). It says what can be observed, not
/// how the observation is served.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(into = "RawViewSpec")]
pub struct ViewSpec {
    /// Its stable identity.
    pub name: QualifiedName,
    /// The entity it projects.
    pub source: QualifiedName,
    /// A reusable named struct describing what it exposes.
    ///
    /// Exactly one of this and [`Self::fields`] is present. A shape is expanded and checked against
    /// the source entity just like inline fields; it is retained as a reference in compiled IR so a
    /// contract generator can reuse the named schema rather than copy it.
    pub shape: Option<QualifiedName>,
    /// What it exposes inline, in declaration order.
    ///
    /// Empty when [`Self::shape`] names the reusable row struct.
    pub fields: Vec<Field>,
    /// What the caller must supply to ask for this view, in declaration order.
    ///
    /// Empty is the common case: most views are one answer the whole system shares, and a reader
    /// asks for the lot. A parameter is for the view that has no such answer — ACD's position in a
    /// queue is *per queue*, and one ranked list of every queued call is a different thing from
    /// what the implementation can be asked. Before this, a specification with a parameterised
    /// read either declared the unparameterised view and generated scenarios nothing could answer,
    /// or did not declare it.
    ///
    /// A parameter is read in [`Self::filter`] as `param.<name>`, which is a fact path like any
    /// other — so the predicate grammar needs nothing new, and what makes the name a parameter is
    /// that it is declared here.
    pub params: Vec<Field>,
    /// Which instances it contains. Absent means all of them.
    pub filter: Option<Predicate>,
    /// What makes this an aggregate view, where it is one.
    ///
    /// Present exactly when at least one field declares `aggregate:`. The filter runs per source
    /// row first; the admitted rows are then partitioned by [`Aggregation::group_by`] and each
    /// aggregate is computed per partition (`docs/design/aggregate-views.md`). The aggregate fields
    /// stay in [`Self::fields`] under their declared result types, so every reader of the row shape
    /// keeps working unchanged.
    pub aggregation: Option<Aggregation>,
    /// The order the rows are ranked in, most significant key first. Empty means unordered.
    ///
    /// A view named for a position — `CallPosition`, `TopAgents` — says nothing about position
    /// without this. Absent is a real answer and stays the default: most views are sets, and a
    /// specification that made every one of them declare an order would be inventing a promise the
    /// implementation never made.
    pub order_by: Vec<Ranking>,
    /// How soon it reflects a command that has already returned.
    pub consistency: Consistency,
    /// What it is called on the wire and shown as.
    pub naming: Naming,
}

impl ViewSpec {
    /// Whether this view computes aggregates over its source's rows rather than projecting them.
    pub fn is_aggregate(&self) -> bool {
        self.aggregation.is_some()
    }

    /// The inline projected field with this name.
    ///
    /// Use [`Self::projected_fields`] when the view may declare a reusable shape.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.name == name)
    }

    /// The fields this view projects, whether written inline or supplied by a named struct shape.
    ///
    /// `None` means the shape does not resolve to a struct. [`Self::validate`] reports the precise
    /// refusal; this accessor is for consumers of an already validated specification.
    pub fn projected_fields<'a>(&'a self, types: &'a TypeRegistry) -> Option<&'a [Field]> {
        match &self.shape {
            None => Some(&self.fields),
            Some(shape) => match &types.get(shape)?.body {
                TypeBody::Struct { fields, .. } => Some(fields),
                _ => None,
            },
        }
    }

    /// Resolves the projected fields while retaining the diagnostic distinction between a missing
    /// shape and a declared non-struct shape.
    fn validated_projected_fields<'a>(
        &'a self,
        types: &'a TypeRegistry,
        errors: &mut ValidationErrors,
    ) -> Option<&'a [Field]> {
        let Some(shape) = &self.shape else {
            return Some(self.fields.as_slice());
        };
        let at = |suffix: &str| format!("view.{}.{suffix}", self.name);

        let Some(declared) = types.get(shape) else {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    at("shape"),
                    format!(
                        "`{shape}` is not a declared type, so `{}` has no row shape",
                        self.name
                    ),
                )
                .with_hint(format!(
                    "declare `{shape}` as a struct, or choose one of: {}",
                    join(types.iter().map(|declared| &declared.name))
                )),
            );
            return None;
        };

        if let TypeBody::Struct { fields, .. } = &declared.body {
            return Some(fields.as_slice());
        }

        errors.push(
            ValidationError::new(
                ValidationCode::TypeMismatch,
                at("shape"),
                format!(
                    "`{shape}` is not a struct, so it cannot describe the fields of `{}`",
                    self.name
                ),
            )
            .with_hint("a view shape must name a `kind: struct` declaration"),
        );
        None
    }

    /// How a generated scenario must assert this view.
    ///
    /// Exposed on the model so no generator has to decide it: an `eventual` view asserted with
    /// `expect` races the projection, and the usual repair — a sleep — makes the suite test the
    /// machine it runs on.
    pub fn assertion_style(&self) -> AssertionStyle {
        self.consistency.assertion_style()
    }

    /// Checks everything that can be checked without knowing what the domain declares.
    fn validate_shape(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at = |suffix: &str| format!("view.{}.{suffix}", self.name);

        match (&self.shape, self.fields.is_empty()) {
            (None, true) => {
                errors.push(
                    ValidationError::new(
                        ValidationCode::EmptyDeclaration,
                        at("fields"),
                        format!(
                            "`{}` declares neither `shape` nor `fields`, so it observes nothing and \
                             no scenario can assert anything about it",
                            self.name
                        ),
                    )
                    .with_hint(
                        "name a reusable struct with `shape`, or project at least the identity a \
                         scenario looks the entity up by",
                    ),
                );
            }
            (Some(_), false) => {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        at("shape"),
                        format!(
                            "`{}` declares both `shape` and inline `fields`, so there are two row \
                             contracts and no rule for which one wins",
                            self.name
                        ),
                    )
                    .with_hint("keep `shape` or `fields`, not both"),
                );
            }
            (None, false) | (Some(_), true) => {}
        }

        let mut seen = std::collections::BTreeSet::new();
        for (index, field) in self.fields.iter().enumerate() {
            if !seen.insert(field.name.as_str()) {
                errors.push(ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    at(&format!("fields[{index}]")),
                    format!("field `{}` is projected more than once", field.name),
                ));
            }
        }

        errors
    }

    /// Checks this view against the types and entities the system declares.
    ///
    /// Refuses a source entity that does not exist, a projected field the source does not have, a
    /// projected field whose type the source's cannot fill, a filter reading a field that is not on
    /// the source, and a filter comparing a field to a value its type does not have.
    pub fn validate(
        &self,
        types: &TypeRegistry,
        entities: &impl EntityFields,
    ) -> Result<(), ValidationErrors> {
        let mut errors = self.validate_shape();
        let at = |suffix: &str| format!("view.{}.{suffix}", self.name);

        let projected_fields = self.validated_projected_fields(types, &mut errors);

        let Some(source_fields) = entities.entity_fields(&self.source) else {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    at("source"),
                    format!(
                        "`{}` is not a declared entity, so there is nothing for `{}` to project",
                        self.source, self.name
                    ),
                )
                .with_hint(format!(
                    "declared entities: {}",
                    join(entities.entity_names())
                )),
            );
            // Every remaining check reads the source's fields, so reporting them against an entity
            // that does not exist would be noise, not accumulation.
            return errors.into_result(());
        };

        errors.extend(self.validate_projected(projected_fields, source_fields, types));

        for field in &self.params {
            errors.extend(
                types.resolve(&field.type_ref, &at(&format!("params.{}.type", field.name))),
            );
        }
        let mut read_params = BTreeSet::new();
        if let Some(filter) = &self.filter {
            let environment = crate::expression::DomainEnvironment::new(types, source_fields)
                .with_params(&self.params);
            let checked = crate::expression::check_predicate(&environment, filter, &at("filter"));
            for error in &checked.errors {
                let mut diagnostic = error.validation_error();
                if error.code == ValidationCode::UnobservableFact {
                    diagnostic
                        .message
                        .push_str("; a view cannot select on something its source never observes");
                }
                if error.code == ValidationCode::UndeclaredReference
                    && error.path.is_none()
                    && matches!(
                        filter,
                        Predicate::Compare {
                            op: ess_primitives::predicate::CompareOp::Eq,
                            ..
                        }
                    )
                {
                    diagnostic
                        .message
                        .push_str("; this equality selects nothing whatever the system does");
                }
                errors.push(diagnostic);
            }
            read_params = checked.parameters;
        }
        errors.extend(self.validate_params(&read_params));
        errors.extend(self.validate_order(projected_fields));
        errors.extend(self.validate_grouping(types));

        errors.into_result(())
    }

    /// Every projected field against the source: a declared field at a type the source can fill,
    /// or an aggregate, which its own rules check.
    fn validate_projected(
        &self,
        projected: Option<&[Field]>,
        source_fields: &[Field],
        types: &TypeRegistry,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at = |suffix: &str| format!("view.{}.{suffix}", self.name);
        let known = |name: &str| source_fields.iter().find(|field| field.name == name);

        for (index, field) in projected.into_iter().flatten().enumerate() {
            let field_at = if self.shape.is_some() {
                format!("shape.fields[{index}]")
            } else {
                format!("fields[{index}]")
            };
            errors.extend(types.resolve(&field.type_ref, &at(&format!("{field_at}.type"))));
            if let Some(own) = self.validate_aggregate(&field_at, field, types, source_fields) {
                errors.extend(own);
                continue;
            }

            let Some(source_field) = known(&field.name) else {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        at(&field_at),
                        format!(
                            "`{}` has no field `{}`, so `{}` promises an observation nothing \
                             produces",
                            self.source, field.name, self.name
                        ),
                    )
                    .with_hint(format!(
                        "fields of `{}`: {}",
                        self.source,
                        join(source_fields.iter().map(|field| field.name.clone()))
                    )),
                );
                continue;
            };

            if !crate::types::is_assignable(&source_field.type_ref, &field.type_ref) {
                errors.push(ValidationError::new(
                    ValidationCode::TypeMismatch,
                    at(&format!("{field_at}.type")),
                    format!(
                        "`{}` projects `{}` as {}, but `{}` declares it as {}",
                        self.name, field.name, field.type_ref, self.source, source_field.type_ref
                    ),
                ));
            }
        }
        errors
    }

    /// V6–V10: one aggregate field's argument and its declared result type, or `None` where the
    /// field is not an aggregate. An aggregate field is computed, not projected, so these rules
    /// replace the source-field check [`Self::validate`] makes of every other field.
    fn validate_aggregate(
        &self,
        field_at: &str,
        field: &Field,
        types: &TypeRegistry,
        source_fields: &[Field],
    ) -> Option<ValidationErrors> {
        let aggregate = self.aggregation.as_ref()?.function(&field.name)?;
        let mut errors = ValidationErrors::new();
        let at = |suffix: &str| format!("view.{}.{field_at}.{suffix}", self.name);
        let source_type = match &aggregate.input {
            None => None,
            Some(input) if input.contains('.') => {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UnsupportedConstruct,
                        at("aggregate"),
                        format!(
                            "`{}` aggregates `{input}`, a path into a field; an aggregate reads one \
                             top-level field of `{}`",
                            self.name, self.source
                        ),
                    )
                    .with_hint("aggregate a top-level field; struct paths are not in this cut"),
                );
                return Some(errors);
            }
            Some(input) => {
                let Some(source_field) = source_fields.iter().find(|known| &known.name == input)
                else {
                    errors.push(
                        ValidationError::new(
                            ValidationCode::UndeclaredReference,
                            at("aggregate"),
                            format!(
                                "`{}` has no field `{input}`, so `{}.{}` aggregates nothing it \
                                 observes",
                                self.source, self.name, field.name
                            ),
                        )
                        .with_hint(format!(
                            "fields of `{}`: {}",
                            self.source,
                            join(source_fields.iter().map(|field| field.name.clone()))
                        )),
                    );
                    return Some(errors);
                };
                Some(source_field)
            }
        };

        let expected = match source_type {
            None => result_type(aggregate.function, Leaf::Other, &field.type_ref),
            Some(source_field) => {
                let unwrapped = unwrap_chain(&source_field.type_ref, types);
                if unwrapped.optional {
                    errors.push(
                        ValidationError::new(
                            ValidationCode::UnsupportedConstruct,
                            at("aggregate"),
                            format!(
                                "`{}` computes {aggregate} over `{}`, which may be absent ({}); \
                                 an aggregate over an optional field is not in this cut",
                                self.name, source_field.name, source_field.type_ref
                            ),
                        )
                        .with_hint(
                            "aggregate a field every row holds; which rows an absent value is \
                             skipped in cannot be witnessed yet",
                        ),
                    );
                    return Some(errors);
                }
                let Some(expected) =
                    result_type(aggregate.function, unwrapped.leaf, &source_field.type_ref)
                else {
                    errors.push(ValidationError::new(
                        ValidationCode::TypeMismatch,
                        at("aggregate"),
                        format!(
                            "`{}` computes {aggregate}, and `{}` does not admit `{}` of type {}",
                            self.name, aggregate.function, source_field.name, source_field.type_ref
                        ),
                    ));
                    return Some(errors);
                };
                Some(expected)
            }
        };
        if let Some(expected) = expected {
            if field.type_ref != expected {
                errors.push(
                    ValidationError::new(
                        ValidationCode::TypeMismatch,
                        at("type"),
                        format!(
                            "`{}.{}` computes {aggregate}, whose result type is {expected}, and \
                             declares {}",
                            self.name, field.name, field.type_ref
                        ),
                    )
                    .with_hint(format!("declare `type: {expected}`")),
                );
            }
        }
        Some(errors)
    }

    /// V1, V2, V4, V11, V12 and V14: the group keys, the fields that are neither, and the order.
    fn validate_grouping(&self, types: &TypeRegistry) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let Some(aggregation) = &self.aggregation else {
            return errors;
        };
        let at = |suffix: &str| format!("view.{}.{suffix}", self.name);

        errors.extend(self.validate_keys(aggregation, types));

        for (index, field) in self.fields.iter().enumerate() {
            if aggregation.function(&field.name).is_none()
                && !aggregation.group_by.contains(&field.name)
            {
                errors.push(
                    ValidationError::new(
                        ValidationCode::MissingDeclaration,
                        at(&format!("fields[{index}]")),
                        format!(
                            "`{}` is neither an aggregate nor a group key, so a row has no single \
                             value for it",
                            field.name
                        ),
                    )
                    .with_hint(format!(
                        "list `{}` in `group_by:`, give it an `aggregate:`, or drop it",
                        field.name
                    )),
                );
            }
        }

        if !self.order_by.is_empty() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedConstruct,
                    at("order_by"),
                    format!(
                        "`{}` is an aggregate view and declares `order_by:`; ranking aggregate \
                         rows is a window, which is not in this cut",
                        self.name
                    ),
                )
                .with_hint("drop `order_by:`; the consumer ranks the rows it reads"),
            );
        }
        errors
    }

    /// V1, V2, V11 and V12: each group key names a non-aggregate field of an equality type every
    /// row holds.
    fn validate_keys(&self, aggregation: &Aggregation, types: &TypeRegistry) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at = |suffix: &str| format!("view.{}.{suffix}", self.name);
        for (index, key) in aggregation.group_by.iter().enumerate() {
            let key_at = at(&format!("group_by[{index}]"));
            let Some(field) = self.field(key) else {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        key_at,
                        format!(
                            "`{}` groups by `{key}`, which is not one of its fields",
                            self.name
                        ),
                    )
                    .with_hint(format!(
                        "fields of `{}`: {}",
                        self.name,
                        join(self.fields.iter().map(|field| field.name.clone()))
                    )),
                );
                continue;
            };
            if aggregation.function(key).is_some() {
                errors.push(ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    key_at,
                    format!(
                        "`{}` groups by `{key}`, which it also computes as an aggregate; a group \
                         key is a value each row of the group shares",
                        self.name
                    ),
                ));
                continue;
            }
            let unwrapped = unwrap_chain(&field.type_ref, types);
            if unwrapped.optional || unwrapped.leaf.is(Primitive::Timestamp) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UnsupportedConstruct,
                        key_at,
                        format!(
                            "`{}` groups by `{key}` of type {}: {}",
                            self.name,
                            field.type_ref,
                            if unwrapped.optional {
                                "a group key that may be absent is not in this cut"
                            } else {
                                "grouping by a timestamp is time bucketing, which is not in this \
                                 cut"
                            }
                        ),
                    )
                    .with_hint("group by a field every row holds, of an equality type"),
                );
                continue;
            }
            let equality = unwrapped.leaf == Leaf::Enum
                || [
                    Primitive::String,
                    Primitive::Integer,
                    Primitive::Decimal,
                    Primitive::Boolean,
                    Primitive::Uuid,
                ]
                .iter()
                .any(|admitted| unwrapped.leaf.is(*admitted));
            if !equality {
                errors.push(ValidationError::new(
                    ValidationCode::TypeMismatch,
                    key_at,
                    format!(
                        "`{}` groups by `{key}` of type {}, which has no value equality a group \
                         could be keyed on",
                        self.name, field.type_ref
                    ),
                ));
            }
        }
        errors
    }

    /// Checks that every ranking key names a field this view projects, once.
    ///
    /// **Projected, not merely declared on the source.** A rank over a field the view does not
    /// expose is a promise nobody can check: a consumer reading the rows cannot tell a correctly
    /// ordered page from a wrongly ordered one, and neither can a conformance scenario, which is
    /// the reader this rule is written for.
    fn validate_order(&self, projected: Option<&[Field]>) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if self.order_by.is_empty() {
            return errors;
        }
        let at = format!("view.{}.order_by", self.name);

        // `None` means the shape did not resolve, which is already reported; checking keys against
        // fields nobody could read would be a second report of one defect.
        let Some(projected) = projected else {
            return errors;
        };

        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for ranking in &self.order_by {
            if !projected.iter().any(|field| field.name == ranking.field) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        at.clone(),
                        format!(
                            "`{}` ranks by `{}`, which it does not project, so nothing reading \
                             this view can tell whether the order holds",
                            self.name, ranking.field
                        ),
                    )
                    .with_hint(format!(
                        "projected here: {}",
                        join(projected.iter().map(|field| field.name.clone()))
                    )),
                );
                continue;
            }
            if !seen.insert(ranking.field.as_str()) {
                errors.push(ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    at.clone(),
                    format!(
                        "`{}` ranks by `{}` twice; the second key can never decide anything the \
                         first did not",
                        self.name, ranking.field
                    ),
                ));
            }
        }

        errors
    }

    /// The namespace a parameter is read under in a filter.
    pub const PARAM: &'static str = "param";

    /// Checks that declared parameters and the ones a filter reads are the same set.
    ///
    /// Both directions, and each catches a different mistake. A filter reading `param.queue_id`
    /// that `params:` does not declare is a view nobody can call correctly — a generated scenario
    /// would have to invent the value, which is the invention this crate refuses everywhere. A
    /// declared parameter no filter reads is worse than useless: every caller is made to supply it
    /// and nothing selects on it, so two different values return the same rows and the view looks
    /// parameterised to a reader who then trusts it.
    fn validate_params(&self, read: &BTreeSet<String>) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let declared: BTreeSet<&str> = self
            .params
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        let read: BTreeSet<&str> = read.iter().map(String::as_str).collect();

        for name in declared.difference(&read) {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnobservableFact,
                    format!("view.{}.params", self.name),
                    format!(
                        "`{}` declares the parameter `{name}` and no filter reads it, so every \
                         value of it returns the same rows",
                        self.name
                    ),
                )
                .with_hint(format!(
                    "read it in `filter:` as `param.{name}`, or drop it — a parameter nothing \
                     selects on makes the view look narrower than it is"
                )),
            );
        }
        errors
    }
}

/// Renders a list of names for a diagnostic, saying so when the list is empty.
fn join<T: fmt::Display>(items: impl IntoIterator<Item = T>) -> String {
    let rendered: Vec<String> = items.into_iter().map(|item| format!("`{item}`")).collect();
    if rendered.is_empty() {
        "none are declared".to_owned()
    } else {
        rendered.join(", ")
    }
}

/// A view as written in a document, before validation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawViewSpec {
    /// Its stable identity.
    pub name: QualifiedName,
    /// The entity it projects.
    pub source: QualifiedName,
    /// A reusable named struct describing what it exposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<QualifiedName>,
    /// What it exposes inline. A field may declare `aggregate:`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<RawViewField>,
    /// What the caller must supply to ask for this view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Field>,
    /// Which instances it contains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Predicate>,
    /// The view fields the admitted rows are partitioned by. Absent or `[]`: not grouped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_by: Vec<String>,
    /// The order the rows are ranked in, most significant key first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order_by: Vec<Ranking>,
    /// How soon it reflects a command that has already returned. Defaults to `eventual`.
    #[serde(default)]
    pub consistency: Consistency,
    /// What it is called on the wire and shown as.
    #[serde(default, skip_serializing_if = "Naming::is_empty")]
    pub naming: Naming,
}

impl TryFrom<RawViewSpec> for ViewSpec {
    type Error = ValidationErrors;

    fn try_from(raw: RawViewSpec) -> Result<Self, Self::Error> {
        let mut fields = Vec::with_capacity(raw.fields.len());
        let mut functions = BTreeMap::new();
        for written in raw.fields {
            let (field, aggregate) = written.split();
            if let Some(aggregate) = &aggregate {
                functions
                    .entry(field.name.clone())
                    .or_insert_with(|| Aggregate::from(aggregate));
            }
            fields.push(field);
        }
        let mut grouping = ValidationErrors::new();
        let at = |suffix: &str| format!("view.{}.{suffix}", raw.name);
        // V3: a property of the written list, reported before the list is folded into the model.
        let mut seen = BTreeSet::new();
        for (index, key) in raw.group_by.iter().enumerate() {
            if !seen.insert(key.as_str()) {
                grouping.push(ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    at(&format!("group_by[{index}]")),
                    format!("`{}` groups by `{key}` more than once", raw.name),
                ));
            }
        }
        // V5: a `group_by` with no aggregate has no `Aggregation` to be checked later.
        if !raw.group_by.is_empty() && functions.is_empty() {
            grouping.push(
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    at("group_by"),
                    format!(
                        "`{}` groups by {} and no field declares `aggregate:`, so a group has no \
                         value to report",
                        raw.name,
                        join(raw.group_by.iter())
                    ),
                )
                .with_hint("give at least one field an `aggregate:`, or drop `group_by:`"),
            );
        }
        // V13: a shape's struct fields cannot carry `aggregate:`.
        if !raw.group_by.is_empty() && raw.shape.is_some() {
            grouping.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    at("group_by"),
                    format!(
                        "`{}` declares `group_by` beside `shape`; a shape's fields cannot declare \
                         an aggregate",
                        raw.name
                    ),
                )
                .with_hint("declare an aggregate view's fields inline"),
            );
        }
        let aggregation = (!functions.is_empty()).then_some(Aggregation {
            group_by: raw.group_by,
            functions,
        });
        let spec = Self {
            name: raw.name,
            source: raw.source,
            shape: raw.shape,
            fields,
            params: raw.params,
            filter: raw.filter,
            aggregation,
            order_by: raw.order_by,
            consistency: raw.consistency,
            naming: raw.naming,
        };
        let mut errors = spec.validate_shape();
        errors.extend(grouping);
        errors.into_result(spec)
    }
}

impl From<ViewSpec> for RawViewSpec {
    fn from(view: ViewSpec) -> Self {
        let (group_by, functions) = match view.aggregation {
            Some(aggregation) => (aggregation.group_by, aggregation.functions),
            None => (Vec::new(), BTreeMap::new()),
        };
        let fields = view
            .fields
            .into_iter()
            .map(|field| RawViewField {
                aggregate: functions.get(&field.name).map(RawAggregate::from),
                name: field.name,
                type_ref: field.type_ref,
                naming: field.naming,
            })
            .collect();
        Self {
            name: view.name,
            source: view.source,
            shape: view.shape,
            fields,
            params: view.params,
            filter: view.filter,
            group_by,
            order_by: view.order_by,
            consistency: view.consistency,
            naming: view.naming,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::{NamedType, Primitive, TypeBody, TypeRef};

    fn name(value: &str) -> QualifiedName {
        QualifiedName::new(value).expect("a valid name")
    }

    /// The §31 example's types, as `examples/billing/domains/invoice.yaml` declares them, plus the
    /// enum `Invoice`'s lifecycle synthesises — which is what `Specification::validate` puts in the
    /// registry a view is checked against.
    fn registry() -> TypeRegistry {
        let mut registry = TypeRegistry::new();
        for (declared, body) in [
            (
                "billing.invoice.InvoiceId",
                TypeBody::Newtype {
                    alphabet: None,
                    of: TypeRef::Primitive(Primitive::Uuid),
                    invariants: Vec::new(),
                },
            ),
            (
                "billing.invoice.Email",
                TypeBody::Newtype {
                    alphabet: None,
                    of: TypeRef::Primitive(Primitive::String),
                    invariants: Vec::new(),
                },
            ),
            (
                "billing.invoice.Money",
                TypeBody::Struct {
                    fields: vec![
                        Field::new("amount", TypeRef::Primitive(Primitive::Decimal)),
                        Field::new("currency", TypeRef::Primitive(Primitive::String)),
                    ],
                    invariants: Vec::new(),
                },
            ),
            (
                "billing.invoice.InvoiceRow",
                TypeBody::Struct {
                    fields: vec![
                        Field::new(
                            "invoice_id",
                            TypeRef::Named(name("billing.invoice.InvoiceId")),
                        ),
                        Field::new("total", TypeRef::Named(name("billing.invoice.Money"))),
                    ],
                    invariants: Vec::new(),
                },
            ),
            (
                "billing.invoice.Invoice.State",
                TypeBody::Enum {
                    variants: crate::types::EnumVariant::bare([
                        "Draft",
                        "Issued",
                        "Paid",
                        "Cancelled",
                    ]),
                },
            ),
        ] {
            registry
                .insert(NamedType {
                    reading: None,
                    name: name(declared),
                    body,
                    naming: Naming::default(),
                })
                .expect("new");
        }
        registry
    }

    /// `Invoice`'s observable surface, as `EntitySpec::observable_fields` builds it: its identity,
    /// its fields, and its state typed as its own lifecycle.
    fn entities() -> BTreeMap<QualifiedName, Vec<Field>> {
        [(
            name("billing.invoice.Invoice"),
            vec![
                Field::new(
                    "invoice_id",
                    TypeRef::Named(name("billing.invoice.InvoiceId")),
                ),
                Field::new(
                    "customer_email",
                    TypeRef::Named(name("billing.invoice.Email")),
                ),
                Field::new("total", TypeRef::Named(name("billing.invoice.Money"))),
                Field::new(
                    "state",
                    TypeRef::Named(name("billing.invoice.Invoice.State")),
                ),
            ],
        )]
        .into()
    }

    const INVOICE_BY_ID: &str = r"
name: billing.invoice.InvoiceById
source: billing.invoice.Invoice
consistency: eventual
fields:
  - name: invoice_id
    type: billing.invoice.InvoiceId
  - name: total
    type: billing.invoice.Money
";

    const SHAPED_INVOICE_BY_ID: &str = r"
name: billing.invoice.InvoiceById
source: billing.invoice.Invoice
shape: billing.invoice.InvoiceRow
consistency: read_your_writes
";

    fn invoice_by_id() -> ViewSpec {
        let raw: RawViewSpec = serde_yaml::from_str(INVOICE_BY_ID).expect("parses");
        ViewSpec::try_from(raw).expect("a valid view")
    }

    fn shaped_invoice_by_id() -> ViewSpec {
        let raw: RawViewSpec = serde_yaml::from_str(SHAPED_INVOICE_BY_ID).expect("parses");
        ViewSpec::try_from(raw).expect("a shape is a complete view declaration")
    }

    fn refuse(view: &ViewSpec) -> ValidationErrors {
        view.validate(&registry(), &entities())
            .expect_err("expected a refusal")
    }

    fn ordered(order_by: &str) -> Result<ViewSpec, ValidationErrors> {
        let document = format!("{INVOICE_BY_ID}order_by:\n{order_by}");
        let raw: RawViewSpec = serde_yaml::from_str(&document).expect("parses");
        ViewSpec::try_from(raw)
    }

    /// `InvoiceById`, asked about one customer rather than about the whole system.
    fn parameterised(params: &str, filter: &str) -> ViewSpec {
        let document = format!("{INVOICE_BY_ID}params:\n{params}filter: {filter}\n");
        let raw: RawViewSpec = serde_yaml::from_str(&document).expect("parses");
        ViewSpec::try_from(raw).expect("the shape is well formed")
    }

    const ONE_PARAM: &str = "  - name: customer_email\n    type: billing.invoice.Email\n";

    #[test]
    fn a_view_may_be_asked_about_one_value_the_caller_supplies() {
        // Most views are one answer the whole system shares. A parameter is for the view that has
        // no such answer — ACD's position in a queue is *per queue*, and one ranked list of every
        // queued call is a different thing from what the implementation can be asked. Without
        // this, a specification either declared the unparameterised view and generated scenarios
        // nothing could answer, or did not declare it.
        let view = parameterised(ONE_PARAM, "customer_email == param.customer_email");
        assert!(
            view.validate(&registry(), &entities()).is_ok(),
            "{:?}",
            view.validate(&registry(), &entities())
        );
        assert_eq!(
            view.params
                .iter()
                .map(|field| field.name.clone())
                .collect::<Vec<_>>(),
            vec!["customer_email".to_owned()],
            "and it is typed, because a caller binding one has to know what it may send"
        );

        let round_tripped: RawViewSpec =
            serde_yaml::from_str(&serde_yaml::to_string(&view).expect("writes")).expect("re-reads");
        assert_eq!(
            ViewSpec::try_from(round_tripped)
                .expect("valid")
                .params
                .len(),
            1
        );
    }

    #[test]
    fn a_parameter_and_the_filter_that_reads_it_must_be_the_same_set() {
        // Both directions, and each catches a different mistake.

        // A filter reading a parameter nothing declared: no caller can bind it, so a generated
        // scenario would have to invent the value.
        let errors = refuse(&parameterised("", "customer_email == param.customer_email"));
        assert!(
            errors.contains(ValidationCode::UndeclaredReference),
            "{errors}"
        );
        assert!(
            errors.to_string().contains("param.customer_email"),
            "the refusal names the parameter: {errors}"
        );

        // A declared parameter no filter reads is worse than useless: every caller is made to
        // supply it, nothing selects on it, and two different values return the same rows — so the
        // view looks narrower than it is to a reader who then trusts it.
        let errors = refuse(&parameterised(ONE_PARAM, "total >= 0"));
        assert!(
            errors.contains(ValidationCode::UnobservableFact),
            "{errors}"
        );
        assert!(
            errors
                .to_string()
                .contains("every value of it returns the same rows"),
            "{errors}"
        );
    }

    #[test]
    fn a_view_declares_the_order_its_rows_come_back_in() {
        let view = ordered("  - total desc\n  - invoice_id\n").expect("a valid view");
        assert!(view.validate(&registry(), &entities()).is_ok());

        assert_eq!(
            view.order_by,
            vec![
                Ranking {
                    field: "total".to_owned(),
                    direction: Direction::Descending
                },
                Ranking {
                    field: "invoice_id".to_owned(),
                    direction: Direction::Ascending
                },
            ],
            "a key with no direction is ascending, which is what a reader assumes"
        );

        let round_tripped: RawViewSpec =
            serde_yaml::from_str(&serde_yaml::to_string(&view).expect("writes")).expect("re-reads");
        assert_eq!(
            ViewSpec::try_from(round_tripped).expect("valid").order_by,
            view.order_by
        );
    }

    #[test]
    fn a_view_says_nothing_about_order_unless_it_declares_one() {
        assert!(
            invoice_by_id().order_by.is_empty(),
            "most views are sets, and making every one of them declare an order would invent a \
             promise the implementation never made"
        );
    }

    #[test]
    fn ranking_by_a_field_the_view_does_not_project_is_refused() {
        // `state` is a field of the entity and not of this view. Ordering by it would be a promise
        // nothing reading the view could check.
        let view = ordered("  - state desc\n").expect("parses");
        let errors = view
            .validate(&registry(), &entities())
            .expect_err("expected a refusal");
        let refusal = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::UndeclaredReference)
            .unwrap_or_else(|| panic!("expected an undeclared reference, got {errors}"));
        assert!(
            refusal.message.contains("`state`") && refusal.message.contains("does not project"),
            "{}",
            refusal.message
        );
    }

    #[test]
    fn ranking_by_one_field_twice_is_refused() {
        let view = ordered("  - total desc\n  - total asc\n").expect("parses");
        let errors = view
            .validate(&registry(), &entities())
            .expect_err("expected a refusal");
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::DuplicateDeclaration),
            "{errors}"
        );
    }

    #[test]
    fn a_direction_that_is_not_a_direction_is_refused_by_name() {
        // A ranking is refused where it is read, not where the view is validated: `sideways` is
        // not a direction in any view, so there is nothing about this view to check it against.
        let read = |order_by: &str| {
            serde_yaml::from_str::<RawViewSpec>(&format!("{INVOICE_BY_ID}order_by:\n{order_by}"))
                .expect_err("refused")
                .to_string()
        };

        let unknown = read("  - total sideways\n");
        assert!(unknown.contains("sideways"), "{unknown}");

        let extra = read("  - total desc please\n");
        assert!(extra.contains("please"), "{extra}");
    }

    #[test]
    fn a_projection_makes_a_scenario_assert_eventually_rather_than_immediately() {
        let view = invoice_by_id();
        assert!(view.validate(&registry(), &entities()).is_ok());

        assert_eq!(view.consistency, Consistency::Eventual);
        assert_eq!(view.assertion_style(), AssertionStyle::Eventually);
        assert_eq!(
            view.assertion_style().as_str(),
            "eventually",
            "the generated scenario must put this view under `eventually:`, never under `expect:`"
        );
        assert!(
            view.assertion_style().is_retried(),
            "a projection is retried until it catches up; a sleep would test the machine instead"
        );
    }

    #[test]
    fn a_read_your_writes_view_is_asserted_immediately() {
        let mut view = invoice_by_id();
        view.consistency = Consistency::ReadYourWrites;

        assert_eq!(view.assertion_style(), AssertionStyle::Expect);
        assert_eq!(view.assertion_style().as_str(), "expect");
        assert!(!view.assertion_style().is_retried());
        assert_eq!(view.consistency.to_string(), "read_your_writes");
    }

    #[test]
    fn a_view_that_does_not_declare_its_consistency_is_eventual() {
        let raw: RawViewSpec = serde_yaml::from_str(
            r"
name: billing.invoice.InvoiceById
source: billing.invoice.Invoice
fields:
  - name: total
    type: billing.invoice.Money
",
        )
        .expect("parses");
        let view = ViewSpec::try_from(raw).expect("valid");

        assert_eq!(
            view.consistency,
            Consistency::Eventual,
            "the cheap mistake is the default: a wrong `eventual` is slow, a wrong \
             `read_your_writes` is flaky"
        );
        assert_eq!(view.assertion_style(), AssertionStyle::Eventually);
    }

    #[test]
    fn a_view_round_trips_through_yaml() {
        let view = invoice_by_id();
        let rendered = serde_yaml::to_string(&view).expect("serialises");
        let reparsed: RawViewSpec = serde_yaml::from_str(&rendered).expect("re-parses");
        let round_tripped = ViewSpec::try_from(reparsed).expect("still valid");

        assert_eq!(round_tripped, view, "{rendered}");
        assert!(
            rendered.contains("consistency: eventual"),
            "consistency survives the document form, or a generator loses it: {rendered}"
        );
    }

    #[test]
    fn a_named_struct_supplies_the_projected_fields_and_round_trips_as_a_reference() {
        let view = shaped_invoice_by_id();
        let types = registry();

        assert!(view.validate(&types, &entities()).is_ok());
        assert!(
            view.fields.is_empty(),
            "the view does not copy the row fields"
        );
        let fields = view
            .projected_fields(&types)
            .expect("the named shape is a struct");
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["invoice_id", "total"]
        );

        let rendered = serde_yaml::to_string(&view).expect("serialises");
        assert!(
            rendered.contains("shape: billing.invoice.InvoiceRow"),
            "the reusable identity survives: {rendered}"
        );
        assert!(
            !rendered.contains("fields:"),
            "serialisation must not expand the shape back into boilerplate: {rendered}"
        );
        let raw: RawViewSpec = serde_yaml::from_str(&rendered).expect("re-parses");
        assert_eq!(ViewSpec::try_from(raw).expect("still valid"), view);
    }

    #[test]
    fn a_view_cannot_declare_both_a_shape_and_inline_fields() {
        let raw: RawViewSpec = serde_yaml::from_str(
            r"
name: billing.invoice.InvoiceById
source: billing.invoice.Invoice
shape: billing.invoice.InvoiceRow
fields:
  - name: invoice_id
    type: billing.invoice.InvoiceId
",
        )
        .expect("parses");

        let errors = ViewSpec::try_from(raw).expect_err("two row contracts must be refused");
        assert!(errors.contains(ValidationCode::ConflictingDeclaration));
        assert!(errors
            .to_string()
            .contains("both `shape` and inline `fields`"));
    }

    #[test]
    fn a_shape_must_name_a_declared_struct() {
        let mut missing = shaped_invoice_by_id();
        missing.shape = Some(name("billing.invoice.MissingRow"));
        let errors = refuse(&missing);
        assert!(errors.contains(ValidationCode::UndeclaredReference));
        assert!(errors.to_string().contains("has no row shape"));

        let mut scalar = shaped_invoice_by_id();
        scalar.shape = Some(name("billing.invoice.Email"));
        let errors = refuse(&scalar);
        assert!(errors.contains(ValidationCode::TypeMismatch));
        assert!(errors.to_string().contains("is not a struct"));
    }

    #[test]
    fn every_field_in_a_named_shape_is_checked_against_the_source_entity() {
        let mut types = registry();
        types
            .insert(NamedType {
                reading: None,
                name: name("billing.invoice.ImpossibleRow"),
                body: TypeBody::Struct {
                    fields: vec![Field::new(
                        "customer_name",
                        TypeRef::Primitive(Primitive::String),
                    )],
                    invariants: Vec::new(),
                },
                naming: Naming::default(),
            })
            .expect("a new type");
        let mut view = shaped_invoice_by_id();
        view.shape = Some(name("billing.invoice.ImpossibleRow"));

        let errors = view
            .validate(&types, &entities())
            .expect_err("the source cannot supply that row");
        assert!(errors.contains(ValidationCode::UndeclaredReference));
        assert!(errors.to_string().contains("has no field `customer_name`"));
    }

    #[test]
    fn a_view_of_an_entity_that_does_not_exist_is_refused() {
        let mut view = invoice_by_id();
        view.source = name("billing.invoice.Receipt");

        let errors = refuse(&view);
        assert!(errors.contains(ValidationCode::UndeclaredReference));
        let rendered = errors.to_string();
        assert!(
            rendered.contains("`billing.invoice.Receipt` is not a declared entity"),
            "{rendered}"
        );
        assert!(
            rendered.contains("billing.invoice.Invoice"),
            "and what was available: {rendered}"
        );
        assert_eq!(
            errors.len(),
            1,
            "with no source there is nothing to check the fields against: {rendered}"
        );
    }

    #[test]
    fn a_view_field_the_source_does_not_have_is_refused() {
        let mut view = invoice_by_id();
        view.fields.push(Field::new(
            "customer_name",
            TypeRef::Primitive(Primitive::String),
        ));

        let errors = refuse(&view);
        assert!(errors.contains(ValidationCode::UndeclaredReference));
        let rendered = errors.to_string();
        assert!(
            rendered.contains("has no field `customer_name`"),
            "{rendered}"
        );
        assert!(
            rendered.contains("promises an observation nothing produces"),
            "{rendered}"
        );
        assert!(
            rendered.contains("`total`"),
            "and what the entity does have: {rendered}"
        );
    }

    #[test]
    fn a_view_field_whose_type_disagrees_with_the_entity_is_refused() {
        let mut view = invoice_by_id();
        view.fields = vec![Field::new("total", TypeRef::Primitive(Primitive::Decimal))];

        let errors = refuse(&view);
        assert!(errors.contains(ValidationCode::TypeMismatch));
        assert!(
            errors
                .to_string()
                .contains("projects `total` as Decimal, but `billing.invoice.Invoice` declares it as billing.invoice.Money"),
            "the refusal must name both types: {errors}"
        );
    }

    #[test]
    fn a_view_field_may_widen_a_required_entity_field_to_an_optional_one() {
        let mut view = invoice_by_id();
        view.fields = vec![Field::new(
            "total",
            TypeRef::Optional(Box::new(TypeRef::Named(name("billing.invoice.Money")))),
        )];

        assert!(
            view.validate(&registry(), &entities()).is_ok(),
            "a value that is always present can fill a slot that may be absent"
        );
    }

    #[test]
    fn a_filter_reading_a_field_the_source_does_not_have_is_refused() {
        let mut view = invoice_by_id();
        view.filter = Some(Predicate::parse_expression("balance > 0").expect("parses"));

        let errors = refuse(&view);
        assert!(errors.contains(ValidationCode::UnobservableFact));
        let rendered = errors.to_string();
        assert!(rendered.contains("reads `balance`"), "{rendered}");
        assert!(
            rendered.contains("cannot select on something its source never observes"),
            "{rendered}"
        );
    }

    #[test]
    fn a_filter_may_read_a_field_the_view_does_not_project() {
        let mut view = invoice_by_id();
        view.filter = Some(Predicate::parse_expression("state == Issued").expect("parses"));

        assert!(
            view.field("state").is_none(),
            "the fixture projects the total, not the state"
        );
        assert!(
            view.validate(&registry(), &entities()).is_ok(),
            "`OutstandingInvoices` selects on the state without exposing it, which is normal"
        );
    }

    #[test]
    fn a_filter_comparing_a_state_to_a_name_the_lifecycle_does_not_have_is_refused() {
        let mut view = invoice_by_id();
        view.filter = Some(Predicate::parse_expression("state == Issed").expect("parses"));

        let errors = refuse(&view);
        assert!(errors.contains(ValidationCode::UndeclaredReference));
        let rendered = errors.to_string();
        assert!(
            rendered.contains("compares `state` to `Issed`"),
            "{rendered}"
        );
        assert!(
            rendered.contains("selects nothing whatever the system does"),
            "a filter nothing can satisfy generates a scenario nothing can fail: {rendered}"
        );
        assert!(
            rendered.contains("`Issued`"),
            "and the name that was meant: {rendered}"
        );
    }

    #[test]
    fn every_branch_of_a_composite_filter_is_checked_against_the_lifecycle() {
        let mut view = invoice_by_id();
        view.filter = Some(Predicate::any(vec![
            Predicate::parse_expression("state == Issued").expect("parses"),
            Predicate::parse_expression("state == Overdue").expect("parses"),
        ]));

        let errors = refuse(&view);
        assert!(errors.contains(ValidationCode::UndeclaredReference));
        assert_eq!(
            errors.len(),
            1,
            "the declared state is accepted and only the invented one is reported: {errors}"
        );
        assert!(errors.to_string().contains("`Overdue`"), "{errors}");
    }

    #[test]
    fn a_filter_on_a_field_whose_type_lists_no_values_is_left_to_the_ir() {
        let mut view = invoice_by_id();
        view.filter = Some(
            Predicate::parse_expression(r#"customer_email == "someone@example.com""#)
                .expect("parses"),
        );

        assert!(
            view.validate(&registry(), &entities()).is_ok(),
            "a newtype over a string has no set of names to check a comparison against"
        );
    }

    #[test]
    fn a_view_that_projects_nothing_is_refused() {
        let raw: RawViewSpec = serde_yaml::from_str(
            r"
name: billing.invoice.InvoiceById
source: billing.invoice.Invoice
",
        )
        .expect("parses");

        let errors = ViewSpec::try_from(raw).expect_err("a view with no fields");
        assert!(errors.contains(ValidationCode::EmptyDeclaration));
        assert!(
            errors
                .to_string()
                .contains("declares neither `shape` nor `fields`"),
            "{errors}"
        );
    }

    #[test]
    fn a_field_projected_twice_is_refused() {
        let raw: RawViewSpec = serde_yaml::from_str(
            r"
name: billing.invoice.InvoiceById
source: billing.invoice.Invoice
fields:
  - name: total
    type: billing.invoice.Money
  - name: total
    type: billing.invoice.Money
",
        )
        .expect("parses");

        let errors = ViewSpec::try_from(raw).expect_err("a field projected twice");
        assert!(errors.contains(ValidationCode::DuplicateDeclaration));
        assert!(
            errors
                .to_string()
                .contains("`total` is projected more than once"),
            "{errors}"
        );
    }

    #[test]
    fn a_projected_field_must_name_a_declared_type() {
        let mut view = invoice_by_id();
        view.fields = vec![Field::new(
            "total",
            TypeRef::Named(name("billing.invoice.Amount")),
        )];

        let errors = refuse(&view);
        assert!(errors.contains(ValidationCode::UndeclaredReference));
        assert!(
            errors
                .to_string()
                .contains("`billing.invoice.Amount` is not a declared type"),
            "{errors}"
        );
    }
}
