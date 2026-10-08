//! Shared finite type authority for response observations and pre-execution fixture values.
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedBody};
use ess_domain::{Field, QualifiedName, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn declarations<'a>(
    ir: &EssIr,
    fields: impl IntoIterator<Item = &'a Field>,
) -> Result<BTreeMap<QualifiedName, Declaration>, String> {
    declarations_with_presence(ir, fields, false, false)
}

/// [`declarations`] for a response-mapped event payload, admitting String-newtype constraints
/// (beyond10x/ess#499); the caller carries them beside the declarations.
pub(crate) fn response_payload_declarations<'a>(
    ir: &EssIr,
    fields: impl IntoIterator<Item = &'a Field>,
) -> Result<BTreeMap<QualifiedName, Declaration>, String> {
    declarations_with_presence(ir, fields, false, true)
}

/// The declarations of a direct return, admitting String-newtype constraints
/// (beyond10x/ess#499); the caller carries them beside the declarations.
pub(crate) fn direct_response_declarations<'a>(
    ir: &EssIr,
    fields: impl IntoIterator<Item = &'a Field>,
) -> Result<BTreeMap<QualifiedName, Declaration>, String> {
    declarations_with_presence(ir, fields, true, true)
}

pub(crate) fn one_time_declarations<'a>(
    ir: &EssIr,
    fields: impl IntoIterator<Item = &'a Field>,
) -> Result<BTreeMap<QualifiedName, Declaration>, String> {
    declarations_with_presence(ir, fields, true, true)
}

fn declarations_with_presence<'a>(
    ir: &EssIr,
    fields: impl IntoIterator<Item = &'a Field>,
    preserve_presence: bool,
    carry_string_constraints: bool,
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
        if ty.reading.is_some() {
            return Err(format!(
                "reading-attached response type `{name}` needs a reading observer, which a \
                 response observation does not have"
            ));
        }
        if ty.body.is_constrained() {
            match ty.body {
                ResolvedBody::Struct { .. } => {
                    return Err(format!(
                        "record invariant on a response type `{name}` has no executable observer"
                    ))
                }
                ResolvedBody::Newtype { .. } if carry_string_constraints => {}
                _ => {
                    return Err(format!(
                        "constrained type `{name}` has no executable observer here"
                    ))
                }
            }
        }
        let body = match &ty.body {
            ResolvedBody::Newtype { of, .. } => Declaration::Newtype {
                of: crate::accessor::unresolve(of),
            },
            ResolvedBody::Struct { fields, .. } => Declaration::Struct {
                fields: fields
                    .iter()
                    .map(|f| {
                        let mut field =
                            Field::new(&f.name, crate::accessor::unresolve(&f.type_ref));
                        if preserve_presence {
                            field.naming.presence = f.naming.presence;
                        }
                        field
                    })
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
                    .map(|(tag, ty)| (tag.clone(), ty.as_ref().map(crate::accessor::unresolve)))
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

/// Which side of a command a typed contract describes, which decides what recursion means.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Position {
    /// An observed response: every runner's observer walks the declared type without a
    /// termination rule, so a recursive response type stays refused.
    Response,
    /// A pre-execution fixture value (beyond10x/ess#416): a type that reaches itself only behind
    /// `Optional`, `List` or `Map` — the base cases the model's own inhabitation rule admits — has
    /// finite values, and the supplied value is checked structurally under the value depth guard.
    Input,
}

impl Position {
    fn noun(self) -> &'static str {
        match self {
            Self::Response => "response",
            Self::Input => "fixture input",
        }
    }
}

/// Admit a response contract's closed finite type graph.
pub(crate) fn validate<'a>(
    groups: impl IntoIterator<Item = &'a [Field]>,
    declarations: &BTreeMap<QualifiedName, Declaration>,
) -> Result<(), String> {
    validate_at(Position::Response, groups, declarations)
}

/// Admit a fixture contract's closed type graph, recursive where every cycle has a base case.
pub(crate) fn validate_input<'a>(
    groups: impl IntoIterator<Item = &'a [Field]>,
    declarations: &BTreeMap<QualifiedName, Declaration>,
) -> Result<(), String> {
    validate_at(Position::Input, groups, declarations)
}

fn validate_at<'a>(
    position: Position,
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
    let groups: Vec<&[Field]> = groups.into_iter().collect();
    let mut used = BTreeSet::new();
    for fields in &groups {
        let mut seen = BTreeSet::new();
        for field in *fields {
            if field.name.is_empty() || !seen.insert(&field.name) {
                return Err(match position {
                    Position::Response => "duplicate response contract field".into(),
                    Position::Input => "duplicate fixture input field".into(),
                });
            }
            registry
                .resolve(&field.type_ref, position.noun())
                .into_result(())
                .map_err(|e| e.to_string())?;
            let mut walk = Walk {
                position,
                stack: BTreeSet::new(),
                path: vec![field.name.clone()],
            };
            check_type(declarations, &field.type_ref, &mut used, &mut walk, 0)?;
        }
    }
    if used.len() != declarations.len() {
        return Err(match position {
            Position::Response => "response contract carries unrelated type declarations".into(),
            Position::Input => "fixture contract carries unrelated type declarations".into(),
        });
    }
    if position == Position::Input {
        let unfinite = registry.without_finite_value();
        if !unfinite.is_empty() {
            for field in groups.iter().flat_map(|fields| fields.iter()) {
                if let Some((path, name)) = unfinite_path(declarations, field, &unfinite) {
                    return Err(format!(
                        "fixture input `{}` has no finite value at `{path}`: `{name}` recurs with \
                         no Optional, List or Map boundary",
                        field.name
                    ));
                }
            }
            // `used` covers every declaration, so some field reaches each of them.
            return Err("fixture input has no finite value".into());
        }
    }
    Ok(())
}

/// The walk's position, the declarations it is inside, and the members it descended through.
struct Walk {
    position: Position,
    stack: BTreeSet<QualifiedName>,
    path: Vec<String>,
}

/// A declaration's references, each with the member that holds it: a struct field's name or a
/// union variant's tag. A newtype's representation is the value itself and adds no member.
fn members(declaration: &Declaration) -> Vec<(Option<&str>, &TypeRef)> {
    match declaration {
        Declaration::Newtype { of } => vec![(None, of)],
        Declaration::Struct { fields } => fields
            .iter()
            .map(|field| (Some(field.name.as_str()), &field.type_ref))
            .collect(),
        Declaration::Enum { .. } => Vec::new(),
        // A unit variant (ess/22) carries no payload to check.
        Declaration::Union { variants, .. } => variants
            .iter()
            .filter_map(|(tag, ty)| ty.as_ref().map(|ty| (Some(tag.as_str()), ty)))
            .collect(),
    }
}

fn check_type(
    declarations: &BTreeMap<QualifiedName, Declaration>,
    ty: &TypeRef,
    used: &mut BTreeSet<QualifiedName>,
    walk: &mut Walk,
    depth: usize,
) -> Result<(), String> {
    if depth > 128 {
        return Err(format!("{} type depth limit", walk.position.noun()));
    }
    match ty {
        TypeRef::Named(name) => {
            if walk.stack.contains(name) {
                return match walk.position {
                    Position::Response => Err(format!(
                        "recursive response type `{name}` cannot be finitely admitted at \
                         response field `{}`",
                        walk.path.join(".")
                    )),
                    // Whether this cycle has a base case is the inhabitation rule's to answer,
                    // once the whole graph is known.
                    Position::Input => Ok(()),
                };
            }
            if used.contains(name) {
                return Ok(());
            }
            walk.stack.insert(name.clone());
            let declaration = declarations
                .get(name)
                .ok_or_else(|| format!("missing {} type", walk.position.noun()))?;
            for (member, child) in members(declaration) {
                if let Some(member) = member {
                    walk.path.push(member.to_owned());
                }
                check_type(declarations, child, used, walk, depth + 1)?;
                if member.is_some() {
                    walk.path.pop();
                }
            }
            used.insert(name.clone());
            walk.stack.remove(name);
        }
        TypeRef::Optional(of) | TypeRef::List(of) => {
            check_type(declarations, of, used, walk, depth + 1)?;
        }
        TypeRef::Map(key, value) => {
            if *key != ess_domain::Primitive::String {
                return Err(format!("{} map keys require String", walk.position.noun()));
            }
            check_type(declarations, value, used, walk, depth + 1)?;
        }
        TypeRef::Primitive(ess_domain::Primitive::Binary64) => {
            return Err(match walk.position {
                Position::Response => "response Binary64 observation is not admitted".into(),
                Position::Input => "fixture input Binary64 value is not admitted".into(),
            })
        }
        TypeRef::Primitive(_) => {}
    }
    Ok(())
}

/// Where `field` first reaches a declaration with no finite value, followed through the members
/// that keep it so until the cycle closes: the path and the declaration it closes on.
///
/// `check_type` has already admitted the graph, so every name is declared and the first descent
/// is no deeper than its depth limit; the cycle is followed iteratively and visits each
/// declaration at most once.
fn unfinite_path(
    declarations: &BTreeMap<QualifiedName, Declaration>,
    field: &Field,
    unfinite: &BTreeSet<QualifiedName>,
) -> Option<(String, QualifiedName)> {
    fn reach(
        declarations: &BTreeMap<QualifiedName, Declaration>,
        ty: &TypeRef,
        unfinite: &BTreeSet<QualifiedName>,
        path: &mut Vec<String>,
        seen: &mut BTreeSet<QualifiedName>,
    ) -> Option<QualifiedName> {
        match ty {
            TypeRef::Named(name) if unfinite.contains(name) => Some(name.clone()),
            TypeRef::Named(name) => {
                if !seen.insert(name.clone()) {
                    return None;
                }
                for (member, child) in members(declarations.get(name)?) {
                    if let Some(member) = member {
                        path.push(member.to_owned());
                    }
                    if let Some(found) = reach(declarations, child, unfinite, path, seen) {
                        return Some(found);
                    }
                    if member.is_some() {
                        path.pop();
                    }
                }
                None
            }
            TypeRef::Optional(of) | TypeRef::List(of) | TypeRef::Map(_, of) => {
                reach(declarations, of, unfinite, path, seen)
            }
            TypeRef::Primitive(_) => None,
        }
    }
    let mut path = vec![field.name.clone()];
    let mut current = reach(
        declarations,
        &field.type_ref,
        unfinite,
        &mut path,
        &mut BTreeSet::new(),
    )?;
    let mut visited = BTreeSet::new();
    while visited.insert(current.clone()) {
        // A declaration with no finite value has a requirement with none either: a struct field,
        // a newtype's representation, or (every one of) a union's variants.
        let (member, next) =
            members(declarations.get(&current)?)
                .into_iter()
                .find_map(|(member, ty)| match ty {
                    TypeRef::Named(next) if unfinite.contains(next) => Some((member, next.clone())),
                    _ => None,
                })?;
        if let Some(member) = member {
            path.push(member.to_owned());
        }
        current = next;
    }
    Some((path.join("."), current))
}
