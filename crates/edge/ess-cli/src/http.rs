//! `ess specify http`: an `ess-http/1` binding of commands and views to HTTP routes.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result};
use ess_compiler::EssIr;

use crate::{render, resolved, Format};

/// `ess specify http`: an `ess-http/1` document against one exact ESS.
#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Validate and resolve an `ess-http/1` document.
    Validate(Input),
    /// Compile a document into its canonical `ess-http-ir/1` form.
    Compile {
        #[command(flatten)]
        input: Input,
        /// Where to write canonical JSON IR.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Debug, clap::Args)]
pub struct Input {
    /// One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml`.
    #[arg(long, default_value = ".")]
    path: PathBuf,
    /// An `ess-http/1` JSON or YAML document.
    #[arg(long)]
    binding: PathBuf,
    /// The `ess-http/1` binding the document refines, whose bytes its `refines:` digest pins.
    #[arg(long)]
    refines: Option<PathBuf>,
    /// Output and diagnostic rendering.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

fn is_json(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "json")
}

/// Reads the binding at `path` refines, by its file's exact bytes, whose digest `refines:` pins.
fn read_refined(path: &Path) -> Result<ess_http::Refined> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    if is_json(path) {
        ess_http::Refined::from_json(&bytes)
            .with_context(|| format!("reading {} as refined HTTP binding JSON", path.display()))
    } else {
        ess_http::Refined::from_yaml(&bytes)
            .with_context(|| format!("reading {} as refined HTTP binding YAML", path.display()))
    }
}

/// Reads an HTTP binding, and the binding it refines when `refines` names one, and compiles it
/// against `ir`, printing every refusal.
///
/// `Ok(None)` is a refusal already reported; the caller exits 1.
pub fn load(
    path: &Path,
    refines: Option<&Path>,
    ir: &EssIr,
    format: Format,
) -> Result<Option<ess_http::HttpIr>> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let spec = if is_json(path) {
        ess_http::HttpSpec::from_json(&text)
            .with_context(|| format!("reading {} as HTTP binding JSON", path.display()))?
    } else {
        ess_http::HttpSpec::from_yaml(&text)
            .with_context(|| format!("reading {} as HTTP binding YAML", path.display()))?
    };
    let protocol = refines.map(read_refined).transpose()?;
    match ess_http::compile_refining(&spec, protocol.as_ref(), ir, &derived_routes(ir)) {
        Ok(http) => Ok(Some(http)),
        Err(diagnostics) => {
            if matches!(format, Format::Text) {
                eprintln!("{} was refused:\n{diagnostics}", path.display());
            } else {
                render(&diagnostics, format)?;
            }
            Ok(None)
        }
    }
}

/// The routes ESS derives for every component of `ir` without a binding, read from the one route
/// table every HTTP artifact reads (`ess_gen::http::routes`). A binding is checked against these:
/// `ess-http` takes them as an input rather than depending on the generator.
pub fn derived_routes(ir: &EssIr) -> Vec<ess_http::DerivedRoute> {
    ess_gen::http::derived(ir)
}

/// `ess generate --kind openapi --http FILE [--refines FILE]`: the `OpenAPI` documents with the
/// methods and paths the binding declares.
///
/// The binding is compiled exactly as `ess specify http compile` compiles it, with the same
/// refusals, and then refused when a route it declares has no path: a published document cannot
/// carry an open route (`docs/design/http-presentation-binding.md`, decided for stage 1, answer 1).
/// `Ok(None)` is a refusal already reported; the caller exits 1 and writes nothing.
pub fn openapi(
    binding: &Path,
    refines: Option<&Path>,
    ir: &EssIr,
    format: Format,
) -> Result<Option<BTreeMap<String, ess_gen::Artifact>>> {
    let Some(http) = load(binding, refines, ir, format)? else {
        return Ok(None);
    };
    match ess_gen::http::Binding::new(http) {
        Ok(bound) => Ok(Some(ess_gen::artifact::run(
            &ess_gen::openapi::BoundOpenApi(bound),
            ir,
        )?)),
        Err(unprojectable) => {
            eprintln!(
                "{} cannot be projected to OpenAPI:\n{unprojectable}",
                binding.display()
            );
            Ok(None)
        }
    }
}

/// `ess generate synthesize --target rust|go --http FILE [--refines FILE]`: the binding a synthesized
/// server serves.
///
/// Compiled exactly as `ess specify http compile` compiles it, with the same refusals, then held
/// to a path on every route: a server cannot serve an open route. Whether a route claims a path the
/// server answers itself is the synthesis's to refuse (`ess_synth::synthesize_bound`). `Ok(None)`
/// is a refusal already reported; the caller exits 1 and writes nothing.
pub fn server(
    binding: &Path,
    refines: Option<&Path>,
    ir: &EssIr,
    format: Format,
) -> Result<Option<ess_gen::http::Binding>> {
    let Some(http) = load(binding, refines, ir, format)? else {
        return Ok(None);
    };
    match ess_gen::http::Binding::new(http) {
        Ok(bound) => Ok(Some(bound)),
        Err(unservable) => {
            eprintln!("{} cannot be served:\n{unservable}", binding.display());
            Ok(None)
        }
    }
}

pub fn run(command: &Command) -> Result<ExitCode> {
    let (input, out) = match command {
        Command::Validate(input) => (input, None),
        Command::Compile { input, out } => (input, out.as_deref()),
    };
    let Ok((ir, _)) = resolved(&input.path, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let Some(http) = load(&input.binding, input.refines.as_deref(), &ir, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let open = http
        .routes()
        .iter()
        .filter(|route| route.path.is_none())
        .count();
    let summary = format!(
        "{} — {} route(s), {open} open",
        input.binding.display(),
        http.routes().len()
    );
    match command {
        Command::Validate(_) => match input.format {
            Format::Text => println!("{summary}, valid"),
            _ => render(&http, input.format)?,
        },
        Command::Compile { .. } => {
            let json = http.to_canonical_json();
            if let Some(path) = out {
                fs::write(path, &json).with_context(|| format!("writing {}", path.display()))?;
            }
            match input.format {
                Format::Text => println!(
                    "{summary}, compiled{}",
                    out.map_or_else(String::new, |path| format!(" to {}", path.display()))
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&http, Format::Yaml)?,
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
