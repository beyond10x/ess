//! `ess specify transport` and the `--transport` input of `ess generate --kind asyncapi`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result};
use ess_compiler::EssIr;

use crate::{render, resolved, Format};

/// `ess specify transport`: an `ess-transport/1` or `/2` document against one exact ESS.
#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Validate and resolve an `ess-transport/1` or `/2` document.
    Validate(Input),
    /// Compile a document into its canonical `ess-transport-ir/1` or `/2` form.
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
    /// An `ess-transport/1` or `/2` JSON or YAML document.
    #[arg(long)]
    path: PathBuf,
    /// One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml`.
    #[arg(long = "spec")]
    specification: PathBuf,
    /// Output and diagnostic rendering.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// Reads a transport document and compiles it against `ir`, printing every refusal.
///
/// `Ok(None)` is a refusal already reported; the caller exits 1.
pub fn load(path: &Path, ir: &EssIr, format: Format) -> Result<Option<ess_transport::TransportIr>> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let spec = if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        ess_transport::TransportSpec::from_json(&text)
            .with_context(|| format!("reading {} as transport JSON", path.display()))?
    } else {
        ess_transport::TransportSpec::from_yaml(&text)
            .with_context(|| format!("reading {} as transport YAML", path.display()))?
    };
    match ess_transport::compile(&spec, ir) {
        Ok(transport) => Ok(Some(transport)),
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

/// The `AsyncAPI` documents with a transport document beside the model; `Ok(None)` is a refusal
/// already reported.
pub fn asyncapi(
    path: &Path,
    ir: &EssIr,
    format: Format,
) -> Result<Option<BTreeMap<String, ess_gen::Artifact>>> {
    let Some(transport) = load(path, ir, format)? else {
        return Ok(None);
    };
    Ok(Some(ess_gen::artifact::run(
        &ess_gen::asyncapi::TransportedAsyncApi(transport),
        ir,
    )?))
}

pub fn run(command: &Command) -> Result<ExitCode> {
    let (input, out) = match command {
        Command::Validate(input) => (input, None),
        Command::Compile { input, out } => (input, out.as_deref()),
    };
    let Ok((ir, _)) = resolved(&input.specification, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let Some(transport) = load(&input.path, &ir, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let summary = format!(
        "{} — {} channel(s), {} stream(s)",
        input.path.display(),
        transport.channels().len(),
        transport.streams().len()
    );
    match command {
        Command::Validate(_) => match input.format {
            Format::Text => println!("{summary}, valid"),
            _ => render(&transport, input.format)?,
        },
        Command::Compile { .. } => {
            let json = transport.to_canonical_json();
            if let Some(path) = out {
                fs::write(path, &json).with_context(|| format!("writing {}", path.display()))?;
            }
            match input.format {
                Format::Text => println!(
                    "{summary}, compiled{}",
                    out.map_or_else(String::new, |path| format!(" to {}", path.display()))
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&transport, Format::Yaml)?,
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
