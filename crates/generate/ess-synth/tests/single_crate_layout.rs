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
    synthesize_crate, synthesize_for, synthesize_laid_out, CrateManifest, OutputLayout, Synthesis,
    SynthesisFailure, Target,
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

/// A consumer's model crate: one bounded context with a newtype, an enum, two structs (one with an
/// `any:` invariant) and a provided entity with a one-state lifecycle, and no component, so no
/// server is generated.
fn shop() -> EssIr {
    let documents = [
        (
            "system.yaml",
            "format: ess/37
system: shop
version: v1
domains: [shop.catalog]
",
        ),
        (
            "domains/catalog.yaml",
            "domain: shop.catalog
types:
  - {name: shop.catalog.ItemId, kind: newtype, of: String}
  - {name: shop.catalog.Mode, kind: enum, variants: [enforce, report]}
  - name: shop.catalog.Limit
    kind: struct
    fields:
      - {name: min, type: Optional<Decimal>}
      - {name: max, type: Optional<Decimal>}
    invariants:
      - any: [\"defined(min)\", \"defined(max)\"]
  - name: shop.catalog.Entry
    kind: struct
    fields:
      - {name: item, type: shop.catalog.ItemId}
      - {name: mode, type: shop.catalog.Mode}
      - {name: limit, type: shop.catalog.Limit}
      - {name: tags, type: List<String>}
      - {name: labels, type: \"Optional<Map<String, String>>\"}
entities:
  - name: shop.catalog.Catalog
    provided: true
    identity: {name: project, type: String}
    fields:
      - {name: format, type: String}
      - {name: entries, type: List<shop.catalog.Entry>}
    lifecycle:
      initial: Declared
      states: [Declared]
      terminal: [Declared]
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

/// The crate layout with the consumer's manifest options.
fn manifested(ir: &EssIr, manifest: &CrateManifest) -> Result<Synthesis, SynthesisFailure> {
    synthesize_crate(ir, Target::Rust, OutputLayout::Crate, manifest, None)
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
            manifest.contains("\n[features]\nserver = [\"dep:clap\", \"dep:uuid\", \"dep:time\"]\n"),
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

/// Every model of this file and the consumer's library model.
fn all_models() -> Vec<(&'static str, EssIr)> {
    let mut all = models();
    all.push(("shop", shop()));
    all
}

#[test]
fn without_the_manifest_options_the_crate_is_byte_for_byte_as_before() {
    for (name, ir) in all_models() {
        let before = crate_layout(&ir);
        let unnamed = manifested(&ir, &CrateManifest::default()).expect("synthesizes");
        assert_eq!(
            before.artifacts.keys().collect::<Vec<_>>(),
            unnamed.artifacts.keys().collect::<Vec<_>>(),
            "{name}"
        );
        for (path, artifact) in &before.artifacts {
            assert_eq!(
                artifact.contents, unnamed.artifacts[path].contents,
                "{name}: {path}"
            );
        }
        let manifest = &before.artifacts["Cargo.toml"].contents;
        assert!(
            manifest.contains(&format!("\nname = \"{name}\"\n")),
            "{manifest}"
        );
        assert!(manifest.ends_with("\n[workspace]\n"), "{manifest}");
        let plan = &before.artifacts["plan.json"].contents;
        assert!(
            !plan.contains("\"package\"") && !plan.contains("\"no_workspace_table\""),
            "{name}: {plan}"
        );
    }
}

#[test]
fn the_package_is_named_by_its_consumer() {
    let mut used = false;
    let options = CrateManifest {
        package: Some("consumer-model".to_owned()),
        no_workspace_table: false,
    };
    for (name, ir) in all_models() {
        let synthesis = manifested(&ir, &options).expect("synthesizes");
        let manifest = &synthesis.artifacts["Cargo.toml"].contents;
        assert!(
            manifest.contains("\n[package]\nname = \"consumer-model\"\n"),
            "{name}: {manifest}"
        );
        assert!(manifest.ends_with("\n[workspace]\n"), "{name}: {manifest}");
        let plan: serde_json::Value =
            serde_json::from_str(&synthesis.artifacts["plan.json"].contents).expect("plan.json");
        assert_eq!(plan["scope"]["package"], "consumer-model", "{name}: {plan}");
        assert!(
            plan["scope"].get("no_workspace_table").is_none(),
            "{name}: {plan}"
        );
        let command = "ess synthesize --layout crate --package consumer-model";
        assert!(
            synthesis.artifacts["PLAN.md"]
                .contents
                .contains(&format!("Regenerate with `{command}`.")),
            "{name}"
        );
        for (path, artifact) in &synthesis.artifacts {
            if matches!(extension(path), Some("rs" | "toml")) {
                assert!(
                    artifact
                        .contents
                        .contains(&format!("regenerate with `{command}`")),
                    "{name}: {path} names the command that rewrites it"
                );
            }
            if path.starts_with("src/bin/") {
                let before = &crate_layout(&ir).artifacts[path].contents;
                assert_eq!(
                    artifact.contents,
                    before
                        .replace(&format!("{name}::"), "consumer_model::")
                        .replace("`ess synthesize --layout crate`", &format!("`{command}`")),
                    "{name}: {path} uses the library by the package's name"
                );
                used |= before.contains(&format!("{name}::"));
            }
        }
    }
    assert!(used, "a server binary that uses the library is covered");
}

#[test]
fn the_workspace_table_is_left_out_on_request() {
    let options = CrateManifest {
        package: None,
        no_workspace_table: true,
    };
    for (name, ir) in all_models() {
        let before = crate_layout(&ir);
        let synthesis = manifested(&ir, &options).expect("synthesizes");
        let manifest = &synthesis.artifacts["Cargo.toml"].contents;
        assert!(!manifest.contains("[workspace]"), "{name}: {manifest}");
        let command = "ess synthesize --layout crate --no-workspace-table";
        assert_eq!(
            format!("{manifest}\n[workspace]\n"),
            before.artifacts["Cargo.toml"]
                .contents
                .replace("`ess synthesize --layout crate`", &format!("`{command}`")),
            "{name}: only the table and the command differ"
        );
        let plan: serde_json::Value =
            serde_json::from_str(&synthesis.artifacts["plan.json"].contents).expect("plan.json");
        assert_eq!(plan["scope"]["no_workspace_table"], true, "{name}: {plan}");
        assert!(plan["scope"].get("package").is_none(), "{name}: {plan}");
        assert!(
            synthesis.artifacts["PLAN.md"]
                .contents
                .contains(&format!("Regenerate with `{command}`.")),
            "{name}"
        );
    }
    let both = manifested(
        &shop(),
        &CrateManifest {
            package: Some("shop_model".to_owned()),
            no_workspace_table: true,
        },
    )
    .expect("synthesizes");
    assert!(both.artifacts["PLAN.md"].contents.contains(
        "Regenerate with `ess synthesize --layout crate --package shop_model --no-workspace-table`."
    ));
}

#[test]
fn the_manifest_options_are_refused_outside_the_crate_layout_and_for_a_bad_name() {
    let ir = shop();
    for (option, manifest) in [
        (
            "--package",
            CrateManifest {
                package: Some("shop-model".to_owned()),
                no_workspace_table: false,
            },
        ),
        (
            "--no-workspace-table",
            CrateManifest {
                package: None,
                no_workspace_table: true,
            },
        ),
    ] {
        let Err(SynthesisFailure::Manifest(refusal)) =
            synthesize_crate(&ir, Target::Rust, OutputLayout::Workspace, &manifest, None)
        else {
            panic!("the workspace layout must refuse `{option}`");
        };
        let message = refusal.to_string();
        assert!(
            message.contains(&format!("`{option}`")) && message.contains("`--layout crate`"),
            "{message}"
        );
        let Err(SynthesisFailure::Layout(_)) =
            synthesize_crate(&ir, Target::Go, OutputLayout::Crate, &manifest, None)
        else {
            panic!("`go` refuses the crate layout before `{option}`");
        };
    }
    for bad in [
        "",
        "Shop",
        "1shop",
        "-shop",
        "shop model",
        "shop.model",
        "crate",
        "std",
        "match",
        "clap",
    ] {
        let Err(SynthesisFailure::Manifest(refusal)) = manifested(
            &ir,
            &CrateManifest {
                package: Some(bad.to_owned()),
                no_workspace_table: false,
            },
        ) else {
            panic!("`{bad}` is not a package name the crate can take");
        };
        let message = refusal.to_string();
        assert!(
            message.contains(&format!("`{bad}`")) && message.contains("--package"),
            "{message}"
        );
    }
    manifested(
        &ir,
        &CrateManifest {
            package: Some("x".repeat(64)),
            no_workspace_table: false,
        },
    )
    .expect("64 characters is a package name");
    assert!(matches!(
        manifested(
            &ir,
            &CrateManifest {
                package: Some("x".repeat(65)),
                no_workspace_table: false,
            },
        ),
        Err(SynthesisFailure::Manifest(_))
    ));
}

/// The `pub fn` lines of a module: each codec's name and signature.
fn functions(module: &str) -> Vec<&str> {
    module
        .lines()
        .filter(|line| line.starts_with("pub fn "))
        .collect()
}

/// Codecs that belong to a served component's commands and views.
const SERVED_ONLY: [&str; 5] = [
    "pub fn encode_command_",
    "pub fn decode_command_",
    "pub fn encode_outcome_",
    "pub fn encode_view_",
    "pub fn decode_params_",
];

#[test]
fn a_crate_with_no_component_carries_its_codecs_at_the_root() {
    let synthesis = crate_layout(&shop());
    for path in ["src/wire.rs", "src/json.rs"] {
        assert!(
            synthesis.artifacts.contains_key(path),
            "no `{path}` in {:?}",
            synthesis.artifacts.keys().collect::<Vec<_>>()
        );
    }
    assert!(
        !synthesis.artifacts.keys().any(|path| is_server(path)),
        "no server without a served component"
    );
    let lib = &synthesis.artifacts["src/lib.rs"].contents;
    assert!(
        lib.contains("\npub mod json;\n") && lib.contains("\npub mod wire;\n"),
        "{lib}"
    );
    let manifest = &synthesis.artifacts["Cargo.toml"].contents;
    assert!(!manifest.contains("[features]"), "{manifest}");
    let wire = &synthesis.artifacts["src/wire.rs"].contents;
    assert!(wire.contains("\nuse crate::json;\n"), "{wire}");
    for (declared, rust) in [
        ("item_id", "ItemId"),
        ("mode", "Mode"),
        ("limit", "Limit"),
        ("entry", "Entry"),
    ] {
        for function in [
            format!(
                "pub fn encode_shop_catalog_{declared}(value: &crate::catalog::{rust}, out: &mut \
                 String) {{"
            ),
            format!(
                "pub fn decode_shop_catalog_{declared}(value: &json::Value, at: &str) -> \
                 Result<crate::catalog::{rust}, json::DecodeError> {{"
            ),
        ] {
            assert!(
                functions(wire).contains(&function.as_str()),
                "no `{function}` in:\n{wire}"
            );
        }
    }
    for served in SERVED_ONLY {
        assert!(!wire.contains(served), "`{served}` needs a server:\n{wire}");
    }
}

#[test]
fn a_crate_with_components_and_no_server_carries_no_command_or_view_codec() {
    let synthesis = crate_layout(&example("billing"));
    assert!(!synthesis.artifacts.keys().any(|path| is_server(path)));
    let wire = &synthesis.artifacts["src/wire.rs"].contents;
    assert!(
        functions(wire)
            .iter()
            .any(|line| line.starts_with("pub fn decode_billing_")),
        "{wire}"
    );
    for served in SERVED_ONLY {
        assert!(!wire.contains(served), "`{served}` needs a server:\n{wire}");
    }
}

#[test]
fn a_served_crate_keeps_one_wire_module_and_every_codec_by_name_and_signature() {
    let mut served_models = 0;
    for (name, ir) in models() {
        let workspace = synthesize_for(&ir, Target::Rust).expect("synthesizes");
        let Some(before) = workspace
            .artifacts
            .get(&format!("crates/{name}-server/src/wire.rs"))
        else {
            continue;
        };
        served_models += 1;
        let synthesis = crate_layout(&ir);
        let root = &synthesis.artifacts["src/wire.rs"].contents;
        let server = &synthesis.artifacts["src/server/wire.rs"].contents;
        assert!(
            server.contains("\npub use crate::wire::*;\n"),
            "{name}: {server}"
        );
        assert!(
            synthesis.artifacts["src/server/json.rs"]
                .contents
                .contains("\npub use crate::json::*;\n"),
            "{name}"
        );
        let mut kept = functions(root);
        kept.extend(functions(server));
        for function in functions(&before.contents) {
            // A bounded context named like a root module moves aside in this layout (`demo`).
            let mut moved = function.to_owned();
            for module in ["json", "ports", "server", "system", "wire"] {
                moved = moved.replace(
                    &format!("{name}_types::{module}::"),
                    &format!("crate::{module}_domain::"),
                );
            }
            let moved = moved
                .replace(&format!("{name}_types::"), "crate::")
                .replace(&format!("{name}_system::"), "crate::system::");
            assert!(
                kept.contains(&moved.as_str()),
                "{name}: `{function}` lost its name or signature"
            );
        }
        for function in functions(server) {
            assert!(
                !functions(root).contains(&function),
                "{name}: `{function}` is written twice"
            );
        }
    }
    assert_eq!(served_models, 2, "gatepass and demo are served");
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

/// The single crate's optional server dependencies take compatible ranges at the tested
/// minimum, as the workspace layout's server crate does (beyond10x/ess#660).
#[test]
fn generated_single_manifest_uses_caret_ranges() {
    let synthesis = crate_layout(&example("gatepass"));
    let manifest = &synthesis.artifacts["Cargo.toml"].contents;
    let mut requirements = manifest
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(" = { version = \"")?;
            let (requirement, rest) = value.split_once('"')?;
            Some((name, requirement, rest.contains("optional = true")))
        })
        .collect::<Vec<_>>();
    requirements.sort_unstable();
    assert_eq!(
        requirements,
        [
            ("clap", "^4.6.7", true),
            ("time", "^0.3.45", true),
            ("uuid", "^1.26.1", true)
        ],
        "{manifest}"
    );
    assert!(
        !manifest.contains("\"="),
        "an exact pin remains:\n{manifest}"
    );
}
