//! The paths read inside the expressions of a document, resolved: `section_refs` for
//! `section.<name>` and its member, `expr_paths` for `state.<name>`, `params.<name>` and
//! `shell.<name>`, and `expr_roots` for a dotted word that begins with no root, which stays
//! literal text.
//!
//! The sites are the fields the schema types `expr`, found by following each construct's field
//! types through `optional`, `list`, `map`, `record` and `one_of` over the document as written, so
//! a new `expr` field is covered without a rule of its own. A composite or primitive written
//! inline (`component`, `primitive`) adds the fields of its kind to those of the construct it
//! stands in. A widget's `args` are the one place a value is an expression though the schema types
//! it `json`: an argument is a path or a literal, so a string argument is read as an expression.
//! The row filter of a read is left to `filter_expr`, `filter_roots` and `filter_paths`, which
//! hold it to the same scope.

use std::collections::BTreeSet;

use ess_ui::{Document, NodePath, NodeRef};
use ess_yaml::Value;

use crate::expr;
use crate::filter::Scope;
use crate::raw::{locate, segment};
use crate::schema::{self, is_unmapped_marker};
use crate::walk::page_of;
use crate::Sink;

pub(crate) fn run(text: &str, document: &Document, sink: &mut Sink) {
    let Ok(value) = ess_yaml::from_str::<Value>(text) else {
        return;
    };
    let mut walker = Walker {
        document,
        nodes: document
            .nodes()
            .into_iter()
            .map(|located| located.path)
            .collect(),
        seen: BTreeSet::new(),
    };
    walker.construct("Document", &value, &NodePath::root(), sink);
}

struct Walker<'a> {
    document: &'a Document,
    nodes: BTreeSet<NodePath>,
    /// What has been reported, so a field two tables of one node both type is reported once.
    seen: BTreeSet<(&'static str, NodePath, String)>,
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

    /// The fields of `name` and, for a mapping that names a composite, a primitive or a widget
    /// inline, those of its kind: each present key is read as the type it has there.
    fn construct(&mut self, name: &str, value: &Value, at: &NodePath, sink: &mut Sink) {
        let Some(mapping) = value.as_mapping() else {
            return;
        };
        let tables = schema::field_tables(name, mapping);
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
                    ("Reads", "filter") => {}
                    ("WidgetInstance", "args") => {
                        for (arg, text) in child.as_mapping().into_iter().flatten() {
                            if let Some(arg) = arg.as_str() {
                                self.expression(text, &at.child(arg), sink);
                            }
                        }
                    }
                    _ => self.typed(&spec["type"], child, &at, Some(key), sink),
                }
            }
        }
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
                self.expression(value, at, sink);
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
                    self.expression(&Value::String(entry_key.to_owned()), &under, sink);
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
    fn expression(&mut self, value: &Value, at: &NodePath, sink: &mut Sink) {
        let Some(text) = value.as_str() else {
            return;
        };
        if is_unmapped_marker(text) {
            return;
        }
        let Some(reading) = expr::read(text) else {
            return;
        };
        for word in &reading.strays {
            let message = format!(
                "`{word}` reads no path: `{}` is no root ({}), so it stays literal text; write \
                 `\"{word}\"` to say so",
                word.split('.').next().unwrap_or(word),
                listed(&expr::roots())
            );
            self.report(sink, "expr_roots", at, &message);
        }
        for path in &reading.paths {
            let segments: Vec<&str> = path.split('.').collect();
            match segments[0] {
                "section" => self.section(&segments, at, sink),
                "state" | "params" | "shell" => self.declared(&segments, at, sink),
                _ => {}
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
    fn declared(&mut self, segments: &[&str], at: &NodePath, sink: &mut Sink) {
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
        if names.contains(*name) {
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
