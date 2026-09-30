//! Naming the node a typed read refused.
//!
//! serde reports *what* was wrong (an unknown field, say) but not *where* once a value has been
//! buffered for a tagged union. So when the whole document is refused, the expanded value is
//! walked again, deepest nodes first, and each node is read on its own as the type its position
//! says it is. The first one refused is the node at fault, and its canonical path is known.

use serde::de::DeserializeOwned;
use serde_yaml::{Mapping, Value};

use std::collections::BTreeSet;

use crate::expand::entry_segment;
use crate::model::{Action, Channel, Field, Node, Overlay, Page, PageKind, Section, Shell, Widget};
use crate::schema::{Schema, Shape};
use crate::{LoadError, NodePath};

pub(crate) fn locate(expanded: &Value) -> Option<LoadError> {
    visit(expanded, &NodePath::root())
}

fn visit(value: &Value, at: &NodePath) -> Option<LoadError> {
    let Value::Mapping(mapping) = value else {
        if let Value::Sequence(entries) = value {
            return entries
                .iter()
                .find_map(|entry| visit(entry, &at.child(&entry_segment(entry))));
        }
        return None;
    };
    // A page kind is a partial template, read as types only inside the pages that use it.
    let template = at.segments().first().map(String::as_str) == Some("page_kinds");
    for (key, child) in mapping {
        let Some(key) = key.as_str() else { continue };
        if template || matches!(key, "types" | "props" | "default" | "args" | "fixtures") {
            continue;
        }
        if let Some(error) = visit(child, &at.child(key)) {
            return Some(error);
        }
    }
    let refusal = match classify(at.segments(), mapping) {
        Some(Expected::Page) => refused::<Page>(value),
        Some(Expected::PageKind) => refused::<PageKind>(value),
        Some(Expected::Shell) => refused::<Shell>(value),
        Some(Expected::Channel) => refused::<Channel>(value),
        Some(Expected::Widget) => refused::<Widget>(value),
        Some(Expected::Section) => refused::<Section>(value),
        Some(Expected::Overlay) => refused::<Overlay>(value),
        Some(Expected::Action) => refused::<Action>(value),
        Some(Expected::Field) => refused::<Field>(value),
        Some(Expected::Node) => refused::<Node>(value),
        None => None,
    };
    refusal.map(|message| LoadError::new(at.clone(), message))
}

enum Expected {
    Page,
    PageKind,
    Shell,
    Channel,
    Widget,
    Section,
    Overlay,
    Action,
    Field,
    Node,
}

/// What a map at this path is, from the container keys in the path.
fn classify(segments: &[String], mapping: &Mapping) -> Option<Expected> {
    let at = |index: usize| segments.get(index).map(String::as_str);
    let from_end = |back: usize| {
        segments
            .len()
            .checked_sub(back)
            .and_then(|index| segments.get(index))
            .map(String::as_str)
    };
    match (segments.len(), at(0)) {
        (2, Some("pages")) => return Some(Expected::Page),
        (2, Some("page_kinds")) => return Some(Expected::PageKind),
        (2, Some("shells")) => return Some(Expected::Shell),
        (2, Some("channels")) => return Some(Expected::Channel),
        (2, Some("widgets")) => return Some(Expected::Widget),
        (4, Some("pages")) if at(2) == Some("sections") => return Some(Expected::Section),
        _ => {}
    }
    if from_end(2) == Some("overlays")
        || (from_end(2), from_end(1)) == (Some("confirm"), Some("overlay"))
    {
        return Some(Expected::Overlay);
    }
    let positions = Schema::embedded().positions();
    let held = |set: &BTreeSet<(String, Shape)>, key: Option<&str>, shape: Shape| {
        key.is_some_and(|key| set.contains(&(key.to_owned(), shape)))
    };
    let is_node = mapping.contains_key("component") || mapping.contains_key("primitive");
    if held(&positions.actions, from_end(2), Shape::List)
        || (!is_node && held(&positions.actions, from_end(1), Shape::One))
    {
        return Some(Expected::Action);
    }
    if is_node {
        return Some(Expected::Node);
    }
    if mapping.contains_key("field") && held(&positions.fields, from_end(2), Shape::List) {
        return Some(Expected::Field);
    }
    None
}

fn refused<T: DeserializeOwned>(value: &Value) -> Option<String> {
    serde_yaml::from_value::<T>(value.clone())
        .err()
        .map(|error| explain_marker(error.to_string()))
}

/// A marker refused by a boolean, number or record field is refused by decision, not by accident:
/// `unmapped_marker.accepted_by` lists string, enum and reference fields only.
fn explain_marker(message: String) -> String {
    if message.contains("string \"UNMAPPED: ") {
        format!(
            "an `UNMAPPED:` marker stands only in a string, enum or reference field, and this \
             field is none of those ({message})"
        )
    } else {
        message
    }
}
