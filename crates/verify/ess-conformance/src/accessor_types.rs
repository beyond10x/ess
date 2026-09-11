//! Closed nominal facts needed to decide whether wire observations determine native presence.
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedTypeRef};
use ess_domain::{QualifiedName, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Body {
    Newtype { of: TypeRef },
    Struct { members: Vec<TypeRef> },
    Enum,
    Union { members: Vec<TypeRef> },
}

/// Bounded nominal declarations supporting an accessor's observation capability.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeFacts {
    nodes: BTreeMap<QualifiedName, Body>,
}

impl TypeFacts {
    pub(crate) fn of(ir: &EssIr, root: &ResolvedTypeRef) -> Result<Self, String> {
        let mut result = Self::default();
        let mut seen = BTreeSet::new();
        let mut pending = vec![root];
        let mut edges = 0_usize;
        let mut bytes = 0_usize;
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            if seen.len() > 4096 {
                return Err("AccessorResource: observation type nodes exceed 4096".into());
            }
            bytes = bytes.saturating_add(ty.to_string().len().saturating_mul(3) + 128);
            if bytes > 1_048_576 {
                return Err("AccessorResource: observation type bytes exceed 1_048_576".into());
            }
            let children: Vec<_> = match ty {
                ResolvedTypeRef::Primitive { .. } => Vec::new(),
                ResolvedTypeRef::Optional { of }
                | ResolvedTypeRef::List { of }
                | ResolvedTypeRef::Map { value: of, .. } => vec![of.as_ref()],
                ResolvedTypeRef::Declared { name } => {
                    let body = &ir.named_type(name).body;
                    let count = match body {
                        ResolvedBody::Struct { fields, .. } => fields.len(),
                        ResolvedBody::Union { variants, .. } => variants.len(),
                        ResolvedBody::Newtype { .. } => 1,
                        ResolvedBody::Enum { .. } => 0,
                    };
                    if edges.saturating_add(count) > 16384 {
                        return Err("AccessorResource: observation type edges exceed 16384".into());
                    }
                    let children: Vec<_> = match body {
                        ResolvedBody::Newtype { of, .. } => vec![of],
                        ResolvedBody::Struct { fields, .. } => {
                            fields.iter().map(|f| &f.type_ref).collect()
                        }
                        ResolvedBody::Union { variants, .. } => variants.values().collect(),
                        ResolvedBody::Enum { .. } => Vec::new(),
                    };
                    for child in &children {
                        bytes =
                            bytes.saturating_add(child.to_string().len().saturating_mul(3) + 32);
                        if bytes > 1_048_576 {
                            return Err(
                                "AccessorResource: observation type bytes exceed 1_048_576".into(),
                            );
                        }
                    }
                    let declared = match body {
                        ResolvedBody::Newtype { of, .. } => Body::Newtype {
                            of: super::accessor::unresolve(of),
                        },
                        ResolvedBody::Struct { .. } => Body::Struct {
                            members: children
                                .iter()
                                .map(|ty| super::accessor::unresolve(ty))
                                .collect(),
                        },
                        ResolvedBody::Union { .. } => Body::Union {
                            members: children
                                .iter()
                                .map(|ty| super::accessor::unresolve(ty))
                                .collect(),
                        },
                        ResolvedBody::Enum { .. } => Body::Enum,
                    };
                    result.nodes.insert(name.name().clone(), declared);
                    children
                }
            };
            edges += children.len();
            if edges > 16384 {
                return Err("AccessorResource: observation type edges exceed 16384".into());
            }
            pending.extend(children.into_iter().rev());
        }
        Ok(result)
    }

    fn children<'a>(
        &'a self,
        ty: &'a TypeRef,
        nested: bool,
    ) -> Result<Vec<(&'a TypeRef, bool)>, String> {
        let ambiguity =
            || "AmbiguousOptionalObservation: hidden nullable terminal representation".to_owned();
        Ok(match ty {
            TypeRef::Primitive(ess_domain::Primitive::Binary64) => {
                return Err("UnsupportedBinary64 accessor observation".into())
            }
            TypeRef::Primitive(_) => Vec::new(),
            TypeRef::Optional(of) => {
                if nested {
                    return Err(ambiguity());
                }
                vec![(of, true)]
            }
            TypeRef::List(of) | TypeRef::Map(_, of) => vec![(of, false)],
            TypeRef::Named(name) => match self
                .nodes
                .get(name)
                .ok_or("missing nominal observation type facts")?
            {
                Body::Newtype { of } => {
                    if matches!(of, TypeRef::Optional(_)) {
                        return Err(ambiguity());
                    }
                    vec![(of, nested)]
                }
                Body::Struct { members } | Body::Union { members } => {
                    if members.len() > 16384 {
                        return Err("AccessorResource: observation type edges".into());
                    }
                    members.iter().map(|ty| (ty, false)).collect()
                }
                Body::Enum => Vec::new(),
            },
        })
    }

    pub(crate) fn validate(&self, mut root: &TypeRef) -> Result<(), String> {
        while let TypeRef::Optional(of) = root {
            root = of;
        }
        if self.nodes.len() > 4096 {
            return Err("AccessorResource: observation type nodes".into());
        }
        let mut states = BTreeMap::new();
        let mut used = BTreeSet::new();
        let mut pending = vec![(root, false, false)];
        let mut edges = 0_usize;
        while let Some((ty, nested, finishing)) = pending.pop() {
            let key = (ty, nested);
            if finishing {
                let depth = self
                    .children(ty, nested)?
                    .iter()
                    .map(|child| states.get(child).copied().unwrap_or(0))
                    .max()
                    .unwrap_or(0)
                    + 1;
                if depth > 128 {
                    return Err("AccessorResource: observation type operations".into());
                }
                states.insert(key, depth);
                continue;
            }
            match states.get(&key) {
                Some(0) => {
                    return Err("AmbiguousOptionalObservation: recursive terminal type".into())
                }
                Some(_) => continue,
                None => {}
            }
            if states.len() >= 4096 {
                return Err("AccessorResource: observation type nodes".into());
            }
            if let TypeRef::Named(name) = ty {
                used.insert(name);
            }
            states.insert(key, 0);
            let children = self.children(ty, nested)?;
            edges += children.len();
            if edges > 16384 {
                return Err("AccessorResource: observation type edges".into());
            }
            pending.push((ty, nested, true));
            pending.extend(
                children
                    .into_iter()
                    .rev()
                    .map(|(ty, nested)| (ty, nested, false)),
            );
        }
        if used.len() != self.nodes.len() {
            return Err("unreachable nominal observation type facts".into());
        }
        Ok(())
    }
}
