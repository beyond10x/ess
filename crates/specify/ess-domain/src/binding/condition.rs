//! A binding's event-payload condition (ess/22, beyond10x/ess#268 and beyond10x/ess#194).
//!
//! `docs/design/conditional-binding-failure-policies.md`, "Payload condition", is the binding
//! design. A local or external event cause may carry `where:`, a finite typed predicate over the
//! declared event payload:
//!
//! ```yaml
//! when:
//!   event: demo.messages.MessageReceived
//!   where: [defined(event.order), event.kind == ship]
//! ```
//!
//! | rule | code |
//! |---|---|
//! | `where:` below ess/22 | [`UnsupportedFormatVersion`](ValidationCode::UnsupportedFormatVersion) |
//! | `where:` on a periodic cause | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | a construct outside the bounded vocabulary | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | a root other than `event`, or a member the event does not declare | [`UnobservableFact`](ValidationCode::UnobservableFact) |
//! | a comparison of a leaf that is not String or an enum, or a literal that is not one of its variants | [`TypeMismatch`](ValidationCode::TypeMismatch) |
//!
//! The vocabulary is the bounded selection fragment with root `event` instead of `item`: Always,
//! Never, `defined(...)`, `==`/`!=` of a declared String or enum leaf against a literal, and `all`,
//! `any`, `not`. A path names one to three declared members after `event`, through structs and
//! Optional structs, never through a union.
//!
//! # Evaluation
//!
//! Before selections, conversions, mapping or invocation. True invokes, False skips this binding
//! occurrence and nothing else, Unknown is an unmet obligation that invokes nothing. Absent and
//! `null` Optional members keep the evaluator's existing meaning: `defined` of one is False, a
//! comparison reading one is Unknown, and Kleene `all`/`any` decide around it.
//!
//! # Presence proof (#194)
//!
//! [`ConditionPlan::proves_present`] is the set of member paths the condition being True proves
//! present, prefix-closed. A mapping reading an Optional member into a required input is admitted
//! exactly where every Optional member on its path is in that set: `defined(event.order)` admits
//! `event.order.id` into a required `String`, and does not admit `event.order.note` when `note`
//! is itself Optional.

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate, Truth};

use crate::command::EventSpec;
use crate::system::FormatVersion;
use crate::types::{TypeBody, TypeRef, TypeRegistry};

/// The root every condition path starts with.
pub const ROOT: &str = "event";
/// The most declared members a path names after [`ROOT`].
pub const MAX_MEMBERS: usize = 3;
/// The most predicate nodes one condition may hold, as for one selection predicate.
pub const MAX_NODES: usize = crate::selection::MAX_PREDICATE_NODES;
/// The first source format that admits `where:` on a binding cause.
pub const FORMAT: FormatVersion = FormatVersion::V22;

/// What a condition path ends at when a comparison reads it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Leaf {
    /// Any text.
    String,
    /// One of these declared variant names.
    Enum {
        /// The variants, in declaration order.
        variants: Vec<String>,
    },
    /// Anything else: only `defined(...)` reads it.
    Other,
}

/// One path the condition reads, resolved against the declared event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionRead {
    /// The declared members after `event`, one to three.
    pub members: Vec<String>,
    /// Whether the member at the same position may be absent (its declared type is Optional,
    /// through any newtypes).
    pub optional: Vec<bool>,
    /// What the path ends at.
    pub leaf: Leaf,
}

impl ConditionRead {
    /// The path as written, `event.<members>`.
    pub fn path(&self) -> String {
        format!("{ROOT}.{}", self.members.join("."))
    }
}

/// A condition, admitted against the declared event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConditionPlan {
    /// The predicate as written.
    pub predicate: Predicate,
    /// Every path it reads, keyed by `event.<members>`.
    pub reads: BTreeMap<String, ConditionRead>,
}

/// Where evaluation could not read the payload as declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Malformed(pub String);

impl std::fmt::Display for Malformed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn refused(code: ValidationCode, at: &str, message: impl Into<String>) -> ValidationError {
    ValidationError::new(code, at, message)
}

/// Strips Optional and newtype wrappers, reporting whether any Optional was crossed.
fn transparent<'a>(mut ty: &'a TypeRef, types: &'a TypeRegistry) -> Option<(&'a TypeRef, bool)> {
    let mut optional = false;
    for _ in 0..=crate::accessor::MAX_OPERATIONS {
        match ty {
            TypeRef::Optional(of) => {
                optional = true;
                ty = of;
            }
            TypeRef::Named(name) => match &types.get(name)?.body {
                TypeBody::Newtype { of, .. } => ty = of,
                _ => return Some((ty, optional)),
            },
            _ => return Some((ty, optional)),
        }
    }
    None
}

/// Resolves one path against the event, or the refusal that says why it names nothing.
pub fn resolve_path(
    path: &FactPath,
    event: &EventSpec,
    types: &TypeRegistry,
    at: &str,
) -> Result<ConditionRead, ValidationError> {
    let segments = path.segments();
    if segments.first().map(String::as_str) != Some(ROOT) {
        return Err(refused(
            ValidationCode::UnobservableFact,
            at,
            format!(
                "`{path}` is not read from the event payload; a binding condition reads only \
                 `{ROOT}.<member>`"
            ),
        )
        .with_hint(
            "outcome names, command input, stored rows, the caller and the delivery context are \
             not condition roots",
        ));
    }
    let members = &segments[1..];
    if members.is_empty() || members.len() > MAX_MEMBERS {
        return Err(refused(
            ValidationCode::UnsupportedConstruct,
            at,
            format!(
                "`{path}` names {} members; a condition path names one to {MAX_MEMBERS}",
                members.len()
            ),
        ));
    }
    let mut optional = Vec::new();
    let mut ty = &event
        .field(&members[0])
        .ok_or_else(|| {
            refused(
                ValidationCode::UnobservableFact,
                at,
                format!(
                    "`{path}`: `{}` is not a field of `{}`",
                    members[0], event.name
                ),
            )
            .with_hint(crate::binding::readable(event))
        })?
        .type_ref;
    for index in 0..members.len() {
        let unknown = || {
            refused(
                ValidationCode::UnobservableFact,
                at,
                format!("`{path}` does not resolve through declared types"),
            )
        };
        let (present, may_miss) = transparent(ty, types).ok_or_else(unknown)?;
        optional.push(may_miss);
        if index + 1 == members.len() {
            let leaf = match present {
                TypeRef::Primitive(crate::Primitive::String) => Leaf::String,
                TypeRef::Named(name) => match types.get(name).map(|declared| &declared.body) {
                    Some(TypeBody::Enum { variants }) => Leaf::Enum {
                        variants: variants.iter().map(|v| v.name().to_owned()).collect(),
                    },
                    _ => Leaf::Other,
                },
                _ => Leaf::Other,
            };
            return Ok(ConditionRead {
                members: members.to_vec(),
                optional,
                leaf,
            });
        }
        let next = &members[index + 1];
        let TypeRef::Named(name) = present else {
            return Err(refused(
                ValidationCode::UnsupportedConstruct,
                at,
                format!("`{path}` reads `{next}` of a value that is not a declared struct"),
            ));
        };
        let Some(TypeBody::Struct { fields, .. }) = types.get(name).map(|declared| &declared.body)
        else {
            return Err(refused(
                ValidationCode::UnsupportedConstruct,
                at,
                format!(
                    "`{path}` reads `{next}` of `{name}`; a condition path traverses structs and \
                     Optional structs only, never a union"
                ),
            ));
        };
        ty = &fields
            .iter()
            .find(|field| &field.name == next)
            .ok_or_else(|| {
                refused(
                    ValidationCode::UnobservableFact,
                    at,
                    format!("`{path}`: `{next}` is not a field of `{name}`"),
                )
            })?
            .type_ref;
    }
    unreachable!("a nonempty path returns at its last member")
}

impl ConditionPlan {
    /// Admits `predicate` against `event`: the bounded vocabulary, every path, every literal.
    pub fn resolve(
        predicate: &Predicate,
        event: &EventSpec,
        types: &TypeRegistry,
        at: &str,
    ) -> Result<Self, ValidationErrors> {
        let mut errors = ValidationErrors::new();
        let mut reads = BTreeMap::new();
        let mut pending = vec![predicate];
        let mut count = 0;
        while let Some(node) = pending.pop() {
            count += 1;
            if count > MAX_NODES {
                errors.push(refused(
                    ValidationCode::AccessorResource,
                    at,
                    format!("a binding condition holds at most {MAX_NODES} predicate nodes"),
                ));
                break;
            }
            match node {
                Predicate::Always | Predicate::Never => {}
                Predicate::All(children) | Predicate::Any(children) => pending.extend(children),
                Predicate::Not(child) => pending.push(child),
                Predicate::Defined(path) => match resolve_path(path, event, types, at) {
                    Ok(read) => {
                        reads.insert(path.to_string(), read);
                    }
                    Err(error) => errors.push(error),
                },
                Predicate::Compare {
                    left: Operand::Fact(path),
                    op: CompareOp::Eq | CompareOp::Ne,
                    right: Operand::Literal(literal),
                    kind: CompareKind::Value,
                } => match resolve_path(path, event, types, at) {
                    Ok(read) => {
                        if let Err(error) = literal_fits(&read, literal, at) {
                            errors.push(error);
                        }
                        reads.insert(path.to_string(), read);
                    }
                    Err(error) => errors.push(error),
                },
                other => errors.push(
                    refused(
                        ValidationCode::UnsupportedConstruct,
                        at,
                        format!("`{other}` is outside the bounded binding condition"),
                    )
                    .with_hint(
                        "a binding condition admits true, false, defined(event.<path>), \
                         `event.<path> == <literal>` and `!=` over a String or enum leaf, and \
                         all/any/not of those",
                    ),
                ),
            }
        }
        errors.into_result(Self {
            predicate: predicate.clone(),
            reads,
        })
    }

    /// The member paths the condition being True proves present, prefix-closed.
    pub fn proves_present(&self) -> BTreeSet<Vec<String>> {
        facts(&self.predicate, true, &self.reads)
    }

    /// Evaluates the condition over one admitted event payload.
    ///
    /// # Errors
    ///
    /// [`Malformed`] where a member the path crosses is present and is not an object, which normal
    /// event admission refuses before a condition is evaluated.
    pub fn evaluate(&self, payload: &BTreeMap<String, Node>) -> Result<Truth, Malformed> {
        let mut store = FactStore::new();
        for (written, read) in &self.reads {
            let path = FactPath::new(written).map_err(|error| Malformed(error.to_string()))?;
            let mut value = payload.get(&read.members[0]);
            for member in &read.members[1..] {
                value = match value {
                    None | Some(Node::Null) => None,
                    Some(Node::Map(fields)) => fields.get(member),
                    Some(_) => {
                        return Err(Malformed(format!(
                            "`{}` crosses a value that is not an object",
                            read.path()
                        )))
                    }
                };
            }
            match value {
                None | Some(Node::Null) => {}
                Some(Node::Text(text)) => store.set(path, FactValue::Text(text.clone())),
                Some(_) => store.mark_present(path),
            }
        }
        Ok(self.predicate.evaluate(&store))
    }
}

fn literal_fits(
    read: &ConditionRead,
    literal: &FactValue,
    at: &str,
) -> Result<(), ValidationError> {
    let mismatch = |message: String| refused(ValidationCode::TypeMismatch, at, message);
    let FactValue::Text(text) = literal else {
        return Err(mismatch(format!(
            "`{}` is compared with `{literal}`, which is not text; a binding condition compares \
             a String or enum leaf with a text literal",
            read.path()
        )));
    };
    match &read.leaf {
        Leaf::String => Ok(()),
        Leaf::Enum { variants } if variants.iter().any(|variant| variant == text) => Ok(()),
        Leaf::Enum { variants } => Err(mismatch(format!(
            "`{text}` is not a variant of `{}`; it has {}",
            read.path(),
            variants.join(", ")
        ))),
        Leaf::Other => Err(mismatch(format!(
            "`{}` is not a String or enum leaf, so it cannot be compared; `defined(...)` may test \
             its presence",
            read.path()
        ))),
    }
}

fn prefixes(members: &[String]) -> BTreeSet<Vec<String>> {
    (1..=members.len())
        .map(|end| members[..end].to_vec())
        .collect()
}

/// The paths a node's definite `truth` proves present: Defined when True proves its path,
/// a definite comparison proves the path it read, Not swaps, All-true and Any-false union,
/// All-false and Any-true intersect, Always and Never prove nothing.
fn facts(
    predicate: &Predicate,
    truth: bool,
    reads: &BTreeMap<String, ConditionRead>,
) -> BTreeSet<Vec<String>> {
    let path_of = |path: &FactPath| {
        reads
            .get(&path.to_string())
            .map(|read| prefixes(&read.members))
            .unwrap_or_default()
    };
    match predicate {
        Predicate::Defined(path) if truth => path_of(path),
        Predicate::Compare {
            left: Operand::Fact(path),
            ..
        } => path_of(path),
        Predicate::Not(child) => facts(child, !truth, reads),
        Predicate::All(children) | Predicate::Any(children) => {
            let union = matches!(predicate, Predicate::All(_)) == truth;
            let mut sets = children.iter().map(|child| facts(child, truth, reads));
            if union {
                sets.flatten().collect()
            } else {
                let Some(first) = sets.next() else {
                    return BTreeSet::new();
                };
                sets.fold(first, |acc, set| acc.intersection(&set).cloned().collect())
            }
        }
        _ => BTreeSet::new(),
    }
}

/// The Optional member positions along `members`, read through the event's declared types, or
/// `None` where the path crosses a union or does not resolve as a struct path.
pub fn optional_positions(
    members: &[String],
    event: &EventSpec,
    types: &TypeRegistry,
) -> Option<Vec<bool>> {
    let path =
        FactPath::from_segments(std::iter::once(ROOT.to_owned()).chain(members.iter().cloned()));
    resolve_path(&path, event, types, "")
        .ok()
        .map(|read| read.optional)
}

/// Whether `proved` makes every Optional member on `members` present.
pub fn covers(proved: &BTreeSet<Vec<String>>, members: &[String], optional: &[bool]) -> bool {
    optional
        .iter()
        .enumerate()
        .all(|(index, optional)| !optional || proved.contains(&members[..=index].to_vec()))
}

/// The type a proved-present member path has once its Optional wrappers are removed: the
/// declared terminal, with every Optional the proof covers stripped from its outside.
pub fn present_type(ty: &TypeRef) -> TypeRef {
    let mut ty = ty;
    while let TypeRef::Optional(of) = ty {
        ty = of;
    }
    ty.clone()
}

/// The format gate, and the periodic refusal, for every binding of the specification.
pub fn validate_specification(spec: &crate::Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for binding in spec.bindings().values() {
        let Some(condition) = &binding.condition else {
            continue;
        };
        let at = format!("binding.{}.when.where", binding.name);
        if spec.system().format.major() < FORMAT.major() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    at.clone(),
                    format!(
                        "an event-payload condition (`where: {condition}`) on a binding requires \
                         specification format ess/22"
                    ),
                )
                .with_hint("declare `format: ess/22`, or remove `where:`"),
            );
        }
    }
    errors
}
