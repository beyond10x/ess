//! Model-root data realization through the existing wire schema and shared target plan.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};

#[derive(Debug, clap::Args)]
#[command(group(clap::ArgGroup::new("selection").required(true).args(["root", "all_types"])))]
pub struct Args {
    #[command(flatten)]
    input: crate::SpecLocation,
    /// Qualified model type root. Repeat for a shared transitive closure.
    #[arg(long)]
    root: Vec<String>,
    /// Explicitly select every named type in the resolved model.
    #[arg(long)]
    all_types: bool,
    #[command(flatten)]
    options: crate::schema_bundle::TypeOptions,
    /// Library destination, outside the specification input tree.
    #[arg(long)]
    out: PathBuf,
}

pub fn run(args: &Args) -> Result<ExitCode> {
    let Ok((ir, _)) = crate::resolved(&args.input.path, crate::Format::Text)? else {
        return Ok(ExitCode::from(1));
    };
    let roots = if args.all_types {
        ir.types().keys().map(ToString::to_string).collect()
    } else {
        args.root.iter().cloned().collect()
    };
    let selection = match ess_gen::schema::ModelTypes::select(&ir, &roots) {
        Ok(selection) => selection,
        Err(errors) => {
            for error in errors {
                eprintln!("{}: {}: {}", error.name, error.rule, error.detail);
            }
            return Ok(ExitCode::from(1));
        }
    };
    let plan = schema_contract::realize::Plan::from_model(&selection)?;
    let mut files = crate::schema_bundle::type_files(&plan, &args.options)?;
    files.insert("source.schema.json".to_owned(), selection.to_json());
    let destination = crate::resolve_output_directory(&args.out)?;
    let source = std::fs::canonicalize(&args.input.path)?;
    if (source.is_dir() && destination.starts_with(&source))
        || files.keys().any(|name| destination.join(name) == source)
    {
        bail!("model type output must not replace or reside within its specification input");
    }
    crate::write_owned_files(
        &destination,
        "model-types",
        files
            .iter()
            .map(|(path, contents)| (path.as_str(), contents.as_str())),
    )?;
    println!(
        "{} model type(s), written to {}; runtime obligations in types-report.json",
        plan.declarations().len(),
        args.out.display()
    );
    Ok(ExitCode::SUCCESS)
}
