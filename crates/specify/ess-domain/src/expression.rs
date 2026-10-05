//! Shared semantic admission for the existing predicate tree.
//!
//! Resolution describes required accesses; it does not promise that a producer publishes them,
//! that an optional value exists, or that any witness satisfies the predicate.

use std::collections::BTreeSet;
use std::fmt;

pub mod lexical;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{
    CompareKind, CompareOp, FoldOp, OffsetMagnitude, OffsetOperand, Operand, Predicate, Quantified,
    TextOp,
};

use crate::{Field, Primitive, TypeBody, TypeRef, TypeRegistry};

/// The three scalar representations understood by the predicate evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    /// Boolean truth.
    Bool,
    /// Integer or decimal numeric representation.
    Number,
    /// Text, including enums and text-backed primitives.
    Text,
}

impl ScalarKind {
    /// The existing evaluator representation of a declared primitive, or `None` for `Json`, which
    /// is no scalar: a predicate reads nothing from a JSON value (beyond10x/ess#138).
    pub fn of(primitive: Primitive) -> Option<Self> {
        match primitive {
            Primitive::Boolean => Some(Self::Bool),
            Primitive::Integer | Primitive::Decimal | Primitive::Binary64 => Some(Self::Number),
            Primitive::String
            | Primitive::Timestamp
            | Primitive::Duration
            | Primitive::Uuid
            | Primitive::Bytes => Some(Self::Text),
            Primitive::Json => None,
        }
    }

    fn literal(value: &FactValue) -> Self {
        match value {
            FactValue::Bool(_) => Self::Bool,
            FactValue::Number(_) => Self::Number,
            FactValue::Text(_) => Self::Text,
        }
    }
}

impl fmt::Display for ScalarKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// One shallow type shape; references remain in the environment's own vocabulary.
#[derive(Debug, Clone)]
pub enum Shape<T> {
    /// A primitive scalar.
    Scalar(ScalarKind),
    /// A closed text vocabulary.
    Enum(Vec<String>),
    /// A transparent named representation.
    Alias(T),
    /// A transparent possibly absent representation.
    Optional(T),
    /// A struct whose members are looked up through the environment.
    Struct,
    /// An ordered collection of elements.
    List(T),
    /// A map, quantified over its values.
    Map(T),
    /// Any JSON value (beyond10x/ess#138): no selectors, and no scalar a predicate could compare.
    Json,
    /// A tagged union with no predicate selectors.
    Union,
}

/// A read-only adapter over declarations or resolved compiler types.
pub trait TypeEnvironment {
    /// A reference owned by this environment, never a mirrored serialized model.
    type Type: Clone + fmt::Display;

    /// A declared observable root.
    fn root(&self, name: &str) -> Option<Self::Type>;
    /// The Integer reference used by collection cardinality.
    fn cardinality_type(&self) -> Self::Type;
    /// One shallow shape, or the name of an unresolved declaration.
    fn shape(&self, reference: &Self::Type) -> Result<Shape<Self::Type>, String>;
    /// Whether this nominal type requires occurrence-scoped clock evidence rather than scalar comparison.
    fn is_clock_reading(&self, _reference: &Self::Type) -> bool {
        false
    }
    /// Whether this terminal type is the `Timestamp` primitive, ordered by the instant it names.
    fn is_instant(&self, _reference: &Self::Type) -> bool {
        false
    }
    /// Whether this terminal type is the `Duration` primitive, which has no ordering.
    ///
    /// A `Duration` is carried as ISO 8601 text, and text is ordered by its bytes, which puts
    /// `PT10M` below `PT5M`. So an ordering over one is refused rather than answered wrongly.
    fn is_duration(&self, _reference: &Self::Type) -> bool {
        false
    }
    /// Whether this terminal type is the `String` primitive, the one type a string operator
    /// (`starts_with`, `ends_with`, `contains`) applies to.
    ///
    /// Asked of the resolved terminal, so a newtype of `String` at any depth and an `Optional` of
    /// one answer `true` too. [`ScalarKind::Text`] cannot say it: it also covers `Timestamp`,
    /// `Duration`, `Uuid`, `Bytes` and enums.
    fn is_string(&self, _reference: &Self::Type) -> bool {
        false
    }
    /// Whether this terminal type is the `Integer` primitive, the one numeric type an Integer
    /// offset (`upper == lower + 5`, `docs/design/expression-family-source22.md` A2) moves.
    ///
    /// Asked of the resolved terminal, so a newtype of `Integer` at any depth, an `Optional` of one
    /// and a collection's `.count` answer `true`; a `Decimal` and a `Binary64` answer `false`.
    fn is_integer(&self, _reference: &Self::Type) -> bool {
        false
    }
    /// Whether this environment admits `.count` on a `String`, the length in Unicode scalar values
    /// that `ess/11` introduced.
    ///
    /// `true` by default: the compiler's environment reads an IR, which exists only after
    /// validation admitted it. [`DomainEnvironment`] answers from the format its registry serves.
    fn admits_text_length(&self) -> bool {
        true
    }
    /// Whether this environment admits `defined(x)` where `x` is an `Optional` aggregate — a
    /// struct, list, map, union or `Json` — which `ess/16` introduced (beyond10x/ess#176).
    ///
    /// `true` by default, for the reason [`Self::admits_text_length`] is.
    fn admits_aggregate_presence(&self) -> bool {
        true
    }
    /// Where and under which format this environment admits the current-time operand, `now`, in an
    /// ordering over a `Timestamp` (beyond10x/ess#171, `ess/16`).
    ///
    /// Admitted everywhere by default, for the reason [`Self::admits_text_length`] is: the
    /// compiler's environment reads an IR that validation already admitted. [`DomainEnvironment`]
    /// admits it only where a command outcome's input guard (`when:`) is checked, under `ess/16`
    /// or later.
    fn current_time(&self) -> CurrentTimeAdmission {
        CurrentTimeAdmission {
            site: true,
            format: true,
        }
    }
    /// Whether a one-segment fact that no binder names may stand on the right of a comparison —
    /// the operand the canonical form writes as `{fact: …}`, which `ess/22` introduced
    /// (`docs/design/expression-family-source22.md`, A1).
    ///
    /// `true` by default, for the reason [`Self::admits_text_length`] is.
    fn admits_root_facts(&self) -> bool {
        true
    }
    /// Whether this environment checks an authored source whose bare words name roots (`ess/22`
    /// or later): a binder that shadows a root is then refused where a bare word would read it,
    /// and a quoted word naming a root is refused with the unquoted repair.
    ///
    /// `false` by default: an IR keeps no bare words, and a binder that shadowed a root in an
    /// admitted older source keeps meaning the binder.
    fn resolves_bare_words(&self) -> bool {
        false
    }
    /// One declared struct member, without using wire aliases.
    fn member(&self, reference: &Self::Type, name: &str) -> Option<Self::Type>;
    /// Whether the reserved filter parameter namespace exists in this environment.
    fn has_parameters(&self) -> bool {
        false
    }
    /// One declared filter parameter.
    fn parameter(&self, _name: &str) -> Option<Self::Type> {
        None
    }
    /// The root that names the parameter namespace: `param` for a view filter, `input` for a
    /// stored-field predicate reading the command's input (beyond10x/ess#157).
    fn parameter_namespace(&self) -> &'static str {
        "param"
    }
    /// Whether `caller.<attribute>` reads the authenticated caller in this environment (ess/16,
    /// beyond10x/ess#168). A root field named `caller` keeps being read as itself.
    fn has_caller(&self) -> bool {
        false
    }
    /// One attribute every actor that may invoke the command declares.
    fn caller_attribute(&self, _name: &str) -> Option<Self::Type> {
        None
    }
}

/// Whether a predicate site admits the current-time operand (beyond10x/ess#171), in two parts
/// because they are refused differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentTimeAdmission {
    /// The site is a command outcome's input guard (a `when:`, alone or beside a held state, a
    /// state change, a stored field, a `when_subject:` or an external cause): the predicate over a
    /// request's input, read while it is being handled, which is the moment `now` names.
    pub site: bool,
    /// The specification's format is `ess/16` or later.
    pub format: bool,
}

/// Required traversal operations, independent of projection capability.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Access {
    /// The path requires collection cardinality or element projection.
    pub collection: bool,
    /// The path reads the length of a text (`keys.count` over a `String`, ess/11).
    ///
    /// Apart from [`Self::collection`], which a producer reads as "count the elements": a text
    /// length is a leaf read the evaluator derives from the text itself.
    pub text_length: bool,
    /// Transparent unwraps and member/element descents required by the path.
    pub depth: usize,
}

/// The result of one complete path lookup, tied to the checked environment.
#[derive(Debug, Clone)]
pub struct Resolution<T> {
    /// The terminal type after transparent unwrapping.
    pub terminal: T,
    /// The declared terminal reference, retaining its nominal name for diagnostics.
    pub declared: String,
    /// Scalar representation; None means a resolved aggregate.
    pub scalar: Option<ScalarKind>,
    /// Closed enum vocabulary, when the scalar is enum-backed.
    pub variants: Option<Vec<String>>,
    /// Whether any traversal passed through Optional.
    pub optional: bool,
    /// Traversal required of a producer.
    pub access: Access,
}

/// An ordered semantic failure, converted to the existing diagnostic envelope at the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionError {
    /// Existing domain error classification.
    pub code: ValidationCode,
    /// The owner declaration and predicate location.
    pub owner: String,
    /// The full original fact path, when the failure concerns a path.
    pub path: Option<FactPath>,
    /// The first selector that could not be resolved.
    pub segment: Option<String>,
    /// An aggregate boundary crossed by a forbidden selector, for existing projection refusals.
    pub boundary: Option<&'static str>,
    /// The complete diagnostic without a fabricated source span.
    pub message: String,
}

impl ExpressionError {
    /// Convert without changing the public diagnostic wire shape.
    pub fn validation_error(&self) -> ValidationError {
        ValidationError::new(self.code, self.owner.clone(), self.message.clone())
    }
}

/// One read checked directly from the AST, including quantified bodies.
#[derive(Debug, Clone)]
pub struct Read<T> {
    /// Original path, retaining lexical binder names.
    pub path: FactPath,
    /// Whether the root is free rather than a lexical binder.
    pub free: bool,
    /// Whether this read is a quantifier target rather than a scalar operand.
    pub collection_target: bool,
    /// Complete type and access information.
    pub resolution: Resolution<T>,
}

/// Transient results of checking one predicate against one environment.
#[derive(Debug, Clone)]
pub struct Checked<T> {
    /// Semantic errors in stable AST order.
    pub errors: Vec<ExpressionError>,
    /// Successfully resolved reads in stable AST order.
    pub reads: Vec<Read<T>>,
    /// Free parameter names used at any depth, excluding shadowed binders.
    pub parameters: BTreeSet<String>,
    /// Every `Timestamp` path ordered against the current-time operand (beyond10x/ess#171), in
    /// stable AST order: what a consumer that cannot read a clock refuses by name.
    pub current_time: Vec<FactPath>,
}

impl<T> Checked<T> {
    /// Existing domain errors, with owner locations preserved.
    pub fn validation_errors(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        for error in &self.errors {
            errors.push(error.validation_error());
        }
        errors
    }
}

/// Declaration adapter with owner fields and an optional filter parameter namespace.
pub struct DomainEnvironment<'a> {
    registry: &'a TypeRegistry,
    fields: &'a [Field],
    params: Option<&'a [Field]>,
    namespace: &'static str,
    current_time: bool,
    caller: Option<&'a [Field]>,
}

impl<'a> DomainEnvironment<'a> {
    /// Observe these owner fields using the complete registry.
    pub fn new(registry: &'a TypeRegistry, fields: &'a [Field]) -> Self {
        Self {
            registry,
            fields,
            params: None,
            namespace: "param",
            current_time: false,
            caller: None,
        }
    }
    /// Enable the `caller` namespace over the attributes every actor that may invoke the command
    /// declares (ess/16, beyond10x/ess#168).
    #[must_use]
    pub fn with_caller(mut self, attributes: &'a [Field]) -> Self {
        self.caller = Some(attributes);
        self
    }
    /// Enable the reserved param namespace, including when no parameters are declared.
    #[must_use]
    pub fn with_params(mut self, params: &'a [Field]) -> Self {
        self.params = Some(params);
        self
    }
    /// Enable the `input` namespace over a command's input fields, for a stored-field predicate
    /// comparing the subject with the input (beyond10x/ess#157). Replaces any param namespace.
    #[must_use]
    pub fn with_input(mut self, input: &'a [Field]) -> Self {
        self.params = Some(input);
        self.namespace = "input";
        self
    }
    /// Admit the current-time operand, `now`, where the format does (`ess/16`): the environment a
    /// command outcome's input guard (`when:`) is checked in (beyond10x/ess#171).
    #[must_use]
    pub fn with_current_time(mut self) -> Self {
        self.current_time = true;
        self
    }
}

impl TypeEnvironment for DomainEnvironment<'_> {
    fn is_instant(&self, reference: &TypeRef) -> bool {
        matches!(reference, TypeRef::Primitive(Primitive::Timestamp))
    }
    fn is_duration(&self, reference: &TypeRef) -> bool {
        matches!(reference, TypeRef::Primitive(Primitive::Duration))
    }
    fn is_string(&self, reference: &TypeRef) -> bool {
        matches!(reference, TypeRef::Primitive(Primitive::String))
    }
    fn is_integer(&self, reference: &TypeRef) -> bool {
        matches!(reference, TypeRef::Primitive(Primitive::Integer))
    }
    fn admits_text_length(&self) -> bool {
        self.registry
            .format()
            .is_none_or(|format| format.major() >= crate::system::FormatVersion::V11.major())
    }
    fn admits_aggregate_presence(&self) -> bool {
        self.registry
            .format()
            .is_none_or(|format| format.major() >= crate::system::FormatVersion::V16.major())
    }
    fn admits_root_facts(&self) -> bool {
        self.registry
            .format()
            .is_none_or(|format| format.major() >= crate::system::FormatVersion::V22.major())
    }
    fn resolves_bare_words(&self) -> bool {
        self.registry
            .format()
            .is_some_and(|format| format.major() >= crate::system::FormatVersion::V22.major())
    }
    fn is_clock_reading(&self, reference: &TypeRef) -> bool {
        matches!(reference, TypeRef::Named(name) if self.registry.get(name).is_some_and(|declared| declared.reading.is_some()))
    }
    fn current_time(&self) -> CurrentTimeAdmission {
        CurrentTimeAdmission {
            site: self.current_time,
            format: self
                .registry
                .format()
                .is_none_or(|format| format.major() >= crate::system::FormatVersion::V16.major()),
        }
    }
    type Type = TypeRef;

    fn root(&self, name: &str) -> Option<TypeRef> {
        self.fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.type_ref.clone())
    }
    fn cardinality_type(&self) -> TypeRef {
        TypeRef::Primitive(Primitive::Integer)
    }
    fn shape(&self, reference: &TypeRef) -> Result<Shape<TypeRef>, String> {
        Ok(match reference {
            TypeRef::Primitive(primitive) => {
                ScalarKind::of(*primitive).map_or(Shape::Json, Shape::Scalar)
            }
            TypeRef::Optional(of) => Shape::Optional((**of).clone()),
            TypeRef::List(of) => Shape::List((**of).clone()),
            TypeRef::Map(_, value) => Shape::Map((**value).clone()),
            TypeRef::Named(name) => match &self
                .registry
                .get(name)
                .ok_or_else(|| name.to_string())?
                .body
            {
                TypeBody::Newtype { of, .. } => Shape::Alias(of.clone()),
                TypeBody::Struct { .. } => Shape::Struct,
                TypeBody::Enum { variants } => Shape::Enum(
                    variants
                        .iter()
                        .map(|variant| variant.name().to_owned())
                        .collect(),
                ),
                TypeBody::Union { .. } => Shape::Union,
            },
        })
    }
    fn member(&self, reference: &TypeRef, name: &str) -> Option<TypeRef> {
        let TypeRef::Named(declared) = reference else {
            return None;
        };
        let TypeBody::Struct { fields, .. } = &self.registry.get(declared)?.body else {
            return None;
        };
        fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.type_ref.clone())
    }
    fn has_parameters(&self) -> bool {
        self.params.is_some()
    }
    fn parameter_namespace(&self) -> &'static str {
        self.namespace
    }
    fn parameter(&self, name: &str) -> Option<TypeRef> {
        self.params?
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.type_ref.clone())
    }
    fn has_caller(&self) -> bool {
        self.caller.is_some()
    }
    fn caller_attribute(&self, name: &str) -> Option<TypeRef> {
        self.caller?
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.type_ref.clone())
    }
}

#[derive(Clone)]
struct Binding<T> {
    name: String,
    reference: Option<T>,
    access: Access,
    optional: bool,
}

fn error(
    owner: &str,
    code: ValidationCode,
    path: Option<&FactPath>,
    segment: Option<&str>,
    message: String,
) -> ExpressionError {
    ExpressionError {
        code,
        owner: owner.to_owned(),
        path: path.cloned(),
        segment: segment.map(str::to_owned),
        boundary: None,
        message,
    }
}

/// Resolve a complete free path, without imposing a projection or witness depth limit.
pub fn resolve_path<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
    owner: &str,
) -> Result<Resolution<E::Type>, ExpressionError> {
    resolve(environment, path, owner, &[])
}

struct Cursor<T> {
    current: T,
    position: usize,
    optional: bool,
    access: Access,
    context: String,
}

fn root_cursor<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
    owner: &str,
    bindings: &[Binding<E::Type>],
) -> Result<Cursor<E::Type>, ExpressionError> {
    let segments = path.segments();
    let root = path.namespace();
    let binder = bindings.iter().rev().find(|binding| binding.name == root);
    let context = binder.map_or(String::new(), |binding| {
        format!(" in binder `{}`", binding.name)
    });
    let mut position = 1;
    let optional = binder.is_some_and(|binding| binding.optional);
    let access = binder.map_or(Access::default(), |binding| binding.access);
    let current = if let Some(binding) = binder {
        binding.reference.clone()
    } else if environment.has_parameters() && root == environment.parameter_namespace() {
        let (space, article, noun) = if root == "param" {
            ("parameter", "a", "parameter")
        } else {
            (root, "an", "input field")
        };
        let Some(name) = segments.get(1) else {
            return Err(error(
                owner,
                ValidationCode::UnobservableFact,
                Some(path),
                Some(root),
                format!("`{path}` names the {space} namespace without {article} {noun} selector"),
            ));
        };
        position = 2;
        Some(environment.parameter(name).ok_or_else(|| {
            error(
                owner,
                ValidationCode::UndeclaredReference,
                Some(path),
                Some(name),
                format!("`{path}` reads undeclared {noun} `{name}`"),
            )
        })?)
    } else if environment.has_caller()
        && root == crate::command::caller_value::CALLER_NAMESPACE
        && environment.root(root).is_none()
    {
        position = 2;
        Some(caller_root(environment, path, owner)?)
    } else {
        environment.root(root)
    }
    .ok_or_else(|| {
        error(
            owner,
            ValidationCode::UnobservableFact,
            Some(path),
            Some(root),
            format!("`{path}` reads `{root}`, which is not a declared observable root{context}"),
        )
    })?;
    Ok(Cursor {
        current,
        position,
        optional,
        access,
        context,
    })
}

/// The type of the caller attribute `caller.<attribute>` names (ess/16, beyond10x/ess#168).
fn caller_root<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
    owner: &str,
) -> Result<E::Type, ExpressionError> {
    let root = path.namespace();
    let Some(name) = path.segments().get(1) else {
        return Err(error(
            owner,
            ValidationCode::UnobservableFact,
            Some(path),
            Some(root),
            format!("`{path}` names the caller without an attribute selector"),
        ));
    };
    environment.caller_attribute(name).ok_or_else(|| {
        error(
            owner,
            ValidationCode::UndeclaredReference,
            Some(path),
            Some(name),
            format!(
                "`{path}` reads caller attribute `{name}`, which not every actor that may \
                 invoke the command declares"
            ),
        )
    })
}

fn refuse_clock_reading<E: TypeEnvironment>(
    environment: &E,
    current: &E::Type,
    path: &FactPath,
    owner: &str,
) -> Result<(), ExpressionError> {
    if environment.is_clock_reading(current) {
        return Err(error(
                owner,
                ValidationCode::TypeMismatch,
                Some(path),
                None,
                format!(
                    "`{path}` is a clock reading; comparison requires observed source, epoch and formatter evidence"
                ),
            ));
    }
    Ok(())
}

fn resolve<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
    owner: &str,
    bindings: &[Binding<E::Type>],
) -> Result<Resolution<E::Type>, ExpressionError> {
    let segments = path.segments();
    let Cursor {
        mut current,
        mut position,
        mut optional,
        mut access,
        context,
    } = root_cursor(environment, path, owner, bindings)?;
    let (mut declared, mut seen) = (current.to_string(), BTreeSet::new());
    loop {
        refuse_clock_reading(environment, &current, path, owner)?;
        let identity = current.to_string();
        if !seen.insert((identity.clone(), position)) {
            return Err(error(owner, ValidationCode::SelfReference, Some(path), segments.get(position).map(String::as_str),
                format!("`{path}` cannot resolve through the non-progress type cycle at `{identity}`{context}; traversed {}", seen.iter().map(|(name,_)| name.as_str()).collect::<Vec<_>>().join(", "))));
        }
        let shape = environment.shape(&current).map_err(|name| {
            error(
                owner,
                ValidationCode::UndeclaredReference,
                Some(path),
                segments
                    .get(position)
                    .or_else(|| segments.last())
                    .map(String::as_str),
                format!("`{path}` reaches `{name}`, which is not a declared type{context}"),
            )
        })?;
        match shape {
            Shape::Alias(of) => {
                current = of;
                access.depth += 1;
            }
            Shape::Optional(of) => {
                current = of;
                optional = true;
                access.depth += 1;
            }
            shape => {
                let Some(segment) = segments.get(position) else {
                    let (scalar, variants) = match shape {
                        Shape::Scalar(kind) => (Some(kind), None),
                        Shape::Enum(variants) => (Some(ScalarKind::Text), Some(variants)),
                        _ => (None, None),
                    };
                    return Ok(Resolution {
                        terminal: current,
                        declared,
                        scalar,
                        variants,
                        optional,
                        access,
                    });
                };
                let is_struct = matches!(shape, Shape::Struct);
                let boundary = match &shape {
                    Shape::List(_) => Some("a list"),
                    Shape::Map(_) => Some("a map"),
                    Shape::Union => Some("a union"),
                    Shape::Json => Some("a JSON value"),
                    _ => None,
                };
                let next = match shape {
                    Shape::Struct => environment.member(&current, segment),
                    Shape::Scalar(ScalarKind::Text)
                        if segment == "count" && environment.is_string(&current) =>
                    {
                        let at = (position, optional, access);
                        return text_length(environment, path, owner, at, &context);
                    }
                    Shape::List(_) | Shape::Map(_) if segment == "count" => {
                        access.collection = true;
                        if let Some(next) = segments.get(position + 1) {
                            return Err(error(owner, ValidationCode::UnobservableFact, Some(path), Some(next),
                                format!("`{path}` cannot select `{next}` from collection count of type Integer (Number){context}")));
                        }
                        return Ok(count_of(environment, optional, access));
                    }
                    Shape::List(of) if canonical_ordinal(segment) => {
                        access.collection = true;
                        Some(of)
                    }
                    _ => None,
                };
                current = next.ok_or_else(|| {
                    let message = if is_struct {
                        format!("`{path}`: `{identity}` has no field `{segment}` (declared prefix `{declared}`){context}")
                    } else {
                        format!("`{path}` cannot select `{segment}` from `{declared}` (resolved prefix type `{identity}`){context}")
                    };
                    let mut failed = error(owner, ValidationCode::UnobservableFact, Some(path), Some(segment), message);
                    failed.boundary = boundary;
                    failed
                })?;
                declared = current.to_string();
                position += 1;
                access.depth += 1;
            }
        }
    }
}

/// `.count` on a `String` at `(position, optional, access)`: the length of a text in Unicode scalar
/// values (beyond10x/ess#104).
///
/// Only a `String` has one — asked of the resolved terminal, so a newtype of one at any depth and
/// an `Optional` of one are admitted — and every other text scalar falls through to the
/// `cannot select` refusal it always had.
fn text_length<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
    owner: &str,
    (position, optional, mut access): (usize, bool, Access),
    context: &str,
) -> Result<Resolution<E::Type>, ExpressionError> {
    access.text_length = true;
    text_length_refusal(environment, path, position, owner, context)
        .map(|()| count_of(environment, optional, access))
}

/// A `.count` — of a collection or of a text — resolves to an `Integer`.
fn count_of<E: TypeEnvironment>(
    environment: &E,
    optional: bool,
    access: Access,
) -> Resolution<E::Type> {
    Resolution {
        terminal: environment.cardinality_type(),
        declared: "Integer".to_owned(),
        scalar: Some(ScalarKind::Number),
        variants: None,
        optional,
        access,
    }
}

/// The two refusals a text length can meet: a format before ess/11, and a selector past it.
fn text_length_refusal<E: TypeEnvironment>(
    environment: &E,
    path: &FactPath,
    position: usize,
    owner: &str,
    context: &str,
) -> Result<(), ExpressionError> {
    let segments = path.segments();
    let through = FactPath::from_segments(&segments[..=position]);
    if !environment.admits_text_length() {
        return Err(error(
            owner,
            ValidationCode::UnsupportedFormatVersion,
            Some(path),
            segments.get(position).map(String::as_str),
            format!("`{through}`: the length of a String requires specification format ess/11"),
        ));
    }
    if let Some(next) = segments.get(position + 1) {
        return Err(error(
            owner,
            ValidationCode::UnobservableFact,
            Some(path),
            Some(next),
            format!(
                "`{through}` is a text length of type Integer (Number); `{next}` selects nothing \
                 from it{context}"
            ),
        ));
    }
    Ok(())
}

fn canonical_ordinal(segment: &str) -> bool {
    segment == "0"
        || (!segment.starts_with('0')
            && !segment.is_empty()
            && segment.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Decides every bare word of an authored predicate against `environment`
/// (`docs/design/expression-family-source22.md`, A1), returning the resolved predicate.
///
/// In this order, for an unquoted, undotted word on the right of a comparison that no binder in
/// scope names (a binder was already read as the binder, #289):
///
/// 1. the left side is enum-backed and declares the word as a variant: the variant, so an admitted
///    `state == Open` keeps its meaning even beside a root named `Open`;
/// 2. the word is exactly an observable root of the environment: that fact — a root named `now`
///    included, which is why the current-time reading needs no root of that name;
/// 3. the text is `<binder | root | dotted path> ws? (+|-) ws? <magnitude>` whose base resolves here
///    and whose magnitude reads ([`OffsetMagnitude::parse`]): one constant offset of that fact (A2,
///    final review decision 4, rule 3a) — whatever the base's type, which the checker holds to
///    `Integer` or `Timestamp`;
/// 4. otherwise the text it always was.
///
/// Nothing is refused here. [`check_predicate`] checks the result as it checks any predicate.
pub fn resolve_lexical<E: TypeEnvironment>(
    environment: &E,
    lexical: &lexical::LexicalPredicate,
) -> Predicate {
    resolve_lexical_reading(environment, lexical, false)
}

/// [`resolve_lexical`], with `input_namespace` saying whether `input.<path>` names the input
/// `<path>` here — a plain input guard whose command declares no root named `input`, which
/// [`read_input_namespace`] rewrites afterwards (decision 6) — so an offset's base written
/// `input.lower` resolves as `lower` does.
pub(crate) fn resolve_lexical_reading<E: TypeEnvironment>(
    environment: &E,
    lexical: &lexical::LexicalPredicate,
    input_namespace: bool,
) -> Predicate {
    let resolved = lexical.lower_scoped(
        &mut |left, _, spelling, scope| {
            let bindings = scope_bindings(environment, scope);
            let word = match spelling {
                lexical::Spelling::Word(word) => word,
                // A dotted path with a `-`: the fact it reads where it names one, else the offset
                // it also spells (`window.lower-5`), else the fact it always was, which the checker
                // refuses as unobservable as before.
                lexical::Spelling::Dotted(path) => {
                    if base_resolves(environment, &bindings, path, input_namespace) {
                        return Operand::Fact(path.clone());
                    }
                    return offset_spelled(
                        environment,
                        &bindings,
                        &path.to_string(),
                        input_namespace,
                    )
                    .map_or_else(|| Operand::Fact(path.clone()), Operand::Offset);
                }
            };
            let variant = match left {
                Operand::Fact(path) => resolve(environment, path, "", &bindings)
                    .ok()
                    .and_then(|resolved| resolved.variants)
                    .is_some_and(|variants| variants.iter().any(|variant| variant == word)),
                Operand::Literal(_) | Operand::Offset(_) => false,
            };
            let text = Operand::Literal(FactValue::Text(word.to_owned()));
            if variant {
                return text;
            }
            if environment.root(word).is_some() {
                if let Ok(path) = FactPath::new(word) {
                    return Operand::Fact(path);
                }
            }
            offset_spelled(environment, &bindings, word, input_namespace)
                .map_or(text, Operand::Offset)
        },
        &mut Vec::new(),
    );
    tag_instants(environment, resolved, &mut Vec::new())
}

/// The first way `text` splits into `<base> ± <magnitude>` ([`OffsetOperand::spellings`]) whose base
/// resolves under `bindings` and whose magnitude reads, as the offset it spells.
fn offset_spelled<E: TypeEnvironment>(
    environment: &E,
    bindings: &[Binding<E::Type>],
    text: &str,
    input_namespace: bool,
) -> Option<OffsetOperand> {
    OffsetOperand::spellings(text)
        .into_iter()
        .find_map(|(base, direction, magnitude)| {
            let magnitude = OffsetMagnitude::parse(magnitude)?;
            base_resolves(environment, bindings, &base, input_namespace).then_some(OffsetOperand {
                base,
                direction,
                magnitude,
            })
        })
}

/// Whether an offset's base names something here: a binder in scope, a root, or a dotted path
/// through either — and, with `input_namespace`, `input.<path>` naming the input `<path>`.
fn base_resolves<E: TypeEnvironment>(
    environment: &E,
    bindings: &[Binding<E::Type>],
    base: &FactPath,
    input_namespace: bool,
) -> bool {
    if resolve(environment, base, "", bindings).is_ok() {
        return true;
    }
    let namespace = crate::command::subject_fact::INPUT_NAMESPACE;
    input_namespace
        && base.namespace() == namespace
        && base.segments().len() > 1
        && !bindings.iter().any(|binding| binding.name == namespace)
        && resolve(
            environment,
            &FactPath::from_segments(&base.segments()[1..]),
            "",
            bindings,
        )
        .is_ok()
}

/// Tags every comparison of two facts that both resolve to `Timestamp` to compare instants
/// (`docs/design/expression-family-source22.md`, final review decision 2), so a reader with no
/// declared types compares the instants and never the spellings. Everything else is unchanged.
fn tag_instants<E: TypeEnvironment>(
    environment: &E,
    predicate: Predicate,
    scope: &mut Vec<(FactPath, String)>,
) -> Predicate {
    match predicate {
        Predicate::Compare {
            left: Operand::Fact(left),
            op,
            right: Operand::Fact(right),
            kind: CompareKind::Value,
        } => {
            let pairs: Vec<(&FactPath, &str)> = scope
                .iter()
                .map(|(over, bind)| (over, bind.as_str()))
                .collect();
            let bindings = bindings_of(environment, &pairs);
            let instant = |path: &FactPath| {
                resolve(environment, path, "", &bindings).is_ok_and(|resolved| {
                    resolved.scalar.is_some() && environment.is_instant(&resolved.terminal)
                })
            };
            let kind = if instant(&left) && instant(&right) {
                CompareKind::Instant
            } else {
                CompareKind::Value
            };
            Predicate::Compare {
                left: Operand::Fact(left),
                op,
                right: Operand::Fact(right),
                kind,
            }
        }
        Predicate::All(children) => Predicate::All(
            children
                .into_iter()
                .map(|child| tag_instants(environment, child, scope))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .into_iter()
                .map(|child| tag_instants(environment, child, scope))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(tag_instants(environment, *inner, scope))),
        Predicate::Forall(quantified) => {
            Predicate::Forall(Box::new(tag_quantified(environment, *quantified, scope)))
        }
        Predicate::Exists(quantified) => {
            Predicate::Exists(Box::new(tag_quantified(environment, *quantified, scope)))
        }
        other => other,
    }
}

fn tag_quantified<E: TypeEnvironment>(
    environment: &E,
    quantified: Quantified,
    scope: &mut Vec<(FactPath, String)>,
) -> Quantified {
    scope.push((quantified.over.clone(), quantified.bind.clone()));
    let body = tag_instants(environment, quantified.body, scope);
    scope.pop();
    Quantified {
        over: quantified.over,
        bind: quantified.bind,
        body,
    }
}

/// The binders `scope` introduces, typed as [`Checker::quantified`] types them.
fn scope_bindings<E: TypeEnvironment>(
    environment: &E,
    scope: &[&lexical::LexicalQuantified],
) -> Vec<Binding<E::Type>> {
    let pairs: Vec<(&FactPath, &str)> = scope
        .iter()
        .map(|quantified| (&quantified.over, quantified.bind.as_str()))
        .collect();
    bindings_of(environment, &pairs)
}

/// The binders of `(collection, binder)` pairs, outermost first, typed as
/// [`Checker::quantified`] types them.
fn bindings_of<E: TypeEnvironment>(
    environment: &E,
    scope: &[(&FactPath, &str)],
) -> Vec<Binding<E::Type>> {
    let mut bindings: Vec<Binding<E::Type>> = Vec::new();
    for (over, bind) in scope {
        let target = resolve(environment, over, "", &bindings).ok();
        let reference =
            target
                .as_ref()
                .and_then(|target| match environment.shape(&target.terminal) {
                    Ok(Shape::List(element) | Shape::Map(element)) if target.scalar.is_none() => {
                        Some(element)
                    }
                    _ => None,
                });
        bindings.push(Binding {
            name: (*bind).to_owned(),
            reference,
            access: Access {
                collection: true,
                text_length: false,
                depth: target.as_ref().map_or(0, |target| target.access.depth + 1),
            },
            optional: target.is_some_and(|target| target.optional),
        });
    }
    bindings
}

/// A command's input guard with `input.<path>` read as the input `<path>` names
/// (`docs/design/expression-family-source22.md`, A1, decision 6).
///
/// For a command that declares no input field named `input`, the caller's guard: the plain `when:`
/// reads the command's input as its roots, so `input.depends_on` is `depends_on`. A path under a
/// binder named `input` is the binder's and is left alone, and so is a bare `input`, which the
/// checker refuses as the root it is not.
pub fn read_input_namespace(predicate: &Predicate) -> Predicate {
    fn strip(path: &FactPath, bound: &[&str]) -> FactPath {
        let namespace = crate::command::subject_fact::INPUT_NAMESPACE;
        if path.namespace() == namespace && path.segments().len() > 1 && !bound.contains(&namespace)
        {
            FactPath::from_segments(&path.segments()[1..])
        } else {
            path.clone()
        }
    }
    fn operand(operand: &Operand, bound: &[&str]) -> Operand {
        operand.map_path(|path| strip(path, bound))
    }
    fn walk<'a>(predicate: &'a Predicate, bound: &mut Vec<&'a str>) -> Predicate {
        match predicate {
            Predicate::Always | Predicate::Never => predicate.clone(),
            Predicate::All(children) => {
                Predicate::All(children.iter().map(|child| walk(child, bound)).collect())
            }
            Predicate::Any(children) => {
                Predicate::Any(children.iter().map(|child| walk(child, bound)).collect())
            }
            Predicate::Not(inner) => Predicate::Not(Box::new(walk(inner, bound))),
            Predicate::Compare {
                left,
                op,
                right,
                kind,
            } => Predicate::Compare {
                kind: *kind,
                left: operand(left, bound),
                op: *op,
                right: operand(right, bound),
            },
            Predicate::Truthy(path) => Predicate::Truthy(strip(path, bound)),
            Predicate::Defined(path) => Predicate::Defined(strip(path, bound)),
            Predicate::AnyOf { path, values } => Predicate::AnyOf {
                path: strip(path, bound),
                values: values.clone(),
            },
            Predicate::NoneOf { path, values } => Predicate::NoneOf {
                path: strip(path, bound),
                values: values.clone(),
            },
            Predicate::TextMatch { path, op, value } => Predicate::TextMatch {
                path: strip(path, bound),
                op: *op,
                value: value.clone(),
            },
            Predicate::FoldMatch { path, op, values } => Predicate::FoldMatch {
                path: strip(path, bound),
                op: *op,
                values: values.clone(),
            },
            Predicate::Forall(quantified) => {
                Predicate::Forall(Box::new(quantifier(quantified, bound)))
            }
            Predicate::Exists(quantified) => {
                Predicate::Exists(Box::new(quantifier(quantified, bound)))
            }
        }
    }
    fn quantifier<'a>(quantified: &'a Quantified, bound: &mut Vec<&'a str>) -> Quantified {
        let over = strip(&quantified.over, bound);
        bound.push(&quantified.bind);
        let body = walk(&quantified.body, bound);
        bound.pop();
        Quantified {
            over,
            bind: quantified.bind.clone(),
            body,
        }
    }
    walk(predicate, &mut Vec::new())
}

/// Check every child and operand without evaluating or rewriting the predicate.
pub fn check_predicate<E: TypeEnvironment>(
    environment: &E,
    predicate: &Predicate,
    owner: &str,
) -> Checked<E::Type> {
    let mut checker = Checker {
        environment,
        owner,
        bindings: Vec::new(),
        checked: Checked {
            errors: Vec::new(),
            reads: Vec::new(),
            parameters: BTreeSet::new(),
            current_time: Vec::new(),
        },
    };
    checker.predicate(predicate);
    checker.checked
}

struct Checker<'a, E: TypeEnvironment> {
    environment: &'a E,
    owner: &'a str,
    bindings: Vec<Binding<E::Type>>,
    checked: Checked<E::Type>,
}

// Each flag is one question the environment answers of the terminal type, asked by a different
// comparison rule; `command.rs` keeps its outcome flags the same way.
#[allow(clippy::struct_excessive_bools)]
struct ValueType {
    declared: String,
    /// The type that declares `variants`, which is not always `declared`.
    ///
    /// `declared` is the fact's *declared* type; through a newtype that is the wrapper, and the
    /// variants belong to the enum the wrapper transparently aliases. Naming only the wrapper
    /// leaves the reader to open it, read its `of:`, and open the enum — and the enum is the
    /// declaration they have to edit to make the literal legal.
    declaring_variants: Option<String>,
    scalar: Option<ScalarKind>,
    variants: Option<Vec<String>>,
    /// Whether the terminal type is `Timestamp`, ordered by the RFC 3339 instant it names.
    instant: bool,
    /// Whether the terminal type is `Duration`, which has no ordering.
    duration: bool,
    /// Whether the terminal type is `String`, which a string operator applies to.
    string: bool,
    /// Whether the terminal type is `Integer`, which an Integer offset moves (A2).
    integer: bool,
}

impl<E: TypeEnvironment> Checker<'_, E> {
    fn read(&mut self, path: &FactPath, collection_target: bool) -> Option<Resolution<E::Type>> {
        let binder = self
            .bindings
            .iter()
            .rev()
            .find(|binding| binding.name == path.namespace());
        let free = binder.is_none();
        if binder.is_some_and(|binding| binding.reference.is_none()) {
            return None;
        }
        if free
            && self.environment.has_parameters()
            && path.namespace() == self.environment.parameter_namespace()
        {
            if let Some(name) = path.segments().get(1) {
                self.checked.parameters.insert(name.clone());
            }
        }
        match resolve(self.environment, path, self.owner, &self.bindings) {
            Ok(resolution) => {
                self.checked.reads.push(Read {
                    path: path.clone(),
                    free,
                    collection_target,
                    resolution: resolution.clone(),
                });
                Some(resolution)
            }
            Err(error) => {
                self.checked.errors.push(error);
                None
            }
        }
    }

    /// What a resolved read is, as an operand.
    fn typed(&self, resolved: Resolution<E::Type>) -> ValueType {
        ValueType {
            instant: self.environment.is_instant(&resolved.terminal),
            duration: self.environment.is_duration(&resolved.terminal),
            string: self.environment.is_string(&resolved.terminal),
            integer: self.environment.is_integer(&resolved.terminal),
            declaring_variants: resolved
                .variants
                .is_some()
                .then(|| resolved.terminal.to_string()),
            declared: resolved.declared,
            scalar: resolved.scalar,
            variants: resolved.variants,
        }
    }

    fn operand(&mut self, operand: &Operand) -> Option<ValueType> {
        match operand {
            Operand::Fact(path) => self.read(path, false).map(|resolved| self.typed(resolved)),
            // An offset is a value of its base's type, which `offset` holds to the magnitude.
            Operand::Offset(offset) => self
                .read(&offset.base, false)
                .map(|resolved| self.typed(resolved)),
            Operand::Literal(value) => {
                let scalar = ScalarKind::literal(value);
                Some(ValueType {
                    declared: format!("{scalar} literal `{value}`"),
                    declaring_variants: None,
                    scalar: Some(scalar),
                    variants: None,
                    instant: false,
                    duration: false,
                    string: false,
                    integer: false,
                })
            }
        }
    }

    fn mismatch(
        &mut self,
        expression: &Predicate,
        operator: &str,
        left: &ValueType,
        right: Option<&ValueType>,
    ) {
        let describe = |value: &ValueType| {
            format!(
                "`{}` ({})",
                value.declared,
                value
                    .scalar
                    .map_or_else(|| "aggregate".to_owned(), |kind| kind.to_string())
            )
        };
        let operands = right.map_or_else(
            || describe(left),
            |right| format!("{} and {}", describe(left), describe(right)),
        );
        let unreadable = match expression {
            Predicate::Truthy(path) | Predicate::Defined(path) => Some(path),
            Predicate::AnyOf { path, .. }
            | Predicate::NoneOf { path, .. }
            | Predicate::TextMatch { path, .. }
            | Predicate::FoldMatch { path, .. }
                if left.scalar.is_none() =>
            {
                Some(path)
            }
            Predicate::Compare {
                left: Operand::Fact(path),
                ..
            } if left.scalar.is_none() => Some(path),
            Predicate::Compare {
                right: Operand::Fact(path),
                ..
            } if right.is_some_and(|value| value.scalar.is_none()) => Some(path),
            _ => None,
        };
        self.checked.errors.push(error(
            self.owner,
            ValidationCode::TypeMismatch,
            unreadable,
            None,
            format!("`{expression}`: operator `{operator}` does not admit {operands}"),
        ));
    }

    fn enum_literal(&mut self, expression: &Predicate, typed: &ValueType, literal: &FactValue) {
        let (Some(variants), FactValue::Text(text)) = (&typed.variants, literal) else {
            return;
        };
        if !variants.contains(text) {
            let path = match expression {
                Predicate::Compare {
                    left: Operand::Fact(path),
                    ..
                }
                | Predicate::Compare {
                    right: Operand::Fact(path),
                    ..
                }
                | Predicate::AnyOf { path, .. }
                | Predicate::NoneOf { path, .. } => path.to_string(),
                _ => typed.declared.clone(),
            };
            let names = variants
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ");
            // The enum, not the field's declared type: `story:enum-variant-in-an-entity-invariant`
            // asks that the refusal name the declaration the author has to edit, so the fix is one
            // lookup away. Through a newtype those differ, and the wrapper is kept beside the enum
            // rather than in place of it — the reader still needs to know how the field reached it.
            let declaring = typed
                .declaring_variants
                .as_deref()
                .unwrap_or(&typed.declared);
            let reached = if declaring == typed.declared {
                String::new()
            } else {
                format!(", reached through `{}`", typed.declared)
            };
            self.checked.errors.push(error(self.owner, ValidationCode::UndeclaredReference, None, None,
                format!("`{expression}` compares `{path}` to `{text}`, which `{declaring}` (Text{reached}) does not declare as an enum variant; values: {names}")));
        }
    }

    /// A text literal compared with a fact, held to what the evaluator will actually do with it.
    ///
    /// A right-hand side without a dot is a literal, so `ends_at > starts_at` compares `ends_at`
    /// with the text `"starts_at"`. Validation accepted that and synthesis could not decide it
    /// (beyond10x/ess#74), so a bare word naming a declared field — and not a variant of the enum
    /// it is compared with — is refused with the spelling that reads the field. An ordering against
    /// a `Timestamp` needs an RFC 3339 instant, which is the only text synthesis can order.
    fn text_literal(
        &mut self,
        expression: &Predicate,
        fact: &Operand,
        op: CompareOp,
        literal: &Operand,
        typed: &ValueType,
    ) {
        let (Operand::Fact(path), Operand::Literal(FactValue::Text(text))) = (fact, literal) else {
            return;
        };
        let enum_variant = typed
            .variants
            .as_ref()
            .is_some_and(|variants| variants.contains(text));
        if !enum_variant && self.environment.root(text).is_some() {
            let message = if self.environment.resolves_bare_words() {
                format!(
                    "`{expression}` reads `{text}` as the text literal \"{text}\", not the field \
                     `{text}`: a quoted word, like the equality shorthand, is always text. To \
                     compare with the field, write it unquoted, such as `{path} {op} {text}`"
                )
            } else {
                format!(
                    "`{expression}` reads `{text}` as the text literal \"{text}\", not the field \
                     `{text}`: a right-hand side without a dot is a literal. To compare two fields, \
                     declare them in one struct and compare its members, such as \
                     `window.{path} {op} window.{text}`"
                )
            };
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(path),
                None,
                message,
            ));
            return;
        }
        // An unquoted bare word naming a binder in scope already reads the binder
        // (beyond10x/ess#289). What is still text here was quoted or written in the equality
        // shorthand, which is always a literal: the same misread, refused the same way.
        if !enum_variant && self.bindings.iter().any(|binding| binding.name == *text) {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(path),
                None,
                format!(
                    "`{expression}` reads `{text}` as the text literal \"{text}\", not the binder \
                     `{text}`: the equality shorthand and a quoted word are always text. To compare \
                     with the binder, write it bare in a comparison, such as `{path} {op} {text}`"
                ),
            ));
            return;
        }
        // Only where text is no value of the fact anyway: a `String` compared with `read-only` beside
        // a field named `read` keeps its text, quoted or not (rule 3a reads only a magnitude).
        let offset_typed = typed.instant || typed.scalar == Some(ScalarKind::Number);
        if !enum_variant && offset_typed && self.malformed_offset(expression, path, text) {
            return;
        }
        if typed.instant && self.current_time_literal(expression, path, op, text) {
            return;
        }
        if typed.instant
            && op.needs_ordering()
            && ess_primitives::time::Rfc3339Instant::parse_rfc3339(text).is_none()
        {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(path),
                None,
                format!(
                    "`{expression}` orders the Timestamp `{path}` against \"{text}\", which is \
                     not an RFC 3339 instant; write one such as \"2020-01-01T00:00:00Z\""
                ),
            ));
        }
    }

    /// A text literal written as the current-time operand against the `Timestamp` `path`
    /// (beyond10x/ess#171, `ess/16`): recorded where it is admitted, refused by what it lacks
    /// elsewhere. `true` when this decided the literal; `false` leaves it to the rules every other
    /// text literal meets, which is what a format before `ess/16` keeps for everything but an
    /// ordering in a `when:` — so an equality with the word `now` means what it meant there.
    fn current_time_literal(
        &mut self,
        expression: &Predicate,
        path: &FactPath,
        op: CompareOp,
        text: &str,
    ) -> bool {
        use ess_primitives::time::CurrentTime;
        if !CurrentTime::mentions(text) {
            return false;
        }
        let admission = self.environment.current_time();
        let parsed = CurrentTime::parse(text).is_some();
        let (code, message, hint) = match (op.needs_ordering(), parsed, admission) {
            (
                true,
                true,
                CurrentTimeAdmission {
                    site: true,
                    format: true,
                },
            ) => {
                self.checked.current_time.push(path.clone());
                return true;
            }
            (
                true,
                true,
                CurrentTimeAdmission {
                    site: true,
                    format: false,
                },
            ) => (
                ValidationCode::UnsupportedFormatVersion,
                format!(
                    "`{expression}` orders the Timestamp `{path}` against the current time, \
                     `{text}`, which requires specification format ess/16"
                ),
                "write `format: ess/16` on the source that declares the system".to_owned(),
            ),
            // A site that refuses the operand names where it is admitted, whatever else is wrong
            // with the comparison, and never suggests writing it again here.
            (
                ordering,
                parsed,
                CurrentTimeAdmission {
                    site: false,
                    format: true,
                },
            ) if ordering || parsed => (
                ValidationCode::TypeMismatch,
                format!(
                    "`{expression}` compares the Timestamp `{path}` with the current time, \
                     `{text}`, which is admitted only in a command outcome's `when:` over its \
                     input: that is the one predicate read while a request is being handled, and \
                     an invariant, a view filter, a selection or a `when_subject:` predicate over \
                     stored fields is not"
                ),
                "compare with a fixed RFC 3339 instant here, such as \"2020-01-01T00:00:00Z\", \
                 or move the rule into the command's `when:`"
                    .to_owned(),
            ),
            (true, false, CurrentTimeAdmission { format: true, .. }) => (
                ValidationCode::TypeMismatch,
                format!(
                    "`{expression}` orders the Timestamp `{path}` against \"{text}\", which is \
                     neither an RFC 3339 instant nor the current time; write {} (a whole number \
                     of seconds, minutes or hours, at most {} seconds either way)",
                    CurrentTime::SPELLINGS,
                    CurrentTime::MAX_OFFSET_SECONDS
                ),
                "a day is written `24h`: the offset has no calendar".to_owned(),
            ),
            (false, true, CurrentTimeAdmission { format: true, .. }) => (
                ValidationCode::TypeMismatch,
                format!(
                    "`{expression}` compares the Timestamp `{path}` with the current time by \
                     `{op}`; an instant is ordered against now, never equated with it, because \
                     no request arrives at exactly the moment it names"
                ),
                format!("order it, such as `{path} >= {text}`"),
            ),
            _ => return false,
        };
        self.checked.errors.push(error(
            self.owner,
            code,
            Some(path),
            None,
            format!("{message}; {hint}"),
        ));
        true
    }

    /// Refuses an ordering over a `Duration`, which has none: its ISO 8601 text would put `PT10M`
    /// below `PT5M` under the byte order text is compared by. One refusal per comparison.
    fn duration_ordering(&mut self, predicate: &Predicate, operands: [(&Operand, &ValueType); 2]) {
        let Some((operand, _)) = operands.into_iter().find(|(_, typed)| typed.duration) else {
            return;
        };
        self.checked.errors.push(error(
            self.owner,
            ValidationCode::TypeMismatch,
            operand.fact_path(),
            None,
            format!(
                "`{predicate}`: a Duration has no ordering, because its ISO 8601 text would put \
                 `PT10M` below `PT5M`; compare it with `==` or `!=`, or declare the length as a \
                 number, such as whole seconds in an Integer, to order it"
            ),
        ));
    }

    fn quantified(&mut self, predicate: &Predicate, quantified: &Quantified) {
        let target = self.read(&quantified.over, true);
        let mut reference = None;
        let mut access = Access {
            collection: true,
            text_length: false,
            depth: 0,
        };
        let mut optional = false;
        if let Some(target) = target {
            access.depth = target.access.depth + 1;
            optional = target.optional;
            match self.environment.shape(&target.terminal) {
                Ok(Shape::List(element) | Shape::Map(element)) if target.scalar.is_none() => {
                    reference = Some(element);
                }
                _ => {
                    // No `ValueType` is built here. This message names the type the author *wrote*
                    // and its scalar kind, and nothing on this path reads `variants` or
                    // `declaring_variants`: `enum_literal` is reached only from
                    // `Predicate::Compare`, never from a quantifier target. Computing the enum's
                    // own name for a field no consumer reads is what adversary pass 2, A6, found —
                    // a value written and never read reads as a promise this message keeps, and it
                    // does not keep it.
                    self.checked.errors.push(error(self.owner, ValidationCode::TypeMismatch, None, None,
                                format!("`{predicate}`: quantifier target `{}` is `{}` ({}) and not a collection; Forall/Exists require List or Map",
                                    quantified.over, target.declared, target.scalar.map_or_else(|| "aggregate".to_owned(), |kind| kind.to_string()))));
                }
            }
        }
        self.bindings.push(Binding {
            name: quantified.bind.clone(),
            reference,
            access,
            optional,
        });
        self.predicate(&quantified.body);
        self.bindings.pop();
    }

    /// One comparison: its operands agree in kind, and every ordering, enum and text-literal rule
    /// that applies to them holds.
    /// The format gate and the operand rule of a comparison tagged to compare instants
    /// (`docs/design/expression-family-source22.md`, final review decision 2): from `ess/22`, and
    /// only between two facts that both resolve to `Timestamp`. `false` when the format refuses
    /// it, so it is not checked a second time.
    fn tagged_instants(
        &mut self,
        predicate: &Predicate,
        left: &Operand,
        right: &Operand,
        kind: CompareKind,
    ) -> bool {
        if kind != CompareKind::Instant {
            return true;
        }
        if !self.environment.admits_root_facts() {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::UnsupportedFormatVersion,
                left.fact_path(),
                None,
                format!(
                    "`{predicate}` is tagged to compare instants, written `as: timestamp`, which \
                     requires specification format ess/22"
                ),
            ));
            return false;
        }
        for operand in [left, right] {
            let instant = match operand {
                Operand::Fact(path) => {
                    let bindings = self.bindings.clone();
                    resolve(self.environment, path, self.owner, &bindings).is_ok_and(|resolved| {
                        resolved.scalar.is_some() && self.environment.is_instant(&resolved.terminal)
                    })
                }
                Operand::Literal(_) | Operand::Offset(_) => false,
            };
            if !instant {
                self.checked.errors.push(error(
                    self.owner,
                    ValidationCode::TypeMismatch,
                    operand.fact_path(),
                    None,
                    format!(
                        "`{predicate}` is tagged to compare instants, and `{operand}` is not a \
                         Timestamp fact; only two Timestamp facts compare `as: timestamp`"
                    ),
                ));
            }
        }
        true
    }

    fn compare(&mut self, predicate: &Predicate, left: &Operand, op: CompareOp, right: &Operand) {
        match (left, right) {
            (_, Operand::Offset(offset)) => return self.offset(predicate, left, offset),
            (Operand::Offset(offset), _) => {
                self.checked.errors.push(error(
                    self.owner,
                    ValidationCode::TypeMismatch,
                    Some(&offset.base),
                    None,
                    format!(
                        "`{predicate}` puts the offset `{offset}` on the left; one constant offset \
                         of a fact stands on the right of a comparison only, such as \
                         `upper == lower + 5`"
                    ),
                ));
                return;
            }
            _ => {}
        }
        if !self.root_fact_operand(predicate, right) {
            return;
        }
        let left_type = self.operand(left);
        let right_type = self.operand(right);
        if let (Some(left_type), Some(right_type)) = (left_type, right_type) {
            let compatible = left_type.scalar.is_some()
                && left_type.scalar == right_type.scalar
                && (matches!(op, CompareOp::Eq | CompareOp::Ne)
                    || left_type.scalar != Some(ScalarKind::Bool));
            if !compatible {
                self.mismatch(predicate, &op.to_string(), &left_type, Some(&right_type));
            }
            if op.needs_ordering() {
                self.duration_ordering(predicate, [(left, &left_type), (right, &right_type)]);
            }
            if let Operand::Literal(value) = right {
                self.enum_literal(predicate, &left_type, value);
            }
            if let Operand::Literal(value) = left {
                self.enum_literal(predicate, &right_type, value);
            }
            self.text_literal(predicate, left, op, right, &left_type);
            self.text_literal(predicate, right, op, left, &right_type);
            if let (Operand::Fact(left_path), Operand::Fact(right_path)) = (left, right) {
                if compatible && op.needs_ordering() && left_type.instant != right_type.instant {
                    let (instant, text) = if left_type.instant {
                        (left_path, right_path)
                    } else {
                        (right_path, left_path)
                    };
                    self.checked.errors.push(error(
                        self.owner,
                        ValidationCode::TypeMismatch,
                        Some(text),
                        None,
                        format!(
                            "`{predicate}`: cannot order the Timestamp `{instant}` against \
                             `{text}`, which is not a Timestamp; a Timestamp is ordered only \
                             against another Timestamp or an RFC 3339 instant literal"
                        ),
                    ));
                }
            }
        }
    }

    /// One constant offset on the right (`docs/design/expression-family-source22.md`, A2): from
    /// `ess/22`; an Integer magnitude between an `Integer` fact on the left and an `Integer` base,
    /// an elapsed one between two `Timestamp` facts, each through newtypes and `Optional`. All six
    /// operators are admitted. `Decimal` and `Binary64` are refused rather than coerced.
    fn offset(&mut self, predicate: &Predicate, left: &Operand, offset: &OffsetOperand) {
        if !self.environment.admits_root_facts() {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::UnsupportedFormatVersion,
                Some(&offset.base),
                None,
                format!(
                    "`{predicate}` compares with one constant offset of `{}`, written `{offset}`, \
                     which requires specification format ess/22",
                    offset.base
                ),
            ));
            return;
        }
        let Operand::Fact(_) = left else {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(&offset.base),
                None,
                format!(
                    "`{predicate}` compares a literal with an offset; the left of an offset \
                     comparison is a fact"
                ),
            ));
            return;
        };
        let left_type = self.operand(left);
        let base_type = self.operand(&Operand::Fact(offset.base.clone()));
        let (Some(left_type), Some(base_type)) = (left_type, base_type) else {
            return;
        };
        let describe = |value: &ValueType| {
            format!(
                "`{}` ({})",
                value.declared,
                value
                    .scalar
                    .map_or_else(|| "aggregate".to_owned(), |kind| kind.to_string())
            )
        };
        let (fits, wanted) = match offset.magnitude {
            OffsetMagnitude::Integer(_) => (
                left_type.integer
                    && base_type.integer
                    && left_type.scalar.is_some()
                    && base_type.scalar.is_some()
                    && offset.magnitude.integer().is_some(),
                "an Integer offset — a whole number from 0 to 9223372036854775807 — compares two \
                 Integer facts; Decimal and Binary64 are refused rather than coerced",
            ),
            OffsetMagnitude::ElapsedSeconds { seconds, .. } => (
                left_type.instant
                    && base_type.instant
                    && left_type.scalar.is_some()
                    && base_type.scalar.is_some()
                    && (0..=ess_primitives::time::CurrentTime::MAX_OFFSET_SECONDS)
                        .contains(&seconds),
                "an elapsed offset — a whole number of `s`, `m` or `h` — compares two Timestamp \
                 facts",
            ),
        };
        if !fits {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(&offset.base),
                None,
                format!(
                    "`{predicate}`: {wanted}, and it reads {} against {}",
                    describe(&left_type),
                    describe(&base_type)
                ),
            ));
        }
    }

    /// From `ess/22`, a text literal spelled `<fact> ± <something>` whose base names a fact here,
    /// compared with an `Integer` or a `Timestamp` (`docs/design/expression-family-source22.md`, A2,
    /// rule 3a): what reaches the checker as text is either quoted or a magnitude that does not read
    /// — `lower + 05`, `lower + 5 + 3`, `issued_at - 1d` — and is refused naming the offset grammar
    /// rather than as a mere text against a number. Against a `String` such text is the text it
    /// always was; the caller asks only where text is no value of the fact. `true` when refused.
    fn malformed_offset(&mut self, expression: &Predicate, path: &FactPath, text: &str) -> bool {
        if !self.environment.resolves_bare_words() {
            return false;
        }
        let bindings = self.bindings.clone();
        let Some((base, _, magnitude)) = OffsetOperand::spellings(text)
            .into_iter()
            .find(|(base, _, _)| base_resolves(self.environment, &bindings, base, false))
        else {
            return false;
        };
        let reason = if OffsetMagnitude::parse(magnitude).is_some() {
            "quoted, it is the text, which would compare with the spelling rather than the fact; \
             write it unquoted"
                .to_owned()
        } else {
            format!(
                "`{magnitude}` is no offset magnitude: an Integer offset is a whole number without \
                 a sign, a fraction or a leading zero, and a Timestamp offset a whole number of \
                 `s`, `m` or `h` (a day is `24h`), at most {} seconds; one offset per comparison",
                ess_primitives::time::CurrentTime::MAX_OFFSET_SECONDS
            )
        };
        self.checked.errors.push(error(
            self.owner,
            ValidationCode::TypeMismatch,
            Some(path),
            None,
            format!(
                "`{expression}` reads \"{text}\" as text, but it is spelled as an offset of the fact \
                 `{base}`: {reason}"
            ),
        ));
        true
    }

    /// The format gate and the shadowing rule for a one-segment fact on the right
    /// (`docs/design/expression-family-source22.md`, A1, decision 5). `false` when refused here,
    /// so the comparison is not checked a second time.
    fn root_fact_operand(&mut self, predicate: &Predicate, right: &Operand) -> bool {
        let Operand::Fact(path) = right else {
            return true;
        };
        if path.segments().len() != 1 {
            return true;
        }
        let name = path.namespace();
        let bound = self.bindings.iter().any(|binding| binding.name == name);
        if !bound && !self.environment.admits_root_facts() {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::UnsupportedFormatVersion,
                Some(path),
                None,
                format!(
                    "`{predicate}` compares with the fact `{name}` on its right, written \
                     `{{fact: {name}}}`, which requires specification format ess/22"
                ),
            ));
            return false;
        }
        if bound && self.environment.resolves_bare_words() && self.environment.root(name).is_some()
        {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(path),
                None,
                format!(
                    "`{predicate}` reads `{name}` as the binder in scope, which shadows the field \
                     `{name}`: from ess/22 a bare word on the right names a field, so a binder of \
                     the same name would make one spelling mean two things; rename the binder"
                ),
            ));
            return false;
        }
        true
    }

    /// A string operator (beyond10x/ess#95): a `String` fact, or a newtype of one at any depth,
    /// against a text literal that is not empty and does not name a field.
    fn text_match(
        &mut self,
        predicate: &Predicate,
        path: &FactPath,
        op: TextOp,
        value: &FactValue,
    ) {
        if let Some(typed) = self.operand(&Operand::Fact(path.clone())) {
            if !typed.string {
                self.mismatch(predicate, op.keyword(), &typed, None);
            }
        }
        let text = match value {
            FactValue::Text(text) => text,
            other => {
                self.checked.errors.push(error(
                    self.owner,
                    ValidationCode::TypeMismatch,
                    Some(path),
                    None,
                    format!(
                        "`{predicate}`: the operand is a {}, not text; `{op}` compares with a text \
                         literal. YAML reads an unquoted scalar such as `+44` as a number and \
                         `true` as a Boolean before ESS sees it, so the spelling it had is gone: \
                         quote the literal exactly as written",
                        other.type_name()
                    ),
                ));
                return;
            }
        };
        if text.is_empty() {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::EmptyDeclaration,
                Some(path),
                None,
                format!(
                    "`{predicate}` holds for every text, and its negation for none; write \
                     `defined({path})` if presence is meant"
                ),
            ));
            return;
        }
        // The #74 refusal, as for `==`: a bare word naming a declared field reads as that text.
        if self.environment.root(text).is_some() {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(path),
                None,
                format!(
                    "`{predicate}` reads `{text}` as the text literal \"{text}\", not the field \
                     `{text}`: a string operator compares with a literal only"
                ),
            ));
        }
    }

    /// A case-insensitive operator (beyond10x/ess#140): a `String` fact, or a newtype of one at any
    /// depth, against text literals — at least one of them, none naming a field. The empty text is a
    /// literal like any other here: `equals_ignore_case: ""` is the empty text, which folding does
    /// not change.
    fn fold_match(
        &mut self,
        predicate: &Predicate,
        path: &FactPath,
        op: FoldOp,
        values: &[FactValue],
    ) {
        if let Some(typed) = self.operand(&Operand::Fact(path.clone())) {
            if !typed.string {
                self.mismatch(predicate, op.keyword(), &typed, None);
            }
        }
        if values.is_empty() {
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::EmptyDeclaration,
                Some(path),
                None,
                format!("`{predicate}` lists no literal, so it holds for no text"),
            ));
            return;
        }
        for value in values {
            let FactValue::Text(text) = value else {
                self.checked.errors.push(error(
                    self.owner,
                    ValidationCode::TypeMismatch,
                    Some(path),
                    None,
                    format!(
                        "`{predicate}`: the operand {value} is a {}, not text; `{op}` compares with \
                         text literals. YAML reads an unquoted scalar such as `44` as a number and \
                         `true` as a Boolean before ESS sees it: quote the literal exactly as \
                         written",
                        value.type_name()
                    ),
                ));
                continue;
            };
            // The #74 refusal, as for `==`: a bare word naming a declared field reads as that text.
            if self.environment.root(text).is_some() {
                self.checked.errors.push(error(
                    self.owner,
                    ValidationCode::TypeMismatch,
                    Some(path),
                    None,
                    format!(
                        "`{predicate}` reads `{text}` as the text literal \"{text}\", not the field \
                         `{text}`: `{op}` compares with literals only"
                    ),
                ));
            }
        }
    }

    fn predicate(&mut self, predicate: &Predicate) {
        match predicate {
            Predicate::Always | Predicate::Never => {}
            Predicate::TextMatch { path, op, value } => {
                self.text_match(predicate, path, *op, value);
            }
            Predicate::FoldMatch { path, op, values } => {
                self.fold_match(predicate, path, *op, values);
            }
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    self.predicate(child);
                }
            }
            Predicate::Not(inner) => self.predicate(inner),
            Predicate::Compare {
                left,
                op,
                right,
                kind,
            } => {
                if self.tagged_instants(predicate, left, right, *kind) {
                    self.compare(predicate, left, *op, right);
                }
            }
            Predicate::Truthy(path) | Predicate::Defined(path) => {
                let Some(resolved) = self.read(path, false) else {
                    return;
                };
                if resolved.scalar.is_some() {
                    return;
                }
                let optional = resolved.optional;
                let value = self.typed(resolved);
                let defined = matches!(predicate, Predicate::Defined(_));
                // Presence is a property of the `Optional`, not of what it holds (beyond10x/ess#176):
                // an `Optional` aggregate is present or absent like an `Optional` scalar is.
                if defined && optional {
                    if !self.environment.admits_aggregate_presence() {
                        self.checked.errors.push(error(
                            self.owner,
                            ValidationCode::UnsupportedFormatVersion,
                            Some(path),
                            None,
                            format!(
                                "`{predicate}`: `defined()` over an Optional aggregate (`{}`) \
                                 requires specification format ess/16",
                                value.declared
                            ),
                        ));
                    }
                    return;
                }
                self.mismatch(
                    predicate,
                    if defined { "Defined" } else { "Truthy" },
                    &value,
                    None,
                );
            }
            Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
                let Some(typed) = self.operand(&Operand::Fact(path.clone())) else {
                    return;
                };
                let op = if matches!(predicate, Predicate::AnyOf { .. }) {
                    "AnyOf"
                } else {
                    "NoneOf"
                };
                if typed.scalar.is_none() {
                    self.mismatch(predicate, op, &typed, None);
                    return;
                }
                for value in values {
                    let kind = ScalarKind::literal(value);
                    if typed.scalar != Some(kind) {
                        let literal = ValueType {
                            declared: format!("{kind} literal `{value}`"),
                            declaring_variants: None,
                            scalar: Some(kind),
                            variants: None,
                            instant: false,
                            duration: false,
                            string: false,
                            integer: false,
                        };
                        self.mismatch(predicate, op, &typed, Some(&literal));
                    }
                    self.enum_literal(predicate, &typed, value);
                }
            }
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                self.quantified(predicate, quantified);
            }
        }
    }
}
