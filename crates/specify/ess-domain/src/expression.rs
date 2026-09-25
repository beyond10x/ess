//! Shared semantic admission for the existing predicate tree.
//!
//! Resolution describes required accesses; it does not promise that a producer publishes them,
//! that an optional value exists, or that any witness satisfies the predicate.

use std::collections::BTreeSet;
use std::fmt;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Quantified};

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
    /// The existing evaluator representation of a declared primitive.
    pub fn of(primitive: Primitive) -> Self {
        match primitive {
            Primitive::Boolean => Self::Bool,
            Primitive::Integer | Primitive::Decimal | Primitive::Binary64 => Self::Number,
            Primitive::String
            | Primitive::Timestamp
            | Primitive::Duration
            | Primitive::Uuid
            | Primitive::Bytes => Self::Text,
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
}

/// Required traversal operations, independent of projection capability.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Access {
    /// The path requires collection cardinality or element projection.
    pub collection: bool,
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
}

impl<'a> DomainEnvironment<'a> {
    /// Observe these owner fields using the complete registry.
    pub fn new(registry: &'a TypeRegistry, fields: &'a [Field]) -> Self {
        Self {
            registry,
            fields,
            params: None,
        }
    }
    /// Enable the reserved param namespace, including when no parameters are declared.
    #[must_use]
    pub fn with_params(mut self, params: &'a [Field]) -> Self {
        self.params = Some(params);
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
    fn is_clock_reading(&self, reference: &TypeRef) -> bool {
        matches!(reference, TypeRef::Named(name) if self.registry.get(name).is_some_and(|declared| declared.reading.is_some()))
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
            TypeRef::Primitive(primitive) => Shape::Scalar(ScalarKind::of(*primitive)),
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
    fn parameter(&self, name: &str) -> Option<TypeRef> {
        self.params?
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
    } else if environment.has_parameters() && root == "param" {
        let Some(name) = segments.get(1) else {
            return Err(error(
                owner,
                ValidationCode::UnobservableFact,
                Some(path),
                Some(root),
                format!("`{path}` names the parameter namespace without a parameter selector"),
            ));
        };
        position = 2;
        Some(environment.parameter(name).ok_or_else(|| {
            error(
                owner,
                ValidationCode::UndeclaredReference,
                Some(path),
                Some(name),
                format!("`{path}` reads undeclared parameter `{name}`"),
            )
        })?)
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
                    _ => None,
                };
                let next = match shape {
                    Shape::Struct => environment.member(&current, segment),
                    Shape::List(_) | Shape::Map(_) if segment == "count" => {
                        access.collection = true;
                        if let Some(next) = segments.get(position + 1) {
                            return Err(error(owner, ValidationCode::UnobservableFact, Some(path), Some(next),
                                format!("`{path}` cannot select `{next}` from collection count of type Integer (Number){context}")));
                        }
                        return Ok(Resolution {
                            terminal: environment.cardinality_type(),
                            declared: "Integer".to_owned(),
                            scalar: Some(ScalarKind::Number),
                            variants: None,
                            optional,
                            access,
                        });
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

fn canonical_ordinal(segment: &str) -> bool {
    segment == "0"
        || (!segment.starts_with('0')
            && !segment.is_empty()
            && segment.bytes().all(|byte| byte.is_ascii_digit()))
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
        if free && self.environment.has_parameters() && path.namespace() == "param" {
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

    fn operand(&mut self, operand: &Operand) -> Option<ValueType> {
        match operand {
            Operand::Fact(path) => self.read(path, false).map(|resolved| ValueType {
                instant: self.environment.is_instant(&resolved.terminal),
                duration: self.environment.is_duration(&resolved.terminal),
                declaring_variants: resolved
                    .variants
                    .is_some()
                    .then(|| resolved.terminal.to_string()),
                declared: resolved.declared,
                scalar: resolved.scalar,
                variants: resolved.variants,
            }),
            Operand::Literal(value) => {
                let scalar = ScalarKind::literal(value);
                Some(ValueType {
                    declared: format!("{scalar} literal `{value}`"),
                    declaring_variants: None,
                    scalar: Some(scalar),
                    variants: None,
                    instant: false,
                    duration: false,
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
            Predicate::AnyOf { path, .. } | Predicate::NoneOf { path, .. }
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
            self.checked.errors.push(error(
                self.owner,
                ValidationCode::TypeMismatch,
                Some(path),
                None,
                format!(
                    "`{expression}` reads `{text}` as the text literal \"{text}\", not the field \
                     `{text}`: a right-hand side without a dot is a literal. To compare two fields, \
                     declare them in one struct and compare its members, such as \
                     `window.{path} {op} window.{text}`"
                ),
            ));
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

    /// Refuses an ordering over a `Duration`, which has none: its ISO 8601 text would put `PT10M`
    /// below `PT5M` under the byte order text is compared by. One refusal per comparison.
    fn duration_ordering(&mut self, predicate: &Predicate, operands: [(&Operand, &ValueType); 2]) {
        let Some((operand, _)) = operands.into_iter().find(|(_, typed)| typed.duration) else {
            return;
        };
        self.checked.errors.push(error(
            self.owner,
            ValidationCode::TypeMismatch,
            match operand {
                Operand::Fact(path) => Some(path),
                Operand::Literal(_) => None,
            },
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
    fn compare(&mut self, predicate: &Predicate, left: &Operand, op: CompareOp, right: &Operand) {
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

    fn predicate(&mut self, predicate: &Predicate) {
        match predicate {
            Predicate::Always | Predicate::Never => {}
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    self.predicate(child);
                }
            }
            Predicate::Not(inner) => self.predicate(inner),
            Predicate::Compare { left, op, right } => self.compare(predicate, left, *op, right),
            Predicate::Truthy(path) | Predicate::Defined(path) => {
                if let Some(value) = self.operand(&Operand::Fact(path.clone())) {
                    if value.scalar.is_none() {
                        self.mismatch(
                            predicate,
                            if matches!(predicate, Predicate::Truthy(_)) {
                                "Truthy"
                            } else {
                                "Defined"
                            },
                            &value,
                            None,
                        );
                    }
                }
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
