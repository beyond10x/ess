//! Questions about where a node sits and what its composite holds, shared by the document and
//! model checks.

use ess_ui::{Body, Composite, Document, NodePath, NodeRef, Page, Reads};

/// The member of the composite union, by its `component` name.
pub(crate) fn composite_kind(composite: &Composite) -> &'static str {
    match composite {
        Composite::Collection(_) => "collection",
        Composite::Record(_) => "record",
        Composite::Form(_) => "form",
        Composite::Choice(_) => "choice",
        Composite::FilterBar(_) => "filter_bar",
        Composite::Confirm(_) => "confirm",
        Composite::Metric(_) => "metric",
        Composite::Chart(_) => "chart",
        Composite::Board(_) => "board",
        Composite::GraphEditor(_) => "graph_editor",
        Composite::RichText(_) => "rich_text",
        Composite::References(_) => "references",
    }
}

/// The composite's own read, with the key it sits under (`reads`, or `loads` for a form).
pub(crate) fn composite_reads(composite: &Composite) -> Option<(&'static str, &Reads)> {
    match composite {
        Composite::Collection(collection) => {
            collection.reads.as_ref().map(|reads| ("reads", reads))
        }
        Composite::Record(record) => record.reads.as_ref().map(|reads| ("reads", reads)),
        Composite::Form(form) => form.loads.as_ref().map(|reads| ("loads", reads)),
        Composite::Choice(choice) => choice.reads.as_ref().map(|reads| ("reads", reads)),
        Composite::Metric(metric) => metric.reads.as_ref().map(|reads| ("reads", reads)),
        Composite::Chart(chart) => Some(("reads", &chart.reads)),
        Composite::Board(board) => Some(("reads", &board.reads)),
        Composite::GraphEditor(editor) => Some(("reads", &editor.reads)),
        Composite::References(references) => Some(("reads", &references.reads)),
        Composite::FilterBar(_) | Composite::Confirm(_) | Composite::RichText(_) => None,
    }
}

/// The page a path lies inside, if it lies inside one.
pub(crate) fn page_of<'a>(document: &'a Document, path: &NodePath) -> Option<(&'a str, &'a Page)> {
    match path.segments() {
        [pages, name, ..] if pages == "pages" => document
            .pages
            .get_key_value(name)
            .map(|(name, page)| (name.as_str(), page)),
        _ => None,
    }
}

/// The section of its page a path lies inside, if it lies inside one.
pub(crate) fn section_of<'a>(
    document: &'a Document,
    path: &NodePath,
) -> Option<&'a ess_ui::Section> {
    let (_, page) = page_of(document, path)?;
    match path.segments() {
        [_, _, sections, name, ..] if sections == "sections" => {
            page.sections.iter().find(|section| &section.name == name)
        }
        _ => None,
    }
}

/// `true` when the path is a section itself: `pages/<page>/sections/<section>`.
pub(crate) fn is_section(path: &NodePath) -> bool {
    matches!(path.segments(), [pages, _, sections, _] if pages == "pages" && sections == "sections")
}

/// The body of every node that has one: a section, an overlay (a page's, a shell's or an
/// action's inline confirm) and a nested node (a child, an item, a field's or an action's
/// choice, a board widget, a tab form, …), which is every construct `Document::nodes` yields
/// that can hold a composite, a primitive or a widget use.
pub(crate) fn body_of(node: NodeRef<'_>) -> Option<&Body> {
    match node {
        NodeRef::Section(section) => Some(&section.body),
        NodeRef::Overlay(overlay) => Some(&overlay.body),
        NodeRef::Node(node) => Some(&node.body),
        _ => None,
    }
}

/// `true` inside a widget declaration (`widgets/<name>/…`), where `args.<param>` is unbound and
/// no use site's profile applies. No per-node check runs there: every one runs at each use, on
/// the body the loader expanded with that use's arguments.
pub(crate) fn in_declaration(path: &NodePath) -> bool {
    path.segments().first().is_some_and(|top| top == "widgets")
}
