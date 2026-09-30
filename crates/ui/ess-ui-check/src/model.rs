//! The checks that need the ESS model: every view, command and event the document names exists
//! in it, and every section's read is readable by some actor.
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
use std::path::{Path, PathBuf};

use ess_compiler::ir::DomainHandle;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_ui::{ActorSource, Body, Carries, Composite, Document, NavPages, NodePath, NodeRef};

use crate::walk::{composite_reads, in_declaration, is_section, section_of};
use crate::{CheckError, Sink};

/// A compiled ESS model, indexed by the names a document can use.
///
/// Readability is an approximation: ESS grants commands, not views, so a view counts as readable
/// when some actor may invoke a command of the bounded context that owns it. A document with
/// `actor: anonymous` is not held to it, since no grant decides what an anonymous reader sees.
#[derive(Debug)]
pub struct Model {
    system: String,
    views: BTreeMap<String, DomainHandle>,
    commands: BTreeSet<String>,
    events: BTreeSet<String>,
    readable: BTreeSet<DomainHandle>,
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
    let ir = ess_compiler::compile(&specification, &sources)
        .map_err(|diagnostics| refused(vec![diagnostics.to_string()]))?;
    Ok(Model::index(&ir))
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
                .map(|(name, view)| (name.to_string(), view.domain.clone()))
                .collect(),
            commands: ir.commands().keys().map(ToString::to_string).collect(),
            events: ir.events().keys().map(ToString::to_string).collect(),
            readable,
        }
    }

    /// The model's qualified name for `name`, when it has one.
    fn qualify(&self, name: &str, known: impl Fn(&str) -> bool) -> Option<String> {
        let prefixed = format!("{}.{name}", self.system);
        [name.to_owned(), prefixed]
            .into_iter()
            .find(|candidate| known(candidate))
    }

    fn view(&self, name: &str) -> Option<(String, &DomainHandle)> {
        let qualified = self.qualify(name, |candidate| self.views.contains_key(candidate))?;
        let domain = &self.views[&qualified];
        Some((qualified, domain))
    }

    fn has_command(&self, name: &str) -> bool {
        self.qualify(name, |candidate| self.commands.contains(candidate))
            .is_some()
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
        if !self.has_command(name) {
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
        for located in document.nodes() {
            let path = &located.path;
            if in_declaration(path) {
                continue; // checked at each use, on the expanded body
            }
            let reader = grants_apply
                .then(|| reading_section(document, path))
                .flatten();
            let reader = reader.as_ref();
            match located.node {
                NodeRef::Shell(shell) => {
                    for view in shell.preload.iter().flat_map(|preload| &preload.views) {
                        self.view_ref(sink, &path.child("preload"), &view.view);
                    }
                }
                NodeRef::NavSection(section) => {
                    if let NavPages::Dynamic(entries) = &section.pages {
                        self.view_ref(sink, &path.child("pages"), &entries.from_view);
                    }
                }
                NodeRef::Channel(channel) => {
                    let carries = path.child("carries");
                    match &channel.carries {
                        Carries::Events(events) => {
                            for event in &events.events {
                                self.event_ref(sink, &carries, event);
                            }
                        }
                        Carries::View(view) => self.view_ref(sink, &carries, &view.view),
                    }
                    for command in &channel.sends {
                        self.command_ref(sink, &path.child("sends"), command);
                    }
                }
                NodeRef::Section(section) => {
                    if let Some(live) = &section.live {
                        for event in &live.on {
                            self.event_ref(sink, &path.child("live"), event);
                        }
                    }
                    self.body(sink, path, reader, &section.body);
                }
                NodeRef::Overlay(overlay) => self.body(sink, path, reader, &overlay.body),
                NodeRef::Node(node) => self.body(sink, path, reader, &node.body),
                NodeRef::Action(action) => {
                    if let Some(command) = &action.does {
                        self.command_ref(sink, path, command);
                    }
                    if let Some(upload) = &action.upload {
                        self.command_ref(sink, path, &upload.does);
                    }
                    if let Some(export) = &action.export {
                        self.view_ref(sink, path, &export.reads);
                    }
                    if let Some(view) = action.loads.as_ref().and_then(|reads| reads.view.as_ref())
                    {
                        self.view_ref(sink, &path.child("loads"), view);
                    }
                }
                NodeRef::FormGroup(group) => {
                    if let Some(command) = &group.does {
                        self.command_ref(sink, path, command);
                    }
                }
                _ => {}
            }
        }
    }

    fn body(&self, sink: &mut Sink, path: &NodePath, reader: Option<&NodePath>, body: &Body) {
        let Body::Composite(composite) = body else {
            return;
        };
        if let Some((key, reads)) = composite_reads(composite) {
            if let Some(view) = &reads.view {
                match self.view(view) {
                    None => self.view_ref(sink, &path.child(key), view),
                    Some((qualified, domain)) => {
                        if let Some(section) = reader {
                            if !self.readable.contains(domain) {
                                sink.push(
                                    "section_readable",
                                    section,
                                    format!(
                                        "no actor can read `{qualified}`: no actor may invoke any \
                                         command of `{domain}`, the bounded context that owns \
                                         it. This is an approximation: ESS grants commands, not \
                                         views, so a view counts as readable when some actor \
                                         may invoke a command of its context"
                                    ),
                                );
                            }
                        }
                    }
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
            self.command_ref(sink, path, command);
        }
        let views: Vec<&String> = match composite {
            Composite::Confirm(confirm) => confirm.references.iter().collect(),
            Composite::RichText(text) => text.completes.iter().collect(),
            _ => Vec::new(),
        };
        for view in views {
            self.view_ref(sink, path, view);
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
