//! Checked client row predicates, independently of server parameter binding.
use std::collections::BTreeSet;

use ess_primitives::predicate::{CompareOp, Operand, Predicate as ModelPredicate};
use ess_ui::filter::{self, Operator, Predicate};
use ess_ui::{Body, Composite, Document, Expr, NavPages, NodePath, NodeRef, Paging, Reads};

use crate::model::Model;
use crate::walk::{body_of, composite_reads, in_declaration, page_of};
use crate::Sink;

pub(crate) fn run(document: &Document, model: Option<&Model>, sink: &mut Sink) {
    for located in document.nodes() {
        if in_declaration(&located.path) {
            continue;
        }
        if let Some(Body::Composite(composite)) = body_of(located.node) {
            if let Some((key, reads)) = composite_reads(composite) {
                let allowed = !matches!(
                    composite,
                    Composite::Record(_) | Composite::Metric(_) | Composite::Form(_)
                );
                read(
                    document,
                    model,
                    reads,
                    &located.path.child(key),
                    allowed,
                    sink,
                );
            }
            if let Composite::GraphEditor(editor) = composite {
                if let Some(reads) = editor.edges.as_ref().and_then(|edges| edges.reads.as_ref()) {
                    read(
                        document,
                        model,
                        reads,
                        &located.path.child("edges").child("reads"),
                        true,
                        sink,
                    );
                }
            }
        }
        if let NodeRef::Action(action) = located.node {
            if let Some(reads) = &action.loads {
                read(
                    document,
                    model,
                    reads,
                    &located.path.child("loads"),
                    false,
                    sink,
                );
            }
            if let Some(export) = &action.export {
                if let Some(section) = export
                    .params
                    .as_ref()
                    .and_then(|expr| expr.0.strip_prefix("same_as("))
                    .and_then(|text| text.strip_suffix(')'))
                {
                    let filtered = page_of(document, &located.path).is_some_and(|(_, page)| page.sections.iter().any(|candidate| candidate.name == section && matches!(&candidate.body, Body::Composite(composite) if composite_reads(composite).is_some_and(|(_, reads)| reads.filter.is_some()))));
                    if filtered {
                        sink.push("filter_export", &located.path.child("export").child("params"), "`same_as` copies read params only; this export can include rows hidden by the section's client filter");
                    }
                }
            }
        }
        if let NodeRef::NavSection(nav) = located.node {
            if let NavPages::Dynamic(entries) = &nav.pages {
                if let Some(filter) = &entries.filter {
                    check(
                        document,
                        model,
                        filter,
                        Some(&entries.from_view),
                        &located.path.child("from_view").child("filter"),
                        sink,
                    );
                }
            }
        }
    }
}

fn read(
    document: &Document,
    model: Option<&Model>,
    reads: &Reads,
    path: &NodePath,
    allowed: bool,
    sink: &mut Sink,
) {
    let Some(filter) = &reads.filter else {
        return;
    };
    let path = path.child("filter");
    if !allowed {
        sink.push(
            "filter_place",
            &path,
            "a row filter belongs on a listing read or choice, not a record, metric or loads",
        );
    }
    if matches!(
        reads.paging,
        Some(Paging::Server | Paging::Cursor | Paging::Append)
    ) {
        sink.push("filter_paging", &path, "a client row filter requires `paging: client` or `none`; it cannot filter a server page");
    }
    check(document, model, filter, reads.view.as_deref(), &path, sink);
}

fn check(
    document: &Document,
    model: Option<&Model>,
    expr: &Expr,
    view: Option<&str>,
    path: &NodePath,
    sink: &mut Sink,
) {
    let Some(predicate) = filter::parse(&expr.0) else {
        sink.push(
            "filter_expr",
            path,
            "a filter must parse as a comparison, membership, not, and or or expression",
        );
        // Calls are outside this grammar, including rooted calls. Quoted text is skipped.
        if calls(&expr.0) {
            sink.push(
                "filter_roots",
                path,
                "function calls are not admitted in a client row filter",
            );
        }
        return;
    };
    if !predicate.boolean() {
        sink.push(
            "filter_expr",
            path,
            "a filter needs a boolean top-level form; bare paths and literals are not predicates",
        );
    }
    if !predicate.allowed_roots() {
        sink.push(
            "filter_roots",
            path,
            "only row, params, state, shell and widget args may be read by a client filter",
        );
    }
    let scope = Scope::at(document, path);
    let view = view
        .and_then(|view| model.and_then(|model| model.view(view)))
        .map(|(_, view)| view);
    for segments in predicate.paths() {
        let root = segments[0].as_str();
        let first = &segments[1];
        let known = match root {
            "params" => scope.params.contains(first),
            "state" => scope.state.contains(first),
            "shell" => scope.shell.contains(first),
            "args" => false, // expanded widgets have substituted their declared arguments
            "row" => view.is_none_or(|view| view.fields.contains_key(first)),
            _ => true, // filter_roots already names this refusal
        };
        if !known {
            sink.push(
                "filter_paths",
                path,
                format!(
                    "`{}` is not declared in this filter's scope",
                    segments.join(".")
                ),
            );
        }
    }
    if let Some(model_filter) = view.and_then(|view| view.filter.as_ref()) {
        let mut params = Vec::new();
        parameter_conjuncts(model_filter, &mut params);
        over_param(&predicate, &params, path, sink);
    }
}

fn calls(text: &str) -> bool {
    let mut quote = None;
    let mut previous = ' ';
    for c in text.chars() {
        if let Some(open) = quote {
            if c == open {
                quote = None;
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            previous = ' ';
            continue;
        }
        if c == '(' && (previous.is_ascii_alphanumeric() || previous == '_') {
            return true;
        }
        if !c.is_whitespace() {
            previous = c;
        }
    }
    false
}

#[derive(Default)]
struct Scope {
    params: BTreeSet<String>,
    state: BTreeSet<String>,
    shell: BTreeSet<String>,
}
impl Scope {
    fn at(document: &Document, path: &NodePath) -> Self {
        let mut scope = Self::default();
        let shell = page_of(document, path)
            .map(|(_, page)| page.shell.as_str())
            .or_else(|| match path.segments() {
                [top, shell, ..] if top == "shells" => Some(shell.as_str()),
                _ => None,
            });
        for (name, def) in &document.shells {
            if shell.is_none_or(|shell| shell == name) {
                scope.shell.extend(def.state.keys().cloned());
            }
        }
        // Dynamic menu entries are shell scoped; no page params or page state leaks into them.
        if path
            .segments()
            .first()
            .is_some_and(|top| top == "navigation")
        {
            scope.state.clone_from(&scope.shell);
            return scope;
        }
        for ancestor in document.nodes() {
            if !path.segments().starts_with(ancestor.path.segments()) {
                continue;
            }
            match ancestor.node {
                NodeRef::Page(page) => {
                    scope.params.extend(page.params.keys().cloned());
                    scope.state.extend(page.state.keys().cloned());
                }
                NodeRef::Overlay(overlay) => {
                    scope.params.extend(overlay.params.keys().cloned());
                    scope.state.extend(overlay.common.state.keys().cloned());
                }
                NodeRef::Section(section) => {
                    scope.state.extend(section.common.state.keys().cloned());
                }
                NodeRef::Node(node) => scope.state.extend(node.common.state.keys().cloned()),
                _ => {}
            }
        }
        scope
    }
}

fn parameter_conjuncts(predicate: &ModelPredicate, out: &mut Vec<(String, String)>) {
    match predicate {
        ModelPredicate::All(items) => items.iter().for_each(|item| parameter_conjuncts(item, out)),
        ModelPredicate::Compare {
            left: Operand::Fact(field),
            op: CompareOp::Eq,
            right: Operand::Fact(param),
            ..
        } => {
            let field = field.to_string();
            if let Some(param) = param.to_string().strip_prefix("param.") {
                if !field.contains('.') {
                    out.push((field, param.to_owned()));
                }
            }
        }
        _ => {}
    }
}

fn over_param(
    predicate: &Predicate,
    params: &[(String, String)],
    path: &NodePath,
    sink: &mut Sink,
) {
    match predicate {
        Predicate::Binary(Operator::And, left, right) => {
            over_param(left, params, path, sink);
            over_param(right, params, path, sink);
        }
        Predicate::Binary(Operator::Equal, left, right) => {
            if let Predicate::Path(field) = left.as_ref() {
                if field.len() == 2 && field[0] == "row" {
                    for (_, param) in params.iter().filter(|(name, _)| name == &field[1]) {
                        sink.push("filter_over_param", path, format!("the view can filter `{}` on the server; bind `params: {{{param}: {}}}` instead", field[1], expression(right)));
                    }
                }
            }
        }
        _ => {}
    }
}

fn expression(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Path(path) => path.join("."),
        Predicate::Literal(value) => {
            serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
        }
        _ => "<expression>".to_owned(),
    }
}
