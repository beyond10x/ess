//! Naming the node a typed read refused.
//!
//! serde reports *what* was wrong (an unknown field, say) but not *where* once a value has been
//! buffered for a tagged union. So when the whole document is refused, the expanded value is
//! walked again, deepest nodes first, and each node is read on its own as the type its position
//! says it is. The first one refused is the node at fault, and its canonical path is known.
//!
//! A position is read from the container keys above the node only. A list entry's segment (its
//! `name`, `field` or `view`) and the key of an entry of a map of named nodes (`state`,
//! `overlays`, … and the document's own maps of names, such as `tone_maps`) are names the author
//! chose: a form field `{field: action}` is a field, a state named `states` a state and a label
//! map named `action` a label map, whatever a key of that name would hold. A section's `states`
//! and the states in it are read as such only where the schema puts them, on a section.

use ess_yaml::{Mapping, Value};
use serde::de::DeserializeOwned;

use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::expand::entry_segment;
use crate::model::{
    Action, Channel, EmptyState, FailedState, Field, ForbiddenState, Header, HeaderBadge, Node,
    Overlay, Page, PageKind, Section, SectionStates, Shell, StaleState, Widget,
};
use crate::schema::{Schema, Shape};
use crate::{LoadError, NodePath};

pub(crate) fn locate(expanded: &Value) -> Option<LoadError> {
    visit(expanded, &NodePath::root(), &[])
}

/// `keys` holds one entry per segment of `at`: the container key it is, or `None` where the
/// segment is a name, that of a list entry or the key of an entry of a map of named nodes.
fn visit<'a>(value: &'a Value, at: &NodePath, keys: &[Option<&'a str>]) -> Option<LoadError> {
    let Value::Mapping(mapping) = value else {
        if let Value::Sequence(entries) = value {
            let keys = [keys, &[None]].concat();
            return entries
                .iter()
                .find_map(|entry| visit(entry, &at.child(&entry_segment(entry)), &keys));
        }
        return None;
    };
    // A page kind is a partial template, read as types only inside the pages that use it.
    let template = at.segments().first().map(String::as_str) == Some("page_kinds");
    let grammar = Grammar::get();
    let names = match keys {
        [Some(key)] => grammar.named.contains(*key) || grammar.document_named.contains(*key),
        [.., Some(key)] => grammar.named.contains(*key),
        _ => false,
    };
    for (key, child) in mapping {
        let Some(key) = key.as_str() else { continue };
        if template
            || (!names && matches!(key, "types" | "props" | "default" | "args" | "fixtures"))
        {
            continue;
        }
        let keys = [keys, &[(!names).then_some(key)]].concat();
        if let Some(error) = visit(child, &at.child(key), &keys) {
            return Some(error);
        }
    }
    let refusal = match classify(keys, mapping) {
        Some(Expected::Page) => refused::<Page>(value),
        Some(Expected::PageKind) => refused::<PageKind>(value),
        Some(Expected::Shell) => refused::<Shell>(value),
        Some(Expected::Channel) => refused::<Channel>(value),
        Some(Expected::Widget) => refused::<Widget>(value),
        Some(Expected::Section) => refused::<Section>(value),
        Some(Expected::Overlay) => refused::<Overlay>(value),
        Some(Expected::States) => refused::<SectionStates>(value),
        Some(Expected::Empty) => refused::<EmptyState>(value),
        Some(Expected::Failed) => refused::<FailedState>(value),
        Some(Expected::Forbidden) => refused::<ForbiddenState>(value),
        Some(Expected::Stale) => refused::<StaleState>(value),
        Some(Expected::Header) => refused::<Header>(value),
        Some(Expected::HeaderBadge) => refused::<HeaderBadge>(value),
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
    /// A section's `states`.
    States,
    /// `states.empty`.
    Empty,
    /// `states.failed`.
    Failed,
    /// `states.forbidden`.
    Forbidden,
    /// `states.stale`.
    Stale,
    /// A page's `header`.
    Header,
    /// A page header's `badge`.
    HeaderBadge,
    Action,
    Field,
    Node,
}

/// What a map at this path is, from the container keys in the path (`None` for a segment that
/// names a list entry, which is never read as a key).
fn classify(keys: &[Option<&str>], mapping: &Mapping) -> Option<Expected> {
    let at = |index: usize| keys.get(index).copied().flatten();
    let from_end = |back: usize| {
        keys.len()
            .checked_sub(back)
            .and_then(|index| keys.get(index))
            .copied()
            .flatten()
    };
    match (keys.len(), at(0)) {
        (2, Some("pages")) => return Some(Expected::Page),
        (2, Some("page_kinds")) => return Some(Expected::PageKind),
        (2, Some("shells")) => return Some(Expected::Shell),
        (2, Some("channels")) => return Some(Expected::Channel),
        (2, Some("widgets")) => return Some(Expected::Widget),
        _ if section_at(keys) => return Some(Expected::Section),
        (3, Some("pages")) if at(2) == Some("header") => return Some(Expected::Header),
        (4, Some("pages")) if at(2) == Some("header") && at(3) == Some("badge") => {
            return Some(Expected::HeaderBadge)
        }
        _ => {}
    }
    if from_end(2) == Some("overlays")
        || (from_end(2), from_end(1)) == (Some("confirm"), Some("overlay"))
    {
        return Some(Expected::Overlay);
    }
    if states_at(keys) {
        return Some(Expected::States);
    }
    if let Some((last, above)) = keys.split_last() {
        if states_at(above) {
            match *last {
                Some("empty") => return Some(Expected::Empty),
                Some("failed") => return Some(Expected::Failed),
                Some("forbidden") => return Some(Expected::Forbidden),
                Some("stale") => return Some(Expected::Stale),
                _ => {}
            }
        }
    }
    let positions = Schema::embedded().positions();
    let held = |set: &BTreeSet<(String, Shape)>, key: Option<&str>, shape: Shape| {
        key.is_some_and(|key| set.contains(&(key.to_owned(), shape)))
    };
    let is_node = mapping.contains_key("component") || mapping.contains_key("primitive");
    // Under a type expression or a map of expressions every key is author data: a record field or
    // a param named like an action position is not one.
    let in_data = keys.split_last().is_some_and(|(_, above)| {
        above
            .iter()
            .any(|key| matches!(key, Some("type" | "params")))
    });
    if !in_data
        && (held(&positions.actions, from_end(2), Shape::List)
            || (!is_node && held(&positions.actions, from_end(1), Shape::One)))
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

/// A section of a page (`pages.<page>.<key>[<entry>]`), at a key where `Page` holds a list of
/// `Section`. A page kind's sections are read only inside the pages that use the kind.
fn section_at(keys: &[Option<&str>]) -> bool {
    matches!(
        keys,
        [Some("pages"), None, Some(key), None] if Grammar::get().sections.contains(*key)
    )
}

/// A section's states: a key where `Section` holds its `SectionStates`, on a section.
fn states_at(keys: &[Option<&str>]) -> bool {
    matches!(
        keys.split_last(),
        Some((Some(key), above)) if Grammar::get().section_states.contains(*key) && section_at(above)
    )
}

/// The parts of the embedded schema that say which keys are names and where a section and its
/// states stand.
struct Grammar {
    /// The containers whose entries are named by their key (`NodePath.syntax.containers`, those
    /// with `child_name: map_key`), at any depth: a key of such a map is a name.
    named: BTreeSet<String>,
    /// The document's fields that map a name to a value (`themes`, `tone_maps`, …): at the top
    /// level a key of such a map is a name too.
    document_named: BTreeSet<String>,
    /// The keys at which `Page` holds a list of `Section`.
    sections: BTreeSet<String>,
    /// The keys at which `Section` holds its `SectionStates`.
    section_states: BTreeSet<String>,
}

impl Grammar {
    fn get() -> &'static Self {
        static GRAMMAR: OnceLock<Grammar> = OnceLock::new();
        GRAMMAR.get_or_init(|| {
            let schema: Value =
                ess_yaml::from_str(crate::SCHEMA).expect("the embedded schema is YAML");
            let constructs = &schema["constructs"];
            let named: BTreeSet<String> = constructs["NodePath"]["syntax"]["containers"]
                .as_sequence()
                .into_iter()
                .flatten()
                .filter(|entry| entry["child_name"].as_str() == Some("map_key"))
                .flat_map(|entry| entry["container"].as_sequence().into_iter().flatten())
                .filter_map(|container| container.as_str().map(str::to_owned))
                .collect();
            let fields_where = |construct: &str, holds: &dyn Fn(&Value) -> bool| {
                constructs[construct]["fields"]
                    .as_mapping()
                    .into_iter()
                    .flatten()
                    .filter(|(_, spec)| holds(&spec["type"]))
                    .filter_map(|(key, _)| key.as_str().map(str::to_owned))
                    .collect::<BTreeSet<String>>()
            };
            let grammar = Self {
                document_named: fields_where("Document", &|ty| {
                    ty["map"]["key"].as_str() == Some("name")
                }),
                sections: fields_where("Page", &|ty| ty["list"].as_str() == Some("Section")),
                section_states: fields_where("Section", &|ty| ty.as_str() == Some("SectionStates")),
                named,
            };
            assert!(
                grammar.named.contains("state")
                    && grammar.document_named.contains("tone_maps")
                    && grammar.sections.contains("sections")
                    && grammar.section_states.contains("states"),
                "the schema names the maps of names, a page's sections and a section's states"
            );
            grammar
        })
    }
}

fn refused<T: DeserializeOwned>(value: &Value) -> Option<String> {
    ess_yaml::from_value::<T>(value.clone())
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
