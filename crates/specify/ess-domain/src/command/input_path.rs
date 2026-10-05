//! A value read from a member of a struct input, `input.<path>` (source format `ess/22`, Family F
//! A4, beyond10x/ess#233, `docs/design/expression-family-source22.md`).
//!
//! A path names declared fields only: the first segment is a command input, and each further
//! segment is a member of the struct the segment before resolves to, through any `Optional` and
//! newtype around it. A primitive, an enum, a list, a map or a union has no members, so a path
//! stops there. The value read is the last segment's; an absent `Optional` anywhere before it
//! leaves the value absent, so a path that crosses one is read at `Optional<…>` of the last
//! segment's type.
//!
//! One segment is the input field it always was, and is resolved exactly as before.

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError};

use super::CommandSpec;
use crate::system::FormatVersion;
use crate::types::{Field, TypeRef, TypeRegistry};

/// `true` when `field` is a path of more than one segment.
pub fn is_path(field: &str) -> bool {
    field.contains('.')
}

/// `true` when a specification in `format` reads input paths: from `ess/22`.
pub fn admitted(format: Option<FormatVersion>) -> bool {
    format.is_some_and(|format| format.major() >= FormatVersion::V22.major())
}

/// A resolved `input.<path>`: what the last segment holds, and whether an `Optional` before it may
/// leave it absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPath {
    /// The declared segments joined by `.`, as written.
    pub written: String,
    /// The last segment's declared type.
    pub terminal: TypeRef,
    /// Whether an `Optional` is crossed before the last segment.
    pub route_optional: bool,
}

impl InputPath {
    /// The type the value is read at: the last segment's, or `Optional<…>` of it where an
    /// `Optional` before it may leave it absent.
    pub fn type_ref(&self) -> TypeRef {
        if self.route_optional && !self.terminal.is_optional() {
            TypeRef::Optional(Box::new(self.terminal.clone()))
        } else {
            self.terminal.clone()
        }
    }

    /// Whether the value may be absent: an `Optional` anywhere on the route, the last segment
    /// included.
    pub fn may_be_absent(&self) -> bool {
        self.route_optional || self.terminal.is_optional()
    }

    /// The read as a field named by its path, at [`Self::type_ref`], for the checks that take one.
    pub fn as_field(&self) -> Field {
        Field::new(self.written.clone(), self.type_ref())
    }
}

/// Why a path resolves to nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unresolved {
    /// The first segment is no input of the command.
    Root(String),
    /// A segment names no member of the struct before it.
    Member {
        /// The path up to the struct, joined by `.`.
        at: String,
        /// The segment written.
        member: String,
        /// The struct's type, as declared.
        owner: TypeRef,
        /// The struct's members, in declaration order.
        declared: Vec<String>,
    },
    /// A segment names a typed attribute of the enum before it (`ess/23`, beyond10x/ess#450),
    /// which this cut reads only in a predicate.
    Attribute {
        /// The path up to the enum, joined by `.`.
        at: String,
        /// The attribute named.
        attribute: String,
        /// The enum.
        owner: crate::name::QualifiedName,
    },
    /// A segment follows a value that has no members.
    Opaque {
        /// The path up to that value, joined by `.`.
        at: String,
        /// Its type.
        type_ref: TypeRef,
    },
}

/// Resolves `path` (without its `input.` prefix) against `command`'s input and the declared types.
pub fn resolve(
    command: &CommandSpec,
    types: &TypeRegistry,
    path: &str,
) -> Result<InputPath, Unresolved> {
    let mut segments = path.split('.');
    let root = segments.next().unwrap_or_default();
    let Some(read) = command.input_field(root) else {
        return Err(Unresolved::Root(root.to_owned()));
    };
    let mut current = read.type_ref.clone();
    let mut at = root.to_owned();
    let mut route_optional = false;
    for segment in segments {
        let Some(members) = types.struct_fields(&current) else {
            if let Some(owner) = attributed_enum(types, &current, segment) {
                return Err(Unresolved::Attribute {
                    at,
                    attribute: segment.to_owned(),
                    owner,
                });
            }
            return Err(Unresolved::Opaque {
                at,
                type_ref: current,
            });
        };
        route_optional |= types.newtype_layers(&current).optional;
        let Some(member) = members.iter().find(|member| member.name == segment) else {
            return Err(Unresolved::Member {
                at,
                member: segment.to_owned(),
                owner: current.clone(),
                declared: members.iter().map(|member| member.name.clone()).collect(),
            });
        };
        current = member.type_ref.clone();
        at = format!("{at}.{segment}");
    }
    Ok(InputPath {
        written: path.to_owned(),
        terminal: current,
        route_optional,
    })
}

impl Unresolved {
    /// The refusal at `at`, naming the command whose input the path reads.
    pub fn refusal(&self, at: &ConstructRef, command: &CommandSpec, path: &str) -> ValidationError {
        let (code, message, hint) = self.parts(command, path);
        ValidationError::at(at.clone(), code, message).with_hint(hint)
    }

    /// [`Self::refusal`] at a location written as a dotted path.
    pub fn refusal_at(&self, at: String, command: &CommandSpec, path: &str) -> ValidationError {
        let (code, message, hint) = self.parts(command, path);
        ValidationError::new(code, at, message).with_hint(hint)
    }

    fn parts(&self, command: &CommandSpec, path: &str) -> (ValidationCode, String, String) {
        match self {
            Self::Root(root) => (
                ValidationCode::UndeclaredReference,
                format!(
                    "`input.{path}` reads `{root}`, which `{}` does not declare as input",
                    command.name
                ),
                format!(
                    "declared input: {}",
                    command
                        .input
                        .iter()
                        .map(|field| format!("`{}`", field.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
            Self::Member {
                at: before,
                member,
                owner,
                declared,
            } => (
                ValidationCode::UndeclaredReference,
                format!(
                    "`input.{path}` reads `{member}` of `input.{before}`, and `{member}` is not a \
                     field of `{owner}`"
                ),
                format!("`{owner}` has: {}", declared.join(", ")),
            ),
            Self::Attribute {
                at: before,
                attribute,
                owner,
            } => (
                ValidationCode::UnsupportedConstruct,
                format!(
                    "`input.{path}` reads the attribute `{attribute}` of `{owner}` as a value; \
                     this cut reads an enum attribute only in a predicate, where it is lowered to \
                     membership over the variants (ess/23)"
                ),
                format!(
                    "read `input.{before}` whole, or decide by the attribute in a guard: \
                     `when: {before}.{attribute} == …`"
                ),
            ),
            Self::Opaque {
                at: before,
                type_ref,
            } => (
                ValidationCode::TypeMismatch,
                format!(
                    "`input.{path}` reads past `input.{before}`, which is `{type_ref}` and has no \
                     members; a path follows struct fields only"
                ),
                format!("read `input.{before}` whole, or name a struct input"),
            ),
        }
    }
}

/// The refusal for a path below `ess/22`, where `what` is the form that needs it.
pub fn below_ess_22(at: &ConstructRef, what: &str) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::UnsupportedFormatVersion,
        format!("{what} requires specification format ess/22"),
    )
    .with_hint("write `format: ess/22` on the source that declares the system")
}

/// The refusal for a path whose route may leave it absent where it would supply `what`: an
/// identity, or the address of another row (decision 8 of the design's final review).
pub fn optional_route(at: &ConstructRef, path: &str, what: &str) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::TypeMismatch,
        format!(
            "`input.{path}` crosses an `Optional`, and a dotted input path supplies {what} only \
             when its whole route is required"
        ),
    )
    .with_hint(
        "make every struct on the route required, or read the identity from a top-level input",
    )
}

/// The enum `type_ref` is, through `Optional` and newtypes, where it declares `attribute`.
fn attributed_enum(
    types: &TypeRegistry,
    type_ref: &TypeRef,
    attribute: &str,
) -> Option<crate::name::QualifiedName> {
    let mut current = type_ref;
    for _ in 0..=crate::types::MAX_TYPE_DEPTH {
        match current {
            TypeRef::Optional(inner) => current = inner,
            TypeRef::Named(name) => match &types.get(name)?.body {
                crate::types::TypeBody::Newtype { of, .. } => current = of,
                crate::types::TypeBody::Enum { variants } => {
                    return variants.first()?.attribute(attribute).map(|_| name.clone());
                }
                _ => return None,
            },
            _ => return None,
        }
    }
    None
}
