// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

//! Ephemeral generated server. Durable storage and production authentication remain ports.
use clap::Parser;

#[derive(Parser)]
struct Options {
#[arg(long, default_value = "127.0.0.1:8080")]
listen: String,
#[arg(long, default_value = "none", value_parser = ["none", "actor-header"])]
callers: String,
#[arg(long = "static")]
static_root: Option<std::path::PathBuf>,
}

fn main() -> std::process::ExitCode {
let options = Options::parse();
let _ = options;
eprintln!("unmet startup obligations:\ncommand behaviour: gatepass.visit.RegisterVisit");
std::process::ExitCode::FAILURE
}
