//! `ess generate client`: a typed event publisher for one component over its transport document
//! (beyond10x/ess#395; `docs/design/event-publishers.md`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context as _, Result};

use crate::Format;

/// The language a publisher is generated in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Target {
    /// A Cargo library, with the `JetStream` adapter as the `nats/` crate beside it.
    Rust,
    /// A Go module, with the `JetStream` adapter as the `natsjs/` module beside it.
    Go,
}

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    input: crate::SpecLocation,
    /// The component whose published events the publisher sends.
    #[arg(long)]
    component: String,
    /// The `ess-transport/1` document binding those events.
    #[arg(long)]
    transport: PathBuf,
    /// The language to generate.
    #[arg(long, value_enum)]
    target: Target,
    /// Native package identity.
    #[arg(long)]
    package: String,
    /// Go module identity, required only for Go.
    #[arg(long)]
    module: Option<String>,
    /// Library destination, outside the specification input tree.
    #[arg(long)]
    out: PathBuf,
}

pub fn run(args: &Args) -> Result<ExitCode> {
    if args.target == Target::Rust && args.module.is_some() {
        bail!("--module is only supported for the Go target");
    }
    let Ok((ir, _)) = crate::resolved(&args.input.path, Format::Text)? else {
        return Ok(ExitCode::from(1));
    };
    let Some(transport) = crate::transport::load(&args.transport, &ir, Format::Text)? else {
        return Ok(ExitCode::from(1));
    };
    let roots: BTreeSet<String> =
        ess_publisher::PublisherPlan::roots(&ir, &args.component, &transport)
            .into_iter()
            .collect();
    let types = if roots.is_empty() {
        None
    } else {
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
        Some((selection, plan))
    };
    let realization = match (&types, args.target) {
        (None, _) => None,
        (Some((_, plan)), Target::Rust) => Some(plan.rust(&args.package)?),
        (Some((_, plan)), Target::Go) => {
            let module = args
                .module
                .as_deref()
                .context("the Go target requires --module")?;
            Some(plan.go(&args.package, module)?)
        }
    };
    let declarations: BTreeMap<String, String> = match &realization {
        Some(realization) => serde_json::from_value(
            serde_json::to_value(&realization.report)?["declarations"].clone(),
        )?,
        None => BTreeMap::new(),
    };
    let publisher = match ess_publisher::plan(&ir, &args.component, &transport, &declarations) {
        Ok(publisher) => publisher,
        Err(refusals) => {
            for refusal in refusals {
                eprintln!("refused: {}: {}", refusal.rule, refusal.detail);
            }
            return Ok(ExitCode::from(1));
        }
    };
    let (Some((selection, _)), Some(realization)) = (types, realization) else {
        bail!("a publisher with operations always has payload types");
    };
    let report = format!("{}\n", serde_json::to_string_pretty(&realization.report)?);
    let mut files = realization.supporting;
    let client = match args.target {
        Target::Rust => {
            let manifest = files
                .get("Cargo.toml")
                .context("the Rust types library has a manifest")?
                .clone();
            files.insert("types.rs".to_owned(), realization.declarations);
            ess_publisher::rust(&publisher, &transport, &args.package, &manifest)
                .map_err(|refusal| anyhow::anyhow!("{}: {}", refusal.rule, refusal.detail))?
        }
        Target::Go => {
            files.insert("types.go".to_owned(), realization.declarations);
            ess_publisher::go(
                &publisher,
                &transport,
                &args.package,
                args.module.as_deref().unwrap_or_default(),
            )
        }
    };
    files.extend(client);
    files.insert("types-report.json".to_owned(), report);
    files.insert("source.schema.json".to_owned(), selection.to_json());
    write(args, &files)?;
    println!(
        "{} publish operation(s) for `{}`, written to {}; obligations in client-report.json",
        publisher.operations.len(),
        args.component,
        args.out.display()
    );
    Ok(ExitCode::SUCCESS)
}

/// Writes the library, refusing a destination inside the specification input.
fn write(args: &Args, files: &BTreeMap<String, String>) -> Result<()> {
    let destination = crate::resolve_output_directory(&args.out)?;
    let source = std::fs::canonicalize(&args.input.path)?;
    if (source.is_dir() && destination.starts_with(&source))
        || files.keys().any(|name| destination.join(name) == source)
    {
        bail!("client output must not replace or reside within its specification input");
    }
    crate::write_owned_files(
        &destination,
        "client",
        files
            .iter()
            .map(|(path, contents)| (path.as_str(), contents.as_str())),
    )?;
    Ok(())
}
