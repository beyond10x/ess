//! `enum_values`: a value written where the schema declares a closed enum is one of its values.
//!
//! Most of the schema's enums are Rust enums in `ess-ui`, so the loader refuses a value outside
//! them. The fields below are read as open strings (or, for `badge.tone_by.map`, as data) and are
//! checked here, against the values the schema lists ([`schema::enum_fields`]). The unit test
//! fails when the schema declares an enum this module neither checks nor names as typed.

use ess_ui::{Body, ChartKind, Composite, Document, Located, NodePath, NodeRef, Primitive};
use serde_yaml::Value;

use crate::schema::{self, is_unmapped_marker};
use crate::walk::in_declaration;
use crate::Sink;

/// The enum fields this module checks, as `schema::enum_fields` keys them.
#[cfg(test)]
const CHECKED: &[&str] = &[
    "Action.export.as",
    "Channel.lifecycle",
    "Channel.scope",
    "Field.as",
    "FormGroup.save",
    "Navigation.search.over",
    "State.clear_on",
    "State.scope",
    "badge.tone_by.map.*",
    "chart.chart",
    "divider.orientation",
    "image.fit",
    "input.as",
];

pub(crate) fn run(document: &Document, located: &[Located<'_>], sink: &mut Sink) {
    if let Some(search) = &document.navigation.search {
        let at = NodePath::root().child("navigation");
        for over in &search.over {
            value(sink, &at, "Navigation.search.over", "search.over", over);
        }
    }
    for node in located {
        let path = &node.path;
        if in_declaration(path) {
            continue;
        }
        match node.node {
            NodeRef::Field(field) => {
                if let Some(written) = &field.field_as {
                    value(sink, path, "Field.as", "as", written);
                }
            }
            NodeRef::FormGroup(group) => {
                if let Some(written) = &group.save {
                    value(sink, path, "FormGroup.save", "save", written);
                }
            }
            NodeRef::Action(action) => {
                if let Some(export) = &action.export {
                    value(
                        sink,
                        path,
                        "Action.export.as",
                        "export.as",
                        &export.export_as,
                    );
                }
            }
            NodeRef::Channel(channel) => {
                if let Some(written) = &channel.scope {
                    value(sink, path, "Channel.scope", "scope", written);
                }
                for written in &channel.lifecycle {
                    value(sink, path, "Channel.lifecycle", "lifecycle", written);
                }
            }
            NodeRef::State(state) => {
                if let Some(written) = &state.scope {
                    value(sink, path, "State.scope", "scope", written);
                }
                for written in &state.clear_on {
                    value(sink, path, "State.clear_on", "clear_on", written);
                }
            }
            NodeRef::Section(section) => body(sink, path, &section.body),
            NodeRef::Overlay(overlay) => body(sink, path, &overlay.body),
            NodeRef::Node(nested) => body(sink, path, &nested.body),
            _ => {}
        }
    }
}

fn body(sink: &mut Sink, path: &NodePath, body: &Body) {
    match body {
        Body::Composite(Composite::Chart(chart)) => {
            if let ChartKind::Fixed(kind) = &chart.chart {
                value(sink, path, "chart.chart", "chart", kind);
            }
        }
        Body::Primitive(Primitive::Image(image)) => {
            if let Some(fit) = &image.fit {
                value(sink, path, "image.fit", "fit", fit);
            }
        }
        Body::Primitive(Primitive::Divider(divider)) => {
            if let Some(orientation) = &divider.orientation {
                value(
                    sink,
                    path,
                    "divider.orientation",
                    "orientation",
                    orientation,
                );
            }
        }
        Body::Primitive(Primitive::Input(input)) => {
            if let Some(written) = &input.input_as {
                value(sink, path, "input.as", "as", written);
            }
        }
        Body::Primitive(Primitive::Badge(badge)) => {
            let tones = badge
                .tone_by
                .as_ref()
                .and_then(|tone_by| tone_by.map.as_mapping());
            for tone in tones.into_iter().flatten().map(|(_, tone)| tone) {
                if let Value::String(tone) = tone {
                    value(sink, path, "badge.tone_by.map.*", "tone_by.map", tone);
                }
            }
        }
        _ => {}
    }
}

/// Reports `written` at `path` when it is neither a value of the enum `field` nor the
/// `UNMAPPED` marker, which an enum field accepts (`unmapped_marker.accepted_by`).
fn value(sink: &mut Sink, path: &NodePath, field: &str, key: &str, written: &str) {
    let Some(values) = schema::enum_fields().get(field) else {
        return;
    };
    if values.contains(written) || is_unmapped_marker(written) {
        return;
    }
    sink.push(
        "enum_values",
        path,
        format!(
            "`{written}` is not a value of `{key}`; the schema's `{field}` lists {}",
            values.iter().cloned().collect::<Vec<_>>().join(", ")
        ),
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::CHECKED;
    use crate::schema;

    /// The enum fields `ess-ui` reads into Rust enums, so the loader refuses a value outside them.
    const TYPED: &[&str] = &[
        "Action.as",
        "Channel.delivery",
        "Channel.direction",
        "Channel.resume",
        "Degrades.(capability)",
        "Document.actor",
        "Document.placement_profile",
        "Live.effect",
        "Live.when_paged_away",
        "Navigation.visibility",
        "Page.profile",
        "PageLayout.columns.width",
        "Preload.policy",
        "Primitive.primitive",
        "Reads.paging",
        "Region.kind",
        "Section.load",
        "Section.profile",
        "SectionStates.loading",
        "SectionStates.refreshing",
        "SectionStates.stale.mark",
        "StateClass.(value)",
        "Store.(value)",
        "Widget.arrange",
        "badge.tone",
        "button.tone",
        "choice.style",
        "collection.selection",
        "collection.selection.mode",
        "collection.sort.dir",
        "collection.sort.mode",
        "collection.style",
        "form.save",
        "icon.tone",
        "metric.aggregate",
        "metric.format",
        "overlay.kind",
        "rich_text.syntax",
        "text.format",
        "text.style",
    ];

    #[test]
    fn every_schema_enum_is_typed_or_checked() {
        let declared: BTreeSet<&str> = schema::enum_fields().keys().map(String::as_str).collect();
        let covered: BTreeSet<&str> = CHECKED.iter().chain(TYPED).copied().collect();
        assert_eq!(
            declared.difference(&covered).collect::<Vec<_>>(),
            Vec::<&&str>::new(),
            "schema enums neither typed in ess-ui nor checked by enum_values"
        );
        assert_eq!(
            covered.difference(&declared).collect::<Vec<_>>(),
            Vec::<&&str>::new(),
            "listed enums the schema does not declare"
        );
    }
}
