//! Canonical node paths (`NodePath` in the schema) and the walk that yields every node.
//!
//! A path is the chain of container keys and names from the document root, joined with `/`:
//! map entries are named by their key, list entries by their `name`. No path holds a list index,
//! so inserting, removing or reordering a sibling leaves every other node's path unchanged.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::model::{
    Action, ActionConfirm, Body, Columns, Composite, Document, Field, FormGroup, Guard, Header,
    LayoutColumn, NavSection, Node, Overlay, Page, PageKind, PageLayout, Primitive, Reads, Region,
    Section, Shell, State, Tab, TabFields, TabForm, Widget,
};
use crate::LoadError;

/// The stable address of a node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NodePath(Vec<String>);

impl NodePath {
    /// The document root.
    pub fn root() -> Self {
        Self(Vec::new())
    }

    /// This path extended by one segment.
    #[must_use]
    pub fn child(&self, segment: &str) -> Self {
        let mut segments = self.0.clone();
        segments.push(segment.to_owned());
        Self(segments)
    }

    /// The segments, root first.
    pub fn segments(&self) -> &[String] {
        &self.0
    }
}

impl fmt::Display for NodePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            formatter.write_str("/")
        } else {
            formatter.write_str(&self.0.join("/"))
        }
    }
}

/// A node of the typed document.
#[derive(Debug, Clone, Copy)]
pub enum NodeRef<'a> {
    /// `shells/<name>`.
    Shell(&'a Shell),
    /// `shells/<name>/regions/<name>`.
    Region(&'a Region),
    /// `shells/<name>/guards/<name>`.
    Guard(&'a Guard),
    /// A state entry, under any `state` map.
    State(&'a State),
    /// `navigation/sections/<name>`.
    NavSection(&'a NavSection),
    /// `page_kinds/<name>`.
    PageKind(&'a PageKind),
    /// `widgets/<name>`.
    Widget(&'a Widget),
    /// `channels/<name>`.
    Channel(&'a crate::model::Channel),
    /// `pages/<name>`.
    Page(&'a Page),
    /// `pages/<name>/layout/columns/<name>`.
    LayoutColumn(&'a LayoutColumn),
    /// `pages/<name>/header`.
    Header(&'a Header),
    /// `pages/<name>/sections/<name>`.
    Section(&'a Section),
    /// An overlay of a page or shell, or an inline confirm.
    Overlay(&'a Overlay),
    /// A composite, widget instance or primitive nested anywhere.
    Node(&'a Node),
    /// A column or input.
    Field(&'a Field),
    /// An action.
    Action(&'a Action),
    /// A tab.
    Tab(&'a Tab),
    /// A form group.
    FormGroup(&'a FormGroup),
}

/// A node with its canonical path.
#[derive(Debug, Clone)]
pub struct Located<'a> {
    /// Its canonical path.
    pub path: NodePath,
    /// The node.
    pub node: NodeRef<'a>,
}

impl Document {
    /// Every node of the document with its canonical path, parents before children.
    pub fn nodes(&self) -> Vec<Located<'_>> {
        let mut walk = Walk::default();
        let root = NodePath::root();
        let shells = root.child("shells");
        for (name, shell) in &self.shells {
            let at = shells.child(name);
            walk.push(&at, NodeRef::Shell(shell));
            for (region, value) in &shell.regions {
                walk.push(&at.child("regions").child(region), NodeRef::Region(value));
            }
            for guard in &shell.guards {
                walk.push(
                    &at.child("guards").child(&guard.name),
                    NodeRef::Guard(guard),
                );
            }
            walk.states(&at, &shell.state);
            walk.overlays(&at, &shell.overlays);
        }
        let navigation = root.child("navigation").child("sections");
        for section in &self.navigation.sections {
            walk.push(
                &navigation.child(&section.name),
                NodeRef::NavSection(section),
            );
        }
        for (name, kind) in &self.page_kinds {
            walk.push(
                &root.child("page_kinds").child(name),
                NodeRef::PageKind(kind),
            );
        }
        for (name, widget) in &self.widgets {
            let at = root.child("widgets").child(name);
            walk.push(&at, NodeRef::Widget(widget));
            walk.node_list(&at, "body", &widget.body);
        }
        for (name, channel) in &self.channels {
            let at = root.child("channels").child(name);
            walk.push(&at, NodeRef::Channel(channel));
            if let Some(buffer) = &channel.buffer {
                walk.push(&at.child("buffer"), NodeRef::State(buffer));
            }
        }
        for (name, page) in &self.pages {
            let at = root.child("pages").child(name);
            walk.push(&at, NodeRef::Page(page));
            if let PageLayout::Columns(layout) = &page.layout {
                for column in &layout.columns {
                    walk.push(
                        &at.child("layout").child("columns").child(&column.name),
                        NodeRef::LayoutColumn(column),
                    );
                }
            }
            walk.states(&at, &page.state);
            if let Some(header) = &page.header {
                walk.header(&at.child("header"), header);
            }
            walk.sections(&at, &page.sections);
            walk.overlays(&at, &page.overlays);
        }
        walk.found
    }
}

#[derive(Default)]
struct Walk<'a> {
    found: Vec<Located<'a>>,
}

impl<'a> Walk<'a> {
    fn push(&mut self, path: &NodePath, node: NodeRef<'a>) {
        self.found.push(Located {
            path: path.clone(),
            node,
        });
    }

    fn states(&mut self, at: &NodePath, states: &'a BTreeMap<String, State>) {
        for (name, state) in states {
            self.push(&at.child("state").child(name), NodeRef::State(state));
        }
    }

    fn overlays(&mut self, at: &NodePath, overlays: &'a BTreeMap<String, Overlay>) {
        for (name, overlay) in overlays {
            self.overlay(&at.child("overlays").child(name), overlay);
        }
    }

    fn overlay(&mut self, at: &NodePath, overlay: &'a Overlay) {
        self.push(at, NodeRef::Overlay(overlay));
        self.states(at, &overlay.common.state);
        self.body(at, &overlay.body);
    }

    fn header(&mut self, at: &NodePath, header: &'a Header) {
        self.push(at, NodeRef::Header(header));
        self.actions(at, "actions", &header.actions);
        self.node_list(at, "metrics", &header.metrics);
    }

    fn sections(&mut self, at: &NodePath, sections: &'a [Section]) {
        for section in sections {
            let here = at.child("sections").child(&section.name);
            self.push(&here, NodeRef::Section(section));
            self.states(&here, &section.common.state);
            if let Some(action) = section
                .states
                .as_ref()
                .and_then(|states| states.empty.as_ref())
                .and_then(|empty| empty.action.as_ref())
            {
                self.action(&here.child("states").child("empty").child("action"), action);
            }
            self.body(&here, &section.body);
            self.node_list(&here, "children", &section.children);
        }
    }

    fn node_list(&mut self, at: &NodePath, key: &str, nodes: &'a [Node]) {
        for node in nodes {
            let name = node.common.name.as_deref().unwrap_or("");
            self.node(&at.child(key).child(name), node);
        }
    }

    fn node(&mut self, at: &NodePath, node: &'a Node) {
        self.push(at, NodeRef::Node(node));
        self.states(at, &node.common.state);
        self.body(at, &node.body);
    }

    fn single(&mut self, at: &NodePath, key: &str, node: Option<&'a Node>) {
        if let Some(node) = node {
            self.node(&at.child(key), node);
        }
    }

    fn fields(&mut self, at: &NodePath, key: &str, fields: &'a [Field]) {
        for field in fields {
            let here = at.child(key).child(&field.name);
            self.push(&here, NodeRef::Field(field));
            self.single(&here, "choice", field.choice.as_deref());
        }
    }

    fn actions(&mut self, at: &NodePath, key: &str, actions: &'a [Action]) {
        for action in actions {
            self.action(&at.child(key).child(&action.name), action);
        }
    }

    fn action(&mut self, at: &NodePath, action: &'a Action) {
        self.push(at, NodeRef::Action(action));
        self.single(at, "choice", action.choice.as_deref());
        if let Some(ActionConfirm::Inline(inline)) = &action.confirm {
            self.overlay(&at.child("confirm").child("overlay"), &inline.overlay);
        }
    }

    fn tabs(&mut self, at: &NodePath, tabs: &'a [Tab]) {
        for tab in tabs {
            let here = at.child("tabs").child(&tab.name);
            self.push(&here, NodeRef::Tab(tab));
            if let Some(TabFields::Fields(fields)) = &tab.fields {
                self.fields(&here, "fields", fields);
            }
            match &tab.form {
                Some(TabForm::Node(node)) => self.node(&here.child("form"), node),
                Some(TabForm::Action(action)) => self.action(&here.child("form"), action),
                None => {}
            }
        }
    }

    fn body(&mut self, at: &NodePath, body: &'a Body) {
        match body {
            Body::Composite(composite) => self.composite(at, composite),
            Body::Widget(instance) => self.node_list(at, "body", &instance.body),
            Body::Primitive(Primitive::Button(button)) => {
                self.action(&at.child("action"), &button.action);
            }
            Body::Primitive(Primitive::Toggle(toggle)) => {
                if let Some(action) = &toggle.action {
                    self.action(&at.child("action"), action);
                }
            }
            Body::Primitive(_) => {}
        }
    }

    fn composite(&mut self, at: &NodePath, composite: &'a Composite) {
        match composite {
            Composite::Collection(collection) => {
                match &collection.columns {
                    Some(Columns::Fixed(fields)) => self.fields(at, "columns", fields),
                    Some(Columns::Selectable(selectable)) => {
                        self.fields(&at.child("columns"), "all", &selectable.all);
                    }
                    Some(Columns::Unmapped(_)) | None => {}
                }
                self.actions(at, "row_actions", &collection.row_actions);
                self.actions(at, "bulk_actions", &collection.bulk_actions);
                self.actions(at, "actions", &collection.actions);
                self.single(at, "expand", collection.expand.as_deref());
                self.node_list(at, "item", &collection.item);
            }
            Composite::Record(record) => {
                self.fields(at, "fields", &record.fields);
                self.tabs(at, &record.tabs);
                self.node_list(at, "item", &record.item);
                self.actions(at, "actions", &record.actions);
            }
            Composite::Form(form) => {
                self.fields(at, "fields", &form.fields);
                for group in &form.groups {
                    let here = at.child("groups").child(&group.name);
                    self.push(&here, NodeRef::FormGroup(group));
                    self.fields(&here, "fields", &group.fields);
                    self.actions(&here, "actions", &group.actions);
                }
                self.tabs(at, &form.tabs);
                self.node_list(at, "parts", &form.parts);
                self.actions(at, "actions", &form.actions);
                self.single(at, "result", form.result.as_deref());
                self.single(at, "record", form.record.as_deref());
                if let Some(draft) = &form.draft {
                    self.push(&at.child("draft"), NodeRef::State(draft));
                }
            }
            Composite::FilterBar(bar) => {
                self.node_list(at, "choices", &bar.choices);
                self.fields(at, "inputs", &bar.inputs);
                self.actions(at, "actions", &bar.actions);
            }
            Composite::Confirm(confirm) => {
                self.actions(at, "alternatives", &confirm.alternatives);
            }
            Composite::Board(board) => {
                for (kind, node) in &board.widgets {
                    self.node(&at.child("widgets").child(kind), node);
                }
                self.actions(at, "item_actions", &board.item_actions);
            }
            Composite::GraphEditor(editor) => {
                self.actions(at, "node_actions", &editor.node_actions);
                self.actions(at, "edge_actions", &editor.edge_actions);
                self.node_list(at, "toolbar", &editor.toolbar);
            }
            Composite::References(references) => {
                self.fields(at, "columns", &references.columns);
            }
            Composite::Choice(_)
            | Composite::Metric(_)
            | Composite::Chart(_)
            | Composite::RichText(_) => {}
        }
    }
}

/// The checks that need the typed document: names unique among siblings, and the
/// `exactly_one_of` rules of `Action` and `Reads`.
pub(crate) fn check(document: &Document) -> Result<(), LoadError> {
    let mut seen = BTreeSet::new();
    for located in document.nodes() {
        if located.path.segments().iter().any(String::is_empty) {
            return Err(LoadError::new(
                located.path,
                "a node in a list needs a `name`",
            ));
        }
        if let Some(bad) = located
            .path
            .segments()
            .iter()
            .find(|segment| !is_segment(segment))
        {
            return Err(LoadError::new(
                located.path.clone(),
                format!(
                    "`{bad}` is not a node name: a name matches NodePath.syntax.segment_pattern \
                     `{SEGMENT_PATTERN}`"
                ),
            ));
        }
        if !seen.insert(located.path.clone()) {
            return Err(LoadError::new(
                located.path,
                "two siblings share this name (names_unique)",
            ));
        }
        let body = match located.node {
            NodeRef::Action(action) => {
                check_action(&located.path, action)?;
                if let Some(reads) = &action.loads {
                    check_reads(&located.path.child("loads"), reads)?;
                }
                None
            }
            NodeRef::Section(section) => Some(&section.body),
            NodeRef::Overlay(overlay) => Some(&overlay.body),
            NodeRef::Node(node) => Some(&node.body),
            _ => None,
        };
        if let Some(Body::Composite(composite)) = body {
            if let Some((key, reads)) = composite_reads(composite) {
                check_reads(&located.path.child(key), reads)?;
            }
        }
    }
    Ok(())
}

/// `NodePath.syntax.segment_pattern`; `tests/schema.rs` holds this text to the schema's.
pub const SEGMENT_PATTERN: &str = "^[A-Za-z0-9_.-]+$";

fn is_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_.-".contains(character))
}

fn composite_reads(composite: &Composite) -> Option<(&'static str, &Reads)> {
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

fn check_action(path: &NodePath, action: &Action) -> Result<(), LoadError> {
    let effects = [
        action.does.is_some(),
        action.opens.is_some(),
        action.navigate.is_some(),
        action.export.is_some(),
        action.upload.is_some(),
        action.copy.is_some(),
        !action.sets.is_empty(),
    ];
    match effects.iter().filter(|present| **present).count() {
        1 => Ok(()),
        count => Err(LoadError::new(
            path.clone(),
            format!(
                "an action has exactly one of does, opens, navigate, export, upload, copy, sets; \
                 this one has {count}"
            ),
        )),
    }
}

fn check_reads(path: &NodePath, reads: &Reads) -> Result<(), LoadError> {
    match (&reads.view, &reads.placeholder, &reads.fixture) {
        (Some(_), None, _) | (None, Some(_), Some(_)) => Ok(()),
        (None, Some(_), None) => Err(LoadError::new(
            path.clone(),
            "a placeholder read names the `fixture` that answers it",
        )),
        _ => Err(LoadError::new(
            path.clone(),
            "a read has exactly one of `view` and `placeholder`",
        )),
    }
}
