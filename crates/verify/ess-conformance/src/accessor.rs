//! Wire-observable accessor expectations, separate from native Optional reconstruction.
use ess_compiler::ir::{EssIr, ResolvedTypeRef, TypeHandle};
use ess_domain::{
    accessor::{AccessorPlan, Operation, ValueShape},
    TypeRef,
};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

/// Exact target field assertion; absence never disappears into a subset comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expected {
    /// The input member must not occur.
    Absent,
    /// The input member must occur with this value (including explicit null).
    Present(Node),
}

/// An admitted observable subset of a typed assignment.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Observation {
    /// Shared typed traversal, with declared member spellings.
    pub plan: AccessorPlan,
    /// Exact assignment target, including each Optional layer.
    pub target: TypeRef,
    /// Closed nominal facts proving that hidden Optional states cannot change this observation.
    pub types: crate::accessor_types::TypeFacts,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObservation {
    plan: AccessorPlan,
    target: TypeRef,
    types: crate::accessor_types::TypeFacts,
}
impl<'de> serde::Deserialize<'de> for Observation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = <RawObservation as serde::Deserialize>::deserialize(d)?;
        let value = Self {
            plan: raw.plan,
            target: raw.target,
            types: raw.types,
        };
        value.assignment().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

pub(crate) fn unresolve(ty: &ResolvedTypeRef) -> TypeRef {
    match ty {
        ResolvedTypeRef::Primitive { name } => TypeRef::Primitive(*name),
        ResolvedTypeRef::Declared { name } => TypeRef::Named(name.name().clone()),
        ResolvedTypeRef::Optional { of } => TypeRef::Optional(Box::new(unresolve(of))),
        ResolvedTypeRef::List { of } => TypeRef::List(Box::new(unresolve(of))),
        ResolvedTypeRef::Map { key, value } => TypeRef::Map(*key, Box::new(unresolve(value))),
    }
}

impl Observation {
    /// Plan a mechanically decidable observation, without guessing a host conversion.
    pub fn of(
        ir: &EssIr,
        plan: &AccessorPlan,
        types: &BTreeMap<ess_domain::QualifiedName, TypeHandle>,
        target: &ResolvedTypeRef,
    ) -> Result<Self, String> {
        let mut leaf = plan.leaf();
        while let TypeRef::Optional(of) = leaf {
            leaf = of;
        }
        let result = Self {
            plan: plan.clone(),
            target: unresolve(target),
            types: crate::accessor_types::TypeFacts::of(ir, &resolve(leaf, types))?,
        };
        result.assignment()?;
        Ok(result)
    }

    fn assignment(&self) -> Result<(usize, usize), String> {
        self.plan.validate().map_err(|e| e.to_string())?;
        self.assignment_graph()
    }

    fn assignment_graph(&self) -> Result<(usize, usize), String> {
        for node in &self.plan.nodes {
            let mut shape = &node.shape;
            while let ValueShape::Optional { of } = shape {
                shape = of;
            }
            if matches!(
                shape,
                ValueShape::Primitive {
                    name: ess_domain::Primitive::Binary64
                }
            ) {
                return Err("unsupported Binary64 accessor observation".into());
            }
        }
        self.types.validate(self.plan.leaf())?;
        self.bytes()?;
        if self.plan.may_miss() && !matches!(self.target, TypeRef::Optional(_)) {
            return Err("partial accessor requires Optional target".into());
        }
        let leaf = self.plan.leaf();
        let mut target = &self.target;
        let mut added = 0;
        while target != leaf {
            let TypeRef::Optional(of) = target else {
                return Err("accessor observation has no identity-first assignment".into());
            };
            added += 1;
            target = of;
        }
        let mut source = leaf;
        let mut depth = 0;
        while let TypeRef::Optional(of) = source {
            depth += 1;
            source = of;
        }
        if depth >= 2 && added == 0 {
            return Err(
                "AmbiguousOptionalObservation: nested terminal Optional with equal target".into(),
            );
        }
        Ok((depth, added))
    }

    /// Serialized size of the complete observation plan, including nominal facts.
    pub fn bytes(&self) -> Result<usize, String> {
        struct Counter(usize);
        impl std::io::Write for Counter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0 = self.0.saturating_add(bytes.len());
                if self.0 > 1_048_576 {
                    return Err(std::io::Error::other("accessor observation byte bound"));
                }
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut count = Counter(0);
        serde_json::to_writer(&mut count, self)
            .map_err(|_| "AccessorResource: observation plan exceeds 1048576 bytes".to_owned())?;
        Ok(count.0)
    }

    /// Evaluate only wire-observable facts, retaining missing/null/value until assignment.
    pub fn evaluate(&self, payload: &BTreeMap<String, Node>) -> Result<Expected, String> {
        let (depth, added) = self.assignment()?;
        self.evaluate_assignment(payload.get(&self.plan.root.name), depth, added)
    }

    fn evaluate_assignment(
        &self,
        value: Option<&Node>,
        depth: usize,
        added: usize,
    ) -> Result<Expected, String> {
        let Projected::Terminal(value) = project(&self.plan, value)? else {
            return Ok(Expected::Absent);
        };
        if value.is_none() || matches!(value, Some(Node::Null)) {
            if depth == 0 {
                return Err("required accessor terminal is missing or null".into());
            }
            return Ok(if added == 0 {
                Expected::Absent
            } else {
                Expected::Present(Node::Null)
            });
        }
        let value = value.expect("non-null terminal");
        let mut ty = self.plan.leaf();
        while let TypeRef::Optional(of) = ty {
            ty = of;
        }
        if !shape(ty, value) {
            return Err("accessor terminal has wrong payload kind".into());
        }
        Ok(Expected::Present(value.clone()))
    }
}

/// Raw presence after declared traversal, before any target assignment.
pub(crate) enum Projected<'a> {
    Unavailable,
    Terminal(Option<&'a Node>),
}

/// One shared bounded DAG interpreter for event and selection-item observations.
pub(crate) fn project<'a>(
    plan: &AccessorPlan,
    mut value: Option<&'a Node>,
) -> Result<Projected<'a>, String> {
    let mut id = plan.start;
    for _ in 0..ess_domain::accessor::MAX_OPERATIONS {
        let node = &plan.nodes[id];
        if let Some(value) = value {
            if !wire_shape(&node.shape, value) {
                return Err("accessor value has wrong declared payload kind".into());
            }
        } else if !matches!(node.shape, ValueShape::Optional { .. }) {
            return Err("required accessor member is missing".into());
        }
        match &node.operation {
            Operation::Missing => return Ok(Projected::Unavailable),
            Operation::Leaf => return Ok(Projected::Terminal(value)),
            Operation::Optional { next } => {
                if value.is_none() || matches!(value, Some(Node::Null)) {
                    return Ok(Projected::Unavailable);
                }
                id = *next;
            }
            Operation::Newtype { next } => {
                if value.is_none() {
                    return Err("required accessor wrapper is missing".into());
                }
                id = *next;
            }
            Operation::Field { field, next } => {
                let Some(Node::Map(fields)) = value else {
                    return Err("accessor requires a declared struct object".into());
                };
                value = fields.get(&field.name);
                id = *next;
            }
            Operation::Union { tag, variants } => {
                let Some(Node::Map(fields)) = value else {
                    return Err("accessor requires a tagged union object".into());
                };
                let Some(Node::Text(label)) = fields.get(tag) else {
                    return Err("accessor union discriminator missing or not text".into());
                };
                id = *variants
                    .get(label)
                    .ok_or("accessor union discriminator is unknown")?;
                value = Some(
                    fields
                        .get(ess_gen::schema::union_content_key(tag))
                        .ok_or("accessor union payload is missing")?,
                );
                if matches!(value, Some(Node::Null))
                    && !matches!(plan.nodes[id].shape, ValueShape::Optional { .. })
                {
                    return Err("required union payload is null".into());
                }
            }
        }
    }
    Err("accessor operation budget exceeded".into())
}

fn shape(ty: &TypeRef, value: &Node) -> bool {
    match ty {
        TypeRef::Primitive(p) => crate::scenario::Holds::Primitive { kind: *p }.admits(value),
        TypeRef::List(_) => matches!(value, Node::Seq(_)),
        TypeRef::Map(_, _) => matches!(value, Node::Map(_)),
        TypeRef::Named(_) => true,
        TypeRef::Optional(_) => false,
    }
}

/// Whether this executable suite contains the new observation vocabulary.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| scenario.steps.iter().any(|step| matches!(step, crate::ScenarioStep::ExpectInvocation { input, .. } if input.values().any(|v| matches!(v, crate::ScenarioValue::ObservedAccessor { .. })))))
}

fn wire_shape(shape: &ValueShape, value: &Node) -> bool {
    match shape {
        ValueShape::Optional { of } => matches!(value, Node::Null) || wire_shape(of, value),
        ValueShape::Primitive { name } => {
            crate::scenario::Holds::Primitive { kind: *name }.admits(value)
        }
        ValueShape::Enum { variants } => {
            matches!(value, Node::Text(text) if variants.contains(text))
        }
        ValueShape::Object => matches!(value, Node::Map(_)),
        ValueShape::Sequence => matches!(value, Node::Seq(_)),
    }
}

fn resolve(
    ty: &TypeRef,
    types: &BTreeMap<ess_domain::QualifiedName, TypeHandle>,
) -> ResolvedTypeRef {
    match ty {
        TypeRef::Primitive(name) => ResolvedTypeRef::Primitive { name: *name },
        TypeRef::Named(name) => ResolvedTypeRef::Declared {
            name: types[name].clone(),
        },
        TypeRef::Optional(of) => ResolvedTypeRef::Optional {
            of: Box::new(resolve(of, types)),
        },
        TypeRef::List(of) => ResolvedTypeRef::List {
            of: Box::new(resolve(of, types)),
        },
        TypeRef::Map(key, value) => ResolvedTypeRef::Map {
            key: *key,
            value: Box::new(resolve(value, types)),
        },
    }
}

/// An item-root projection with the same observable Optional assignment rules.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProjectionObservation {
    /// Separately admitted zero-to-three-member item projection.
    pub projection: ess_domain::accessor::ProjectionPlan,
    /// Exact target type.
    pub target: TypeRef,
    /// Nominal Optional observability facts.
    pub types: crate::accessor_types::TypeFacts,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProjectionObservation {
    projection: ess_domain::accessor::ProjectionPlan,
    target: TypeRef,
    types: crate::accessor_types::TypeFacts,
}
impl<'de> serde::Deserialize<'de> for ProjectionObservation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = <RawProjectionObservation as serde::Deserialize>::deserialize(d)?;
        let result = Self {
            projection: raw.projection,
            target: raw.target,
            types: raw.types,
        };
        result.validate().map_err(serde::de::Error::custom)?;
        Ok(result)
    }
}
impl ProjectionObservation {
    /// Prove the wire-observable subset using the existing nominal certificate.
    pub fn of(
        ir: &EssIr,
        projection: &ess_domain::accessor::ProjectionPlan,
        handles: &BTreeMap<ess_domain::QualifiedName, TypeHandle>,
        target: &ResolvedTypeRef,
    ) -> Result<Self, String> {
        let mut leaf = projection.0.leaf();
        while let TypeRef::Optional(of) = leaf {
            leaf = of;
        }
        let result = Self {
            projection: projection.clone(),
            target: unresolve(target),
            types: crate::accessor_types::TypeFacts::of(ir, &resolve(leaf, handles))?,
        };
        result.validate()?;
        Ok(result)
    }
    fn observation(&self) -> Observation {
        Observation {
            plan: self.projection.0.clone(),
            target: self.target.clone(),
            types: self.types.clone(),
        }
    }
    /// Validate the item-root graph and Optional observability independently.
    pub fn validate(&self) -> Result<(), String> {
        self.projection
            .validate()
            .map_err(|error| error.to_string())?;
        self.observation().assignment_graph().map(|_| ())
    }
    /// Evaluate the original selected item without reconstructing hidden native Optional states.
    pub fn evaluate(&self, item: &Node) -> Result<Expected, String> {
        self.validate()?;
        let observation = self.observation();
        let (depth, added) = observation.assignment_graph()?;
        observation.evaluate_assignment(Some(item), depth, added)
    }
}
