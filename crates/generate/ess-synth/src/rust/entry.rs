//! Generated executable wiring, with explicit caller policy and named startup refusals.
use super::{behaviour, context, layout::Layout, name};
use crate::{
    plan::{SynthesisPlan, REGENERATE},
    served::Reachable,
};
use ess_compiler::ir::{EssIr, ResolvedComponent};
use ess_gen::Artifact;
use std::fmt::Write as _;

pub(super) fn binary(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    component: &ResolvedComponent,
) -> Artifact {
    let reachable = Reachable::of(ir, component);
    let mut unmet = reachable.obligations(plan);
    unmet.extend(context::unmet(
        ir,
        &behaviour::requirements(ir, plan, layout, Some(&reachable)),
    ));
    let server = Layout::crate_ident(layout.server_package());
    let types = Layout::crate_ident(layout.package());
    let system = Layout::crate_ident(layout.system_package());
    let module = name::value_ident(&component.name.to_string());
    let mut out = plan.provenance.commented_for("//", REGENERATE);
    out.push_str("\n//! Ephemeral generated server. Durable storage and production authentication remain ports.\nuse clap::Parser;\n\n#[derive(Parser)]\nstruct Options {\n#[arg(long, default_value = \"127.0.0.1:8080\")]\nlisten: String,\n#[arg(long, default_value = \"none\", value_parser = [\"none\", \"actor-header\"])]\ncallers: String,\n#[arg(long = \"static\")]\nstatic_root: Option<std::path::PathBuf>,\n}\n\nfn main() -> std::process::ExitCode {\nlet options = Options::parse();\n");
    if unmet.is_empty() {
        let _ = writeln!(out, "let ports = {server}::memory::MemoryPorts::default();");
        let mut arguments = Vec::new();
        for name in ir.components().keys() {
            let package = Layout::crate_ident(layout.component_package(name));
            let port = super::name::pascal(&name.to_string());
            let behavior = if behaviour::used(ir) {
                format!("{types}::behaviour::Generated::new(ports.clone())")
            } else {
                format!("{server}::memory::MemoryPorts::default()")
            };
            arguments.push(format!("{package}::{port}::new({behavior})"));
        }
        if super::system::has_obligations(ir, plan) {
            arguments.push(format!("{system}::obligations::Unimplemented"));
        }
        let _ = writeln!(out, "let mut system = {system}::System::new({});\nlet mode = options.callers;\neprintln!(\"{{}}\", if mode == \"actor-header\" {{ r#\"{{\"format\":\"ess/1\",\"callers\":\"actor-header\",\"demonstration\":true}}\"# }} else {{ r#\"{{\"format\":\"ess/1\",\"callers\":\"none\"}}\"# }});", arguments.join(", "));
        let auth = if ess_gen::http::checks_grants(ir) {
            let mut counts = std::collections::BTreeMap::new();
            for actor in ir.actors().keys() {
                *counts
                    .entry(actor.to_string().rsplit('.').next().unwrap().to_owned())
                    .or_insert(0_usize) += 1;
            }
            let mut aliases = String::new();
            for actor in ir.actors().keys() {
                let full = actor.to_string();
                let short = full.rsplit('.').next().unwrap();
                if counts[short] == 1 {
                    let _ = writeln!(aliases, "{short:?} => {full:?},");
                }
            }
            format!(", |request| {{ if mode != \"actor-header\" {{ return None; }} let header = request.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(\"Authorization\"))?.1.strip_prefix(\"Actor \")?; let actor = match header {{ {aliases} other => other }}; {types}::actor::Actor::ALL.iter().find(|candidate| candidate.name() == actor).map(|actor| {types}::actor::Caller {{ actor: *actor }}) }}")
        } else {
            String::new()
        };
        let _ = writeln!(out, "match {server}::{module}::serve_with_static(&mut system, &options.listen{auth}, options.static_root.as_deref()) {{ Ok(()) => std::process::ExitCode::SUCCESS, Err(error) => {{ eprintln!(\"{{error}}\"); std::process::ExitCode::FAILURE }} }}\n}}");
    } else {
        let message = format!(
            "unmet startup obligations:\n{}",
            unmet.into_iter().collect::<Vec<_>>().join("\n")
        );
        let _ = writeln!(
            out,
            "let _ = options;\neprintln!({message:?});\nstd::process::ExitCode::FAILURE\n}}"
        );
    }
    Artifact::new(
        format!(
            "crates/{}/src/bin/{}-server.rs",
            layout.server_package(),
            component.name
        ),
        out,
    )
}

/// Binary-safe static files, constrained to the canonical selected directory.
pub(super) const STATIC: &str = r#"
//! Static fallback for paths outside a served route table.
use std::io::Write as _;

pub(crate) fn answer(stream: &mut std::net::TcpStream, root: &std::path::Path, request: &crate::http::Request) -> std::io::Result<()> {
    if request.method != "GET" && request.method != "HEAD" {
        return crate::http::write(stream, &crate::http::Response::new(405, "text/plain", "method not allowed"));
    }
    let Some(path) = resolve(root, &request.path) else {
        return crate::http::write(stream, &crate::http::Response::new(404, "text/plain", "not found"));
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return crate::http::write(stream, &crate::http::Response::new(404, "text/plain", "not found"));
    };
    let mime = match path.extension().and_then(|value| value.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8", "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8", "json" => "application/json",
        "wasm" => "application/wasm", "svg" => "image/svg+xml", "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg", "ico" => "image/x-icon", _ => "application/octet-stream",
    };
    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len())?;
    if request.method != "HEAD" { stream.write_all(&bytes)?; }
    stream.flush()
}

fn resolve(root: &std::path::Path, raw: &str) -> Option<std::path::PathBuf> {
    let mut bytes = Vec::new();
    let mut input = raw.as_bytes().iter().copied();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = char::from(input.next()?).to_digit(16)?;
            let low = char::from(input.next()?).to_digit(16)?;
            bytes.push(u8::try_from(high * 16 + low).ok()?);
        } else { bytes.push(byte); }
    }
    let decoded = std::str::from_utf8(&bytes).ok()?;
    if decoded.contains('\\') || decoded.contains('\0') { return None; }
    let relative = std::path::Path::new(decoded.strip_prefix('/')?);
    if relative.components().any(|component| !matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir)) { return None; }
    let mut path = root.join(relative).canonicalize().ok()?;
    if path.is_dir() { path = path.join("index.html").canonicalize().ok()?; }
    (path.starts_with(root) && path.is_file()).then_some(path)
}
"#;
