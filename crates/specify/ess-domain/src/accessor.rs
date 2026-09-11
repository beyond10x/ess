//! Bounded, shared plans for declared event projections. No runtime expression language.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use ess_primitives::error::{ValidationCode, ValidationError};

use crate::{EventSpec, Field, QualifiedName, TypeBody, TypeRef, TypeRegistry};

/// Maximum author-written fields after `event`.
pub const MAX_SEGMENTS: usize = 3;
/// Maximum actual typed operations along a plan path.
pub const MAX_OPERATIONS: usize = 128;
/// Maximum shared nodes in one accessor.
pub const MAX_NODES: usize = 4_096;
/// Maximum child references in one accessor.
pub const MAX_EDGES: usize = 16_384;
/// Maximum canonical plan bytes in one accessor.
pub const MAX_BYTES: usize = 1_048_576;

/// One shared operation. Child references index the owning plan's node table.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    /// Copy the entire terminal value, including its Optional wrappers.
    Leaf,
    /// This union alternative does not declare the selected member.
    Missing,
    /// Select a declared member of a struct.
    Field {
        /// The actual member, including its wire spelling and declared type.
        field: Field,
        /// Next shared node.
        next: usize,
    },
    /// Traverse a nominal wrapper without changing the terminal's identity.
    Newtype {
        /// Next shared node.
        next: usize,
    },
    /// Stop with unavailable when this container is absent.
    Optional {
        /// Next shared node.
        next: usize,
    },
    /// Select the actual variant, never a guessed branch.
    Union {
        /// Declared discriminator field.
        tag: String,
        /// Variant labels and their shared plans, in deterministic order.
        variants: BTreeMap<String, usize>,
    },
}

impl Operation {
    /// Child references in their declared deterministic order.
    pub fn children(&self) -> Vec<usize> {
        match self {
            Self::Leaf | Self::Missing => Vec::new(),
            Self::Field { next, .. } | Self::Newtype { next } | Self::Optional { next } => {
                vec![*next]
            }
            Self::Union { variants, .. } => variants.values().copied().collect(),
        }
    }

    fn remap(&mut self, ids: &[usize]) {
        match self {
            Self::Leaf | Self::Missing => {}
            Self::Field { next, .. } | Self::Newtype { next } | Self::Optional { next } => {
                *next = ids[*next];
            }
            Self::Union { variants, .. } => {
                for next in variants.values_mut() {
                    *next = ids[*next];
                }
            }
        }
    }
}

/// The observable outer shape of a typed value, including transparent nominal wrappers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValueShape {
    /// A primitive's established wire representation.
    Primitive {
        /// Actual primitive.
        name: crate::Primitive,
    },
    /// A struct, tagged union or map.
    Object,
    /// A list.
    Sequence,
    /// An admitted enum label.
    Enum {
        /// The declared variants, in source order.
        variants: Vec<String>,
    },
    /// Outer presence is optional; non-null values use the inner shape.
    Optional {
        /// Shape when present.
        of: Box<ValueShape>,
    },
}

fn value_shape(
    source: &TypeRef,
    types: &TypeRegistry,
    at: &str,
) -> Result<ValueShape, ValidationError> {
    let mut source = source;
    let mut seen = BTreeSet::new();
    let mut optional = 0;
    let shape = loop {
        if seen.len() + optional > MAX_OPERATIONS {
            return Err(resource(
                at,
                "shape operations",
                seen.len() + optional,
                MAX_OPERATIONS,
            ));
        }
        match source {
            TypeRef::Primitive(name) => break ValueShape::Primitive { name: *name },
            TypeRef::Optional(of) => {
                optional += 1;
                source = of;
            }
            TypeRef::List(_) => break ValueShape::Sequence,
            TypeRef::Map(_, _) => break ValueShape::Object,
            TypeRef::Named(name) => {
                if !seen.insert(name) {
                    return Err(error(
                        ValidationCode::AccessorTraversal,
                        at,
                        "recursive transparent value shape",
                    ));
                }
                let declared = types.get(name).ok_or_else(|| {
                    error(
                        ValidationCode::UndeclaredReference,
                        at,
                        "undeclared accessor value type",
                    )
                })?;
                match &declared.body {
                    TypeBody::Newtype { of, .. } => source = of,
                    TypeBody::Enum { variants } => {
                        let mut bytes = Bytes::default();
                        serde_json::to_writer(&mut bytes, variants)
                            .map_err(|_| resource(at, "enum shape bytes", bytes.0, MAX_BYTES))?;
                        break ValueShape::Enum {
                            variants: variants.clone(),
                        };
                    }
                    TypeBody::Struct { .. } | TypeBody::Union { .. } => break ValueShape::Object,
                }
            }
        }
    };
    Ok(if optional == 0 {
        shape
    } else {
        ValueShape::Optional {
            of: Box::new(shape),
        }
    })
}

/// A typed node in an accessor DAG.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    /// Source type on entering this operation.
    pub source: TypeRef,
    /// Declared wire kind checked even when the selected union member is unavailable.
    pub shape: ValueShape,
    /// Index of the next author-written segment to consume.
    pub position: usize,
    /// One typed operation.
    pub operation: Operation,
    /// Terminal nominal type, or none for a missing union branch.
    pub leaf: Option<TypeRef>,
    /// Whether traversal can stop before reaching that terminal.
    pub may_miss: bool,
    /// Maximum remaining operations, including this node.
    pub depth: usize,
}

/// One resolved accessor. A completed branch is serialized once even in a union diamond.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AccessorPlan {
    /// Source spelling, excluding `event`.
    pub segments: Vec<String>,
    /// The event member that starts the walk.
    pub root: Field,
    /// Index of the first typed operation.
    pub start: usize,
    /// Shared nodes, sorted by source type and segment position.
    pub nodes: Vec<Node>,
}

/// An input plan still requiring bounded graph admission.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAccessorPlan {
    segments: Vec<String>,
    root: Field,
    start: usize,
    #[serde(deserialize_with = "bounded_nodes")]
    nodes: Vec<Node>,
}

fn bounded_nodes<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<Node>, D::Error> {
    struct Nodes;
    impl<'de> serde::de::Visitor<'de> for Nodes {
        type Value = Vec<Node>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("at most 4096 shared accessor nodes")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut nodes = Vec::new();
            while nodes.len() < MAX_NODES {
                let Some(node) = seq.next_element()? else {
                    return Ok(nodes);
                };
                nodes.push(node);
            }
            if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom(
                    "AccessorResource: nodes exceed 4096",
                ));
            }
            Ok(nodes)
        }
    }
    deserializer.deserialize_seq(Nodes)
}

fn error(code: ValidationCode, at: &str, message: impl Into<String>) -> ValidationError {
    ValidationError::new(code, at, message)
}

fn resource(at: &str, account: &str, attempted: usize, maximum: usize) -> ValidationError {
    error(
        ValidationCode::AccessorResource,
        at,
        format!("accessor {account}: attempted {attempted}, limit {maximum}"),
    )
}

/// Check the finite source spelling without reading model declarations.
pub fn validate_segments(segments: &[String], at: &str) -> Result<(), ValidationError> {
    if segments.len() > MAX_SEGMENTS {
        return Err(error(
            ValidationCode::AccessorDepth,
            at,
            format!(
                "accessor has {} segments; maximum is {MAX_SEGMENTS}",
                segments.len()
            ),
        ));
    }
    if segments.len() < 2
        || segments
            .iter()
            .any(|s| crate::types::field_name(s).is_err())
    {
        return Err(error(
            ValidationCode::UnobservableFact,
            at,
            "accessor requires two or three declared field names without empty segments",
        ));
    }
    Ok(())
}

#[derive(Default)]
struct Bytes(usize);
impl Write for Bytes {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 = self.0.saturating_add(buf.len());
        if self.0 > MAX_BYTES {
            return Err(io::Error::other("accessor canonical byte limit"));
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AccessorPlan {
    /// The exact terminal type, preserving nominal and Optional identity.
    pub fn leaf(&self) -> &TypeRef {
        self.nodes[self.start]
            .leaf
            .as_ref()
            .expect("admitted accessor reaches a leaf")
    }

    /// Whether an Optional container or union branch can make this projection unavailable.
    pub fn may_miss(&self) -> bool {
        self.nodes[self.start].may_miss
    }

    /// Type used for assignability and exact conversion lookup.
    pub fn effective_type(&self) -> TypeRef {
        let leaf = self.leaf().clone();
        if self.may_miss() && !matches!(leaf, TypeRef::Optional(_)) {
            TypeRef::Optional(Box::new(leaf))
        } else {
            leaf
        }
    }

    /// Source path, exactly as a mapping displays it.
    pub fn path(&self) -> String {
        format!("event.{}", self.segments.join("."))
    }

    /// All nominal types traversed, for compiler handles and provenance.
    pub fn dependencies(&self) -> BTreeSet<QualifiedName> {
        self.nodes
            .iter()
            .flat_map(|node| node.source.named_dependencies().into_iter().cloned())
            .collect()
    }

    /// Exact bounded compact serialization size, counted without building an output string.
    pub fn bytes(&self) -> Result<usize, ValidationError> {
        let mut count = Bytes::default();
        serde_json::to_writer(&mut count, self)
            .map_err(|_| resource(&self.path(), "bytes", count.0, MAX_BYTES))?;
        Ok(count.0)
    }

    /// Admit a plan received over a suite boundary before any lookup or execution.
    pub fn admit(raw: RawAccessorPlan) -> Result<Self, ValidationError> {
        let plan = Self {
            segments: raw.segments,
            root: raw.root,
            start: raw.start,
            nodes: raw.nodes,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Recheck graph shape and derived metadata; public fields are not admission evidence.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let at = self.path();
        validate_segments(&self.segments, &at)?;
        self.validate_graph()
    }

    fn validate_graph(&self) -> Result<(), ValidationError> {
        let at = self.path();
        if self.nodes.len() > MAX_NODES {
            return Err(resource(&at, "nodes", self.nodes.len(), MAX_NODES));
        }
        if self.segments.first() != Some(&self.root.name) || self.start >= self.nodes.len() {
            return Err(error(
                ValidationCode::UnobservableFact,
                &at,
                "invalid accessor root",
            ));
        }
        if self.nodes[self.start].source != self.root.type_ref
            || self.nodes[self.start].position != 1
        {
            return Err(error(
                ValidationCode::TypeMismatch,
                &at,
                "accessor root type disagrees",
            ));
        }
        let edges: usize = self
            .nodes
            .iter()
            .map(|n| n.operation.children().len())
            .sum();
        if edges > MAX_EDGES {
            return Err(resource(&at, "edges", edges, MAX_EDGES));
        }
        let mut marks = vec![0_u8; self.nodes.len()];
        let mut tasks = vec![(self.start, false)];
        while let Some((id, finishing)) = tasks.pop() {
            let Some(node) = self.nodes.get(id) else {
                return Err(error(
                    ValidationCode::UnobservableFact,
                    &at,
                    "dangling accessor node",
                ));
            };
            if finishing {
                let (leaf, missing, depth) =
                    summarize(&node.source, &node.operation, &self.nodes, &at)?;
                if node.leaf.as_ref() != leaf || node.may_miss != missing || node.depth != depth {
                    return Err(error(
                        ValidationCode::TypeMismatch,
                        &at,
                        "accessor node metadata disagrees",
                    ));
                }
                marks[id] = 2;
                continue;
            }
            if marks[id] == 2 {
                continue;
            }
            if marks[id] == 1 {
                return Err(error(
                    ValidationCode::AccessorTraversal,
                    &at,
                    "cyclic accessor graph",
                ));
            }
            marks[id] = 1;
            check_edges(node, &self.nodes, &self.segments, &at)?;
            check_shape(node, &self.nodes, &at)?;
            tasks.push((id, true));
            for child in node.operation.children().into_iter().rev() {
                tasks.push((child, false));
            }
        }
        if marks.contains(&0) || self.nodes[self.start].leaf.is_none() {
            return Err(error(
                ValidationCode::UnobservableFact,
                &at,
                "accessor has unreachable nodes or no declared terminal field",
            ));
        }
        let keys: Vec<_> = self.nodes.iter().map(|n| (&n.source, n.position)).collect();
        if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(error(
                ValidationCode::AccessorTraversal,
                &at,
                "accessor nodes must be unique and sorted",
            ));
        }
        self.bytes()?;
        Ok(())
    }
}

fn check_shape(node: &Node, nodes: &[Node], at: &str) -> Result<(), ValidationError> {
    let invalid = || {
        error(
            ValidationCode::TypeMismatch,
            at,
            "accessor wire shape disagrees with typed operation",
        )
    };
    let mut ty = &node.source;
    let shape = if let ValueShape::Optional { of } = &node.shape {
        of.as_ref()
    } else {
        &node.shape
    };
    if matches!(ty, TypeRef::Optional(_)) && !matches!(node.shape, ValueShape::Optional { .. }) {
        return Err(invalid());
    }
    while let TypeRef::Optional(of) = ty {
        ty = of;
    }
    if !matches!(node.source, TypeRef::Optional(_) | TypeRef::Named(_))
        && matches!(node.shape, ValueShape::Optional { .. })
    {
        return Err(invalid());
    }
    let consistent = match ty {
        TypeRef::Primitive(name) => shape == &ValueShape::Primitive { name: *name },
        TypeRef::List(_) => shape == &ValueShape::Sequence,
        TypeRef::Map(_, _) => shape == &ValueShape::Object,
        TypeRef::Named(_) => true,
        TypeRef::Optional(_) => unreachable!("stripped outer Optional"),
    };
    if !consistent {
        return Err(invalid());
    }
    match &node.operation {
        Operation::Field { .. } | Operation::Union { .. } if node.shape != ValueShape::Object => {
            Err(invalid())
        }
        Operation::Newtype { next } if node.shape != nodes[*next].shape => Err(invalid()),
        Operation::Optional { next } => {
            let next = match &nodes[*next].shape {
                ValueShape::Optional { of } => of.as_ref(),
                shape => shape,
            };
            if shape != next {
                return Err(invalid());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn check_edges(
    node: &Node,
    nodes: &[Node],
    segments: &[String],
    at: &str,
) -> Result<(), ValidationError> {
    let invalid = || {
        error(
            ValidationCode::TypeMismatch,
            at,
            "accessor operation disagrees with source type or segment",
        )
    };
    let child = |id: usize| nodes.get(id).ok_or_else(invalid);
    match &node.operation {
        Operation::Leaf if node.position == segments.len() => {}
        Operation::Missing if node.position < segments.len() => {}
        Operation::Field { field, next } => {
            let next = child(*next)?;
            if !matches!(node.source, TypeRef::Named(_))
                || segments.get(node.position) != Some(&field.name)
                || next.source != field.type_ref
                || next.position != node.position + 1
            {
                return Err(invalid());
            }
        }
        Operation::Optional { next } => {
            let TypeRef::Optional(of) = &node.source else {
                return Err(invalid());
            };
            let next = child(*next)?;
            if node.position >= segments.len()
                || next.source != **of
                || next.position != node.position
            {
                return Err(invalid());
            }
        }
        Operation::Newtype { next } => {
            if !matches!(node.source, TypeRef::Named(_))
                || child(*next)?.position != node.position
                || node.position >= segments.len()
            {
                return Err(invalid());
            }
        }
        Operation::Union { tag, variants } => {
            if !matches!(node.source, TypeRef::Named(_))
                || tag.is_empty()
                || variants.is_empty()
                || node.position >= segments.len()
            {
                return Err(invalid());
            }
            for next in variants.values() {
                if child(*next)?.position != node.position {
                    return Err(invalid());
                }
            }
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

fn summarize<'a>(
    source: &'a TypeRef,
    operation: &Operation,
    nodes: &'a [Node],
    at: &str,
) -> Result<(Option<&'a TypeRef>, bool, usize), ValidationError> {
    let children = operation.children();
    let depth = children
        .iter()
        .map(|id| nodes[*id].depth)
        .max()
        .unwrap_or(0)
        + 1;
    if depth > MAX_OPERATIONS {
        return Err(resource(at, "operations", depth, MAX_OPERATIONS));
    }
    if matches!(operation, Operation::Leaf) {
        return Ok((Some(source), false, depth));
    }
    let mut leaf = None;
    let mut missing = matches!(
        operation,
        Operation::Missing | Operation::Optional { .. } | Operation::Union { .. }
    );
    for id in children {
        let child = &nodes[id];
        missing |= child.may_miss;
        if let Some(found) = &child.leaf {
            if leaf.is_some_and(|current| current != found) {
                return Err(error(
                    ValidationCode::TypeMismatch,
                    at,
                    match operation {
                        Operation::Union { variants, .. } => format!(
                            "union accessor alternatives have different terminal types: {}",
                            variants
                                .iter()
                                .filter_map(|(name, id)| nodes[*id]
                                    .leaf
                                    .as_ref()
                                    .map(|leaf| format!("{name}: {leaf}")))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        _ => "accessor alternatives have different terminal types".into(),
                    },
                ));
            }
            leaf = Some(found);
        }
    }
    Ok((leaf, missing, depth))
}

struct Builder<'a> {
    types: &'a TypeRegistry,
    segments: &'a [String],
    at: &'a str,
    ids: BTreeMap<TypeRef, BTreeMap<usize, usize>>,
    nodes: Vec<Node>,
    marks: Vec<u8>,
    edges: usize,
    bytes: usize,
}

impl Builder<'_> {
    fn charge(
        &mut self,
        value: &impl serde::Serialize,
        copies: usize,
    ) -> Result<(), ValidationError> {
        let mut bytes = Bytes::default();
        serde_json::to_writer(&mut bytes, value)
            .map_err(|_| resource(self.at, "construction bytes", bytes.0, MAX_BYTES))?;
        self.bytes = self.bytes.saturating_add(bytes.0.saturating_mul(copies));
        if self.bytes > MAX_BYTES {
            return Err(resource(
                self.at,
                "construction bytes",
                self.bytes,
                MAX_BYTES,
            ));
        }
        Ok(())
    }

    fn intern(&mut self, source: &TypeRef, position: usize) -> Result<usize, ValidationError> {
        if let Some(id) = self
            .ids
            .get(source)
            .and_then(|positions| positions.get(&position))
        {
            return Ok(*id);
        }
        if self.nodes.len() == MAX_NODES {
            return Err(resource(self.at, "nodes", MAX_NODES + 1, MAX_NODES));
        }
        self.charge(source, 3)?;
        self.bytes = self.bytes.saturating_add(256);
        self.charge(&position, 1)?;
        let id = self.nodes.len();
        let shape = value_shape(source, self.types, self.at)?;
        self.charge(&shape, 1)?;
        self.nodes.push(Node {
            source: source.clone(),
            shape,
            position,
            operation: Operation::Missing,
            leaf: None,
            may_miss: false,
            depth: 0,
        });
        self.marks.push(0);
        self.ids
            .entry(source.clone())
            .or_default()
            .insert(position, id);
        Ok(id)
    }

    fn next(&mut self, source: &TypeRef, position: usize) -> Result<usize, ValidationError> {
        self.edges += 1;
        if self.edges > MAX_EDGES {
            return Err(resource(self.at, "edges", self.edges, MAX_EDGES));
        }
        self.intern(source, position)
    }

    fn operation(
        &mut self,
        source: &TypeRef,
        position: usize,
    ) -> Result<Operation, ValidationError> {
        if position == self.segments.len() {
            return Ok(Operation::Leaf);
        }
        match source {
            TypeRef::Optional(of) => Ok(Operation::Optional {
                next: self.next(of, position)?,
            }),
            TypeRef::List(_) | TypeRef::Map(_, _) => Err(error(
                ValidationCode::AccessorTraversal,
                self.at,
                format!(
                    "accessor cannot traverse {source} at {}",
                    self.segments[position]
                ),
            )),
            TypeRef::Primitive(_) => Ok(Operation::Missing),
            TypeRef::Named(name) => {
                let declared = self.types.get(name).ok_or_else(|| {
                    error(
                        ValidationCode::UndeclaredReference,
                        self.at,
                        format!("accessor type {name} is not declared"),
                    )
                })?;
                match &declared.body {
                    TypeBody::Newtype { of, .. } => Ok(Operation::Newtype {
                        next: self.next(of, position)?,
                    }),
                    TypeBody::Struct { fields, .. } => {
                        let Some(field) = fields.iter().find(|f| f.name == self.segments[position])
                        else {
                            return Ok(Operation::Missing);
                        };
                        self.charge(field, 1)?;
                        let field = field.clone();
                        Ok(Operation::Field {
                            next: self.next(&field.type_ref, position + 1)?,
                            field,
                        })
                    }
                    TypeBody::Enum { .. } => Ok(Operation::Missing),
                    TypeBody::Union { tag, variants } => {
                        self.charge(tag, 1)?;
                        let tag = tag.clone();
                        let mut branches = BTreeMap::new();
                        for (label, ty) in variants {
                            self.charge(label, 1)?;
                            self.bytes = self.bytes.saturating_add(32);
                            branches.insert(label.clone(), self.next(ty, position)?);
                        }
                        Ok(Operation::Union {
                            tag,
                            variants: branches,
                        })
                    }
                }
            }
        }
    }

    fn build(&mut self, start: usize) -> Result<(), ValidationError> {
        let mut pending = vec![(start, false)];
        while let Some((id, finishing)) = pending.pop() {
            if finishing {
                let node = &self.nodes[id];
                let (leaf, may_miss, depth) =
                    summarize(&node.source, &node.operation, &self.nodes, self.at)?;
                let mut bytes = Bytes::default();
                serde_json::to_writer(&mut bytes, &leaf)
                    .map_err(|_| resource(self.at, "leaf metadata bytes", bytes.0, MAX_BYTES))?;
                self.bytes = self.bytes.saturating_add(bytes.0);
                if self.bytes > MAX_BYTES {
                    return Err(resource(
                        self.at,
                        "construction bytes",
                        self.bytes,
                        MAX_BYTES,
                    ));
                }
                self.nodes[id].leaf = leaf.cloned();
                self.nodes[id].may_miss = may_miss;
                self.nodes[id].depth = depth;
                self.marks[id] = 2;
                continue;
            }
            if self.marks[id] == 2 {
                continue;
            }
            if self.marks[id] == 1 {
                return Err(error(
                    ValidationCode::AccessorTraversal,
                    self.at,
                    "recursive transparent accessor traversal",
                ));
            }
            self.marks[id] = 1;
            let source = self.nodes[id].source.clone();
            let op = self.operation(&source, self.nodes[id].position)?;
            let children = op.children();
            self.nodes[id].operation = op;
            pending.push((id, true));
            for child in children.into_iter().rev() {
                pending.push((child, false));
            }
        }
        Ok(())
    }
}

/// Resolve every declared segment once, retaining shared nodes across union alternatives.
pub fn resolve(
    event: &EventSpec,
    segments: &[String],
    types: &TypeRegistry,
    at: &str,
) -> Result<AccessorPlan, ValidationError> {
    validate_segments(segments, at)?;
    let root = event.field(&segments[0]).ok_or_else(|| {
        error(
            ValidationCode::UnobservableFact,
            at,
            format!(
                "event {} has no field {}; available declared members: {}",
                event.name,
                segments[0],
                event
                    .fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
    })?;
    build_projection(root, segments, types, at)
}

fn build_projection(
    root: &Field,
    segments: &[String],
    types: &TypeRegistry,
    at: &str,
) -> Result<AccessorPlan, ValidationError> {
    let mut builder = Builder {
        types,
        segments,
        at,
        ids: BTreeMap::new(),
        nodes: Vec::new(),
        marks: Vec::new(),
        edges: 0,
        bytes: 0,
    };
    builder.charge(root, 1)?;
    builder.charge(&segments, 1)?;
    let root = root.clone();
    let start = builder.intern(&root.type_ref, 1)?;
    builder.build(start)?;
    if builder.nodes[start].leaf.is_none() {
        let missing = builder
            .nodes
            .iter()
            .filter(|node| matches!(node.operation, Operation::Missing))
            .min_by_key(|node| (node.position, &node.source))
            .expect("a complete path without a leaf has a missing operation");
        let available = if let TypeRef::Named(name) = &missing.source {
            match &types.get(name).expect("resolved source type").body {
                TypeBody::Struct { fields, .. } => fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => String::new(),
            }
        } else {
            String::new()
        };
        return Err(error(
            ValidationCode::UnobservableFact,
            at,
            format!(
                "{} has no declared member {}; available declared members: [{}]",
                missing.source, segments[missing.position], available
            ),
        ));
    }
    let mut remap = vec![0; builder.nodes.len()];
    for (new, old) in builder.ids.values().flat_map(BTreeMap::values).enumerate() {
        remap[*old] = new;
    }
    let mut nodes = Vec::with_capacity(builder.nodes.len());
    for old in builder.ids.values().flat_map(BTreeMap::values) {
        let mut node = builder.nodes[*old].clone();
        node.operation.remap(&remap);
        nodes.push(node);
    }
    let plan = AccessorPlan {
        segments: segments.to_vec(),
        root,
        start: remap[start],
        nodes,
    };
    plan.validate_graph()?;
    Ok(plan)
}

/// Model-wide account for new plans only; legacy specifications consume no capacity.
#[derive(Default)]
pub struct Account {
    nodes: usize,
    edges: usize,
    bytes: usize,
}

impl Account {
    /// Charge a complete plan before accepting another binding into the model.
    pub fn add(&mut self, plan: &AccessorPlan) -> Result<(), ValidationError> {
        self.nodes = self.nodes.saturating_add(plan.nodes.len());
        self.edges = self.edges.saturating_add(
            plan.nodes
                .iter()
                .map(|n| n.operation.children().len())
                .sum(),
        );
        self.bytes = self.bytes.saturating_add(plan.bytes()?);
        for (name, used, max) in [
            ("model nodes", self.nodes, 65_536),
            ("model edges", self.edges, 262_144),
            ("model bytes", self.bytes, 16_777_216),
        ] {
            if used > max {
                return Err(resource(&plan.path(), name, used, max));
            }
        }
        Ok(())
    }
}

impl<'de> serde::Deserialize<'de> for AccessorPlan {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = <RawAccessorPlan as serde::Deserialize>::deserialize(deserializer)?;
        Self::admit(raw).map_err(serde::de::Error::custom)
    }
}

/// A projection from an explicitly typed selector item. Its root is not an event field.
/// The existing DAG operations and graph admission are shared with event accessors.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct ProjectionPlan(pub AccessorPlan);

impl ProjectionPlan {
    /// Resolve zero to three declared members from the selected record.
    pub fn resolve(
        item: &TypeRef,
        path: &[String],
        types: &TypeRegistry,
        at: &str,
    ) -> Result<Self, ValidationError> {
        if path.len() > MAX_SEGMENTS
            || path
                .iter()
                .any(|part| crate::types::field_name(part).is_err())
        {
            return Err(error(
                ValidationCode::AccessorDepth,
                at,
                "selection projection requires zero to three declared field names",
            ));
        }
        let root = Field::new("item", item.clone());
        let segments: Vec<_> = std::iter::once("item".to_owned())
            .chain(path.iter().cloned())
            .collect();
        let plan = Self(build_projection(&root, &segments, types, at)?);
        plan.validate()?;
        Ok(plan)
    }

    /// Recheck the separate item-root bounds and all shared typed DAG invariants.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let plan = &self.0;
        if plan.root.name != "item"
            || plan.segments.is_empty()
            || plan.segments.len() > MAX_SEGMENTS + 1
            || plan
                .segments
                .iter()
                .any(|part| crate::types::field_name(part).is_err())
        {
            return Err(error(
                ValidationCode::AccessorDepth,
                "selection projection",
                "invalid item projection root or member bound",
            ));
        }
        plan.validate_graph()
    }
}

impl<'de> serde::Deserialize<'de> for ProjectionPlan {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = <RawAccessorPlan as serde::Deserialize>::deserialize(deserializer)?;
        let plan = Self(AccessorPlan {
            segments: raw.segments,
            root: raw.root,
            start: raw.start,
            nodes: raw.nodes,
        });
        plan.validate().map_err(serde::de::Error::custom)?;
        Ok(plan)
    }
}
