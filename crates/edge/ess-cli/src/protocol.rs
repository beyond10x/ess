//! Explicit experimental protocol sidecar commands. Existing projection routes stay unchanged.
use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use ess_compiler::protocol::{parse_and_compile, CompiledProtocol};
use ess_conformance::protocol::{check_trace, explore, simulate, Action, Trace, Verdict};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Subcommand)]
pub(crate) enum Specify {
    /// Validate an experimental ess-protospec/1 protocol document.
    Validate {
        #[arg(long)]
        path: PathBuf,
    },
    /// Write canonical admitted protocol JSON (create-new output only).
    Compile {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}
#[derive(Debug, Subcommand)]
pub(crate) enum Verify {
    /// Simulate an unambiguous action sequence; this is model evidence, not target conformance.
    Run {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        actions: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Check an exact ess-prototrace/1 observation record against its model.
    Replay {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        trace: PathBuf,
    },
    /// Explore bounded schedules with optional finite typed input witnesses.
    Explore {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        inputs: Option<PathBuf>,
    },
}
fn model(path: &Path) -> Result<CompiledProtocol> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    parse_and_compile(&text).map_err(|errors| {
        anyhow!(
            "{}",
            errors
                .iter()
                .map(|e| format!("{} [{}]: {}", e.path, e.code, e.message))
                .collect::<Vec<_>>()
                .join("\n")
        )
    })
}
fn write(bytes: &str, path: Option<&Path>) -> Result<()> {
    match path {
        Some(path) => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .with_context(|| format!("create new output {}", path.display()))?;
            file.write_all(bytes.as_bytes())?;
        }
        None => {
            std::io::stdout().lock().write_all(bytes.as_bytes())?;
        }
    }
    Ok(())
}
fn json<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)? + "\n")
}
fn exit(verdict: Verdict) -> ExitCode {
    match verdict {
        Verdict::Passed => ExitCode::SUCCESS,
        Verdict::Failed => ExitCode::from(1),
        Verdict::Inconclusive => ExitCode::from(2),
    }
}
pub(crate) fn specify(command: &Specify) -> Result<ExitCode> {
    match command {
        Specify::Validate { path } => {
            let model = model(path)?;
            println!(
                "{} — valid {} ({})",
                model.model().name,
                model.model().format,
                model.digest()
            );
        }
        Specify::Compile { path, out } => {
            let model = model(path)?;
            write(model.to_canonical_json(), out.as_deref())?;
        }
    }
    Ok(ExitCode::SUCCESS)
}
pub(crate) fn verify(command: &Verify) -> Result<ExitCode> {
    match command {
        Verify::Run { path, actions, out } => {
            let model = model(path)?;
            let actions: Vec<Action> = serde_json::from_str(&fs::read_to_string(actions)?)?;
            let trace = simulate(&model, &actions)?;
            write(&json(&trace)?, out.as_deref())?;
            eprintln!(
                "model simulation only; {} steps; complete={}",
                trace.steps.len(),
                trace.complete
            );
            Ok(exit(check_trace(&model, &trace).verdict))
        }
        Verify::Replay { path, trace } => {
            let model = model(path)?;
            let trace: Trace = serde_json::from_str(&fs::read_to_string(trace)?)?;
            let report = check_trace(&model, &trace);
            write(&json(&report)?, None)?;
            Ok(exit(report.verdict))
        }
        Verify::Explore { path, inputs } => {
            let model = model(path)?;
            let inputs: Vec<Action> = inputs
                .as_ref()
                .map(|path| -> Result<_> { Ok(serde_json::from_str(&fs::read_to_string(path)?)?) })
                .transpose()?
                .unwrap_or_default();
            let report = explore(&model, &inputs);
            write(&json(&report)?, None)?;
            Ok(exit(report.verdict))
        }
    }
}
