//! `TypeExpr` → TypeScript types, and each state's initial value.

use std::collections::{BTreeMap, BTreeSet};

use ess_ui::TypeExpr;
use serde_yaml::Value;

use crate::ts;

/// Lowers types, remembering every model type a document names but does not define.
pub(crate) struct Types<'a> {
    defined: &'a BTreeMap<String, TypeExpr>,
    pub(crate) model: BTreeSet<String>,
}

fn scalar(name: &str) -> Option<&'static str> {
    Some(match name {
        "string" | "duration" | "timestamp" | "date" | "time" | "expr" | "name" => "string",
        "integer" | "number" => "number",
        "boolean" => "boolean",
        "json" => "Json",
        _ => return None,
    })
}

impl<'a> Types<'a> {
    pub(crate) fn new(defined: &'a BTreeMap<String, TypeExpr>) -> Self {
        Self {
            defined,
            model: BTreeSet::new(),
        }
    }

    /// The TypeScript type, qualified with `M.` for names from `src/model.ts`.
    pub(crate) fn lower(&mut self, ty: &TypeExpr) -> String {
        self.lower_in(ty, "M.")
    }

    /// The TypeScript type as written inside `src/model.ts` itself.
    pub(crate) fn lower_local(&mut self, ty: &TypeExpr) -> String {
        self.lower_in(ty, "")
    }

    fn lower_in(&mut self, ty: &TypeExpr, prefix: &str) -> String {
        match ty {
            TypeExpr::Named(name) => {
                if let Some(scalar) = scalar(name) {
                    return if scalar == "Json" {
                        format!("{prefix}Json")
                    } else {
                        scalar.to_owned()
                    };
                }
                if !self.defined.contains_key(name) {
                    self.model.insert(name.clone());
                }
                format!("{prefix}{name}")
            }
            TypeExpr::List(list) => format!("Array<{}>", self.lower_in(&list.list, prefix)),
            TypeExpr::Map(map) => format!(
                "{{ [key: string]: {} }}",
                self.lower_in(&map.map.value, prefix)
            ),
            TypeExpr::Optional(optional) => {
                format!("{} | null", self.lower_in(&optional.optional, prefix))
            }
            TypeExpr::Enum(values) => {
                if values.values.is_empty() {
                    "never".to_owned()
                } else {
                    values
                        .values
                        .iter()
                        .map(|value| ts::string(value))
                        .collect::<Vec<_>>()
                        .join(" | ")
                }
            }
            TypeExpr::OneOf(one_of) => {
                let parts: Vec<String> = one_of
                    .one_of
                    .iter()
                    .map(|member| format!("({})", self.lower_in(member, prefix)))
                    .collect();
                if parts.is_empty() {
                    "never".to_owned()
                } else {
                    parts.join(" | ")
                }
            }
            TypeExpr::Record(record) => {
                if record.record.is_empty() {
                    return "{}".to_owned();
                }
                let fields: Vec<String> = record
                    .record
                    .iter()
                    .map(|(name, field)| {
                        format!("{}?: {}", ts::key(name), self.lower_in(field, prefix))
                    })
                    .collect();
                format!("{{ {} }}", fields.join("; "))
            }
            TypeExpr::Ref(_) => "string".to_owned(),
            TypeExpr::Const(constant) => ts::literal(&constant.value),
        }
    }

    /// The initial value of a state of this type with no `default`, as a literal.
    pub(crate) fn initial(&self, ty: &TypeExpr) -> String {
        self.initial_depth(ty, 0)
    }

    fn initial_depth(&self, ty: &TypeExpr, depth: usize) -> String {
        if depth > 16 {
            return "null".to_owned();
        }
        match ty {
            TypeExpr::Named(name) => match scalar(name) {
                Some("string") => "\"\"".to_owned(),
                Some("number") => "0".to_owned(),
                Some("boolean") => "false".to_owned(),
                Some(_) => "null".to_owned(),
                None => self.defined.get(name).map_or_else(
                    || "null".to_owned(),
                    |inner| self.initial_depth(inner, depth + 1),
                ),
            },
            TypeExpr::List(_) => "[]".to_owned(),
            TypeExpr::Map(_) | TypeExpr::Record(_) => "{}".to_owned(),
            TypeExpr::Enum(values) => values
                .values
                .first()
                .map_or_else(|| "null".to_owned(), |value| ts::string(value)),
            TypeExpr::OneOf(one_of) => one_of.one_of.first().map_or_else(
                || "null".to_owned(),
                |first| self.initial_depth(first, depth + 1),
            ),
            TypeExpr::Ref(_) => "\"\"".to_owned(),
            TypeExpr::Optional(_) => "null".to_owned(),
            TypeExpr::Const(constant) => ts::literal(&constant.value),
        }
    }
}

/// A default value as a literal.
pub(crate) fn default_literal(value: &Value) -> String {
    ts::literal(value)
}
