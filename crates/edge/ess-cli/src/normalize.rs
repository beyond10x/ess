//! Offline checking and reference execution of explicitly source-pinned recipes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::Args;
use schema_contract::bundle::Bundle;
use schema_contract::realize::normalize::Plan;

#[derive(Debug, Args)]
pub struct Sources {
    /// Authored ess-normalization/1 recipe; every branch is checked before execution.
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
