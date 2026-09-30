//! Steps the terminal refuses because they cannot mean the same thing in both renderers.
//!
//! One test file runs headless against the terminal and is emitted as a Playwright spec for the
//! generated React project. Where the two renderers draw a node differently, a step at it would
//! pass in one and fail in the other. The terminal refuses such a step with a reason naming the
//! path, and the Playwright spec marks the same test `test.fixme` with the same reason
//! ([`crate::playwright`]). Both read the reasons from here, so the two lists cannot drift.
//!
//! The rules follow what each renderer draws on elements (React's `data-ui-path`) or cells of
//! their own (`ess_ui_tui::Region`):
//!
//! - a collection that is a section's or an overlay's body draws its rows, their row actions, a
//!   table's column headers, and a row's cells, except in cards, a list or a tree with an `item`,
//!   where React renders the item instead of the cells;
//! - a references list's rows are not addressed by React, so a path into them is refused;
//! - `text` reads a node that has cells of its own: the page, a section, an overlay, the page
//!   header and its actions, a section's child, and the collection parts above; and never an
//!   icon action or an action drawn as a choice, which show no label in the browser.

use ess_ui::{ActionAs, Body, Collection, Composite, Document, NodeRef};

use crate::target::Target;

/// What the node at a path holds rows as.
enum Held<'d> {
    Collection(&'d Collection),
    References,
    Other,
}

/// What the node at `path` holds, and whether it is a section's or an overlay's own body (the
/// only collections whose parts the terminal draws on cells of their own).
fn body_at<'d>(document: &'d Document, path: &str) -> Option<(Held<'d>, bool)> {
    let located = document
        .nodes()
        .into_iter()
        .find(|located| located.path.to_string() == path)?;
    let (body, own) = match located.node {
        NodeRef::Section(section) => (&section.body, true),
        NodeRef::Overlay(overlay) => (&overlay.body, true),
        NodeRef::Node(node) => (&node.body, false),
        _ => return None,
    };
    let held = match body {
        Body::Composite(Composite::Collection(collection)) => Held::Collection(collection),
        Body::Composite(Composite::References(_)) => Held::References,
        _ => Held::Other,
    };
    Some((held, own))
}

/// Whether `collection` draws as a table (no `style`, or `table`).
fn is_table(collection: &Collection) -> bool {
    matches!(
        collection.style,
        None | Some(ess_ui::CollectionStyle::Table)
    )
}

/// `columns/<name>` or `columns/all/<name>`.
fn is_column(rest: &[&str]) -> bool {
    matches!(rest, ["columns", name] if !name.is_empty())
        || matches!(rest, ["columns", "all", name] if !name.is_empty())
}

/// Why a step at `target` is refused because the generated React project renders no element at
/// it, if it is. Holds for every step, not only `text`.
pub(crate) fn unrendered(document: &Document, target: &Target) -> Option<String> {
    let path = &target.written;
    if let Some(row) = &target.row {
        let rest = relative(&target.node, &row.container)?;
        return match body_at(document, &row.container)?.0 {
            Held::References => Some(format!(
                "{path}: the generated app does not address the rows of a references list, so \
                 one test cannot select them in both renderers"
            )),
            Held::Collection(collection)
                if is_column(&rest) && !is_table(collection) && !collection.item.is_empty() =>
            {
                Some(format!(
                    "{path}: a {} collection with an `item` renders the item, not its column \
                     cells, in the generated app; address the row instead",
                    style_name(collection)
                ))
            }
            _ => None,
        };
    }
    let segments: Vec<&str> = target.node.split('/').collect();
    let at = segments.iter().rposition(|segment| *segment == "columns")?;
    let rest = &segments[at..];
    if !is_column(rest) {
        return None;
    }
    match body_at(document, &segments[..at].join("/"))?.0 {
        Held::Collection(collection) if !is_table(collection) => Some(format!(
            "{path}: a {} collection has no column headers in the generated app",
            style_name(collection)
        )),
        _ => None,
    }
}

/// Why `text` (or `not_text`) at `target` is refused, if it is: a node the generated app renders
/// no element at ([`unrendered`]), a node the terminal does not draw on cells of its own, or an
/// action that shows no label in the browser.
pub(crate) fn text(document: &Document, target: &Target) -> Option<String> {
    if let Some(reason) = unrendered(document, target) {
        return Some(reason);
    }
    let path = &target.written;
    let own_cells = || {
        format!(
            "{path}: the terminal does not draw this node on cells of its own, so its text cannot \
             be told from its neighbours'; expect text at the row, section, overlay or page instead"
        )
    };
    let segments: Vec<&str> = target.node.split('/').collect();
    if let Some(row) = &target.row {
        let Some(rest) = relative(&target.node, &row.container) else {
            return Some(own_cells());
        };
        let (held, own) = body_at(document, &row.container)?;
        if !own {
            return Some(own_cells());
        }
        let Held::Collection(collection) = held else {
            // A graph editor's rows are drawn as a collection's; the nodes inside them are not.
            return (!rest.is_empty()).then(own_cells);
        };
        return match rest.as_slice() {
            [] => None,
            _ if is_column(&rest) => None,
            ["row_actions", name] => collection
                .row_actions
                .iter()
                .find(|action| action.name == *name)
                .and_then(|action| labelless(action, path)),
            _ => Some(own_cells()),
        };
    }
    if target.is_page() || target.is_section() {
        return None;
    }
    if let Some((_, overlay)) = target.overlay() {
        if overlay == target.node {
            return None;
        }
    }
    match segments.as_slice() {
        ["pages", _, "header"] | ["pages", _, "sections", _, "children", _] => None,
        ["pages", page, "header", "actions", name] => document
            .pages
            .get(*page)
            .and_then(|page| page.header.as_ref())
            .and_then(|header| header.actions.iter().find(|action| action.name == *name))
            .and_then(|action| labelless(action, path)),
        _ => {
            let column = segments.iter().rposition(|segment| *segment == "columns");
            match column {
                Some(at)
                    if is_column(&segments[at..])
                        && matches!(
                            body_at(document, &segments[..at].join("/")),
                            Some((Held::Collection(_), true))
                        ) =>
                {
                    None
                }
                _ => Some(own_cells()),
            }
        }
    }
}

/// Why an action's text is refused: an icon, or an action drawn as a choice, shows no label.
fn labelless(action: &ess_ui::Action, path: &str) -> Option<String> {
    match action.action_as {
        Some(ActionAs::Icon) => Some(format!(
            "{path}: an icon action shows no label in the generated app"
        )),
        Some(ActionAs::Choice) if action.choice.is_some() => Some(format!(
            "{path}: an action drawn as a choice shows its options, not a label, in the \
             generated app"
        )),
        _ => None,
    }
}

/// `node`'s segments below `container`.
fn relative<'a>(node: &'a str, container: &str) -> Option<Vec<&'a str>> {
    if node == container {
        return Some(Vec::new());
    }
    node.strip_prefix(container)?
        .strip_prefix('/')
        .map(|rest| rest.split('/').collect())
}

fn style_name(collection: &Collection) -> &'static str {
    match collection.style {
        Some(ess_ui::CollectionStyle::Cards) => "cards",
        Some(ess_ui::CollectionStyle::List) => "list",
        Some(ess_ui::CollectionStyle::Tree) => "tree",
        _ => "table",
    }
}
