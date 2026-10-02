//! The Rust target's opt-in single-crate layout (`story:single-crate-rust-layout`).
//!
//! `--layout crate` writes one crate at the output root instead of a workspace: a module per
//! bounded context, the component ports under `ports`, the bindings in `system`, and the HTTP
//! surface under `server`, behind a `server` Cargo feature that is off by default. The default
//! layout's bytes do not move. The conformance suite run against the single-crate output is in
//! `tests/declared_behaviour.rs`, beside the workspace run it mirrors.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{
    synthesize_for, synthesize_laid_out, OutputLayout, Synthesis, SynthesisFailure, Target,
};

/// An example directory, compiled where it lives.
fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name);
    let mut labels = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                labels.push(
                    path.strip_prefix(&base)
                        .expect("inside the example")
                        .display()
                        .to_string(),
                );
            }
        }
    }
    labels.sort();
    let documents: Vec<(String, String)> = labels
        .iter()
        .map(|label| {
            (
                label.clone(),
                std::fs::read_to_string(base.join(label)).expect("readable"),
            )
        })
        .collect();
    compile(&documents)
}

/// Named YAML documents, compiled.
fn compile(documents: &[(String, String)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("`{label}` is well formed: {error}"));
        sources.insert(label.clone(), text.clone());
        labels.push(label.clone());
        parsed.push((Source::new(label.clone()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

/// Three bounded contexts whose module names are exactly the three the single crate adds at its
/// root — `system`, `ports` and `server` — one of them using `Json`, served over the network so
/// the HTTP surface and its codecs are emitted too.
fn reserved_names() -> EssIr {
    let documents = [
        (
            "system.yaml",
            "format: ess/15
system: demo
version: v1
domains: [demo.system, demo.ports, demo.server]
components:
  - component: msgs-service
    owns: {domains: [demo.system, demo.ports, demo.server]}
    accepts: {commands: [demo.system.Send]}
    publishes: {events: [demo.system.Sent]}
    reached_by: network
",
        ),
        (
            "domains/system.yaml",
            "domain: demo.system
types:
  - {name: demo.system.Body, kind: newtype, of: Json}
events:
  - name: demo.system.Sent
    fields:
      - {name: body, type: demo.system.Body}
      - {name: host, type: demo.server.Host}
commands:
  - name: demo.system.Send
    input:
      - {name: body, type: demo.system.Body}
      - {name: host, type: demo.server.Host}
      - {name: port, type: demo.ports.Port}
    outcomes:
      - name: sent
        emits: [demo.system.Sent]
        payload:
          demo.system.Sent: {body: input.body, host: input.host}
",
        ),
        (
            "domains/ports.yaml",
            "domain: demo.ports
types:
  - {name: demo.ports.Port, kind: newtype, of: Integer}
",
        ),
        (
            "domains/server.yaml",
            "domain: demo.server
types:
  - {name: demo.server.Host, kind: newtype, of: String}
",
        ),
    ];
    compile(
        &documents
            .iter()
            .map(|(label, text)| ((*label).to_owned(), (*text).to_owned()))
            .collect::<Vec<_>>(),
    )
}

/// Every model this file lays out as one crate, by name.
fn models() -> Vec<(&'static str, EssIr)> {
    vec![
        ("gatepass", example("gatepass")),
        ("billing", example("billing")),
        ("demo", reserved_names()),
    ]
}

fn crate_layout(ir: &EssIr) -> Synthesis {
    synthesize_laid_out(ir, Target::Rust, OutputLayout::Crate)
        .unwrap_or_else(|failure| panic!("the model synthesizes as one crate: {failure}"))
}

/// A path's extension.
fn extension(path: &str) -> Option<&str> {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
}

/// Sources that belong to the HTTP surface: `src/server.rs` and everything under `src/server/`.
fn is_server(path: &str) -> bool {
    path == "src/server.rs" || path.starts_with("src/server/")
}

#[test]
fn the_workspace_layout_is_the_default_and_unchanged() {
    for (name, ir) in models() {
        let default = synthesize_for(&ir, Target::Rust).expect("synthesizes");
        let workspace =
            synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace).expect("synthesizes");
        assert_eq!(
            default.artifacts.keys().collect::<Vec<_>>(),
            workspace.artifacts.keys().collect::<Vec<_>>(),
            "{name}"
        );
        for (path, artifact) in &default.artifacts {
            assert_eq!(
                artifact.contents, workspace.artifacts[path].contents,
                "{name}: {path}"
            );
        }
        let plan = &default.artifacts["plan.json"].contents;
        assert!(!plan.contains("\"layout\""), "{name}: {plan}");
    }
}

#[test]
fn the_crate_layout_is_one_crate_at_most_two_levels_deep() {
    for (name, ir) in models() {
        let synthesis = crate_layout(&ir);
        for path in synthesis.artifacts.keys() {
            assert!(
                path.split('/').count() <= 3,
                "{name}: `{path}` is deeper than `src/<module>/<file>`"
            );
            assert!(!path.starts_with("crates/"), "{name}: `{path}`");
        }
        for required in ["Cargo.toml", "src/lib.rs", "src/ports.rs", "src/system.rs"] {
            assert!(
                synthesis.artifacts.contains_key(required),
                "{name}: no `{required}` in {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            );
        }
        for component in ir.components().keys() {
            let module = component.to_string().replace('-', "_");
            assert!(
                synthesis
                    .artifacts
                    .contains_key(&format!("src/ports/{module}.rs")),
                "{name}: `{component}` has no port module"
            );
        }
        let manifest = &synthesis.artifacts["Cargo.toml"].contents;
        assert!(
            manifest.contains(&format!("\nname = \"{name}\"\n")),
            "{manifest}"
        );
        assert!(!manifest.contains("members"), "{manifest}");
        let served = synthesis.artifacts.keys().any(|path| is_server(path));
        assert_eq!(
            manifest.contains("\n[features]\nserver = [\"dep:clap\", \"memory\"]\n"),
            served,
            "{name}: the `server` feature exists exactly when there is an HTTP surface:\n{manifest}"
        );
        if served {
            for module in ["http", "wire", "entry", "json"] {
                assert!(
                    synthesis
                        .artifacts
                        .contains_key(&format!("src/server/{module}.rs")),
                    "{name}: no `src/server/{module}.rs`"
                );
            }
            let lib = &synthesis.artifacts["src/lib.rs"].contents;
            assert!(
                lib.contains("#[cfg(feature = \"server\")]\npub mod server;\n"),
                "{lib}"
            );
        }
    }
}

#[test]
fn a_bounded_context_named_like_a_root_module_moves_aside_only_in_the_crate_layout() {
    let ir = reserved_names();
    let synthesis = crate_layout(&ir);
    for module in ["system_domain", "ports_domain", "server_domain"] {
        assert!(
            synthesis
                .artifacts
                .contains_key(&format!("src/{module}.rs")),
            "no `src/{module}.rs` in {:?}",
            synthesis.artifacts.keys().collect::<Vec<_>>()
        );
    }
    let workspace = synthesize_for(&ir, Target::Rust).expect("synthesizes");
    for module in ["system", "ports", "server"] {
        assert!(
            workspace
                .artifacts
                .contains_key(&format!("crates/demo-types/src/{module}.rs")),
            "the workspace layout keeps `{module}`"
        );
    }
}

#[test]
fn the_plan_names_the_crate_layout() {
    let ir = example("gatepass");
    let synthesis = crate_layout(&ir);
    let plan: serde_json::Value =
        serde_json::from_str(&synthesis.artifacts["plan.json"].contents).expect("plan.json");
    assert_eq!(plan["scope"]["layout"], "crate", "{plan}");
    let markdown = &synthesis.artifacts["PLAN.md"].contents;
    assert!(
        markdown.contains("Regenerate with `ess synthesize --layout crate`."),
        "{markdown}"
    );
    for (path, artifact) in &synthesis.artifacts {
        if matches!(extension(path), Some("rs" | "toml")) {
            assert!(
                artifact
                    .contents
                    .contains("regenerate with `ess synthesize --layout crate`"),
                "{path} names the command that rewrites it"
            );
        }
    }
}

#[test]
fn every_other_target_refuses_the_crate_layout_by_name() {
    let ir = example("gatepass");
    for target in [Target::Go, Target::Web, Target::Clap] {
        let Err(SynthesisFailure::Layout(refusal)) =
            synthesize_laid_out(&ir, target, OutputLayout::Crate)
        else {
            panic!("`{}` must refuse `--layout crate`", target.name());
        };
        assert_eq!(refusal.target(), target.name());
        let message = refusal.to_string();
        assert!(
            message.contains(&format!("`{}`", target.name())) && message.contains("crate"),
            "{message}"
        );
    }
}

// ---- built -----------------------------------------------------------------------------------

/// A scratch directory under this test binary's target, removed when dropped.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs cargo offline in `directory` with warnings denied.
fn cargo(directory: &Path, target: &Path, arguments: &[&str]) -> (bool, String) {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .arg("--offline")
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", target)
        .env("RUSTFLAGS", "-D warnings")
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .output()
        .expect("cargo runs");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

#[test]
fn without_its_server_feature_the_crate_uses_no_std_net_and_builds_with_warnings_denied() {
    let scratch = Scratch(
        Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("single-crate-layout-{}", std::process::id())),
    );
    let _ = std::fs::remove_dir_all(&scratch.0);
    let target = scratch.0.join("target");
    for (name, ir) in models() {
        let synthesis = crate_layout(&ir);
        for (path, artifact) in &synthesis.artifacts {
            if extension(path) == Some("rs") && !is_server(path) {
                assert!(
                    !artifact.contents.contains("std::net"),
                    "{name}: `{path}` uses `std::net` outside the `server` feature"
                );
            }
        }
        let tree = scratch.0.join(name);
        for artifact in synthesis.artifacts.values() {
            let path = tree.join(&artifact.path);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
            std::fs::write(&path, &artifact.contents).expect("write");
        }
        let (built, log) = cargo(&tree, &target, &["check", "--all-targets"]);
        assert!(
            built,
            "{name}: the crate builds without features, warnings denied:\n{log}"
        );
        if synthesis.artifacts.keys().any(|path| is_server(path)) {
            let (built, log) = cargo(
                &tree,
                &target,
                &["check", "--all-targets", "--features", "server"],
            );
            assert!(
                built,
                "{name}: the crate builds with `server`, warnings denied:\n{log}"
            );
        }
    }
    drop(scratch);
}
