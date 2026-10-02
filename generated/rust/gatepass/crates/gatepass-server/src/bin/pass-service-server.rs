// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

#[derive(clap::ValueEnum, Clone, Copy)]
enum CallerMode { None, ActorHeader }

#[derive(clap::Parser)]
#[command(about = "Generated ephemeral demonstration server")]
struct Arguments {
    #[arg(long, default_value = "127.0.0.1:8080")]
    listen: String,
    #[arg(long, value_enum, default_value = "none")]
    callers: CallerMode,
    #[arg(long = "static")]
    static_directory: Option<std::path::PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Arguments as clap::Parser>::parse();
    let _ = args;
    Err("generated entry point requires a realization: CommandBehavior `gatepass.visit.RegisterVisit`".into())
}
