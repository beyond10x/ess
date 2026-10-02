//! Outcomes selected by whether the addressed record exists (ess/16, beyond10x/ess#164, #310).
//!
//! The Rust target generates both forms as behaviour over its storage port: before a branch is
//! taken, the generated seam looks the addressed identity up. A record that carries it selects the
//! `existing_instance:` refusal, or the updating branch beside a creating `unknown_instance:`
//! branch; otherwise the creation runs. The generated server is built, an in-memory storage port is
//! placed beside it (`tests/fixtures/upsert-by-existence-harness/`, holding no behaviour), and the
//! suite the same specification synthesizes runs against it through the generated HTTP surface.
//! The same server with the lookup taken out fails an existence scenario.
//!
//! The Go target generates the same lookup over its own storage port (`story:go-generated-behaviour`),
//! and its server passes the same suite through `tests/fixtures/upsert-by-existence-go-harness/`.
//! Web links the Rust behavior; Clap projects explicit handler obligations.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

/// The model, served by one component.
const COMPONENT: &str = "components:
  - component: items-service
    owns: {domains: [demo.items]}
    accepts: {commands: [demo.items.PutItem, demo.items.BookSlot]}
    publishes: {events: [demo.items.ItemStored, demo.items.SlotBooked]}
    reached_by: network
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("upsert-by-existence.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn served_ir() -> EssIr {
    ir(&format!("{MODEL}{COMPONENT}"))
}

#[test]
fn web_existence_branches_reach_linked_rust_behavior() {
    let ir = served_ir();
    let root = scratch("web");
    for (target, folder) in [(Target::Rust, "rust"), (Target::Web, "web")] {
        let synthesis = synthesize_for(&ir, target).expect("existence branches project");
        write_artifacts(
            &synthesis,
            &root.join("generated").join(folder).join("demo"),
        );
    }
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(harness.join("Cargo.toml"), "[package]\nname = \"web-existence-host\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[lib]\ncrate-type = [\"cdylib\"]\n[workspace]\n[dependencies]\ndemo-web = { path = \"../generated/web/demo/crates/demo-web\" }\ndemo-types = { path = \"../generated/rust/demo/crates/demo-types\" }\ndemo-system = { path = \"../generated/rust/demo/crates/demo-system\" }\nitems-service = { path = \"../generated/rust/demo/crates/items-service\" }\n").unwrap();
    let storage = include_str!("fixtures/upsert-by-existence-harness/main.rs");
    let storage = storage
        .split_once("/// One scenario's rows.")
        .unwrap()
        .1
        .split_once("type System =")
        .unwrap()
        .0;
    let host = format!("use std::{{cell::RefCell, collections::BTreeMap, rc::Rc}};\nuse demo_types::{{behaviour::{{Generated, ItemStorage, SlotStorage}}, items}};\n{storage}\n#[no_mangle]\npub extern \"C\" fn existence_proof() -> u32 {{\nlet system = demo_system::System::new(items_service::ItemsService::new(Generated::new(Ports::default())));\ndemo_web::install(Box::new(system));\n");
    let mut host = host;
    for (command, identity, label, expected) in [
        ("BookSlot", "slot_id", "first", "booked"),
        ("BookSlot", "slot_id", "second", "already-booked"),
        ("PutItem", "item_id", "first", "created"),
        ("PutItem", "item_id", "second", "updated"),
    ] {
        let request = serde_json::json!({"request": "command", "command": format!("demo.items.{command}"), "input": {identity: "chosen", "label": label}}).to_string();
        writeln!(host, "let answer = demo_web::serve({request:?});\nlet parsed = demo_web::json::parse(&answer).unwrap();\nassert_eq!(parsed.member(\"ok\"), Some(&demo_web::json::Value::Bool(true)), \"{{answer}}\");\nassert_eq!(parsed.member(\"outcome\").and_then(|value| value.member(\"outcome\")), Some(&demo_web::json::Value::Text({expected:?}.into())), \"{{answer}}\");").unwrap();
    }
    host.push_str("4\n}\n");
    std::fs::write(harness.join("src/lib.rs"), host).unwrap();
    let target = root.join("target");
    let build = Command::new(std::env::var_os("CARGO").unwrap())
        .args([
            "build",
            "--offline",
            "--quiet",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
        ])
        .arg(&target)
        .current_dir(&harness)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    // The same Node bootstrap as feasibility.rs; every behavior assertion is in the Rust export.
    let output = Command::new("node")
        .args(["-e", "WebAssembly.instantiate(require('node:fs').readFileSync(process.argv[1]),{}).then(({instance})=>{if(instance.exports.existence_proof()!==4)process.exit(1)}).catch(e=>{console.error(e);process.exit(1)})"])
        .arg(target.join("wasm32-unknown-unknown/debug/web_existence_host.wasm"))
        .output().unwrap();
    assert!(output.status.success(), "{output:?}");
}

fn write_artifacts(synthesis: &ess_synth::Synthesis, root: &Path) {
    for (relative, artifact) in &synthesis.artifacts {
        let destination = root.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &artifact.contents).unwrap();
    }
}

#[test]
fn clap_existence_branches_emit_named_handler_obligations() {
    let component = COMPONENT.replace("    reached_by: network\n", "    reached_by: command_line\n    cli:\n      binary: items\n      commands: [demo.items.PutItem, demo.items.BookSlot]\n      views: [demo.items.ItemDetails, demo.items.SlotDetails]\n");
    let ir = ir(&format!("{MODEL}{component}"));
    let synthesis = synthesize_for(&ir, Target::Clap).expect("Clap projects explicit handlers");
    let root = scratch("clap");
    write_artifacts(&synthesis, &root);
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo-cli\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    let manifest = synthesis
        .artifacts
        .keys()
        .find(|path| path.ends_with("Cargo.toml"))
        .unwrap();
    let crate_root = root.join(manifest).parent().unwrap().to_path_buf();
    let target = root.join("target");
    assert!(cargo(&crate_root, &target).status.success());
    for (word, flag, qualified) in [
        ("BookSlot", "--slot_id", "demo.items.BookSlot"),
        ("PutItem", "--item_id", "demo.items.PutItem"),
    ] {
        let output = Command::new(target.join("debug/items"))
            .args([word, flag, "chosen", "--label", "first"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains(qualified) && error.contains("is an obligation nothing has implemented"),
            "{error}"
        );
    }
}

/// Each target's own `workspace` answers as `synthesize_for` does, so a caller that skips it gets
/// the same result.
#[test]
fn every_direct_workspace_entry_answers_as_synthesis_does() {
    let ir = ir(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    if let Err(failure) = ess_synth::rust::workspace(&ir, &plan) {
        panic!("rust carries both forms: {}", failure.to_canonical_json());
    }
    // `story:go-generated-behaviour`: the Go target selects by existence over its storage port too.
    if let Err(failure) = ess_synth::go::workspace(&ir, &plan) {
        panic!("go carries both forms: {}", failure.to_canonical_json());
    }
    synthesize_for(&ir, Target::Go).expect("go synthesizes both forms");
    let emissions = [
        (
            Target::Web,
            ess_synth::web::workspace(&ir, &plan)
                .expect("Web projects")
                .artifacts,
        ),
        (
            Target::Clap,
            ess_synth::clap::workspace(&ir, &plan)
                .expect("Clap projects")
                .artifacts,
        ),
    ];
    let rust = synthesize_for(&ir, Target::Rust).unwrap();
    for (target, artifacts) in emissions {
        let public = synthesize_for(&ir, target).unwrap();
        for artifact in artifacts {
            assert_eq!(public.artifacts[&artifact.path].contents, artifact.contents);
        }
        for path in ["PLAN.md", "plan.json"] {
            assert_eq!(
                public.artifacts[path].contents,
                rust.artifacts[path].contents
            );
        }
    }
}

#[test]
fn an_unknown_instance_refusal_is_not_refused_as_selection_by_existence() {
    // Without either form the model still synthesizes: the refusal is about these branches only.
    let model = MODEL
        .replace("        unknown_instance: true\n", "")
        .replace(
            "      - name: updated\n        updates: demo.items.Item\n        instance: item_id\n        emits: [demo.items.ItemStored]\n        payload:\n          demo.items.ItemStored: {item_id: input.item_id, label: input.label}\n        sets:\n          label: input.label\n        summary: An item with this id exists; its label is replaced.\n",
            "",
        )
        .replace(
            "      - {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}\n",
            "",
        );
    assert_ne!(model, MODEL);
    let ir = ir(&model);
    for target in [Target::Rust, Target::Go] {
        if let Err(failure) = synthesize_for(&ir, target) {
            let text = format!("{failure:?}");
            assert!(
                !text.contains("unknown_instance") && !text.contains("existing_instance"),
                "{target:?}: {text}"
            );
        }
    }
}

#[test]
fn the_rust_plan_generates_both_commands_behaviour() {
    let synthesis = synthesize_for(&served_ir(), Target::Rust).expect("Rust carries both forms");
    for command in ["demo.items.PutItem", "demo.items.BookSlot"] {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, command),
            Some(&SynthesisDisposition::Generated),
            "{command}"
        );
    }
}

// ---- the generated server, built and run against its own suite ---------------------------------

/// The create-or-refuse lookup, as it is emitted.
const REFUSE_LOOKUP: &str = "if SlotStorage::get(&self.ports, &input.slot_id).is_some() {";
/// The same lookup taken out: the creation runs whether or not the slot is booked.
const REFUSE_SKIPPED: &str = "if false {";
/// The create-or-update lookup, as it is emitted.
const UPDATE_LOOKUP: &str = "let Some(held) = ItemStorage::get(&self.ports, &input.item_id) else {";
/// The same lookup taken out: every call creates.
const UPDATE_SKIPPED: &str = "let Some(held) = None::<crate::items::ItemSnapshot> else {";

fn scratch(label: &str) -> PathBuf {
    // One directory per test process: the build rewrites the generated behaviour in place to make
    // each mutant.
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "upsert-by-existence-{label}-{}",
        std::process::id()
    ))
}

/// Cargo inside the harness, with every warning an error.
fn cargo(directory: &Path, target: &Path) -> Output {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(["build", "--offline", "--quiet", "--target-dir"])
        .arg(target)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo runs");
    eprintln!(
        "cargo build in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The harness built against the generated server, and against the same server with each lookup
/// taken out.
struct Built {
    served: PathBuf,
    refuse_skipped: PathBuf,
    update_skipped: PathBuf,
}

fn built() -> &'static Built {
    static BUILT: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let synthesis = synthesize_for(&served_ir(), Target::Rust).expect("the model synthesizes");
        let root = scratch("served");
        let _ = std::fs::remove_dir_all(&root);
        let tree = root.join("demo");
        for (relative, artifact) in &synthesis.artifacts {
            let destination = tree.join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(&destination, &artifact.contents).unwrap();
        }
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).unwrap();
        let dependencies = ["demo-server", "demo-system", "items-service", "demo-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../demo/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"existence-harness\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"harness\"\npath = \
                 \"src/main.rs\"\n\n[dependencies]\n{dependencies}"
            ),
        )
        .unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/upsert-by-existence-harness/main.rs"),
            harness.join("src/main.rs"),
        )
        .unwrap();
        let target = root.join("target");
        let kept = root.join("kept");
        std::fs::create_dir_all(&kept).unwrap();
        let build = |name: &str| {
            assert!(
                cargo(&harness, &target).status.success(),
                "the harness builds"
            );
            std::fs::copy(target.join("debug/harness"), kept.join(name)).unwrap();
            kept.join(name)
        };
        let served = build("served");
        let behaviour = tree.join("crates/demo-types/src/behaviour.rs");
        let source = std::fs::read_to_string(&behaviour).unwrap();
        for lookup in [REFUSE_LOOKUP, UPDATE_LOOKUP] {
            assert_eq!(
                source.matches(lookup).count(),
                1,
                "one `{lookup}`:\n{source}"
            );
        }
        std::fs::write(&behaviour, source.replace(REFUSE_LOOKUP, REFUSE_SKIPPED)).unwrap();
        let refuse_skipped = build("refuse-skipped");
        std::fs::write(&behaviour, source.replace(UPDATE_LOOKUP, UPDATE_SKIPPED)).unwrap();
        let update_skipped = build("update-skipped");
        Built {
            served,
            refuse_skipped,
            update_skipped,
        }
    })
}

/// Every scenario of the suite the model synthesizes, run through one harness binary.
fn run(binary: &Path) -> Vec<(String, Status)> {
    let ir = served_ir();
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).expect("admits");
    let target = Served::start(binary, &ir);
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    for scenario in &report.scenarios {
        if scenario.status != Status::Passed {
            eprintln!(
                "{} via {}: {scenario:#?}",
                scenario.scenario,
                binary.display()
            );
        }
    }
    report
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// The scenarios that witness selection by existence: the existing-instance refusal, and the
/// update sent for an identity the same command created.
const EXISTENCE: [&str; 2] = [
    "demo.items.BookSlot/outcome/already-booked",
    "demo.items.PutItem/outcome/updated",
];

#[test]
fn the_generated_rust_server_passes_every_scenario_of_its_suite() {
    let ran = run(&built().served);
    let ids: Vec<&str> = ran.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        [
            EXISTENCE[0],
            "demo.items.BookSlot/outcome/booked",
            "demo.items.PutItem/outcome/created",
            EXISTENCE[1],
        ],
        "the suite witnesses both forms, each half once"
    );
    let failed: Vec<&(String, Status)> = ran
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:?} of {ids:?}");
}

#[test]
fn a_server_that_skips_the_lookup_fails_an_existence_scenario() {
    for (binary, scenario) in [
        (&built().refuse_skipped, EXISTENCE[0]),
        (&built().update_skipped, EXISTENCE[1]),
    ] {
        let ran = run(binary);
        assert!(
            ran.iter()
                .any(|(id, status)| id == scenario && *status == Status::Failed),
            "{} fails `{scenario}`: {ran:?}",
            binary.display()
        );
    }
}

// ---- the Go server (`story:go-generated-behaviour`) -----------------------------------------------

/// The Go create-or-refuse lookup, as it is emitted.
const GO_REFUSE_LOOKUP: &str = "if _, found := b.ports.SlotStorage.Get(input.SlotId); found {";
/// The same lookup taken out: the creation runs whether or not the slot is booked.
const GO_REFUSE_SKIPPED: &str =
    "if _, found := b.ports.SlotStorage.Get(input.SlotId); found && false {";
/// The Go create-or-update lookup, as it is emitted.
const GO_UPDATE_LOOKUP: &str = "held, found := b.ports.ItemStorage.Get(input.ItemId)";
/// The same lookup taken out: every call creates.
const GO_UPDATE_SKIPPED: &str = "held, found := items.ItemSnapshot{}, false";

/// Runs a Go tool in `directory` with nothing fetched from a network.
fn go_tool(directory: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go runs");
    eprintln!(
        "go {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The Go harness built against the generated Go server, and against the same server with each
/// lookup taken out; `None` where this machine has no Go toolchain.
fn go_built() -> Option<&'static Built> {
    static BUILT: std::sync::OnceLock<Option<Built>> = std::sync::OnceLock::new();
    BUILT
        .get_or_init(|| {
            Command::new("go").arg("version").output().ok()?;
            let synthesis =
                synthesize_for(&served_ir(), Target::Go).expect("the model synthesizes to Go");
            let root = scratch("go");
            let _ = std::fs::remove_dir_all(&root);
            let tree = root.join("demo");
            for (relative, artifact) in &synthesis.artifacts {
                let destination = tree.join(relative);
                std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
                std::fs::write(&destination, &artifact.contents).unwrap();
            }
            assert!(
                go_tool(&tree, &["vet", "./..."]).status.success(),
                "the generated Go is vet clean"
            );
            let harness = root.join("harness");
            std::fs::create_dir_all(&harness).unwrap();
            std::fs::write(
                harness.join("go.mod"),
                "module existenceharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\n\
                 replace example.invalid/demo => ../demo\n",
            )
            .unwrap();
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/upsert-by-existence-go-harness/main.go"),
                harness.join("main.go"),
            )
            .unwrap();
            let kept = root.join("kept");
            std::fs::create_dir_all(&kept).unwrap();
            let build = |name: &str| {
                let binary = kept.join(name);
                assert!(
                    go_tool(
                        &harness,
                        &["build", "-o", binary.to_str().expect("a UTF-8 path"), "."]
                    )
                    .status
                    .success(),
                    "the Go harness builds"
                );
                binary
            };
            let served = build("served");
            let behaviour = tree.join("types/behaviour/behaviour.go");
            let source = std::fs::read_to_string(&behaviour).unwrap();
            for lookup in [GO_REFUSE_LOOKUP, GO_UPDATE_LOOKUP] {
                assert_eq!(
                    source.matches(lookup).count(),
                    1,
                    "one `{lookup}`:\n{source}"
                );
            }
            std::fs::write(
                &behaviour,
                source.replace(GO_REFUSE_LOOKUP, GO_REFUSE_SKIPPED),
            )
            .unwrap();
            let refuse_skipped = build("refuse-skipped");
            std::fs::write(
                &behaviour,
                source.replace(GO_UPDATE_LOOKUP, GO_UPDATE_SKIPPED),
            )
            .unwrap();
            let update_skipped = build("update-skipped");
            Some(Built {
                served,
                refuse_skipped,
                update_skipped,
            })
        })
        .as_ref()
}

#[test]
fn the_generated_go_server_passes_every_scenario_of_its_suite() {
    let Some(built) = go_built() else {
        eprintln!("no Go toolchain on this machine; the Go existence lookup is unchecked here");
        return;
    };
    let ran = run(&built.served);
    let ids: Vec<&str> = ran.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        [
            EXISTENCE[0],
            "demo.items.BookSlot/outcome/booked",
            "demo.items.PutItem/outcome/created",
            EXISTENCE[1],
        ],
        "the suite witnesses both forms, each half once"
    );
    let failed: Vec<&(String, Status)> = ran
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:?} of {ids:?}");
    eprintln!(
        "{} of {} scenarios passed against the generated Go existence lookup",
        ran.len(),
        ids.len()
    );
}

#[test]
fn a_go_server_that_skips_the_lookup_fails_an_existence_scenario() {
    let Some(built) = go_built() else {
        eprintln!("no Go toolchain on this machine; the Go existence lookup is unchecked here");
        return;
    };
    for (binary, scenario) in [
        (&built.refuse_skipped, EXISTENCE[0]),
        (&built.update_skipped, EXISTENCE[1]),
    ] {
        let ran = run(binary);
        assert!(
            ran.iter()
                .any(|(id, status)| id == scenario && *status == Status::Failed),
            "{} fails `{scenario}`: {ran:?}",
            binary.display()
        );
    }
}

// ---- the adapter: the suite's requests, over the harness's line protocol ----------------------

/// One running harness, and the events one scenario published.
struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ess_conformance::ObservedEvent>>,
    sequence: RefCell<u64>,
}

impl Served {
    fn start(binary: &Path, ir: &EssIr) -> Self {
        let mut routes = Vec::new();
        for component in ir.components().values() {
            for route in ess_gen::http::routes(ir, component) {
                let name = match route.serves {
                    ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string(),
                    ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string(),
                };
                routes.extend([name, route.method.as_str().to_owned(), route.path]);
            }
        }
        let mut child = Command::new(binary)
            .args(routes)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the harness starts");
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped"));
        Self {
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            published: RefCell::default(),
            sequence: RefCell::new(0),
        }
    }

    fn ask(&self, request: &serde_json::Value) -> serde_json::Value {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"))
    }

    fn tick(&self) -> u64 {
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        *sequence
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

/// A suite value as JSON, a number in its exact spelling.
fn json_of(node: &Node) -> serde_json::Value {
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(number) => {
            serde_json::from_str(&number.to_string()).expect("a number's spelling is JSON")
        }
        Node::Text(text) => serde_json::Value::String(text.clone()),
        Node::Seq(items) => serde_json::Value::Array(items.iter().map(json_of).collect()),
        Node::Map(members) => serde_json::Value::Object(
            members
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        ),
    }
}

fn node_of(value: &serde_json::Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

fn failure(observation: &str, answer: &serde_json::Value) -> ess_conformance::TargetError {
    ess_conformance::TargetError::unavailable(observation, answer.to_string())
}

impl ess_conformance::ConformanceTarget for Served {
    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        Ok(ess_conformance::ImplementationIdentity::new(
            "upsert-by-existence-generated",
            "1",
        ))
    }

    fn begin_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        let answer = self.ask(&serde_json::json!({"op": "reset"}));
        if answer["ok"] != serde_json::json!(true) {
            return Err(failure("opening a scenario", &answer));
        }
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn end_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }

    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
        let observation = format!("invoking `{}`", request.command);
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        );
        let answer = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string),
            "body": body.to_string(),
        }));
        let status = answer["status"]
            .as_u64()
            .ok_or_else(|| failure(&observation, &answer))?;
        let served = &answer["answer"];
        if status == 403 && served["refused"] == "not granted" {
            return Err(ess_conformance::TargetError::not_granted(
                served["actor"].as_str(),
            ));
        }
        if status == 501 {
            return Ok(ess_conformance::SemanticCommandResult::undeclared());
        }
        let outcome = served["outcome"]
            .as_str()
            .ok_or_else(|| failure(&observation, &answer))?;
        let outcome = OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            outcome
                .parse()
                .map_err(|_| failure(&observation, &answer))?,
        );
        let error = match served.get("error") {
            Some(error) => {
                let name: ErrorRef = error
                    .as_str()
                    .and_then(|name| name.parse().ok())
                    .ok_or_else(|| failure(&observation, &answer))?;
                let mut declared = ess_conformance::DeclaredErrorValue::new(name);
                if let Some(serde_json::Value::Object(payload)) = served.get("payload") {
                    for (field, value) in payload {
                        if !value.is_null() {
                            declared = declared.with(field.clone(), node_of(value));
                        }
                    }
                }
                Some(declared)
            }
            None => None,
        };
        let mut direct_events = Vec::new();
        for published in served["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(|| failure(&observation, &answer))?;
            let mut event = ess_conformance::ObservedEvent::new(reference);
            if let Some(serde_json::Value::Object(payload)) = published.get("payload") {
                for (field, value) in payload {
                    event = event.with(field.clone(), node_of(value));
                }
            }
            let event = event
                .in_activity(request.correlation.clone())
                .at(self.tick());
            self.published.borrow_mut().push(event.clone());
            direct_events.push(event);
        }
        let consistency = Some(
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.tick()))
                .map_err(|why| {
                    ess_conformance::TargetError::unavailable(observation.clone(), why.to_string())
                })?,
        );
        Ok(ess_conformance::SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency,
            direct_events,
            response: None,
        })
    }

    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let observation = format!("reading `{}`", request.view);
        let answer = self.ask(&serde_json::json!({"op": "view", "view": request.view.to_string()}));
        let Some(rows) = answer["answer"]["rows"].as_array() else {
            return Err(failure(&observation, &answer));
        };
        Ok(ess_conformance::SemanticViewResult::of(rows.iter().map(
            |row| {
                row.as_object()
                    .into_iter()
                    .flatten()
                    .map(|(field, value)| (field.clone(), node_of(value)))
                    .collect::<ess_conformance::target::ViewRow>()
            },
        )))
    }

    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }

    fn configure_external_outcome(
        &self,
        _: ess_conformance::ExternalOutcomeControl,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "forcing an outcome",
            "none is external",
        ))
    }

    fn redeliver_event(
        &self,
        _: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "redelivering",
            "the model declares no binding",
        ))
    }
}
