//! Declared occurrence selection observed from an event, with independently admitted type facts.
use crate::accessor::{unresolve, Expected, ProjectionObservation};
use ess_compiler::ir::{EssIr, ResolvedBinding, ResolvedBody, ResolvedTypeRef};
use ess_domain::{
    accessor::ProjectionPlan,
    selection::{First, InputSource, Selection, SelectionInput, SelectionOperation, SelectionPlan},
    EventSpec, Field, NamedType, QualifiedName, TypeRef, TypeRegistry,
};
use ess_primitives::{
    facts::{FactPath, FactStore, FactValue},
    node::Node,
    predicate::Truth,
};
use std::collections::{BTreeMap, BTreeSet};

/// Closed declaration facts needed to validate input shape and rederive selector plans.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Declaration {
    /// Transparent nominal representation.
    Newtype {
        /// Exact representation.
        of: TypeRef,
    },
    /// Required and Optional members, retaining authored names and order.
    Struct {
        /// Declared members.
        fields: Vec<Field>,
    },
    /// Closed admitted vocabulary.
    Enum {
        /// Source labels.
        variants: Vec<String>,
    },
    /// Explicit tagged alternatives.
    Union {
        /// Discriminator member.
        tag: String,
        /// Exact alternatives.
        variants: BTreeMap<String, TypeRef>,
    },
}
impl Declaration {
    pub(crate) fn body(&self) -> ess_domain::types::RawTypeBody {
        match self {
            Self::Newtype { of } => ess_domain::types::RawTypeBody::Newtype {
                of: of.clone(),
                invariants: Vec::new(),
            },
            Self::Struct { fields } => ess_domain::types::RawTypeBody::Struct {
                fields: fields.clone(),
                invariants: Vec::new(),
            },
            Self::Enum { variants } => ess_domain::types::RawTypeBody::Enum {
                variants: variants.clone(),
            },
            Self::Union { tag, variants } => ess_domain::types::RawTypeBody::Union {
                tag: tag.clone(),
                variants: variants.clone(),
            },
        }
    }
    pub(crate) fn references(&self) -> Vec<&TypeRef> {
        match self {
            Self::Newtype { of } => vec![of],
            Self::Struct { fields } => fields.iter().map(|f| &f.type_ref).collect(),
            Self::Enum { .. } => Vec::new(),
            Self::Union { variants, .. } => variants.values().collect(),
        }
    }
}

/// An exact selection expectation with finite declaration authority.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Observation {
    /// The event's declared identity; roots below remain its real members.
    pub event_type: QualifiedName,
    /// Shared ordered source and selector plan.
    pub plan: SelectionPlan,
    /// Exact reachable nominal declarations, no expression environment supplied by the target.
    pub declarations: BTreeMap<QualifiedName, Declaration>,
    /// Selected occurrence index in the selector table.
    pub selector: usize,
    /// Target member projection with existing Optional-observation proof.
    pub projection: ProjectionObservation,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObservation {
    event_type: QualifiedName,
    plan: SelectionPlan,
    declarations: BTreeMap<QualifiedName, Declaration>,
    selector: usize,
    projection: ProjectionObservation,
}
impl<'de> serde::Deserialize<'de> for Observation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = <RawObservation as serde::Deserialize>::deserialize(d)?;
        let value = Self {
            event_type: raw.event_type,
            plan: raw.plan,
            declarations: raw.declarations,
            selector: raw.selector,
            projection: raw.projection,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl Observation {
    /// Derive executable direct-list selection, explicitly refusing an opaque input conversion.
    pub fn of(
        ir: &EssIr,
        binding: &ResolvedBinding,
        selector: usize,
        projection: &ProjectionPlan,
        target: &ResolvedTypeRef,
    ) -> Result<Self, String> {
        let selection = binding
            .selection
            .as_ref()
            .ok_or("selection plan is absent")?;
        if selection
            .plan
            .inputs
            .iter()
            .any(|input| input.conversion.is_some())
        {
            return Err("selection-input-conversion: exact host preparation must be independently implemented before selection can be observed".into());
        }
        let mut pending = Vec::new();
        for input in &selection.plan.inputs {
            pending.extend(
                input
                    .source
                    .type_ref()
                    .named_dependencies()
                    .into_iter()
                    .cloned(),
            );
            pending.extend(input.list_type.named_dependencies().into_iter().cloned());
        }
        pending.extend(unresolve(target).named_dependencies().into_iter().cloned());
        let mut declarations = BTreeMap::new();
        while let Some(name) = pending.pop() {
            if declarations.contains_key(&name) {
                continue;
            }
            if declarations.len() >= 4096 {
                return Err("selection declaration resource limit".into());
            }
            let ty = ir
                .types()
                .get(&name)
                .ok_or("missing selection nominal declaration")?;
            if ty.reading.is_some()
                || matches!(&ty.body, ResolvedBody::Newtype { invariants, .. } | ResolvedBody::Struct { invariants, .. } if !invariants.is_empty())
            {
                return Err(format!("selection-constraint: invariant or clock-reading validation is not executable observation authority for {name}"));
            }
            let declaration = match &ty.body {
                ResolvedBody::Newtype { of, .. } => Declaration::Newtype { of: unresolve(of) },
                ResolvedBody::Struct { fields, .. } => Declaration::Struct {
                    fields: fields
                        .iter()
                        .map(|field| Field {
                            name: field.name.clone(),
                            type_ref: unresolve(&field.type_ref),
                            naming: field.naming.clone(),
                        })
                        .collect(),
                },
                ResolvedBody::Enum { variants } => Declaration::Enum {
                    variants: variants.clone(),
                },
                ResolvedBody::Union { tag, variants } => Declaration::Union {
                    tag: tag.clone(),
                    variants: variants
                        .iter()
                        .map(|(name, ty)| (name.clone(), unresolve(ty)))
                        .collect(),
                },
            };
            for reference in declaration.references() {
                pending.extend(reference.named_dependencies().into_iter().cloned());
            }
            declarations.insert(name, declaration);
        }
        let value = Self {
            event_type: binding
                .cause
                .event()
                .ok_or("selection observation requires an event cause")?
                .name()
                .clone(),
            plan: selection.plan.clone(),
            declarations,
            selector,
            projection: ProjectionObservation::of(ir, projection, &selection.types, target)?,
        };
        value.validate()?;
        Ok(value)
    }

    /// Independently rederive all input shapes, predicate reads and backward references.
    pub fn validate(&self) -> Result<(), String> {
        self.bytes()?;
        if self.plan.inputs.len() > 4
            || self.plan.selectors.len() > 8
            || self.declarations.len() > 4096
            || self.selector >= self.plan.selectors.len()
        {
            return Err("selection table bounds or selected index invalid".into());
        }
        let mut registry = TypeRegistry::new();
        for (name, body) in &self.declarations {
            let declared = NamedType::try_from(ess_domain::types::RawNamedType {
                name: name.clone(),
                body: body.body(),
                reading: None,
                naming: ess_domain::Naming::default(),
            })
            .map_err(|e| e.to_string())?;
            registry.insert(declared).map_err(|e| e.to_string())?;
        }
        let mut roots = BTreeMap::new();
        let mut inputs = Vec::new();
        for input in &self.plan.inputs {
            if input.conversion.is_some() {
                return Err(
                    "selection-input-conversion is not executable observation authority".into(),
                );
            }
            let (root, from) = match &input.source {
                InputSource::Field { field } => (field, format!("event.{}", field.name)),
                InputSource::Accessor { plan } => {
                    plan.validate().map_err(|e| e.to_string())?;
                    (&plan.root, plan.path())
                }
            };
            if let Some(existing) = roots.insert(root.name.clone(), root.clone()) {
                if existing != *root {
                    return Err("selection source roots disagree".into());
                }
            }
            inputs.push(SelectionInput {
                name: input.name.clone(),
                from,
                as_type: input.list_type.clone(),
            });
        }
        let event = EventSpec {
            name: self.event_type.clone(),
            fields: roots.into_values().collect(),
            naming: ess_domain::Naming::default(),
        };
        event.validate(&registry).map_err(|e| e.to_string())?;
        let declarations = self.authored_selectors()?;
        let rebuilt = SelectionPlan::resolve(
            &inputs,
            &declarations,
            &event,
            &registry,
            &ess_domain::types::ConversionRegistry::default(),
            "selection observation",
        )
        .map_err(|e| e.to_string())?;
        if rebuilt != self.plan {
            return Err("selection retained typed plan disagrees with declarations".into());
        }
        let (_, projection, effective) = rebuilt
            .projection(
                &rebuilt.selectors[self.selector].name,
                &self.projection.projection.0.segments[1..],
                &registry,
                "selection observation",
            )
            .map_err(|e| e.to_string())?;
        if projection != self.projection.projection
            || !ess_domain::types::is_assignable(&effective, &self.projection.target)
            || !matches!(self.projection.target, TypeRef::Optional(_))
        {
            return Err("selection projection identity or Optional target disagrees".into());
        }
        self.projection.validate()?;
        self.validate_reachable(&event)?;
        self.bytes()?;
        Ok(())
    }

    fn validate_reachable(&self, event: &EventSpec) -> Result<(), String> {
        let mut reachable = BTreeSet::new();
        let mut pending = event
            .fields
            .iter()
            .flat_map(|field| field.type_ref.named_dependencies().into_iter().cloned())
            .collect::<Vec<_>>();
        pending.extend(
            self.projection
                .target
                .named_dependencies()
                .into_iter()
                .cloned(),
        );
        while let Some(name) = pending.pop() {
            if reachable.insert(name.clone()) {
                let body = self
                    .declarations
                    .get(&name)
                    .ok_or("missing selection nominal fact")?;
                for reference in body.references() {
                    pending.extend(reference.named_dependencies().into_iter().cloned());
                }
            }
        }
        if reachable.len() != self.declarations.len() {
            return Err("selection contains unreachable declaration facts".into());
        }
        Ok(())
    }

    fn authored_selectors(&self) -> Result<Vec<Selection>, String> {
        let mut declarations = Vec::new();
        for (index, selector) in self.plan.selectors.iter().enumerate() {
            let input = self
                .plan
                .inputs
                .get(selector.input)
                .ok_or("selection input index invalid")?;
            let names = |indices: &[usize]| -> Result<Vec<String>, String> {
                if indices.len() > 4 || indices.iter().any(|prior| *prior >= index) {
                    return Err("selection references must point earlier within bound".into());
                }
                Ok(indices
                    .iter()
                    .map(|prior| self.plan.selectors[*prior].name.clone())
                    .collect())
            };
            declarations.push(match &selector.operation {
                SelectionOperation::First {
                    excluding,
                    predicate,
                    reads,
                } => {
                    if reads.len() > 64 {
                        return Err("selection read bound".into());
                    }
                    Selection {
                        name: selector.name.clone(),
                        first: Some(First {
                            input: input.name.clone(),
                            excluding: names(excluding)?,
                            predicate: predicate.clone(),
                        }),
                        first_present: None,
                    }
                }
                SelectionOperation::FirstPresent { selections } => Selection {
                    name: selector.name.clone(),
                    first: None,
                    first_present: Some(names(selections)?),
                },
            });
        }
        Ok(declarations)
    }

    /// Bounded serialization account shared by suite admission.
    pub fn bytes(&self) -> Result<usize, String> {
        struct Count(usize);
        impl std::io::Write for Count {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0 = self.0.saturating_add(bytes.len());
                if self.0 > 1_048_576 {
                    Err(std::io::Error::other("selection observation byte limit"))
                } else {
                    Ok(bytes.len())
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut count = Count(0);
        serde_json::to_writer(&mut count, self).map_err(|e| e.to_string())?;
        Ok(count.0)
    }

    /// Evaluate every input and required read before selecting any occurrence.
    fn observed_inputs<'a>(
        &self,
        payload: &'a BTreeMap<String, Node>,
    ) -> Result<Vec<&'a Vec<Node>>, String> {
        let mut inputs = Vec::new();
        for (input, plan) in self.plan.inputs.iter().enumerate() {
            let value = match &plan.source {
                InputSource::Field { field } => payload
                    .get(&field.name)
                    .ok_or_else(|| failure("invalid_input", input, None, None))?,
                InputSource::Accessor { plan } => project_value(plan, payload.get(&plan.root.name))
                    .map_err(|_| failure("invalid_input", input, None, None))?
                    .ok_or_else(|| failure("invalid_input", input, None, None))?,
            };
            let Node::Seq(values) = value else {
                return Err(failure("invalid_input", input, None, None));
            };
            if values.len() > 64 {
                return Err(failure("resource", input, None, None));
            }
            inputs.push(values);
        }
        Ok(inputs)
    }

    /// Observe bounded declared inputs before selecting and projecting an occurrence.
    pub fn evaluate(&self, payload: &BTreeMap<String, Node>) -> Result<Expected, String> {
        self.validate()?;
        let inputs = self.observed_inputs(payload)?;
        let mut bytes = 0usize;
        let mut truth = vec![Vec::new(); self.plan.selectors.len()];
        for (input, values) in inputs.iter().enumerate() {
            for (index, value) in values.iter().enumerate() {
                if matches!(value, Node::Null) && self.plan.inputs[input].optional_items {
                    for (selector, plan) in self
                        .plan
                        .selectors
                        .iter()
                        .enumerate()
                        .filter(|(_, plan)| plan.input == input)
                    {
                        if matches!(plan.operation, SelectionOperation::First { .. }) {
                            truth[selector].push(false);
                        }
                    }
                    continue;
                }
                validate_value(
                    &self.plan.inputs[input].item_type,
                    Some(value),
                    &self.declarations,
                    &mut bytes,
                    0,
                )
                .map_err(|cause| failure(cause, input, Some(index), None))?;
                if bytes > 1024 * 1024 {
                    return Err(failure("resource", input, Some(index), None));
                }
                for (selector, plan) in self
                    .plan
                    .selectors
                    .iter()
                    .enumerate()
                    .filter(|(_, plan)| plan.input == input)
                {
                    let SelectionOperation::First {
                        predicate, reads, ..
                    } = &plan.operation
                    else {
                        continue;
                    };
                    let mut facts = FactStore::new();
                    for (path, projection) in reads {
                        let projected =
                            project_value(&projection.0, Some(value)).map_err(|_| {
                                failure("invalid_input", input, Some(index), Some(selector))
                            })?;
                        if let Some(Node::Text(text)) = projected {
                            bytes = bytes.saturating_add(path.len()).saturating_add(text.len());
                            if text.len() > 4096 || bytes > 1_048_576 {
                                return Err(failure(
                                    "resource",
                                    input,
                                    Some(index),
                                    Some(selector),
                                ));
                            }
                            facts.set(
                                FactPath::new(path).map_err(|e| e.to_string())?,
                                FactValue::Text(text.clone()),
                            );
                        }
                    }
                    truth[selector].push(match predicate.evaluate(&facts) {
                        Truth::True => true,
                        Truth::False => false,
                        Truth::Unknown => {
                            return Err(failure("unknown", input, Some(index), Some(selector)))
                        }
                    });
                }
            }
        }
        let mut selected = vec![None; self.plan.selectors.len()];
        for (index, selector) in self.plan.selectors.iter().enumerate() {
            selected[index] = match &selector.operation {
                SelectionOperation::First { excluding, .. } => truth[index]
                    .iter()
                    .enumerate()
                    .find_map(|(occurrence, matches)| {
                        (*matches
                            && excluding
                                .iter()
                                .all(|prior| selected[*prior] != Some(occurrence)))
                        .then_some(occurrence)
                    }),
                SelectionOperation::FirstPresent { selections } => {
                    selections.iter().find_map(|prior| selected[*prior])
                }
            };
        }
        match selected[self.selector] {
            None => Ok(Expected::Absent),
            Some(index) => self
                .projection
                .evaluate(&inputs[self.plan.selectors[self.selector].input][index]),
        }
    }
}

fn failure(cause: &str, input: usize, index: Option<usize>, selector: Option<usize>) -> String {
    format!("SelectionFailure:{cause}:input={input}:index={index:?}:selector={selector:?}")
}

fn validate_value(
    ty: &TypeRef,
    value: Option<&Node>,
    declarations: &BTreeMap<QualifiedName, Declaration>,
    bytes: &mut usize,
    depth: usize,
) -> Result<(), &'static str> {
    validate_value_inner(ty, value, declarations, bytes, depth, false)
}
pub(crate) fn validate_response_value(
    ty: &TypeRef,
    value: Option<&Node>,
    declarations: &BTreeMap<QualifiedName, Declaration>,
    bytes: &mut usize,
) -> Result<(), &'static str> {
    validate_value_inner(ty, value, declarations, bytes, 0, true)
}
fn validate_value_inner(
    ty: &TypeRef,
    value: Option<&Node>,
    declarations: &BTreeMap<QualifiedName, Declaration>,
    bytes: &mut usize,
    depth: usize,
    response: bool,
) -> Result<(), &'static str> {
    if depth > 128 || *bytes > 1_048_576 {
        return Err("resource");
    }
    if let TypeRef::Optional(of) = ty {
        return if value.is_none() || matches!(value, Some(Node::Null)) {
            Ok(())
        } else {
            validate_value_inner(of, value, declarations, bytes, depth + 1, response)
        };
    }
    let value = value.ok_or("invalid_input")?;
    match ty {
        TypeRef::Named(name) => match declarations.get(name).ok_or("invalid_input")? {
            Declaration::Newtype { of } => {
                validate_value_inner(of, Some(value), declarations, bytes, depth + 1, response)
            }
            Declaration::Enum { variants } => match value {
                Node::Text(text) if variants.contains(text) => {
                    *bytes = bytes.saturating_add(text.len());
                    Ok(())
                }
                _ => Err("invalid_input"),
            },
            Declaration::Struct { fields } => {
                let Node::Map(values) = value else {
                    return Err("invalid_input");
                };
                if response
                    && values
                        .keys()
                        .any(|key| !fields.iter().any(|f| &f.name == key))
                {
                    return Err("invalid_input");
                }
                for field in fields {
                    *bytes = bytes.saturating_add(field.name.len());
                    validate_value_inner(
                        &field.type_ref,
                        values.get(&field.name),
                        declarations,
                        bytes,
                        depth + 1,
                        response,
                    )?;
                }
                Ok(())
            }
            Declaration::Union { tag, variants } => {
                let Node::Map(values) = value else {
                    return Err("invalid_input");
                };
                let Some(Node::Text(label)) = values.get(tag) else {
                    return Err("invalid_input");
                };
                let content = ess_gen::schema::union_content_key(tag);
                if response && values.keys().any(|key| key != tag && key != content) {
                    return Err("invalid_input");
                }
                let ty = variants.get(label).ok_or("invalid_input")?;
                validate_value_inner(
                    ty,
                    values.get(content),
                    declarations,
                    bytes,
                    depth + 1,
                    response,
                )
            }
        },
        TypeRef::Primitive(kind) => validate_primitive(*kind, value, bytes),
        TypeRef::List(of) => {
            let Node::Seq(values) = value else {
                return Err("invalid_input");
            };
            if values.len() > 64 {
                return Err("resource");
            }
            for value in values {
                validate_value_inner(of, Some(value), declarations, bytes, depth + 1, response)?;
            }
            Ok(())
        }
        TypeRef::Map(key, of) if response && *key == ess_domain::Primitive::String => {
            let Node::Map(values) = value else {
                return Err("invalid_input");
            };
            if values.len() > 64 {
                return Err("resource");
            }
            for (name, child) in values {
                *bytes = bytes.saturating_add(name.len());
                validate_value_inner(of, Some(child), declarations, bytes, depth + 1, response)?;
            }
            Ok(())
        }
        TypeRef::Map(_, _) => Err("unsupported"),
        TypeRef::Optional(_) => unreachable!(),
    }
}

fn project_value<'a>(
    plan: &ess_domain::accessor::AccessorPlan,
    value: Option<&'a Node>,
) -> Result<Option<&'a Node>, String> {
    match crate::accessor::project(plan, value)? {
        crate::accessor::Projected::Unavailable => Ok(None),
        crate::accessor::Projected::Terminal(value) => {
            Ok(value.filter(|value| !matches!(value, Node::Null)))
        }
    }
}

/// Whether a suite carries the closed observed-selection vocabulary.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| scenario.steps.iter().any(|step| matches!(step, crate::ScenarioStep::ExpectInvocation { input, .. } if input.values().any(|value| matches!(value, crate::ScenarioValue::ObservedSelection { .. })))))
}

fn validate_primitive(
    kind: ess_domain::Primitive,
    value: &Node,
    bytes: &mut usize,
) -> Result<(), &'static str> {
    if !(crate::scenario::Holds::Primitive { kind }).admits(value) {
        return Err("invalid_input");
    }
    if let Node::Text(text) = value {
        *bytes = bytes.saturating_add(text.len());
        if text.len() > 4096 {
            return Err("resource");
        }
    }
    Ok(())
}
