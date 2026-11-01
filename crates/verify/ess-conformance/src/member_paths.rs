//! A claim about one member of a nested value, named by its path
//! (story:an-authored-claim-may-compare-a-nested-field,
//! story:an-authored-event-claim-names-a-struct-leaf).
//!
//! A nested value in a claim is compared whole: `checkpoint: {checkpoint_id: cp-1}` says the
//! checkpoint has that id **and no other member**, so a checkpoint carrying its format and its call
//! as well fails, and the authored literal is refused for the members it leaves out. From
//! `ess-scenario/6` a claim may name one member alone by its dotted path instead,
//! `checkpoint.checkpoint_id: cp-1`, in an act's `response:`, an event's `payload:`, an error's
//! `fields:` and a row's `contains`, `excludes` and `at`. Each path is compared alone; the members
//! it does not name are not read. `Optional` and newtypes along the path are transparent, as they
//! are to a fact path; a path through a list, a map, a union, an enum or a primitive names no
//! member and is refused by name, as is one naming no declared member. The values a command is
//! sent, an arrangement and a caller are sent whole, so a path there is refused by name too.
//!
//! An event payload, an error's fields and a row are compared at suite/26 dotted keys already
//! ([`crate::leaf_payloads`]): every released reader that admits suite/26 reads such a key as a
//! path, so those claims keep their suite format. A path ending at a struct is written as the
//! struct's leaves, the keys the payload shape names.
//!
//! A response observation's `expected` member had no paths: an older reader looks for a returned
//! field named `checkpoint.checkpoint_id`, finds none, and fails a conforming target. A suite that
//! carries one is therefore ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], cumulative
//! over every pair below, and every reader refuses it under a lower major before target effects. A
//! suite without one keeps its format and bytes.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedField, ResolvedTypeRef};
use ess_domain::types::MAX_TYPE_DEPTH;
use ess_domain::{Field, QualifiedName, TypeRef};
use ess_primitives::node::Node;

use crate::admission::AdmissionError;
use crate::scenario::ScenarioStep;
use crate::selection::Declaration;
use crate::ConformanceSuite;

/// The ordinary suite major whose response observations may name a member by its path.
pub const ORDINARY: u32 = 62;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 63;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What a refusal under a lower major says.
pub const REQUIRES: &str = "a response member path requires suite/62 or /63";

/// The first scenario format whose claims may name a member by its path.
pub const SCENARIO_FORMAT: &str = "ess-scenario/6";

/// Whether `key` is a member path rather than a field name: a field name holds no dot.
pub fn is_path(key: &str) -> bool {
    key.contains('.')
}

/// Whether a response observation of the suite names a member by its path.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::ExpectDirectResponse { response }
                if response.expected.keys().any(|key| is_path(key)))
        })
    })
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

/// Refuses a response member path under a major below [`ORDINARY`].
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}

/// The declared response member `path` names, through the observation's own declarations.
pub(crate) fn wire_member(
    fields: &[Field],
    declarations: &BTreeMap<QualifiedName, Declaration>,
    path: &str,
) -> Result<Field, String> {
    let mut segments = path.split('.');
    let first = segments.next().unwrap_or_default();
    let mut member = fields
        .iter()
        .find(|field| field.name == first)
        .ok_or_else(|| format!("response path {path} names no declared member"))?
        .clone();
    let mut at = first.to_owned();
    for segment in segments {
        let members = wire_struct(&member.type_ref, declarations, 0).ok_or_else(|| {
            format!(
                "response path {path} goes through {at}, a {}, which has no members",
                member.type_ref
            )
        })?;
        member = members
            .iter()
            .find(|field| field.name == segment)
            .ok_or_else(|| format!("response path {path} names no declared member"))?
            .clone();
        at.push('.');
        at.push_str(segment);
    }
    Ok(member)
}

/// The members of the struct `ty` is, through `Optional` and newtypes; `None` for anything else.
fn wire_struct<'a>(
    ty: &TypeRef,
    declarations: &'a BTreeMap<QualifiedName, Declaration>,
    depth: usize,
) -> Option<&'a [Field]> {
    if depth > MAX_TYPE_DEPTH {
        return None;
    }
    match ty {
        TypeRef::Optional(of) => wire_struct(of, declarations, depth + 1),
        TypeRef::Named(name) => match declarations.get(name)? {
            Declaration::Newtype { of } => wire_struct(of, declarations, depth + 1),
            Declaration::Struct { fields } => Some(fields),
            Declaration::Enum { .. } | Declaration::Union { .. } => None,
        },
        TypeRef::Primitive(_) | TypeRef::List(_) | TypeRef::Map(..) => None,
    }
}

/// The value at `path` in an actual response; `None` where a member along it is absent or null,
/// or the value there is no object.
pub(crate) fn reach<'a>(actual: &'a BTreeMap<String, Node>, path: &str) -> Option<&'a Node> {
    let mut segments = path.split('.');
    let mut here = actual.get(segments.next()?)?;
    for segment in segments {
        let Node::Map(members) = here else {
            return None;
        };
        here = members.get(segment)?;
    }
    (!matches!(here, Node::Null)).then_some(here)
}

/// Why a written path reaches no member of the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Unreached {
    /// A segment names no declared member, or is empty.
    NoMember,
    /// The value at `at` holds `holds`, which has no members to name.
    Through {
        /// The path up to the value that has no members.
        at: String,
        /// Its declared type, as the model writes it.
        holds: String,
    },
}

/// The declared type of the member `path` names among `fields`, through `Optional` and newtypes.
pub(crate) fn member(
    ir: &EssIr,
    fields: &[ResolvedField],
    path: &str,
) -> Result<ResolvedTypeRef, Unreached> {
    let mut segments = path.split('.');
    let first = segments.next().unwrap_or_default();
    let mut ty = fields
        .iter()
        .find(|field| field.name == first)
        .ok_or(Unreached::NoMember)?
        .type_ref
        .clone();
    let mut at = first.to_owned();
    for segment in segments {
        let members = ir_struct(ir, &ty, 0).ok_or_else(|| Unreached::Through {
            at: at.clone(),
            holds: ty.to_string(),
        })?;
        ty = members
            .iter()
            .find(|field| field.name == segment)
            .ok_or(Unreached::NoMember)?
            .type_ref
            .clone();
        at.push('.');
        at.push_str(segment);
    }
    Ok(ty)
}

/// The members of the struct `ty` is, through `Optional` and newtypes; `None` for anything else.
fn ir_struct<'a>(ir: &'a EssIr, ty: &ResolvedTypeRef, depth: usize) -> Option<&'a [ResolvedField]> {
    if depth > MAX_TYPE_DEPTH {
        return None;
    }
    match ty {
        ResolvedTypeRef::Optional { of } => ir_struct(ir, of, depth + 1),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => ir_struct(ir, of, depth + 1),
            ResolvedBody::Struct { fields, .. } => Some(fields),
            ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => None,
        },
        ResolvedTypeRef::Primitive { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => None,
    }
}

/// Writes `value`, of the declared type `ty`, at `path` as the payload shape's leaves: a struct
/// (through `Optional` and newtypes) is written member by member, anything else at `path` itself.
/// A member the value leaves out, or a value that is null, is written as null at each leaf below
/// it: absent, which is what the claim says of it.
///
/// The same walk the payload shape takes (`synthesize::describe`), so every key written here is a
/// leaf of the step's own shape.
pub(crate) fn leaves(
    ir: &EssIr,
    ty: &ResolvedTypeRef,
    path: &str,
    value: Option<&Node>,
    depth: usize,
    out: &mut BTreeMap<String, Node>,
) {
    if depth > MAX_TYPE_DEPTH {
        return;
    }
    let value = value.filter(|value| !matches!(value, Node::Null));
    match ty {
        ResolvedTypeRef::Optional { of } => leaves(ir, of, path, value, depth + 1, out),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => leaves(ir, of, path, value, depth + 1, out),
            ResolvedBody::Struct { fields, .. } => {
                let members = match value {
                    Some(Node::Map(members)) => Some(members),
                    _ => None,
                };
                for field in fields {
                    leaves(
                        ir,
                        &field.type_ref,
                        &format!("{path}.{}", field.name),
                        members.and_then(|members| members.get(&field.name)),
                        depth + 1,
                        out,
                    );
                }
            }
            ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => {
                out.insert(path.to_owned(), value.cloned().unwrap_or(Node::Null));
            }
        },
        ResolvedTypeRef::Primitive { .. }
        | ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. } => {
            out.insert(path.to_owned(), value.cloned().unwrap_or(Node::Null));
        }
    }
}

/// `written` with every member path folded into the value of the field it starts at, so a check
/// that reads fields whole sees the members a path claims: `to.city: Lisbon` beside
/// `to.zip: 1000` reads as `to: {city: Lisbon, zip: "1000"}`. A key that is no path, and a path
/// whose field a key already holds as something other than a mapping, are kept as written.
pub(crate) fn nested<V: Clone>(
    written: &BTreeMap<String, V>,
    literal: impl Fn(&V) -> Option<&Node>,
    wrap: impl Fn(Node) -> V,
) -> BTreeMap<String, V> {
    let mut folded: BTreeMap<String, Node> = BTreeMap::new();
    let mut kept = BTreeMap::new();
    for (key, value) in written {
        let node = literal(value);
        match (is_path(key), node) {
            (true, Some(node)) => {
                let segments: Vec<&str> = key.split('.').collect();
                let Some((first, rest)) = segments.split_first() else {
                    continue;
                };
                let here = folded
                    .entry((*first).to_owned())
                    .or_insert_with(|| Node::Map(BTreeMap::new()));
                insert_at(here, rest, node.clone());
            }
            _ => {
                kept.insert(key.clone(), value.clone());
            }
        }
    }
    for (key, node) in folded {
        kept.entry(key).or_insert_with(|| wrap(node));
    }
    kept
}

/// Writes `value` at `segments` below `node`, making a mapping for each member on the way; a
/// member already holding something other than a mapping is left as it is.
fn insert_at(node: &mut Node, segments: &[&str], value: Node) {
    let Node::Map(members) = node else {
        return;
    };
    match segments {
        [] => {}
        [last] => {
            members.insert((*last).to_owned(), value);
        }
        [first, rest @ ..] => insert_at(
            members
                .entry((*first).to_owned())
                .or_insert_with(|| Node::Map(BTreeMap::new())),
            rest,
            value,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_reaches_through_members_and_stops_at_null() {
        let actual = BTreeMap::from([(
            "checkpoint".to_owned(),
            Node::Map(BTreeMap::from([
                ("checkpoint_id".to_owned(), Node::Text("cp-1".into())),
                ("call".to_owned(), Node::Null),
            ])),
        )]);
        assert_eq!(
            reach(&actual, "checkpoint.checkpoint_id"),
            Some(&Node::Text("cp-1".into()))
        );
        assert_eq!(reach(&actual, "checkpoint.call"), None);
        assert_eq!(reach(&actual, "checkpoint.checkpoint_id.more"), None);
        assert_eq!(reach(&actual, "checkpoint.format"), None);
    }

    #[test]
    fn paths_fold_into_the_field_they_start_at() {
        let written = BTreeMap::from([
            ("to.city".to_owned(), Node::Text("Lisbon".into())),
            ("to.region.code".to_owned(), Node::Text("PT".into())),
            ("parcel_id".to_owned(), Node::Text("p-1".into())),
        ]);
        let folded = nested(&written, |node| Some(node), |node| node);
        assert_eq!(folded.len(), 2);
        assert_eq!(
            folded["to"],
            Node::Map(BTreeMap::from([
                ("city".to_owned(), Node::Text("Lisbon".into())),
                (
                    "region".to_owned(),
                    Node::Map(BTreeMap::from([(
                        "code".to_owned(),
                        Node::Text("PT".into())
                    )]))
                ),
            ]))
        );
    }
}
