//! Closed structural authority for response-derived nested event members.
use super::Observation;
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedPayloadField, ResolvedPayloadValue};
use ess_domain::{Field, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
struct Member(String);
impl TryFrom<String> for Member {
    type Error = String;
    fn try_from(value: String) -> Result<Self, String> {
        ess_domain::types::is_field_name(&value)
            .then_some(Self(value))
            .ok_or_else(|| "invalid nested response field name".into())
    }
}
impl From<Member> for String {
    fn from(value: Member) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "Vec<Member>", into = "Vec<Member>")]
struct TargetPath(Vec<Member>);
impl TryFrom<Vec<Member>> for TargetPath {
    type Error = String;
    fn try_from(value: Vec<Member>) -> Result<Self, String> {
        (2..=33)
            .contains(&value.len())
            .then_some(Self(value))
            .ok_or_else(|| "nested response path requires 2 through 33 segments".into())
    }
}
impl From<TargetPath> for Vec<Member> {
    fn from(value: TargetPath) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructuralField {
    name: Member,
    #[serde(rename = "type")]
    type_ref: TypeRef,
}
impl StructuralField {
    fn of(field: &Field) -> Result<Self, String> {
        Ok(Self {
            name: Member::try_from(field.name.clone())?,
            type_ref: field.type_ref.clone(),
        })
    }
    fn field(&self) -> Field {
        Field::new(&self.name.0, self.type_ref.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum StructuralDeclaration {
    Newtype {
        of: TypeRef,
    },
    Struct {
        fields: Vec<StructuralField>,
    },
    Enum {
        variants: Vec<String>,
    },
    Union {
        tag: String,
        variants: BTreeMap<String, TypeRef>,
    },
}
impl StructuralDeclaration {
    fn of(body: &Declaration) -> Result<Self, String> {
        Ok(match body {
            Declaration::Newtype { of } => Self::Newtype { of: of.clone() },
            Declaration::Struct { fields } => Self::Struct {
                fields: fields
                    .iter()
                    .map(StructuralField::of)
                    .collect::<Result<_, _>>()?,
            },
            Declaration::Enum { variants } => Self::Enum {
                variants: variants.clone(),
            },
            Declaration::Union { tag, variants } => Self::Union {
                tag: tag.clone(),
                variants: variants.clone(),
            },
        })
    }
    fn declaration(&self) -> Declaration {
        match self {
            Self::Newtype { of } => Declaration::Newtype { of: of.clone() },
            Self::Struct { fields } => Declaration::Struct {
                fields: fields.iter().map(StructuralField::field).collect(),
            },
            Self::Enum { variants } => Declaration::Enum {
                variants: variants.clone(),
            },
            Self::Union { tag, variants } => Declaration::Union {
                tag: tag.clone(),
                variants: variants.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mapping {
    target: TargetPath,
    source: Member,
}

/// Complete representation of the event roots used by checked nested relationships.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NestedTargets {
    roots: Vec<StructuralField>,
    declarations: BTreeMap<QualifiedName, StructuralDeclaration>,
    mappings: Vec<Mapping>,
}

impl NestedTargets {
    pub(super) fn of(ir: &EssIr, fields: &[ResolvedPayloadField]) -> Result<Option<Self>, String> {
        let mut roots = Vec::new();
        let mut mappings = Vec::new();
        for field in fields {
            if !matches!(field.value, ResolvedPayloadValue::Struct { .. }) {
                continue;
            }
            let before = mappings.len();
            collect(field, &mut Vec::new(), false, &mut mappings)?;
            if mappings.len() != before {
                roots.push(StructuralField {
                    name: Member::try_from(field.target.clone())?,
                    type_ref: crate::accessor::unresolve(&field.target_type),
                });
            }
        }
        if mappings.is_empty() {
            return Ok(None);
        }
        mappings.sort_by(|a, b| a.target.cmp(&b.target));
        let mut declarations = BTreeMap::new();
        let mut pending: Vec<_> = roots
            .iter()
            .flat_map(|f| f.type_ref.named_dependencies().into_iter().cloned())
            .collect();
        while let Some(name) = pending.pop() {
            if declarations.contains_key(&name) {
                continue;
            }
            if declarations.len() >= 4096 {
                return Err("nested response declaration limit".into());
            }
            let ty = ir
                .types()
                .get(&name)
                .ok_or("missing nested response type")?;
            let body = match &ty.body {
                ResolvedBody::Newtype { of, .. } => Declaration::Newtype {
                    of: crate::accessor::unresolve(of),
                },
                ResolvedBody::Struct { fields, .. } => Declaration::Struct {
                    fields: fields
                        .iter()
                        .map(|f| Field::new(&f.name, crate::accessor::unresolve(&f.type_ref)))
                        .collect(),
                },
                ResolvedBody::Enum { variants } => Declaration::Enum {
                    variants: variants.iter().map(|v| v.name().to_owned()).collect(),
                },
                ResolvedBody::Union { tag, variants } => Declaration::Union {
                    tag: tag.clone(),
                    variants: variants
                        .iter()
                        .map(|(k, v)| (k.clone(), crate::accessor::unresolve(v)))
                        .collect(),
                },
            };
            for reference in body.references() {
                pending.extend(reference.named_dependencies().into_iter().cloned());
            }
            declarations.insert(name, StructuralDeclaration::of(&body)?);
        }
        Ok(Some(Self {
            roots,
            declarations,
            mappings,
        }))
    }

    pub(super) fn len(&self) -> usize {
        self.mappings.len()
    }

    pub(super) fn validate(&self, observation: &Observation) -> Result<(), String> {
        if self.roots.is_empty()
            || self.roots.len() > 256
            || self.mappings.is_empty()
            || self.mappings.len() + observation.mappings.len() > 256
        {
            return Err("nested response relationship bound".into());
        }
        let names: BTreeSet<_> = self
            .declarations
            .keys()
            .chain(observation.declarations.keys())
            .collect();
        if names.len() > 4096 {
            return Err("nested response declaration union limit".into());
        }
        for (name, declaration) in &self.declarations {
            if let Some(other) = observation.declarations.get(name) {
                if *declaration != StructuralDeclaration::of(other)? {
                    return Err("conflicting nested response declaration".into());
                }
            }
        }
        value_metadata(observation)?;
        self.validate_schema()?;
        let mut roots = BTreeMap::new();
        for root in &self.roots {
            if roots.insert(&root.name, &root.type_ref).is_some()
                || observation.mappings.contains_key(&root.name.0)
            {
                return Err("duplicate or overlapping nested response root".into());
            }
        }
        let mut paths = BTreeSet::new();
        let mut used = BTreeSet::new();
        let mut resolved = BTreeMap::new();
        for mapping in &self.mappings {
            let path = &mapping.target.0;
            if !paths.insert(path) {
                return Err("duplicate nested response path".into());
            }
            let root = path.first().ok_or("empty nested response path")?;
            used.insert(root);
            let mut terminal = (*roots.get(root).ok_or("undeclared nested response root")?).clone();
            for member in &path[1..] {
                let name = self.struct_name(&terminal, &mut resolved)?;
                let Some(StructuralDeclaration::Struct { fields }) = self.declarations.get(&name)
                else {
                    return Err("nested response ancestor is not a struct".into());
                };
                terminal = fields
                    .iter()
                    .find(|f| f.name == *member)
                    .ok_or("undeclared nested response member")?
                    .type_ref
                    .clone();
            }
            let source = observation
                .fields
                .iter()
                .find(|f| f.name == mapping.source.0)
                .ok_or("undeclared nested response source")?;
            if !ess_domain::types::is_assignable(&source.type_ref, &terminal) {
                return Err("nested response terminal type mismatch".into());
            }
        }
        let ordered: Vec<_> = paths.into_iter().collect();
        if ordered.windows(2).any(|p| p[1].starts_with(p[0])) {
            return Err("overlapping nested response paths".into());
        }
        if used.len() != roots.len() {
            return Err("unused nested response root".into());
        }
        Ok(())
    }

    fn validate_schema(&self) -> Result<(), String> {
        let mut registry = ess_domain::TypeRegistry::new();
        for (name, body) in &self.declarations {
            if let StructuralDeclaration::Struct { fields } = body {
                if fields
                    .iter()
                    .map(|f| &f.name)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != fields.len()
                {
                    return Err("duplicate structural member".into());
                }
            }
            let declared = ess_domain::NamedType::try_from(ess_domain::types::RawNamedType {
                name: name.clone(),
                body: body.declaration().body(),
                naming: ess_domain::Naming::default(),
                reading: None,
            })
            .map_err(|e| e.to_string())?;
            registry.insert(declared).map_err(|e| e.to_string())?;
        }
        let mut pending: Vec<_> = self.roots.iter().map(|f| f.type_ref.clone()).collect();
        let mut used = BTreeSet::new();
        while let Some(ty) = pending.pop() {
            registry
                .resolve(&ty, "nested response")
                .into_result(())
                .map_err(|e| e.to_string())?;
            for name in ty.named_dependencies() {
                if used.insert(name.clone()) {
                    let body = self
                        .declarations
                        .get(name)
                        .ok_or("missing nested response type")?
                        .declaration();
                    pending.extend(body.references().into_iter().cloned());
                }
            }
        }
        if used.len() != self.declarations.len() {
            return Err("unrelated nested response declarations".into());
        }
        Ok(())
    }

    fn struct_name(
        &self,
        ty: &TypeRef,
        memo: &mut BTreeMap<TypeRef, QualifiedName>,
    ) -> Result<QualifiedName, String> {
        let mut current = ty.clone();
        let mut visited = BTreeSet::new();
        loop {
            if let Some(name) = memo.get(&current).cloned() {
                for walked in visited {
                    memo.insert(walked, name.clone());
                }
                return Ok(name);
            }
            if !visited.insert(current.clone()) {
                return Err("cyclic nested response ancestor".into());
            }
            match &current {
                TypeRef::Optional(of) => current = *of.clone(),
                TypeRef::Named(name) => match self.declarations.get(name) {
                    Some(StructuralDeclaration::Newtype { of }) => current = of.clone(),
                    Some(StructuralDeclaration::Struct { .. }) => {
                        for walked in visited {
                            memo.insert(walked, name.clone());
                        }
                        return Ok(name.clone());
                    }
                    _ => return Err("nested response ancestor is not a struct".into()),
                },
                _ => return Err("nested response ancestor is not a struct".into()),
            }
        }
    }

    pub(super) fn compare(
        &self,
        observation: &Observation,
        response: &BTreeMap<String, Node>,
        payload: &BTreeMap<String, Node>,
    ) -> Result<(), String> {
        for mapping in &self.mappings {
            let mut object = payload;
            let path = &mapping.target.0;
            for member in &path[..path.len() - 1] {
                let Some(Node::Map(next)) = object.get(&member.0) else {
                    return Err(format!(
                        "nested response ancestor {} is absent or not an object",
                        member.0
                    ));
                };
                object = next;
            }
            let emitted = object.get(&path[path.len() - 1].0);
            let actual = response.get(&mapping.source.0);
            let optional = observation
                .fields
                .iter()
                .any(|f| f.name == mapping.source.0 && matches!(f.type_ref, TypeRef::Optional(_)));
            if optional
                && actual.is_none_or(|v| *v == Node::Null)
                && emitted.is_none_or(|v| *v == Node::Null)
            {
                continue;
            }
            if actual.is_none() || actual != emitted {
                return Err(format!(
                    "event path {} differs from actual response field {}",
                    path.iter()
                        .map(|m| m.0.as_str())
                        .collect::<Vec<_>>()
                        .join("."),
                    mapping.source.0
                ));
            }
        }
        Ok(())
    }
}

fn collect(
    field: &ResolvedPayloadField,
    prefix: &mut Vec<Member>,
    converted: bool,
    mappings: &mut Vec<Mapping>,
) -> Result<(), String> {
    prefix.push(Member::try_from(field.target.clone())?);
    let converted = converted || field.conversion.is_some();
    match &field.value {
        ResolvedPayloadValue::Struct { fields } => {
            for child in fields {
                collect(child, prefix, converted, mappings)?;
            }
        }
        ResolvedPayloadValue::ResponseField { field: source, .. } => {
            if converted {
                return Err("response conversion requires an executable adapter".into());
            }
            mappings.push(Mapping {
                target: TargetPath::try_from(prefix.clone())?,
                source: Member::try_from(source.clone())?,
            });
        }
        _ => {}
    }
    prefix.pop();
    Ok(())
}

fn value_metadata(observation: &Observation) -> Result<(), String> {
    for field in &observation.fields {
        Member::try_from(field.name.clone())?;
        let mut naming = field.naming.clone();
        naming.presence = None;
        if !naming.is_empty() {
            return Err("nested response fields permit only name/type/presence".into());
        }
    }
    let members = observation
        .declarations
        .values()
        .filter_map(|d| match d {
            Declaration::Struct { fields } => Some(fields.as_slice()),
            _ => None,
        })
        .flatten();
    for field in observation.targets.iter().chain(members) {
        Member::try_from(field.name.clone())?;
        if !field.naming.is_empty() {
            return Err("nested response representation fields permit only name/type".into());
        }
    }
    Ok(())
}

/// Response leaves use their own complete value assertion; siblings keep ordinary shape checks.
pub(super) fn mapped_paths(fields: &[ResolvedPayloadField]) -> Vec<Vec<String>> {
    fn walk(
        fields: &[ResolvedPayloadField],
        prefix: &mut Vec<String>,
        paths: &mut Vec<Vec<String>>,
    ) {
        for field in fields {
            prefix.push(field.target.clone());
            match &field.value {
                ResolvedPayloadValue::ResponseField { .. } => paths.push(prefix.clone()),
                ResolvedPayloadValue::Struct { fields } => walk(fields, prefix, paths),
                _ => {}
            }
            prefix.pop();
        }
    }
    let mut paths = Vec::new();
    walk(fields, &mut Vec::new(), &mut paths);
    paths
}
