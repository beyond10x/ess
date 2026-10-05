//! `ess ui` and `ess generate ui`: the `crates/ui/` entry points, mounted on the command line.
//!
//! Each command is a thin shell over one crate's own entry point — [`ess_ui::check`], [`ess_ui_check::run`],
//! [`ess_ui_docs::run`], [`ess_ui_tui::run`], [`ess_ui_test::run`], [`ess_ui_react::run`] and
//! [`ess_ui_tui::generate::generate`] — so what the command does is what that crate does, and
//! what it prints on a refusal is that crate's message verbatim.

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
    /// Run an `ess-ui/1` document, answering reads from its fixtures, or with `--model` reading
    /// and commanding the HTTP surface the specification serves.
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
    /// The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`,
    /// or one file): a choice's `options` naming one of its enums list that enum's variants.
    #[arg(long)]
    model: Option<PathBuf>,
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
    project: ess_ui_react::ReactArgs,
}

/// The applications `ess generate ui` generates.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Target {
    /// A React + TypeScript project: react and react-dom only, generated routing, built with esbuild;
    /// with `--model`, bound to the HTTP surface the specification serves.
    React,
    /// A Rust terminal application crate: a clap command line over `ess-ui-tui`, fetched by the
    /// Git tag of this ESS version, bound to the HTTP surface the specification serves (`--model`
    /// is required) and run with `--base-url`.
    Tui,
}

/// Runs an `ess ui` command.
pub(crate) fn run(command: &Command) -> ExitCode {
    match command {
        Command::Load(load) => {
            let loaded = match load.model.as_deref().map(model).transpose() {
                Ok(Some(model)) => ess_ui::check_with(&load.path, &model),
                Ok(None) => ess_ui::check(&load.path),
                Err(error) => return refusal(&format!("{error:#}")),
            };
            match loaded {
                Ok(summary) => success(&summary.to_string()),
                Err(error) => refusal(&format!("{}: {}", error.path(), error.message())),
            }
        }
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
            let document = &run.document;
            let binding = match document
                .model
                .as_deref()
                .map(|model| bind(&document.path, model))
            {
                None => None,
                Some(Ok(binding)) => Some(binding),
                Some(Err(error)) => return refusal(&format!("{error:#}")),
            };
            match ess_ui_tui::run(document, binding.as_ref()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => refusal(&error.to_string()),
            }
        }
        Command::Test(args) => {
            let tested = match args.model.as_deref().map(model).transpose() {
                Ok(model) => ess_ui_test::run_with_model(
                    args,
                    model
                        .as_ref()
                        .map(|model| model as &dyn ess_ui::binding::ModelEnums),
                ),
                Err(error) => return refusal(&format!("{error:#}")),
            };
            match tested {
                Ok(code) => code,
                Err(error) => refusal(&error.to_string()),
            }
        }
    }
}

/// Runs `ess generate ui`.
pub(crate) fn generate(arguments: &Generate) -> ExitCode {
    match arguments.target {
        Target::React => {
            let react = &arguments.project;
            let binding = match react.model.as_deref().map(|model| bind(&react.path, model)) {
                None => None,
                Some(Ok(binding)) => Some(binding),
                Some(Err(error)) => return refusal(&format!("{error:#}")),
            };
            match ess_ui_react::run(react, binding.as_ref()) {
                Ok(summary) => success(&summary),
                Err(error) => refusal(&error.to_string()),
            }
        }
        Target::Tui => {
            let project = &arguments.project;
            let Some(model) = project.model.as_deref() else {
                return refusal(
                    "--target tui needs --model: the terminal app is bound to the HTTP surface \
                     the specification serves",
                );
            };
            let binding = match bind(&project.path, model) {
                Ok(binding) => binding,
                Err(error) => return refusal(&format!("{error:#}")),
            };
            match ess_ui_tui::generate::generate(&project.path, &binding, &project.out) {
                Ok(summary) => success(&summary),
                Err(error) => refusal(&error.to_string()),
            }
        }
    }
}

/// The route table the document at `document` binds to on the surface the specification at
/// `model` serves, through `ess_ui_check::binding`. The document is loaded with the model's enums,
/// so a choice's `options` may name one (beyond10x/ess#330).
fn bind(document: &Path, model: &Path) -> anyhow::Result<ess_ui::binding::Binding> {
    let (sources, shown) = sources(model)?;
    let enums = ess_ui_check::model_from_sources(&sources, shown)?;
    let loaded = ess_ui::load_path_with(document, &enums)
        .map_err(|error| anyhow::anyhow!("{}: {error}", document.display()))?;
    Ok(ess_ui_check::binding(&loaded, &sources)?)
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
    let (sources, path) = sources(path)?;
    Ok(ess_ui_check::model_from_sources(&sources, path)?)
}

/// The `(label, text)` sources of a `--model`, resolved as [`model`] says, and the path a refusal
/// names it by.
fn sources(path: &Path) -> anyhow::Result<(Vec<(String, String)>, &Path)> {
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
    Ok((sources, path))
}
