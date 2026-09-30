//! Stands in for `ess ui docs` until the `ess` binary wires [`ess_ui_docs::run`]:
//! `cargo run -p ess-ui-docs --example render -- --format md --out website/docs/reference/ess-ui.md`.

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    docs: ess_ui_docs::DocsArgs,
}

fn main() {
    match ess_ui_docs::run(&Cli::parse().docs) {
        Ok(line) => println!("{line}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
