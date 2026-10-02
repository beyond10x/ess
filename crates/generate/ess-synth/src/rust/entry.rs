//! Executables that opt in to an ephemeral realization through explicit CLI flags.
use super::{behaviour::Uses, context, http, layout::Layout, name};
use crate::plan::{SynthesisPlan, REGENERATE};
use ess_compiler::ir::EssIr;
use ess_gen::Artifact;
use std::fmt::Write as _;

pub(super) fn binaries(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    uses: &Uses,
) -> Vec<Artifact> {
    let mut refusals = context::refusals(ir, layout, uses);
    refusals.extend(
        plan.obligations()
            .map(|(capability, _)| format!("{:?} `{}`", capability.kind, capability.source)),
    );
    let mut artifacts = Vec::new();
    for component in http::served(ir) {
        let mut out = plan.provenance.commented_for("//", REGENERATE);
        out.push_str(FLAGS);
        out.push_str("\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    let args = <Arguments as clap::Parser>::parse();\n");
        if !refusals.is_empty() {
            let _ = writeln!(
                out,
                "    let _ = args;\n    Err({:?}.into())\n}}",
                format!(
                    "generated entry point requires a realization: {}",
                    refusals.join("; ")
                )
            );
            artifacts.push(Artifact::new(
                format!(
                    "crates/{}/src/bin/{}-server.rs",
                    layout.server_package(),
                    component.name
                ),
                out,
            ));
            continue;
        }
        {
            let types = Layout::crate_ident(layout.package());
            let system = Layout::crate_ident(layout.system_package());
            let server = Layout::crate_ident(layout.server_package());
            let bundle = if super::behaviour::used(ir) {
                let _ = writeln!(
                    out,
                    "    let ports = {types}::memory::InMemoryPorts::default();"
                );
                format!("{types}::behaviour::Generated::new(ports.clone())")
            } else {
                "()".to_owned()
            };
            let ports: Vec<_> = ir
                .components()
                .keys()
                .map(|component| {
                    format!(
                        "{}::{}::new({bundle})",
                        Layout::crate_ident(layout.component_package(component)),
                        name::pascal(&component.to_string())
                    )
                })
                .collect();
            let _ = writeln!(
                out,
                "    let mut system = {system}::System::new({});",
                ports.join(", ")
            );
            let authentication = if ess_gen::http::checks_grants(ir) {
                let _ = writeln!(out, "    let authenticate = |request: &{server}::http::Request| {{\n        if !matches!(args.callers, CallerMode::ActorHeader) {{ return None; }}\n        let value = request.headers.iter().find(|(name, _)| name.eq_ignore_ascii_case(\"authorization\"))?.1.strip_prefix(\"Actor \")?;\n        let actors: Vec<_> = {types}::actor::Actor::ALL.iter().copied().filter(|actor| actor.name() == value || actor.name().rsplit('.').next() == Some(value)).collect();\n        if actors.len() == 1 {{ Some({types}::actor::Caller {{ actor: actors[0] }}) }} else {{ None }}\n    }};");
                ", authenticate"
            } else {
                ""
            };
            out.push_str("    eprintln!(\"generated entry: ephemeral storage; callers={}{}\", if matches!(args.callers, CallerMode::ActorHeader) { \"actor-header\" } else { \"none\" }, if matches!(args.callers, CallerMode::ActorHeader) { \" (demonstration mode; not authentication)\" } else { \"\" });\n");
            let _ = writeln!(out, "    {server}::{}::serve_with_static(&mut system, &args.listen{authentication}, args.static_directory.as_deref())?;\n    Ok(())\n}}", name::value_ident(&component.name.to_string()));
        }
        artifacts.push(Artifact::new(
            format!(
                "crates/{}/src/bin/{}-server.rs",
                layout.server_package(),
                component.name
            ),
            out,
        ));
    }
    artifacts
}

const FLAGS: &str = r#"
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
"#;
