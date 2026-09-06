//! Offline checking and reference execution of explicitly source-pinned recipes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Args, ValueEnum};
use schema_contract::bundle::Bundle;
use schema_contract::realize::normalize::Plan;

#[derive(Debug, Args)]
pub struct Sources {
    /// Authored ess-normalization/1 or /2 recipe; every branch is checked first.
    #[arg(long)]
    recipe: PathBuf,
    /// Replay-checked source bundles referenced by canonical digest. Repeat as needed.
    #[arg(long, required = true)]
    bundle: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[command(flatten)]
    sources: Sources,
    /// Write the canonical checked recipe here; omit for JSON on stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[command(flatten)]
    sources: Sources,
    /// Exact external discriminator; never inferred from the input object.
    #[arg(long)]
    branch: String,
    /// JSON instance with unique keys and losslessly representable numbers.
    #[arg(long)]
    input: PathBuf,
    /// Write the complete result here; omit for JSON on stdout. Refusals write no result.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Target {
    /// Standalone Rust library with the reference execution engine.
    Rust,
    /// Standalone Go library with checked operation bindings.
    Go,
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[command(flatten)]
    sources: Sources,
    /// Executable normalization target, not structural type projection.
    #[arg(long, value_enum)]
    target: Target,
    /// Native library package identity.
    #[arg(long)]
    package: String,
    /// Go module identity; required for Go and refused for Rust.
    #[arg(long)]
    module: Option<String>,
    /// Directory for generated library, source inputs and provenance report.
    #[arg(long)]
    out: PathBuf,
    /// Check planned file bytes without writing; unrelated files are not inspected or removed.
    #[arg(long)]
    check: bool,
}

impl Sources {
    fn plan(&self) -> Result<Plan> {
        let bundles = self
            .bundle
            .iter()
            .map(|path| {
                Bundle::read(&read(path)?)
                    .with_context(|| format!("checking bundle {}", path.display()))
            })
            .collect::<Result<Vec<_>>>()?;
        Plan::read(&read(&self.recipe)?, &bundles)
            .with_context(|| format!("checking normalization recipe {}", self.recipe.display()))
    }

    fn paths(&self) -> impl Iterator<Item = &Path> {
        std::iter::once(self.recipe.as_path()).chain(self.bundle.iter().map(PathBuf::as_path))
    }
}

pub fn check(args: &CheckArgs) -> Result<ExitCode> {
    let plan = args.sources.plan()?;
    emit(args.sources.paths(), args.out.as_deref(), &plan.to_json())?;
    Ok(ExitCode::SUCCESS)
}

pub fn run(args: &RunArgs) -> Result<ExitCode> {
    let plan = args.sources.plan()?;
    let value = plan.run_json(&args.branch, &read(&args.input)?)?;
    let contents = format!("{}\n", serde_json::to_string_pretty(&value)?);
    emit(
        args.sources
            .paths()
            .chain(std::iter::once(args.input.as_path())),
        args.out.as_deref(),
        &contents,
    )?;
    Ok(ExitCode::SUCCESS)
}

pub fn generate(args: &GenerateArgs) -> Result<ExitCode> {
    let plan = args.sources.plan()?;
    let generated = match args.target {
        Target::Rust => {
            if args.module.is_some() {
                bail!("--module is only supported for the Go target");
            }
            plan.rust(&args.package)?
        }
        Target::Go => plan.go(
            &args.package,
            args.module
                .as_deref()
                .context("the Go target requires --module")?,
        )?,
    };
    let root = crate::preflight_generated_files(
        &args.out,
        &generated
            .files
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )?;
    for input in args.sources.paths() {
        let input = fs::canonicalize(input)?;
        if generated.files.keys().any(|path| root.join(path) == input) {
            bail!("normalization output must not replace a recipe or bundle input");
        }
    }
    if args.check {
        return check_files(&root, &generated.files);
    }
    crate::write_preflighted_files(
        generated
            .files
            .iter()
            .map(|(path, contents)| (root.join(path), contents.as_str())),
    )?;
    println!(
        "{} file(s), written to {}; provenance in normalization-report.json",
        generated.files.len(),
        args.out.display()
    );
    Ok(ExitCode::SUCCESS)
}

fn check_files(
    root: &Path,
    files: &std::collections::BTreeMap<String, String>,
) -> Result<ExitCode> {
    let mut differs = false;
    for (path, expected) in files {
        let destination = root.join(path);
        let state = match fs::read(&destination) {
            Ok(actual) if actual == expected.as_bytes() => continue,
            Ok(_) => "stale",
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => "missing",
            Err(error) => {
                return Err(error).with_context(|| format!("reading {}", destination.display()))
            }
        };
        println!("{path}: {state}");
        differs = true;
    }
    if differs {
        Ok(ExitCode::from(1))
    } else {
        println!("{} file(s): current", files.len());
        Ok(ExitCode::SUCCESS)
    }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

fn emit<'a>(
    inputs: impl Iterator<Item = &'a Path>,
    output: Option<&Path>,
    contents: &str,
) -> Result<()> {
    if let Some(output) = output {
        let destination = crate::preflight_named_output(output)?;
        for input in inputs {
            if fs::canonicalize(input)? == destination {
                bail!("normalization output must not replace a recipe, bundle or instance input");
            }
        }
        crate::write_preflighted_files([(destination, contents)])?;
        println!("wrote {}", output.display());
    } else {
        print!("{contents}");
    }
    Ok(())
}
