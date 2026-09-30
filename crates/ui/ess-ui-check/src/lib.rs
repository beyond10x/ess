//! Every check an `ess-ui/1` document is held to, reported by canonical node path.
//!
//! The checks are the schema's `checks.list` (`schemas/ui/ess-ui.schema.yaml`), the rules the
//! schema states elsewhere that the loader does not refuse (placement refusals, the type rule, a
//! widget containing itself, the `degrades` capabilities, a primitive's `exactly_one_of`), and —
//! given an ESS model — that every view, command and event the document names exists in it and
//! that every section's read is readable by some actor — an approximation, since ESS grants
//! commands and not views (documented on [`Model`]). [`CHECKS`] lists them all.
//!
//! A document the loader refuses yields exactly one finding, the refusal, classified under the
//! check it breaks. A document that loads is checked in full, and every finding names its node by
//! the [`ess_ui::NodePath`] the loader gives it, so a finding keeps its path when a sibling is
//! added or removed.
//!
//! [`check`] is the entry point; [`run`] wraps it for the `ess ui check` command, rendering human
//! text or JSON (`ess-ui-check/1`) and exiting 1 when any finding is an error.

mod classify;
mod expr;
mod model;
mod raw;
mod rules;
mod schema;
mod walk;

use std::fmt::{self, Write as _};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub use model::{load_model, Model};

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
    schema_check("page_reachable", Severity::Error),
    schema_check("opens_resolves", Severity::Error),
    schema_check("same_as_resolves", Severity::Error),
    schema_check("page_refs", Severity::Error),
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
    // The loader refused the document for a reason no other check names.
    rule("document_loads"),
    // `layers`: a node stands where its layer may not.
    rule("layer_rules"),
    // `Degrades.fields.(capability)`: a key names a capability, its value one of its fallbacks.
    rule("degrades_known"),
    // `Degrades.rule`: a renderer lacking a capability finds a fallback that is not `refuse`.
    rule("degrades_cover"),
    // With `--model`: the names the document resolves in the ESS model.
    rule("view_in_model"),
    rule("command_in_model"),
    rule("event_in_model"),
    rule("section_readable"),
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
}

#[derive(serde::Serialize)]
struct Json<'a> {
    format: &'static str,
    document: &'a str,
    findings: &'a [Finding],
}

impl Report {
    fn new(document: &str, sink: Sink) -> Self {
        let mut findings = sink.findings;
        findings.sort();
        findings.dedup();
        Self {
            document: document.to_owned(),
            findings,
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
    match ess_ui::load_str(text) {
        Err(error) => classify::refusal(text, &error, &mut sink),
        Ok(document) => {
            rules::run(&document, base, options, &mut sink);
            raw::run(text, &document, &mut sink);
            if let Some(model) = model {
                model.check(&document, &mut sink);
            }
        }
    }
    Report::new(label, sink)
}

fn report(args: &CheckArgs) -> Result<Report, CheckError> {
    let options = Options {
        lacks: args.lacks.clone(),
    };
    check_with(&args.path, args.model.as_deref(), &options)
}

/// The exit status `ess ui check` ends with: 1 when any finding is an error, else 0.
pub fn exit_code(args: &CheckArgs) -> Result<u8, CheckError> {
    report(args).map(|report| u8::from(report.has_errors()))
}

/// Runs `ess ui check`: prints the report to standard output and returns the exit status.
pub fn run(args: &CheckArgs) -> Result<ExitCode, CheckError> {
    let report = report(args)?;
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(report.render(args.format).as_bytes())
        .and_then(|()| stdout.flush())
        .map_err(|error| CheckError(format!("cannot write the report: {error}")))?;
    Ok(ExitCode::from(u8::from(report.has_errors())))
}
