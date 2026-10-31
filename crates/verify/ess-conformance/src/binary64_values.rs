//! A finite `Binary64` value in a conformance suite
//! (story:a-binary64-value-round-trips-through-a-conformance-suite,
//! `docs/design/model-binary64.md`).
//!
//! A suite carries a finite `Binary64` in an entity's fields, a command's input and response, an
//! event's and an error's fields, and a view's fields and row shape. Each runner reads and compares
//! one by its nearest binary64 under IEEE equality, so `1.5` equals `1.50` and `-0.0` equals `0`;
//! a token with no finite binary64 image (`1e400`) is refused by name before target effects.
//!
//! Released readers below suite/[`ORDINARY`] either refused a `Binary64` shape at the step or
//! compared an untyped number by its decimal spelling, so a suite of a model that reaches a carried
//! `Binary64` is ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the pair a response
//! member path opened ([`crate::member_paths`]), cumulative over every pair below. Every reader
//! refuses a typed `Binary64` record under a lower major before target effects, each at its own
//! location. A suite of a model without one keeps its earlier format and bytes.
//!
//! An entity's identity, a view's parameters, an actor's attributes, a binding and a conversion
//! do not carry one: the model is refused at that position
//! ([`ess_compiler::binary64::uncarried_uses`]).

use crate::admission::{escape, AdmissionError, AdmissionIssue};
use crate::scenario::{Holds, ScenarioStep};
use crate::selection::Declaration;
use crate::ConformanceSuite;
use ess_domain::{Field, Primitive, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

/// The ordinary suite major whose records may carry a `Binary64`.
pub const ORDINARY: u32 = crate::member_paths::ORDINARY;

/// Its coverage counterpart.
pub const COVERAGE: u32 = crate::member_paths::COVERAGE;

/// What a refusal under a lower major says.
pub const REQUIRES: &str = "a Binary64 value requires suite/62 or /63";

/// Whether the model reaches a `Binary64` a scenario may send or read.
pub fn used_by(ir: &ess_compiler::EssIr) -> bool {
    !ess_compiler::binary64::reached_uses(ir).is_empty()
}

/// The ordinary major a fresh suite of `ir` needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(ir: &ess_compiler::EssIr) -> Option<u32> {
    used_by(ir).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite of `ir` needs at least: [`COVERAGE`] when
/// [`used_by`].
pub fn coverage_floor(ir: &ess_compiler::EssIr) -> Option<u32> {
    used_by(ir).then_some(COVERAGE)
}

/// Refuses every typed `Binary64` record under a major below [`ORDINARY`], each at its location.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if suite.provenance.suite_version.major() >= ORDINARY {
        return Ok(());
    }
    let found = records(suite);
    if found.is_empty() {
        return Ok(());
    }
    Err(AdmissionError {
        issues: found
            .into_iter()
            .map(|path| AdmissionIssue {
                reason: "UnsupportedPrimitive",
                path,
                detail: REQUIRES.into(),
            })
            .collect(),
    })
}

/// The JSON Pointer of every typed record of the suite that names a `Binary64`, in document order.
pub(crate) fn records(suite: &ConformanceSuite) -> Vec<String> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let at = format!("/scenarios/{}", escape(&id.to_string()));
        if let Some(trace) = &scenario.one_time_response {
            for (index, origin) in trace.origins.iter().enumerate() {
                schema(
                    &origin.response.fields,
                    &origin.response.declarations,
                    &format!("{at}/one_time_response/origins/{index}/response"),
                    &mut found,
                );
            }
        }
        for (index, step) in scenario.steps.iter().enumerate() {
            let step_at = format!("{at}/steps/{index}");
            match step {
                ScenarioStep::ExpectEvent { shape, .. }
                | ScenarioStep::EventuallyEvent { shape, .. }
                | ScenarioStep::ExpectEventValues { shape, .. }
                | ScenarioStep::AwaitOutcome { shape, .. } => {
                    for (name, leaf) in shape.leaves() {
                        if matches!(
                            leaf.holds,
                            Holds::Primitive {
                                kind: Primitive::Binary64
                            }
                        ) {
                            found.push(format!("{step_at}/shape/{}", escape(name)));
                        }
                    }
                }
                ScenarioStep::SnapshotCompleteSubject { shape, .. } => schema(
                    &shape.fields,
                    &shape.declarations,
                    &format!("{step_at}/shape"),
                    &mut found,
                ),
                ScenarioStep::CaptureCommandResult { capture }
                | ScenarioStep::ExpectReplayResult { capture } => schema(
                    &capture.fields,
                    &capture.declarations,
                    &format!("{step_at}/capture"),
                    &mut found,
                ),
                ScenarioStep::ExpectResponsePayload { response } => {
                    let response_at = format!("{step_at}/response");
                    schema(
                        &response.fields,
                        &response.declarations,
                        &response_at,
                        &mut found,
                    );
                    members(
                        &response.targets,
                        &format!("{response_at}/targets"),
                        &mut found,
                    );
                }
                ScenarioStep::ExpectDirectResponse { response } => schema(
                    &response.fields,
                    &response.declarations,
                    &format!("{step_at}/response"),
                    &mut found,
                ),
                ScenarioStep::ResolveFixtures { fixtures, .. } => {
                    let fixtures_at = format!("{step_at}/fixtures");
                    for (index, field) in fixtures.fields.iter().enumerate() {
                        if mentions(&field.type_ref) {
                            found.push(format!("{fixtures_at}/fields/{index}/type"));
                        }
                    }
                    declarations(&fixtures.declarations, &fixtures_at, &mut found);
                }
                _ => {}
            }
        }
    }
    found
}

fn schema(
    fields: &[Field],
    declared: &BTreeMap<QualifiedName, Declaration>,
    at: &str,
    found: &mut Vec<String>,
) {
    members(fields, &format!("{at}/fields"), found);
    declarations(declared, at, found);
}

fn members(fields: &[Field], at: &str, found: &mut Vec<String>) {
    for (index, field) in fields.iter().enumerate() {
        if mentions(&field.type_ref) {
            found.push(format!("{at}/{index}/type"));
        }
    }
}

fn declarations(
    declared: &BTreeMap<QualifiedName, Declaration>,
    at: &str,
    found: &mut Vec<String>,
) {
    for (name, declaration) in declared {
        let carries = match declaration {
            Declaration::Newtype { of } => mentions(of),
            Declaration::Struct { fields } => fields.iter().any(|field| mentions(&field.type_ref)),
            Declaration::Union { variants, .. } => variants.values().flatten().any(mentions),
            Declaration::Enum { .. } => false,
        };
        if carries {
            found.push(format!("{at}/declarations/{}", escape(&name.to_string())));
        }
    }
}

/// Whether `type_ref` names a `Binary64` directly, through any wrapper.
fn mentions(type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Primitive(primitive) => *primitive == Primitive::Binary64,
        TypeRef::Optional(of) | TypeRef::List(of) => mentions(of),
        TypeRef::Map(key, value) => *key == Primitive::Binary64 || mentions(value),
        TypeRef::Named(_) => false,
    }
}

/// Whether two rows of one exact observer's `fields` are the same value: equal, or equal once
/// every `Binary64` they reach is read as its nearest binary64 and compared under IEEE equality,
/// so `1` equals `1.0000000000000000001` and `-0.0` equals `0`. Every other leaf compares exactly.
pub(crate) fn same_row(
    fields: &[Field],
    declarations: &BTreeMap<QualifiedName, Declaration>,
    left: &BTreeMap<String, Node>,
    right: &BTreeMap<String, Node>,
) -> bool {
    if left == right {
        return true;
    }
    left.len() == right.len()
        && left.iter().all(|(name, value)| {
            right.get(name).is_some_and(|other| {
                fields
                    .iter()
                    .find(|field| &field.name == name)
                    .map_or(value == other, |field| {
                        same(&field.type_ref, declarations, value, other, 0)
                    })
            })
        })
}

/// Whether a value of `ty` holds a `Binary64` anywhere, through every wrapper and declaration.
pub(crate) fn reaches(
    ty: &TypeRef,
    declarations: &BTreeMap<QualifiedName, Declaration>,
    depth: usize,
) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match ty {
        TypeRef::Primitive(primitive) => *primitive == Primitive::Binary64,
        TypeRef::Optional(of) | TypeRef::List(of) => reaches(of, declarations, depth + 1),
        TypeRef::Map(key, of) => {
            *key == Primitive::Binary64 || reaches(of, declarations, depth + 1)
        }
        TypeRef::Named(name) => match declarations.get(name) {
            Some(Declaration::Newtype { of }) => reaches(of, declarations, depth + 1),
            Some(Declaration::Struct { fields }) => fields
                .iter()
                .any(|field| reaches(&field.type_ref, declarations, depth + 1)),
            Some(Declaration::Union { variants, .. }) => variants
                .values()
                .flatten()
                .any(|of| reaches(of, declarations, depth + 1)),
            Some(Declaration::Enum { .. }) | None => false,
        },
    }
}

/// [`same_row`] for one value of `ty`.
pub(crate) fn same(
    ty: &TypeRef,
    declarations: &BTreeMap<QualifiedName, Declaration>,
    left: &Node,
    right: &Node,
    depth: usize,
) -> bool {
    if left == right {
        return true;
    }
    if depth > MAX_DEPTH {
        return false;
    }
    match (ty, left, right) {
        (TypeRef::Primitive(Primitive::Binary64), Node::Number(a), Node::Number(b)) => {
            ieee_equal(a.get(), b.get())
        }
        (TypeRef::Optional(of), _, _) => same(of, declarations, left, right, depth + 1),
        (TypeRef::List(of), Node::Seq(a), Node::Seq(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(a, b)| same(of, declarations, a, b, depth + 1))
        }
        (TypeRef::Map(_, of), Node::Map(a), Node::Map(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, a)| {
                    b.get(key)
                        .is_some_and(|b| same(of, declarations, a, b, depth + 1))
                })
        }
        (TypeRef::Named(name), _, _) => match declarations.get(name) {
            Some(Declaration::Newtype { of }) => same(of, declarations, left, right, depth + 1),
            Some(Declaration::Struct { fields }) => {
                let (Node::Map(a), Node::Map(b)) = (left, right) else {
                    return false;
                };
                members_same(
                    a,
                    b,
                    depth,
                    |key| {
                        fields
                            .iter()
                            .find(|field| field.name == key)
                            .map(|field| &field.type_ref)
                    },
                    declarations,
                )
            }
            Some(Declaration::Union { tag, variants }) => {
                let (Node::Map(a), Node::Map(b)) = (left, right) else {
                    return false;
                };
                if a.get(tag) != b.get(tag) {
                    return false;
                }
                let content = a
                    .get(tag)
                    .and_then(Node::as_text)
                    .and_then(|label| variants.get(label))
                    .and_then(Option::as_ref);
                members_same(
                    a,
                    b,
                    depth,
                    |key| (key != tag).then_some(content).flatten(),
                    declarations,
                )
            }
            Some(Declaration::Enum { .. }) | None => false,
        },
        _ => false,
    }
}

/// IEEE equality, the contract itself: `-0.0` equals `0.0`, and there is no epsilon.
#[allow(clippy::float_cmp)] // Exact binary64 equality is what the suite promises.
fn ieee_equal(left: f64, right: f64) -> bool {
    left == right
}

/// The deepest a typed comparison descends: the selection value walk's own bound.
const MAX_DEPTH: usize = 128;

fn members_same<'a>(
    left: &BTreeMap<String, Node>,
    right: &BTreeMap<String, Node>,
    depth: usize,
    member: impl Fn(&str) -> Option<&'a TypeRef>,
    declarations: &BTreeMap<QualifiedName, Declaration>,
) -> bool {
    left.len() == right.len()
        && left.iter().all(|(key, a)| {
            right.get(key).is_some_and(|b| match member(key) {
                Some(ty) => same(ty, declarations, a, b, depth + 1),
                None => a == b,
            })
        })
}
