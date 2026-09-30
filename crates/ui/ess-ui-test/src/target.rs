//! Resolving a step's node path against the document.
//!
//! A path is a canonical node path (`ess_ui::NodePath`, the path `ess-ui-check` reports and the
//! generated React project renders as `data-ui-path`), or, for an item of a collection, the
//! collection's path, `rows`, the row's key and optionally the path of a node inside the row
//! relative to the collection: `pages/p/sections/list/rows/pt-003/row_actions/delete`. That is the
//! path React scopes a row's nodes to.

use ess_ui::{Body, Composite, Document, NodeRef};

/// A resolved path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Target {
    /// The path as written.
    pub written: String,
    /// The canonical path of the node, with any row scope removed.
    pub node: String,
    /// The collection and key when the path is inside a row.
    pub row: Option<Row>,
}

/// A row of a collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Row {
    /// The collection's canonical path.
    pub container: String,
    /// The row's key.
    pub key: String,
}

impl Target {
    fn segments(&self) -> Vec<&str> {
        self.node.split('/').collect()
    }

    /// The page the node is on: `pages/<page>/…`.
    pub fn page(&self) -> Option<String> {
        match self.segments().as_slice() {
            ["pages", page, ..] => Some((*page).to_owned()),
            _ => None,
        }
    }

    /// The section the node is in: `pages/<page>/sections/<section>/…`.
    pub fn section(&self) -> Option<String> {
        match self.segments().as_slice() {
            ["pages", _, "sections", section, ..] => Some((*section).to_owned()),
            _ => None,
        }
    }

    /// Whether the node is exactly a section.
    pub fn is_section(&self) -> bool {
        matches!(self.segments().as_slice(), ["pages", _, "sections", _])
    }

    /// Whether the node is exactly a page.
    pub fn is_page(&self) -> bool {
        matches!(self.segments().as_slice(), ["pages", _])
    }

    /// The overlay the node is in, and its path: a page's or shell's `overlays/<name>`, or an
    /// action's inline `confirm/overlay`.
    pub fn overlay(&self) -> Option<(String, String)> {
        let segments = self.segments();
        if let Some(at) = segments.iter().position(|segment| *segment == "overlays") {
            if let Some(name) = segments.get(at + 1) {
                return Some(((*name).to_owned(), segments[..at + 2].join("/")));
            }
        }
        let at = segments
            .windows(2)
            .position(|pair| pair == ["confirm", "overlay"])?;
        let action = segments.get(at.checked_sub(1)?)?;
        Some(((*action).to_owned(), segments[..at + 2].join("/")))
    }
}

/// Resolves `written` against `document`, or says which path names no node.
pub(crate) fn resolve(document: &Document, written: &str) -> Result<Target, String> {
    let nodes = document.nodes();
    let find = |path: &str| {
        nodes
            .iter()
            .find(|located| located.path.to_string() == path)
    };
    let segments: Vec<&str> = written.split('/').collect();
    for (at, segment) in segments.iter().enumerate() {
        if *segment != "rows" || at + 1 >= segments.len() {
            continue;
        }
        let container = segments[..at].join("/");
        let Some(located) = find(&container) else {
            continue;
        };
        if !holds_rows(located.node) {
            continue;
        }
        let rest = &segments[at + 2..];
        let node = if rest.is_empty() {
            container.clone()
        } else {
            format!("{container}/{}", rest.join("/"))
        };
        if find(&node).is_none() || rest.iter().any(|segment| segment.is_empty()) {
            return Err(format!("no node at {written}"));
        }
        return Ok(Target {
            written: written.to_owned(),
            node,
            row: Some(Row {
                container,
                key: segments[at + 1].to_owned(),
            }),
        });
    }
    if written.is_empty() || find(written).is_none() {
        return Err(format!("no node at {written}"));
    }
    Ok(Target {
        written: written.to_owned(),
        node: written.to_owned(),
        row: None,
    })
}

fn holds_rows(node: NodeRef<'_>) -> bool {
    let body = match node {
        NodeRef::Section(section) => &section.body,
        NodeRef::Node(node) => &node.body,
        NodeRef::Overlay(overlay) => &overlay.body,
        _ => return false,
    };
    matches!(
        body,
        Body::Composite(
            Composite::Collection(_) | Composite::References(_) | Composite::GraphEditor(_)
        )
    )
}
