//! Every check an `ess-ui/1` document is held to, reported by canonical node path.
//!
//! The checks are the schema's `checks.list` (`schemas/ui/ess-ui.schema.yaml`), the rules the
//! schema states elsewhere that the loader does not refuse (placement refusals, the type rule, a
//! widget containing itself, the `degrades` capabilities, a primitive's `exactly_one_of`, the
//! values of a closed enum, an overlay's params read inside it), and — given an ESS model — that
//! every view, command, event and page parameter type the document names exists in it, that form
//! fields, `bind` keys, columns and `row.<field>` paths name inputs and fields that exist, and
//! that every section's read is readable by some actor — an approximation, since ESS grants
//! commands and not views (documented on [`Model`]) — and that a page naming its `actor` sends
//! only commands that actor is granted. [`CHECKS`] lists them all. A host slot's `reads` and
//! `commands` are held to every one of the model checks a generated node's are.
//!
//! The report lists every host slot ([`HostEntry`]) and, given a model, its [`Coverage`]: the
//! model's views and commands a generated node reaches, those only a host slot reaches, and the
//! rest. A host slot never adds to the covered count.
//!
//! A document the loader refuses yields exactly one finding, the refusal, classified under the
//! check it breaks. A document that loads is checked in full, and every finding names its node by
//! the [`ess_ui::NodePath`] the loader gives it, so a finding keeps its path when a sibling is
//! added or removed.
//!
//! [`check`] is the entry point; [`run`] wraps it for the `ess ui check` command, rendering human
//! text or JSON (`ess-ui-check/1`) and exiting 1 when any finding is an error.

mod classify;
mod enums;
mod expr;
mod filter;
mod hosts;
mod model;
mod names;
mod raw;
mod rules;
mod schema;
mod style;
mod walk;

use std::fmt::{self, Write as _};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub use model::{
    binding, binding_over, compile_sources, load_model, model_from_sources, BindingError,
    HttpBinding, Model, Refusal,
};

/// The format marker of the JSON report.
pub const FORMAT: &str = "ess-ui-check/1";

/// How bad a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The document breaks a rule; `ess ui check` exits 1.
    Error,
    /// Reported, and the check still passes: a placeholder, a gap a retrofit left, a view no
    /// fixture answers.
    Warning,
}

impl Severity {
    /// `error` or `warning`, as the schema and the JSON report spell it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One finding: the node, the check it breaks, how badly, and what is wrong.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Finding {
    /// The canonical path of the failing node (`/` for the document itself).
    pub path: String,
    /// The id of the check, one of [`CHECKS`].
    pub check: String,
    /// The check's severity.
    pub severity: Severity,
    /// What is wrong.
    pub message: String,
}

/// A check this crate runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Check {
    /// Its id, as findings carry it.
    pub id: &'static str,
    /// The severity of every finding it reports.
    pub severity: Severity,
    /// `true` when the schema's `checks.list` declares it (with this id and severity).
    pub from_schema: bool,
}

const fn schema_check(id: &'static str, severity: Severity) -> Check {
    Check {
        id,
        severity,
        from_schema: true,
    }
}

const fn rule(id: &'static str) -> Check {
    Check {
        id,
        severity: Severity::Error,
        from_schema: false,
    }
}

/// Every check, in the schema's order, then the rules the schema states elsewhere, then the
/// model checks.
pub const CHECKS: &[Check] = &[
    schema_check("names_unique", Severity::Error),
    schema_check("nav_resolves", Severity::Error),
    schema_check("nav_unique", Severity::Error),
    schema_check("page_reachable", Severity::Error),
    schema_check("opens_resolves", Severity::Error),
    schema_check("same_as_resolves", Severity::Error),
    schema_check("page_refs", Severity::Error),
    schema_check("shell_refs", Severity::Error),
    schema_check("page_outlet", Severity::Error),
    schema_check("channel_refs", Severity::Error),
    schema_check("section_refs", Severity::Error),
    schema_check("layout_complete", Severity::Warning),
    schema_check("widget_expands", Severity::Error),
    schema_check("primitive_props", Severity::Error),
    schema_check("unbound_placeholder", Severity::Warning),
    schema_check("fixture_per_view", Severity::Warning),
    schema_check("script_per_channel", Severity::Warning),
    schema_check("state_resolves", Severity::Error),
    schema_check("types_structural", Severity::Error),
    schema_check("unmapped_reported", Severity::Warning),
    schema_check("token_values", Severity::Error),
    schema_check("token_names", Severity::Error),
    schema_check("token_refs", Severity::Error),
    schema_check("theme_tokens", Severity::Error),
    schema_check("theme_choice", Severity::Error),
    schema_check("tone_map_refs", Severity::Error),
    schema_check("tone_map_unused", Severity::Warning),
    schema_check("tone_bands", Severity::Error),
    schema_check("label_map_refs", Severity::Error),
    schema_check("label_map_unused", Severity::Warning),
    schema_check("filter_expr", Severity::Error),
    schema_check("filter_roots", Severity::Error),
    schema_check("filter_paths", Severity::Error),
    schema_check("filter_place", Severity::Error),
    schema_check("filter_paging", Severity::Error),
    schema_check("filter_over_param", Severity::Warning),
    schema_check("filter_export", Severity::Warning),
    schema_check("pager_paging", Severity::Error),
    schema_check("pager_params", Severity::Error),
    schema_check("pager_sizes", Severity::Error),
    schema_check("pager_outside_collection", Severity::Error),
    schema_check("account_menu_keys", Severity::Error),
    schema_check("checked_needs_toggle", Severity::Error),
    schema_check("suggests_needs_tags", Severity::Error),
    schema_check("toggle_state_unknown", Severity::Warning),
    schema_check("checked_unreadable", Severity::Warning),
    schema_check("host_slots", Severity::Error),
    schema_check("hosts_registered", Severity::Error),
    schema_check("board_widget_kinds", Severity::Warning),
    // The loader refused the document for a reason no other check names.
    rule("document_loads"),
    // `layers`: a node stands where its layer may not.
    rule("layer_rules"),
    // `Degrades.fields.(capability)`: a key names a capability, its value one of its fallbacks.
    rule("degrades_known"),
    // `Degrades.rule`: a renderer lacking a capability finds a fallback that is not `refuse`.
    rule("degrades_cover"),
    // A value written where the schema declares a closed enum is one of its values.
    rule("enum_values"),
    // `PageLayout`: `title` and `collapsed` belong to `frame: shared`, a collapsed frame or
    // section has a `title`, a collapsed group has a `label`, and a shared frame holds
    // consecutive sections.
    rule("frame_rules"),
    // Every key of an overlay's `params` is read inside it as `params.<key>`.
    rule("overlay_params"),
    // With `--model`: the names the document resolves in the ESS model; a view or command no
    // `reached_by: network` component serves is refused under the first two.
    rule("view_in_model"),
    rule("command_in_model"),
    rule("event_in_model"),
    rule("section_readable"),
    // With `--model`: a page's `actor` names an actor of the model (beyond10x/ess#284).
    rule("actor_in_model"),
    // With `--model`: every command a page with an `actor` sends is in that actor's `may`
    // (beyond10x/ess#284).
    rule("page_actor_grants"),
    // With `--model`: a read binds a parameter its view does not declare, or leaves a required
    // one unbound.
    rule("read_params"),
    // With `--model`: a page parameter's named type resolves in the document, the schema or
    // the model.
    rule("type_in_model"),
    // With `--model`: form fields and `bind` keys are inputs of their command; columns, record
    // fields and `row.<field>` paths are fields of the view read.
    rule("field_in_model"),
    // With `--model`: a `group_by`, an aggregate's `field`, a `label_from`, or a choice's `value`
    // or `label` names a row field the view does not have.
    rule("row_fields"),
    // With `--model`: a `group_order` value, or a form choice's fixed option, is no variant of the
    // enum the field holds, or the options leave a variant out.
    rule("model_enum_values"),
    // `metric`: `aggregate` reads rows, so it needs `reads`; every aggregate but `count` needs a
    // `field`, and `field` means nothing without one.
    rule("metric_aggregate"),
    // `collection`: `group_order` and `show_empty_groups` order the groups of `group_by`.
    rule("group_order"),
    // A choice's `options` name an enum of the document, or with `--model` exactly one enum of
    // the model (beyond10x/ess#330).
    rule("options_enum"),
    // `choice`: `value` and `label` name fields of the rows of `reads`, so they need `reads`
    // (beyond10x/ess#328).
    rule("choice_projection"),
    // `header.title_from` names a section that reads: the title's record is its first row
    // (beyond10x/ess#354).
    rule("header_record"),
    // An export's params read `selection.<field>` only on a bulk action, where the selection is
    // in scope (beyond10x/ess#723).
    rule("export_selection"),
];

fn severity_of(id: &str) -> Severity {
    CHECKS
        .iter()
        .find(|check| check.id == id)
        .unwrap_or_else(|| panic!("`{id}` is not in CHECKS"))
        .severity
}

/// Findings as they are collected.
#[derive(Default)]
pub(crate) struct Sink {
    findings: Vec<Finding>,
    backlog: Vec<BacklogEntry>,
    hosts: Vec<HostEntry>,
    coverage: Option<Coverage>,
}

impl Sink {
    /// Records a finding of check `id` at `path`.
    pub(crate) fn push(
        &mut self,
        id: &'static str,
        path: &ess_ui::NodePath,
        message: impl Into<String>,
    ) {
        self.findings.push(Finding {
            path: path.to_string(),
            check: id.to_owned(),
            severity: severity_of(id),
            message: message.into(),
        });
    }
}

/// What a check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The document checked, as it was named.
    pub document: String,
    /// Every finding, ordered by path, then check, then message.
    pub findings: Vec<Finding>,
    /// The generator's backlog: every host slot that stands in for `none`, ordered by path.
    pub backlog: Vec<BacklogEntry>,
    /// Every host slot of the document, ordered by path.
    pub hosts: Vec<HostEntry>,
    /// With a model, how many of its views and commands the document reaches through generated
    /// nodes, through host slots only, and not at all. `None` without a model, whose views and
    /// commands are the rows counted.
    pub coverage: Option<Coverage>,
}

/// A host slot of the document: a place the host application draws with its own component.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct HostEntry {
    /// The slot's canonical path.
    pub path: String,
    /// The component name the host registers.
    pub host: String,
    /// Why the generator cannot draw it.
    pub reason: String,
    /// The generator item the slot replaces: a composite, a primitive, or `none`.
    pub stands_in_for: String,
}

/// The views and commands of the model — the rows — split three ways by what reaches them.
///
/// A row is **covered** when a generated node of the document reads or sends it, **host-covered**
/// when only a host slot's `reads` or `commands` reach it, and **uncovered** when nothing in the
/// document does. A row a generated node and a slot both reach is covered, so a slot never adds to
/// the covered count: the document with its slots removed has the same one. The three add up to
/// the model's views and commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub struct Coverage {
    /// Rows a generated node reaches.
    pub covered: usize,
    /// Rows only a host slot reaches.
    pub host_covered: usize,
    /// Rows nothing in the document reaches.
    pub uncovered: usize,
}

impl Coverage {
    /// Every row: the model's views and commands.
    pub fn rows(&self) -> usize {
        self.covered + self.host_covered + self.uncovered
    }
}

/// A host slot no generator item fits (`stands_in_for: none`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct BacklogEntry {
    /// The slot's canonical path.
    pub path: String,
    /// The component name the host registers.
    pub host: String,
    /// Why the generator cannot draw it.
    pub reason: String,
}

#[derive(serde::Serialize)]
struct Json<'a> {
    format: &'static str,
    document: &'a str,
    findings: &'a [Finding],
    #[serde(skip_serializing_if = "<[BacklogEntry]>::is_empty")]
    backlog: &'a [BacklogEntry],
    #[serde(skip_serializing_if = "<[HostEntry]>::is_empty")]
    hosts: &'a [HostEntry],
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage: Option<Coverage>,
}

impl Report {
    fn new(document: &str, sink: Sink) -> Self {
        let mut findings = sink.findings;
        findings.sort();
        findings.dedup();
        let mut backlog = sink.backlog;
        backlog.sort();
        let mut hosts = sink.hosts;
        hosts.sort();
        Self {
            document: document.to_owned(),
            findings,
            backlog,
            hosts,
            coverage: sink.coverage,
        }
    }

    /// How many findings are errors.
    pub fn errors(&self) -> usize {
        self.count(Severity::Error)
    }

    /// How many findings are warnings.
    pub fn warnings(&self) -> usize {
        self.count(Severity::Warning)
    }

    fn count(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == severity)
            .count()
    }

    /// `true` when any finding is an error: the document fails the check.
    pub fn has_errors(&self) -> bool {
        self.errors() > 0
    }

    /// The report as human text or as an `ess-ui-check/1` JSON document, ending in a newline.
    pub fn render(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Text => {
                let mut text = String::new();
                for finding in &self.findings {
                    let _ = writeln!(
                        text,
                        "{} {} {}: {}",
                        finding.severity, finding.check, finding.path, finding.message
                    );
                }
                for entry in &self.backlog {
                    let _ = writeln!(
                        text,
                        "backlog {} {}: {}",
                        entry.path, entry.host, entry.reason
                    );
                }
                for entry in &self.hosts {
                    let _ = writeln!(
                        text,
                        "host {} {} stands in for {}: {}",
                        entry.path, entry.host, entry.stands_in_for, entry.reason
                    );
                }
                if let Some(coverage) = &self.coverage {
                    let _ = writeln!(
                        text,
                        "coverage: {} covered, {} host-covered, {} uncovered of {} views and \
                         commands",
                        coverage.covered,
                        coverage.host_covered,
                        coverage.uncovered,
                        coverage.rows()
                    );
                }
                let _ = writeln!(
                    text,
                    "{}: {} error(s), {} warning(s)",
                    self.document,
                    self.errors(),
                    self.warnings()
                );
                text
            }
            OutputFormat::Json => {
                let json = Json {
                    format: FORMAT,
                    document: &self.document,
                    findings: &self.findings,
                    backlog: &self.backlog,
                    hosts: &self.hosts,
                    coverage: self.coverage,
                };
                let mut text =
                    serde_json::to_string_pretty(&json).expect("a report serializes to JSON");
                text.push('\n');
                text
            }
        }
    }
}

/// What a check is run with beyond the document and the model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// Capabilities the target renderer lacks (`Degrades.capabilities`, such as
    /// `no_file_upload`): every construct using one must degrade to something other than
    /// `refuse`.
    pub lacks: Vec<String>,
    /// The host component names the host registers (`--hosts`): each must be declared by a host
    /// slot of the document. `None` checks no registration.
    pub hosts: Option<Vec<String>>,
}

/// A check that could not run: an unreadable document, or a model that does not compile. A
/// document that does not load is not this; it is a finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckError(pub(crate) String);

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CheckError {}

/// How `ess ui check` renders its report.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// One line per finding, then a count.
    #[default]
    Text,
    /// An `ess-ui-check/1` document.
    Json,
}

/// The arguments of `ess ui check`.
#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct CheckArgs {
    /// The `ess-ui/1` document to check.
    #[arg(long)]
    pub path: PathBuf,
    /// An ESS specification (one file, or a directory with `system.yaml`) the document's views,
    /// commands and events must exist in.
    #[arg(long)]
    pub model: Option<PathBuf>,
    /// Output rendering.
    #[arg(long, value_enum, default_value_t)]
    pub format: OutputFormat,
    /// A capability the target renderer lacks (repeatable), such as `no_file_upload`.
    #[arg(long = "lacks", value_name = "CAPABILITY")]
    pub lacks: Vec<String>,
    /// A file of the host component names the host application registers, one per line (`#`
    /// starts a comment); a name no host slot of the document declares is refused.
    #[arg(long, value_name = "FILE")]
    pub hosts: Option<PathBuf>,
}

/// The names a `--hosts` file lists: one per line, trimmed, a leading byte-order mark dropped, blank
/// lines and `#` comments skipped.
fn registered_hosts(file: &Path) -> Result<Vec<String>, CheckError> {
    let text = std::fs::read_to_string(file)
        .map_err(|error| CheckError(format!("cannot read {}: {error}", file.display())))?;
    Ok(text
        .strip_prefix('\u{feff}')
        .unwrap_or(&text)
        .lines()
        .map(|line| line.split_once('#').map_or(line, |(name, _)| name).trim())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect())
}

/// The [`Options`] `args` asks for.
fn options(args: &CheckArgs) -> Result<Options, CheckError> {
    Ok(Options {
        lacks: args.lacks.clone(),
        hosts: args.hosts.as_deref().map(registered_hosts).transpose()?,
    })
}

/// Checks a document file, optionally against an ESS specification.
pub fn check(document: &Path, model: Option<&Path>) -> Result<Report, CheckError> {
    check_with(document, model, &Options::default())
}

/// [`check`] with [`Options`].
pub fn check_with(
    document: &Path,
    model: Option<&Path>,
    options: &Options,
) -> Result<Report, CheckError> {
    let text = std::fs::read_to_string(document)
        .map_err(|error| CheckError(format!("cannot read {}: {error}", document.display())))?;
    let model = model.map(load_model).transpose()?;
    let base = document.parent().unwrap_or_else(|| Path::new("."));
    Ok(check_source(
        &text,
        &document.display().to_string(),
        base,
        model.as_ref(),
        options,
    ))
}

/// Checks document text. `label` names it in the report; `base` is the directory its fixture
/// paths are relative to.
pub fn check_source(
    text: &str,
    label: &str,
    base: &Path,
    model: Option<&Model>,
    options: &Options,
) -> Report {
    let mut sink = Sink::default();
    let loaded = match model {
        Some(model) => ess_ui::load_str_with(text, model),
        None => ess_ui::load_str(text),
    };
    match loaded {
        Err(error) => classify::refusal(text, &error, &mut sink),
        Ok(document) => {
            rules::run(&document, base, options, &mut sink);
            filter::run(&document, model, &mut sink);
            style::run(&document, &mut sink);
            raw::run(text, &document, &mut sink);
            hosts::run(&document, options, &mut sink);
            if let Some(model) = model {
                model.check(&document, &mut sink);
                names::run(model, &document, &mut sink);
            }
        }
    }
    Report::new(label, sink)
}

fn report(args: &CheckArgs) -> Result<Report, CheckError> {
    let options = options(args)?;
    check_with(&args.path, args.model.as_deref(), &options)
}

/// The exit status `ess ui check` ends with: 1 when any finding is an error, else 0.
pub fn exit_code(args: &CheckArgs) -> Result<u8, CheckError> {
    report(args).map(|report| u8::from(report.has_errors()))
}

/// Runs `ess ui check` with the model already compiled, ignoring `args.model`: for a caller that
/// resolves the specification itself (the CLI reads a directory through its `ess-inputs.yaml`).
pub fn run_with_model(args: &CheckArgs, model: Option<&Model>) -> Result<ExitCode, CheckError> {
    let text = std::fs::read_to_string(&args.path)
        .map_err(|error| CheckError(format!("cannot read {}: {error}", args.path.display())))?;
    let options = options(args)?;
    let base = args.path.parent().unwrap_or_else(|| Path::new("."));
    let report = check_source(
        &text,
        &args.path.display().to_string(),
        base,
        model,
        &options,
    );
    print(&report, args.format)
}

/// Runs `ess ui check`: prints the report to standard output and returns the exit status.
pub fn run(args: &CheckArgs) -> Result<ExitCode, CheckError> {
    let report = report(args)?;
    print(&report, args.format)
}

fn print(report: &Report, format: OutputFormat) -> Result<ExitCode, CheckError> {
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(report.render(format).as_bytes())
        .and_then(|()| stdout.flush())
        .map_err(|error| CheckError(format!("cannot write the report: {error}")))?;
    Ok(ExitCode::from(u8::from(report.has_errors())))
}
