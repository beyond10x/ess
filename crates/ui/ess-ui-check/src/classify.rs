//! A document the loader refuses becomes one finding, filed under the check the refusal is an
//! instance of.
//!
//! `ess_ui::LoadError` carries a path and a message and no check id, so the id is read off the
//! message the loader writes, never off the authored text: the loader names the node in the
//! expanded document, which the authored text may not contain (a widget body reached through its
//! instance, a section inherited from a page kind). Each class has a test in `tests/checks.rs`
//! that trips it through the loader, so a reworded loader message fails a named test rather than
//! quietly filing its refusals under `document_loads`.
//!
//! The one refusal with no useful path is YAML's own duplicate key, which the parser reports at
//! the root; for it the text is re-read with duplicates kept, to find the key written twice. The
//! key is `names_unique` only when it names a node: when the mapping holding it sits under a key
//! whose entries the schema types as named constructs (`schema::node_maps`: a page, a widget, a
//! state, an overlay, …). Any other key is a property written twice, which the loader refuses and
//! no check of the list names: `document_loads`, at the node the property belongs to.

use std::collections::BTreeSet;
use std::fmt;

use ess_ui::{LoadError, NodePath};
use serde::de::{
    Deserialize, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde_yaml::{Mapping, Value};

use crate::Sink;
use crate::{raw, schema};

pub(crate) fn refusal(text: &str, error: &LoadError, sink: &mut Sink) {
    let message = error.message();
    if message.contains("duplicate entry") {
        match duplicated(text) {
            Some(Duplicate::Node(path)) => sink.push(
                "names_unique",
                &path,
                format!("this name is written twice among its siblings ({message})"),
            ),
            Some(Duplicate::Property(path, key)) => sink.push(
                "document_loads",
                &path,
                format!("the key `{key}` is written twice ({message})"),
            ),
            None => sink.push("document_loads", error.path(), message),
        }
        return;
    }
    sink.push(classify(message), error.path(), message);
}

fn classify(message: &str) -> &'static str {
    if message.contains("(names_unique)") {
        "names_unique"
    } else if message.contains("names no entry of `tone_maps`") {
        "tone_map_refs"
    } else if message.contains("names no overlay") || message.contains("`same_as` chains") {
        "same_as_resolves"
    } else if message.contains("names neither a member of the composite union nor a widget")
        || message.contains("of widget `")
        || message.contains("so a widget of that name could never be used")
        || message.contains("widget expansion exceeds")
        || (message.starts_with("widget `")
            && (message.contains("contains itself")
                || message.contains("has no param")
                || message.contains("needs the argument")))
    {
        "widget_expands"
    } else if message.contains("cannot stand here") {
        "layer_rules"
    } else if is_foreign_primitive_prop(message) {
        "primitive_props"
    } else {
        "document_loads"
    }
}

/// `true` for serde's `unknown field` refusal whose expected fields are exactly one primitive
/// kind's props: the refused node is that primitive, wherever it was written.
fn is_foreign_primitive_prop(message: &str) -> bool {
    let Some(expected) = message
        .strip_prefix("unknown field `")
        .and_then(|rest| rest.split_once(", expected ").map(|(_, expected)| expected))
    else {
        return false;
    };
    let fields: BTreeSet<String> = expected
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect();
    schema::primitive_props().contains(&fields)
}

/// What a YAML mapping writes twice.
enum Duplicate {
    /// A key naming a node, at the node's path.
    Node(NodePath),
    /// A property, at the path of the node it belongs to, with the key.
    Property(NodePath, String),
}

/// The first key a YAML mapping writes twice, read with duplicates kept. Its path is built the
/// way `raw` builds one: a list entry by the segment `raw::segment` derives for it, so an unnamed
/// action is named by its derived name. A property is reported at the nearest node enclosing its
/// mapping, as the loader walks the document with every first occurrence kept; when that
/// document does not load either, at the mapping itself.
fn duplicated(text: &str) -> Option<Duplicate> {
    let tree = Tree::deserialize(serde_yaml::Deserializer::from_str(text)).ok()?;
    let found = tree.duplicate(&NodePath::root(), Held::Root)?;
    if found.names_node {
        return Some(Duplicate::Node(found.holder.child(&found.key)));
    }
    let nodes: Option<BTreeSet<NodePath>> = serde_yaml::to_string(&tree.first_kept())
        .ok()
        .and_then(|kept| ess_ui::load_str(&kept).ok())
        .map(|document| {
            document
                .nodes()
                .into_iter()
                .map(|located| located.path)
                .collect()
        });
    let Some(nodes) = nodes else {
        return Some(Duplicate::Property(found.holder, found.key));
    };
    let segments = found.holder.segments();
    let owner = (0..=segments.len())
        .rev()
        .map(|depth| {
            segments[..depth]
                .iter()
                .fold(NodePath::root(), |path, segment| path.child(segment))
        })
        .find(|path| path.segments().is_empty() || nodes.contains(path))
        .unwrap_or_default();
    Some(Duplicate::Property(owner, found.key))
}

/// Where a value sits in its parent.
#[derive(Clone, Copy)]
enum Held<'a> {
    /// The document itself.
    Root,
    /// Under a mapping key.
    Key(&'a str),
    /// An entry of the list under a mapping key.
    Entry(&'a str),
}

/// A key written twice, and the mapping holding it.
struct Found {
    holder: NodePath,
    key: String,
    /// The holding mapping's entries are named nodes (`schema::node_maps`).
    names_node: bool,
}

/// A YAML value with every mapping entry kept, duplicates included.
enum Tree {
    Map(Vec<(Value, Tree)>),
    Seq(Vec<Tree>),
    Leaf(Value),
}

impl Tree {
    /// The first key written twice at or below this value, which sits at `at`, `held` in its
    /// parent.
    fn duplicate(&self, at: &NodePath, held: Held<'_>) -> Option<Found> {
        match self {
            Self::Map(entries) => {
                let mut seen = BTreeSet::new();
                for (name, value) in entries {
                    let name = key_text(name);
                    if !seen.insert(name.clone()) {
                        let names_node = matches!(
                            held,
                            Held::Key(key) if schema::node_maps().contains(key)
                        );
                        return Some(Found {
                            holder: at.clone(),
                            key: name,
                            names_node,
                        });
                    }
                    if let Some(found) = value.duplicate(&at.child(&name), Held::Key(&name)) {
                        return Some(found);
                    }
                }
                None
            }
            Self::Seq(entries) => {
                let key = match held {
                    Held::Key(key) | Held::Entry(key) => Some(key),
                    Held::Root => None,
                };
                entries.iter().find_map(|entry| {
                    let here = raw::segment(&entry.first_kept(), key)
                        .map_or_else(|| at.clone(), |name| at.child(&name));
                    entry.duplicate(&here, key.map_or(Held::Root, Held::Entry))
                })
            }
            Self::Leaf(_) => None,
        }
    }

    /// The value serde would have read had YAML kept the first of each duplicated key.
    fn first_kept(&self) -> Value {
        match self {
            Self::Map(entries) => {
                let mut mapping = Mapping::new();
                for (key, value) in entries {
                    if !mapping.contains_key(key) {
                        mapping.insert(key.clone(), value.first_kept());
                    }
                }
                Value::Mapping(mapping)
            }
            Self::Seq(entries) => Value::Sequence(entries.iter().map(Self::first_kept).collect()),
            Self::Leaf(value) => value.clone(),
        }
    }
}

fn key_text(key: &Value) -> String {
    match key {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => String::new(),
    }
}

impl<'de> Deserialize<'de> for Tree {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(TreeVisitor)
    }
}

struct TreeVisitor;

impl<'de> Visitor<'de> for TreeVisitor {
    type Value = Tree;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("any YAML value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Tree, E> {
        Ok(Tree::Leaf(Value::from(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Tree, E> {
        Ok(Tree::Leaf(Value::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Tree, E> {
        Ok(Tree::Leaf(Value::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Tree, E> {
        Ok(Tree::Leaf(Value::from(value)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Tree, E> {
        Ok(Tree::Leaf(Value::from(value)))
    }

    fn visit_unit<E>(self) -> Result<Tree, E> {
        Ok(Tree::Leaf(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Tree, E> {
        Ok(Tree::Leaf(Value::Null))
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Tree, D::Error> {
        Tree::deserialize(deserializer)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Tree, A::Error> {
        let mut entries = Vec::new();
        while let Some(entry) = access.next_element::<Tree>()? {
            entries.push(entry);
        }
        Ok(Tree::Seq(entries))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Tree, A::Error> {
        let mut entries = Vec::new();
        while let Some((key, value)) = access.next_entry::<Tree, Tree>()? {
            entries.push((key.first_kept(), value));
        }
        Ok(Tree::Map(entries))
    }

    fn visit_enum<A: EnumAccess<'de>>(self, access: A) -> Result<Tree, A::Error> {
        let (_tag, variant) = access.variant::<String>()?;
        variant.newtype_variant::<Tree>()
    }
}
