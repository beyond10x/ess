//! `ess specify http`: an `ess-http/1` binding of commands and views to HTTP routes.

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
    /// Output and diagnostic rendering.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// Reads an HTTP binding and compiles it against `ir`, printing every refusal.
///
/// `Ok(None)` is a refusal already reported; the caller exits 1.
pub fn load(path: &Path, ir: &EssIr, format: Format) -> Result<Option<ess_http::HttpIr>> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let spec = if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        ess_http::HttpSpec::from_json(&text)
            .with_context(|| format!("reading {} as HTTP binding JSON", path.display()))?
    } else {
        ess_http::HttpSpec::from_yaml(&text)
            .with_context(|| format!("reading {} as HTTP binding YAML", path.display()))?
    };
    match ess_http::compile(&spec, ir) {
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

pub fn run(command: &Command) -> Result<ExitCode> {
    let (input, out) = match command {
        Command::Validate(input) => (input, None),
        Command::Compile { input, out } => (input, out.as_deref()),
    };
    let Ok((ir, _)) = resolved(&input.path, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let Some(http) = load(&input.binding, &ir, input.format)? else {
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
