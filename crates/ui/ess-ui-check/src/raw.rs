//! The checks whose subject is every value of the document as written: `unmapped_reported`
//! (`subject: "**"`) and the `channel.<name>` references inside expressions (`channel_refs`).
//!
//! These read the authored YAML rather than the typed document, because a marker or an
//! expression can stand in any string field and the typed document keeps no list of them. The
//! path is built the way `NodePath` builds it — map entries by key, list entries by `name`
//! (`field` for a field, `view` for a preload, the derived name for an unnamed action) — and a
//! list entry with none of those is reported at its list. A value written in a page kind or a
//! widget declaration is reported where it is written, once, not at every page or use it reaches.

use std::collections::BTreeSet;

use ess_ui::{Document, NodePath};
use serde_yaml::Value;

use crate::schema::{self, is_unmapped_marker};
use crate::Sink;

/// Keys whose string values are prose for a reader, never an expression.
const PROSE: &[&str] = &[
    "alt",
    "body",
    "confirm_label",
    "consequences",
    "doc",
    "endpoint",
    "derived",
    "label",
    "message",
    "note",
    "placeholder",
    "purpose",
    "source",
    "summary",
    "synonyms",
    "title",
    "transport_today",
];

pub(crate) fn run(text: &str, document: &Document, sink: &mut Sink) {
    let Ok(value) = serde_yaml::from_str::<Value>(text) else {
        return;
    };
    let walker = Walker {
        document,
        nodes: document
            .nodes()
            .into_iter()
            .map(|located| located.path)
            .collect(),
    };
    walker.value(&value, &NodePath::root(), None, sink);
}

struct Walker<'a> {
    document: &'a Document,
    nodes: BTreeSet<NodePath>,
}

impl Walker<'_> {
    /// Reports at the nearest node enclosing `at` (a finding names a node, not a field), with
    /// the field it was found in named in the message.
    fn report(&self, sink: &mut Sink, id: &'static str, at: &NodePath, message: String) {
        let segments = at.segments();
        let depth = (0..=segments.len())
            .rev()
            .find(|depth| *depth == 0 || self.nodes.contains(&path_of(&segments[..*depth])))
            .unwrap_or(0);
        let node = path_of(&segments[..depth]);
        let field = segments[depth..].join("/");
        if field.is_empty() {
            sink.push(id, &node, message);
        } else {
            sink.push(id, &node, format!("{message} (in `{field}`)"));
        }
    }

    fn value(&self, value: &Value, at: &NodePath, key: Option<&str>, sink: &mut Sink) {
        match value {
            Value::Mapping(mapping) => {
                for (child_key, child) in mapping {
                    let Some(child_key) = child_key.as_str() else {
                        continue;
                    };
                    if child_key == "fixtures" && at.segments().is_empty() {
                        continue;
                    }
                    if child_key == "unmapped" {
                        self.unmapped_list(child, &at.child("unmapped"), sink);
                        continue;
                    }
                    self.value(child, &at.child(child_key), Some(child_key), sink);
                }
            }
            Value::Sequence(entries) => {
                for entry in entries {
                    match segment(entry, key) {
                        Some(name) => self.value(entry, &at.child(&name), key, sink),
                        None => self.value(entry, at, key, sink),
                    }
                }
            }
            Value::String(text) => self.string(text, at, key, sink),
            Value::Tagged(tagged) => self.value(&tagged.value, at, key, sink),
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }

    fn unmapped_list(&self, value: &Value, at: &NodePath, sink: &mut Sink) {
        for entry in value.as_sequence().into_iter().flatten() {
            if let Some(gap) = entry.as_str() {
                self.report(
                    sink,
                    "unmapped_reported",
                    at,
                    format!("a gap the retrofit could not resolve: {gap}"),
                );
            }
        }
    }

    fn string(&self, text: &str, at: &NodePath, key: Option<&str>, sink: &mut Sink) {
        if is_unmapped_marker(text) {
            self.report(
                sink,
                "unmapped_reported",
                at,
                format!("a value the retrofit could not determine: `{text}`"),
            );
            return;
        }
        if key.is_some_and(|key| PROSE.contains(&key)) {
            return;
        }
        for name in unknown_channels(text, self.document) {
            self.report(
                sink,
                "channel_refs",
                at,
                format!("`channel.{name}` names no channel"),
            );
        }
    }
}

/// The path segment of a list entry held under `key`: its `name`, `field` or `view`, or — for an
/// unnamed action — the name the loader derives from the schema's `first_present` sources, so
/// the path is the one `Document::nodes` gives the action.
pub(crate) fn segment(entry: &Value, key: Option<&str>) -> Option<String> {
    if let Some(name) = ["name", "field", "view"]
        .iter()
        .find_map(|field| entry.get(field).and_then(Value::as_str))
    {
        return Some(name.to_owned());
    }
    let is_action = key.is_some_and(|key| {
        ess_ui::positions()
            .actions
            .contains(&(key.to_owned(), ess_ui::Shape::List))
    });
    if !is_action {
        return None;
    }
    let mapping = entry.as_mapping()?;
    schema::action_name_sources()
        .iter()
        .find_map(|source| derived(mapping, source))
}

/// One `first_present` source: a dotted path into the action, whose last segment may be the
/// schema's `last_segment_snake_case` or `first_key` transform.
fn derived(mapping: &serde_yaml::Mapping, source: &str) -> Option<String> {
    let mut segments = source.split('.');
    let mut current = mapping.get(segments.next()?)?;
    for segment in segments {
        match segment {
            "last_segment_snake_case" => {
                return current
                    .as_str()
                    .and_then(|text| text.rsplit('.').next())
                    .map(snake_case);
            }
            "first_key" => {
                return current
                    .as_mapping()
                    .and_then(|map| map.keys().next())
                    .and_then(|key| key.as_str().map(str::to_owned));
            }
            field => current = current.get(field)?,
        }
    }
    match current {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn snake_case(text: &str) -> String {
    let mut out = String::new();
    for (index, character) in text.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(character);
        }
    }
    out
}

fn is_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

/// The channel names `channel.<name>` references in `text` that the document does not declare.
/// A channel name may contain dots, so a reference resolves when a declared name is a prefix of
/// what follows `channel.` and ends at a name boundary or a dot.
fn unknown_channels(text: &str, document: &Document) -> Vec<String> {
    let mut unknown = Vec::new();
    for (index, _) in text.match_indices("channel.") {
        let preceded = text[..index]
            .chars()
            .next_back()
            .is_some_and(|before| is_name_character(before) || before == '.');
        if preceded {
            continue;
        }
        let rest = &text[index + "channel.".len()..];
        if rest.is_empty() {
            continue;
        }
        let declared = document.channels.keys().any(|channel| {
            rest.strip_prefix(channel.as_str()).is_some_and(|after| {
                after
                    .chars()
                    .next()
                    .is_none_or(|next| next == '.' || !is_name_character(next))
            })
        });
        if !declared {
            let name: String = rest.chars().take_while(|c| is_name_character(*c)).collect();
            if !name.is_empty() {
                unknown.push(name);
            }
        }
    }
    unknown
}

fn path_of(segments: &[String]) -> NodePath {
    segments
        .iter()
        .fold(NodePath::root(), |path, segment| path.child(segment))
}
