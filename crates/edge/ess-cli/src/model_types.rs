//! Model-root data realization through the existing wire schema and shared target plan.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};

#[derive(Debug, clap::Args)]
#[command(group(clap::ArgGroup::new("selection").required(true).multiple(true).args(["root", "event", "all_types"])))]
pub struct Args {
    #[command(flatten)]
    input: crate::SpecLocation,
    /// Qualified model type root. Repeat for a shared transitive closure.
    #[arg(long)]
    root: Vec<String>,
    /// Qualified event payload root. Repeat or combine with explicit type roots.
    #[arg(long)]
    event: Vec<ess_domain::name::QualifiedName>,
    /// Explicitly select every named type in the resolved model.
    #[arg(long, conflicts_with_all = ["root", "event"])]
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
    let selected = if args.event.is_empty() {
        ess_gen::schema::ModelTypes::select(&ir, &roots)
    } else {
        let mut typed = std::collections::BTreeSet::new();
        for root in &roots {
            typed.insert(ess_gen::schema::ModelRoot::Type(root.parse()?));
        }
        typed.extend(
            args.event
                .iter()
                .cloned()
                .map(ess_gen::schema::ModelRoot::Event),
        );
        ess_gen::schema::ModelTypes::select_roots(&ir, &typed)
    };
    let selection = match selected {
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
