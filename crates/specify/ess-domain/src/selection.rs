//! Finite, typed first-occurrence selection local to one event binding.

use crate::{
    accessor::{AccessorPlan, ProjectionPlan},
    expression::{DomainEnvironment, ScalarKind},
    types::ConversionRegistry,
    EventSpec, Field, Primitive, TypeBody, TypeRef, TypeRegistry,
};
use ess_primitives::error::{ValidationCode, ValidationError};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};
use std::collections::{BTreeMap, BTreeSet};

/// Maximum local typed inputs.
pub const MAX_INPUTS: usize = 4;
/// Maximum ordered selectors.
pub const MAX_SELECTORS: usize = 8;
/// Maximum elements of each input, including absent elements.
pub const MAX_ITEMS: usize = 64;
/// Maximum earlier references per selector.
pub const MAX_REFERENCES: usize = 4;
/// Maximum AST nodes in one predicate.
pub const MAX_PREDICATE_NODES: usize = 64;
/// Maximum AST nodes across the binding.
pub const MAX_TOTAL_PREDICATE_NODES: usize = 256;
/// Maximum examined scalar bytes.
pub const MAX_TEXT_BYTES: usize = 4096;
/// Maximum examined value and path bytes per event occurrence.
pub const MAX_EXAMINED_BYTES: usize = 1_048_576;

/// An explicitly declared binding-local input; conversions use the existing exact registry.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SelectionInput {
    /// Binding-local identity.
    pub name: String,
    /// An event field or bounded accessor; never an invented producer field.
    pub from: String,
    /// Required list type after any explicitly declared preparation conversion.
    #[serde(rename = "as")]
    pub as_type: TypeRef,
}

/// One ordered selection declaration.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Unique local selector identity.
    pub name: String,
    /// Lowest eligible occurrence in the declared input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first: Option<First>,
    /// First present result among earlier selectors of the same input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_present: Option<Vec<String>>,
}

/// A bounded predicate and earlier occurrence exclusions.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct First {
    /// Local input identity.
    #[serde(rename = "in")]
    pub input: String,
    /// Earlier selected occurrences excluded by original index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluding: Vec<String>,
    /// Existing finite predicate syntax, checked against the item declaration.
    #[serde(rename = "where")]
    pub predicate: Predicate,
}

/// Exact event source retained independently of a selection result.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InputSource {
    /// A declared event member.
    Field {
        /// Actual member.
        field: Field,
    },
    /// Existing bounded event DAG.
    Accessor {
        /// Complete admitted source plan.
        plan: AccessorPlan,
    },
}
impl InputSource {
    /// Effective source type, including absence introduced by traversal.
    pub fn type_ref(&self) -> TypeRef {
        match self {
            Self::Field { field } => field.type_ref.clone(),
            Self::Accessor { plan } => plan.effective_type(),
        }
    }
}

/// A resolved local input; source preparation is explicit and evaluated once.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputPlan {
    /// Original local declaration identity.
    pub name: String,
    /// Declared source projection.
    pub source: InputSource,
    /// Required declared list type, retaining aliases.
    pub list_type: TypeRef,
    /// Present element type, retaining nominal identity.
    pub item_type: TypeRef,
    /// Whether null items are skipped.
    pub optional_items: bool,
    /// Explicit conversion reason; none means identity/ordinary assignability.
    pub conversion: Option<String>,
}

/// One admitted selector, with backward-only references.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectorPlan {
    /// Stable local identity.
    pub name: String,
    /// Index into the input table.
    pub input: usize,
    /// Finite selection operation.
    pub operation: SelectionOperation,
}

/// Closed executable selection vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SelectionOperation {
    /// Scan original input order, excluding only these earlier occurrences.
    First {
        /// Prior selector indices.
        excluding: Vec<usize>,
        /// Existing finite AST.
        predicate: Predicate,
        /// Declared projections for each fact path.
        reads: BTreeMap<String, ProjectionPlan>,
    },
    /// The first non-absent earlier selection, without copying identity.
    FirstPresent {
        /// Prior selector indices in authored order.
        selections: Vec<usize>,
    },
}

/// Ordered typed plan shared by native and conformance realizers.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct SelectionPlan {
    /// Inputs in declaration order.
    pub inputs: Vec<InputPlan>,
    /// Selectors in declaration order.
    pub selectors: Vec<SelectorPlan>,
}

fn err(at: &str, message: impl Into<String>) -> ValidationError {
    ValidationError::new(ValidationCode::UnsupportedConstruct, at, message)
}
fn bound(at: &str, name: &str, actual: usize, max: usize) -> Result<(), ValidationError> {
    if actual > max {
        return Err(ValidationError::new(
            ValidationCode::AccessorResource,
            at,
            format!("selection {name}: attempted {actual}, limit {max}"),
        ));
    }
    Ok(())
}
fn transparent<'a>(
    mut ty: &'a TypeRef,
    types: &'a TypeRegistry,
    at: &str,
) -> Result<&'a TypeRef, ValidationError> {
    let mut seen = BTreeSet::new();
    while let TypeRef::Named(name) = ty {
        if !seen.insert(name) || seen.len() > crate::accessor::MAX_OPERATIONS {
            return Err(err(at, "recursive or over-limit selection alias"));
        }
        match &types
            .get(name)
            .ok_or_else(|| err(at, format!("undeclared selection type {name}")))?
            .body
        {
            TypeBody::Newtype { of, .. } => ty = of,
            _ => break,
        }
    }
    Ok(ty)
}

fn predicate_count(predicate: &Predicate, at: &str) -> Result<usize, ValidationError> {
    let mut pending = vec![predicate];
    let mut count = 0;
    while let Some(node) = pending.pop() {
        count += 1;
        bound(at, "predicate nodes", count, MAX_PREDICATE_NODES)?;
        match node {

            Predicate::All(children) | Predicate::Any(children) => pending.extend(children),
            Predicate::Not(child) => pending.push(child),
            Predicate::Always | Predicate::Never | Predicate::Defined(_) | Predicate::Compare { left: Operand::Fact(_), op: CompareOp::Eq | CompareOp::Ne, right: Operand::Literal(ess_primitives::facts::FactValue::Text(_)) } => {}
            _ => return Err(err(at, "selection admits only Always/Never/Defined, String or enum Eq/Ne literals, All/Any/Not")),
        }
    }
    Ok(count)
}

impl SelectionPlan {
    /// Resolve declarations and all predicate reads, even for unused selectors.
    pub fn resolve(
        inputs: &[SelectionInput],
        selections: &[Selection],
        event: &EventSpec,
        types: &TypeRegistry,
        conversions: &ConversionRegistry,
        at: &str,
    ) -> Result<Self, ValidationError> {
        bound(at, "inputs", inputs.len(), MAX_INPUTS)?;
        bound(at, "selectors", selections.len(), MAX_SELECTORS)?;
        let mut plan = Self {
            inputs: resolve_inputs(inputs, event, types, conversions, at)?,
            ..Self::default()
        };
        let mut count = 0;
        for selection in selections {
            if crate::types::field_name(&selection.name).is_err()
                || plan.selectors.iter().any(|s| s.name == selection.name)
            {
                return Err(err(
                    at,
                    format!("invalid or duplicate selector {}", selection.name),
                ));
            }
            let (input, operation) = match (&selection.first, &selection.first_present) {
                (Some(first), None) => {
                    let input = plan
                        .inputs
                        .iter()
                        .position(|i| i.name == first.input)
                        .ok_or_else(|| {
                            err(at, format!("undeclared selection input {}", first.input))
                        })?;
                    let excluding = plan.references(&first.excluding, Some(input), at)?;
                    count += predicate_count(&first.predicate, at)?;
                    bound(
                        at,
                        "total predicate nodes",
                        count,
                        MAX_TOTAL_PREDICATE_NODES,
                    )?;
                    let reads =
                        resolve_reads(&plan.inputs[input].item_type, &first.predicate, types, at)?;
                    (
                        input,
                        SelectionOperation::First {
                            excluding,
                            predicate: first.predicate.clone(),
                            reads,
                        },
                    )
                }
                (None, Some(prior)) if !prior.is_empty() => {
                    let references = plan.references(prior, None, at)?;
                    let input = plan.selectors[references[0]].input;
                    if references
                        .iter()
                        .any(|index| plan.selectors[*index].input != input)
                    {
                        return Err(err(at, "first_present must reference the same list input"));
                    }
                    (
                        input,
                        SelectionOperation::FirstPresent {
                            selections: references,
                        },
                    )
                }
                _ => {
                    return Err(err(
                        at,
                        "selector requires exactly one nonempty first or first_present operation",
                    ))
                }
            };
            plan.selectors.push(SelectorPlan {
                name: selection.name.clone(),
                input,
                operation,
            });
        }
        plan.validate_structure()?;
        Ok(plan)
    }

    fn references(
        &self,
        names: &[String],
        input: Option<usize>,
        at: &str,
    ) -> Result<Vec<usize>, ValidationError> {
        bound(at, "earlier references", names.len(), MAX_REFERENCES)?;
        let mut seen = BTreeSet::new();
        names
            .iter()
            .map(|name| {
                if !seen.insert(name) {
                    return Err(err(at, "duplicate earlier selector reference"));
                }
                let index = self
                    .selectors
                    .iter()
                    .position(|s| &s.name == name)
                    .ok_or_else(|| err(at, format!("selector {name} must be declared earlier")))?;
                if input.is_some_and(|input| input != self.selectors[index].input) {
                    return Err(err(at, "excluded selector must use the same list input"));
                }
                Ok(index)
            })
            .collect()
    }

    /// Resolve one mapping and retain selection absence as Optional, without coercion.
    pub fn projection(
        &self,
        name: &str,
        path: &[String],
        types: &TypeRegistry,
        at: &str,
    ) -> Result<(usize, ProjectionPlan, TypeRef), ValidationError> {
        let index = self
            .selectors
            .iter()
            .position(|s| s.name == name)
            .ok_or_else(|| err(at, format!("undeclared selector {name}")))?;
        let input = &self.inputs[self.selectors[index].input];
        let projection = ProjectionPlan::resolve(&input.item_type, path, types, at)?;
        let ty = projection.0.effective_type();
        let ty = if matches!(ty, TypeRef::Optional(_)) {
            ty
        } else {
            TypeRef::Optional(Box::new(ty))
        };
        Ok((index, projection, ty))
    }
}

impl SelectionPlan {
    /// Validate all finite local references and typed projection roots without external lookup.
    /// Conformance admission additionally rederives these facts from retained declarations.
    pub fn validate_structure(&self) -> Result<(), ValidationError> {
        let at = "selection plan";
        bound(at, "inputs", self.inputs.len(), MAX_INPUTS)?;
        bound(at, "selectors", self.selectors.len(), MAX_SELECTORS)?;
        let mut names = BTreeSet::new();
        for input in &self.inputs {
            if crate::types::field_name(&input.name).is_err() || !names.insert(&input.name) {
                return Err(err(at, "invalid or duplicate local input name"));
            }
            if let InputSource::Accessor { plan } = &input.source {
                plan.validate()?;
            }
        }
        names.clear();
        let mut nodes = 0;
        for (index, selector) in self.selectors.iter().enumerate() {
            if selector.input >= self.inputs.len()
                || crate::types::field_name(&selector.name).is_err()
                || !names.insert(&selector.name)
            {
                return Err(err(at, "invalid selector identity or input"));
            }
            let prior = match &selector.operation {
                SelectionOperation::First {
                    excluding,
                    predicate,
                    reads,
                } => {
                    nodes += predicate_count(predicate, at)?;
                    bound(
                        at,
                        "total predicate nodes",
                        nodes,
                        MAX_TOTAL_PREDICATE_NODES,
                    )?;
                    let expected: BTreeSet<_> = predicate
                        .fact_paths()
                        .into_iter()
                        .map(ToString::to_string)
                        .collect();
                    if expected != reads.keys().cloned().collect() {
                        return Err(err(at, "selection predicate and read table disagree"));
                    }
                    for (path, projection) in reads {
                        projection.validate()?;
                        if path != &projection.0.segments.join(".")
                            || projection.0.root.type_ref != self.inputs[selector.input].item_type
                        {
                            return Err(err(
                                at,
                                "selection predicate projection identity disagrees",
                            ));
                        }
                    }
                    excluding
                }
                SelectionOperation::FirstPresent { selections } => {
                    if selections.is_empty() {
                        return Err(err(at, "first_present requires earlier alternatives"));
                    }
                    selections
                }
            };
            bound(at, "references", prior.len(), MAX_REFERENCES)?;
            let mut unique = BTreeSet::new();
            for previous in prior {
                if *previous >= index
                    || !unique.insert(previous)
                    || self.selectors[*previous].input != selector.input
                {
                    return Err(err(
                        at,
                        "selector references must be unique earlier occurrences in the same input",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSelectionPlan {
    #[serde(deserialize_with = "bounded_inputs")]
    inputs: Vec<InputPlan>,
    #[serde(deserialize_with = "bounded_selectors")]
    selectors: Vec<SelectorPlan>,
}
fn bounded_inputs<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<InputPlan>, D::Error> {
    bounded_vec(d, MAX_INPUTS)
}
fn bounded_selectors<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<SelectorPlan>, D::Error> {
    bounded_vec(d, MAX_SELECTORS)
}
fn bounded_vec<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    d: D,
    limit: usize,
) -> Result<Vec<T>, D::Error> {
    struct Items<T> {
        limit: usize,
        item: std::marker::PhantomData<T>,
    }
    impl<'de, T: serde::Deserialize<'de>> serde::de::Visitor<'de> for Items<T> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "at most {} selection entries", self.limit)
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut items = Vec::new();
            while items.len() < self.limit {
                let Some(item) = seq.next_element()? else {
                    return Ok(items);
                };
                items.push(item);
            }
            if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom("selection entry resource limit"));
            }
            Ok(items)
        }
    }
    d.deserialize_seq(Items {
        limit,
        item: std::marker::PhantomData,
    })
}
impl<'de> serde::Deserialize<'de> for SelectionPlan {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = <RawSelectionPlan as serde::Deserialize>::deserialize(d)?;
        let plan = Self {
            inputs: raw.inputs,
            selectors: raw.selectors,
        };
        plan.validate_structure()
            .map_err(serde::de::Error::custom)?;
        Ok(plan)
    }
}

fn resolve_inputs(
    inputs: &[SelectionInput],
    event: &EventSpec,
    types: &TypeRegistry,
    conversions: &ConversionRegistry,
    at: &str,
) -> Result<Vec<InputPlan>, ValidationError> {
    let mut resolved = Vec::new();
    let mut names = BTreeSet::new();
    for input in inputs {
        if crate::types::field_name(&input.name).is_err() || !names.insert(&input.name) {
            return Err(err(
                at,
                format!("invalid or duplicate selection input {}", input.name),
            ));
        }
        let source = match crate::binding::MappingSource::parse(&input.from) {
            crate::binding::MappingSource::EventField { field } => InputSource::Field {
                field: event
                    .field(&field)
                    .ok_or_else(|| err(at, format!("undeclared event field {field}")))?
                    .clone(),
            },
            crate::binding::MappingSource::EventAccessor { segments } => InputSource::Accessor {
                plan: crate::accessor::resolve(event, &segments, types, at)?,
            },
            _ => {
                return Err(err(
                    at,
                    "selection input source must be a declared event field or accessor",
                ))
            }
        };
        let TypeRef::List(element) = transparent(&input.as_type, types, at)? else {
            return Err(err(at, "selection input requires List<T> or List<Optional<T>>; Optional lists require an explicit preparation conversion"));
        };
        let original_element = element.as_ref();
        let element = transparent(original_element, types, at)?;
        let (item, optional_items) = if let TypeRef::Optional(of) = element {
            (of.as_ref(), true)
        } else {
            (original_element, false)
        };
        let present = transparent(item, types, at)?;
        let TypeRef::Named(record) = present else {
            return Err(err(at, "selection item must be a declared record"));
        };
        if !matches!(
            types.get(record).map(|ty| &ty.body),
            Some(TypeBody::Struct { .. } | TypeBody::Union { .. })
        ) {
            return Err(err(at, "selection item must be a declared record"));
        }
        let source_type = source.type_ref();
        let conversion = if crate::types::is_assignable(&source_type, &input.as_type) {
            None
        } else {
            Some(conversions.iter().find(|c| c.from == source_type && c.to == input.as_type).ok_or_else(|| err(at, format!("selection input {} requires an explicit exact conversion from {source_type} to {}", input.name, input.as_type)))?.because.clone())
        };
        resolved.push(InputPlan {
            name: input.name.clone(),
            source,
            list_type: input.as_type.clone(),
            item_type: item.clone(),
            optional_items,
            conversion,
        });
    }
    Ok(resolved)
}

fn resolve_reads(
    item_type: &TypeRef,
    predicate: &Predicate,
    types: &TypeRegistry,
    at: &str,
) -> Result<BTreeMap<String, ProjectionPlan>, ValidationError> {
    let fields = [Field::new("item", item_type.clone())];
    let environment = DomainEnvironment::new(types, &fields);
    let checked = crate::expression::check_predicate(&environment, predicate, at);
    if let Some(error) = checked.errors.first() {
        return Err(error.validation_error());
    }
    let mut reads = BTreeMap::new();
    for read in checked.reads {
        let parts = read.path.segments();
        if parts.first().map(String::as_str) != Some("item")
            || parts.len() < 2
            || parts.len() > 4
            || read.resolution.access.collection
            || read.resolution.scalar != Some(ScalarKind::Text)
        {
            return Err(err(
                at,
                format!(
                    "selection predicate requires a declared String/enum item leaf: {}",
                    read.path
                ),
            ));
        }
        let projection = ProjectionPlan::resolve(item_type, &parts[1..], types, at)?;
        let terminal = transparent(&read.resolution.terminal, types, at)?;
        if !matches!(
            terminal,
            TypeRef::Primitive(Primitive::String) | TypeRef::Named(_)
        ) {
            return Err(err(
                at,
                "selection predicates require String or enum leaves",
            ));
        }
        reads.insert(read.path.to_string(), projection);
    }
    Ok(reads)
}
