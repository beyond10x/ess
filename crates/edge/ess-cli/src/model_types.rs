//! Model-root data realization through the existing wire schema and shared target plan.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};

#[derive(Debug, clap::Args)]
#[command(group(
    clap::ArgGroup::new("selection")
        .required(true)
        .multiple(true)
        .args(["root", "all_types", "all_events", "all_commands", "all_views"])
))]
#[allow(
    clippy::struct_excessive_bools,
    reason = "each `--all-*` selector is an independent command-line switch"
)]
pub struct Args {
    #[command(flatten)]
    input: crate::SpecLocation,
    /// Qualified model type or event root, or a body: `<command>.input`, `<command>.response`,
    /// `<view>.row`. Repeat for a shared transitive closure.
    #[arg(long, conflicts_with_all = ["all_types", "all_events", "all_commands", "all_views"])]
    root: Vec<String>,
    /// Explicitly select every named type in the resolved model.
    #[arg(long)]
    all_types: bool,
    /// Explicitly select every event payload in the resolved model; combines with `--all-types`.
    #[arg(long)]
    all_events: bool,
    /// Explicitly select every command's input, and its response when it declares one; combines
    /// with the other `--all-*` selectors.
    #[arg(long)]
    all_commands: bool,
    /// Explicitly select every view's row; combines with the other `--all-*` selectors.
    #[arg(long)]
    all_views: bool,
    /// An `ess-http/1` document binding commands and views to methods and paths. Each command's
    /// `.input` is then the request body the `OpenAPI` document under that binding publishes: the
    /// input less the fields its route carries in the path. A command whose route publishes no body
    /// has no `.input` root: naming it is refused, and `--all-commands` leaves it out. Compiled as
    /// `ess specify http compile` compiles it, and refused when a route has no path.
    #[arg(long)]
    http: Option<PathBuf>,
    /// The `ess-http/1` binding the `--http` document refines, whose bytes its `refines:` digest
    /// pins.
    #[arg(long, requires = "http")]
    refines: Option<PathBuf>,
    #[command(flatten)]
    options: crate::schema_bundle::TypeOptions,
    /// How declarations are named: the qualified model name, or its last segment.
    #[arg(long, value_enum, default_value_t = Names::Qualified)]
    names: Names,
    /// Library destination, outside the specification input tree.
    #[arg(long)]
    out: PathBuf,
    /// Compare the library with what `--out` holds instead of writing it: exit 1 naming each file
    /// that is missing, differs, or is recorded in `.ess-output` and no longer generated. Writes
    /// nothing.
    #[arg(long, requires = "out")]
    check: bool,
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
    // Loaded and held to a path on every route as `ess generate --kind openapi --http` does, so
    // the bodies are the ones that document publishes.
    let mut binding = None;
    if let Some(path) = args.http.as_deref() {
        let Some(http) =
            crate::http::load(path, args.refines.as_deref(), &ir, crate::Format::Text)?
        else {
            return Ok(ExitCode::from(1));
        };
        match ess_gen::http::Binding::new(http) {
            Ok(bound) => binding = Some(bound),
            Err(unprojectable) => {
                eprintln!(
                    "{} cannot be projected to OpenAPI:\n{unprojectable}",
                    path.display()
                );
                return Ok(ExitCode::from(1));
            }
        }
    }
    let roots = if args.all_types || args.all_events || args.all_commands || args.all_views {
        let types = ir.types().keys().filter(|_| args.all_types);
        let events = ir.events().keys().filter(|_| args.all_events);
        let mut roots: std::collections::BTreeSet<String> =
            types.chain(events).map(ToString::to_string).collect();
        if args.all_commands {
            roots.extend(ess_gen::schema::ModelTypes::bound_command_roots(
                &ir,
                binding.as_ref(),
            ));
        }
        if args.all_views {
            roots.extend(ess_gen::schema::ModelTypes::view_roots(&ir));
        }
        roots
    } else {
        args.root.iter().cloned().collect()
    };
    let selection = match ess_gen::schema::ModelTypes::select_bound(&ir, &roots, binding.as_ref()) {
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
    let owned = files
        .iter()
        .map(|(path, contents)| (path.as_str(), contents.as_str()));
    if args.check {
        if !crate::check_owned_tree(&destination, "model-types", owned, "ess generate types")? {
            return Ok(ExitCode::from(1));
        }
        println!(
            "{} model type(s), current in {}",
            plan.declarations().len(),
            args.out.display()
        );
        return Ok(ExitCode::SUCCESS);
    }
    crate::write_owned_files(&destination, "model-types", owned)?;
    println!(
        "{} model type(s), written to {}; runtime obligations in types-report.json",
        plan.declarations().len(),
        args.out.display()
    );
    Ok(ExitCode::SUCCESS)
}
