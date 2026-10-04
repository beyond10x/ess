//! `ess-ui-test/1`: UI behaviour tests over an `ess-ui/1` document.
//!
//! A test file names the document it tests and holds tests of steps. Steps select nodes by
//! canonical node path — the path `ess-ui-check` reports and the generated React project renders
//! as `data-ui-path` — with `rows/<key>` addressing a collection's item. A test can replace the
//! document's fixture views and channel scripts, play live events from the scripts and move the
//! virtual clock, so one test means the same thing to every renderer:
//!
//! - [`run`] runs every test headless against the terminal renderer (`ess-ui-tui`, drawn into
//!   ratatui's test backend) and reports `ess-ui-test-report/1`; it is what an
//!   `ess ui test --path <document> <tests…>` command wraps.
//! - [`playwright`] emits the same tests as a Playwright spec for the generated React project.
//!
//! The language is documented in `website/docs/reference/ess-ui-test.md`.

mod clock;
mod numbers;
mod parity;
mod playwright;
mod runner;
mod screen;
mod spec;
mod target;

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;

pub use playwright::{playwright, playwright_bound};
pub use runner::MAX_CYCLES_PER_ADVANCE;
pub use spec::{
    parse_steps, parse_str, Check, Fixtures, Play, PlayBeat, SectionState, Step, Test, TestFile,
};

/// The format marker every test file carries.
pub const FORMAT: &str = "ess-ui-test/1";
/// The format marker of the JSON report.
pub const REPORT_FORMAT: &str = "ess-ui-test-report/1";
/// Every step keyword, in the order the reference documents them.
pub const STEPS: [&str; 10] = [
    "open",
    "select",
    "type",
    "choose",
    "act",
    "page",
    "expect",
    "play",
    "advance",
    "expect_command",
];

/// A run that could not produce a report: a test file or the document does not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestError(pub(crate) String);

impl fmt::Display for TestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TestError {}

/// Passed or failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Every step held.
    Passed,
    /// A step did not.
    Failed,
}

/// One test's result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outcome {
    /// The test.
    pub name: String,
    /// Its status.
    pub status: Status,
    /// The step that failed, from 1; absent when the test passed or its setup failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<usize>,
    /// Why it failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// `ess-ui-test-report/1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// Always [`REPORT_FORMAT`].
    pub format: String,
    /// The document tested, as given.
    pub document: String,
    /// Every test, in file order.
    pub tests: Vec<Outcome>,
}

impl Report {
    /// Whether every test passed.
    pub fn passed(&self) -> bool {
        self.tests
            .iter()
            .all(|outcome| outcome.status == Status::Passed)
    }
}

/// How the report is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// One line per test, then a count.
    Text,
    /// `ess-ui-test-report/1`.
    Json,
}

/// The command line of `ess ui test`.
#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct TestArgs {
    /// The `ess-ui/1` document under test; every test file must name it.
    #[arg(long)]
    pub path: PathBuf,
    /// `ess-ui-test/1` files.
    #[arg(required = true)]
    pub tests: Vec<PathBuf>,
    /// Report format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
    /// Write the tests as a Playwright spec for the generated React project to this file,
    /// instead of running them.
    #[arg(long, value_name = "OUT")]
    pub playwright: Option<PathBuf>,
    /// The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`,
    /// or one file): a choice's `options` naming one of its enums list that enum's variants.
    /// Reads still come from the fixtures.
    #[arg(long)]
    pub model: Option<PathBuf>,
}

/// Reads a test file.
pub fn load(file: &Path) -> Result<TestFile, TestError> {
    let text = std::fs::read_to_string(file)
        .map_err(|error| TestError(format!("cannot read {}: {error}", file.display())))?;
    parse_str(&text, file)
}

/// Runs every test of `files` against `document`, headless in the terminal renderer.
///
/// Every file must name `document`. A failing test is an outcome in the report; an error is a
/// run that could not start.
pub fn execute(document: &Path, files: &[TestFile]) -> Result<Report, TestError> {
    execute_with(document, files, None)
}

/// A choice's `options` naming an enum the document does not declare, recorded where it was
/// written, when a run has no model to list it from (beyond10x/ess#330). The run still starts:
/// the options are empty, and a test fails at the step whose page shows such a choice.
#[derive(Default)]
struct Deferred(std::cell::RefCell<Vec<(ess_ui::NodePath, String)>>);

impl ess_ui::binding::ModelEnums for Deferred {
    fn system(&self) -> &'static str {
        ""
    }

    fn lookup(&self, name: &str) -> ess_ui::binding::EnumLookup {
        ess_ui::binding::EnumLookup::Enum {
            name: name.to_owned(),
            variants: Vec::new(),
        }
    }

    fn lookup_at(&self, name: &str, at: &ess_ui::NodePath) -> ess_ui::binding::EnumLookup {
        self.0.borrow_mut().push((at.clone(), name.to_owned()));
        self.lookup(name)
    }
}

/// [`execute`], with the enums of the document's model when `model` is given (`--model`).
///
/// Without one, a choice's `options` naming an enum the document does not declare lists nothing,
/// and every test fails at the first step that leaves a page showing such a choice on screen,
/// naming where it is written and `ess ui test --model`.
pub fn execute_with(
    document: &Path,
    files: &[TestFile],
    model: Option<&dyn ess_ui::binding::ModelEnums>,
) -> Result<Report, TestError> {
    check_documents(document, files)?;
    let deferred = Deferred::default();
    let doc = ess_ui::load_path_with(document, model.unwrap_or(&deferred))
        .map_err(|error| TestError(format!("{}: {error}", document.display())))?;
    let unlisted = deferred.0.into_inner();
    // Absolute, so the fixture paths a test rewrites are not resolved against the document's
    // directory a second time when the document was named by a relative path.
    let base = document
        .canonicalize()
        .map_err(|error| TestError(format!("cannot read {}: {error}", document.display())))?
        .parent()
        .map_or_else(PathBuf::new, Path::to_path_buf);
    // Removed when `dir` drops: when the run ends, and also when a test panics.
    let dir = tempfile::Builder::new()
        .prefix("ess-ui-test-")
        .tempdir()
        .map_err(|error| TestError(format!("cannot create a run directory: {error}")))?;
    let mut tests = Vec::new();
    for file in files {
        for test in &file.tests {
            let here = dir.path().join(tests.len().to_string());
            tests.push(run_test(&doc, &base, test, &here, &unlisted));
        }
    }
    dir.close()
        .map_err(|error| TestError(format!("cannot remove the run directory: {error}")))?;
    Ok(Report {
        format: REPORT_FORMAT.to_owned(),
        document: document.display().to_string(),
        tests,
    })
}

fn run_test(
    doc: &ess_ui::Document,
    base: &Path,
    test: &Test,
    dir: &Path,
    unlisted: &[(ess_ui::NodePath, String)],
) -> Outcome {
    let failed = |step: Option<usize>, message: String| Outcome {
        name: test.name.clone(),
        status: Status::Failed,
        step,
        message: Some(message),
    };
    let mut runner = match runner::Runner::start(doc, base, test, dir) {
        Ok(runner) => runner,
        Err(message) => return failed(None, format!("setup: {message}")),
    };
    if let Some(message) = unlisted_on(&runner, unlisted) {
        return failed(None, format!("setup: {message}"));
    }
    for (index, step) in test.steps.iter().enumerate() {
        if let Err(message) = runner.step(step) {
            return failed(Some(index + 1), format!("{step}: {message}"));
        }
        if let Some(message) = unlisted_on(&runner, unlisted) {
            return failed(Some(index + 1), format!("{step}: {message}"));
        }
    }
    Outcome {
        name: test.name.clone(),
        status: Status::Passed,
        step: None,
        message: None,
    }
}

/// Why the page on screen cannot be tested without a model: a choice on it, or in a shell, whose
/// `options` name an enum the document does not declare (beyond10x/ess#330).
fn unlisted_on(
    runner: &runner::Runner<'_>,
    unlisted: &[(ess_ui::NodePath, String)],
) -> Option<String> {
    let page = runner.current_page();
    unlisted
        .iter()
        .find(|(at, _)| match at.segments() {
            [top, ..] if top == "shells" => true,
            [top, name, ..] if top == "pages" => name == page,
            _ => false,
        })
        .map(|(at, name)| {
            format!(
                "{at} names `{name}`, which is no enum type of this document; a model enum is \
                 listed only with the model: run `ess ui test --model <specification>`"
            )
        })
}

/// Every file names `document`, relative to itself.
fn check_documents(document: &Path, files: &[TestFile]) -> Result<(), TestError> {
    let wanted = document
        .canonicalize()
        .map_err(|error| TestError(format!("cannot read {}: {error}", document.display())))?;
    for file in files {
        let named = file
            .source
            .parent()
            .unwrap_or(Path::new(""))
            .join(&file.document);
        let found = named.canonicalize().map_err(|error| {
            TestError(format!(
                "{}: document {}: {error}",
                file.source.display(),
                named.display()
            ))
        })?;
        if found != wanted {
            return Err(TestError(format!(
                "{} tests {}, not {}",
                file.source.display(),
                named.display(),
                document.display()
            )));
        }
    }
    Ok(())
}

/// Runs `args`, writing the report (or, with `--playwright`, where the spec went) to `out`.
///
/// Exits 1 when any test failed.
pub fn run_to(args: &TestArgs, out: &mut dyn Write) -> Result<ExitCode, TestError> {
    if let Some(model) = &args.model {
        return Err(TestError(format!(
            "--model {}: no model was compiled for it",
            model.display()
        )));
    }
    run_to_with(args, out, None)
}

/// [`run_to`] with the model `--model` names already compiled, which the caller does: this crate
/// reads the enums of a model and never compiles one (beyond10x/ess#330).
pub fn run_to_with(
    args: &TestArgs,
    out: &mut dyn Write,
    model: Option<&dyn ess_ui::binding::ModelEnums>,
) -> Result<ExitCode, TestError> {
    let files = args
        .tests
        .iter()
        .map(|file| load(file))
        .collect::<Result<Vec<_>, _>>()?;
    let io = |error: std::io::Error| TestError(error.to_string());
    if let Some(spec_file) = &args.playwright {
        check_documents(&args.path, &files)?;
        let document = match model {
            Some(model) => ess_ui::load_path_with(&args.path, model),
            None => ess_ui::load_path(&args.path),
        }
        .map_err(|error| TestError(format!("{}: {error}", args.path.display())))?;
        std::fs::write(spec_file, playwright(&files, &document))
            .map_err(|error| TestError(format!("{}: {error}", spec_file.display())))?;
        writeln!(out, "wrote {}", spec_file.display()).map_err(io)?;
        return Ok(ExitCode::SUCCESS);
    }
    let report = execute_with(&args.path, &files, model)?;
    match args.format {
        OutputFormat::Json => {
            let text = serde_json::to_string_pretty(&report)
                .map_err(|error| TestError(error.to_string()))?;
            writeln!(out, "{text}").map_err(io)?;
        }
        OutputFormat::Text => write_text(&report, out).map_err(io)?,
    }
    Ok(if report.passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn write_text(report: &Report, out: &mut dyn Write) -> std::io::Result<()> {
    for outcome in &report.tests {
        match outcome.status {
            Status::Passed => writeln!(out, "passed  {}", outcome.name)?,
            Status::Failed => {
                match outcome.step {
                    Some(step) => writeln!(out, "failed  {} (step {step})", outcome.name)?,
                    None => writeln!(out, "failed  {}", outcome.name)?,
                }
                for line in outcome.message.as_deref().unwrap_or_default().lines() {
                    writeln!(out, "        {line}")?;
                }
            }
        }
    }
    let passed = report
        .tests
        .iter()
        .filter(|outcome| outcome.status == Status::Passed)
        .count();
    writeln!(
        out,
        "{passed} passed, {} failed",
        report.tests.len() - passed
    )
}

/// Runs `args`, writing the report to standard output; the entry point `ess ui test` wraps.
pub fn run(args: &TestArgs) -> Result<ExitCode, TestError> {
    run_to(args, &mut std::io::stdout().lock())
}

/// [`run`] with the model `--model` names already compiled: what `ess ui test --model` wraps.
pub fn run_with_model(
    args: &TestArgs,
    model: Option<&dyn ess_ui::binding::ModelEnums>,
) -> Result<ExitCode, TestError> {
    run_to_with(args, &mut std::io::stdout().lock(), model)
}
