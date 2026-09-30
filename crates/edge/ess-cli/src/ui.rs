//! `ess ui` and `ess generate ui`: the `crates/ui/` entry points, mounted on the command line.
//!
//! Each command is a thin shell over one crate's own entry point — [`ess_ui::check`], [`ess_ui_check::run`],
//! [`ess_ui_docs::run`], [`ess_ui_tui::run`], [`ess_ui_test::run`] and [`ess_ui_react::run`] — so what the command does
//! is what that crate does, and what it prints on a refusal is that crate's message verbatim.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Subcommand, ValueEnum};

/// `ess ui`: renderer-neutral UI documents (`ess-ui/1`) — `crates/ui/`.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Load an `ess-ui/1` document and print what it holds, or name the node that refuses it.
    Load(Load),
    /// Check an `ess-ui/1` document, and with `--model` its references into an ESS model; each
    /// finding is named by node path. Exits 1 when any finding is an error.
    Check(ess_ui_check::CheckArgs),
    /// Render the `ess-ui/1` reference from its schema, as HTML or Markdown.
    Docs(ess_ui_docs::DocsArgs),
    /// Run an `ess-ui/1` document, answering reads from its fixtures.
    Run(Run),
    /// Run `ess-ui-test/1` tests headless against the terminal renderer, or with `--playwright`
    /// write them as a Playwright spec for the generated React project. Exits 1 when a test fails.
    Test(ess_ui_test::TestArgs),
}

/// What `ess ui load` loads.
#[derive(Debug, clap::Args)]
pub(crate) struct Load {
    /// The `ess-ui/1` document to load.
    #[arg(long)]
    path: PathBuf,
}

/// Which renderer `ess ui run` runs the document in, and the document.
#[derive(Debug, clap::Args)]
pub(crate) struct Run {
    /// Run in the terminal; the only renderer this command offers so far, so it must be named.
    #[arg(long, required = true)]
    tui: bool,
    #[command(flatten)]
    document: ess_ui_tui::TuiArgs,
}

/// `ess generate ui`: an `ess-ui/1` document becomes an application.
#[derive(Debug, clap::Args)]
pub(crate) struct Generate {
    /// What to generate.
    #[arg(long, value_enum)]
    target: Target,
    #[command(flatten)]
    react: ess_ui_react::ReactArgs,
}

/// The applications `ess generate ui` generates.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Target {
    /// A Vite + React + TypeScript project.
    React,
}

/// Runs an `ess ui` command.
pub(crate) fn run(command: &Command) -> ExitCode {
    match command {
        Command::Load(load) => match ess_ui::check(&load.path) {
            Ok(summary) => success(&summary.to_string()),
            Err(error) => refusal(&format!("{}: {}", error.path(), error.message())),
        },
        Command::Check(args) => {
            let checked = match args.model.as_deref().map(model).transpose() {
                Ok(model) => ess_ui_check::run_with_model(args, model.as_ref()),
                Err(error) => return refusal(&format!("{error:#}")),
            };
            match checked {
                Ok(code) => code,
                Err(error) => refusal(&error.to_string()),
            }
        }
        Command::Docs(args) => match ess_ui_docs::run(args) {
            Ok(summary) => success(&summary),
            Err(error) => refusal(&error.problems().join("\n")),
        },
        Command::Run(run) => {
            debug_assert!(run.tui, "clap requires `--tui`");
            match ess_ui_tui::run(&run.document) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => refusal(&error.to_string()),
            }
        }
        Command::Test(args) => match ess_ui_test::run(args) {
            Ok(code) => code,
            Err(error) => refusal(&error.to_string()),
        },
    }
}

/// Runs `ess generate ui`.
pub(crate) fn generate(arguments: &Generate) -> ExitCode {
    match arguments.target {
        Target::React => match ess_ui_react::run(&arguments.react) {
            Ok(summary) => success(&summary),
            Err(error) => refusal(&error.to_string()),
        },
    }
}

fn success(summary: &str) -> ExitCode {
    println!("{summary}");
    ExitCode::SUCCESS
}

fn refusal(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(1)
}

/// The `--model` of `ess ui check`, resolved the way `ess specify validate --path` resolves a
/// specification: a directory through its `ess-inputs.yaml` when it has one, and a path naming
/// that manifest as its directory (beyond10x/ess#262).
fn model(path: &Path) -> anyhow::Result<ess_ui_check::Model> {
    let path = if path
        .file_name()
        .is_some_and(|name| name == "ess-inputs.yaml")
    {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    let base = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let sources: Vec<(String, String)> =
        crate::input_discovery::acquire(path, crate::input_discovery::Kind::Specification)?
            .into_iter()
            .map(|input| {
                let label = input
                    .origin
                    .strip_prefix(base)
                    .unwrap_or(&input.origin)
                    .display()
                    .to_string();
                (label, input.text)
            })
            .collect();
    Ok(ess_ui_check::model_from_sources(&sources, path)?)
}
