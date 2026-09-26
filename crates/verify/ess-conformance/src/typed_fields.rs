//! Shared finite type authority for response observations and pre-execution fixture values.
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedBody};
use ess_domain::{Field, QualifiedName, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn declarations<'a>(
    ir: &EssIr,
    fields: impl IntoIterator<Item = &'a Field>,
) -> Result<BTreeMap<QualifiedName, Declaration>, String> {
    let mut declarations = BTreeMap::new();
    let mut pending: Vec<_> = fields
        .into_iter()
        .flat_map(|f| f.type_ref.named_dependencies().into_iter().cloned())
        .collect();
    while let Some(name) = pending.pop() {
        if declarations.contains_key(&name) {
            continue;
        }
        if declarations.len() >= 4096 {
            return Err("response declaration resource limit".into());
        }
        let ty = ir.types().get(&name).ok_or("response type is absent")?;
        if ty.reading.is_some() || ty.body.is_constrained() {
            return Err(
                "response constrained type needs an executable invariant/reading observer".into(),
            );
        }
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
                variants: variants
                    .iter()
                    .map(|variant| variant.name().to_owned())
                    .collect(),
            },
            ResolvedBody::Union { tag, variants } => Declaration::Union {
                tag: tag.clone(),
                variants: variants
                    .iter()
                    .map(|(tag, ty)| (tag.clone(), crate::accessor::unresolve(ty)))
                    .collect(),
            },
        };
        for ty in body.references() {
            pending.extend(ty.named_dependencies().into_iter().cloned());
        }
        declarations.insert(name, body);
    }
    Ok(declarations)
}

pub(crate) fn validate<'a>(
    groups: impl IntoIterator<Item = &'a [Field]>,
    declarations: &BTreeMap<QualifiedName, Declaration>,
) -> Result<(), String> {
    let mut registry = ess_domain::TypeRegistry::new();
    for (name, body) in declarations {
        let declared = ess_domain::NamedType::try_from(ess_domain::types::RawNamedType {
            name: name.clone(),
            body: body.body(),
            naming: ess_domain::Naming::default(),
            reading: None,
        })
        .map_err(|e| e.to_string())?;
        registry.insert(declared).map_err(|e| e.to_string())?;
    }
    let mut used = BTreeSet::new();
    for fields in groups {
        let mut seen = BTreeSet::new();
        for field in fields {
            if field.name.is_empty() || !seen.insert(&field.name) {
                return Err("duplicate response contract field".into());
            }
            registry
                .resolve(&field.type_ref, "response")
                .into_result(())
                .map_err(|e| e.to_string())?;
            check_type(
                declarations,
                &field.type_ref,
                &mut used,
                &mut BTreeSet::new(),
                0,
            )?;
        }
    }
    if used.len() != declarations.len() {
        return Err("response contract carries unrelated type declarations".into());
    }
    Ok(())
}

fn check_type(
    declarations: &BTreeMap<QualifiedName, Declaration>,
    ty: &TypeRef,
    used: &mut BTreeSet<QualifiedName>,
    stack: &mut BTreeSet<QualifiedName>,
    depth: usize,
) -> Result<(), String> {
    if depth > 128 {
        return Err("response type depth limit".into());
    }
    match ty {
        TypeRef::Named(name) => {
            if !stack.insert(name.clone()) {
                return Err("recursive response type cannot be finitely admitted".into());
            }
            if used.contains(name) {
                stack.remove(name);
                return Ok(());
            }
            for child in declarations
                .get(name)
                .ok_or("missing response type")?
                .references()
            {
                check_type(declarations, child, used, stack, depth + 1)?;
            }
            used.insert(name.clone());
            stack.remove(name);
        }
        TypeRef::Optional(of) | TypeRef::List(of) => {
            check_type(declarations, of, used, stack, depth + 1)?;
        }
        TypeRef::Map(key, value) => {
            if *key != ess_domain::Primitive::String {
                return Err("response map keys require String".into());
            }
            check_type(declarations, value, used, stack, depth + 1)?;
        }
        TypeRef::Primitive(ess_domain::Primitive::Binary64) => {
            return Err("response Binary64 observation is not admitted".into())
        }
        TypeRef::Primitive(_) => {}
    }
    Ok(())
}
