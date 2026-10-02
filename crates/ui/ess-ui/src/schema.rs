//! The embedded schema, read once, and the parts of it expansion is driven by.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde_yaml::{Mapping, Value};

/// The parsed schema.
pub(crate) struct Schema {
    value: &'static Value,
}

impl Schema {
    /// The schema compiled into this crate.
    pub(crate) fn embedded() -> Self {
        static PARSED: OnceLock<Value> = OnceLock::new();
        let value = PARSED.get_or_init(|| {
            serde_yaml::from_str(crate::SCHEMA).expect("the embedded schema is YAML")
        });
        Self { value }
    }

    /// The `expands_to` of the `shorthands.index` entry for `construct` at `at` (`""` for none).
    ///
    /// Every shorthand the loader implements asks for its entry here, so an entry the schema drops
    /// fails the first document that needs it rather than going quietly unimplemented.
    pub(crate) fn template(&self, construct: &str, at: &str) -> &'static Value {
        &self.entry(construct, at)["expands_to"]
    }

    /// The `accepts` of the `shorthands.index` entry for `construct` at `at`: what the short form
    /// may be. Every shorthand checks its input against this before expanding it.
    pub(crate) fn accepts(&self, construct: &str, at: &str) -> &'static Value {
        &self.entry(construct, at)["accepts"]
    }

    fn entry(&self, construct: &str, at: &str) -> &'static Value {
        self.value["shorthands"]["index"]
            .as_sequence()
            .and_then(|entries| {
                entries.iter().find(|entry| {
                    entry["construct"].as_str() == Some(construct)
                        && entry["at"].as_str().unwrap_or("") == at
                })
            })
            .unwrap_or_else(|| panic!("the schema has no shorthand for {construct} at `{at}`"))
    }

    /// The built-in token table (`constructs.Tokens.builtins`).
    pub(crate) fn builtin_tokens(&self) -> crate::model::Tokens {
        serde_yaml::from_value(self.value["constructs"]["Tokens"]["builtins"].clone())
            .expect("the schema declares a built-in token table")
    }

    /// The built-in page kinds (`constructs.PageKind.builtins`).
    pub(crate) fn builtin_kinds(&self) -> Mapping {
        self.value["constructs"]["PageKind"]["builtins"]
            .as_mapping()
            .cloned()
            .expect("the schema declares built-in page kinds")
    }

    /// Every key at which the schema expects a `Node`, `Action`, `Field` or `Reads`, found by
    /// walking each construct field's type expression (through `optional`, `list`, `map`,
    /// `one_of` and nested `record` fields).
    pub(crate) fn positions(&self) -> &'static Positions {
        static POSITIONS: OnceLock<Positions> = OnceLock::new();
        POSITIONS.get_or_init(|| self.scan_positions())
    }

    fn scan_positions(&self) -> Positions {
        let mut positions = Positions::default();
        let constructs = self.value["constructs"]
            .as_mapping()
            .expect("the schema declares constructs");
        for construct in constructs.values() {
            let Some(fields) = construct["fields"].as_mapping() else {
                continue;
            };
            for (key, spec) in fields {
                let Some(key) = key.as_str().filter(|key| !key.starts_with('(')) else {
                    continue;
                };
                positions.scan(key, &spec["type"], Shape::One);
            }
        }
        positions
    }

    /// The fields a page inherits from its kind (`constructs.Page.inheritance.applies_to`).
    pub(crate) fn inherited_fields(&self) -> Vec<String> {
        self.value["constructs"]["Page"]["inheritance"]["applies_to"]
            .as_sequence()
            .expect("the schema says what a page inherits")
            .iter()
            .filter_map(|field| field.as_str().map(str::to_owned))
            .collect()
    }
}

/// How a position holds its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shape {
    /// One value under the key.
    One,
    /// A list of values under the key.
    List,
    /// A map of values under the key.
    Map,
}

/// The keys at which the schema expects each construct that has a shorthand or a derived name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Positions {
    /// Keys holding a `Node`.
    pub nodes: BTreeSet<(String, Shape)>,
    /// Keys holding an `Action`.
    pub actions: BTreeSet<(String, Shape)>,
    /// Keys holding a `Field`.
    pub fields: BTreeSet<(String, Shape)>,
    /// Keys holding a `Reads`.
    pub reads: BTreeSet<(String, Shape)>,
}

impl Positions {
    fn scan(&mut self, key: &str, ty: &Value, shape: Shape) {
        match ty {
            Value::String(name) => {
                let set = match name.as_str() {
                    "Node" => &mut self.nodes,
                    "Action" => &mut self.actions,
                    "Field" => &mut self.fields,
                    "Reads" => &mut self.reads,
                    _ => return,
                };
                set.insert((key.to_owned(), shape));
            }
            Value::Mapping(constructor) => {
                if let Some(inner) = constructor.get("optional") {
                    self.scan(key, inner, shape);
                }
                if let Some(inner) = constructor.get("list") {
                    self.scan(key, inner, Shape::List);
                }
                if let Some(inner) = constructor.get("map").and_then(|map| map.get("value")) {
                    self.scan(key, inner, Shape::Map);
                }
                for alternative in constructor
                    .get("one_of")
                    .and_then(Value::as_sequence)
                    .into_iter()
                    .flatten()
                {
                    self.scan(key, alternative, shape);
                }
                if let Some(record) = constructor.get("record").and_then(Value::as_mapping) {
                    for (field, inner) in record {
                        if let Some(field) = field.as_str() {
                            self.scan(field, inner, Shape::One);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
