//! The paths read inside the expressions of a document, resolved: `section_refs` for
//! `section.<name>` and its member, `expr_paths` for `state.<name>`, `params.<name>` and
//! `shell.<name>`, and `expr_roots` for a dotted word that begins with no root, which stays
//! literal text.
//!
//! The sites are the fields the schema types `expr`, found by following each construct's field
//! types through `optional`, `list`, `map`, `record` and `one_of` over the document as written, so
//! a new `expr` field is covered without a rule of its own. A composite or primitive written
//! inline (`component`, `primitive`) adds the fields of its kind to those of the construct it
//! stands in; a node the document writes with neither takes the kind the loader gave it. A
//! widget's `args` are the one place a value is an expression though the schema types it `json`:
//! an argument is a path or a literal, so a string argument is read as an expression. A dotted
//! word under no root is called a stray there only when the widget's declaration substitutes that
//! parameter into an `expr` position outside the arguments of a call (directly or through another
//! widget it hands it to); where it lands in text alone, or in a call's arguments, the word is what
//! is shown or a command name. An argument may also name state the widget's body declares around
//! every read of the parameter. A page kind's nested nodes take the kind they restate or
//! inherit from the kinds it extends.
//! The row filter of a read is left to `filter_expr`, `filter_roots` and `filter_paths`, which
//! hold it to the same scope.

use std::collections::{BTreeMap, BTreeSet};

use ess_ui::{Body, Document, NodePath, NodeRef, Primitive};
use ess_yaml::Value;

use crate::expr;
use crate::filter::Scope;
use crate::raw::{locate, segment};
use crate::schema::{self, is_unmapped_marker, Kind};
use crate::walk::{body_of, composite_kind, page_of};
use crate::Sink;

pub(crate) fn run(text: &str, document: &Document, sink: &mut Sink) {
    let Ok(value) = ess_yaml::from_str::<Value>(text) else {
        return;
    };
    let located = document.nodes();
    let mut walker = Walker {
        document,
        nodes: located.iter().map(|located| located.path.clone()).collect(),
        kinds: located
            .iter()
            .filter_map(|located| {
                let kind = match body_of(located.node)? {
                    Body::Composite(composite) => {
                        Kind::Component(composite_kind(composite).to_owned())
                    }
                    Body::Widget(widget) => Kind::Component(widget.component.clone()),
                    Body::Primitive(primitive) => Kind::Primitive(primitive_kind(primitive)),
                    Body::Host(_) => return None,
                };
                Some((located.path.clone(), kind))
            })
            .collect(),
        seen: BTreeSet::new(),
        gathering: true,
        in_expr: BTreeMap::new(),
        declares: BTreeMap::new(),
        direct: BTreeMap::new(),
        forwards: Vec::new(),
        around: Vec::new(),
        learning: None,
    };
    // The loaded document lists a page kind as one node: its nested nodes take the kind they
    // restate, or the one the kind they extend gives them.
    let mut learned = BTreeSet::new();
    for name in value
        .get("page_kinds")
        .and_then(Value::as_mapping)
        .into_iter()
        .flatten()
        .filter_map(|(name, _)| name.as_str())
    {
        walker.learn(&value, name, &mut learned);
    }
    // A first walk learns which parameters of each widget its declaration substitutes into an
    // `expr` position, and the state its body declares around them; the second reports, knowing
    // that.
    walker.construct("Document", &value, &NodePath::root(), &mut Sink::default());
    walker.settle();
    walker.gathering = false;
    walker.seen.clear();
    walker.construct("Document", &value, &NodePath::root(), sink);
}

/// The name of a primitive, as `primitive:` writes it.
fn primitive_kind(primitive: &Primitive) -> String {
    match primitive {
        Primitive::Text(_) => "text",
        Primitive::Badge(_) => "badge",
        Primitive::Icon(_) => "icon",
        Primitive::Button(_) => "button",
        Primitive::Link(_) => "link",
        Primitive::Input(_) => "input",
        Primitive::Toggle(_) => "toggle",
        Primitive::Image(_) => "image",
        Primitive::Divider(_) => "divider",
    }
    .to_owned()
}

/// Where an expression stands.
#[derive(Clone)]
enum Site {
    /// A field typed `expr`.
    Field,
    /// The value of widget `widget`'s argument `param` (`None`: the widget is not known here).
    Argument {
        widget: Option<String>,
        param: String,
    },
}

struct Walker<'a> {
    document: &'a Document,
    nodes: BTreeSet<NodePath>,
    /// What each section, overlay and nested node holds once loaded, by its path: the kind of a
    /// node whose `component` or `primitive` the document leaves to a page kind or a `same_as`.
    kinds: BTreeMap<NodePath, Kind>,
    /// What has been reported, so a field two tables of one node both type is reported once.
    seen: BTreeSet<(&'static str, NodePath, String)>,
    /// `true` during the walk that only learns `in_expr` and `forwards`.
    gathering: bool,
    /// By widget, the parameters its declaration reads inside an `expr` position
    /// (`text: args.who`): an argument for one is an expression where it stands.
    in_expr: BTreeMap<String, BTreeSet<String>>,
    /// By widget and parameter, the state its body declares around every read of the parameter,
    /// on the node that reads it or on one around it: an argument may name it, since it is
    /// substituted in where it is declared. A state around one read alone does not resolve at
    /// the others. A parameter with no read has none.
    declares: BTreeMap<(String, String), BTreeSet<String>>,
    /// The same over the reads written directly in an `expr` position, before the hand-ons.
    direct: BTreeMap<(String, String), BTreeSet<String>>,
    /// A declaration that hands its parameter on as the argument of another widget.
    forwards: Vec<Forward>,
    /// The state declared by the nodes the walk is inside of, outermost first.
    around: Vec<String>,
    /// While a page kind is walked to learn the kinds of its nested nodes: the kinds it extends,
    /// nearest first.
    learning: Option<Vec<String>>,
}

struct Forward {
    widget: String,
    param: String,
    inner: String,
    inner_param: String,
    /// Read outside the arguments of a call.
    outside: bool,
    /// The state declared around where it is handed on.
    around: BTreeSet<String>,
}

/// The page kind `name`, declared in the document or built in.
fn page_kind<'v>(root: &'v Value, name: &str) -> Option<&'v Value> {
    root.get("page_kinds")
        .and_then(|declared| declared.get(name))
        .or_else(|| schema::builtin_page_kinds().get(name))
        .filter(|kind| kind.as_mapping().is_some())
}

impl Walker<'_> {
    fn report(&mut self, sink: &mut Sink, id: &'static str, at: &NodePath, message: &str) {
        let (node, field) = locate(&self.nodes, at);
        let message = if field.is_empty() {
            message.to_owned()
        } else {
            format!("{message} (in `{field}`)")
        };
        if self.seen.insert((id, node.clone(), message.clone())) {
            sink.push(id, &node, message);
        }
    }

    /// Walks page kind `name` after the kinds it extends, noting the kind of each node nested in
    /// it by `kinds`.
    fn learn(&mut self, root: &Value, name: &str, learned: &mut BTreeSet<String>) {
        if !learned.insert(name.to_owned()) {
            return;
        }
        let Some(body) = page_kind(root, name) else {
            return;
        };
        let mut chain: Vec<String> = Vec::new();
        let mut next = body.get("extends").and_then(Value::as_str);
        while let Some(parent) = next {
            if parent == name || chain.iter().any(|seen| seen == parent) {
                break;
            }
            chain.push(parent.to_owned());
            next = page_kind(root, parent)
                .and_then(|kind| kind.get("extends"))
                .and_then(Value::as_str);
        }
        for parent in &chain {
            self.learn(root, parent, learned);
        }
        self.learning = Some(chain);
        let at = NodePath::root().child("page_kinds").child(name);
        self.construct("PageKind", body, &at, &mut Sink::default());
        self.learning = None;
    }

    /// The kind of the node at `at` inside the page kind being learned: what it writes, or what
    /// the node of that name in a kind it extends has.
    fn learned_kind(&self, at: &NodePath, value: &Value) -> Option<Kind> {
        let chain = self.learning.as_ref()?;
        let [_, _, rest @ ..] = at.segments() else {
            return None;
        };
        if rest.is_empty() {
            return None;
        }
        if let Some(primitive) = value.get("primitive").and_then(Value::as_str) {
            return Some(Kind::Primitive(primitive.to_owned()));
        }
        if let Some(component) = value.get("component").and_then(Value::as_str) {
            return Some(Kind::Component(component.to_owned()));
        }
        chain.iter().find_map(|parent| {
            let mut under = NodePath::root().child("page_kinds").child(parent);
            for segment in rest {
                under = under.child(segment);
            }
            self.kinds.get(&under).cloned()
        })
    }

    /// The fields of `name` and, for a mapping that names a composite, a primitive or a widget
    /// inline, those of its kind: each present key is read as the type it has there.
    fn construct(&mut self, name: &str, value: &Value, at: &NodePath, sink: &mut Sink) {
        let Some(mapping) = value.as_mapping() else {
            return;
        };
        if let Some(kind) = self.learned_kind(at, value) {
            self.kinds.insert(at.clone(), kind);
        }
        let tables = schema::field_tables(name, mapping, self.kinds.get(at));
        let depth = self.around.len();
        self.around.extend(
            value
                .get("state")
                .and_then(Value::as_mapping)
                .into_iter()
                .flatten()
                .filter_map(|(state, _)| state.as_str().map(str::to_owned)),
        );
        let widget = match (
            mapping.get("component").and_then(Value::as_str),
            self.kinds.get(at),
        ) {
            (Some(own), _) => Some(own.to_owned()),
            (None, Some(Kind::Component(inherited))) if !mapping.contains_key("primitive") => {
                Some(inherited.clone())
            }
            _ => None,
        }
        .filter(|used| self.document.widgets.contains_key(used));
        for (key, child) in mapping {
            let Some(key) = key.as_str() else {
                continue;
            };
            for (owner, fields) in &tables {
                let Some(spec) = fields.get(key) else {
                    continue;
                };
                let at = at.child(key);
                match (owner.as_str(), key) {
                    // Held to the same scope by `filter_expr`, `filter_roots` and `filter_paths`.
                    ("Reads" | "DynamicNavEntries", "filter") => {}
                    // An argument is a path or a literal. Where the widget substitutes it into an
                    // `expr` position it is an expression, and a dotted word under no root is a
                    // stray; where it lands in text alone, that word is what is shown.
                    ("WidgetInstance", "args") => {
                        for (arg, text) in child.as_mapping().into_iter().flatten() {
                            if let Some(arg) = arg.as_str() {
                                let site = Site::Argument {
                                    widget: widget.clone(),
                                    param: arg.to_owned(),
                                };
                                self.expression(text, &at.child(arg), &site, sink);
                            }
                        }
                    }
                    _ => self.typed(&spec["type"], child, &at, Some(key), sink),
                }
            }
        }
        self.around.truncate(depth);
    }

    fn typed(
        &mut self,
        ty: &Value,
        value: &Value,
        at: &NodePath,
        key: Option<&str>,
        sink: &mut Sink,
    ) {
        if let Value::Tagged(tagged) = value {
            return self.typed(ty, &tagged.value, at, key, sink);
        }
        if let Some(name) = ty.as_str() {
            if name == "expr" {
                self.expression(value, at, &Site::Field, sink);
            } else if schema::is_construct(name) {
                self.construct(name, value, at, sink);
            }
            return;
        }
        if let Some(inner) = ty.get("optional") {
            self.typed(inner, value, at, key, sink);
        } else if let Some(inner) = ty.get("list") {
            for entry in value.as_sequence().into_iter().flatten() {
                match segment(entry, key) {
                    Some(name) => self.typed(inner, entry, &at.child(&name), key, sink),
                    None => self.typed(inner, entry, at, key, sink),
                }
            }
        } else if let Some(map) = ty.get("map") {
            for (entry_key, entry) in value.as_mapping().into_iter().flatten() {
                let Some(entry_key) = entry_key.as_str() else {
                    continue;
                };
                let under = at.child(entry_key);
                if map["key"].as_str() == Some("expr") {
                    self.expression(
                        &Value::String(entry_key.to_owned()),
                        &under,
                        &Site::Field,
                        sink,
                    );
                }
                self.typed(&map["value"], entry, &under, key, sink);
            }
        } else if let Some(branches) = ty.get("one_of") {
            for branch in branches.as_sequence().into_iter().flatten() {
                self.typed(branch, value, at, key, sink);
            }
        } else if let Some(record) = ty.get("record").and_then(Value::as_mapping) {
            for (field, field_ty) in record {
                if let Some((field, child)) =
                    field.as_str().zip(value.get(field.as_str().unwrap_or("")))
                {
                    self.typed(field_ty, child, &at.child(field), Some(field), sink);
                }
            }
        }
    }

    /// One value written where an expression stands.
    fn expression(&mut self, value: &Value, at: &NodePath, site: &Site, sink: &mut Sink) {
        let Some(text) = value.as_str() else {
            return;
        };
        if is_unmapped_marker(text) {
            return;
        }
        let Some(reading) = expr::read(text) else {
            return;
        };
        if self.gathering {
            self.gather(&reading, at, site);
        }
        let stray_words = match site {
            Site::Field => true,
            Site::Argument { widget, param } => widget
                .as_ref()
                .and_then(|widget| self.in_expr.get(widget))
                .is_some_and(|params| params.contains(param)),
        };
        for word in reading.strays.iter().filter(|_| stray_words) {
            let message = format!(
                "`{word}` reads no path: `{}` is no root ({}), so it stays literal text; write \
                 `\"{word}\"` to say so",
                word.split('.').next().unwrap_or(word),
                listed(&expr::roots())
            );
            self.report(sink, "expr_roots", at, &message);
        }
        // An argument may name state the widget's body declares where it substitutes it in.
        let body_state = match site {
            Site::Argument {
                widget: Some(widget),
                param,
            } => self
                .declares
                .get(&(widget.clone(), param.clone()))
                .cloned()
                .unwrap_or_default(),
            _ => BTreeSet::new(),
        };
        for path in &reading.paths {
            let segments: Vec<&str> = path.split('.').collect();
            match segments[0] {
                "section" => self.section(&segments, at, sink),
                "state" | "params" | "shell" => self.declared(&segments, at, &body_state, sink),
                _ => {}
            }
        }
    }

    /// Inside the declaration of a widget, notes each `args.<param>` it reads: in an `expr`
    /// position (outside the arguments of a call), or handed on as the argument of another
    /// widget; and the state its body declares around the read.
    fn gather(&mut self, reading: &expr::Reading, at: &NodePath, site: &Site) {
        let [top, widget, ..] = at.segments() else {
            return;
        };
        if top != "widgets" {
            return;
        }
        let around: BTreeSet<String> = self.around.iter().cloned().collect();
        for path in &reading.paths {
            let mut segments = path.split('.');
            let (Some("args"), Some(param)) = (segments.next(), segments.next()) else {
                continue;
            };
            let outside = reading.outside.contains(path);
            match site {
                Site::Field => {
                    self.direct
                        .entry((widget.clone(), param.to_owned()))
                        .and_modify(|held| held.retain(|state| around.contains(state)))
                        .or_insert_with(|| around.clone());
                    if outside {
                        self.in_expr
                            .entry(widget.clone())
                            .or_default()
                            .insert(param.to_owned());
                    }
                }
                Site::Argument {
                    widget: Some(inner),
                    param: inner_param,
                } => self.forwards.push(Forward {
                    widget: widget.clone(),
                    param: param.to_owned(),
                    inner: inner.clone(),
                    inner_param: inner_param.clone(),
                    outside,
                    around: around.clone(),
                }),
                Site::Argument { widget: None, .. } => {}
            }
        }
    }

    /// Carries `in_expr` and `declares` through the declarations that hand a parameter on to
    /// another widget. A hand-on is a read too: the state around it together with what the inner
    /// widget's parameter has. `declares` only shrinks from the first value a read gives it.
    fn settle(&mut self) {
        self.declares = self.direct.clone();
        loop {
            let mut grew = false;
            for forward in std::mem::take(&mut self.forwards) {
                let reaches = forward.outside
                    && self
                        .in_expr
                        .get(&forward.inner)
                        .is_some_and(|params| params.contains(&forward.inner_param));
                if reaches
                    && self
                        .in_expr
                        .entry(forward.widget.clone())
                        .or_default()
                        .insert(forward.param.clone())
                {
                    grew = true;
                }
                // An inner parameter nothing reads (yet) gives the hand-on no read to hold.
                if let Some(inner) = self
                    .declares
                    .get(&(forward.inner.clone(), forward.inner_param.clone()))
                {
                    let mut states = forward.around.clone();
                    states.extend(inner.iter().cloned());
                    let own = (forward.widget.clone(), forward.param.clone());
                    if let Some(held) = self.declares.get_mut(&own) {
                        let before = held.len();
                        held.retain(|state| states.contains(state));
                        grew |= held.len() != before;
                    } else {
                        self.declares.insert(own, states);
                        grew = true;
                    }
                }
                self.forwards.push(forward);
            }
            if !grew {
                break;
            }
        }
    }

    /// `section.<name>.<member>` names a section of the page it stands in and a member the
    /// reference lists.
    fn section(&mut self, segments: &[&str], at: &NodePath, sink: &mut Sink) {
        let Some((_, page)) = page_of(self.document, at) else {
            return; // a page kind, a widget declaration or a shell: no page to resolve in
        };
        let Some(name) = segments.get(1) else {
            return;
        };
        if !page.sections.iter().any(|section| section.name == *name) {
            let sections: Vec<&str> = page.sections.iter().map(|s| s.name.as_str()).collect();
            let message = format!(
                "`section.{name}` names no section of this page ({})",
                if sections.is_empty() {
                    "it has none".to_owned()
                } else {
                    format!("its sections are {}", listed(&sections))
                }
            );
            self.report(sink, "section_refs", at, &message);
            return;
        }
        // A bare `section.<name>` is the section's whole value and names no member to resolve.
        let Some(member) = segments.get(2) else {
            return;
        };
        let members = expr::section_members();
        if !members.iter().any(|listed| listed == member) {
            let message = format!(
                "`section.{name}.{member}` names no member a section has: the members are {}",
                listed(&members)
            );
            self.report(sink, "section_refs", at, &message);
        }
    }

    /// `true` inside an overlay that declares no `params`: its opener hands it values (the row it
    /// opens on), which are not known where the expression stands.
    fn opener_passes_params(&self, at: &NodePath) -> bool {
        self.document.nodes().iter().any(|ancestor| {
            matches!(ancestor.node, NodeRef::Overlay(overlay) if overlay.params.is_empty())
                && at.segments().starts_with(ancestor.path.segments())
        })
    }

    /// `state.<name>`, `params.<name>` and `shell.<name>` name a declaration in scope where the
    /// expression stands: the scope a row filter is held to.
    fn declared(
        &mut self,
        segments: &[&str],
        at: &NodePath,
        body_state: &BTreeSet<String>,
        sink: &mut Sink,
    ) {
        let Some(name) = segments.get(1) else {
            return;
        };
        let in_page = page_of(self.document, at).is_some();
        let top = at.segments();
        let in_shell = matches!(top, [shells, ..] if shells == "shells");
        // The frame of a shell stands outside every page: only its own state is known there.
        let frame = in_shell && top.get(2).is_none_or(|key| key != "overlays");
        let judged = match segments[0] {
            "shell" => in_page || in_shell,
            _ => in_page || (in_shell && !frame),
        };
        if !judged {
            return; // a page kind, a widget declaration or the navigation: scope not known here
        }
        if segments[0] == "params" && self.opener_passes_params(at) {
            return;
        }
        let scope = Scope::at(self.document, at);
        let (names, what) = match segments[0] {
            "state" => (&scope.state, "state"),
            "params" => (&scope.params, "parameter"),
            _ => (&scope.shell, "shell state"),
        };
        if names.contains(*name) || (segments[0] == "state" && body_state.contains(*name)) {
            return;
        }
        let in_scope: Vec<&str> = names.iter().map(String::as_str).collect();
        let message = if in_scope.is_empty() {
            format!(
                "`{}.{name}` names no {what}: no {what} is in scope here",
                segments[0]
            )
        } else {
            format!(
                "`{}.{name}` names no {what} in scope here; in scope: {}",
                segments[0],
                listed(&in_scope)
            )
        };
        self.report(sink, "expr_paths", at, &message);
    }
}

fn listed<S: AsRef<str>>(names: &[S]) -> String {
    names
        .iter()
        .map(|name| format!("`{}`", name.as_ref()))
        .collect::<Vec<_>>()
        .join(", ")
}
