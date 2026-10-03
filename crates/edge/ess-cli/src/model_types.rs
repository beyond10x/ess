//! Model-root data realization through the existing wire schema and shared target plan.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};

#[derive(Debug, clap::Args)]
#[command(group(
    clap::ArgGroup::new("selection")
        .required(true)
        .multiple(true)
        .args(["root", "all_types", "all_events"])
))]
pub struct Args {
    #[command(flatten)]
    input: crate::SpecLocation,
    /// Qualified model type or event root. Repeat for a shared transitive closure.
    #[arg(long, conflicts_with_all = ["all_types", "all_events"])]
    root: Vec<String>,
    /// Explicitly select every named type in the resolved model.
    #[arg(long)]
    all_types: bool,
    /// Explicitly select every event payload in the resolved model; combines with `--all-types`.
    #[arg(long)]
    all_events: bool,
    #[command(flatten)]
    options: crate::schema_bundle::TypeOptions,
    /// How declarations are named: the qualified model name, or its last segment.
    #[arg(long, value_enum, default_value_t = Names::Qualified)]
    names: Names,
    /// Library destination, outside the specification input tree.
    #[arg(long)]
    out: PathBuf,
}

/// How generated declarations are named (beyond10x/ess#409).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Names {
    /// The qualified model name, every segment kept: `MeteringItemsUsageRecorded`.
    Qualified,
    /// The last segment, refused where two selected declarations would share one: `UsageRecorded`.
    Short,
}

impl Names {
    /// Rename the plan's declarations if short names were asked for.
    pub fn apply(
        self,
        plan: schema_contract::realize::Plan,
    ) -> Result<schema_contract::realize::Plan, schema_contract::realize::Refused> {
        match self {
            Self::Qualified => Ok(plan),
            Self::Short => plan.with_short_names(),
        }
    }
}

pub fn run(args: &Args) -> Result<ExitCode> {
    let Ok((ir, _)) = crate::resolved(&args.input.path, crate::Format::Text)? else {
        return Ok(ExitCode::from(1));
    };
    let roots = if args.all_types || args.all_events {
        let types = ir.types().keys().filter(|_| args.all_types);
        let events = ir.events().keys().filter(|_| args.all_events);
        types.chain(events).map(ToString::to_string).collect()
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
    let plan = args
        .names
        .apply(schema_contract::realize::Plan::from_model(&selection)?)?;
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
