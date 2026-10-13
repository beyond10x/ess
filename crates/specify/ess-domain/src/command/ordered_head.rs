//! The head of an ordered selection as a value source, `{related: {entity, where, order_by,
//! first}}` (source format `ess/25`, beyond10x/ess#543, `docs/design/ordered-related-head.md`).
//!
//! ```yaml
//! element_key:
//!   related:
//!     entity: demo.ranked.Membership
//!     where: {all: [slot_id == input.slot_id, state == Present]}
//!     order_by: [element_score desc, membership_id]
//!     first: element_key
//! ```
//!
//! `first` is read from the first row of the rows `where` selects, ranked by `order_by`. The
//! selector is a filtered read's ([`super::row_set`]); a ranking key is a view's
//! ([`crate::view::Ranking`]), a bare field of the entity, its identity or `state`.
//!
//! **A total order.** The head must be the same row for every evaluator, and the only key ESS can
//! show to be unique is the entity's identity, so the last key is the identity and no key follows
//! it. A struct identity is unique by construction and ranks by its members in declaration order,
//! each by its own type's order.
//!
//! **Declared-unordered ties.** `ties: unordered` beside the keys states that rows tying on every
//! key are in no order the model fixes: the order need not end in the identity, and must not hold
//! it. A head that ties is then the implementation's choice, which no evaluator answers for it.
//!
//! **Recognition.** The shape is recognised by its exact shape, and its list-valued `order_by:` is
//! what makes that safe: a list under a nested mapping's key was refused at every format, so no
//! document any release accepted holds it. A text-valued `order_by:` is not the shape, and the
//! mapping stays the nested one it always was. A shape whose `where:` the nested reading refuses is
//! recognised too, with what is wrong with it, so its refusal can say what was meant.

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::Predicate;

use super::{PayloadSource, RawPayloadSource};
use crate::entity::EntitySpec;
use crate::name::QualifiedName;
use crate::types::{Primitive, TypeBody, TypeRef, TypeRegistry};
use crate::view::Ranking;

/// The key a ranking is written under.
pub const ORDER_BY: &str = "order_by";
/// The key naming the field read from the head.
pub const FIRST: &str = "first";
/// The key declaring how rows tying on every key rank.
pub const TIES: &str = "ties";
/// The one value [`TIES`] takes.
pub const UNORDERED: &str = "unordered";

/// How rows that tie on every ranking key rank among themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Ties {
    /// The order is total: its last key is the identity, so no two rows tie.
    #[default]
    Total,
    /// `ties: unordered`: rows tying on every key are in no order the model fixes, and a head that
    /// ties is the implementation's choice.
    Unordered,
}

impl Ties {
    /// Whether the order is total, as an order without `ties:` is.
    #[must_use]
    pub fn is_total(&self) -> bool {
        *self == Self::Total
    }
}

/// What is wrong with a recognised shape, where something is: the refusals that need no
/// declaration in hand, decided from the keys alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conflict {
    /// `first:` written beside `field:`.
    FirstBesideField,
    /// An order read with `field:` and no `first:`.
    OrderWithField,
    /// An order read with neither `first:` nor `field:`.
    OrderWithoutFirst,
    /// `first:` written with no list-valued `order_by:`.
    FirstWithoutOrder,
}

/// The source as written, where the document wrote it: what the reading found wrong with it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Written {
    /// Why the `where:` value is not a predicate, where it is not.
    pub(super) predicate: Option<String>,
    /// Each key that did not parse, by position, with the reader's message.
    pub(super) keys: Vec<(usize, String)>,
    /// The `ties:` value written, where it is not [`UNORDERED`].
    pub(super) ties: Option<String>,
    /// What the keys say is wrong with the shape.
    pub(super) conflict: Option<Conflict>,
}

/// `{related: {entity, where, order_by, first}}` as the document writes it.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawRelatedHead {
    related: RawHead,
}

/// What `related:` holds in a head read.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RawHead {
    /// The entity whose rows are candidates.
    entity: QualifiedName,
    /// What a candidate must satisfy: its fields bare, the input under `input.`, the addressed
    /// subject under `subject.`.
    #[serde(rename = "where")]
    filter: Predicate,
    /// The ranking keys, most significant first: `<field>` or `<field> asc|desc`, ending in the
    /// entity's identity.
    #[schemars(length(min = 1))]
    order_by: Vec<String>,
    /// `unordered` where rows tying on every key are in no order the model fixes; the order then
    /// need not end in the identity, and must not hold it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^unordered$"))]
    ties: Option<String>,
    /// The field of the head row read.
    first: String,
    /// How the document wrote `where`, where it was read from one.
    #[serde(skip)]
    #[schemars(skip)]
    written: Option<Box<serde_yaml::Value>>,
    /// What the keys say is wrong with the shape.
    #[serde(skip)]
    #[schemars(skip)]
    conflict: Option<Conflict>,
}

/// The value of `key` among `entries`.
fn entry<'a>(entries: &'a [(String, RawPayloadSource)], key: &str) -> Option<&'a RawPayloadSource> {
    entries
        .iter()
        .find_map(|(name, value)| (name == key).then_some(value))
}

/// Whether the entries of a `related:` mapping are a head read's, as written or with a conflict
/// this module refuses by name: a qualified `entity`, a kept `where`, and a list-valued `order_by`
/// beside `first`, `field` or both; or `first` with no list, where the nested reading of `where`
/// refuses it anyway.
pub(super) fn selects(entries: &[(String, RawPayloadSource)]) -> bool {
    let selector = matches!(entry(entries, "entity"), Some(RawPayloadSource::Text(entity))
        if entity.parse::<QualifiedName>().is_ok())
        && matches!(entry(entries, "where"), Some(RawPayloadSource::Captured(_)));
    if !selector {
        return false;
    }
    let known = entries.iter().all(|(key, value)| match key.as_str() {
        "entity" | "where" => true,
        ORDER_BY => matches!(
            value,
            RawPayloadSource::Texts(_) | RawPayloadSource::Text(_)
        ),
        FIRST | "field" | TIES => matches!(value, RawPayloadSource::Text(_)),
        _ => false,
    });
    if !known {
        return false;
    }
    match entry(entries, ORDER_BY) {
        Some(RawPayloadSource::Texts(_)) => true,
        // No list: a nested mapping, unless its `where:` was never readable as one.
        _ => {
            entry(entries, FIRST).is_some()
                && entry(entries, "field").is_none()
                && matches!(entry(entries, "where"), Some(RawPayloadSource::Captured(value))
                    if super::legacy_reading(value)
                        .and_then(|raw| PayloadSource::try_from(raw).map_err(str::to_owned))
                        .is_err())
        }
    }
}

impl RawRelatedHead {
    /// The mapping `{related: …}` whose entries [`selects`] admits, taken out of `entries`.
    pub(super) fn recognise(entries: &mut Vec<(String, RawPayloadSource)>) -> Option<Self> {
        let [(key, RawPayloadSource::Nested(super::RawNestedSources(inner)))] = entries.as_slice()
        else {
            return None;
        };
        if key != "related" || !selects(inner) {
            return None;
        }
        let Some((_, RawPayloadSource::Nested(super::RawNestedSources(inner)))) = entries.pop()
        else {
            return None;
        };
        let (mut entity, mut written, mut order_by, mut first, mut field, mut ties) =
            (None, None, None, None, None, None);
        for (key, value) in inner {
            match (key.as_str(), value) {
                ("entity", RawPayloadSource::Text(text)) => entity = text.parse().ok(),
                ("where", RawPayloadSource::Captured(value)) => written = Some(value),
                (ORDER_BY, RawPayloadSource::Texts(keys)) => order_by = Some(keys),
                (FIRST, RawPayloadSource::Text(text)) => first = Some(text),
                ("field", RawPayloadSource::Text(text)) => field = Some(text),
                (TIES, RawPayloadSource::Text(text)) => ties = Some(text),
                _ => {}
            }
        }
        let conflict = match (&order_by, &first, &field) {
            (Some(_), Some(_), Some(_)) => Some(Conflict::FirstBesideField),
            (Some(_), None, Some(_)) => Some(Conflict::OrderWithField),
            (Some(_), None, None) => Some(Conflict::OrderWithoutFirst),
            (None, _, _) => Some(Conflict::FirstWithoutOrder),
            (Some(_), Some(_), None) => None,
        };
        Some(Self {
            related: RawHead {
                entity: entity?,
                filter: Predicate::Always,
                order_by: order_by.unwrap_or_default(),
                ties,
                first: first.or(field).unwrap_or_default(),
                written: Some(written?),
                conflict,
            },
        })
    }

    /// The written form of a head read.
    pub(super) fn of(
        selection: &super::row_set::RowSelection,
        order_by: &[Ranking],
        ties: Ties,
        first: &str,
    ) -> Self {
        Self {
            related: RawHead {
                entity: selection.entity.clone(),
                filter: selection.filter.clone(),
                order_by: order_by.iter().map(ToString::to_string).collect(),
                ties: (ties == Ties::Unordered).then(|| UNORDERED.to_owned()),
                first: first.to_owned(),
                written: None,
                conflict: None,
            },
        }
    }

    /// The source it is: the selector parsed as a predicate and each key as a ranking, with what
    /// did not parse kept for validation to refuse where it knows the format.
    pub(super) fn into_source(self) -> PayloadSource {
        use serde::Deserialize as _;
        let RawHead {
            entity,
            filter,
            order_by,
            ties,
            first,
            written,
            conflict,
        } = self.related;
        let (filter, predicate) = match written {
            Some(written) => match Predicate::deserialize(*written) {
                Ok(predicate) => (predicate, None),
                Err(error) => (Predicate::Always, Some(error.to_string())),
            },
            None => (filter, None),
        };
        let mut rankings = Vec::new();
        let mut keys = Vec::new();
        for (index, key) in order_by.iter().enumerate() {
            match Ranking::parse(key) {
                Ok(ranking) => rankings.push(ranking),
                Err(error) => keys.push((index, error.to_string())),
            }
        }
        let (ties, unread) = match ties {
            None => (Ties::Total, None),
            Some(word) if word == UNORDERED => (Ties::Unordered, None),
            Some(word) => (Ties::Total, Some(word)),
        };
        PayloadSource::RelatedHead {
            selection: super::row_set::RowSelection { entity, filter },
            order_by: rankings,
            ties,
            first,
            written: Written {
                predicate,
                keys,
                ties: unread,
                conflict,
            },
        }
    }
}

/// The refusal of a `related:` mapping that writes a list-valued `order_by:` beside `via:`: the
/// identity-addressed read reads one row, where an order means nothing.
pub(super) fn beside_via(entries: &[(String, RawPayloadSource)]) -> Option<String> {
    let [(key, RawPayloadSource::Nested(super::RawNestedSources(inner)))] = entries else {
        return None;
    };
    (key == "related"
        && entry(inner, "via").is_some()
        && matches!(entry(inner, ORDER_BY), Some(RawPayloadSource::Texts(_))))
    .then(|| {
        "`order_by:` ranks the rows a selector selects, `{related: {entity, where, order_by, \
         first}}` (format ess/25); `via:` reads the one row an identity names, \
         `{related: {via, field}}`, where an order decides nothing; write one of the two"
            .to_owned()
    })
}

/// The refusal of a shape [`RawRelatedHead::recognise`] found wrong by its keys, at the source
/// site `at`.
pub(super) fn conflict(at: &ConstructRef, conflict: &Conflict) -> ValidationError {
    match conflict {
        Conflict::FirstBesideField => ValidationError::at(
            at.clone(),
            ValidationCode::ConflictingDeclaration,
            "`{related: …}` writes `first:` beside `field:`; `field:` reads the one row a \
             selector selects, and `first:` the head of the rows it ranks",
        )
        .with_hint("write one: `first:` with `order_by:`, or `field:` without an order"),
        Conflict::OrderWithField => ValidationError::at(
            at.clone(),
            ValidationCode::ConflictingDeclaration,
            "`{related: …}` ranks its rows with `order_by:` and reads them with `field:`, which \
             reads exactly one row; an ordered selection is read with `first:`",
        )
        .with_hint("write `first: <field>` in place of `field:`"),
        Conflict::OrderWithoutFirst => ValidationError::at(
            at.clone(),
            ValidationCode::MissingDeclaration,
            "`{related: …}` ranks its rows with `order_by:` and reads no field; an ordered \
             selection is read with `first:`",
        )
        .with_hint("write `first: <field>`, the field read from the head"),
        Conflict::FirstWithoutOrder => first_without_order(at),
    }
}

/// `first:` with no order: the first of an unordered selection is no particular row.
pub(super) fn first_without_order(at: &ConstructRef) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::MissingDeclaration,
        "`{related: …}` reads `first:` with no `order_by:` list; the first of an unordered \
         selection is no particular row",
    )
    .with_hint(
        "write `order_by: [<field> desc, <identity>]` (format ess/25), or read the one selected \
         row with `field:`",
    )
}

/// The refusal of the shape below `ess/25`.
pub(super) fn below_ess_25(at: &ConstructRef) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::UnsupportedFormatVersion,
        "the head of an ordered selection — `{related: {entity, where, order_by, first}}` — \
         requires specification format ess/25",
    )
    .with_hint("declare `format: ess/25`")
}

/// Why a nested mapping refused over a field that is no struct is a head read written without its
/// list: `{related: {entity, where, first}}`, with `order_by:` absent or a text.
pub(super) fn misread_mapping(
    at: &ConstructRef,
    fields: &[super::PayloadField],
) -> Option<ValidationError> {
    let [written] = fields else {
        return None;
    };
    let PayloadSource::Struct { fields: inner } = &written.source else {
        return None;
    };
    let has = |key: &str| inner.iter().any(|leaf| leaf.target == key);
    (written.target == "related"
        && has(FIRST)
        && has("entity")
        && has("where")
        && !has("via")
        && !has("field"))
    .then(|| first_without_order(at))
}

/// Every rule of the order over `entity`'s rows: every key is a bare field of the candidate row,
/// written once, required and of an orderable type, a struct identity through every member; under
/// a total order the identity is the last key, and under `ties: unordered` no key is the identity.
/// `at` is the source site; a key's refusal is reported at `related.order_by[<i>]`, the flag's at
/// `related.ties`.
pub(super) fn check_order(
    entity: &EntitySpec,
    types: &TypeRegistry,
    order_by: &[Ranking],
    ties: Ties,
    at: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let order = at.clone().key("related").key(ORDER_BY);
    if order_by.is_empty() {
        errors.push(
            ValidationError::at(
                order,
                ValidationCode::EmptyDeclaration,
                "`order_by: []` ranks nothing, so no row is the first",
            )
            .with_hint(format!(
                "write the ranking keys, ending in `{}`",
                entity.identity.name
            )),
        );
        return errors;
    }
    let identity = &entity.identity.name;
    let fields = super::row_set::candidate_fields(entity);
    let mut seen = std::collections::BTreeSet::new();
    for (index, ranking) in order_by.iter().enumerate() {
        if let Some(refusal) = check_key(
            entity,
            types,
            &fields,
            ranking,
            order.clone().index(index),
            &mut seen,
        ) {
            errors.push(refusal);
            continue;
        }
        if ranking.field == *identity && ties == Ties::Unordered {
            errors.push(
                ValidationError::at(
                    at.clone().key("related").key(TIES),
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "`ties: unordered` leaves rows that tie on every key in no order, and the \
                         order ranks by the identity `{identity}`, which no two rows share, so no \
                         two rows tie"
                    ),
                )
                .with_hint(format!(
                    "drop `{identity}` from the order, or drop `ties: unordered` for a total order \
                     ending in `{identity}`"
                )),
            );
            continue;
        }
        if ranking.field == *identity && index + 1 < order_by.len() {
            errors.push(
                ValidationError::at(
                    order.clone().index(index + 1),
                    ValidationCode::UnsupportedConstruct,
                    format!(
                        "the order ranks by `{}` after the identity `{identity}`; keys after the \
                         identity never decide, because no two rows share it",
                        order_by[index + 1].field
                    ),
                )
                .with_hint(format!("end the order at `{identity}`")),
            );
        }
    }
    if !errors.is_empty() {
        return errors;
    }
    if ties == Ties::Total && order_by.last().is_some_and(|last| last.field != *identity) {
        errors.push(
            ValidationError::at(
                order,
                ValidationCode::MissingDeclaration,
                format!(
                    "the order is not total: rows that tie on every key leave no first, and only \
                     the identity `{identity}` is unique among the rows of `{}`; append \
                     `{identity}` as the last key",
                    entity.name
                ),
            )
            .with_hint(format!(
                "write `order_by: [{}, {identity}]`, or declare `ties: unordered` where the \
                 implementation breaks ties its own way",
                keys(order_by)
            )),
        );
    }
    errors
}

/// What the reading found wrong with the shape as written, at the source site `at`: the keys'
/// conflict, a `where:` that is no predicate, a `ties:` value other than `unordered`, or keys that
/// do not parse. Empty where nothing was.
pub(super) fn written_refusals(at: &ConstructRef, written: &Written) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if let Some(conflict) = &written.conflict {
        errors.push(self::conflict(at, conflict));
    } else if let Some(why) = &written.predicate {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::TypeMismatch,
            format!(
                "the `where:` of `{{related: {{entity, where, order_by, first}}}}` is no \
                 predicate: {why}"
            ),
        ));
    } else if let Some(word) = &written.ties {
        errors.push(unread_ties(at, word));
    } else {
        for (index, why) in &written.keys {
            errors.push(
                ValidationError::at(
                    at.clone().key("related").key(ORDER_BY).index(*index),
                    ValidationCode::TypeMismatch,
                    why.clone(),
                )
                .with_hint("write a ranking key as `<field>` or `<field> asc|desc`"),
            );
        }
    }
    errors
}

/// The refusal of a `ties:` value other than `unordered`, at the source site `at`.
pub(super) fn unread_ties(at: &ConstructRef, written: &str) -> ValidationError {
    ValidationError::at(
        at.clone().key("related").key(TIES),
        ValidationCode::TypeMismatch,
        format!(
            "`ties: {written}` is no declaration of how tying rows rank; `ties:` takes \
             `unordered`, which leaves rows tying on every key in no order the model fixes"
        ),
    )
    .with_hint(
        "write `ties: unordered`, or leave `ties:` out for a total order ending in the identity",
    )
}

/// One key's own rules, at `key`: a bare field of the candidate row, written once, required and
/// of an orderable type. `seen` holds the keys before it.
fn check_key<'f>(
    entity: &EntitySpec,
    types: &TypeRegistry,
    fields: &[crate::types::Field],
    ranking: &'f Ranking,
    key: ConstructRef,
    seen: &mut std::collections::BTreeSet<&'f str>,
) -> Option<ValidationError> {
    let Some(field) = fields.iter().find(|field| field.name == ranking.field) else {
        return Some(
            ValidationError::at(
                key,
                ValidationCode::UndeclaredReference,
                format!(
                    "`{}` is no field of `{}`; a ranking key is a bare field of the candidate row",
                    ranking.field, entity.name
                ),
            )
            .with_hint(format!(
                "rank by one of: {}",
                fields
                    .iter()
                    .map(|field| format!("`{}`", field.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        );
    };
    if !seen.insert(ranking.field.as_str()) {
        return Some(ValidationError::at(
            key,
            ValidationCode::DuplicateDeclaration,
            format!(
                "the order ranks by `{}` twice; the second key can never decide anything the \
                 first did not",
                ranking.field
            ),
        ));
    }
    // A struct identity is unique by construction, and ranks by its members in declaration order.
    if ranking.field == entity.identity.name && struct_members(types, &field.type_ref).is_some() {
        return unranked_member(types, &field.type_ref, &ranking.field, 0)
            .map(|(path, refusal, type_ref)| refusal.at(key, &path, &type_ref));
    }
    unorderable(types, &field.type_ref)
        .map(|refusal| refusal.at(key, &ranking.field, &field.type_ref))
}

/// The members of `type_ref` where it is a struct, through newtypes but never through
/// `Optional<…>`.
fn struct_members<'t>(
    types: &'t TypeRegistry,
    type_ref: &'t TypeRef,
) -> Option<&'t [crate::types::Field]> {
    let mut current = type_ref;
    for _ in 0..=types.len() {
        let TypeRef::Named(name) = current else {
            return None;
        };
        match &types.get(name)?.body {
            TypeBody::Newtype { of, .. } => current = of,
            TypeBody::Struct { fields, .. } => return Some(fields),
            TypeBody::Enum { .. } | TypeBody::Union { .. } => return None,
        }
    }
    None
}

/// The first member of the struct `type_ref`, at `path`, that has no rank, with why and its type:
/// each member ranks by its own type's order, a struct member by its own members.
fn unranked_member(
    types: &TypeRegistry,
    type_ref: &TypeRef,
    path: &str,
    depth: usize,
) -> Option<(String, Unorderable, TypeRef)> {
    let members = struct_members(types, type_ref)?;
    if depth > types.len() {
        return Some((path.to_owned(), Unorderable::Kind, type_ref.clone()));
    }
    members.iter().find_map(|member| {
        let at = format!("{path}.{}", member.name);
        if struct_members(types, &member.type_ref).is_some() {
            return unranked_member(types, &member.type_ref, &at, depth + 1);
        }
        unorderable(types, &member.type_ref).map(|why| (at, why, member.type_ref.clone()))
    })
}

/// The keys as written, joined.
fn keys(order_by: &[Ranking]) -> String {
    order_by
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Why a key's type has no rank.
enum Unorderable {
    /// `Optional<…>`: absence has no rank.
    Optional,
    /// A type whose values are not ranked.
    Kind,
}

impl Unorderable {
    fn at(&self, key: ConstructRef, field: &str, type_ref: &TypeRef) -> ValidationError {
        match self {
            Self::Optional => ValidationError::at(
                key,
                ValidationCode::TypeMismatch,
                format!(
                    "`{field}` is `{type_ref}`, and absence has no rank, so a row without it has \
                     no place in the order"
                ),
            )
            .with_hint("rank by a required field"),
            Self::Kind => ValidationError::at(
                key,
                ValidationCode::TypeMismatch,
                format!("`{field}` is `{type_ref}`, whose values have no rank"),
            )
            .with_hint(
                "rank by an `Integer`, `Decimal`, `Binary64`, `String`, `Uuid`, `Boolean`, \
                 `Timestamp` or enum field, or `state`",
            ),
        }
    }
}

/// Why `type_ref` has no rank, through newtypes; `None` where it ranks as a view's key does.
fn unorderable(types: &TypeRegistry, type_ref: &TypeRef) -> Option<Unorderable> {
    let mut current = type_ref;
    for _ in 0..=types.len() {
        match current {
            TypeRef::Optional(_) => return Some(Unorderable::Optional),
            TypeRef::List(_) | TypeRef::Map(..) => return Some(Unorderable::Kind),
            TypeRef::Primitive(primitive) => {
                return match primitive {
                    Primitive::Integer
                    | Primitive::Decimal
                    | Primitive::Binary64
                    | Primitive::String
                    | Primitive::Uuid
                    | Primitive::Boolean
                    | Primitive::Timestamp => None,
                    Primitive::Duration | Primitive::Bytes | Primitive::Json => {
                        Some(Unorderable::Kind)
                    }
                };
            }
            TypeRef::Named(name) => match types.get(name).map(|named| &named.body) {
                Some(TypeBody::Newtype { of, .. }) => current = of,
                Some(TypeBody::Enum { .. }) => return None,
                Some(TypeBody::Struct { .. } | TypeBody::Union { .. }) | None => {
                    return Some(Unorderable::Kind);
                }
            },
        }
    }
    Some(Unorderable::Kind)
}
