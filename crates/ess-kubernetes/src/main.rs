//! Standalone credential-edge entry point for the ESS Kubernetes adapter.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ess-kubernetes", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List kubeconfig contexts available to the adapter.
    Contexts,
    /// Scan one cluster into a sanitized `infra-observation/1` bundle.
    Scan {
        /// Kubeconfig context; the current context when omitted.
        #[arg(long)]
        context: Option<String>,
        /// Destination for the sanitized observation.
        #[arg(long)]
        out: std::path::PathBuf,
    },
}

fn main() -> std::process::ExitCode {
    let result = match Cli::parse().command {
        Command::Contexts => ess_kubernetes::contexts(),
        Command::Scan { context, out } => ess_kubernetes::scan(context.as_deref(), &out),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}
