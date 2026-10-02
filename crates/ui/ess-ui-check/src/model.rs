//! The checks that need the ESS model: every view, command and event the document names exists
//! in it, every section's read is readable by some actor, and every read binds exactly the
//! parameters its view declares — and, from the same walk, the [`binding`] of a document to the
//! HTTP surface the model's served components answer.
//!
//! A name resolves when it is a qualified name of the model, or becomes one with the system's
//! name in front: in system `shop`, `stock.Items` names `shop.stock.Items`.
//!
//! # Readability is an approximation
//!
//! ESS grants commands, not views (`ActorSpec::may`), so readability is read from the grants: a
//! view counts as readable when some actor may invoke a command of the bounded context that owns
//! it. A section reading a view of a context no actor is granted anything in is shown to nobody.
//! The rule over-approximates — a grant to write one entity of a context counts as reading every
//! view of it — and it will stay an approximation until the model can state read grants. A
//! section rendered by a widget is judged by the reads of the widget's expanded body.
//!
//! A document with `actor: anonymous` is read by nobody signed in, so no grant decides what it
//! shows, and `section_readable` does not apply to it.
//!
//! Widget declarations are not checked here: `args.<param>` is unbound in them. Their views,
//! commands and events are checked at each use, on the expanded body.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use ess_compiler::ir::{DomainHandle, ResolvedBody, ResolvedTypeRef, ResolvedView};
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::component::Reach;
use ess_domain::types::Primitive;
use ess_gen::http::{self, Served};
use ess_ui::binding::{Binding, CommandRoute, ErrorRoute, QueryParam, ViewRoute};
use ess_ui::{
    Action, ActorSource, Body, Carries, Channel, Composite, Document, Expr, NavPages, NodePath,
    NodeRef, Paging, Reads, Region, RegionKind,
};
use serde_yaml::Value;

use crate::walk::{composite_reads, in_declaration, is_section, page_of, section_of};
use crate::{CheckError, Sink};

/// Fields by name (and wire name), each with the wire variants of its enum, if it is one.
pub(crate) type Fields = BTreeMap<String, Option<Vec<String>>>;

/// A compiled ESS model, indexed by the names a document can use.
///
/// Readability is an approximation: ESS grants commands, not views, so a view counts as readable
/// when some actor may invoke a command of the bounded context that owns it. A document with
/// `actor: anonymous` is not held to it, since no grant decides what an anonymous reader sees.
#[derive(Debug)]
pub struct Model {
    system: String,
    views: BTreeMap<String, View>,
    commands: BTreeSet<String>,
    /// Each command's input fields by qualified name, as [`View::fields`] holds a row's.
    pub(crate) inputs: BTreeMap<String, Fields>,
    events: BTreeSet<String>,
    readable: BTreeSet<DomainHandle>,
    /// The qualified names a document type can name: the model's types, entities and views.
    pub(crate) type_names: BTreeSet<String>,
}

/// One view of the model: the bounded context that owns it and the parameters it declares.
#[derive(Debug)]
pub(crate) struct View {
    domain: DomainHandle,
    params: Vec<Param>,
    /// Its row fields, by name and by wire name: the variants each holds when it is an enum.
    pub(crate) fields: Fields,
}

/// One declared view parameter: its name, and whether a read must bind it — every parameter but
/// an `Optional` one and the two a `paging:` block names, which the contract publishes optional.
#[derive(Debug)]
struct Param {
    name: String,
    required: bool,
}

/// Reads and compiles an ESS specification: one file, or a directory holding `system.yaml`
/// whose every `.yaml` and `.yml` file below it is read.
pub fn load_model(path: &Path) -> Result<Model, CheckError> {
    let files = spec_files(path)?;
    let base = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let mut labels = Vec::new();
    let mut texts = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file)
            .map_err(|error| CheckError(format!("cannot read {}: {error}", file.display())))?;
        let label = file
            .strip_prefix(base)
            .unwrap_or(file)
            .display()
            .to_string();
        labels.push(label);
        texts.push(text);
    }
    let sources: Vec<(String, String)> = labels.into_iter().zip(texts).collect();
    model_from_sources(&sources, path)
}

/// Compiles an ESS specification from `(label, text)` sources already selected — as a caller that
/// resolves a directory through its `ess-inputs.yaml` has them. `shown` names the model in a
/// refusal.
pub fn model_from_sources(sources: &[(String, String)], shown: &Path) -> Result<Model, CheckError> {
    compile_sources(sources, shown).map(|ir| Model::index(&ir))
}

/// The compiled IR of `(label, text)` sources, refused as [`model_from_sources`] refuses them.
pub fn compile_sources(sources: &[(String, String)], shown: &Path) -> Result<EssIr, CheckError> {
    let labels: Vec<String> = sources.iter().map(|(label, _)| label.clone()).collect();
    let texts: Vec<String> = sources.iter().map(|(_, text)| text.clone()).collect();
    let path = shown;
    let every: Vec<&str> = texts.iter().map(String::as_str).collect();
    let mut parsed = Vec::new();
    let mut sources = SourceMap::new();
    let mut problems = Vec::new();
    for ((label, text), result) in labels
        .iter()
        .zip(&texts)
        .zip(ess_domain::spec::RawSpecFile::parse_all(&every))
    {
        let source = ess_domain::system::Source::new(label.clone());
        sources.insert(source.as_str(), text.as_str());
        match result {
            Ok(raw) => parsed.push((source, raw)),
            Err(error) => problems.push(format!("{label}: {error}")),
        }
    }
    let refused = |problems: Vec<String>| {
        CheckError(format!(
            "the model {} does not compile:\n{}",
            path.display(),
            problems.join("\n")
        ))
    };
    if !problems.is_empty() {
        return Err(refused(problems));
    }
    let specification = ess_domain::spec::Specification::assemble(parsed)
        .map_err(|errors| refused(errors.as_slice().iter().map(ToString::to_string).collect()))?;
    ess_compiler::compile(&specification, &sources)
        .map_err(|diagnostics| refused(vec![diagnostics.to_string()]))
}

fn spec_files(path: &Path) -> Result<Vec<PathBuf>, CheckError> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.join("system.yaml").is_file() {
        return Err(CheckError(format!(
            "{} is not an ESS specification: give one file, or a directory holding `system.yaml`",
            path.display()
        )));
    }
    let mut files = Vec::new();
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(&directory)
            .map_err(|error| CheckError(format!("cannot read {}: {error}", directory.display())))?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                CheckError(format!("cannot read {}: {error}", directory.display()))
            })?;
            let child = entry.path();
            if child.is_dir() {
                pending.push(child);
            } else if child
                .extension()
                .is_some_and(|extension| extension == "yaml" || extension == "yml")
            {
                files.push(child);
            }
        }
    }
    files.sort();
    Ok(files)
}

impl Model {
    fn index(ir: &EssIr) -> Self {
        let readable = ir
            .actors()
            .values()
            .flat_map(|actor| &actor.may)
            .map(|command| ir.command(command).domain.clone())
            .collect();
        Self {
            system: ir.system().to_string(),
            views: ir
                .views()
                .iter()
                .map(|(name, view)| {
                    let params = view
                        .params
                        .iter()
                        .map(|param| Param {
                            name: param.name.clone(),
                            required: required(view, &param.name, &param.type_ref),
                        })
                        .collect();
                    (
                        name.to_string(),
                        View {
                            domain: view.domain.clone(),
                            params,
                            fields: fields_of(ir, &view.fields),
                        },
                    )
                })
                .collect(),
            commands: ir.commands().keys().map(ToString::to_string).collect(),
            inputs: ir
                .commands()
                .iter()
                .map(|(name, command)| (name.to_string(), fields_of(ir, &command.input)))
                .collect(),
            events: ir.events().keys().map(ToString::to_string).collect(),
            readable,
            type_names: ir
                .types()
                .keys()
                .chain(ir.entities().keys())
                .chain(ir.views().keys())
                .map(ToString::to_string)
                .collect(),
        }
    }

    /// The model's qualified name for `name`, when it has one.
    fn qualify(&self, name: &str, known: impl Fn(&str) -> bool) -> Option<String> {
        let prefixed = format!("{}.{name}", self.system);
        [name.to_owned(), prefixed]
            .into_iter()
            .find(|candidate| known(candidate))
    }

    pub(crate) fn view(&self, name: &str) -> Option<(String, &View)> {
        let qualified = self.qualify(name, |candidate| self.views.contains_key(candidate))?;
        let view = &self.views[&qualified];
        Some((qualified, view))
    }

    pub(crate) fn command(&self, name: &str) -> Option<String> {
        self.qualify(name, |candidate| self.commands.contains(candidate))
    }

    /// Whether `name` names a type, entity or view of the model: qualified, qualified but for the
    /// system, or by the trailing segments of some qualified name.
    pub(crate) fn has_type(&self, name: &str) -> bool {
        let suffix = format!(".{name}");
        self.qualify(name, |candidate| self.type_names.contains(candidate))
            .is_some()
            || self
                .type_names
                .iter()
                .any(|candidate| candidate.ends_with(&suffix))
    }

    fn has_event(&self, name: &str) -> bool {
        self.qualify(name, |candidate| self.events.contains(candidate))
            .is_some()
    }

    fn view_ref(&self, sink: &mut Sink, path: &NodePath, name: &str) {
        if self.view(name).is_none() {
            sink.push(
                "view_in_model",
                path,
                format!("`{name}` names no view of model `{}`", self.system),
            );
        }
    }

    fn command_ref(&self, sink: &mut Sink, path: &NodePath, name: &str) {
        if self.command(name).is_none() {
            sink.push(
                "command_in_model",
                path,
                format!("`{name}` names no command of model `{}`", self.system),
            );
        }
    }

    fn event_ref(&self, sink: &mut Sink, path: &NodePath, name: &str) {
        if !self.has_event(name) {
            sink.push(
                "event_in_model",
                path,
                format!("`{name}` names no event of model `{}`", self.system),
            );
        }
    }

    pub(crate) fn check(&self, document: &Document, sink: &mut Sink) {
        // `actor: anonymous`: nobody signs in, so no grant decides what is shown; ESS has no read
        // grants, so every view is read without one and `section_readable` does not apply. An
        // UNMAPPED actor is reported by `unmapped_reported` and decides nothing here either.
        let grants_apply = matches!(document.actor, None | Some(ActorSource::FromSession));
        for named in names(document) {
            match named.kind {
                Kind::Event(event) => self.event_ref(sink, &named.at, event),
                Kind::Command(command) => self.command_ref(sink, &named.at, command),
                Kind::View { name, bound, body } => {
                    let Some((qualified, view)) = self.view(name) else {
                        self.view_ref(sink, &named.at, name);
                        continue;
                    };
                    if body && grants_apply {
                        if let Some(section) = reading_section(document, &named.node) {
                            self.readable(sink, &section, &qualified, &view.domain);
                        }
                    }
                    if let Some(params) = bound_params(document, &named.node, bound) {
                        read_params(sink, &named.at, &qualified, view, params);
                    }
                }
            }
        }
        self.values(document, sink);
    }

    /// The qualified name and row fields of the view `reads` names, when the model has it.
    fn row_fields(&self, reads: Option<&Reads>) -> Option<(String, &Fields)> {
        let name = reads?.view.as_ref()?;
        self.view(name)
            .map(|(qualified, view)| (qualified, &view.fields))
    }

    /// The fields and values the document reads from rows and inputs the model types: a
    /// `group_by`, `group_order`, an aggregate's `field`, a `label_from` and a form choice's fixed
    /// options (beyond10x/ess#351, #358, #364, #330).
    fn values(&self, document: &Document, sink: &mut Sink) {
        for located in document.nodes() {
            let path = &located.path;
            if in_declaration(path) {
                continue;
            }
            if let NodeRef::Field(field) = located.node {
                if let Some(label_from) = &field.label_from {
                    if let Some((qualified, view)) = self.view(&label_from.view) {
                        for (key, name) in
                            [("field", &*label_from.field), ("key", label_from.key())]
                        {
                            let at = path.child("label_from").child(key);
                            row_field(sink, &at, &qualified, &view.fields, name);
                        }
                    }
                }
                continue;
            }
            let Some(Body::Composite(composite)) = crate::walk::body_of(located.node) else {
                continue;
            };
            match composite {
                Composite::Collection(collection) => {
                    let (Some(by), Some((view, fields))) = (
                        &collection.group_by,
                        self.row_fields(collection.reads.as_ref()),
                    ) else {
                        continue;
                    };
                    if row_field(sink, &path.child("group_by"), &view, fields, by) {
                        if let Some(Some(variants)) = fields.get(by.as_str()) {
                            enum_values(
                                sink,
                                &path.child("group_order"),
                                &format!("`{by}` of `{view}`"),
                                variants,
                                &collection.group_order,
                                false,
                            );
                        }
                    }
                }
                Composite::Metric(metric) => {
                    if let (Some(field), Some((view, fields)), Some(_)) = (
                        &metric.field,
                        self.row_fields(metric.reads.as_ref()),
                        metric.aggregate,
                    ) {
                        row_field(sink, &path.child("field"), &view, fields, field);
                    }
                }
                Composite::Form(form) => self.form_options(form, path, sink),
                _ => {}
            }
        }
    }

    /// A form field whose choice lists fixed options, over a command input that is an enum:
    /// the options are the enum's variants.
    fn form_options(&self, form: &ess_ui::Form, path: &NodePath, sink: &mut Sink) {
        let mut fields: Vec<(NodePath, &ess_ui::Field, &str)> = form
            .fields
            .iter()
            .map(|field| {
                (
                    path.child("fields").child(&field.name),
                    field,
                    form.does.as_str(),
                )
            })
            .collect();
        for group in &form.groups {
            let does = group.does.as_deref().unwrap_or(&form.does);
            let at = path.child("groups").child(&group.name).child("fields");
            fields.extend(
                group
                    .fields
                    .iter()
                    .map(|field| (at.child(&field.name), field, does)),
            );
        }
        for tab in &form.tabs {
            if let Some(ess_ui::TabFields::Fields(tab_fields)) = &tab.fields {
                let at = path.child("tabs").child(&tab.name).child("fields");
                fields.extend(
                    tab_fields
                        .iter()
                        .map(|field| (at.child(&field.name), field, form.does.as_str())),
                );
            }
        }
        for (at, field, does) in fields {
            let Some(Body::Composite(Composite::Choice(choice))) =
                field.choice.as_deref().map(|node| &node.body)
            else {
                continue;
            };
            if choice.options.is_empty() || field.binds.is_some() {
                continue;
            }
            let Some(Some(Some(variants))) = self
                .command(does)
                .and_then(|command| self.inputs.get(&command))
                .map(|inputs| inputs.get(field.field.as_str()))
            else {
                continue;
            };
            let written: Vec<String> = choice
                .options
                .iter()
                .map(|option| match &option.value {
                    serde_yaml::Value::String(text) => text.clone(),
                    other => serde_yaml::to_string(other)
                        .unwrap_or_default()
                        .trim()
                        .to_owned(),
                })
                .collect();
            enum_values(
                sink,
                &at.child("choice").child("options"),
                &format!("input `{}` of `{does}`", field.field),
                variants,
                &written,
                true,
            );
        }
    }

    fn readable(
        &self,
        sink: &mut Sink,
        section: &NodePath,
        qualified: &str,
        domain: &DomainHandle,
    ) {
        if !self.readable.contains(domain) {
            sink.push(
                "section_readable",
                section,
                format!(
                    "no actor can read `{qualified}`: no actor may invoke any command of \
                     `{domain}`, the bounded context that owns it. This is an approximation: ESS \
                     grants commands, not views, so a view counts as readable when some actor may \
                     invoke a command of its context"
                ),
            );
        }
    }
}

/// `true` when `name` is a row field of `view`; else reports it under `row_fields`.
fn row_field(sink: &mut Sink, at: &NodePath, view: &str, fields: &Fields, name: &str) -> bool {
    if fields.contains_key(name) {
        return true;
    }
    sink.push(
        "row_fields",
        at,
        format!(
            "`{name}` is no row field of `{view}`; it has {}",
            fields
                .keys()
                .map(|key| format!("`{key}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );
    false
}

/// Values a document writes for an enum of the model: each must be a variant, and when `every`
/// holds, every variant must be written.
fn enum_values(
    sink: &mut Sink,
    at: &NodePath,
    what: &str,
    variants: &[String],
    written: &[String],
    every: bool,
) {
    let unknown: Vec<&String> = written
        .iter()
        .filter(|value| !variants.contains(value))
        .collect();
    let missing: Vec<&String> = variants
        .iter()
        .filter(|variant| !written.contains(variant))
        .collect();
    let list = |values: &[&String]| {
        values
            .iter()
            .map(|value| format!("`{value}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    if !unknown.is_empty() {
        sink.push(
            "model_enum_values",
            at,
            format!(
                "{} {} no variant of the enum {what} ({})",
                list(&unknown),
                if unknown.len() == 1 { "is" } else { "are" },
                variants.join(", ")
            ),
        );
    }
    if every && !missing.is_empty() {
        sink.push(
            "model_enum_values",
            at,
            format!(
                "the enum {what} also has {}, which the options leave out",
                list(&missing)
            ),
        );
    }
}

/// No parameter at all: what a read with no `params:` slot binds.
static NONE: BTreeMap<String, Expr> = BTreeMap::new();

/// The parameters a view named at `node` is bound with, when the document states them: a read's
/// `params:` map; nothing, for a menu's `from_view` and an export without `params:`; and for an
/// export's `params: same_as(<section>)`, that section's read's map. `None` where the document
/// does not say — any other export expression, a channel's view, a confirm's references, a
/// completion — so no parameter is reported unbound there.
fn bound_params<'a>(
    document: &'a Document,
    node: &NodePath,
    bound: Bound<'a>,
) -> Option<&'a BTreeMap<String, Expr>> {
    match bound {
        Bound::Read(read) => Some(read.params),
        Bound::Nothing | Bound::Export(None) => Some(&NONE),
        Bound::Export(Some(expr)) => {
            let section = expr
                .0
                .trim()
                .strip_prefix("same_as(")?
                .strip_suffix(')')?
                .trim();
            let (_, page) = page_of(document, node)?;
            let section = page.sections.iter().find(|found| found.name == section)?;
            let Body::Composite(composite) = &section.body else {
                return None;
            };
            composite_reads(composite).map(|(_, reads)| &reads.params)
        }
        Bound::Elsewhere => None,
    }
}

/// Whether a read must bind `name`: it is not `Optional`, and it is neither of the two parameters
/// a `paging:` block names, which the published contract makes optional whatever their type.
fn required(view: &ResolvedView, name: &str, type_ref: &ResolvedTypeRef) -> bool {
    !matches!(type_ref, ResolvedTypeRef::Optional { .. })
        && !view
            .paging
            .as_ref()
            .is_some_and(|paging| paging.params().contains(&name))
}

/// `read_params`: a read binds exactly parameters its view declares, and every required one.
fn read_params(
    sink: &mut Sink,
    at: &NodePath,
    qualified: &str,
    view: &View,
    bound: &BTreeMap<String, Expr>,
) {
    let declared: Vec<&str> = view
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    for name in bound.keys() {
        if !declared.contains(&name.as_str()) {
            let listing = if declared.is_empty() {
                "declares no parameter".to_owned()
            } else {
                let names: Vec<String> = declared.iter().map(|name| format!("`{name}`")).collect();
                format!("declares {}", names.join(", "))
            };
            sink.push(
                "read_params",
                at,
                format!(
                    "`{name}` is not a parameter of view `{qualified}`, which {listing}: the \
                     served surface would never read it"
                ),
            );
        }
    }
    for param in view.params.iter().filter(|param| param.required) {
        if !bound.contains_key(&param.name) {
            sink.push(
                "read_params",
                at,
                format!(
                    "view `{qualified}` requires the parameter `{}` and this read binds none: \
                     the served surface cannot answer it",
                    param.name
                ),
            );
        }
    }
}

/// The section whose read a node at `path` is: the section itself, or — for a section rendered by
/// a widget — the section a node of the widget's expanded body sits in.
fn reading_section(document: &Document, path: &NodePath) -> Option<NodePath> {
    if is_section(path) {
        return Some(path.clone());
    }
    let [pages, page, sections, section, body, ..] = path.segments() else {
        return None;
    };
    let widget_rendered =
        matches!(section_of(document, path), Some(found) if matches!(found.body, Body::Widget(_)));
    (body == "body" && widget_rendered).then(|| {
        NodePath::root()
            .child(pages)
            .child(page)
            .child(sections)
            .child(section)
    })
}

// ── the names a document writes ──────────────────────────────────────────────────────────────

/// One view, command or event name the document writes, outside widget declarations.
struct Named<'a> {
    /// The node it is written on: a section, an overlay, a nested node, an action, a channel, …
    node: NodePath,
    /// Where a finding about it goes.
    at: NodePath,
    kind: Kind<'a>,
}

enum Kind<'a> {
    View {
        name: &'a str,
        /// What binds the view's parameters where the name is written.
        bound: Bound<'a>,
        /// `true` for the composite read of a body, which is what a section shows.
        body: bool,
    },
    Command(&'a str),
    Event(&'a str),
}

/// What binds a view's parameters where the document names the view.
#[derive(Clone, Copy)]
enum Bound<'a> {
    /// A `params:` map written beside the view, with the read's paging: a section's, a
    /// composite's, a preload's and an action's `loads`.
    Read(Read<'a>),
    /// No `params:` slot, so nothing: a dynamic menu's `from_view`.
    Nothing,
    /// An export's `params:` expression, when it has one.
    Export(Option<&'a Expr>),
    /// Something the document does not state may supply them — a channel's session, a confirm's
    /// context, a completion's input — so an unbound parameter is not reported there.
    Elsewhere,
}

impl<'a> Bound<'a> {
    fn paging(self) -> Option<&'a Paging> {
        match self {
            Self::Read(read) => read.paging,
            Self::Nothing | Self::Export(_) | Self::Elsewhere => None,
        }
    }
}

#[derive(Clone, Copy)]
struct Read<'a> {
    params: &'a BTreeMap<String, Expr>,
    paging: Option<&'a Paging>,
}

impl<'a> Read<'a> {
    fn of(reads: &'a Reads) -> Self {
        Self {
            params: &reads.params,
            paging: reads.paging.as_ref(),
        }
    }
}

/// Every view, command and event name `document` writes, with the node it is written on: the one
/// walk the model checks and the binding both read, so the two cannot disagree about which names
/// a document uses.
#[allow(clippy::too_many_lines)] // one arm per kind of node that names a view, command or event
fn names(document: &Document) -> Vec<Named<'_>> {
    let mut out = Vec::new();
    for located in document.nodes() {
        let path = &located.path;
        if in_declaration(path) {
            continue; // checked at each use, on the expanded body
        }
        let mut push = |at: NodePath, kind| {
            out.push(Named {
                node: path.clone(),
                at,
                kind,
            });
        };
        match located.node {
            NodeRef::Shell(shell) => {
                for view in shell.preload.iter().flat_map(|preload| &preload.views) {
                    let read = Read {
                        params: &view.params,
                        paging: None,
                    };
                    push(
                        path.child("preload"),
                        Kind::View {
                            name: &view.view,
                            bound: Bound::Read(read),
                            body: false,
                        },
                    );
                }
            }
            NodeRef::Region(region) => region_names(path, region, &mut push),
            NodeRef::NavSection(section) => {
                if let NavPages::Dynamic(entries) = &section.pages {
                    push(
                        path.child("pages"),
                        view(&entries.from_view, Bound::Nothing),
                    );
                }
            }
            NodeRef::Channel(channel) => channel_names(path, channel, &mut push),
            NodeRef::Section(section) => {
                if let Some(live) = &section.live {
                    for event in &live.on {
                        push(path.child("live"), Kind::Event(event));
                    }
                }
                body_names(path, &section.body, &mut push);
            }
            NodeRef::Overlay(overlay) => body_names(path, &overlay.body, &mut push),
            NodeRef::Node(node) => body_names(path, &node.body, &mut push),
            NodeRef::Action(action) => action_names(path, action, &mut push),
            NodeRef::FormGroup(group) => {
                if let Some(command) = &group.does {
                    push(path.clone(), Kind::Command(command));
                }
            }
            NodeRef::Field(field) => {
                if let Some(label_from) = &field.label_from {
                    // A renderer reads the related view once, without params.
                    push(
                        path.child("label_from"),
                        view(&label_from.view, Bound::Nothing),
                    );
                }
            }
            _ => {}
        }
    }
    out
}

/// The commands a shell region sends: the assistant's `does:`, and each account menu entry's
/// `does:`, named by the entry.
fn region_names<'a>(
    path: &NodePath,
    region: &'a Region,
    push: &mut impl FnMut(NodePath, Kind<'a>),
) {
    match region.kind {
        RegionKind::Assistant => {
            let commands: Vec<&str> = match region.props.get("does") {
                Some(Value::Sequence(items)) => items.iter().filter_map(Value::as_str).collect(),
                Some(Value::String(one)) => vec![one.as_str()],
                _ => Vec::new(),
            };
            for command in commands {
                push(path.child("props").child("does"), Kind::Command(command));
            }
        }
        RegionKind::AccountMenu => {
            let Some(Value::Sequence(entries)) = region.props.get("actions") else {
                return;
            };
            for entry in entries.iter().filter_map(Value::as_mapping) {
                let Some(command) = entry.get("does").and_then(Value::as_str) else {
                    continue;
                };
                let actions = path.child("props").child("actions");
                let at = match entry.get("name").and_then(Value::as_str) {
                    Some(name) => actions.child(name),
                    None => actions,
                };
                push(at, Kind::Command(command));
            }
        }
        _ => {}
    }
}

/// What a channel carries, and the commands it sends.
fn channel_names<'a>(
    path: &NodePath,
    channel: &'a Channel,
    push: &mut impl FnMut(NodePath, Kind<'a>),
) {
    let carries = path.child("carries");
    match &channel.carries {
        Carries::Events(events) => {
            for event in &events.events {
                push(carries.clone(), Kind::Event(event));
            }
        }
        Carries::View(live_view) => {
            push(carries, view(&live_view.view, Bound::Elsewhere));
        }
    }
    for command in &channel.sends {
        push(path.child("sends"), Kind::Command(command));
    }
}

/// What an action sends, uploads, exports and loads.
fn action_names<'a>(
    path: &NodePath,
    action: &'a Action,
    push: &mut impl FnMut(NodePath, Kind<'a>),
) {
    if let Some(command) = &action.does {
        push(path.clone(), Kind::Command(command));
    }
    if let Some(upload) = &action.upload {
        push(path.clone(), Kind::Command(&upload.does));
    }
    if let Some(export) = &action.export {
        push(
            path.clone(),
            view(&export.reads, Bound::Export(export.params.as_ref())),
        );
    }
    if let Some(reads) = &action.loads {
        if let Some(name) = &reads.view {
            push(
                path.child("loads"),
                Kind::View {
                    name,
                    bound: Bound::Read(Read::of(reads)),
                    body: false,
                },
            );
        }
    }
}

/// A view name written outside a body's composite read, its parameters bound as `bound` says.
fn view<'a>(name: &'a str, bound: Bound<'a>) -> Kind<'a> {
    Kind::View {
        name,
        bound,
        body: false,
    }
}

fn body_names<'a>(path: &NodePath, body: &'a Body, push: &mut impl FnMut(NodePath, Kind<'a>)) {
    let Body::Composite(composite) = body else {
        return;
    };
    if let Some((key, reads)) = composite_reads(composite) {
        if let Some(name) = &reads.view {
            push(
                path.child(key),
                Kind::View {
                    name,
                    bound: Bound::Read(Read::of(reads)),
                    body: true,
                },
            );
        }
    }
    if let Composite::GraphEditor(editor) = composite {
        if let Some(reads) = editor.edges.as_ref().and_then(|e| e.reads.as_ref()) {
            if let Some(name) = &reads.view {
                push(
                    path.child("edges").child("reads"),
                    Kind::View {
                        name,
                        bound: Bound::Read(Read::of(reads)),
                        body: false,
                    },
                );
            }
        }
    }
    let commands: Vec<&String> = match composite {
        Composite::Form(form) => vec![&form.does],
        Composite::Confirm(confirm) => confirm.does.iter().collect(),
        Composite::Collection(collection) => collection
            .reorder
            .iter()
            .map(|reorder| &reorder.does)
            .collect(),
        Composite::Choice(choice) => choice.creatable.iter().map(|c| &c.does).collect(),
        Composite::Board(board) => board
            .layout
            .iter()
            .flat_map(|layout| std::iter::once(&layout.persisted_by).chain(&layout.editable_by))
            .collect(),
        _ => Vec::new(),
    };
    for command in commands {
        push(path.clone(), Kind::Command(command));
    }
    let views: Vec<&String> = match composite {
        Composite::Confirm(confirm) => confirm.references.iter().collect(),
        Composite::RichText(text) => text.completes.iter().collect(),
        _ => Vec::new(),
    };
    for name in views {
        push(path.clone(), view(name, Bound::Elsewhere));
    }
}

// ── the binding ──────────────────────────────────────────────────────────────────────────────

/// One reason a document cannot bind to the served surface, at the node that causes it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Refusal {
    /// The canonical path of the node.
    pub path: String,
    /// Why it cannot bind.
    pub message: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

/// Why [`binding`] produced no binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingError {
    /// The model does not compile.
    Model(CheckError),
    /// The document names something the served surface cannot answer as written; every refusal,
    /// ordered by path.
    Refused(Vec<Refusal>),
}

impl fmt::Display for BindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(error) => write!(formatter, "{error}"),
            Self::Refused(refusals) => {
                formatter.write_str("the document does not bind to the served surface:")?;
                for refusal in refusals {
                    write!(formatter, "\n{refusal}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for BindingError {}

/// The routes `document` binds to on the HTTP surface the model `sources` determine, computed by
/// `ess_gen::http::routes` and covering only the views and commands the document names.
///
/// Refused, each at the node that causes it: a name the model does not declare or no
/// `reached_by: network` component serves; a view parameter that is not a scalar, which no query
/// string carries; a view the model declares with `paging:`, and a read paged by anything but
/// the renderer (`paging:` `server`, `cursor` or `append`), since every code target refuses
/// paging; and state placed in `server` or
/// `server_session`, which the served surface does not hold.
pub fn binding(document: &Document, sources: &[(String, String)]) -> Result<Binding, BindingError> {
    let ir = compile_sources(sources, Path::new(&document.model)).map_err(BindingError::Model)?;
    let model = Model::index(&ir);
    let served = Surface::of(&ir);
    let mut binding = Binding {
        system: model.system.clone(),
        components: BTreeMap::new(),
        names: BTreeMap::new(),
    };
    let mut refusals = Vec::new();
    let mut refuse = |at: &NodePath, message: String| {
        refusals.push(Refusal {
            path: at.to_string(),
            message,
        });
    };
    for named in names(document) {
        match named.kind {
            Kind::Event(_) => {} // no server stream: a channel polls or refuses
            Kind::Command(name) => {
                let Some(qualified) = model.command(name) else {
                    refuse(
                        &named.at,
                        format!("`{name}` names no command of model `{}`", model.system),
                    );
                    continue;
                };
                let Some((component, route)) = served.commands.get(&qualified) else {
                    refuse(&named.at, unserved("command", &qualified));
                    continue;
                };
                binding.names.insert(name.to_owned(), qualified.clone());
                binding
                    .components
                    .entry(component.clone())
                    .or_default()
                    .commands
                    .insert(qualified, route.clone());
            }
            Kind::View { name, bound, .. } => {
                let Some((qualified, _)) = model.view(name) else {
                    refuse(
                        &named.at,
                        format!("`{name}` names no view of model `{}`", model.system),
                    );
                    continue;
                };
                let Some((component, route)) = served.views.get(&qualified) else {
                    refuse(&named.at, unserved("view", &qualified));
                    continue;
                };
                let unreadable = read_refusals(&served, &qualified, bound);
                if unreadable.is_empty() {
                    binding.names.insert(name.to_owned(), qualified.clone());
                    binding
                        .components
                        .entry(component.clone())
                        .or_default()
                        .views
                        .insert(qualified, route.clone());
                }
                for message in unreadable {
                    refuse(&named.at, message);
                }
            }
        }
    }
    refusals.extend(state_refusals(document));
    if refusals.is_empty() {
        Ok(binding)
    } else {
        refusals.sort();
        refusals.dedup();
        Err(BindingError::Refused(refusals))
    }
}

/// Why a served view cannot be read as `read` reads it: a view the model pages, a read paged by
/// anything but the renderer, and each declared parameter no query string carries.
fn read_refusals(served: &Surface, qualified: &str, bound: Bound<'_>) -> Vec<String> {
    let mut refusals = Vec::new();
    if served.paged.contains(qualified) {
        refusals.push(format!(
            "view `{qualified}` declares `paging:` in the model, and every code target refuses a \
             paged view (`ess-synth/src/paging.rs`): no served surface answers it"
        ));
    }
    if let Some(paged) = bound.paging().and_then(server_paged) {
        refusals.push(format!(
            "this read of `{qualified}` is paged by `{paged}`, and no served surface pages a \
             view: every code target refuses paging. Page it in the renderer (`paging: client`) \
             or not at all"
        ));
    }
    for param in &served.non_scalar[qualified] {
        refusals.push(format!(
            "view `{qualified}` declares the parameter `{param}` at a type that is not a scalar, \
             and a query string carries only scalars"
        ));
    }
    refusals
}

/// Every state entry, outside widget declarations, placed in `server` or `server_session`.
fn state_refusals(document: &Document) -> Vec<Refusal> {
    let mut refusals = Vec::new();
    for located in document.nodes() {
        let NodeRef::State(state) = located.node else {
            continue;
        };
        if in_declaration(&located.path) {
            continue;
        }
        for store in crate::rules::placements(document, &located.path, state) {
            if store == "server" || store == "server_session" {
                refusals.push(Refusal {
                    path: located.path.to_string(),
                    message: format!(
                        "this state is placed in `{store}`, and a document bound to the served \
                         surface keeps its state in the client: the surface holds no session \
                         and no store for UI state"
                    ),
                });
            }
        }
    }
    refusals
}

fn unserved(what: &str, qualified: &str) -> String {
    format!(
        "{what} `{qualified}` is served by no `reached_by: network` component: nothing outside \
         its process can reach it over HTTP"
    )
}

/// The `paging:` value of a read that something other than the renderer pages.
fn server_paged(paging: &Paging) -> Option<&'static str> {
    match paging {
        Paging::Server => Some("server"),
        Paging::Cursor => Some("cursor"),
        Paging::Append => Some("append"),
        Paging::Unmapped(_) => Some("UNMAPPED"),
        Paging::Client | Paging::None => None,
    }
}

/// Every route the model's `reached_by: network` components answer, by qualified name, from
/// `ess_gen::http::routes`.
struct Surface {
    views: BTreeMap<String, (String, ViewRoute)>,
    commands: BTreeMap<String, (String, CommandRoute)>,
    /// Every served view's parameters that are not scalars, by the view's qualified name.
    non_scalar: BTreeMap<String, Vec<String>>,
    /// Every served view whose model declaration carries `paging:`, which no code target serves.
    paged: BTreeSet<String>,
}

impl Surface {
    fn of(ir: &EssIr) -> Self {
        let mut surface = Self {
            views: BTreeMap::new(),
            commands: BTreeMap::new(),
            non_scalar: BTreeMap::new(),
            paged: BTreeSet::new(),
        };
        for component in ir.components().values() {
            if component.reached_by != Reach::Network {
                continue;
            }
            let name = component.name.to_string();
            for route in http::routes(ir, component) {
                match route.serves {
                    Served::Command(handle) => {
                        let command = ir.command(handle);
                        let mut errors = BTreeMap::new();
                        let declared = command.outcomes.iter().chain(
                            ess_gen::unknown_instance::unknown_instance_answer(ir, command),
                        );
                        for outcome in declared {
                            let Some(error) = &outcome.error else {
                                continue;
                            };
                            let error = ir.error(error);
                            errors
                                .entry(error.wire_code())
                                .or_insert_with(|| ErrorRoute {
                                    status: http::status(outcome)
                                        .parse()
                                        .expect("every status the contract declares is a number"),
                                    display: error.naming.display_or(&error.name).to_owned(),
                                });
                        }
                        let route = CommandRoute {
                            path: route.path,
                            body_required: http::body_required(command),
                            errors,
                        };
                        surface
                            .commands
                            .insert(handle.to_string(), (name.clone(), route));
                    }
                    Served::View(handle) => {
                        let view = ir.view(handle);
                        let mut params = Vec::new();
                        let mut non_scalar = Vec::new();
                        for param in &view.params {
                            match scalar(ir, &param.type_ref) {
                                Some(scalar) => params.push(QueryParam {
                                    name: param.name.clone(),
                                    wire: param
                                        .naming
                                        .wire
                                        .clone()
                                        .unwrap_or_else(|| param.name.clone()),
                                    required: required(view, &param.name, &param.type_ref),
                                    scalar: scalar.to_owned(),
                                }),
                                None => non_scalar.push(param.name.clone()),
                            }
                        }
                        let route = ViewRoute {
                            path: route.path,
                            params,
                        };
                        surface.non_scalar.insert(handle.to_string(), non_scalar);
                        if view.paging.is_some() {
                            surface.paged.insert(handle.to_string());
                        }
                        surface
                            .views
                            .insert(handle.to_string(), (name.clone(), route));
                    }
                }
            }
        }
        surface
    }
}

/// Fields by name and by wire name, each with the wire variants of its enum, if it is one.
fn fields_of(ir: &EssIr, fields: &[ess_compiler::ir::ResolvedField]) -> Fields {
    let mut out = BTreeMap::new();
    for field in fields {
        let variants = enum_variants(ir, &field.type_ref);
        if let Some(wire) = &field.naming.wire {
            out.insert(wire.clone(), variants.clone());
        }
        out.insert(field.name.clone(), variants);
    }
    out
}

/// The wire variants of the enum `type_ref` holds, through `Optional`, a list and newtypes.
fn enum_variants(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<Vec<String>> {
    match type_ref {
        ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => enum_variants(ir, of),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Enum { variants } => Some(
                variants
                    .iter()
                    .map(|variant| variant.wire().to_owned())
                    .collect(),
            ),
            ResolvedBody::Newtype { of, .. } => enum_variants(ir, of),
            ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => None,
        },
        ResolvedTypeRef::Primitive { .. } | ResolvedTypeRef::Map { .. } => None,
    }
}

/// The primitive a value of `type_ref` is written as in a query string, through `Optional` and
/// newtypes; `String` for an enum. `None` for a list, a map, a struct, a union and `Json`, which
/// no single query value carries.
fn scalar(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<&'static str> {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => (*name != Primitive::Json).then(|| name.as_str()),
        ResolvedTypeRef::Optional { of } => scalar(ir, of),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => scalar(ir, of),
            ResolvedBody::Enum { .. } => Some(Primitive::String.as_str()),
            ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => None,
        },
        ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => None,
    }
}
