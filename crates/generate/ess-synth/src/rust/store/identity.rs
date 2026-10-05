//! Typed, target-independent identity order, without requiring Ord on domain wrappers.
use super::super::{layout::Layout, name};
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedTypeRef, TypeHandle};
use ess_domain::{name::QualifiedName, types::Primitive};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(super) struct Keys<'a> {
    ir: &'a EssIr,
    layout: &'a Layout,
    named: BTreeMap<QualifiedName, TypeHandle>,
    maps: bool,
    json: bool,
}

impl<'a> Keys<'a> {
    pub fn new(ir: &'a EssIr, layout: &'a Layout) -> Self {
        Self {
            ir,
            layout,
            named: BTreeMap::new(),
            maps: false,
            json: false,
        }
    }

    pub fn expression(&mut self, reference: &ResolvedTypeRef, value: &str) -> String {
        match reference {
            ResolvedTypeRef::Primitive { name } => match name {
                Primitive::Boolean => format!("MemoryKey::Boolean(*({value}))"),
                Primitive::Integer => format!("MemoryKey::Integer(*({value}))"),
                Primitive::Decimal => format!("MemoryKey::Text(({value}).0.clone())"),
                Primitive::String => format!("MemoryKey::Text(({value}).clone())"),
                Primitive::Uuid | Primitive::Timestamp | Primitive::Duration => {
                    format!("MemoryKey::Text(({value}).0.clone())")
                }
                Primitive::Bytes => format!("MemoryKey::Bytes(({value}).clone())"),
                Primitive::Json => {
                    self.json = true;
                    format!("memory_json({value})")
                }
                Primitive::Binary64 => unreachable!("native synthesis refuses Binary64"),
            },
            ResolvedTypeRef::Declared { name } => {
                self.named.insert(name.name().clone(), name.clone());
                format!("{}({value})", self.function(name.name()))
            }
            ResolvedTypeRef::Optional { of } => format!(
                "MemoryKey::Optional(({value}).as_ref().map(|value| Box::new({})))",
                self.expression(of, "value")
            ),
            ResolvedTypeRef::List { of } => format!(
                "MemoryKey::List(({value}).iter().map(|value| {}).collect())",
                self.expression(of, "value")
            ),
            ResolvedTypeRef::Map { key, value: item } => {
                self.maps = true;
                let key = self.expression(&ResolvedTypeRef::Primitive { name: *key }, "key");
                let item = self.expression(item, "value");
                format!(
                    "memory_map(({value}).iter().map(|(key, value)| ({key}, {item})).collect())"
                )
            }
        }
    }

    pub fn helpers(mut self) -> String {
        let mut out = String::new();
        let mut rendered = BTreeSet::new();
        while let Some((name, handle)) = self
            .named
            .iter()
            .find(|(name, _)| !rendered.contains(*name))
            .map(|(name, handle)| (name.clone(), handle.clone()))
        {
            rendered.insert(name.clone());
            let reference = ResolvedTypeRef::Declared { name: handle };
            let types = Layout::crate_ident(self.layout.package());
            let ty = self
                .layout
                .absolute_type(&reference)
                .replace("crate::", &format!("{types}::"));
            let body = self.ir.types()[&name].body.clone();
            let expression = match body {
                ResolvedBody::Newtype { of, .. } => self.expression(&of, "&value.0"),
                ResolvedBody::Struct { fields, .. } => {
                    let values = fields
                        .iter()
                        .map(|field| {
                            self.expression(
                                &field.type_ref,
                                &format!("&value.{}", name::value_ident(&field.name)),
                            )
                        })
                        .collect::<Vec<_>>();
                    format!("MemoryKey::List(vec![{}])", values.join(", "))
                }
                ResolvedBody::Enum { variants } => {
                    let mut arms = String::new();
                    for variant in variants {
                        let _ = writeln!(
                            arms,
                            "{ty}::{} => MemoryKey::Text({:?}.into()),",
                            name::pascal(&variant),
                            variant.to_string()
                        );
                    }
                    format!("match value {{ {arms} }}")
                }
                ResolvedBody::Union { variants, .. } => {
                    let mut arms = String::new();
                    for (variant, reference) in variants {
                        // A unit variant (ess/22) is keyed by its tag alone.
                        let Some(reference) = reference else {
                            let _ = writeln!(arms, "{ty}::{} => MemoryKey::List(vec![MemoryKey::Text({variant:?}.into())]),", name::pascal(&variant));
                            continue;
                        };
                        let key = self.expression(&reference, "value");
                        let _ = writeln!(arms, "{ty}::{}(value) => MemoryKey::List(vec![MemoryKey::Text({variant:?}.into()), {key}]),", name::pascal(&variant));
                    }
                    format!("match value {{ {arms} }}")
                }
            };
            let _ = writeln!(
                out,
                "\nfn {}(value: &{ty}) -> MemoryKey {{ let _ = value; {expression} }}",
                self.function(&name)
            );
        }
        out.push_str(KEY);
        if self.maps {
            out.push_str(MAP);
        }
        if self.json {
            out.push_str(JSON);
        }
        out
    }

    fn function(&self, name: &QualifiedName) -> String {
        let index = self
            .ir
            .types()
            .keys()
            .position(|candidate| candidate == name)
            .expect("resolved type");
        format!("memory_key_{index}")
    }
}

const KEY: &str = r"
// A common structural order; individual specifications use only a subset of the variants.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MemoryKey {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(String),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<MemoryKey>),
    Map(Vec<(MemoryKey, MemoryKey)>),
    Optional(Option<Box<MemoryKey>>),
}

";

const MAP: &str = r"
fn memory_map(mut pairs: Vec<(MemoryKey, MemoryKey)>) -> MemoryKey {
    pairs.sort();
    MemoryKey::Map(pairs)
}
";

const JSON: &str = r"
fn memory_json(value: &crate::json::Value) -> MemoryKey {
    match value {
        crate::json::Value::Null => MemoryKey::Null,
        crate::json::Value::Bool(value) => MemoryKey::Boolean(*value),
        crate::json::Value::Number(value) => MemoryKey::Number(value.clone()),
        crate::json::Value::Text(value) => MemoryKey::Text(value.clone()),
        crate::json::Value::Array(values) => MemoryKey::List(values.iter().map(memory_json).collect()),
        crate::json::Value::Object(pairs) => MemoryKey::Map(pairs.iter().map(|(key, value)| (MemoryKey::Text(key.clone()), memory_json(value))).collect()),
    }
}
";
